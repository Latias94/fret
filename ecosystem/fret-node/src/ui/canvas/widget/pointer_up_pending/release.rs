mod group;
mod node;

use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, low_level_adapter};

pub(in super::super) fn handle_pending_group_drag_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: low_level_adapter::CanvasPointerCaptureReleaseCx<H>,
{
    group::handle_pending_group_drag_release(canvas, cx)
}

pub(in super::super) fn handle_pending_group_resize_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: low_level_adapter::CanvasPointerCaptureReleaseCx<H>,
{
    group::handle_pending_group_resize_release(canvas, cx)
}

pub(in super::super) fn handle_pending_node_resize_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: low_level_adapter::CanvasPointerCaptureReleaseCx<H>,
{
    node::handle_pending_node_resize_release(canvas, cx)
}
