mod pending;
mod threshold;

use fret_core::Point;
use fret_ui::UiHost;

use super::context_menu::opening::ContextMenuOpeningCx;
use super::low_level_adapter::CanvasPointerCaptureReleaseCx;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) use threshold::pending_right_click_exceeded_drag_threshold;

pub(super) trait RightClickCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    ContextMenuOpeningCx<H> + CanvasPointerCaptureReleaseCx<H>
{
}

impl<H, M, T> RightClickCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: ContextMenuOpeningCx<H> + CanvasPointerCaptureReleaseCx<H>,
{
}

pub(super) fn handle_right_click_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl RightClickCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    pending::handle_right_click_pointer_down(canvas, cx, snapshot, position, zoom)
}

pub(super) fn handle_pending_right_click_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl RightClickCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: fret_core::MouseButton,
    zoom: f32,
) -> bool {
    pending::handle_pending_right_click_pointer_up(canvas, cx, snapshot, position, button, zoom)
}
