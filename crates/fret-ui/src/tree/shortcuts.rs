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
                    self.suppress_text_input_until_key_up = Some(params.key);
                    app.push_effect(Effect::Command {
                        window: self.window,
                        command,
                    });
                    return true;
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
                return true;
            }

            if let Some(Some(command)) = matched.exact {
                self.clear_pending_shortcut(app);
                self.suppress_text_input_until_key_up = Some(params.key);
                app.push_effect(Effect::Command {
                    window: self.window,
                    command,
                });
                return true;
            }

            let pending = std::mem::take(&mut self.pending_shortcut);
            if let Some(token) = pending.timer {
                app.push_effect(Effect::CancelTimer { token });
            }
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
            return true;
        }

        if let Some(command) = service.keymap.resolve(params.input_ctx, chord) {
            self.suppress_text_input_until_key_up = Some(params.key);
            app.push_effect(Effect::Command {
                window: self.window,
                command,
            });
            return true;
        }

        false
    }

    pub(super) fn clear_pending_shortcut(&mut self, app: &mut H) {
        if let Some(token) = self.pending_shortcut.timer.take() {
            app.push_effect(Effect::CancelTimer { token });
        }
        self.pending_shortcut = PendingShortcut::default();
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
                app.push_effect(Effect::Command {
                    window: self.window,
                    command,
                });
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
