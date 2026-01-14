use crate::UiHost;
use fret_core::{AppWindowId, Event};
use std::collections::HashMap;

/// Last input modality observed for a window.
///
/// This is a lightweight policy signal used by component-layer behaviors (e.g. Radix-aligned menus)
/// to distinguish "opened via keyboard" from "opened via pointer", without requiring every
/// component to explicitly thread an "open reason" flag through its own model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputModality {
    Keyboard,
    Pointer,
}

#[derive(Default)]
struct InputModalityState {
    per_window: HashMap<AppWindowId, InputModality>,
}

pub fn modality<H: UiHost>(app: &mut H, window: Option<AppWindowId>) -> InputModality {
    let Some(window) = window else {
        return InputModality::Pointer;
    };
    app.with_global_mut(InputModalityState::default, |state, _app| {
        state
            .per_window
            .get(&window)
            .copied()
            .unwrap_or(InputModality::Pointer)
    })
}

pub fn is_keyboard<H: UiHost>(app: &mut H, window: Option<AppWindowId>) -> bool {
    modality(app, window) == InputModality::Keyboard
}

fn set_modality_if_changed<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    modality: InputModality,
) -> bool {
    app.with_global_mut(InputModalityState::default, |state, _app| {
        let prev = state
            .per_window
            .get(&window)
            .copied()
            .unwrap_or(InputModality::Pointer);
        if prev != modality {
            state.per_window.insert(window, modality);
            true
        } else {
            false
        }
    })
}

/// Update the input modality state for a window.
///
/// Returns `true` if the modality changed.
pub fn update_for_event<H: UiHost>(app: &mut H, window: AppWindowId, event: &Event) -> bool {
    match event {
        // Radix-style: any keydown counts as "keyboard interaction" until we see pointer activity.
        Event::KeyDown { .. } => set_modality_if_changed(app, window, InputModality::Keyboard),
        // Any pointer activity switches back to pointer modality.
        Event::Pointer(_) | Event::ExternalDrag(_) | Event::InternalDrag(_) => {
            set_modality_if_changed(app, window, InputModality::Pointer)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use fret_core::{Modifiers, PointerEvent};

    #[test]
    fn defaults_to_pointer() {
        let window = AppWindowId::default();
        let mut app = TestHost::default();
        assert_eq!(modality(&mut app, Some(window)), InputModality::Pointer);
    }

    #[test]
    fn keydown_sets_keyboard_until_pointer_activity() {
        let window = AppWindowId::default();
        let mut app = TestHost::default();

        assert!(update_for_event(
            &mut app,
            window,
            &Event::KeyDown {
                key: fret_core::KeyCode::KeyA,
                modifiers: Modifiers::default(),
                repeat: false,
            }
        ));
        assert_eq!(modality(&mut app, Some(window)), InputModality::Keyboard);

        assert!(update_for_event(
            &mut app,
            window,
            &Event::Pointer(PointerEvent::Move {
                position: fret_core::Point::new(fret_core::Px(1.0), fret_core::Px(2.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            })
        ));
        assert_eq!(modality(&mut app, Some(window)), InputModality::Pointer);
    }
}
