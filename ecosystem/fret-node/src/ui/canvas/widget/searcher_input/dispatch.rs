use fret_core::{KeyCode, Modifiers};
use fret_ui::UiHost;

use super::super::searcher_ui::finish_searcher_event;
use super::super::*;

pub(super) fn handle_searcher_key_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    if matches!(key, KeyCode::Enter | KeyCode::NumpadEnter)
        && canvas.try_activate_active_searcher_row(cx)
    {
        return finish_searcher_event(cx);
    }

    if canvas.interaction.searcher.is_none() {
        return false;
    }

    match key {
        KeyCode::ArrowDown => {
            let _ = canvas.step_searcher_active_row(super::SearcherStepDirection::Forward);
            return finish_searcher_event(cx);
        }
        KeyCode::ArrowUp => {
            let _ = canvas.step_searcher_active_row(super::SearcherStepDirection::Backward);
            return finish_searcher_event(cx);
        }
        _ => {}
    }

    if canvas.update_searcher_query_from_key(key, modifiers) {
        return finish_searcher_event(cx);
    }

    false
}
