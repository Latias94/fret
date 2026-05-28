use super::*;

#[derive(Debug, Clone, Default)]
pub(super) struct DerivedGeometryCacheState {
    pub(super) key: Option<CanvasKey>,
    pub(super) rebuilds: u64,
    pub(super) geom: Option<Arc<CanvasGeometry>>,
    pub(super) index: Option<Arc<CanvasSpatialDerived>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DerivedGeometryCacheKeyV2 {
    graph_rev: u64,
    zoom_q: i32,
    node_origin_x_q: i32,
    node_origin_y_q: i32,
    draw_order_hash: u64,
    presenter_rev: u64,
    geometry_tokens_fingerprint: u64,
    geometry_overrides_rev: u64,
    cell_size_screen_bits: u32,
    min_cell_size_screen_bits: u32,
    edge_aabb_pad_screen_bits: u32,
    edge_interaction_width_bits: u32,
    wire_width_bits: u32,
}

#[derive(Debug, Clone, Default)]
pub(super) struct EdgePaintCacheState {
    pub(super) key: Option<CanvasKey>,
    pub(super) rebuilds: u64,
    pub(super) draws: Option<Arc<Vec<EdgePathDraw>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgePaintCacheKeyV4 {
    graph_rev: u64,
    zoom_q: i32,
    node_origin_x_q: i32,
    node_origin_y_q: i32,
    draw_order_hash: u64,
    derived_geometry_key: u64,
    edge_types_rev: u64,
    skin_rev: u64,
}

#[derive(Debug, Clone)]
pub(super) struct EdgePathDraw {
    pub(super) edge: crate::core::EdgeId,
    pub(super) from: crate::core::PortId,
    pub(super) to: crate::core::PortId,
    pub(super) key: u64,
    pub(super) commands: Box<[PathCommand]>,
    pub(super) bbox: Rect,
    pub(super) color: Color,
    pub(super) route: EdgeRouteKind,
    pub(super) width_mul: f32,
    pub(super) dash: Option<DashPatternV1>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct NodePaintCacheState {
    pub(super) key: Option<CanvasKey>,
    pub(super) rebuilds: u64,
    pub(super) draws: Option<Arc<Vec<NodeRectDraw>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodePaintCacheKeyV3 {
    graph_rev: u64,
    zoom_q: i32,
    node_origin_x_q: i32,
    node_origin_y_q: i32,
    draw_order_hash: u64,
    derived_geometry_key: u64,
}

#[derive(Debug, Clone)]
pub(super) struct NodeRectDraw {
    pub(super) id: crate::core::NodeId,
    pub(super) rect: Rect,
}

pub(super) fn grid_cache_key(bounds: Rect, view: PanZoom2D, style: &NodeGraphStyle) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0);
    let bg = style.paint.background;
    let grid = style.paint.grid_minor_color;

    let step = 64.0f32;
    let (ix0, ix1, iy0, iy1) = if bounds.size.width.0.is_finite()
        && bounds.size.height.0.is_finite()
        && bounds.size.width.0 > 0.0
        && bounds.size.height.0 > 0.0
        && step.is_finite()
        && step > 0.0
    {
        let tl = view.screen_to_canvas(bounds, bounds.origin);
        let br = view.screen_to_canvas(
            bounds,
            Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0),
                Px(bounds.origin.y.0 + bounds.size.height.0),
            ),
        );
        let x0 = tl.x.0.min(br.x.0);
        let x1 = tl.x.0.max(br.x.0);
        let y0 = tl.y.0.min(br.y.0);
        let y1 = tl.y.0.max(br.y.0);
        if x0.is_finite() && x1.is_finite() && y0.is_finite() && y1.is_finite() {
            (
                (x0 / step).floor() as i32 - 1,
                (x1 / step).ceil() as i32 + 1,
                (y0 / step).floor() as i32 - 1,
                (y1 / step).ceil() as i32 + 1,
            )
        } else {
            (0, 0, 0, 0)
        }
    } else {
        (0, 0, 0, 0)
    };
    let v = GridPaintCacheKeyV2 {
        bounds_x_q: quantize_f32(bounds.origin.x.0, 8.0),
        bounds_y_q: quantize_f32(bounds.origin.y.0, 8.0),
        bounds_w_q: quantize_f32(bounds.size.width.0, 8.0),
        bounds_h_q: quantize_f32(bounds.size.height.0, 8.0),
        zoom_q: quantize_f32(zoom, 4096.0),
        ix0,
        ix1,
        iy0,
        iy1,
        bg_r_bits: bg.r.to_bits(),
        bg_g_bits: bg.g.to_bits(),
        bg_b_bits: bg.b.to_bits(),
        bg_a_bits: bg.a.to_bits(),
        grid_r_bits: grid.r.to_bits(),
        grid_g_bits: grid.g.to_bits(),
        grid_b_bits: grid.b.to_bits(),
        grid_a_bits: grid.a.to_bits(),
    };

    CanvasKey::from_hash(&("fret-node.grid.paint-only.v2", v))
}

