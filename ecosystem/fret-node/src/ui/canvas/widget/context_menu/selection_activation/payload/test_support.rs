use super::*;
use crate::core::CanvasPoint;
use fret_core::Px;
use serde_json::Value;

pub(super) fn menu_with_items(items: Vec<NodeGraphContextMenuItem>) -> ContextMenuState {
    ContextMenuState {
        origin: Point::new(Px(1.0), Px(2.0)),
        invoked_at: Point::new(Px(3.0), Px(4.0)),
        target: ContextMenuTarget::BackgroundInsertNodePicker {
            at: CanvasPoint { x: 10.0, y: 20.0 },
        },
        items,
        candidates: vec![InsertNodeCandidate {
            kind: NodeKindKey::new("demo"),
            label: Arc::<str>::from("Demo"),
            enabled: true,
            template: None,
            payload: Value::Null,
        }],
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    }
}
