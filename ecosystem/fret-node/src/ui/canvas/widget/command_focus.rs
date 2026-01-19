use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_focus_next_node<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_next_node(cx.app, true);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_prev_node<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_next_node(cx.app, false);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_next_edge<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_next_edge(cx.app, true);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_prev_edge<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_next_edge(cx.app, false);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_next_port<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_next_port(cx.app, true);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_prev_port<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_next_port(cx.app, false);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_port_left<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_port_direction(cx.app, snapshot, PortNavDir::Left);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_port_right<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_port_direction(cx.app, snapshot, PortNavDir::Right);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_port_up<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_port_direction(cx.app, snapshot, PortNavDir::Up);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_focus_port_down<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.focus_port_direction(cx.app, snapshot, PortNavDir::Down);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }

    pub(super) fn cmd_activate<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.activate_focused_port(cx, snapshot);
        if did {
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
        true
    }
}
