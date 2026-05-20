use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;
use fret_ui::UiHost;

pub(super) fn handle_non_left_releases<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::super::pointer_up_release_cx::PointerUpReleaseCx<H>,
    snapshot: &ViewSnapshot,
    button: fret_core::MouseButton,
) -> bool {
    super::super::pointer_up_state::handle_sticky_wire_ignored_release(canvas, cx, button)
        || super::super::pointer_up_state::handle_pan_release(canvas, cx, snapshot, button)
}