pub(super) fn declarative_presenter_revision(
    measured_geometry: Option<&Arc<MeasuredGeometryStore>>,
) -> u64 {
    measured_geometry
        .map(|measured| {
            MeasuredNodeGraphPresenter::new(DefaultNodeGraphPresenter::default(), measured.clone())
                .geometry_revision()
        })
        .unwrap_or(0)
}

pub(super) fn derived_geometry_cache_key(
    graph_rev: u64,
    zoom: f32,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order: &[crate::core::NodeId],
    interaction: &crate::io::NodeGraphInteractionState,
    style: &NodeGraphStyle,
    presenter_rev: u64,
    geometry_overrides_rev: u64,
    max_edge_interaction_width_override_px: f32,
) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0);
    let origin = node_origin.normalized();

    let tuning = interaction.spatial_index;
    let max_edge_interaction_width_override_px = if max_edge_interaction_width_override_px
        .is_finite()
        && max_edge_interaction_width_override_px >= 0.0
    {
        max_edge_interaction_width_override_px
    } else {
        0.0
    };
    let edge_aabb_pad_screen_px = tuning
        .edge_aabb_pad_screen_px
        .max(interaction.edge_interaction_width)
        .max(max_edge_interaction_width_override_px)
        .max(style.geometry.wire_width)
        .max(0.0);

    let v = DerivedGeometryCacheKeyV2 {
        graph_rev,
        zoom_q: quantize_f32(zoom, 4096.0),
        node_origin_x_q: quantize_f32(origin.x, 4096.0),
        node_origin_y_q: quantize_f32(origin.y, 4096.0),
        draw_order_hash: stable_hash_u64(1, &draw_order),
        presenter_rev,
        geometry_tokens_fingerprint: style.geometry.fingerprint(),
        geometry_overrides_rev,
        cell_size_screen_bits: tuning.cell_size_screen_px.to_bits(),
        min_cell_size_screen_bits: tuning.min_cell_size_screen_px.to_bits(),
        edge_aabb_pad_screen_bits: edge_aabb_pad_screen_px.to_bits(),
        edge_interaction_width_bits: interaction.edge_interaction_width.to_bits(),
        wire_width_bits: style.geometry.wire_width.to_bits(),
    };

    CanvasKey::from_hash(&("fret-node.derived-geometry.paint-only.v3", v))
}

fn build_debug_grid_ops(
    bounds: Rect,
    view: PanZoom2D,
    style: &NodeGraphStyle,
) -> Arc<Vec<fret_core::SceneOp>> {
    let mut ops = Vec::<fret_core::SceneOp>::new();

    ops.push(fret_core::SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: fret_core::scene::Paint::Solid(style.paint.background).into(),
        border: fret_core::Edges::default(),
        border_paint: fret_core::scene::Paint::TRANSPARENT.into(),
        corner_radii: fret_core::Corners::default(),
    });

    let Some(transform) = view.render_transform(bounds) else {
        return Arc::new(ops);
    };

    let push_transform = {
        let is_finite = transform.a.is_finite()
            && transform.b.is_finite()
            && transform.c.is_finite()
            && transform.d.is_finite()
            && transform.tx.is_finite()
            && transform.ty.is_finite();
        is_finite && transform != fret_core::Transform2D::IDENTITY
    };

    if push_transform {
        ops.push(fret_core::SceneOp::PushTransform { transform });
    }

    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let step = 64.0f32;
    let thickness = (1.0 / zoom).max(0.25 / zoom);

    let tl = view.screen_to_canvas(bounds, bounds.origin);
    let br = view.screen_to_canvas(
        bounds,
        Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0),
            Px(bounds.origin.y.0 + bounds.size.height.0),
        ),
    );

    let x0 = tl.x.0.min(br.x.0);
    let x1 = tl.x.0.max(br.x.0);
    let y0 = tl.y.0.min(br.y.0);
    let y1 = tl.y.0.max(br.y.0);

    let ix0 = (x0 / step).floor() as i32 - 1;
    let ix1 = (x1 / step).ceil() as i32 + 1;
    let iy0 = (y0 / step).floor() as i32 - 1;
    let iy1 = (y1 / step).ceil() as i32 + 1;

    let x0 = ix0 as f32 * step;
    let x1 = ix1 as f32 * step;
    let y0 = iy0 as f32 * step;
    let y1 = iy1 as f32 * step;

    let max_lines = 400i32;
    let x_lines = (ix1 - ix0).clamp(0, max_lines);
    let y_lines = (iy1 - iy0).clamp(0, max_lines);

    let grid_paint: fret_core::scene::PaintBindingV1 =
        fret_core::scene::Paint::Solid(style.paint.grid_minor_color).into();

    for i in 0..x_lines {
        let x = (ix0 + i) as f32 * step;
        let rect = Rect::new(
            Point::new(Px(x - 0.5 * thickness), Px(y0)),
            fret_core::Size::new(Px(thickness), Px(y1 - y0)),
        );
        ops.push(fret_core::SceneOp::Quad {
            order: DrawOrder(1),
            rect,
            background: grid_paint,
            border: fret_core::Edges::default(),
            border_paint: fret_core::scene::Paint::TRANSPARENT.into(),
            corner_radii: fret_core::Corners::default(),
        });
    }

    for i in 0..y_lines {
        let y = (iy0 + i) as f32 * step;
        let rect = Rect::new(
            Point::new(Px(x0), Px(y - 0.5 * thickness)),
            fret_core::Size::new(Px(x1 - x0), Px(thickness)),
        );
        ops.push(fret_core::SceneOp::Quad {
            order: DrawOrder(1),
            rect,
            background: grid_paint,
            border: fret_core::Edges::default(),
            border_paint: fret_core::scene::Paint::TRANSPARENT.into(),
            corner_radii: fret_core::Corners::default(),
        });
    }

    if push_transform {
        ops.push(fret_core::SceneOp::PopTransform);
    }

    Arc::new(ops)
}

