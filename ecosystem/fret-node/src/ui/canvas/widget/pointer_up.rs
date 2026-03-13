mod left;
mod release;
mod state;

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
    state::sync_pointer_up_state(canvas, position, modifiers);

    if release::handle_non_left_releases(canvas, cx, snapshot, button) {
        return true;
    }

    if button != MouseButton::Left {
        return false;
    }

    left::handle_left_pointer_up(canvas, cx, snapshot, position, click_count, modifiers, zoom)
}
