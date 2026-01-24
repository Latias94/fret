use super::*;

const PENDING_SHORTCUT_TIMEOUT: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone)]
pub(super) struct CapturedKeystroke {
    pub(super) chord: KeyChord,
    pub(super) text: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct PendingShortcut {
    pub(super) keystrokes: Vec<CapturedKeystroke>,
    pub(super) focus: Option<NodeId>,
    pub(super) barrier_root: Option<NodeId>,
    pub(super) fallback: Option<CommandId>,
    pub(super) timer: Option<fret_runtime::TimerToken>,
    pub(super) capture_next_text_input_key: Option<KeyCode>,
}

pub(super) struct PointerDownOutsideParams<'a> {
    pub(super) input_ctx: &'a InputContext,
    pub(super) active_layer_roots: &'a [NodeId],
    pub(super) base_root: NodeId,
    pub(super) hit: Option<NodeId>,
    pub(super) event: &'a Event,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct PointerDownOutsideOutcome {
    pub(super) dispatched: bool,
    pub(super) suppress_hit_test_dispatch: bool,
}

pub(super) struct KeydownShortcutParams<'a> {
    pub(super) input_ctx: &'a InputContext,
    pub(super) barrier_root: Option<NodeId>,
    pub(super) focus_is_text_input: bool,
    pub(super) key: KeyCode,
    pub(super) modifiers: fret_core::Modifiers,
    pub(super) repeat: bool,
}

impl<H: UiHost> UiTree<H> {
    fn command_is_enabled(
        app: &H,
        window: Option<fret_core::AppWindowId>,
        fallback_input_ctx: &InputContext,
        command: &CommandId,
    ) -> bool {
        let Some(window) = window else {
            return true;
        };
        fret_runtime::command_is_enabled_for_window_with_input_ctx_fallback(
            app,
            window,
            command,
            fallback_input_ctx.clone(),
        )
    }

    pub(super) fn sync_pending_shortcut_overlay_state(
        &mut self,
        app: &mut H,
        input_ctx: Option<&InputContext>,
    ) {
        let Some(window) = self.window else {
            return;
        };

        let sequence: Vec<KeyChord> = self
            .pending_shortcut
            .keystrokes
            .iter()
            .map(|s| s.chord)
            .collect();

        let input_ctx = input_ctx.cloned().unwrap_or_default();

        let continuations = if sequence.is_empty() {
            Vec::new()
        } else if let Some(service) = app.global::<KeymapService>() {
            let mut conts: Vec<crate::pending_shortcut::PendingShortcutContinuation> = service
                .keymap
                .continuations(&input_ctx, &sequence)
                .into_iter()
                .map(|c| crate::pending_shortcut::PendingShortcutContinuation {
                    next: c.next,
                    command: c.matched.exact.clone().flatten(),
                    has_continuation: c.matched.has_continuation,
                })
                .collect();

            conts.sort_by(|a, b| {
                fn mods_key(mods: fret_core::Modifiers) -> u8 {
                    (mods.ctrl as u8)
                        | ((mods.shift as u8) << 1)
                        | ((mods.alt as u8) << 2)
                        | ((mods.meta as u8) << 3)
                        | ((mods.alt_gr as u8) << 4)
                }
                fn key_key(key: KeyCode) -> u8 {
                    match key {
                        KeyCode::ArrowLeft => 0,
                        KeyCode::ArrowRight => 1,
                        KeyCode::ArrowUp => 2,
                        KeyCode::ArrowDown => 3,
                        _ => 255,
                    }
                }

                mods_key(a.next.mods)
                    .cmp(&mods_key(b.next.mods))
                    .then_with(|| key_key(a.next.key).cmp(&key_key(b.next.key)))
            });

            conts
        } else {
            Vec::new()
        };

        app.with_global_mut(
            crate::PendingShortcutOverlayState::default,
            |state, _app| {
                state.set_sequence(window, input_ctx, sequence, continuations);
            },
        );
    }

    pub(super) fn should_defer_keydown_shortcut_matching_to_text_input(
        key: KeyCode,
        modifiers: fret_core::Modifiers,
        focus_is_text_input: bool,
    ) -> bool {
        if !focus_is_text_input {
            return false;
        }
        if modifiers.ctrl || modifiers.alt || modifiers.meta {
            return false;
        }
        matches!(
            key,
            KeyCode::Tab
                | KeyCode::Space
                | KeyCode::Enter
                | KeyCode::NumpadEnter
                | KeyCode::Escape
                | KeyCode::ArrowUp
                | KeyCode::ArrowDown
                | KeyCode::ArrowLeft
                | KeyCode::ArrowRight
                | KeyCode::Backspace
                | KeyCode::Delete
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown
        )
    }