pub(super) fn paint_debug_grid_cached(
    p: &mut CanvasPainter<'_>,
    view: PanZoom2D,
    ops: Option<Arc<Vec<fret_core::SceneOp>>>,
    style: &NodeGraphStyle,
) {
    if let Some(ops) = ops {
        for op in ops.iter().copied() {
            p.scene().push(op);
        }
        return;
    }

    let bounds = p.bounds();
    let ops = build_debug_grid_ops(bounds, view, style);
    for op in ops.iter().copied() {
        p.scene().push(op);
    }
}

pub(super) fn edges_cache_key(
    graph_rev: u64,
    zoom: f32,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order_hash: u64,
    derived_geometry_key: u64,
    edge_types_rev: u64,
    skin_rev: u64,
) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0);
    let origin = node_origin.normalized();
    let v = EdgePaintCacheKeyV4 {
        graph_rev,
        zoom_q: quantize_f32(zoom, 4096.0),
        node_origin_x_q: quantize_f32(origin.x, 4096.0),
        node_origin_y_q: quantize_f32(origin.y, 4096.0),
        draw_order_hash,
        derived_geometry_key,
        edge_types_rev,
        skin_rev,
    };
    CanvasKey::from_hash(&("fret-node.edges.paint-only.v4", v))
}

pub(super) fn nodes_cache_key(
    graph_rev: u64,
    zoom: f32,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order_hash: u64,
    derived_geometry_key: u64,
) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0);
    let origin = node_origin.normalized();
    let v = NodePaintCacheKeyV3 {
        graph_rev,
        zoom_q: quantize_f32(zoom, 4096.0),
        node_origin_x_q: quantize_f32(origin.x, 4096.0),
        node_origin_y_q: quantize_f32(origin.y, 4096.0),
        draw_order_hash,
        derived_geometry_key,
    };
    CanvasKey::from_hash(&("fret-node.nodes.paint-only.v3", v))
}

pub(super) fn canvas_viewport_rect(
    bounds: Rect,
    view: PanZoom2D,
    margin_screen_px: f32,
) -> Option<Rect> {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let margin_canvas = (margin_screen_px / zoom).max(0.0);

    let tl = view.screen_to_canvas(
        bounds,
        Point::new(
            Px(bounds.origin.x.0 - margin_screen_px),
            Px(bounds.origin.y.0 - margin_screen_px),
        ),
    );
    let br = view.screen_to_canvas(
        bounds,
        Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 + margin_screen_px),
            Px(bounds.origin.y.0 + bounds.size.height.0 + margin_screen_px),
        ),
    );

    let x0 = tl.x.0.min(br.x.0) - margin_canvas;
    let x1 = tl.x.0.max(br.x.0) + margin_canvas;
    let y0 = tl.y.0.min(br.y.0) - margin_canvas;
    let y1 = tl.y.0.max(br.y.0) + margin_canvas;

    if !x0.is_finite() || !x1.is_finite() || !y0.is_finite() || !y1.is_finite() {
        return None;
    }

    Some(Rect::new(
        Point::new(Px(x0), Px(y0)),
        fret_core::Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
    ))
}

fn edge_commands_for_route(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
) -> Box<[PathCommand]> {
    match route {
        EdgeRouteKind::Bezier => {
            let (ctrl1, ctrl2) = canvas_wires::wire_ctrl_points(from, to, zoom);
            vec![
                PathCommand::MoveTo(from),
                PathCommand::CubicTo { ctrl1, ctrl2, to },
            ]
            .into_boxed_slice()
        }
        EdgeRouteKind::Straight => {
            vec![PathCommand::MoveTo(from), PathCommand::LineTo(to)].into_boxed_slice()
        }
        EdgeRouteKind::Step => {
            let mid_x = (from.x.0 + to.x.0) * 0.5;
            vec![
                PathCommand::MoveTo(from),
                PathCommand::LineTo(Point::new(Px(mid_x), from.y)),
                PathCommand::LineTo(Point::new(Px(mid_x), to.y)),
                PathCommand::LineTo(to),
            ]
            .into_boxed_slice()
        }
    }
}

