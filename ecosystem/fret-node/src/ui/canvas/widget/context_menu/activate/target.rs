use crate::ui::canvas::widget::*;

#[derive(Debug)]
enum TargetContextActionRoute {
    Ignore,
    BackgroundInsert {
        at: CanvasPoint,
        action: NodeGraphContextMenuAction,
    },
    ConnectionInsert {
        from: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    },
    Edge {
        edge_id: EdgeId,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    },
    EdgeInsert {
        edge_id: EdgeId,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    },
    ConnectionConvert {
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    },
}

fn target_context_action_route(
    target: &ContextMenuTarget,
    invoked_at: Point,
    action: NodeGraphContextMenuAction,
) -> TargetContextActionRoute {
    match target {
        ContextMenuTarget::BackgroundInsertNodePicker { at } => {
            TargetContextActionRoute::BackgroundInsert { at: *at, action }
        }
        ContextMenuTarget::ConnectionInsertNodePicker { from, at } => {
            TargetContextActionRoute::ConnectionInsert {
                from: *from,
                at: *at,
                invoked_at,
                action,
            }
        }
        ContextMenuTarget::Edge(edge_id) => TargetContextActionRoute::Edge {
            edge_id: *edge_id,
            invoked_at,
            action,
        },
        ContextMenuTarget::EdgeInsertNodePicker(edge_id) => TargetContextActionRoute::EdgeInsert {
            edge_id: *edge_id,
            invoked_at,
            action,
        },
        ContextMenuTarget::ConnectionConvertPicker { from, to, at } => {
            TargetContextActionRoute::ConnectionConvert {
                from: *from,
                to: *to,
                at: *at,
                invoked_at,
                action,
            }
        }
        ContextMenuTarget::Background | ContextMenuTarget::Group(_) => {
            TargetContextActionRoute::Ignore
        }
    }
}

