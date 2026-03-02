use super::*;

fn eval_docking_predicate_from_recent_debug_snapshot(
    svc: &UiDiagnosticsService,
    window: AppWindowId,
    predicate: &UiPredicateV1,
    max_age_ms: u64,
) -> Option<bool> {
    let ring = svc.per_window.get(&window)?;
    let snapshot = ring.snapshots.back()?;
    let age_ms = unix_ms_now().saturating_sub(snapshot.timestamp_unix_ms);
    if age_ms > max_age_ms {
        return None;
    }
    let docking = snapshot.debug.docking_interaction.as_ref()?;

    match predicate {
        UiPredicateV1::DockDropPreviewKindIs { preview_kind } => {
            let preview = docking.dock_drop_resolve.as_ref()?.preview.as_ref()?;
            let have = match preview.kind {
                UiDockDropPreviewKindDiagnosticsV1::WrapBinary => "wrap_binary",
                UiDockDropPreviewKindDiagnosticsV1::InsertIntoSplit { .. } => "insert_into_split",
            };
            Some(have == preview_kind.as_str())
        }
        UiPredicateV1::DockDropResolveSourceIs { source } => {
            let resolve = docking.dock_drop_resolve.as_ref()?;
            let have = match resolve.source {
                UiDockDropResolveSourceV1::InvertDocking => "invert_docking",
                UiDockDropResolveSourceV1::OutsideWindow => "outside_window",
                UiDockDropResolveSourceV1::FloatZone => "float_zone",
                UiDockDropResolveSourceV1::EmptyDockSpace => "empty_dock_space",
                UiDockDropResolveSourceV1::LayoutBoundsMiss => "layout_bounds_miss",
                UiDockDropResolveSourceV1::LatchedPreviousHover => "latched_previous_hover",
                UiDockDropResolveSourceV1::TabBar => "tab_bar",
                UiDockDropResolveSourceV1::FloatingTitleBar => "floating_title_bar",
                UiDockDropResolveSourceV1::OuterHintRect => "outer_hint_rect",
                UiDockDropResolveSourceV1::InnerHintRect => "inner_hint_rect",
                UiDockDropResolveSourceV1::None => "none",
            };
            Some(have == source.as_str())
        }
        UiPredicateV1::DockDropResolvedIsSome { some } => Some(
            docking
                .dock_drop_resolve
                .as_ref()
                .is_some_and(|d| d.resolved.is_some() == *some),
        ),
        UiPredicateV1::DockDropResolvedZoneIs { zone } => {
            let resolved = docking.dock_drop_resolve.as_ref()?.resolved.as_ref()?;
            let have = match resolved.zone {
                UiDropZoneV1::Center => "center",
                UiDropZoneV1::Left => "left",
                UiDropZoneV1::Right => "right",
                UiDropZoneV1::Top => "top",
                UiDropZoneV1::Bottom => "bottom",
            };
            Some(have == zone.as_str())
        }
        UiPredicateV1::DockDropResolvedInsertIndexIs { index } => {
            let resolved = docking.dock_drop_resolve.as_ref()?.resolved.as_ref()?;
            Some(resolved.insert_index == Some(*index as u64))
        }
        UiPredicateV1::DockGraphCanonicalIs { canonical } => Some(
            docking
                .dock_graph_stats
                .is_some_and(|s| s.canonical_ok == *canonical),
        ),
        UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { has_nested } => Some(
            docking
                .dock_graph_stats
                .is_some_and(|s| s.has_nested_same_axis_splits == *has_nested),
        ),
        UiPredicateV1::DockGraphNodeCountLe { max } => Some(
            docking
                .dock_graph_stats
                .is_some_and(|s| s.node_count <= *max),
        ),
        UiPredicateV1::DockGraphMaxSplitDepthLe { max } => Some(
            docking
                .dock_graph_stats
                .is_some_and(|s| s.max_split_depth <= *max),
        ),
        UiPredicateV1::DockGraphSignatureIs { signature } => Some(
            docking
                .dock_graph_signature
                .as_ref()
                .is_some_and(|s| s.signature == *signature),
        ),
        UiPredicateV1::DockGraphSignatureContains { needle } => Some(
            docking
                .dock_graph_signature
                .as_ref()
                .is_some_and(|s| s.signature.contains(needle)),
        ),
        UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => Some(
            docking
                .dock_graph_signature
                .as_ref()
                .is_some_and(|s| s.fingerprint64 == *fingerprint64),
        ),
        _ => None,
    }
}

