mod pointer_down;
mod pointer_up;

use fret_core::{MouseButton, Point};

use super::searcher_activation_state::SearcherReleaseCx;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) use pointer_down::SearcherPointerDownCx;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct SearcherPointerHit {
    pub(super) inside: bool,
    pub(super) row_ix: Option<usize>,
}

pub(super) fn handle_searcher_pointer_down_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherPointerDownCx<H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    pointer_down::handle_searcher_pointer_down_event(canvas, cx, position, button, zoom)
}

pub(super) fn handle_searcher_pointer_up_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherReleaseCx<H, M>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    pointer_up::handle_searcher_pointer_up_event(canvas, cx, position, button, zoom)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn arm_searcher_row_drag(
        &mut self,
        cx: &mut impl super::searcher_activation_state::SearcherArmCx,
        row_ix: usize,
        position: Point,
    ) -> bool {
        super::searcher_activation_state::arm_searcher_row_drag(self, cx, row_ix, position)
    }

    pub(super) fn activate_searcher_hit_or_dismiss<H>(
        &mut self,
        cx: &mut impl super::searcher_activation_state::SearcherReleaseCx<H, M>,
        hit: SearcherPointerHit,
    ) {
        super::searcher_activation_state::activate_searcher_hit_or_dismiss(self, cx, hit)
    }
}