fn include_path_point(bounds: &mut Option<(f32, f32, f32, f32)>, point: Point) {
    if !point.x.0.is_finite() || !point.y.0.is_finite() {
        return;
    }
    match bounds {
        Some((min_x, min_y, max_x, max_y)) => {
            *min_x = min_x.min(point.x.0);
            *min_y = min_y.min(point.y.0);
            *max_x = max_x.max(point.x.0);
            *max_y = max_y.max(point.y.0);
        }
        None => {
            *bounds = Some((point.x.0, point.y.0, point.x.0, point.y.0));
        }
    }
}

fn edge_commands_bbox(commands: &[PathCommand], fallback_from: Point, fallback_to: Point) -> Rect {
    let mut bounds = None;
    include_path_point(&mut bounds, fallback_from);
    include_path_point(&mut bounds, fallback_to);

    for command in commands {
        match *command {
            PathCommand::MoveTo(point) | PathCommand::LineTo(point) => {
                include_path_point(&mut bounds, point);
            }
            PathCommand::QuadTo { ctrl, to } => {
                include_path_point(&mut bounds, ctrl);
                include_path_point(&mut bounds, to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                include_path_point(&mut bounds, ctrl1);
                include_path_point(&mut bounds, ctrl2);
                include_path_point(&mut bounds, to);
            }
            PathCommand::Close => {}
        }
    }

    let Some((min_x, min_y, max_x, max_y)) = bounds else {
        return Rect::new(
            Point::new(
                Px(fallback_from.x.0.min(fallback_to.x.0)),
                Px(fallback_from.y.0.min(fallback_to.y.0)),
            ),
            fret_core::Size::new(
                Px((fallback_from.x.0 - fallback_to.x.0).abs()),
                Px((fallback_from.y.0 - fallback_to.y.0).abs()),
            ),
        );
    };

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        fret_core::Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
    )
}

fn padded_edge_bbox(
    commands: &[PathCommand],
    from: Point,
    to: Point,
    zoom: f32,
    style: &NodeGraphStyle,
    width_mul: f32,
) -> Rect {
    let mut bbox = edge_commands_bbox(commands, from, to);
    let pad = (style.geometry.wire_width * width_mul.max(1.0))
        .max(style.paint.wire_interaction_width)
        / zoom;
    let pad = pad.max(0.0);
    bbox = Rect::new(
        Point::new(Px(bbox.origin.x.0 - pad), Px(bbox.origin.y.0 - pad)),
        fret_core::Size::new(
            Px((bbox.size.width.0 + 2.0 * pad).max(0.0)),
            Px((bbox.size.height.0 + 2.0 * pad).max(0.0)),
        ),
    );
    bbox
}

pub(super) fn build_edges_draws_paint_only(
    graph: &Graph,
    graph_rev: u64,
    zoom: f32,
    geom: &CanvasGeometry,
    style: &NodeGraphStyle,
    edge_types_rev: u64,
    skin_rev: u64,
    edge_types: Option<&crate::ui::NodeGraphEdgeTypes>,
    skin: Option<&dyn crate::ui::NodeGraphSkin>,
) -> Arc<Vec<EdgePathDraw>> {
    let mut out = Vec::<EdgePathDraw>::new();
    out.reserve(graph.edges.len().min(4096));

    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0).max(1.0e-6);
    let presenter = DefaultNodeGraphPresenter::default();

    for (edge_id, edge) in &graph.edges {
        let Some(p0) = geom.port_center(edge.from) else {
            continue;
        };
        let Some(p1) = geom.port_center(edge.to) else {
            continue;
        };

        let mut hint = presenter
            .edge_render_hint(graph, *edge_id, style)
            .normalized();
        if let Some(edge_types) = edge_types {
            hint = edge_types.apply(graph, *edge_id, style, hint).normalized();
        }
        if let Some(skin) = skin {
            hint = skin
                .edge_render_hint(graph, *edge_id, style, &hint, false, false)
                .normalized();
        }

        let custom_path = edge_types.and_then(|edge_types| {
            edge_types.custom_path(
                graph,
                *edge_id,
                style,
                &hint,
                EdgePathInput {
                    from: p0,
                    to: p1,
                    zoom,
                },
            )
        });
        let custom_path_key = custom_path.as_ref().map(|path| path.cache_key).unwrap_or(0);
        let commands = custom_path
            .map(|path| path.commands.into_boxed_slice())
            .unwrap_or_else(|| edge_commands_for_route(hint.route, p0, p1, zoom));
        let bbox = padded_edge_bbox(&commands, p0, p1, zoom, style, hint.width_mul);

        let path_key = CanvasKey::from_hash(&(
            "fret-node.edge-path.v2",
            graph_rev,
            edge_id,
            edge_types_rev,
            skin_rev,
            hint.route,
            custom_path_key,
        ))
        .0;
        let color = hint
            .color
            .unwrap_or_else(|| presenter.edge_color(graph, *edge_id, style));

        out.push(EdgePathDraw {
            edge: *edge_id,
            from: edge.from,
            to: edge.to,
            key: path_key,
            commands,
            bbox,
            color,
            route: hint.route,
            width_mul: hint.width_mul,
            dash: hint.dash,
        });
    }

    Arc::new(out)
}

