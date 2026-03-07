use super::*;

pub(super) fn dispatch_pointer_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) {
    if pan_zoom::handle_panning_move(canvas, cx, snapshot, position) {
    } else if marquee::handle_marquee_move(canvas, cx, snapshot, position, modifiers, zoom) {
    } else if pending_group_drag::handle_pending_group_drag_move(
        canvas, cx, snapshot, position, zoom,
    ) {
    } else if group_drag::handle_group_drag_move(canvas, cx, snapshot, position, modifiers, zoom) {
    } else if pending_group_resize::handle_pending_group_resize_move(
        canvas, cx, snapshot, position, zoom,
    ) {
    } else if group_resize::handle_group_resize_move(
        canvas, cx, snapshot, position, modifiers, zoom,
    ) {
    } else if pending_drag::handle_pending_node_drag_move(canvas, cx, snapshot, position, zoom) {
    } else if pending_resize::handle_pending_node_resize_move(canvas, cx, snapshot, position, zoom)
    {
    } else if pending_wire_drag::handle_pending_wire_drag_move(
        canvas, cx, snapshot, position, modifiers, zoom,
    ) {
    } else if edge_insert_drag::handle_pending_edge_insert_drag_move(canvas, cx, snapshot, position)
    {
    } else if node_resize::handle_node_resize_move(canvas, cx, snapshot, position, modifiers, zoom)
    {
    } else if node_drag::handle_node_drag_move(canvas, cx, snapshot, position, modifiers, zoom) {
    } else if wire_drag::handle_wire_drag_move(canvas, cx, snapshot, position, modifiers, zoom) {
    } else if edge_insert_drag::handle_edge_insert_drag_move(canvas, cx, position) {
    } else if edge_drag::handle_edge_drag_move(canvas, cx, snapshot, position, zoom) {
    } else if insert_node_drag::handle_pending_insert_node_drag_move(
        canvas, cx, snapshot, position, buttons, zoom,
    ) {
    } else if searcher::handle_searcher_pointer_move(canvas, cx, position, zoom) {
    } else if context_menu::handle_context_menu_pointer_move(canvas, cx, position, zoom) {
    } else {
        hover::update_hover_edge(canvas, cx, snapshot, position, zoom);
    }
}
