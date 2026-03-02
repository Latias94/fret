
#[derive(Default)]
pub struct UiDiagnosticsService {
    cfg: UiDiagnosticsConfig,
    per_window: HashMap<AppWindowId, WindowRing>,
    text_font_stack_key_stability: HashMap<AppWindowId, TextFontStackKeyStability>,
    known_windows: Vec<AppWindowId>,
    last_trigger_stamp: Option<u64>,
    last_script_trigger_stamp: Option<u64>,
    last_pick_trigger_mtime: Option<std::time::SystemTime>,
    last_inspect_trigger_mtime: Option<std::time::SystemTime>,
    exit_armed: bool,
    exit_last_mtime: Option<std::time::SystemTime>,
    ws_exit_deadline_unix_ms: Option<u64>,
    ready_written: bool,
    ready_write_warned: bool,
    capabilities_written: bool,
    capabilities_write_warned: bool,
    inspector: inspect_controller::InspectController,
    pending_script: Option<PendingScript>,
    pending_script_run_id: Option<u64>,
    active_scripts: HashMap<AppWindowId, ActiveScript>,
    pending_force_dump: Option<fs_triggers::PendingForceDumpRequest>,
    last_dump_dir: Option<PathBuf>,
    last_dump_artifact_stats: Option<UiArtifactStatsV1>,
    last_script_run_id: u64,
    clipboard_text_responses: std::collections::VecDeque<DiagClipboardTextResponse>,
    next_clipboard_token: u64,
    app_snapshot_provider:
        Option<Arc<dyn Fn(&App, AppWindowId) -> Option<serde_json::Value> + 'static>>,
    #[cfg(feature = "diagnostics-ws")]
    pending_devtools_screenshot:
        Option<ui_diagnostics_devtools_ws::PendingDevtoolsScreenshotRequest>,
    #[cfg(feature = "diagnostics-ws")]
    pending_devtools_semantics_node_get:
        Option<ui_diagnostics_devtools_ws::PendingDevtoolsSemanticsNodeGetRequest>,
    #[cfg(feature = "diagnostics-ws")]
    ws_bridge: UiDiagnosticsWsBridge,
}

#[derive(Debug, Clone)]
pub(super) struct DiagClipboardTextResponse {
    pub(super) token: fret_core::ClipboardToken,
    pub(super) kind: DiagClipboardTextResponseKind,
}

#[derive(Debug, Clone)]
pub(super) enum DiagClipboardTextResponseKind {
    Text(String),
    Unavailable { message: Option<String> },
}

#[derive(Debug, Default, Clone, Copy)]
struct TextFontStackKeyStability {
    last_key: Option<u64>,
    stable_frames: u32,
}

#[derive(Debug, Clone)]
struct CachedTestIdPredicateEval {
    used_cache: bool,
    ok: Option<bool>,
    stale: bool,
    test_id: Option<String>,
    age_ms: Option<u64>,
    window_snapshot_seq: Option<u64>,
    max_age_ms: Option<u64>,
}

