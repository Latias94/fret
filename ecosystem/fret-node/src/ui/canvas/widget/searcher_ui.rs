mod event;
mod overlay;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn install_searcher_overlay(&mut self, searcher: SearcherState) {
        overlay::install_searcher_overlay(self, searcher);
    }

    pub(super) fn open_searcher_overlay(
        &mut self,
        invoked_at: Point,
        bounds: Rect,
        snapshot: &ViewSnapshot,
        target: ContextMenuTarget,
        candidates: Vec<InsertNodeCandidate>,
        rows_mode: SearcherRowsMode,
    ) {
        overlay::open_searcher_overlay(
            self, invoked_at, bounds, snapshot, target, candidates, rows_mode,
        );
    }

    pub(super) fn dismiss_searcher_overlay<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        super::searcher_activation_state::dismiss_searcher_overlay(self, cx);
    }
}

pub(super) fn dismiss_searcher_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    event::dismiss_searcher_event(canvas, cx)
}

pub(super) fn take_searcher_overlay(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> Option<SearcherState> {
    overlay::take_searcher_overlay(interaction)
}

pub(super) fn restore_searcher_overlay(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    searcher: SearcherState,
) {
    overlay::restore_searcher_overlay(interaction, searcher);
}

pub(super) fn handle_searcher_escape_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    event::handle_searcher_escape_event(canvas, cx)
}

pub(super) fn invalidate_searcher_paint<H: UiHost>(
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    event::invalidate_searcher_paint(cx);
}

pub(super) fn finish_searcher_event<H: UiHost>(
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    event::finish_searcher_event(cx)
}