pub(super) fn activate_target_context_action<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    target: &ContextMenuTarget,
    invoked_at: Point,
    action: NodeGraphContextMenuAction,
    menu_candidates: &[InsertNodeCandidate],
) {
    match target_context_action_route(target, invoked_at, action) {
        TargetContextActionRoute::BackgroundInsert { at, action } => {
            let _ = canvas.activate_background_context_action(cx, at, action, menu_candidates);
        }
        TargetContextActionRoute::ConnectionInsert {
            from,
            at,
            invoked_at,
            action,
        } => {
            let _ = canvas.activate_connection_insert_picker_action(
                cx,
                from,
                at,
                invoked_at,
                action,
                menu_candidates,
            );
        }
        TargetContextActionRoute::Edge {
            edge_id,
            invoked_at,
            action,
        } => {
            let _ = canvas.activate_edge_context_action(cx, edge_id, invoked_at, action);
        }
        TargetContextActionRoute::EdgeInsert {
            edge_id,
            invoked_at,
            action,
        } => {
            let _ = edge_insert::activate_edge_insert_picker_action(
                canvas,
                cx,
                edge_id,
                invoked_at,
                action,
                menu_candidates,
            );
        }
        TargetContextActionRoute::ConnectionConvert {
            from,
            to,
            at,
            invoked_at,
            action,
        } => {
            let _ = canvas.activate_connection_conversion_picker_action(
                cx,
                from,
                to,
                at,
                invoked_at,
                action,
                menu_candidates,
            );
        }
        TargetContextActionRoute::Ignore => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{TargetContextActionRoute, target_context_action_route};
    use crate::core::{CanvasPoint, EdgeId, GroupId, PortId};
    use crate::ui::canvas::state::ContextMenuTarget;
    use crate::ui::presenter::NodeGraphContextMenuAction;
    use fret_core::{Point, Px};

    #[test]
    fn background_insert_picker_routes_to_background_executor() {
        let at = CanvasPoint { x: 42.0, y: 64.0 };
        let invoked_at = Point::new(Px(12.0), Px(24.0));

        let route = target_context_action_route(
            &ContextMenuTarget::BackgroundInsertNodePicker { at },
            invoked_at,
            NodeGraphContextMenuAction::InsertNodeCandidate(3),
        );

        assert!(matches!(
            route,
            TargetContextActionRoute::BackgroundInsert {
                at: route_at,
                action: NodeGraphContextMenuAction::InsertNodeCandidate(3),
            } if route_at == at
        ));
    }

    #[test]
    fn connection_insert_picker_routes_with_canvas_and_screen_points() {
        let from = PortId::new();
        let at = CanvasPoint { x: 16.0, y: 32.0 };
        let invoked_at = Point::new(Px(120.0), Px(48.0));

        let route = target_context_action_route(
            &ContextMenuTarget::ConnectionInsertNodePicker { from, at },
            invoked_at,
            NodeGraphContextMenuAction::InsertNodeCandidate(1),
        );

        assert!(matches!(
            route,
            TargetContextActionRoute::ConnectionInsert {
                from: route_from,
                at: route_at,
                invoked_at: route_invoked_at,
                action: NodeGraphContextMenuAction::InsertNodeCandidate(1),
            } if route_from == from && route_at == at && route_invoked_at == invoked_at
        ));
    }

    #[test]
    fn edge_targets_route_to_edge_specific_executors() {
        let edge_id = EdgeId::new();
        let invoked_at = Point::new(Px(200.0), Px(300.0));

        let edge_route = target_context_action_route(
            &ContextMenuTarget::Edge(edge_id),
            invoked_at,
            NodeGraphContextMenuAction::DeleteEdge,
        );
        let edge_insert_route = target_context_action_route(
            &ContextMenuTarget::EdgeInsertNodePicker(edge_id),
            invoked_at,
            NodeGraphContextMenuAction::InsertNodeCandidate(0),
        );

        assert!(matches!(
            edge_route,
            TargetContextActionRoute::Edge {
                edge_id: route_edge,
                invoked_at: route_invoked_at,
                action: NodeGraphContextMenuAction::DeleteEdge,
            } if route_edge == edge_id && route_invoked_at == invoked_at
        ));
        assert!(matches!(
            edge_insert_route,
            TargetContextActionRoute::EdgeInsert {
                edge_id: route_edge,
                invoked_at: route_invoked_at,
                action: NodeGraphContextMenuAction::InsertNodeCandidate(0),
            } if route_edge == edge_id && route_invoked_at == invoked_at
        ));
    }

    #[test]
    fn conversion_picker_routes_to_conversion_executor() {
        let from = PortId::new();
        let to = PortId::new();
        let at = CanvasPoint { x: 11.0, y: 22.0 };
        let invoked_at = Point::new(Px(88.0), Px(99.0));

        let route = target_context_action_route(
            &ContextMenuTarget::ConnectionConvertPicker { from, to, at },
            invoked_at,
            NodeGraphContextMenuAction::InsertNodeCandidate(5),
        );

        assert!(matches!(
            route,
            TargetContextActionRoute::ConnectionConvert {
                from: route_from,
                to: route_to,
                at: route_at,
                invoked_at: route_invoked_at,
                action: NodeGraphContextMenuAction::InsertNodeCandidate(5),
            } if route_from == from
                && route_to == to
                && route_at == at
                && route_invoked_at == invoked_at
        ));
    }

    #[test]
    fn background_and_group_targets_ignore_non_command_actions() {
        let invoked_at = Point::new(Px(1.0), Px(2.0));

        let background_route = target_context_action_route(
            &ContextMenuTarget::Background,
            invoked_at,
            NodeGraphContextMenuAction::OpenInsertNodePicker,
        );
        let group_route = target_context_action_route(
            &ContextMenuTarget::Group(GroupId::new()),
            invoked_at,
            NodeGraphContextMenuAction::OpenInsertNodePicker,
        );

        assert!(matches!(background_route, TargetContextActionRoute::Ignore));
        assert!(matches!(group_route, TargetContextActionRoute::Ignore));
    }
}
