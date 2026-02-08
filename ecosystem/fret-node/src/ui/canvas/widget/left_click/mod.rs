use fret_core::{Modifiers, Point, Rect};
use fret_ui::UiHost;

mod handlers;
mod hit;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

fn node_header_hit(rect: Rect, header_height_screen: f32, zoom: f32, position: Point) -> bool {
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let header_h = (header_height_screen / zoom).max(0.0);
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = rect.origin.x.0 + rect.size.width.0;
    let y1 = rect.origin.y.0 + header_h.min(rect.size.height.0.max(0.0));

    position.x.0 >= x0
        && position.y.0 >= y0
        && position.x.0 <= x1
        && position.y.0 <= y1
        && header_h > 0.0
}

pub(super) fn handle_left_click_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    canvas.interaction.hover_edge = None;

    use hit::Hit;
    let hit = hit::compute_hit(canvas, cx, snapshot, position, zoom);

    let selection_key_pressed = snapshot.interaction.selection_key.is_pressed(modifiers);
    canvas.interaction.multi_selection_active = snapshot
        .interaction
        .multi_selection_key
        .is_pressed(modifiers);
    let multi_selection_pressed = canvas.interaction.multi_selection_active;

    if snapshot.interaction.elements_selectable
        && selection_key_pressed
        && !matches!(
            hit,
            Hit::Port(_) | Hit::EdgeAnchor(_, _, _) | Hit::Resize(_, _, _) | Hit::GroupResize(_, _)
        )
        && !matches!(hit, Hit::Background)
    {
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
        canvas.interaction.focused_edge = None;
        canvas.interaction.hover_port = None;
        canvas.interaction.hover_port_valid = false;
        canvas.interaction.hover_port_convertible = false;
        canvas.interaction.hover_port_diagnostic = None;

        super::marquee::begin_background_marquee(canvas, cx, snapshot, position, modifiers, false);
        return true;
    }

    handlers::handle_hit(
        canvas,
        cx,
        snapshot,
        position,
        modifiers,
        zoom,
        hit,
        multi_selection_pressed,
    )
}
