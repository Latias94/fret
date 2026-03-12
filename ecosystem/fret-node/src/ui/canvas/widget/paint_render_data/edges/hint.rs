use super::super::*;

pub(super) fn resolve_edge_render_hint<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    edge_id: EdgeId,
    selected: bool,
    hovered: bool,
) -> EdgeRenderHint {
    let hint = EdgePathContext::new(
        &canvas.style,
        &*canvas.presenter,
        canvas.edge_types.as_ref(),
    )
    .edge_render_hint_normalized(graph, edge_id);
    if let Some(skin) = canvas.skin.as_ref() {
        skin.edge_render_hint(graph, edge_id, &canvas.style, &hint, selected, hovered)
            .normalized()
    } else {
        hint
    }
}

pub(super) fn apply_edge_paint_override<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    edge_id: EdgeId,
    mut hint: EdgeRenderHint,
) -> (
    EdgeRenderHint,
    Option<crate::ui::paint_overrides::EdgePaintOverrideV1>,
) {
    let paint_override = canvas
        .paint_overrides
        .as_ref()
        .and_then(|o| o.edge_paint_override(edge_id));
    if let Some(ov) = paint_override {
        if let Some(dash) = ov.dash {
            hint.dash = Some(dash);
        }
        if let Some(width_mul) = ov.stroke_width_mul {
            hint.width_mul *= width_mul;
        }
    }
    (hint.normalized(), paint_override)
}
