use crate::UiHost;
use fret_core::{AppWindowId, Event, KeyCode, PointerEvent};
use std::collections::HashMap;

#[derive(Default)]
struct FocusVisibleState {
    per_window: HashMap<AppWindowId, bool>,
}

fn is_navigation_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::Tab
            | KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageUp
            | KeyCode::PageDown
    )
}

pub fn is_focus_visible<H: UiHost>(app: &mut H, window: Option<AppWindowId>) -> bool {
    let Some(window) = window else {
        return false;
    };
    let Some(state) = app.global::<FocusVisibleState>() else {
        return false;
    };
    state.per_window.get(&window).copied().unwrap_or(false)
}

fn set_focus_visible_if_changed<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    visible: bool,
) -> bool {
    app.with_global_mut_untracked(FocusVisibleState::default, |state, _app| {
        let prev = state.per_window.get(&window).copied().unwrap_or(false);
        if prev != visible {
            state.per_window.insert(window, visible);
            true
        } else {
            false
        }
    })
}

pub fn update_for_event<H: UiHost>(app: &mut H, window: AppWindowId, event: &Event) -> bool {
    match event {
        Event::Pointer(PointerEvent::Down { .. }) => {
            set_focus_visible_if_changed(app, window, false)
        }
        Event::KeyDown { key, .. } if is_navigation_key(*key) => {
            set_focus_visible_if_changed(app, window, true)
        }
        _ => false,
    }
}
