use super::super::support::stable_hash_u64;
use super::*;

fn outline_hint(
    interaction_hint: crate::ui::InteractionChromeHint,
    edge_selected: bool,
) -> Option<crate::ui::WireOutlineHint> {
    if edge_selected {
        interaction_hint.wire_outline_selected
    } else {
        interaction_hint.wire_outline_base
    }
}

fn outline_path_from_custom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    custom: &crate::ui::edge_types::EdgeCustomPath,
    edge_id: EdgeId,
    outline_width: f32,
    dash: Option<DashPatternV1>,
    zoom: f32,
) -> Option<fret_core::PathId> {
    let dash_key = dash.map(|p| (p.dash.0.to_bits(), p.gap.0.to_bits(), p.phase.0.to_bits()));
    let key = (custom.cache_key, edge_id, outline_width.to_bits(), dash_key);
    let outline_cache_key = stable_hash_u64(1, &key);
    canvas.paint_cache.wire_path_from_commands(
        cx.services,
        outline_cache_key,
        &custom.commands,
        zoom,
        cx.scale_factor,
        outline_width,
        dash,
    )
}

fn outline_path_from_route<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    outline_width: f32,
    dash: Option<DashPatternV1>,
    zoom: f32,
) -> Option<fret_core::PathId> {
    canvas.paint_cache.wire_path(
        cx.services,
        route,
        from,
        to,
        zoom,
        cx.scale_factor,
        outline_width,
        dash,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_edge_outline<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    custom: Option<&crate::ui::edge_types::EdgeCustomPath>,
    interaction_hint: crate::ui::InteractionChromeHint,
    edge_selected: bool,
    edge_id: EdgeId,
    from: Point,
    to: Point,
    route: EdgeRouteKind,
    width: f32,
    dash: Option<DashPatternV1>,
    zoom: f32,
    outline_budget: &mut WorkBudget,
    outline_budget_skipped: &mut u32,
) {
    let Some(outline) = outline_hint(interaction_hint, edge_selected) else {
        return;
    };
    if !outline.width_mul.is_finite() || outline.width_mul <= 1.0e-3 || outline.color.a <= 0.0 {
        return;
    }
    if !outline_budget.try_consume(1) {
        *outline_budget_skipped = outline_budget_skipped.saturating_add(1);
        return;
    }

    let outline_width = width * outline.width_mul.max(0.0);
    let path = if let Some(custom) = custom {
        outline_path_from_custom(canvas, cx, custom, edge_id, outline_width, dash, zoom)
    } else {
        outline_path_from_route(canvas, cx, route, from, to, outline_width, dash, zoom)
    };

    if let Some(path) = path {
        cx.scene.push(SceneOp::Path {
            order: DrawOrder(2),
            origin: Point::new(Px(0.0), Px(0.0)),
            path,
            paint: outline.color.into(),
        });
    }
}
