use fret_core::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PanelKeyboardAction<T> {
    Ignore,
    Select(T),
    Activate(T),
    FocusCanvas,
}

pub(super) fn cyclic_panel_item<T: Copy + Eq>(
    current: Option<T>,
    delta: i32,
    items: &[T],
) -> Option<T> {
    if items.is_empty() {
        return None;
    }

    let len = items.len() as i32;
    let idx0 = current
        .and_then(|item| items.iter().position(|candidate| *candidate == item))
        .unwrap_or(0) as i32;
    let mut next = idx0 + delta;
    next = ((next % len) + len) % len;
    Some(items[next as usize])
}

pub(super) fn panel_keyboard_action<T: Copy + Eq>(
    key: KeyCode,
    current: Option<T>,
    items: &[T],
) -> PanelKeyboardAction<T> {
    match key {
        KeyCode::ArrowDown => cyclic_panel_item(current, 1, items)
            .map(PanelKeyboardAction::Select)
            .unwrap_or(PanelKeyboardAction::Ignore),
        KeyCode::ArrowUp => cyclic_panel_item(current, -1, items)
            .map(PanelKeyboardAction::Select)
            .unwrap_or(PanelKeyboardAction::Ignore),
        KeyCode::Home => items
            .first()
            .copied()
            .map(PanelKeyboardAction::Select)
            .unwrap_or(PanelKeyboardAction::Ignore),
        KeyCode::End => items
            .last()
            .copied()
            .map(PanelKeyboardAction::Select)
            .unwrap_or(PanelKeyboardAction::Ignore),
        KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => current
            .or_else(|| items.first().copied())
            .map(PanelKeyboardAction::Activate)
            .unwrap_or(PanelKeyboardAction::Ignore),
        KeyCode::Escape => PanelKeyboardAction::FocusCanvas,
        _ => PanelKeyboardAction::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::{PanelKeyboardAction, cyclic_panel_item, panel_keyboard_action};
    use fret_core::KeyCode;

    #[test]
    fn cyclic_panel_item_wraps_in_both_directions() {
        let items = [1_u8, 2, 3];
        assert_eq!(cyclic_panel_item(Some(3), 1, &items), Some(1));
        assert_eq!(cyclic_panel_item(Some(1), -1, &items), Some(3));
        assert_eq!(cyclic_panel_item(None, 1, &items), Some(2));
    }

    #[test]
    fn panel_keyboard_action_routes_navigation_activation_and_escape() {
        let items = [10_u8, 20, 30];

        assert_eq!(
            panel_keyboard_action(KeyCode::ArrowDown, Some(10), &items),
            PanelKeyboardAction::Select(20)
        );
        assert_eq!(
            panel_keyboard_action(KeyCode::Home, Some(30), &items),
            PanelKeyboardAction::Select(10)
        );
        assert_eq!(
            panel_keyboard_action(KeyCode::End, Some(10), &items),
            PanelKeyboardAction::Select(30)
        );
        assert_eq!(
            panel_keyboard_action(KeyCode::Enter, Some(20), &items),
            PanelKeyboardAction::Activate(20)
        );
        assert_eq!(
            panel_keyboard_action(KeyCode::Space, None, &items),
            PanelKeyboardAction::Activate(10)
        );
        assert_eq!(
            panel_keyboard_action(KeyCode::Escape, Some(10), &items),
            PanelKeyboardAction::FocusCanvas
        );
        assert_eq!(
            panel_keyboard_action(KeyCode::KeyA, Some(10), &items),
            PanelKeyboardAction::Ignore
        );
    }

    #[test]
    fn panel_keyboard_action_ignores_empty_rosters_except_escape() {
        let items: [u8; 0] = [];
        assert_eq!(
            panel_keyboard_action(KeyCode::ArrowDown, None, &items),
            PanelKeyboardAction::Ignore
        );
        assert_eq!(
            panel_keyboard_action(KeyCode::Enter, None, &items),
            PanelKeyboardAction::Ignore
        );
        assert_eq!(
            panel_keyboard_action(KeyCode::Escape, None, &items),
            PanelKeyboardAction::FocusCanvas
        );
    }
}
