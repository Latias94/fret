use super::*;

pub(super) fn sync_semantics<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut SemanticsCx<'_, H>,
) {
    let theme = Theme::global(&*cx.app).snapshot();
    canvas.sync_style_from_color_mode(theme, None);
    canvas.sync_skin(None);
    canvas.sync_paint_overrides(None);
    canvas.interaction.last_bounds = Some(cx.bounds);
    let snapshot = canvas.sync_view_state(cx.app);

    cx.set_role(fret_core::SemanticsRole::Viewport);
    cx.set_focusable(true);
    cx.set_label(canvas.presenter.a11y_canvas_label().as_ref());
    cx.set_test_id("node_graph.canvas");

    let active_descendant = match (
        canvas.interaction.focused_port.is_some(),
        canvas.interaction.focused_edge.is_some(),
        canvas.interaction.focused_node.is_some(),
    ) {
        (true, _, _) => cx.children.get(0).copied(),
        (false, true, _) => cx.children.get(1).copied(),
        (false, false, true) => cx.children.get(2).copied(),
        _ => None,
    };
    cx.set_active_descendant(active_descendant);

    let (focused_node, focused_port, focused_edge) = (
        canvas.interaction.focused_node,
        canvas.interaction.focused_port,
        canvas.interaction.focused_edge,
    );

    let style = canvas.style.clone();
    let value = canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut parts: Vec<String> = Vec::new();
            parts.push(format!("zoom {:.3}", snapshot.zoom));
            parts.push(format!("panning {}", canvas.interaction.panning));
            parts.push(format!(
                "selected nodes {}, edges {}, groups {}",
                snapshot.selected_nodes.len(),
                snapshot.selected_edges.len(),
                snapshot.selected_groups.len(),
            ));

            if canvas.interaction.wire_drag.is_some() {
                parts.push("connecting".to_string());
            }

            if let Some(node) = focused_node {
                if let Some(label) = canvas.presenter.a11y_node_label(graph, node) {
                    parts.push(format!("focused node {}", label));
                } else {
                    parts.push(format!("focused node {:?}", node));
                }
            }

            if let Some(port) = focused_port {
                if let Some(label) = canvas.presenter.a11y_port_label(graph, port) {
                    parts.push(format!("focused port {}", label));
                } else {
                    parts.push(format!("focused port {:?}", port));
                }
            }

            if let Some(edge) = focused_edge {
                if let Some(label) = canvas.presenter.a11y_edge_label(graph, edge, &style) {
                    parts.push(format!("focused edge {}", label));
                } else {
                    parts.push(format!("focused edge {:?}", edge));
                }
            }

            parts.join("; ")
        })
        .ok()
        .unwrap_or_else(|| format!("zoom {:.3}", snapshot.zoom));

    cx.set_value(value);
}

pub(super) fn layout_widget<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) -> Size {
    let theme = cx.theme().snapshot();
    canvas.sync_style_from_color_mode(theme, Some(cx.services));
    canvas.sync_skin(Some(cx.services));
    observe_layout_models(canvas, cx);
    canvas.interaction.last_bounds = Some(cx.bounds);
    let snapshot = canvas.sync_view_state(cx.app);

    canvas.update_auto_measured_node_sizes(cx);

    if canvas.diagnostics_anchor_ports.is_some() {
        let (geometry, _index) = canvas.canvas_derived(&*cx.app, &snapshot);
        canvas.publish_derived_outputs(&*cx.app, &snapshot, cx.bounds, &geometry);
    }

    layout_children(canvas, cx);
    canvas.drain_edit_queue(cx.app, cx.window);
    let did_view_queue = canvas.drain_view_queue(cx.app, cx.window);
    let did_fit_on_mount =
        canvas.maybe_fit_view_on_mount(cx.app, cx.window, cx.bounds, did_view_queue);
    if did_view_queue || did_fit_on_mount {
        cx.request_redraw();
    }
    cx.available
}

