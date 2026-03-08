use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_focus_next_node<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_cycle::cmd_focus_next_node(self, cx)
    }

    pub(super) fn cmd_focus_prev_node<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_cycle::cmd_focus_prev_node(self, cx)
    }

    pub(super) fn cmd_focus_next_edge<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_cycle::cmd_focus_next_edge(self, cx)
    }

    pub(super) fn cmd_focus_prev_edge<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_cycle::cmd_focus_prev_edge(self, cx)
    }

    pub(super) fn cmd_focus_next_port<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_cycle::cmd_focus_next_port(self, cx)
    }

    pub(super) fn cmd_focus_prev_port<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_cycle::cmd_focus_prev_port(self, cx)
    }

    pub(super) fn cmd_focus_port_left<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_port::cmd_focus_port_left(self, cx, snapshot)
    }

    pub(super) fn cmd_focus_port_right<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_port::cmd_focus_port_right(self, cx, snapshot)
    }

    pub(super) fn cmd_focus_port_up<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_port::cmd_focus_port_up(self, cx, snapshot)
    }

    pub(super) fn cmd_focus_port_down<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_port::cmd_focus_port_down(self, cx, snapshot)
    }

    pub(super) fn cmd_activate<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_focus_port::cmd_activate(self, cx, snapshot)
    }
}