pub(super) fn handle_assert_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    text_font_stack_key_stable_frames: u32,
    font_catalog_populated: bool,
    system_font_rescan_idle: bool,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::Assert {
        window: target_window,
        predicate,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let mut predicate_window = window;
    if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            if UiDiagnosticsService::predicate_can_eval_off_window(&predicate)
                || UiDiagnosticsService::predicate_can_eval_from_cached_test_id_bounds(&predicate)
            {
                predicate_window = target_window;
                output.effects.push(Effect::Redraw(target_window));
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(target_window));
                output.request_redraw = true;
            } else {
                *handoff_to = Some(target_window);
                output.effects.push(Effect::Redraw(target_window));
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(target_window));
                output.request_redraw = true;
            }
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }

    if *stop_script {
        // Fall through to common termination logic.
    } else if handoff_to.is_some() {
        // This step is window-targeted; the runtime will migrate the script.
    } else {
        let cache_eval = svc.eval_predicate_from_cached_test_id_bounds(predicate_window, &predicate);
        if cache_eval.used_cache {
            let kind = if cache_eval.stale {
                "diag.cached_test_id_predicate.stale"
            } else {
                "diag.cached_test_id_predicate.hit"
            };
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: kind.to_string(),
                    step_index: Some(step_index.min(u32::MAX as usize) as u32),
                    note: Some(format!(
                        "predicate_window={} test_id={:?} ok={:?} age_ms={:?} snapshot_seq={:?} max_age_ms={:?}",
                        predicate_window.data().as_ffi(),
                        cache_eval.test_id.as_deref(),
                        cache_eval.ok,
                        cache_eval.age_ms,
                        cache_eval.window_snapshot_seq,
                        cache_eval.max_age_ms
                    )),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
        }

        if cache_eval.used_cache
            && cache_eval.stale
            && UiDiagnosticsService::predicate_can_eval_from_cached_test_id_bounds(&predicate)
        {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-assert-stale-test-id-cache"
            ));
            *stop_script = true;
            *failure_reason = Some("cached_test_id_predicate_stale".to_string());
            output.request_redraw = true;
        } else {
            let ok = match cache_eval.ok {
                Some(ok) => ok,
                None => match &predicate {
            UiPredicateV1::EventKindSeen { event_kind } => svc
                .per_window
                .get(&predicate_window)
                .is_some_and(|ring| ring.events.iter().any(|e| e.kind == *event_kind)),
            UiPredicateV1::RunnerAccessibilityActivated => app
                .global::<fret_runtime::RunnerAccessibilityDiagnosticsStore>()
                .and_then(|store| store.snapshot(predicate_window))
                .is_some_and(|snapshot| snapshot.activation_requests > 0),
            UiPredicateV1::TextFontStackKeyStable { stable_frames } => {
                text_font_stack_key_stable_frames >= *stable_frames
            }
            UiPredicateV1::FontCatalogPopulated => font_catalog_populated,
            UiPredicateV1::SystemFontRescanIdle => system_font_rescan_idle,
            _ => {
                if let Some(ok) = eval_docking_predicate_from_recent_debug_snapshot(
                    svc,
                    predicate_window,
                    &predicate,
                    250,
                ) {
                    ok
                } else {
                    let docking_diag = app
                        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                        .and_then(|store| store.docking_latest_for_window(predicate_window));
                    let workspace_diag = app
                        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                        .and_then(|store| store.workspace_latest_for_window(predicate_window));
                    let input_ctx = app
                        .global::<fret_runtime::WindowInputContextService>()
                        .and_then(|svc| svc.snapshot(predicate_window));
                    let text_input_snapshot = app
                        .global::<fret_runtime::WindowTextInputSnapshotService>()
                        .and_then(|svc| svc.snapshot(predicate_window));
                    let dock_drag_runtime =
                        dock_drag_runtime_state(app, svc.known_windows.as_slice());
                    let platform_caps = app.global::<fret_runtime::PlatformCapabilities>();
                    let open_window_count = UiDiagnosticsService::open_window_count_for_predicates(app);

                    if predicate_window == window {
                        if let Some(snapshot) = semantics_snapshot {
                            record_overlay_placement_trace(
                                &mut active.overlay_placement_trace,
                                element_runtime,
                                Some(snapshot),
                                window,
                                step_index as u32,
                                "assert",
                            );
                            eval_predicate(
                                snapshot,
                                window_bounds,
                                predicate_window,
                                active.scope_root_for_window(predicate_window),
                                input_ctx,
                                element_runtime,
                                text_input_snapshot,
                                app.global::<fret_core::RendererTextPerfSnapshot>().copied(),
                                app.global::<fret_core::RendererTextFontTraceSnapshot>(),
                                svc.known_windows.as_slice(),
                                open_window_count,
                                platform_caps,
                                docking_diag,
                                workspace_diag,
                                dock_drag_runtime.as_ref(),
                                text_font_stack_key_stable_frames,
                                font_catalog_populated,
                                system_font_rescan_idle,
                                &predicate,
                            )
                        } else {
                            eval_predicate_without_semantics(
                                predicate_window,
                                svc.known_windows.as_slice(),
                                open_window_count,
                                platform_caps,
                                docking_diag,
                                workspace_diag,
                                dock_drag_runtime.as_ref(),
                                &predicate,
                            )
                            .unwrap_or_else(|| {
                                *force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-assert-no-semantics"
                                ));
                                *stop_script = true;
                                *failure_reason = Some("no_semantics_snapshot".to_string());
                                output.request_redraw = true;
                                false
                            })
                        }
                    } else {
                        // Off-window predicates must not reuse the current window's semantics snapshot.
                        eval_predicate_without_semantics(
                            predicate_window,
                            svc.known_windows.as_slice(),
                            open_window_count,
                            platform_caps,
                            docking_diag,
                            workspace_diag,
                            dock_drag_runtime.as_ref(),
                            &predicate,
                        )
                        .unwrap_or_else(|| {
                            *force_dump_label =
                                Some(format!("script-step-{step_index:04}-assert-no-semantics"));
                            *stop_script = true;
                            *failure_reason = Some("no_semantics_snapshot".to_string());
                            output.request_redraw = true;
                            false
                        })
                    }
                }
            }
            },
        };

            if ok {
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            } else {
                *force_dump_label = Some(format!("script-step-{step_index:04}-assert-failed"));
                *stop_script = true;
                *failure_reason = Some("assert_failed".to_string());
                output.request_redraw = true;
            }
        }
    }

    true
}
