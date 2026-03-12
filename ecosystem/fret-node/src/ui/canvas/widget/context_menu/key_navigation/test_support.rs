use super::super::super::*;
use fret_core::{Point, Px};
use fret_runtime::CommandId;

pub(super) fn item(label: &str, enabled: bool) -> NodeGraphContextMenuItem {
    NodeGraphContextMenuItem {
        label: Arc::<str>::from(label),
        enabled,
        action: NodeGraphContextMenuAction::Command(CommandId::from("demo.command")),
    }
}

pub(super) fn menu(items: Vec<NodeGraphContextMenuItem>, active_item: usize) -> ContextMenuState {
    ContextMenuState {
        origin: Point::new(Px(0.0), Px(0.0)),
        invoked_at: Point::new(Px(0.0), Px(0.0)),
        target: ContextMenuTarget::Background,
        items,
        candidates: Vec::new(),
        hovered_item: None,
        active_item,
        typeahead: String::new(),
    }
}
