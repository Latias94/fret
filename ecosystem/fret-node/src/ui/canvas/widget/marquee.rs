use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, marquee_cx::MarqueeCx};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn begin_background_marquee<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    pos: Point,
    modifiers: Modifiers,
    clear_selection_on_up: bool,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: MarqueeCx<H>,
{
    super::marquee_begin::begin_background_marquee(
        canvas,
        cx,
        snapshot,
        pos,
        modifiers,
        clear_selection_on_up,
    )
}

pub(super) fn handle_marquee_move<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: MarqueeCx<H>,
{
    if super::marquee_selection::update_active_marquee(canvas, cx, snapshot, position) {
        return true;
    }

    super::marquee_pending::handle_pending_marquee(canvas, cx, snapshot, position, modifiers, zoom)
}

pub(super) fn handle_left_up<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: MarqueeCx<H>,
{
    super::marquee_finish::handle_left_up(canvas, cx)
}
