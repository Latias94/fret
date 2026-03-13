mod dispatch;

use fret_core::{KeyCode, Modifiers};
use fret_ui::UiHost;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SearcherStepDirection {
    Forward,
    Backward,
}

pub(super) fn handle_searcher_key_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    dispatch::handle_searcher_key_down_event(canvas, cx, key, modifiers)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn try_activate_active_searcher_row<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
    ) -> bool {
        super::searcher_input_query::try_activate_active_searcher_row(self, cx)
    }

    pub(super) fn step_searcher_active_row(&mut self, direction: SearcherStepDirection) -> bool {
        super::searcher_input_nav::step_searcher_active_row(self, direction)
    }

    pub(super) fn update_searcher_query_from_key(
        &mut self,
        key: KeyCode,
        modifiers: Modifiers,
    ) -> bool {
        super::searcher_input_query::update_searcher_query_from_key(self, key, modifiers)
    }
}
