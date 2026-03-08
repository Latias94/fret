use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    click_count: u8,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    super::pointer_up_state::sync_pointer_up_state(canvas, position, modifiers);

    if super::pointer_up_state::handle_sticky_wire_ignored_release(canvas, cx, button) {
        return true;
    }

    if super::pointer_up_state::handle_pan_release(canvas, cx, snapshot, button) {
        return true;
    }

    if button != MouseButton::Left {
        return false;
    }

    super::pointer_up_left_route::handle_left_pointer_up(
        canvas,
        cx,
        snapshot,
        position,
        click_count,
        modifiers,
        zoom,
    )
}
