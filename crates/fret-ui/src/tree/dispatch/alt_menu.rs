use super::*;

impl<H: UiHost> UiTree<H> {
    pub(super) fn event_is_scroll_like(event: &Event) -> bool {
        // Wheel-only for now (trackpad pan / inertial scrolling can be added later as explicit
        // inputs without changing the meaning of "Wheel" today).
        matches!(event, Event::Pointer(PointerEvent::Wheel { .. }))
    }

    pub(super) fn handle_alt_menu_bar_activation(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        focus_is_text_input: bool,
        event: &Event,
    ) -> bool {
        match event {
            Event::KeyDown {
                key: key @ (KeyCode::AltLeft | KeyCode::AltRight),
                modifiers,
                repeat: false,
            } => {
                let disqualifying_modifiers =
                    modifiers.shift || modifiers.ctrl || modifiers.meta || modifiers.alt_gr;
                let in_multi_stroke_shortcut =
                    self.replaying_pending_shortcut || !self.pending_shortcut.keystrokes.is_empty();

                if disqualifying_modifiers || in_multi_stroke_shortcut {
                    self.alt_menu_bar_arm_key = None;
                    self.alt_menu_bar_canceled = false;
                    return false;
                }

                self.alt_menu_bar_arm_key = Some(*key);
                self.alt_menu_bar_canceled = false;
            }
            Event::KeyDown { key, .. } => {
                if self.alt_menu_bar_arm_key.is_some()
                    && !matches!(*key, KeyCode::AltLeft | KeyCode::AltRight)
                {
                    self.alt_menu_bar_canceled = true;
                }
            }
            Event::Pointer(PointerEvent::Down { .. }) => {
                if self.alt_menu_bar_arm_key.is_some() {
                    self.alt_menu_bar_canceled = true;
                }
            }
            Event::Ime(_) | Event::TextInput(_) => {
                if self.alt_menu_bar_arm_key.is_some() {
                    self.alt_menu_bar_canceled = true;
                }
            }
            Event::WindowFocusChanged(false) => {
                self.alt_menu_bar_arm_key = None;
                self.alt_menu_bar_canceled = false;
            }
            Event::KeyUp {
                key: key @ (KeyCode::AltLeft | KeyCode::AltRight),
                ..
            } if self.alt_menu_bar_arm_key.is_some() => {
                let should_activate =
                    self.alt_menu_bar_arm_key == Some(*key) && !self.alt_menu_bar_canceled;

                self.alt_menu_bar_arm_key = None;
                self.alt_menu_bar_canceled = false;

                if !should_activate {
                    return false;
                }

                if focus_is_text_input {
                    return false;
                }

                if !matches!(Platform::current(), Platform::Windows | Platform::Linux) {
                    return false;
                }

                let present = app
                    .global::<fret_runtime::WindowMenuBarFocusService>()
                    .is_some_and(|svc| svc.present(window));
                if !present {
                    return false;
                }

                let command = CommandId::from("focus.menu_bar");
                if app.commands().get(command.clone()).is_none() {
                    return false;
                }

                app.push_effect(Effect::Command {
                    window: Some(window),
                    command,
                });
                return true;
            }
            _ => {}
        }

        false
    }
}
