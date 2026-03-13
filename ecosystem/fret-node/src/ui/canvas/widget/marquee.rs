use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn begin_background_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    pos: Point,
    modifiers: Modifiers,
    clear_selection_on_up: bool,
) {
    super::marquee_begin::begin_background_marquee(
        canvas,
        cx,
        snapshot,
        pos,
        modifiers,
        clear_selection_on_up,
    )
}

pub(super) fn handle_marquee_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    if super::marquee_selection::update_active_marquee(canvas, cx, snapshot, position) {
        return true;
    }

    super::marquee_pending::handle_pending_marquee(canvas, cx, snapshot, position, modifiers, zoom)
}

pub(super) fn handle_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    super::marquee_finish::handle_left_up(canvas, cx)
}
