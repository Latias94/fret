use super::*;

impl<H: UiHost, M: NodeGraphCanvasMiddleware> Widget<H> for NodeGraphCanvasWith<M> {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.paint_cache.clear(services);
        self.groups_scene_cache.clear();
        self.nodes_scene_cache.clear();
        self.edges_scene_cache.clear();
        self.edge_labels_scene_cache.clear();
        self.edges_build_states.clear();
        self.edge_labels_build_states.clear();
        self.edge_labels_build_state = None;
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        let theme = cx.theme().snapshot();
        self.sync_style_from_color_mode(theme, Some(cx.services));
        let snapshot = self.sync_view_state(cx.app);
        if cx.input_ctx.focus_is_text_input
            && (command.as_str().starts_with("node_graph.")
                || matches!(
                    command.as_str(),
                    "edit.copy" | "edit.cut" | "edit.paste" | "edit.select_all"
                ))
        {
            return false;
        }

        let mw_outcome = {
            let mw_ctx = NodeGraphCanvasMiddlewareCx {
                graph: &self.graph,
                view_state: &self.view_state,
                style: &self.style,
                bounds: self.interaction.last_bounds,
                pan: snapshot.pan,
                zoom: snapshot.zoom,
            };
            self.middleware.handle_command(cx, &mw_ctx, command)
        };
        if mw_outcome == NodeGraphCanvasCommandOutcome::Handled {
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return true;
        }