pub(super) fn prepaint_cull_window<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PrepaintCx<'_, H>,
) {
    let snapshot = canvas.sync_view_state(cx.app);
    if !snapshot.interaction.only_render_visible_elements {
        canvas.last_cull_window_key = None;
        return;
    }

    let zoom = snapshot.zoom;
    if !zoom.is_finite() || zoom <= 1.0e-6 {
        return;
    }

    let viewport_max_screen_px = cx.bounds.size.width.0.max(cx.bounds.size.height.0);
    if !viewport_max_screen_px.is_finite() || viewport_max_screen_px <= 0.0 {
        return;
    }

    const STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN: u32 = 1024;
    const STATIC_NODES_TILE_MUL: f32 = 2.0;

    let nodes_tile_size_screen_px = next_power_of_two_at_least(
        STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN,
        viewport_max_screen_px * STATIC_NODES_TILE_MUL,
    );
    let nodes_cache_tile_size_canvas = (nodes_tile_size_screen_px as f32 / zoom).max(1.0);

    let viewport = CanvasViewport2D::new(
        cx.bounds,
        PanZoom2D {
            pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
            zoom,
        },
    );
    let viewport_rect = viewport.visible_canvas_rect();
    let center_x = viewport_rect.origin.x.0 + 0.5 * viewport_rect.size.width.0;
    let center_y = viewport_rect.origin.y.0 + 0.5 * viewport_rect.size.height.0;
    if !center_x.is_finite() || !center_y.is_finite() {
        return;
    }

    let tile_x = (center_x / nodes_cache_tile_size_canvas).floor() as i32;
    let tile_y = (center_y / nodes_cache_tile_size_canvas).floor() as i32;

    let mut builder = TileCacheKeyBuilder::new("fret-node.canvas.cull_window.v1");
    builder
        .add_u32(nodes_tile_size_screen_px)
        .add_f32_bits(zoom)
        .add_i32(tile_x)
        .add_i32(tile_y);
    let next_key = builder.finish();

    match canvas.last_cull_window_key {
        None => {
            canvas.last_cull_window_key = Some(next_key);
        }
        Some(prev_key) if prev_key != next_key => {
            cx.debug_record_node_graph_cull_window_shift(next_key);
            canvas.last_cull_window_key = Some(next_key);
        }
        _ => {}
    }
}

fn observe_layout_models<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) {
    cx.observe_model(&canvas.graph, Invalidation::Layout);
    cx.observe_model(&canvas.view_state, Invalidation::Layout);
    if let Some(queue) = canvas.edit_queue.as_ref() {
        cx.observe_model(queue, Invalidation::Layout);
    }
    if let Some(queue) = canvas.view_queue.as_ref() {
        cx.observe_model(queue, Invalidation::Layout);
    }
}

fn layout_children<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) {
    let internals_snapshot = canvas
        .internals
        .as_ref()
        .map(|store| store.snapshot())
        .unwrap_or_default();
    let anchors = canvas.diagnostics_anchor_ports.as_ref();
    let zero = Rect::new(cx.bounds.origin, Size::new(Px(0.0), Px(0.0)));
    for (index, &child) in cx.children.iter().enumerate() {
        if let Some(anchors) = anchors
            && index >= anchors.child_offset
            && index < anchors.child_offset.saturating_add(anchors.ports.len())
        {
            let port_index = index.saturating_sub(anchors.child_offset);
            let port = anchors.ports.get(port_index).copied();
            let rect = port
                .as_ref()
                .and_then(|port| internals_snapshot.ports_window.get(port).copied())
                .unwrap_or(zero);
            cx.layout_in(child, rect);
        } else {
            cx.layout_in(child, cx.bounds);
        }
    }
}

fn next_power_of_two_at_least(min: u32, value: f32) -> u32 {
    let target = value.ceil().max(1.0) as u32;
    let pow2 = target.checked_next_power_of_two().unwrap_or(0x8000_0000);
    pow2.max(min)
}
