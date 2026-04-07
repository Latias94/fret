mod custom_action;
mod delete;
mod open_insert;
mod reroute;

use crate::ui::canvas::widget::*;

#[derive(Debug)]
enum EdgeContextActionRoute {
    Ignore,
    OpenInsertNodePicker { edge_id: EdgeId, invoked_at: Point },
    InsertReroute { edge_id: EdgeId, invoked_at: Point },
    DeleteEdge { edge_id: EdgeId },
    Custom { edge_id: EdgeId, action_id: u64 },
}

fn edge_context_action_route(
    edge_id: EdgeId,
    invoked_at: Point,
    action: NodeGraphContextMenuAction,
) -> EdgeContextActionRoute {
    match action {
        NodeGraphContextMenuAction::OpenInsertNodePicker => {
            EdgeContextActionRoute::OpenInsertNodePicker {
                edge_id,
                invoked_at,
            }
        }
        NodeGraphContextMenuAction::InsertReroute => EdgeContextActionRoute::InsertReroute {
            edge_id,
            invoked_at,
        },
        NodeGraphContextMenuAction::DeleteEdge => EdgeContextActionRoute::DeleteEdge { edge_id },
        NodeGraphContextMenuAction::Custom(action_id) => {
            EdgeContextActionRoute::Custom { edge_id, action_id }
        }
        NodeGraphContextMenuAction::Command(_)
        | NodeGraphContextMenuAction::InsertNodeCandidate(_) => EdgeContextActionRoute::Ignore,
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn activate_edge_context_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        edge_id: EdgeId,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    ) -> bool {
        match edge_context_action_route(edge_id, invoked_at, action) {
            EdgeContextActionRoute::OpenInsertNodePicker {
                edge_id,
                invoked_at,
            } => {
                open_insert::open_edge_insert_context_menu(self, cx, edge_id, invoked_at);
                true
            }
            EdgeContextActionRoute::InsertReroute {
                edge_id,
                invoked_at,
            } => {
                reroute::insert_edge_reroute(self, cx, edge_id, invoked_at);
                true
            }
            EdgeContextActionRoute::DeleteEdge { edge_id } => {
                delete::delete_edge(self, cx, edge_id);
                true
            }
            EdgeContextActionRoute::Custom { edge_id, action_id } => {
                custom_action::apply_custom_edge_context_action(self, cx, edge_id, action_id);
                true
            }
            EdgeContextActionRoute::Ignore => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EdgeContextActionRoute, edge_context_action_route};
    use crate::core::EdgeId;
    use crate::ui::presenter::NodeGraphContextMenuAction;
    use fret_core::{Point, Px};
    use fret_runtime::CommandId;

    #[test]
    fn edge_action_routes_preserve_edge_id_and_invocation_point() {
        let edge_id = EdgeId::new();
        let invoked_at = Point::new(Px(120.0), Px(48.0));

        let open_insert = edge_context_action_route(
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::OpenInsertNodePicker,
        );
        let reroute = edge_context_action_route(
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::InsertReroute,
        );

        assert!(matches!(
            open_insert,
            EdgeContextActionRoute::OpenInsertNodePicker {
                edge_id: route_edge,
                invoked_at: route_invoked_at,
            } if route_edge == edge_id && route_invoked_at == invoked_at
        ));
        assert!(matches!(
            reroute,
            EdgeContextActionRoute::InsertReroute {
                edge_id: route_edge,
                invoked_at: route_invoked_at,
            } if route_edge == edge_id && route_invoked_at == invoked_at
        ));
    }

    #[test]
    fn delete_and_custom_edge_actions_route_to_edge_specific_executors() {
        let edge_id = EdgeId::new();
        let invoked_at = Point::new(Px(10.0), Px(20.0));

        let delete_route =
            edge_context_action_route(edge_id, invoked_at, NodeGraphContextMenuAction::DeleteEdge);
        let custom_route =
            edge_context_action_route(edge_id, invoked_at, NodeGraphContextMenuAction::Custom(7));

        assert!(matches!(
            delete_route,
            EdgeContextActionRoute::DeleteEdge { edge_id: route_edge } if route_edge == edge_id
        ));
        assert!(matches!(
            custom_route,
            EdgeContextActionRoute::Custom {
                edge_id: route_edge,
                action_id: 7,
            } if route_edge == edge_id
        ));
    }

    #[test]
    fn non_edge_actions_are_ignored_by_edge_executor() {
        let edge_id = EdgeId::new();
        let invoked_at = Point::new(Px(1.0), Px(2.0));

        let command_route = edge_context_action_route(
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::Command(CommandId::from("demo.command")),
        );
        let insert_candidate_route = edge_context_action_route(
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::InsertNodeCandidate(0),
        );

        assert!(matches!(command_route, EdgeContextActionRoute::Ignore));
        assert!(matches!(
            insert_candidate_route,
            EdgeContextActionRoute::Ignore
        ));
    }
}