thread_local! {
    static SCRIPT_INJECTION_SCOPE: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

impl UiDiagnosticsService {
    pub(super) fn allocate_clipboard_token(&mut self) -> fret_core::ClipboardToken {
        let next = self.next_clipboard_token.max(1);
        self.next_clipboard_token = next.saturating_add(1);
        fret_core::ClipboardToken(next)
    }

    pub(super) fn clipboard_text_response_for_token(
        &self,
        token: fret_core::ClipboardToken,
    ) -> Option<&DiagClipboardTextResponseKind> {
        self.clipboard_text_responses
            .iter()
            .rev()
            .find(|r| r.token == token)
            .map(|r| &r.kind)
    }

    pub(super) fn reset_clipboard_text_responses(&mut self) {
        self.clipboard_text_responses.clear();
        self.next_clipboard_token = 1;
    }

    fn record_clipboard_text_response(&mut self, response: DiagClipboardTextResponse) {
        // Keep this bounded; clipboard assertions are a diagnostics-only surface and should not
        // grow unbounded during long-running apps.
        const MAX_RESPONSES: usize = 128;
        while self.clipboard_text_responses.len() >= MAX_RESPONSES {
            self.clipboard_text_responses.pop_front();
        }
        self.clipboard_text_responses.push_back(response);
    }

    pub fn with_script_injection_scope<R>(f: impl FnOnce() -> R) -> R {
        SCRIPT_INJECTION_SCOPE.with(|cell| {
            let prev = cell.replace(true);
            let out = f();
            cell.set(prev);
            out
        })
    }

    fn in_script_injection_scope() -> bool {
        SCRIPT_INJECTION_SCOPE.with(|cell| cell.get())
    }

    fn any_script_running(&self) -> bool {
        self.pending_script.is_some() || !self.active_scripts.is_empty()
    }

    pub fn should_ignore_external_pointer_event(&self, event: &Event) -> bool {
        if !self.is_enabled() {
            return false;
        }
        if !self.cfg.isolate_external_pointer_input_while_script_running {
            return false;
        }
        if Self::in_script_injection_scope() {
            return false;
        }
        if !self.any_script_running() {
            return false;
        }

        // Do not block `InternalDrag` while a script is running: multi-window docking relies on
        // runner-routed internal drag hover/drop events, and scripted playback uses cursor/mouse
        // overrides to drive those paths deterministically.
        matches!(event, Event::Pointer(_))
    }

    pub fn should_ignore_external_keyboard_event(&self, event: &Event) -> bool {
        if !self.is_enabled() {
            return false;
        }
        if !self.cfg.isolate_external_keyboard_input_while_script_running {
            return false;
        }
        if Self::in_script_injection_scope() {
            return false;
        }
        if !self.any_script_running() {
            return false;
        }

        matches!(
            event,
            Event::KeyDown { .. } | Event::KeyUp { .. } | Event::TextInput(_) | Event::Ime(_)
        )
    }

    fn is_wasm_ws_only(&self) -> bool {
        cfg!(target_arch = "wasm32") && self.ws_is_configured()
    }

    pub fn known_windows(&self) -> &[AppWindowId] {
        &self.known_windows
    }

    fn poll_ws_inbox_and_is_wasm_ws_only(&mut self) -> bool {
        self.poll_ws_inbox();
        self.is_wasm_ws_only()
    }

    fn note_window_seen(&mut self, window: AppWindowId) {
        if self.known_windows.contains(&window) {
            return;
        }
        self.known_windows.push(window);
    }

    fn resolve_window_target(
        &self,
        current_window: AppWindowId,
        target: Option<&UiWindowTargetV1>,
    ) -> Option<AppWindowId> {
        // `known_windows` is insertion-ordered by first observation. Treat `first_seen` /
        // `last_seen` as "seen order", not numeric window ids (which are not guaranteed to be
        // monotonic across backends).
        let first_seen = self.known_windows.first().copied();
        let last_seen = self.known_windows.last().copied();
        match target.copied().unwrap_or(UiWindowTargetV1::Current) {
            UiWindowTargetV1::Current => Some(current_window),
            UiWindowTargetV1::FirstSeen => first_seen,
            UiWindowTargetV1::FirstSeenOther => self
                .known_windows
                .iter()
                .copied()
                .filter(|w| *w != current_window)
                .next(),
            UiWindowTargetV1::LastSeen => last_seen,
            UiWindowTargetV1::LastSeenOther => self
                .known_windows
                .iter()
                .copied()
                .filter(|w| *w != current_window)
                .last(),
            UiWindowTargetV1::WindowFfi { window } => {
                let want = AppWindowId::from(KeyData::from_ffi(window));
                self.known_windows.contains(&want).then_some(want)
            }
        }
    }

    fn resolve_window_target_for_active_step(
        &self,
        current_window: AppWindowId,
        anchor_window: AppWindowId,
        target: Option<&UiWindowTargetV1>,
    ) -> Option<AppWindowId> {
        let Some(target) = target else {
            return Some(current_window);
        };

        match target {
            UiWindowTargetV1::Current => Some(current_window),
            UiWindowTargetV1::FirstSeen => Some(anchor_window),
            _ => self.resolve_window_target(anchor_window, Some(target)),
        }
    }

    fn cached_test_id_exists(&self, window: AppWindowId, test_id: &str) -> Option<bool> {
        let ring = self.per_window.get(&window)?;
        Some(ring.test_id_bounds.contains_key(test_id))
    }

    fn resolve_window_target_for_active_step_with_test_id_hint(
        &self,
        current_window: AppWindowId,
        anchor_window: AppWindowId,
        target: Option<&UiWindowTargetV1>,
        test_id_hint: Option<&str>,
    ) -> Option<AppWindowId> {
        let resolved = self.resolve_window_target_for_active_step(current_window, anchor_window, target)?;
        let Some(test_id_hint) = test_id_hint else {
            return Some(resolved);
        };
        let Some(target) = target else {
            return Some(resolved);
        };

        // Do not override fully explicit targets.
        match target {
            UiWindowTargetV1::Current | UiWindowTargetV1::FirstSeen | UiWindowTargetV1::WindowFfi { .. } => {
                return Some(resolved);
            }
            UiWindowTargetV1::FirstSeenOther | UiWindowTargetV1::LastSeenOther | UiWindowTargetV1::LastSeen => {}
        }

        let candidates: Vec<AppWindowId> = match target {
            UiWindowTargetV1::FirstSeenOther | UiWindowTargetV1::LastSeenOther => self
                .known_windows
                .iter()
                .copied()
                .filter(|w| *w != anchor_window)
                .collect(),
            UiWindowTargetV1::LastSeen => self.known_windows.clone(),
            _ => Vec::new(),
        };

        if candidates.len() <= 1 {
            return Some(resolved);
        }

        let mut matches: Option<AppWindowId> = None;
        for window in candidates {
            if self.cached_test_id_exists(window, test_id_hint) != Some(true) {
                continue;
            }
            if matches.is_some() {
                // Ambiguous: multiple windows claim the selector.
                return Some(resolved);
            }
            matches = Some(window);
        }

        Some(matches.unwrap_or(resolved))
    }

    fn step_test_id_hint(step: &UiActionStepV2) -> Option<&str> {
        let target = match step {
            UiActionStepV2::Click { target, .. }
            | UiActionStepV2::Tap { target, .. }
            | UiActionStepV2::LongPress { target, .. }
            | UiActionStepV2::Swipe { target, .. }
            | UiActionStepV2::Pinch { target, .. }
            | UiActionStepV2::SetBaseRef { target, .. }
            | UiActionStepV2::MovePointer { target, .. }
            | UiActionStepV2::MovePointerSweep { target, .. }
            | UiActionStepV2::PointerDown { target, .. }
            | UiActionStepV2::DragPointer { target, .. }
            | UiActionStepV2::DragPointerUntil { target, .. }
            | UiActionStepV2::Wheel { target, .. }
            | UiActionStepV2::ClickStable { target, .. }
            | UiActionStepV2::ClickSelectableTextSpanStable { target, .. }
            | UiActionStepV2::WaitBoundsStable { target, .. }
            | UiActionStepV2::EnsureVisible { target, .. }
            | UiActionStepV2::ScrollIntoView { target, .. }
            | UiActionStepV2::TypeTextInto { target, .. }
            | UiActionStepV2::SetSliderValue { target, .. } => Some(target),
            _ => None,
        }?;

        match target {
            UiSelectorV1::TestId { id, .. } => Some(id.as_str()),
            _ => None,
        }
    }

    fn predicate_can_eval_off_window(predicate: &UiPredicateV1) -> bool {
        matches!(
            predicate,
            UiPredicateV1::KnownWindowCountGe { .. }
                | UiPredicateV1::KnownWindowCountIs { .. }
                | UiPredicateV1::PlatformUiWindowHoverDetectionIs { .. }
                | UiPredicateV1::DockDragCurrentWindowIs { .. }
                | UiPredicateV1::DockDragMovingWindowIs { .. }
                | UiPredicateV1::DockDragWindowUnderMovingWindowIs { .. }
                | UiPredicateV1::DockDragActiveIs { .. }
                | UiPredicateV1::DockDragTransparentPayloadAppliedIs { .. }
                | UiPredicateV1::DockDragTransparentPayloadMousePassthroughAppliedIs { .. }
                | UiPredicateV1::DockDragWindowUnderCursorSourceIs { .. }
                | UiPredicateV1::DockDragWindowUnderMovingWindowSourceIs { .. }
                | UiPredicateV1::DockFloatingDragActiveIs { .. }
                | UiPredicateV1::DockDropPreviewKindIs { .. }
                | UiPredicateV1::DockDropResolveSourceIs { .. }
                | UiPredicateV1::DockDropResolvedIsSome { .. }
                | UiPredicateV1::DockDropResolvedZoneIs { .. }
                | UiPredicateV1::DockDropResolvedInsertIndexIs { .. }
                | UiPredicateV1::DockGraphCanonicalIs { .. }
                | UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { .. }
                | UiPredicateV1::DockGraphNodeCountLe { .. }
                | UiPredicateV1::DockGraphMaxSplitDepthLe { .. }
                | UiPredicateV1::DockGraphSignatureIs { .. }
                | UiPredicateV1::DockGraphSignatureContains { .. }
                | UiPredicateV1::DockGraphSignatureFingerprint64Is { .. }
        )
    }

    fn predicate_can_eval_from_cached_test_id_bounds(predicate: &UiPredicateV1) -> bool {
        matches!(
            predicate,
            UiPredicateV1::Exists {
                target: UiSelectorV1::TestId { .. }
            } | UiPredicateV1::NotExists {
                target: UiSelectorV1::TestId { .. }
            }
        )
    }

    pub(super) fn open_window_count_for_predicates(app: &App) -> u32 {
        let from_runner = app
            .global::<fret_runtime::RunnerWindowLifecycleDiagnosticsStore>()
            .map(|store| store.snapshot().open_window_count);
        let from_input_ctx = app
            .global::<fret_runtime::WindowInputContextService>()
            .map(|ctx_svc| ctx_svc.window_count() as u32);

        from_runner
            .or(from_input_ctx)
            .unwrap_or(0)
            .max(1)
    }

    const CACHED_TEST_ID_PREDICATE_MAX_AGE_MS: u64 = 30_000;

    fn eval_predicate_from_cached_test_id_bounds(
        &self,
        window: AppWindowId,
        predicate: &UiPredicateV1,
    ) -> CachedTestIdPredicateEval {
        let test_id = match predicate {
            UiPredicateV1::Exists {
                target: UiSelectorV1::TestId { id, .. },
            }
            | UiPredicateV1::NotExists {
                target: UiSelectorV1::TestId { id, .. },
            } => Some(id.clone()),
            _ => None,
        };
        let Some(test_id) = test_id else {
            return CachedTestIdPredicateEval {
                used_cache: false,
                ok: None,
                stale: false,
                test_id: None,
                age_ms: None,
                window_snapshot_seq: None,
                max_age_ms: None,
            };
        };

        let Some(ring) = self.per_window.get(&window) else {
            return CachedTestIdPredicateEval {
                used_cache: false,
                ok: None,
                stale: false,
                test_id: Some(test_id),
                age_ms: None,
                window_snapshot_seq: None,
                max_age_ms: None,
            };
        };
        let Some(snapshot) = ring.snapshots.back() else {
            return CachedTestIdPredicateEval {
                used_cache: false,
                ok: None,
                stale: false,
                test_id: Some(test_id),
                age_ms: None,
                window_snapshot_seq: None,
                max_age_ms: None,
            };
        };

        let age_ms = unix_ms_now().saturating_sub(snapshot.timestamp_unix_ms);
        if age_ms > Self::CACHED_TEST_ID_PREDICATE_MAX_AGE_MS {
            return CachedTestIdPredicateEval {
                used_cache: true,
                ok: None,
                stale: true,
                test_id: Some(test_id),
                age_ms: Some(age_ms),
                window_snapshot_seq: Some(snapshot.window_snapshot_seq),
                max_age_ms: Some(Self::CACHED_TEST_ID_PREDICATE_MAX_AGE_MS),
            };
        }

        let ok = match predicate {
            UiPredicateV1::Exists { .. } => ring.test_id_bounds.contains_key(&test_id),
            UiPredicateV1::NotExists { .. } => !ring.test_id_bounds.contains_key(&test_id),
            _ => unreachable!("predicate already checked for test_id selector"),
        };
        CachedTestIdPredicateEval {
            used_cache: true,
            ok: Some(ok),
            stale: false,
            test_id: Some(test_id),
            age_ms: Some(age_ms),
            window_snapshot_seq: Some(snapshot.window_snapshot_seq),
            max_age_ms: Some(Self::CACHED_TEST_ID_PREDICATE_MAX_AGE_MS),
        }
    }

    fn preferred_window_for_active_script(active: &ActiveScript) -> Option<AppWindowId> {
        if let Some(step) = active.steps.get(active.next_step) {
            match step {
                UiActionStepV2::WaitUntil { predicate, .. }
                | UiActionStepV2::Assert { predicate, .. }
                    if Self::predicate_can_eval_off_window(predicate) =>
                {
                    // Avoid pinning scripts to a specific window during "read-only" docking
                    // assertions / waits. Overlap + occlusion can prevent the target window from
                    // producing frames, so allowing migration keeps timeouts and gates progressing.
                    return None;
                }
                _ => {}
            }

            // Avoid migrating a newly started script before any per-window state is established.
            // The first few steps typically establish window geometry and must run consistently.
            if active.next_step == 0 {
                return Some(active.anchor_window);
            }

            // `wait_frames` is often used as a short "yield" between runner-level window ops
            // (raise/move) and subsequent assertions. During cross-window drags, the drag source
            // window can become fully occluded and starved of redraw callbacks; honoring an
            // explicit `wait_frames.window` target keeps scripts deterministic without requiring
            // the occluded window to tick.
            match step {
                UiActionStepV2::WaitFrames {
                    window: Some(UiWindowTargetV1::FirstSeen),
                    ..
                } => return Some(active.anchor_window),
                UiActionStepV2::WaitFrames {
                    window: Some(UiWindowTargetV1::WindowFfi { window }),
                    ..
                } => return Some(AppWindowId::from(KeyData::from_ffi(*window))),
                UiActionStepV2::WaitFrames {
                    window: Some(UiWindowTargetV1::Current),
                    ..
                } => return None,
                _ => {}
            }

            // Prefer preserving captured-pointer continuity over window-target pinning. During
            // cross-window drags, the runner may temporarily starve the "under" window of redraw
            // callbacks; pinning to `first_seen` in that state can stall scripts indefinitely.
            if let Some(session) = active.pointer_session.as_ref() {
                return Some(session.window);
            }
            if let Some(state) = active.v2_step_state.as_ref() {
                match state {
                    V2StepState::DragPointer(state) => return Some(state.window),
                    V2StepState::DragPointerUntil(state) => return Some(state.playback.window),
                    V2StepState::DragTo(state) => {
                        if let Some(playback) = state.playback.as_ref() {
                            return Some(playback.window);
                        }
                    }
                    _ => {}
                }
            }

            // Before a step caches any per-window state (pointer session / v2 step state), we may
            // still need to "pin" execution to a specific window to avoid migration loops.
            //
            // Example: a window-targeted drag step (`drag_pointer_until`) can be repeatedly stolen
            // by any window that happens to be producing frames. If the step keeps handing off to
            // its intended window without ever initializing playback, timeouts may never decrement
            // and tooling can hang waiting for `script.result.json` to complete.
            //
            // Prefer a stable window when the step targets:
            // - `first_seen` (use the script's `anchor_window`)
            // - `window_ffi` (resolve directly)
            //
            // Leave other relative targets (last_seen/other) migratable until per-window state is
            // established; those depend on `known_windows`, which is maintained at runtime.
            let step_window_target: Option<&UiWindowTargetV1> = match step {
                UiActionStepV2::Click { window, .. }
                | UiActionStepV2::MovePointer { window, .. }
                | UiActionStepV2::MovePointerSweep { window, .. }
                | UiActionStepV2::PointerDown { window, .. }
                | UiActionStepV2::PointerMove { window, .. }
                | UiActionStepV2::PointerUp { window, .. }
                | UiActionStepV2::DragPointer { window, .. }
                | UiActionStepV2::DragPointerUntil { window, .. }
                | UiActionStepV2::DragTo { window, .. }
                | UiActionStepV2::Wheel { window, .. }
                | UiActionStepV2::ClickStable { window, .. }
                | UiActionStepV2::ClickSelectableTextSpanStable { window, .. }
                | UiActionStepV2::WaitBoundsStable { window, .. }
                | UiActionStepV2::EnsureVisible { window, .. }
                | UiActionStepV2::ScrollIntoView { window, .. }
                | UiActionStepV2::TypeTextInto { window, .. }
                | UiActionStepV2::MenuSelect { window, .. }
                | UiActionStepV2::MenuSelectPath { window, .. }
                | UiActionStepV2::SetSliderValue { window, .. }
                | UiActionStepV2::SetWindowInnerSize { window, .. }
                | UiActionStepV2::SetWindowOuterPosition { window, .. }
                | UiActionStepV2::SetCursorInWindow { window, .. }
                | UiActionStepV2::SetCursorInWindowLogical { window, .. }
                | UiActionStepV2::SetMouseButtons { window, .. }
                | UiActionStepV2::RaiseWindow { window, .. }
                | UiActionStepV2::WaitFrames { window, .. }
                | UiActionStepV2::WaitUntil { window, .. }
                | UiActionStepV2::Assert { window, .. } => window.as_ref(),
                _ => None,
            };
            match step_window_target.copied() {
                Some(UiWindowTargetV1::FirstSeen) => return Some(active.anchor_window),
                Some(UiWindowTargetV1::WindowFfi { window }) => {
                    return Some(AppWindowId::from(KeyData::from_ffi(window)));
                }
                _ => {}
            }
        }

        None
    }

    fn active_step_window_target(active: &ActiveScript) -> Option<UiWindowTargetV1> {
        let step = active.steps.get(active.next_step)?;
        let step_window_target: Option<&UiWindowTargetV1> = match step {
            UiActionStepV2::Click { window, .. }
            | UiActionStepV2::MovePointer { window, .. }
            | UiActionStepV2::MovePointerSweep { window, .. }
            | UiActionStepV2::PointerDown { window, .. }
            | UiActionStepV2::PointerMove { window, .. }
            | UiActionStepV2::PointerUp { window, .. }
            | UiActionStepV2::DragPointer { window, .. }
            | UiActionStepV2::DragPointerUntil { window, .. }
            | UiActionStepV2::DragTo { window, .. }
            | UiActionStepV2::Wheel { window, .. }
            | UiActionStepV2::ClickStable { window, .. }
            | UiActionStepV2::ClickSelectableTextSpanStable { window, .. }
            | UiActionStepV2::WaitBoundsStable { window, .. }
            | UiActionStepV2::EnsureVisible { window, .. }
            | UiActionStepV2::ScrollIntoView { window, .. }
            | UiActionStepV2::TypeTextInto { window, .. }
            | UiActionStepV2::MenuSelect { window, .. }
            | UiActionStepV2::MenuSelectPath { window, .. }
            | UiActionStepV2::SetSliderValue { window, .. }
            | UiActionStepV2::SetWindowInnerSize { window, .. }
            | UiActionStepV2::SetWindowOuterPosition { window, .. }
            | UiActionStepV2::SetCursorInWindow { window, .. }
            | UiActionStepV2::SetCursorInWindowLogical { window, .. }
            | UiActionStepV2::SetMouseButtons { window, .. }
            | UiActionStepV2::RaiseWindow { window, .. }
            | UiActionStepV2::WaitFrames { window, .. }
            | UiActionStepV2::WaitUntil { window, .. }
            | UiActionStepV2::Assert { window, .. } => window.as_ref(),
            _ => None,
        };
        step_window_target.copied()
    }

    fn remap_script_per_window_state_for_migration(
        active: &mut ActiveScript,
        new_window: AppWindowId,
        allow_remap_captured_drag: bool,
    ) {
        if let Some(session) = active.pointer_session.as_mut() {
            session.window = new_window;
        }
        if let Some(state) = active.v2_step_state.as_mut() {
            match state {
                V2StepState::DragPointer(state) => state.window = new_window,
                V2StepState::DragPointerUntil(state) => {
                    // Avoid splitting a captured-pointer gesture across windows. `drag_pointer_until`
                    // is allowed to "hold" the drag across frames; once we've emitted a down/move
                    // segment, keep injecting into the original playback window unless the runner
                    // has migrated the captured drag to a different window (ImGui-style tear-off).
                    if (!state.down_issued && state.playback.frame == 0)
                        || allow_remap_captured_drag
                    {
                        state.playback.window = new_window;
                    }
                }
                V2StepState::DragTo(state) => {
                    if let Some(playback) = state.playback.as_mut() {
                        playback.window = new_window;
                    }
                }
                _ => {}
            }
        }
    }

    fn can_migrate_for_current_target(active: &ActiveScript) -> bool {
        if !matches!(
            Self::active_step_window_target(active),
            Some(UiWindowTargetV1::Current)
        ) {
            return false;
        }

        // Avoid splitting a captured-pointer gesture across windows. After a drag step has issued
        // a pointer down, migrating execution to a different window would cause the corresponding
        // pointer up to land in the wrong window and leave the original runtime drag state stuck.
        match active.v2_step_state.as_ref() {
            None => true,
            Some(V2StepState::DragPointerUntil(state))
                if state.step_index == active.next_step
                    && !state.down_issued
                    && state.playback.frame == 0 =>
            {
                true
            }
            Some(V2StepState::DragPointer(state))
                if state.step_index == active.next_step && state.frame == 0 =>
            {
                true
            }
            Some(V2StepState::DragTo(state))
                if state.step_index == active.next_step && state.playback.is_none() =>
            {
                true
            }
            _ => false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.cfg.enabled
    }

    fn update_text_font_stack_key_stability(&mut self, app: &App, window: AppWindowId) -> u32 {
        let key = app.global::<fret_runtime::TextFontStackKey>().map(|k| k.0);
        let state = self
            .text_font_stack_key_stability
            .entry(window)
            .or_default();

        match (key, state.last_key) {
            (Some(key), Some(prev)) if key == prev => {
                state.stable_frames = state.stable_frames.saturating_add(1);
            }
            (Some(key), _) => {
                state.last_key = Some(key);
                state.stable_frames = 0;
            }
            (None, _) => {
                state.last_key = None;
                state.stable_frames = 0;
            }
        }

        state.stable_frames
    }

    /// Returns the index of the next script step to execute for `window`, if a script is active.
    ///
    /// This is intended for diag-only app logic that wants to run after a particular scripted
    /// step has completed (e.g. "after the baseline screenshot").
    pub fn active_script_next_step_index(&self, window: AppWindowId) -> Option<u32> {
        self.active_scripts
            .get(&window)
            .map(|active| active.next_step.min(u32::MAX as usize) as u32)
    }

    pub fn set_app_snapshot_provider(
        &mut self,
        provider: Option<Arc<dyn Fn(&App, AppWindowId) -> Option<serde_json::Value> + 'static>>,
    ) {
        self.app_snapshot_provider = provider;
    }

    /// Returns `true` if the current diagnostics state would benefit from (or requires) a fresh
    /// semantics snapshot for `window` on this frame.
    ///
    /// This is a performance knob: semantics snapshots are expensive, and many scripted steps only
    /// need semantics for *initial target resolution*. Once a step has cached its target geometry
    /// (via `v2_step_state`), we can often skip requesting semantics until a selector-based step is
    /// about to run again.
    pub fn wants_semantics_snapshot(&mut self, window: AppWindowId) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.note_window_seen(window);

        self.poll_pick_trigger();
        self.poll_inspect_trigger();
        self.poll_script_trigger();

        if self.cfg.capture_semantics {
            return true;
        }

        if self.inspector.pick_armed_run_id.is_some()
            || self
                .inspector
                .pending_pick
                .as_ref()
                .is_some_and(|p| p.window == window)
            || self.inspector.enabled
            || self.inspector.state.locked_windows.contains(&window)
            || self.inspector.state.toast.contains_key(&window)
        {
            return true;
        }

        if self.pending_script.is_some() {
            return true;
        }

        self.active_scripts
            .get(&window)
            .is_some_and(script_engine::active_script_needs_semantics_snapshot)
    }

    pub fn redact_text(&self) -> bool {
        self.cfg.redact_text
    }

    pub fn last_pointer_position(&self, window: AppWindowId) -> Option<Point> {
        self.per_window
            .get(&window)
            .and_then(|ring| ring.last_pointer_position)
    }

    pub fn last_picked_node_id(&self, window: AppWindowId) -> Option<u64> {
        self.inspector.last_picked_node_id.get(&window).copied()
    }

    pub fn pick_is_armed(&self) -> bool {
        self.inspector.pick_armed_run_id.is_some()
    }

    pub(super) fn pick_is_pending(&self, window: AppWindowId) -> bool {
        self.inspector
            .pending_pick
            .as_ref()
            .is_some_and(|pending| pending.window == window)
    }

    pub fn clear_window(&mut self, window: AppWindowId) {
        self.per_window.remove(&window);
        self.known_windows.retain(|w| *w != window);
        self.active_scripts.remove(&window);
        self.inspector.clear_for_window(window);
    }

    fn reset_diagnostics_ring_for_window(&mut self, window: AppWindowId) {
        self.per_window.entry(window).or_default().clear();
    }

    pub fn record_model_changes(&mut self, window: AppWindowId, changed: &[ModelId]) {
        if !self.is_enabled() {
            return;
        }
        let ring = self.per_window.entry(window).or_default();
        ring.last_changed_models = changed.iter().map(|id| id.data().as_ffi()).collect();
    }

    pub fn record_global_changes(
        &mut self,
        app: &App,
        window: AppWindowId,
        changed: &[std::any::TypeId],
    ) {
        if !self.is_enabled() {
            return;
        }
        let ring = self.per_window.entry(window).or_default();
        ring.last_changed_globals = changed
            .iter()
            .map(|&t| {
                app.global_type_name(t)
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| format!("{t:?}"))
            })
            .collect();
    }

    pub fn record_event(&mut self, app: &App, window: AppWindowId, event: &Event) {
        if !self.is_enabled() {
            return;
        }

        self.note_window_seen(window);

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        let ring = self.per_window.entry(window).or_default();
        ring.update_pointer_position(event);

        let mut recorded = RecordedUiEventV1::from_event(app, window, event, self.cfg.redact_text);
        truncate_string_bytes(&mut recorded.debug, self.cfg.max_debug_string_bytes);
        ring.push_event(&self.cfg, recorded);

        match event {
            Event::ClipboardText { token, text } => {
                self.record_clipboard_text_response(DiagClipboardTextResponse {
                    token: *token,
                    kind: DiagClipboardTextResponseKind::Text(text.clone()),
                });
            }
            Event::ClipboardTextUnavailable { token, message } => {
                self.record_clipboard_text_response(DiagClipboardTextResponse {
                    token: *token,
                    kind: DiagClipboardTextResponseKind::Unavailable {
                        message: message.clone(),
                    },
                });
            }
            _ => {}
        }

        if let Some(active) = self.active_scripts.get_mut(&window)
            && let Event::Ime(ime) = event
        {
            let step_index = active
                .last_injected_step
                .unwrap_or_else(|| active.next_step.min(u32::MAX as usize) as u32);
            record_ime_event_trace(&mut active.ime_event_trace, step_index, "record_event", ime);
        }
    }

    pub fn record_viewport_input(&mut self, event: fret_core::ViewportInputEvent) {
        if !self.is_enabled() {
            return;
        }

        let ring = self.per_window.entry(event.window).or_default();
        if ring.viewport_input_this_frame.len() >= self.cfg.max_events {
            return;
        }
        ring.viewport_input_this_frame
            .push(UiViewportInputEventV1::from_event(event));
    }

    pub fn record_snapshot(
        &mut self,
        app: &App,
        window: AppWindowId,
        bounds: Rect,
        scale_factor: f32,
        ui: &mut UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
        scene: &Scene,
    ) {
        if !self.is_enabled() {
            return;
        }

        // Keep `known_windows` aligned to currently-open windows so window targets like
        // `last_seen` do not get stuck pointing at a window that has already been closed (common
        // after tear-off auto-close).
        if let Some(ctx) = app.global::<fret_runtime::WindowInputContextService>() {
            let mut to_clear: Vec<AppWindowId> = Vec::new();
            for w in self.known_windows.iter().copied() {
                if ctx.snapshot(w).is_none() {
                    to_clear.push(w);
                }
            }
            for w in to_clear {
                self.clear_window(w);
            }
        }
        self.note_window_seen(window);

        let last_pointer_position = self
            .per_window
            .get(&window)
            .and_then(|ring| ring.last_pointer_position);
        let hit_test = last_pointer_position.map(|pos| UiHitTestSnapshotV1::from_tree(pos, ui));

        let element_diag = element_runtime.and_then(|runtime| {
            runtime.diagnostics_snapshot(window).map(|snapshot| {
                ElementDiagnosticsSnapshotV1::from_runtime(
                    window,
                    runtime,
                    snapshot,
                    self.cfg.max_debug_string_bytes,
                )
            })
        });

        let raw_semantics = ui.semantics_snapshot();
        let semantics_fingerprint = raw_semantics.map(|snapshot| {
            semantics_fingerprint_v1(
                snapshot,
                self.cfg.redact_text,
                self.cfg.max_debug_string_bytes,
            )
        });

        if self.inspector.enabled {
            let hovered = last_pointer_position.and_then(|pos| {
                raw_semantics.and_then(|snap| {
                    pick::pick_semantics_node_at(snap, ui, pos).map(|n| n.id.data().as_ffi())
                })
            });
            self.update_inspect_hover(window, raw_semantics, hovered, element_runtime);
        }
        self.apply_inspect_navigation(window, raw_semantics, element_runtime);
        self.update_inspect_focus_lines(window, raw_semantics, element_runtime);

        let semantics = self
            .cfg
            .capture_semantics
            .then_some(raw_semantics)
            .flatten()
            .map(|snap| {
                UiSemanticsSnapshotV1::from_snapshot(
                    snap,
                    self.cfg.redact_text,
                    self.cfg.max_debug_string_bytes,
                    self.cfg.max_semantics_nodes,
                    self.cfg.semantics_test_ids_only,
                )
            });

        let ring = self.per_window.entry(window).or_default();
        if let Some(fingerprint) = semantics_fingerprint {
            if ring.test_id_bounds_fingerprint != Some(fingerprint) {
                ring.test_id_bounds.clear();
                if let Some(snapshot) = raw_semantics {
                    // Keep this bounded; scripted diagnostics primarily use `test_id` selectors.
                    // Rebuilding this map on every frame is wasteful, so we key it off of the
                    // semantics fingerprint.
                    let cap = self.cfg.max_semantics_nodes.max(1) as usize;
                    for node in &snapshot.nodes {
                        if ring.test_id_bounds.len() >= cap {
                            break;
                        }
                        if let Some(test_id) = node.test_id.as_deref() {
                            ring.test_id_bounds
                                .insert(test_id.to_string(), node.bounds);
                        }
                    }
                }
                ring.test_id_bounds_fingerprint = Some(fingerprint);
            }
        } else {
            ring.test_id_bounds.clear();
            ring.test_id_bounds_fingerprint = None;
        }
        let viewport_input = std::mem::take(&mut ring.viewport_input_this_frame);

        let changed_models = std::mem::take(&mut ring.last_changed_models);
        let changed_model_sources_top =
            snapshot_recording::changed_model_sources_top(app, &changed_models);

        let resource_caches = snapshot_recording::resource_caches_for_window(
            app,
            window.data().as_ffi(),
            self.cfg.redact_text,
            self.cfg.max_debug_string_bytes,
        );

        let renderer_perf = app
            .global::<fret_render::RendererPerfFrameStore>()
            .and_then(|store| store.latest_for_window(window));

        let mut debug = UiTreeDebugSnapshotV1::from_tree(
            app,
            window,
            ui,
            renderer_perf,
            element_runtime,
            hit_test,
            element_diag,
            semantics,
            self.cfg.max_gating_trace_entries,
            self.cfg.redact_text,
            self.cfg.max_debug_string_bytes,
        );
        debug.viewport_input = viewport_input;

        let app_snapshot = self
            .app_snapshot_provider
            .as_ref()
            .and_then(|provider| provider(app, window));

        let frame_clock = app
            .global::<fret_core::WindowFrameClockService>()
            .and_then(|svc| {
                let snapshot = svc.snapshot(window)?;
                let fixed_delta_ms = svc.effective_fixed_delta(window).map(|d| {
                    let ms = d.as_millis();
                    ms.min(u64::MAX as u128) as u64
                });
                Some(UiFrameClockSnapshotV1 {
                    now_monotonic_ms: {
                        let ms = snapshot.now_monotonic.as_millis();
                        ms.min(u64::MAX as u128) as u64
                    },
                    delta_ms: {
                        let ms = snapshot.delta.as_millis();
                        ms.min(u64::MAX as u128) as u64
                    },
                    fixed_delta_ms,
                })
            });

        let (safe_area_insets, occlusion_insets) = app
            .global::<fret_core::WindowMetricsService>()
            .map(|svc| {
                (
                    svc.safe_area_insets(window).map(ui_edges_from_edges),
                    svc.occlusion_insets(window).map(ui_edges_from_edges),
                )
            })
            .unwrap_or((None, None));

        let input_ctx = app
            .global::<fret_runtime::WindowInputContextService>()
            .and_then(|svc| svc.snapshot(window));

        let window_text_input_snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window));

        let clipboard = app
            .global::<fret_runtime::WindowClipboardDiagnosticsStore>()
            .and_then(|store| {
                let frame_id = app.frame_id();
                let last_read = store.last_read_for_window(window, frame_id);
                let last_write = store.last_write_for_window(window, frame_id);
                if last_read.is_none() && last_write.is_none() {
                    return None;
                }
                Some(UiClipboardDiagnosticsSnapshotV1 {
                    last_read_token: last_read.map(|e| e.token.0),
                    last_read_unavailable: last_read.map(|e| e.unavailable),
                    last_read_message: last_read.and_then(|e| e.message.clone()),
                    last_write_unavailable: last_write.map(|e| e.unavailable),
                    last_write_message: last_write.and_then(|e| e.message.clone()),
                })
            });

        let wgpu_adapter = app
            .global::<fret_render::WgpuAdapterSelectionSnapshot>()
            .and_then(|snapshot| serde_json::to_value(snapshot).ok());

        let window_snapshot_seq = ring.snapshot_seq;
        ring.snapshot_seq = ring.snapshot_seq.saturating_add(1);

        let snapshot = UiDiagnosticsSnapshotV1 {
            schema_version: 1,
            tick_id: app.tick_id().0,
            frame_id: app.frame_id().0,
            window_snapshot_seq,
            window: window.data().as_ffi(),
            timestamp_unix_ms: unix_ms_now(),
            scale_factor,
            window_bounds: RectV1::from(bounds),
            scene_ops: scene.ops_len() as u64,
            scene_fingerprint: scene.fingerprint(),
            semantics_fingerprint,
            debug,
            frame_clock,
            changed_models,
            changed_globals: std::mem::take(&mut ring.last_changed_globals),
            changed_model_sources_top,
            resource_caches,
            app_snapshot,
            safe_area_insets,
            occlusion_insets,
            focus_is_text_input: input_ctx.map(|c| c.focus_is_text_input),
            is_composing: window_text_input_snapshot.map(|s| s.is_composing),
            clipboard,
            primary_pointer_type: ring
                .last_pointer_type
                .map(|t| viewport_pointer_type_label(t).to_string()),
            caps: input_ctx.map(|c| UiPlatformCapabilitiesSummaryV1 {
                platform: c.platform.as_str().to_string(),
                ui_window_hover_detection: c.caps.ui.window_hover_detection.as_str().to_string(),
                clipboard_text: c.caps.clipboard.text.read && c.caps.clipboard.text.write,
                clipboard_text_read: c.caps.clipboard.text.read,
                clipboard_text_write: c.caps.clipboard.text.write,
                clipboard_primary_text: c.caps.clipboard.primary_text,
                ime: c.caps.ime.enabled,
                ime_set_cursor_area: c.caps.ime.set_cursor_area,
                fs_file_dialogs: c.caps.fs.file_dialogs,
                shell_share_sheet: c.caps.shell.share_sheet,
                shell_incoming_open: c.caps.shell.incoming_open,
            }),
            wgpu_adapter,
        };

        ring.push_snapshot(&self.cfg, snapshot);

        self.record_shortcut_routing_trace_for_window(app, window);
        self.record_command_dispatch_trace_for_window(app, window);

        if let Some(pending) = self.inspector.take_pending_pick_for_window(window) {
            self.resolve_pending_pick_for_window(pending, raw_semantics, ui, element_runtime);
        }
    }

    fn record_shortcut_routing_trace_for_window(&mut self, app: &App, window: AppWindowId) {
        let Some(active) = self.active_scripts.get_mut(&window) else {
            return;
        };
        let Some(store) = app.global::<fret_runtime::WindowShortcutRoutingDiagnosticsStore>()
        else {
            return;
        };

        let step_index = active
            .last_injected_step
            .unwrap_or_else(|| active.next_step.min(u32::MAX as usize) as u32);

        let max_entries = MAX_SHORTCUT_ROUTING_TRACE_ENTRIES;
        let decisions = store.snapshot_since(window, active.last_shortcut_routing_seq, max_entries);
        if decisions.is_empty() {
            return;
        }

        for decision in decisions {
            active.last_shortcut_routing_seq = active
                .last_shortcut_routing_seq
                .max(decision.seq.saturating_add(1));

            let phase = match decision.phase {
                fret_runtime::ShortcutRoutingPhase::PreDispatch => "pre_dispatch",
                fret_runtime::ShortcutRoutingPhase::PostDispatch => "post_dispatch",
            };
            let outcome = match decision.outcome {
                fret_runtime::ShortcutRoutingOutcome::ReservedForIme => "reserved_for_ime",
                fret_runtime::ShortcutRoutingOutcome::ConsumedByWidget => "consumed_by_widget",
                fret_runtime::ShortcutRoutingOutcome::CommandDispatched => "command_dispatched",
                fret_runtime::ShortcutRoutingOutcome::CommandDisabled => "command_disabled",
                fret_runtime::ShortcutRoutingOutcome::SequenceContinuation => {
                    "sequence_continuation"
                }
                fret_runtime::ShortcutRoutingOutcome::SequenceReplay => "sequence_replay",
                fret_runtime::ShortcutRoutingOutcome::NoMatch => "no_match",
                fret_runtime::ShortcutRoutingOutcome::NoKeymap => "no_keymap",
            };

            push_shortcut_routing_trace(
                &mut active.shortcut_routing_trace,
                UiShortcutRoutingTraceEntryV1 {
                    step_index,
                    note: None,
                    frame_id: decision.frame_id.0,
                    phase: phase.to_string(),
                    deferred: decision.deferred,
                    focus_is_text_input: decision.focus_is_text_input,
                    ime_composing: decision.ime_composing,
                    key: format!("{:?}", decision.key),
                    modifiers: UiKeyModifiersV1::from_modifiers(decision.modifiers),
                    repeat: decision.repeat,
                    outcome: outcome.to_string(),
                    command: decision.command.as_ref().map(|c| c.as_str().to_string()),
                    command_enabled: decision.command_enabled,
                    pending_sequence_len: Some(decision.pending_sequence_len),
                },
            );
        }
    }

    fn record_command_dispatch_trace_for_window(&mut self, app: &App, window: AppWindowId) {
        let Some(active) = self.active_scripts.get_mut(&window) else {
            return;
        };
        let Some(store) = app.global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
        else {
            return;
        };

        let step_index = active
            .last_injected_step
            .unwrap_or_else(|| active.next_step.min(u32::MAX as usize) as u32);

        let max_entries = MAX_SHORTCUT_ROUTING_TRACE_ENTRIES;
        let decisions =
            store.snapshot_since(window, active.last_command_dispatch_seq, max_entries);
        if decisions.is_empty() {
            return;
        }

        for decision in decisions {
            active.last_command_dispatch_seq = active
                .last_command_dispatch_seq
                .max(decision.seq.saturating_add(1));

            let source_kind = match decision.source.kind {
                fret_runtime::CommandDispatchSourceKindV1::Pointer => "pointer",
                fret_runtime::CommandDispatchSourceKindV1::Keyboard => "keyboard",
                fret_runtime::CommandDispatchSourceKindV1::Shortcut => "shortcut",
                fret_runtime::CommandDispatchSourceKindV1::Programmatic => "programmatic",
            };

            push_command_dispatch_trace(
                &mut active.command_dispatch_trace,
                UiScriptCommandDispatchTraceEntryV1 {
                    step_index,
                    frame_id: decision.frame_id.0,
                    command: decision.command.as_str().to_string(),
                    handled: decision.handled,
                    handled_by_scope: decision.handled_by_scope.map(|s| match s {
                        fret_runtime::CommandScope::Widget => "widget".to_string(),
                        fret_runtime::CommandScope::Window => "window".to_string(),
                        fret_runtime::CommandScope::App => "app".to_string(),
                    }),
                    handled_by_driver: decision.handled_by_driver,
                    stopped: decision.stopped,
                    source_kind: source_kind.to_string(),
                    source_element: decision.source.element,
                    handled_by_element: decision.handled_by_element,
                    started_from_focus: decision.started_from_focus,
                    used_default_root_fallback: decision.used_default_root_fallback,
                },
            );
        }
    }

    fn dump_bundle(&mut self, label: Option<&str>) -> Option<PathBuf> {
        self.dump_bundle_with_options(label, None, None)
    }

    fn dump_bundle_with_options(
        &mut self,
        label: Option<&str>,
        dump_max_snapshots_override: Option<usize>,
        request_id: Option<u64>,
    ) -> Option<PathBuf> {
        bundle_dump::dump_bundle_with_options(self, label, dump_max_snapshots_override, request_id)
    }

    fn next_script_run_id(&mut self) -> u64 {
        let mut id = unix_ms_now();
        if id <= self.last_script_run_id {
            id = self.last_script_run_id.saturating_add(1);
        }
        self.last_script_run_id = id;
        id
    }
}
