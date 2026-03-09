use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::{CanvasRect, GroupId};
use crate::ui::canvas::state::{PendingGroupDrag, PendingGroupResize, ViewSnapshot};
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
    clear_for_group_resize(canvas);
    if snapshot.interaction.elements_selectable {
        select_group_for_pointer_down(canvas, cx, group, multi_selection_pressed);
    }

    canvas.interaction.pending_group_resize = Some(PendingGroupResize {
        group,
        start_pos: position,
        start_rect: rect,
    });
    canvas.interaction.group_resize = None;

    cx.capture_pointer(cx.node);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
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
    clear_for_group_drag(canvas);
    if snapshot.interaction.elements_selectable {
        select_group_for_pointer_down(canvas, cx, group, multi_selection_pressed);
    }

    canvas.interaction.pending_group_drag = Some(PendingGroupDrag {
        group,
        start_pos: position,
        start_rect: rect,
    });
    canvas.interaction.group_drag = None;
    canvas.interaction.pending_group_resize = None;
    canvas.interaction.group_resize = None;

    cx.capture_pointer(cx.node);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}

pub(super) fn handle_background_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
) {
    clear_for_background_interaction(canvas);
    if snapshot.interaction.elements_selectable {
        crate::ui::canvas::widget::marquee::begin_background_marquee(
            canvas, cx, snapshot, position, modifiers, true,
        );
    } else if snapshot.interaction.pan_on_drag.left {
        let _ = crate::ui::canvas::widget::pan_zoom::begin_panning(
            canvas,
            cx,
            snapshot,
            position,
            fret_core::MouseButton::Left,
        );
    }
}

fn clear_for_group_resize<M: NodeGraphCanvasMiddleware>(canvas: &mut NodeGraphCanvasWith<M>) {
    canvas.interaction.pending_group_drag = None;
    canvas.interaction.group_drag = None;
    canvas.interaction.pending_node_drag = None;
    canvas.interaction.node_drag = None;
    canvas.interaction.pending_node_resize = None;
    canvas.interaction.node_resize = None;
    canvas.interaction.pending_wire_drag = None;
    canvas.interaction.wire_drag = None;
    canvas.interaction.click_connect = false;
    canvas.interaction.edge_drag = None;
    canvas.interaction.pending_edge_insert_drag = None;
    canvas.interaction.edge_insert_drag = None;
    canvas.interaction.pending_marquee = None;
    canvas.interaction.marquee = None;
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(&mut canvas.interaction);
}

fn clear_for_group_drag<M: NodeGraphCanvasMiddleware>(canvas: &mut NodeGraphCanvasWith<M>) {
    canvas.interaction.pending_node_drag = None;
    canvas.interaction.node_drag = None;
    canvas.interaction.pending_node_resize = None;
    canvas.interaction.node_resize = None;
    canvas.interaction.pending_wire_drag = None;
    canvas.interaction.wire_drag = None;
    canvas.interaction.click_connect = false;
    canvas.interaction.edge_drag = None;
    canvas.interaction.pending_edge_insert_drag = None;
    canvas.interaction.edge_insert_drag = None;
    canvas.interaction.pending_marquee = None;
    canvas.interaction.marquee = None;
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(&mut canvas.interaction);
}

fn clear_for_background_interaction<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    canvas.interaction.edge_drag = None;
    canvas.interaction.pending_edge_insert_drag = None;
    canvas.interaction.edge_insert_drag = None;
    canvas.interaction.pending_group_drag = None;
    canvas.interaction.group_drag = None;
    canvas.interaction.pending_group_resize = None;
    canvas.interaction.group_resize = None;
    canvas.interaction.pending_node_drag = None;
    canvas.interaction.node_drag = None;
    canvas.interaction.pending_node_resize = None;
    canvas.interaction.node_resize = None;
    canvas.interaction.pending_wire_drag = None;
    canvas.interaction.wire_drag = None;
    canvas.interaction.click_connect = false;
    canvas.interaction.pending_marquee = None;
    canvas.interaction.marquee = None;
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(&mut canvas.interaction);
}

fn select_group_for_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    group: GroupId,
    multi_selection_pressed: bool,
) {
    canvas.update_view_state(cx.app, |s| {
        if multi_selection_pressed {
            if let Some(ix) = s.selected_groups.iter().position(|id| *id == group) {
                s.selected_groups.remove(ix);
            } else {
                s.selected_groups.push(group);
            }
        } else if !s.selected_groups.iter().any(|id| *id == group) {
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_groups.push(group);
        }
        s.group_draw_order.retain(|id| *id != group);
        s.group_draw_order.push(group);
    });
}