    pub(super) fn handle_keydown_shortcuts(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        params: KeydownShortcutParams<'_>,
    ) -> bool {
        if self.replaying_pending_shortcut {
            // Pending shortcut replay bypasses shortcut matching and sequence state.
            return false;
        }

        if params.repeat {
            // Allow key-repeat only for explicitly repeatable commands (e.g. text editing).
            if let Some(service) = app.global::<KeymapService>() {
                let chord = KeyChord::new(params.key, params.modifiers);
                if let Some(command) = service.keymap.resolve(params.input_ctx, chord)
                    && app
                        .commands()
                        .get(command.clone())
                        .is_some_and(|m| m.repeatable)
                {
                    if Self::command_is_enabled(app, self.window, params.input_ctx, &command) {
                        self.suppress_text_input_until_key_up = Some(params.key);
                        app.push_effect(Effect::Command {
                            window: self.window,
                            command,
                        });
                        return true;
                    }

                    return false;
                }
            }
            return false;
        }

        let Some(service) = app.global::<KeymapService>() else {
            return false;
        };

        let chord = KeyChord::new(params.key, params.modifiers);

        if !self.pending_shortcut.keystrokes.is_empty() {
            self.pending_shortcut
                .keystrokes
                .push(CapturedKeystroke { chord, text: None });

            let sequence: Vec<KeyChord> = self
                .pending_shortcut
                .keystrokes
                .iter()
                .map(|s| s.chord)
                .collect();
            let matched = service.keymap.match_sequence(params.input_ctx, &sequence);

            if matched.has_continuation {
                self.pending_shortcut.fallback = matched.exact.and_then(|c| c);
                self.pending_shortcut.focus = self.focus;
                self.pending_shortcut.barrier_root = params.barrier_root;
                self.pending_shortcut.capture_next_text_input_key = (params.focus_is_text_input
                    && !params.modifiers.ctrl
                    && !params.modifiers.meta)
                    .then_some(params.key);
                self.suppress_text_input_until_key_up = Some(params.key);
                self.schedule_pending_shortcut_timeout(app);
                self.sync_pending_shortcut_overlay_state(app, Some(params.input_ctx));
                return true;
            }

            if let Some(Some(command)) = matched.exact {
                if Self::command_is_enabled(app, self.window, params.input_ctx, &command) {
                    self.clear_pending_shortcut(app);
                    self.suppress_text_input_until_key_up = Some(params.key);
                    app.push_effect(Effect::Command {
                        window: self.window,
                        command,
                    });
                    return true;
                }

                // Treat disabled commands as "not matched" so the keystrokes are replayed.
                let pending = std::mem::take(&mut self.pending_shortcut);
                if let Some(token) = pending.timer {
                    app.push_effect(Effect::CancelTimer { token });
                }
                self.sync_pending_shortcut_overlay_state(app, None);
                self.replay_captured_keystrokes(
                    app,
                    services,
                    params.input_ctx,
                    pending.keystrokes,
                );
                return true;
            }

            let pending = std::mem::take(&mut self.pending_shortcut);
            if let Some(token) = pending.timer {
                app.push_effect(Effect::CancelTimer { token });
            }
            self.sync_pending_shortcut_overlay_state(app, None);
            self.replay_captured_keystrokes(app, services, params.input_ctx, pending.keystrokes);
            return true;
        }

        let matched = service
            .keymap
            .match_sequence(params.input_ctx, std::slice::from_ref(&chord));
        if matched.has_continuation {
            self.pending_shortcut.keystrokes = vec![CapturedKeystroke { chord, text: None }];
            self.pending_shortcut.focus = self.focus;
            self.pending_shortcut.barrier_root = params.barrier_root;
            self.pending_shortcut.fallback = matched.exact.and_then(|c| c);
            self.pending_shortcut.capture_next_text_input_key =
                (params.focus_is_text_input && !params.modifiers.ctrl && !params.modifiers.meta)
                    .then_some(params.key);
            self.suppress_text_input_until_key_up = Some(params.key);
            self.schedule_pending_shortcut_timeout(app);
            self.sync_pending_shortcut_overlay_state(app, Some(params.input_ctx));
            return true;
        }

        if let Some(command) = service.keymap.resolve(params.input_ctx, chord) {
            if Self::command_is_enabled(app, self.window, params.input_ctx, &command) {
                self.suppress_text_input_until_key_up = Some(params.key);
                app.push_effect(Effect::Command {
                    window: self.window,
                    command,
                });
                return true;
            }

            // Treat disabled commands as "not matched" so the event can fall through to the
            // normal dispatch path (e.g. text inputs).
            return false;
        }

        false
    }

