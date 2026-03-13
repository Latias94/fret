use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn focus_next_edge<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        focus_nav_traversal::focus_next_edge(self, host, forward)
    }

    pub(super) fn focus_next_node<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        focus_nav_traversal::focus_next_node(self, host, forward)
    }

    pub(super) fn refresh_focused_port_hints<H: UiHost>(&mut self, host: &mut H) {
        focus_nav_ports::refresh_focused_port_hints(self, host)
    }

    pub(super) fn focus_next_port<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        focus_nav_traversal::focus_next_port(self, host, forward)
    }

    pub(super) fn port_center_canvas<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        port: PortId,
    ) -> Option<CanvasPoint> {
        focus_nav_ports::port_center_canvas(self, host, snapshot, port)
    }

    pub(super) fn activate_focused_port<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        focus_nav_ports::activate_focused_port(self, cx, snapshot)
    }
}
