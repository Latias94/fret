mod pointer_down;
mod pointer_up;

use fret_core::MouseButton;
use fret_ui::UiHost;

use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct SearcherPointerHit {
    pub(super) inside: bool,
    pub(super) row_ix: Option<usize>,
}

pub(super) fn handle_searcher_pointer_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    pointer_down::handle_searcher_pointer_down_event(canvas, cx, position, button, zoom)
}

pub(super) fn handle_searcher_pointer_up_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    pointer_up::handle_searcher_pointer_up_event(canvas, cx, position, button, zoom)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn arm_searcher_row_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        row_ix: usize,
        position: Point,
    ) -> bool {
        super::searcher_activation_state::arm_searcher_row_drag(self, cx, row_ix, position)
    }

    pub(super) fn activate_searcher_hit_or_dismiss<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        hit: SearcherPointerHit,
    ) {
        super::searcher_activation_state::activate_searcher_hit_or_dismiss(self, cx, hit)
    }
}