        self.handle_command(cx, &snapshot, command)
    }

    fn command_availability(
        &self,
        cx: &mut CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> CommandAvailability {
        if cx.focus != Some(cx.node) {
            return CommandAvailability::NotHandled;
        }

        let clipboard_text = cx.input_ctx.caps.clipboard.text;
        match command.as_str() {
            "edit.copy" | CMD_NODE_GRAPH_COPY => {
                if !clipboard_text {
                    return CommandAvailability::Blocked;
                }

                let has_copyable_selection = self
                    .view_state
                    .read_ref(cx.app, |state| {
                        !state.selected_nodes.is_empty() || !state.selected_groups.is_empty()
                    })
                    .ok()
                    .unwrap_or(false);

                if has_copyable_selection {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            "edit.cut" | CMD_NODE_GRAPH_CUT => {
                if !clipboard_text {
                    return CommandAvailability::Blocked;
                }

                let has_any_selection = self
                    .view_state
                    .read_ref(cx.app, |state| {
                        !state.selected_nodes.is_empty()
                            || !state.selected_edges.is_empty()
                            || !state.selected_groups.is_empty()
                    })
                    .ok()
                    .unwrap_or(false);

                if has_any_selection {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            "edit.paste" | CMD_NODE_GRAPH_PASTE => {
                if !clipboard_text || cx.window.is_none() {
                    return CommandAvailability::Blocked;
                }
                CommandAvailability::Available
            }
            "edit.select_all" | CMD_NODE_GRAPH_SELECT_ALL => {
                let has_any_content = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        !graph.nodes.is_empty() || !graph.groups.is_empty()
                    })
                    .ok()
                    .unwrap_or(false);

                if has_any_content {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            _ => CommandAvailability::NotHandled,
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        let theme = Theme::global(&*cx.app).snapshot();
        self.sync_style_from_color_mode(theme, None);
        self.interaction.last_bounds = Some(cx.bounds);
        let snapshot = self.sync_view_state(cx.app);

        cx.set_role(fret_core::SemanticsRole::Viewport);
        cx.set_focusable(true);
        cx.set_label(self.presenter.a11y_canvas_label().as_ref());

        let active_descendant = match (
            self.interaction.focused_port.is_some(),
            self.interaction.focused_edge.is_some(),
            self.interaction.focused_node.is_some(),
        ) {
            (true, _, _) => cx.children.get(0).copied(),
            (false, true, _) => cx.children.get(1).copied(),
            (false, false, true) => cx.children.get(2).copied(),
            _ => None,
        };
        cx.set_active_descendant(active_descendant);

        let (focused_node, focused_port, focused_edge) = (
            self.interaction.focused_node,
            self.interaction.focused_port,
            self.interaction.focused_edge,
        );

        let style = self.style.clone();
        let value = self
            .graph
            .read_ref(cx.app, |graph| {
                let mut parts: Vec<String> = Vec::new();
                parts.push(format!("zoom {:.3}", snapshot.zoom));
                parts.push(format!(
                    "selected nodes {}, edges {}, groups {}",
                    snapshot.selected_nodes.len(),
                    snapshot.selected_edges.len(),
                    snapshot.selected_groups.len(),
                ));

                if self.interaction.wire_drag.is_some() {
                    parts.push("connecting".to_string());
                }

                if let Some(node) = focused_node {
                    if let Some(label) = self.presenter.a11y_node_label(graph, node) {
                        parts.push(format!("focused node {}", label));
                    } else {
                        parts.push(format!("focused node {:?}", node));
                    }
                }

                if let Some(port) = focused_port {
                    if let Some(label) = self.presenter.a11y_port_label(graph, port) {
                        parts.push(format!("focused port {}", label));
                    } else {
                        parts.push(format!("focused port {:?}", port));
                    }
                }

                if let Some(edge) = focused_edge {
                    if let Some(label) = self.presenter.a11y_edge_label(graph, edge, &style) {
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

    fn render_transform(&self, bounds: Rect) -> Option<Transform2D> {
        let view = PanZoom2D {
            pan: Point::new(Px(self.cached_pan.x), Px(self.cached_pan.y)),
            zoom: self.cached_zoom,
        };
        view.render_transform(bounds)
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let theme = cx.theme().snapshot();
        self.sync_style_from_color_mode(theme, Some(cx.services));
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.view_state, Invalidation::Layout);
        if let Some(queue) = self.edit_queue.as_ref() {
            cx.observe_model(queue, Invalidation::Layout);
        }
        if let Some(queue) = self.view_queue.as_ref() {
            cx.observe_model(queue, Invalidation::Layout);
        }
        for &child in cx.children {
            cx.layout_in(child, cx.bounds);
        }
        self.interaction.last_bounds = Some(cx.bounds);
        self.sync_view_state(cx.app);
        self.drain_edit_queue(cx.app, cx.window);
        self.update_auto_measured_node_sizes(cx);
        let did_view_queue = self.drain_view_queue(cx.app, cx.window);
        let did_fit_on_mount =
            self.maybe_fit_view_on_mount(cx.app, cx.window, cx.bounds, did_view_queue);
        if did_view_queue || did_fit_on_mount {
            cx.request_redraw();
        }
        cx.available
    }

    fn prepaint(&mut self, cx: &mut PrepaintCx<'_, H>) {
        let snapshot = self.sync_view_state(cx.app);
        if !snapshot.interaction.only_render_visible_elements {
            self.last_cull_window_key = None;
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

        fn next_power_of_two_at_least(min: u32, value: f32) -> u32 {
            let target = value.ceil().max(1.0) as u32;
            let pow2 = target.checked_next_power_of_two().unwrap_or(0x8000_0000);
            pow2.max(min)
        }

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

        let mut b = TileCacheKeyBuilder::new("fret-node.canvas.cull_window.v1");
        b.add_u32(nodes_tile_size_screen_px)
            .add_f32_bits(zoom)
            .add_i32(tile_x)
            .add_i32(tile_y);
        let next_key = b.finish();

        match self.last_cull_window_key {
            None => {
                // Initialize the baseline key without counting it as a "shift".
                self.last_cull_window_key = Some(next_key);
            }
            Some(prev_key) if prev_key != next_key => {
                cx.debug_record_node_graph_cull_window_shift(next_key);
                self.last_cull_window_key = Some(next_key);
            }
            _ => {}
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let theme = cx.theme().snapshot();
        self.sync_style_from_color_mode(theme, Some(cx.services));
        let snapshot = self.sync_view_state(cx.app);
        self.interaction.last_bounds = Some(cx.bounds);

        let mw_outcome = {
            let mw_ctx = NodeGraphCanvasMiddlewareCx {
                graph: &self.graph,
                view_state: &self.view_state,
                style: &self.style,
                bounds: Some(cx.bounds),
                pan: snapshot.pan,
                zoom: snapshot.zoom,
            };
            self.middleware.handle_event(cx, &mw_ctx, event)
        };
        if mw_outcome == NodeGraphCanvasEventOutcome::Handled {
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return;
        }

        self.handle_event(cx, event, &snapshot);
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let theme = cx.theme().snapshot();
        self.sync_style_from_color_mode(theme, Some(cx.services));
        self.paint_root(cx);
    }
}