    pub(super) fn clear_pending_shortcut(&mut self, app: &mut H) {
        if let Some(token) = self.pending_shortcut.timer.take() {
            app.push_effect(Effect::CancelTimer { token });
        }
        self.pending_shortcut = PendingShortcut::default();
        self.sync_pending_shortcut_overlay_state(app, None);
    }

    pub(super) fn schedule_pending_shortcut_timeout(&mut self, app: &mut H) {
        if self.pending_shortcut.keystrokes.is_empty() {
            return;
        }

        if let Some(token) = self.pending_shortcut.timer.take() {
            app.push_effect(Effect::CancelTimer { token });
        }
        let token = app.next_timer_token();
        self.pending_shortcut.timer = Some(token);
        app.push_effect(Effect::SetTimer {
            window: self.window,
            token,
            after: PENDING_SHORTCUT_TIMEOUT,
            repeat: None,
        });
    }

    pub(super) fn replay_captured_keystrokes(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        ctx: &InputContext,
        keystrokes: Vec<CapturedKeystroke>,
    ) {
        let prev = self.replaying_pending_shortcut;
        self.replaying_pending_shortcut = true;

        for stroke in keystrokes {
            if let Some(service) = app.global::<KeymapService>()
                && let Some(command) = service.keymap.resolve(ctx, stroke.chord)
            {
                if Self::command_is_enabled(app, self.window, ctx, &command) {
                    app.push_effect(Effect::Command {
                        window: self.window,
                        command,
                    });
                }
                continue;
            }

            let down = Event::KeyDown {
                key: stroke.chord.key,
                modifiers: stroke.chord.mods,
                repeat: false,
            };
            self.dispatch_event(app, services, &down);

            if let Some(text) = stroke.text {
                let event = Event::TextInput(text);
                self.dispatch_event(app, services, &event);
            }

            let up = Event::KeyUp {
                key: stroke.chord.key,
                modifiers: stroke.chord.mods,
            };
            self.dispatch_event(app, services, &up);
        }

        self.replaying_pending_shortcut = prev;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use fret_core::{AppWindowId, Event, KeyCode, Modifiers, Point, Px, Rect, Size};
    use fret_runtime::keymap::Binding;
    use fret_runtime::{CommandId, Keymap, KeymapService, PlatformFilter};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[derive(Default)]
    struct RootStack;

    impl<H: UiHost> Widget<H> for RootStack {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                let _ = cx.layout_in(child, cx.bounds);
            }
            cx.available
        }
    }

    struct TextInputConsumesArrows {
        saw_arrow_right: Arc<AtomicBool>,
    }

    impl<H: UiHost> Widget<H> for TextInputConsumesArrows {
        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            match event {
                Event::Pointer(fret_core::PointerEvent::Down { .. }) => {
                    cx.request_focus(cx.node);
                    cx.stop_propagation();
                }
                Event::KeyDown {
                    key: KeyCode::ArrowRight,
                    modifiers,
                    repeat: false,
                } if *modifiers == Modifiers::default() => {
                    self.saw_arrow_right.store(true, Ordering::SeqCst);
                    cx.stop_propagation();
                }
                _ => {}
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    #[derive(Default)]
    struct FakeUiServices;

    impl fret_core::TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    #[test]
    fn pending_sequence_matches_reserved_second_chord_before_text_input_consumes() {
        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let command = CommandId::from("test.multi_stroke");
        let mut keymap = Keymap::empty();
        keymap.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![
                KeyChord::new(
                    KeyCode::KeyK,
                    Modifiers {
                        ctrl: true,
                        ..Default::default()
                    },
                ),
                KeyChord::new(KeyCode::ArrowRight, Modifiers::default()),
            ],
            when: None,
            command: Some(command.clone()),
        });
        app.set_global(KeymapService { keymap });

        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(AppWindowId::default());

        let saw_arrow_right = Arc::new(AtomicBool::new(false));
        let root = ui.create_node(RootStack);
        let text_input = ui.create_node(TextInputConsumesArrows {
            saw_arrow_right: saw_arrow_right.clone(),
        });
        ui.add_child(root, text_input);
        ui.set_root(root);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let mut services = FakeUiServices;
        ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(text_input));
        let _ = app.take_effects();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::KeyK,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );
        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .all(|e| !matches!(e, Effect::Command { command: c, .. } if c == &command)),
            "first chord should only enter pending state"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        assert!(
            !saw_arrow_right.load(Ordering::SeqCst),
            "reserved key should not reach the text input while a pending shortcut is active"
        );

        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::Command { command: c, .. } if c == &command)),
            "second chord should dispatch the multi-stroke command"
        );
    }
}
