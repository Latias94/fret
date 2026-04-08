use crate::core::GroupId;
use crate::ui::canvas::widget::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandContextSelectionRoute {
    Ignore,
    SelectGroup(GroupId),
}

fn command_context_selection_route(target: &ContextMenuTarget) -> CommandContextSelectionRoute {
    match target {
        ContextMenuTarget::Group(group_id) => CommandContextSelectionRoute::SelectGroup(*group_id),
        ContextMenuTarget::Background
        | ContextMenuTarget::BackgroundInsertNodePicker { .. }
        | ContextMenuTarget::ConnectionInsertNodePicker { .. }
        | ContextMenuTarget::Edge(_)
        | ContextMenuTarget::EdgeInsertNodePicker(_)
        | ContextMenuTarget::ConnectionConvertPicker { .. } => CommandContextSelectionRoute::Ignore,
    }
}

pub(super) fn activate_command_context_action<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    target: &ContextMenuTarget,
    command: fret_runtime::CommandId,
) {
    match command_context_selection_route(target) {
        CommandContextSelectionRoute::SelectGroup(group_id) => {
            canvas.select_group_context_target(cx.app, group_id);
        }
        CommandContextSelectionRoute::Ignore => {}
    }
    cx.dispatch_command(command);
}

#[cfg(test)]
mod tests {
    use super::{CommandContextSelectionRoute, command_context_selection_route};
    use crate::core::{CanvasPoint, EdgeId, GroupId, PortId};
    use crate::ui::canvas::state::ContextMenuTarget;

    #[test]
    fn group_target_routes_to_selection_sync_before_dispatch() {
        let group_id = GroupId::new();

        let route = command_context_selection_route(&ContextMenuTarget::Group(group_id));

        assert_eq!(route, CommandContextSelectionRoute::SelectGroup(group_id));
    }

    #[test]
    fn non_group_targets_do_not_request_selection_sync() {
        let from = PortId::new();
        let to = PortId::new();
        let edge_id = EdgeId::new();
        let at = CanvasPoint { x: 10.0, y: 20.0 };
        let targets = [
            ContextMenuTarget::Background,
            ContextMenuTarget::BackgroundInsertNodePicker { at },
            ContextMenuTarget::ConnectionInsertNodePicker { from, at },
            ContextMenuTarget::Edge(edge_id),
            ContextMenuTarget::EdgeInsertNodePicker(edge_id),
            ContextMenuTarget::ConnectionConvertPicker { from, to, at },
        ];

        for target in targets {
            assert_eq!(
                command_context_selection_route(&target),
                CommandContextSelectionRoute::Ignore
            );
        }
    }
}
