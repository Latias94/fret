use crate::ui::canvas::widget::*;

pub(super) fn context_menu_activation_payload(
    menu: &ContextMenuState,
    index: usize,
) -> Option<(
    ContextMenuTarget,
    Point,
    NodeGraphContextMenuItem,
    Vec<InsertNodeCandidate>,
)> {
    let item = menu.items.get(index).cloned()?;
    if !item.enabled {
        return None;
    }
    Some((
        menu.target.clone(),
        menu.invoked_at,
        item,
        menu.candidates.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CanvasPoint;
    use fret_core::Px;
    use fret_runtime::CommandId;
    use serde_json::Value;

    fn menu_with_items(items: Vec<NodeGraphContextMenuItem>) -> ContextMenuState {
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

    #[test]
    fn activation_payload_skips_disabled_items() {
        let menu = menu_with_items(vec![NodeGraphContextMenuItem {
            label: Arc::<str>::from("Disabled"),
            enabled: false,
            action: NodeGraphContextMenuAction::Command(CommandId::from("demo.disabled")),
        }]);

        assert!(context_menu_activation_payload(&menu, 0).is_none());
    }

    #[test]
    fn activation_payload_clones_enabled_item_and_context() {
        let menu = menu_with_items(vec![NodeGraphContextMenuItem {
            label: Arc::<str>::from("Enabled"),
            enabled: true,
            action: NodeGraphContextMenuAction::Command(CommandId::from("demo.enabled")),
        }]);

        let Some((target, invoked_at, item, candidates)) =
            context_menu_activation_payload(&menu, 0)
        else {
            panic!("expected enabled menu item to produce an activation payload");
        };

        assert!(matches!(
            target,
            ContextMenuTarget::BackgroundInsertNodePicker {
                at: CanvasPoint { x: 10.0, y: 20.0 }
            }
        ));
        assert_eq!(invoked_at, Point::new(Px(3.0), Px(4.0)));
        assert_eq!(item.label.as_ref(), "Enabled");
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn activation_payload_rejects_out_of_range_index() {
        let menu = menu_with_items(vec![]);

        assert!(context_menu_activation_payload(&menu, 1).is_none());
    }
}