fn scale_dash_pattern_screen_px_to_canvas_units(
    pattern: DashPatternV1,
    zoom: f32,
) -> Option<DashPatternV1> {
    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }

    let dash = pattern.dash.0 / zoom;
    let gap = pattern.gap.0 / zoom;
    let phase = pattern.phase.0 / zoom;
    let period = dash + gap;
    if !dash.is_finite() || !gap.is_finite() || !phase.is_finite() || dash <= 0.0 || period <= 0.0 {
        return None;
    }

    Some(DashPatternV1::new(Px(dash), Px(gap), Px(phase)))
}

pub(super) fn paint_edges_cached(
    p: &mut CanvasPainter<'_>,
    view: PanZoom2D,
    margin_screen_px: f32,
    draws: Option<Arc<Vec<EdgePathDraw>>>,
    geom: Option<Arc<CanvasGeometry>>,
    node_drag: Option<&NodeDragState>,
    style_tokens: &NodeGraphStyle,
    paint_overrides: Option<&dyn crate::ui::paint_overrides::NodeGraphPaintOverrides>,
) {
    let Some(draws) = draws else {
        return;
    };
    let geom = geom.unwrap_or_else(|| Arc::new(CanvasGeometry::default()));

    let bounds = p.bounds();
    let Some(cull) = canvas_viewport_rect(bounds, view, margin_screen_px) else {
        return;
    };
    let Some(transform) = view.render_transform(bounds) else {
        return;
    };

    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let raster_scale_factor = p.scale_factor() * zoom;
    let base_stroke_width = (style_tokens.geometry.wire_width / zoom).max(0.0);

    p.with_transform(transform, |p| {
        let drag_active = node_drag.is_some_and(NodeDragState::is_active);
        let (ddx, ddy) = node_drag
            .filter(|_| drag_active)
            .map(|d| node_drag_delta_canvas(view, d))
            .unwrap_or((0.0, 0.0));

        for d in draws.iter() {
            let mut paint: PaintBindingV1 = d.color.into();
            let mut stroke_width_mul = d.width_mul;
            let mut dash = d
                .dash
                .and_then(|p| scale_dash_pattern_screen_px_to_canvas_units(p, zoom));

            if let Some(o) = paint_overrides
                .and_then(|p| p.edge_paint_override(d.edge))
                .map(|o| o.normalized())
            {
                if let Some(m) = o.stroke_width_mul
                    && m.is_finite()
                    && m > 0.0
                {
                    stroke_width_mul = m;
                }
                if let Some(dash_override) = o.dash {
                    dash = scale_dash_pattern_screen_px_to_canvas_units(dash_override, zoom);
                }
                if let Some(paint_override) = o.stroke_paint {
                    paint = paint_override;
                }
            }

            let stroke_width = (base_stroke_width * stroke_width_mul).max(0.0);
            let style = PathStyle::StrokeV2(StrokeStyleV2 {
                width: Px(stroke_width),
                join: StrokeJoinV1::Round,
                cap: StrokeCapV1::Round,
                miter_limit: 4.0,
                dash,
            });

            let mut affected_by_drag = false;
            if drag_active {
                let from_node = geom.ports.get(&d.from).map(|h| h.node);
                let to_node = geom.ports.get(&d.to).map(|h| h.node);
                affected_by_drag = from_node
                    .is_some_and(|id| node_drag.is_some_and(|drag| node_drag_contains(drag, id)))
                    || to_node.is_some_and(|id| {
                        node_drag.is_some_and(|drag| node_drag_contains(drag, id))
                    });
            }

            if affected_by_drag {
                let Some(mut p0) = geom.port_center(d.from) else {
                    continue;
                };
                let Some(mut p1) = geom.port_center(d.to) else {
                    continue;
                };
                if let Some(from) = geom.ports.get(&d.from)
                    && node_drag.is_some_and(|drag| node_drag_contains(drag, from.node))
                {
                    p0 = Point::new(Px(p0.x.0 + ddx), Px(p0.y.0 + ddy));
                }
                if let Some(to) = geom.ports.get(&d.to)
                    && node_drag.is_some_and(|drag| node_drag_contains(drag, to.node))
                {
                    p1 = Point::new(Px(p1.x.0 + ddx), Px(p1.y.0 + ddy));
                }

                let commands = edge_commands_for_route(d.route, p0, p1, zoom);
                let bbox = padded_edge_bbox(&commands, p0, p1, zoom, style_tokens, d.width_mul);

                if !rects_intersect(cull, bbox) {
                    continue;
                }

                let key = CanvasKey::from_hash(&(
                    "fret-node.edge-path.drag.v1",
                    d.key,
                    d.edge,
                    quantize_f32(ddx, 1024.0),
                    quantize_f32(ddy, 1024.0),
                ))
                .0;
                p.path_paint(
                    key,
                    DrawOrder(2),
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style,
                    paint,
                    raster_scale_factor,
                );
            } else {
                if !rects_intersect(cull, d.bbox) {
                    continue;
                }
                p.path_paint(
                    d.key,
                    DrawOrder(2),
                    Point::new(Px(0.0), Px(0.0)),
                    &d.commands,
                    style,
                    paint,
                    raster_scale_factor,
                );
            }
        }
    });
}

