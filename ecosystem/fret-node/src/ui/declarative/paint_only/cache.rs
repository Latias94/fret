use super::*;

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
    graph: &Model<Graph>,
    derived_cache: &Model<DerivedGeometryCacheState>,
    graph_rev: u64,
    view_for_paint: PanZoom2D,
    view_value: &NodeGraphViewState,
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
        view_value.interaction.node_origin,
        &view_value.draw_order,
        &view_value.resolved_interaction_state(),
        style_tokens,
        presenter_rev,
        geometry_overrides_rev,
        max_edge_interaction_width_override_px,
    );

    if derived_cache_value.key != Some(key) {
        let (geom, index) = cx
            .read_model_ref(graph, Invalidation::Paint, |graph_value| {
                let zoom = PanZoom2D::sanitize_zoom(view_for_paint.zoom, 1.0);
                let z = zoom.max(1.0e-6);

                let geom = build_canvas_geometry_with_overrides(
                    graph_value,
                    view_value,
                    style_tokens,
                    zoom,
                    measured_geometry,
                    geometry_overrides,
                );

                let tuning = view_value.runtime_tuning.spatial_index;
                let edge_aabb_pad_screen_px = tuning
                    .edge_aabb_pad_screen_px
                    .max(view_value.interaction.edge_interaction_width)
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
            .unwrap_or_else(|_| {
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
    graph: &Model<Graph>,
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
            cx.read_model_ref(graph, Invalidation::Paint, |graph_value| {
                build_nodes_draws_paint_only(graph_value, view_for_paint.zoom)
            })
            .unwrap_or_else(|_| Arc::new(Vec::new()))
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
    graph: &Model<Graph>,
    edges_cache: &Model<EdgePaintCacheState>,
    derived_cache_value: &DerivedGeometryCacheState,
    graph_rev: u64,
    view_for_paint: PanZoom2D,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order_hash: u64,
    style_tokens: &NodeGraphStyle,
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
    );
    if edges_cache_value.key != Some(key) {
        let geom_for_edges = derived_cache_value
            .geom
            .clone()
            .unwrap_or_else(|| Arc::new(CanvasGeometry::default()));
        let draws = cx
            .read_model_ref(graph, Invalidation::Paint, |graph_value| {
                build_edges_draws_paint_only(
                    graph_value,
                    graph_rev,
                    view_for_paint.zoom,
                    &geom_for_edges,
                    style_tokens,
                )
            })
            .unwrap_or_else(|_| Arc::new(Vec::new()));
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
            view_value.interaction.node_origin,
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
            view_value.interaction.node_origin,
            &mut presenter,
            geometry_overrides,
        )
    }
}
