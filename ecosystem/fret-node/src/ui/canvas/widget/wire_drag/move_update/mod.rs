mod auto_pan;
mod bundle;
mod hover;
mod prelude;

use prelude::*;

pub(in super::super) fn handle_wire_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(mut w) = canvas.interaction.wire_drag.take() else {
        return false;
    };

    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    auto_pan::update_wire_pos_and_auto_pan(canvas, cx, snapshot, position, &mut w);

    let pos = w.pos;

    bundle::maybe_extend_bundle_on_shift(
        canvas,
        cx.app,
        snapshot,
        modifiers,
        zoom,
        geom.as_ref(),
        index.as_ref(),
        pos,
        &mut w.kind,
    );

    let (from_port, require_from_connectable_start) =
        hover::from_port_and_require_from_connectable_start(&w.kind);
    let new_hover = hover::pick_hover_port(
        canvas,
        cx.app,
        snapshot,
        geom.as_ref(),
        index.as_ref(),
        zoom,
        from_port,
        require_from_connectable_start,
        pos,
    );
    let new_hover_edge = hover::pick_hover_edge_if_no_hover_port(
        canvas,
        cx.app,
        snapshot,
        geom.as_ref(),
        index.as_ref(),
        zoom,
        pos,
        new_hover,
    );
    let (new_hover_valid, new_hover_diag) =
        hover::compute_hover_validity_and_diag(canvas, cx.app, snapshot, &w.kind, new_hover);
    let new_hover_convertible = hover::compute_hover_convertible(
        canvas,
        cx.app,
        snapshot,
        &w.kind,
        new_hover,
        new_hover_valid,
    );

    if canvas.interaction.hover_port != new_hover
        || canvas.interaction.hover_port_valid != new_hover_valid
        || canvas.interaction.hover_port_convertible != new_hover_convertible
        || canvas.interaction.hover_port_diagnostic != new_hover_diag
    {
        canvas.interaction.hover_port = new_hover;
        canvas.interaction.hover_port_valid = new_hover_valid;
        canvas.interaction.hover_port_convertible = new_hover_convertible;
        canvas.interaction.hover_port_diagnostic = new_hover_diag;
    }

    canvas.interaction.hover_edge = new_hover_edge;
    canvas.interaction.wire_drag = Some(w);
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    true
}
