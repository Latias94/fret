use crate::ui::canvas::widget::*;
use crate::ui::commands::{
    CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_INSERT_REROUTE,
    CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn show_context_menu<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        target: ContextMenuTarget,
        items: Vec<NodeGraphContextMenuItem>,
        candidates: Vec<InsertNodeCandidate>,
        clear_hover_edge: bool,
    ) -> bool {
        self.interaction.context_menu = Some(build_context_menu_state(
            self, position, cx.bounds, snapshot, target, items, candidates,
        ));
        if clear_hover_edge {
            self.interaction.hover_edge = None;
        }
        cx.request_focus(cx.node);
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        true
    }

    pub(in crate::ui::canvas::widget) fn build_edge_context_menu_items<H: UiHost>(
        &mut self,
        host: &mut H,
        edge: EdgeId,
    ) -> Vec<NodeGraphContextMenuItem> {
        let presenter = &mut *self.presenter;
        let style = &self.style;
        self.graph
            .read_ref(host, |graph| {
                let mut items = Vec::new();
                presenter.fill_edge_context_menu(graph, edge, style, &mut items);
                items.push(NodeGraphContextMenuItem {
                    label: Arc::<str>::from("Insert Node..."),
                    enabled: true,
                    action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                        CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
                    )),
                });
                items.push(NodeGraphContextMenuItem {
                    label: Arc::<str>::from("Insert Reroute"),
                    enabled: true,
                    action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                        CMD_NODE_GRAPH_INSERT_REROUTE,
                    )),
                });
                items.push(NodeGraphContextMenuItem {
                    label: Arc::<str>::from("Delete"),
                    enabled: true,
                    action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                        CMD_NODE_GRAPH_DELETE_SELECTION,
                    )),
                });
                items
            })
            .ok()
            .unwrap_or_default()
    }
}
