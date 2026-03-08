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
        retained_widget_runtime::handle_retained_command(self, cx, command)
    }

    fn command_availability(
        &self,
        cx: &mut CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> CommandAvailability {
        if cx.focus != Some(cx.node) {
            return CommandAvailability::NotHandled;
        }

        let clipboard_read = cx.input_ctx.caps.clipboard.text.read;
        let clipboard_write = cx.input_ctx.caps.clipboard.text.write;
        match command.as_str() {
            "edit.copy" | CMD_NODE_GRAPH_COPY => {
                if !clipboard_write {
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
                if !clipboard_write {
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
                if !clipboard_read || cx.window.is_none() {
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
        retained_widget_frame::sync_semantics(self, cx);
    }

    fn render_transform(&self, bounds: Rect) -> Option<Transform2D> {
        let view = PanZoom2D {
            pan: Point::new(Px(self.cached_pan.x), Px(self.cached_pan.y)),
            zoom: self.cached_zoom,
        };
        view.render_transform(bounds)
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        retained_widget_frame::layout_widget(self, cx)
    }

    fn prepaint(&mut self, cx: &mut PrepaintCx<'_, H>) {
        retained_widget_frame::prepaint_cull_window(self, cx);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        retained_widget_runtime::handle_retained_event(self, cx, event);
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        retained_widget_runtime::paint_retained_widget(self, cx);
    }
}
