use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_open_insert_node<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_open_insert::cmd_open_insert_node(self, cx, snapshot)
    }

    pub(super) fn cmd_create_group<H: UiHost>(&mut self, cx: &mut CommandCx<'_, H>) -> bool {
        super::command_open_group::cmd_create_group(self, cx)
    }

    pub(super) fn cmd_group_bring_to_front<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_open_group::cmd_group_bring_to_front(self, cx, snapshot)
    }

    pub(super) fn cmd_group_send_to_back<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_open_group::cmd_group_send_to_back(self, cx, snapshot)
    }

    pub(super) fn cmd_group_rename<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_open_group::cmd_group_rename(self, cx, snapshot)
    }

    pub(super) fn cmd_open_split_edge_insert_node<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_open_edge::cmd_open_split_edge_insert_node(self, cx, snapshot)
    }

    pub(super) fn cmd_insert_reroute<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_open_edge::cmd_insert_reroute(self, cx, snapshot)
    }

    pub(super) fn cmd_open_conversion_picker<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_open_conversion::cmd_open_conversion_picker(self, cx, snapshot)
    }
}
