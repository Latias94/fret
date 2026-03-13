mod background;
mod prepare;
mod selection;
mod start;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::{CanvasRect, GroupId};
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_group_resize_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    group: GroupId,
    rect: CanvasRect,
    multi_selection_pressed: bool,
) {
    prepare::clear_for_group_resize(canvas);
    if snapshot.interaction.elements_selectable {
        selection::select_group_for_pointer_down(canvas, cx, group, multi_selection_pressed);
    }

    start::begin_group_resize(canvas, cx, position, group, rect);
}

pub(super) fn handle_group_header_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    group: GroupId,
    rect: CanvasRect,
    multi_selection_pressed: bool,
) {
    prepare::clear_for_group_drag(canvas);
    if snapshot.interaction.elements_selectable {
        selection::select_group_for_pointer_down(canvas, cx, group, multi_selection_pressed);
    }

    start::begin_group_drag(canvas, cx, position, group, rect);
}

pub(super) fn handle_background_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
) {
    prepare::clear_for_background_interaction(canvas);
    background::begin_background_interaction(canvas, cx, snapshot, position, modifiers);
}
