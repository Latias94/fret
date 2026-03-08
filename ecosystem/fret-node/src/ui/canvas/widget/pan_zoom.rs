use fret_core::{MouseButton, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn zoom_about_center_factor(&mut self, bounds: fret_core::Rect, factor: f32) {
        super::pan_zoom_zoom::zoom_about_center_factor(self, bounds, factor)
    }

    pub(super) fn zoom_about_pointer_factor(&mut self, position: Point, factor: f32) {
        super::pan_zoom_zoom::zoom_about_pointer_factor(self, position, factor)
    }
}

pub(super) fn begin_panning<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    button: MouseButton,
) -> bool {
    super::pan_zoom_begin::begin_panning(canvas, cx, snapshot, start_pos, button)
}

pub(super) fn handle_panning_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    super::pan_zoom_move::handle_panning_move(canvas, cx, snapshot, position)
}