fn build_nodes_draws_paint_only(graph: &Graph, zoom: f32) -> Arc<Vec<NodeRectDraw>> {
    const DEFAULT_NODE_W_LOGICAL: f32 = 220.0;
    const DEFAULT_NODE_H_LOGICAL: f32 = 140.0;

    let mut draws = Vec::<NodeRectDraw>::new();
    draws.reserve(graph.nodes.len().min(4096));

    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0).max(1.0e-6);
    let inv_zoom = 1.0 / zoom;

    for (id, node) in &graph.nodes {
        if node.hidden {
            continue;
        }
        let logical_size = node.size.unwrap_or(crate::core::CanvasSize {
            width: DEFAULT_NODE_W_LOGICAL,
            height: DEFAULT_NODE_H_LOGICAL,
        });
        let w = logical_size.width * inv_zoom;
        let h = logical_size.height * inv_zoom;
        if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
            continue;
        }
        let rect = Rect::new(
            Point::new(Px(node.pos.x), Px(node.pos.y)),
            fret_core::Size::new(Px(w), Px(h)),
        );
        draws.push(NodeRectDraw { id: *id, rect });
    }

    Arc::new(draws)
}

pub(super) fn paint_nodes_cached(
    p: &mut CanvasPainter<'_>,
    view: PanZoom2D,
    margin_screen_px: f32,
    draws: Option<Arc<Vec<NodeRectDraw>>>,
    style_tokens: &NodeGraphStyle,
    hovered_node: Option<crate::core::NodeId>,
    selected_nodes: &[crate::core::NodeId],
    node_drag: Option<&NodeDragState>,
    paint_overrides: Option<&dyn crate::ui::paint_overrides::NodeGraphPaintOverrides>,
) {
    let Some(draws) = draws else {
        return;
    };

    let bounds = p.bounds();
    let Some(cull) = canvas_viewport_rect(bounds, view, margin_screen_px) else {
        return;
    };
    let Some(transform) = view.render_transform(bounds) else {
        return;
    };

    let fill = Paint::Solid(style_tokens.paint.node_background).into();
    let transparent_fill = Paint::Solid(Color {
        a: 0.0,
        ..style_tokens.paint.node_background
    })
    .into();
    let border_base = style_tokens.paint.node_border;
    let border_hover = Color {
        a: 0.75,
        ..style_tokens.paint.node_border_selected
    };
    let border_selected = style_tokens.paint.node_border_selected;
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let border_w = (1.0 / zoom).max(0.0);
    let corner_r = (style_tokens.paint.node_corner_radius / zoom).max(0.0);
    let border = fret_core::Edges::all(Px(border_w));
    let no_border = fret_core::Edges::all(Px(0.0));
    let corner_radii = fret_core::Corners::all(Px(corner_r));

    p.with_transform(transform, |p| {
        let drag_active = node_drag.is_some_and(NodeDragState::is_active);
        let (ddx, ddy) = node_drag
            .filter(|_| drag_active)
            .map(|d| node_drag_delta_canvas(view, d))
            .unwrap_or((0.0, 0.0));

        for d in draws.iter() {
            let mut rect = d.rect;
            if drag_active && node_drag.is_some_and(|drag| node_drag_contains(drag, d.id)) {
                rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 + ddx), Px(rect.origin.y.0 + ddy)),
                    rect.size,
                );
            }

            if !rects_intersect(cull, rect) {
                continue;
            }
            let selected = selected_nodes.contains(&d.id);
            let hovered = hovered_node.is_some_and(|id| id == d.id);
            let border_color = if selected {
                border_selected
            } else if hovered {
                border_hover
            } else {
                border_base
            };
            let mut background = fill;
            let mut border_paint = Paint::Solid(border_color).into();

            if let Some(o) = paint_overrides
                .and_then(|p| p.node_paint_override(d.id))
                .map(|o| o.normalized())
            {
                if let Some(paint) = o.body_background {
                    background = paint;
                }
                if let Some(paint) = o.border_paint {
                    border_paint = paint;
                }
            }

            p.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background,
                border: no_border,
                border_paint: transparent_fill,
                corner_radii,
            });
            p.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: transparent_fill,
                border,
                border_paint,
                corner_radii,
            });
        }
    });
}

