use std::sync::Arc;

use crate::ui::presenter::{NodeGraphContextMenuAction, NodeGraphContextMenuItem};

pub(super) fn command_item(
    label: &'static str,
    enabled: bool,
    command: &'static str,
) -> NodeGraphContextMenuItem {
    NodeGraphContextMenuItem {
        label: Arc::<str>::from(label),
        enabled,
        action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(command)),
    }
}
