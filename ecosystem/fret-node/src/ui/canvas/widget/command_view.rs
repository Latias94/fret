use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_frame_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let did = self.frame_nodes_in_view(cx.app, cx.window, bounds, &snapshot.selected_nodes);
        super::command_ui::finish_command_paint_if(cx, did)
    }

    pub(super) fn cmd_frame_all<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let nodes = self
            .graph
            .read_ref(cx.app, |graph| {
                graph.nodes.keys().copied().collect::<Vec<_>>()
            })
            .ok()
            .unwrap_or_default();
        let did = self.frame_nodes_in_view(cx.app, cx.window, bounds, &nodes);
        super::command_ui::finish_command_paint_if(cx, did)
    }

    pub(super) fn cmd_reset_view<H: UiHost>(&mut self, cx: &mut CommandCx<'_, H>) -> bool {
        self.update_view_state(cx.app, |s| {
            s.pan = CanvasPoint::default();
            s.zoom = 1.0;
        });
        super::command_ui::finish_command_paint(cx)
    }

    pub(super) fn cmd_zoom_in<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        self.zoom_about_center_factor(bounds, 1.2);
        let pan = self.cached_pan;
        let zoom = self.cached_zoom;
        self.update_view_state(cx.app, |s| {
            s.pan = pan;
            s.zoom = zoom;
        });
        super::command_ui::finish_command_paint(cx)
    }

    pub(super) fn cmd_zoom_out<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        self.zoom_about_center_factor(bounds, 1.0 / 1.2);
        let pan = self.cached_pan;
        let zoom = self.cached_zoom;
        self.update_view_state(cx.app, |s| {
            s.pan = pan;
            s.zoom = zoom;
        });
        super::command_ui::finish_command_paint(cx)
    }
}
