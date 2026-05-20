mod command;
mod retained_cx;
mod target;

use crate::ui::canvas::widget::*;

pub(in crate::ui::canvas::widget) trait ContextMenuActionCx<M: NodeGraphCanvasMiddleware>:
    command::CommandContextActionCx<M> + target::TargetContextActionCx<M>
{
}

impl<M, T> ContextMenuActionCx<M> for T
where
    M: NodeGraphCanvasMiddleware,
    T: command::CommandContextActionCx<M> + target::TargetContextActionCx<M>,
{
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn activate_context_menu_item(
        &mut self,
        cx: &mut impl ContextMenuActionCx<M>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        menu_candidates: &[InsertNodeCandidate],
    ) {
        let action = item.action;
        if let NodeGraphContextMenuAction::Command(command) = action {
            command::activate_command_context_action(self, cx, target, command);
            return;
        }

        target::activate_target_context_action(
            self,
            cx,
            target,
            invoked_at,
            action,
            menu_candidates,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CanvasPoint, EdgeId, Graph, GraphId, GroupId, PortId};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use fret_core::{Point, Px};
    use fret_runtime::{CommandId, ModelStore};

    #[derive(Debug, PartialEq)]
    enum TargetCall {
        BackgroundInsert {
            at: CanvasPoint,
            action: NodeGraphContextMenuAction,
            candidates: usize,
        },
        ConnectionInsert {
            from: PortId,
            at: CanvasPoint,
            invoked_at: Point,
            action: NodeGraphContextMenuAction,
            candidates: usize,
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
            candidates: usize,
        },
        ConnectionConvert {
            from: PortId,
            to: PortId,
            at: CanvasPoint,
            invoked_at: Point,
            action: NodeGraphContextMenuAction,
            candidates: usize,
        },
    }

    #[derive(Default)]
    struct StubCx {
        selected_groups: Vec<GroupId>,
        dispatched_commands: Vec<CommandId>,
        target_calls: Vec<TargetCall>,
    }

    impl command::CommandContextActionCx<NoopNodeGraphCanvasMiddleware> for StubCx {
        fn select_group_context_target(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            group_id: GroupId,
        ) {
            self.selected_groups.push(group_id);
        }

        fn dispatch_context_command(&mut self, command: CommandId) {
            self.dispatched_commands.push(command);
        }
    }

    impl target::TargetContextActionCx<NoopNodeGraphCanvasMiddleware> for StubCx {
        fn activate_background_context_action(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            at: CanvasPoint,
            action: NodeGraphContextMenuAction,
            menu_candidates: &[InsertNodeCandidate],
        ) -> bool {
            self.target_calls.push(TargetCall::BackgroundInsert {
                at,
                action,
                candidates: menu_candidates.len(),
            });
            true
        }

        fn activate_connection_insert_picker_action(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            from: PortId,
            at: CanvasPoint,
            invoked_at: Point,
            action: NodeGraphContextMenuAction,
            menu_candidates: &[InsertNodeCandidate],
        ) -> bool {
            self.target_calls.push(TargetCall::ConnectionInsert {
                from,
                at,
                invoked_at,
                action,
                candidates: menu_candidates.len(),
            });
            true
        }

        fn activate_edge_context_action(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            edge_id: EdgeId,
            invoked_at: Point,
            action: NodeGraphContextMenuAction,
        ) -> bool {
            self.target_calls.push(TargetCall::Edge {
                edge_id,
                invoked_at,
                action,
            });
            true
        }

        fn activate_edge_insert_picker_action(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            edge_id: EdgeId,
            invoked_at: Point,
            action: NodeGraphContextMenuAction,
            menu_candidates: &[InsertNodeCandidate],
        ) -> bool {
            self.target_calls.push(TargetCall::EdgeInsert {
                edge_id,
                invoked_at,
                action,
                candidates: menu_candidates.len(),
            });
            true
        }

        fn activate_connection_conversion_picker_action(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            from: PortId,
            to: PortId,
            at: CanvasPoint,
            invoked_at: Point,
            action: NodeGraphContextMenuAction,
            menu_candidates: &[InsertNodeCandidate],
        ) -> bool {
            self.target_calls.push(TargetCall::ConnectionConvert {
                from,
                to,
                at,
                invoked_at,
                action,
                candidates: menu_candidates.len(),
            });
            true
        }
    }

    fn test_canvas() -> NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
        let mut models = ModelStore::default();
        let graph = models.insert(Graph::new(GraphId::new()));
        let view = models.insert(NodeGraphViewState::default());
        let editor_config = models.insert(NodeGraphEditorConfig::default());
        NodeGraphCanvasWith::new_with_middleware(
            graph,
            view,
            editor_config,
            NoopNodeGraphCanvasMiddleware,
        )
    }

    fn item(action: NodeGraphContextMenuAction) -> NodeGraphContextMenuItem {
        NodeGraphContextMenuItem {
            label: std::sync::Arc::<str>::from("Action"),
            enabled: true,
            action,
        }
    }

    #[test]
    fn command_items_select_group_before_dispatching_command() {
        let mut canvas = test_canvas();
        let mut cx = StubCx::default();
        let group_id = GroupId::new();
        let command = CommandId::from("node.demo.command");

        canvas.activate_context_menu_item(
            &mut cx,
            &ContextMenuTarget::Group(group_id),
            Point::new(Px(1.0), Px(2.0)),
            item(NodeGraphContextMenuAction::Command(command.clone())),
            &[],
        );

        assert_eq!(cx.selected_groups, vec![group_id]);
        assert_eq!(cx.dispatched_commands, vec![command]);
        assert!(cx.target_calls.is_empty());
    }

    #[test]
    fn non_command_items_delegate_to_target_action_executor() {
        let mut canvas = test_canvas();
        let mut cx = StubCx::default();
        let at = CanvasPoint { x: 32.0, y: 64.0 };
        let invoked_at = Point::new(Px(10.0), Px(20.0));

        canvas.activate_context_menu_item(
            &mut cx,
            &ContextMenuTarget::BackgroundInsertNodePicker { at },
            invoked_at,
            item(NodeGraphContextMenuAction::InsertNodeCandidate(2)),
            &[],
        );

        assert!(cx.selected_groups.is_empty());
        assert!(cx.dispatched_commands.is_empty());
        assert_eq!(
            cx.target_calls,
            vec![TargetCall::BackgroundInsert {
                at,
                action: NodeGraphContextMenuAction::InsertNodeCandidate(2),
                candidates: 0,
            }]
        );
    }

    #[test]
    fn ignored_target_actions_are_side_effect_free() {
        let mut canvas = test_canvas();
        let mut cx = StubCx::default();

        canvas.activate_context_menu_item(
            &mut cx,
            &ContextMenuTarget::Background,
            Point::new(Px(1.0), Px(2.0)),
            item(NodeGraphContextMenuAction::Custom(7)),
            &[],
        );

        assert!(cx.selected_groups.is_empty());
        assert!(cx.dispatched_commands.is_empty());
        assert!(cx.target_calls.is_empty());
    }
}