pub(super) fn sync_grid_cache<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    grid_cache: &Model<GridPaintCacheState>,
    view_for_paint: PanZoom2D,
    style_tokens: &NodeGraphStyle,
) -> GridPaintCacheState {
    let mut grid_cache_value = cx
        .get_model_cloned(grid_cache, Invalidation::Paint)
        .unwrap_or_default();

    if grid_cache_value.bounds.size.width.0 > 0.0
        && grid_cache_value.bounds.size.height.0 > 0.0
        && grid_cache_value.bounds.size.width.0.is_finite()
        && grid_cache_value.bounds.size.height.0.is_finite()
    {
        let key = grid_cache_key(grid_cache_value.bounds, view_for_paint, style_tokens);
        if grid_cache_value.key != Some(key) {
            let ops = build_debug_grid_ops(grid_cache_value.bounds, view_for_paint, style_tokens);
            let _ = cx.app.models_mut().update(grid_cache, |st| {
                st.key = Some(key);
                st.rebuilds = st.rebuilds.saturating_add(1);
                st.ops = Some(ops.clone());
            });
            grid_cache_value.key = Some(key);
            grid_cache_value.rebuilds = grid_cache_value.rebuilds.saturating_add(1);
            grid_cache_value.ops = Some(ops);
        }
    }

    grid_cache_value
}

pub(super) fn sync_derived_cache<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    binding: &crate::ui::NodeGraphSurfaceBinding,
    derived_cache: &Model<DerivedGeometryCacheState>,
    graph_rev: u64,
    view_for_paint: PanZoom2D,
    view_value: &NodeGraphViewState,
    interaction_config: &crate::io::NodeGraphInteractionConfig,
    interaction_state: &crate::io::NodeGraphInteractionState,
    runtime_tuning: crate::io::NodeGraphRuntimeTuning,
    style_tokens: &NodeGraphStyle,
    presenter_rev: u64,
    measured_geometry: Option<&Arc<MeasuredGeometryStore>>,
    geometry_overrides: Option<&dyn crate::ui::geometry_overrides::NodeGraphGeometryOverrides>,
    geometry_overrides_rev: u64,
    max_edge_interaction_width_override_px: f32,
) -> DerivedGeometryCacheState {
    let mut derived_cache_value = cx
        .get_model_cloned(derived_cache, Invalidation::Paint)
        .unwrap_or_default();

    let key = derived_geometry_cache_key(
        graph_rev,
        view_for_paint.zoom,
        interaction_config.node_origin,
        &view_value.draw_order,
        interaction_state,
        style_tokens,
        presenter_rev,
        geometry_overrides_rev,
        max_edge_interaction_width_override_px,
    );

    if derived_cache_value.key != Some(key) {
        let (geom, index) =
            read_authoritative_graph_in_models(cx.app.models_mut(), binding, |graph_value| {
                let zoom = PanZoom2D::sanitize_zoom(view_for_paint.zoom, 1.0);
                let z = zoom.max(1.0e-6);

                let geom = build_canvas_geometry_with_overrides(
                    graph_value,
                    view_value,
                    interaction_config.node_origin,
                    style_tokens,
                    zoom,
                    measured_geometry,
                    geometry_overrides,
                );

                let tuning = runtime_tuning.spatial_index;
                let edge_aabb_pad_screen_px = tuning
                    .edge_aabb_pad_screen_px
                    .max(interaction_config.edge_interaction_width)
                    .max(max_edge_interaction_width_override_px)
                    .max(style_tokens.geometry.wire_width)
                    .max(0.0);
                let cell_size_canvas = (tuning.cell_size_screen_px / z)
                    .max(tuning.min_cell_size_screen_px / z)
                    .max(1.0);
                let max_hit_pad_canvas = (edge_aabb_pad_screen_px / z).max(0.0);

                let index = CanvasSpatialDerived::build(
                    graph_value,
                    &geom,
                    zoom,
                    max_hit_pad_canvas,
                    cell_size_canvas,
                );

                (Arc::new(geom), Arc::new(index))
            })
            .unwrap_or_else(|| {
                (
                    Arc::new(CanvasGeometry::default()),
                    Arc::new(CanvasSpatialDerived::empty()),
                )
            });

        let _ = cx.app.models_mut().update(derived_cache, |st| {
            st.key = Some(key);
            st.rebuilds = st.rebuilds.saturating_add(1);
            st.geom = Some(geom.clone());
            st.index = Some(index.clone());
        });

        derived_cache_value.key = Some(key);
        derived_cache_value.rebuilds = derived_cache_value.rebuilds.saturating_add(1);
        derived_cache_value.geom = Some(geom);
        derived_cache_value.index = Some(index);
    }

    derived_cache_value
}

