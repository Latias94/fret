mod activation_retained_cx;
mod dispatch;

use fret_core::{KeyCode, Modifiers};

use super::widget_tail::WidgetHandledCx;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SearcherStepDirection {
    Forward,
    Backward,
}

pub(in super::super) trait SearcherInputCx<H, M: NodeGraphCanvasMiddleware>:
    WidgetHandledCx<H>
{
    fn try_activate_searcher_row(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        row_ix: usize,
    ) -> bool;
}

pub(super) fn handle_searcher_key_down_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherInputCx<H, M>,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    dispatch::handle_searcher_key_down_event(canvas, cx, key, modifiers)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn try_activate_active_searcher_row<H>(
        &mut self,
        cx: &mut impl SearcherInputCx<H, M>,
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
