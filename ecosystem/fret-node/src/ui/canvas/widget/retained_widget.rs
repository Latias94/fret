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
        retained_widget_command_availability::command_availability(self, cx, command)
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