pub(super) fn sync_nodes_cache<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    binding: &crate::ui::NodeGraphSurfaceBinding,
    nodes_cache: &Model<NodePaintCacheState>,
    derived_cache_value: &DerivedGeometryCacheState,
    graph_rev: u64,
    view_for_paint: PanZoom2D,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order_hash: u64,
) -> NodePaintCacheState {
    let mut nodes_cache_value = cx
        .get_model_cloned(nodes_cache, Invalidation::Paint)
        .unwrap_or_default();

    let derived_geometry_key = derived_cache_value.key.as_ref().map(|k| k.0).unwrap_or(0);
    let key = nodes_cache_key(
        graph_rev,
        view_for_paint.zoom,
        node_origin,
        draw_order_hash,
        derived_geometry_key,
    );
    if nodes_cache_value.key != Some(key) {
        let draws = if let Some(geom) = derived_cache_value.geom.as_deref() {
            let mut out = Vec::<NodeRectDraw>::new();
            out.reserve(geom.order.len().min(4096));
            for id in geom.order.iter().copied() {
                let Some(node) = geom.nodes.get(&id) else {
                    continue;
                };
                out.push(NodeRectDraw {
                    id,
                    rect: node.rect,
                });
            }
            Arc::new(out)
        } else {
            read_authoritative_graph_in_models(cx.app.models_mut(), binding, |graph_value| {
                build_nodes_draws_paint_only(graph_value, view_for_paint.zoom)
            })
            .unwrap_or_else(|| Arc::new(Vec::new()))
        };
        let _ = cx.app.models_mut().update(nodes_cache, |st| {
            st.key = Some(key);
            st.rebuilds = st.rebuilds.saturating_add(1);
            st.draws = Some(draws.clone());
        });
        nodes_cache_value.key = Some(key);
        nodes_cache_value.rebuilds = nodes_cache_value.rebuilds.saturating_add(1);
        nodes_cache_value.draws = Some(draws);
    }

    nodes_cache_value
}

pub(super) fn sync_edges_cache<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    binding: &crate::ui::NodeGraphSurfaceBinding,
    edges_cache: &Model<EdgePaintCacheState>,
    derived_cache_value: &DerivedGeometryCacheState,
    graph_rev: u64,
    view_for_paint: PanZoom2D,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order_hash: u64,
    style_tokens: &NodeGraphStyle,
    edge_types_rev: u64,
    skin_rev: u64,
    edge_types: Option<&crate::ui::NodeGraphEdgeTypes>,
    skin: Option<&dyn crate::ui::NodeGraphSkin>,
) -> EdgePaintCacheState {
    let mut edges_cache_value = cx
        .get_model_cloned(edges_cache, Invalidation::Paint)
        .unwrap_or_default();

    let derived_geometry_key = derived_cache_value.key.as_ref().map(|k| k.0).unwrap_or(0);
    let key = edges_cache_key(
        graph_rev,
        view_for_paint.zoom,
        node_origin,
        draw_order_hash,
        derived_geometry_key,
        edge_types_rev,
        skin_rev,
    );
    if edges_cache_value.key != Some(key) {
        let geom_for_edges = derived_cache_value
            .geom
            .clone()
            .unwrap_or_else(|| Arc::new(CanvasGeometry::default()));
        let draws =
            read_authoritative_graph_in_models(cx.app.models_mut(), binding, |graph_value| {
                build_edges_draws_paint_only(
                    graph_value,
                    graph_rev,
                    view_for_paint.zoom,
                    &geom_for_edges,
                    style_tokens,
                    edge_types_rev,
                    skin_rev,
                    edge_types,
                    skin,
                )
            })
            .unwrap_or_else(|| Arc::new(Vec::new()));
        let _ = cx.app.models_mut().update(edges_cache, |st| {
            st.key = Some(key);
            st.rebuilds = st.rebuilds.saturating_add(1);
            st.draws = Some(draws.clone());
        });
        edges_cache_value.key = Some(key);
        edges_cache_value.rebuilds = edges_cache_value.rebuilds.saturating_add(1);
        edges_cache_value.draws = Some(draws);
    }

    edges_cache_value
}

fn build_canvas_geometry_with_overrides(
    graph_value: &Graph,
    view_value: &NodeGraphViewState,
    node_origin: crate::io::NodeGraphNodeOrigin,
    style_tokens: &NodeGraphStyle,
    zoom: f32,
    measured_geometry: Option<&Arc<MeasuredGeometryStore>>,
    geometry_overrides: Option<&dyn crate::ui::geometry_overrides::NodeGraphGeometryOverrides>,
) -> CanvasGeometry {
    if let Some(measured_geometry) = measured_geometry {
        let mut presenter = MeasuredNodeGraphPresenter::new(
            DefaultNodeGraphPresenter::default(),
            measured_geometry.clone(),
        );
        CanvasGeometry::build_with_presenter(
            graph_value,
            &view_value.draw_order,
            style_tokens,
            zoom,
            node_origin,
            &mut presenter,
            geometry_overrides,
        )
    } else {
        let mut presenter = DefaultNodeGraphPresenter::default();
        CanvasGeometry::build_with_presenter(
            graph_value,
            &view_value.draw_order,
            style_tokens,
            zoom,
            node_origin,
            &mut presenter,
            geometry_overrides,
        )
    }
}
