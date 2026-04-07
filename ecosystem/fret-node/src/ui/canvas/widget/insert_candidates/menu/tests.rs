use super::*;

#[test]
fn build_single_insert_candidate_menu_item_preserves_label_action_and_enabled_state() {
    let item = build_insert_candidate_menu_item(7, Arc::<str>::from("Search Row"), false);

    assert_eq!(item.label.as_ref(), "Search Row");
    assert!(!item.enabled);
    assert!(matches!(
        item.action,
        NodeGraphContextMenuAction::InsertNodeCandidate(7)
    ));
}

#[test]
fn build_insert_candidate_menu_items_preserves_indexes_and_enabled_state() {
    let candidates = super::super::reroute::prepend_reroute_candidate(vec![InsertNodeCandidate {
        kind: NodeKindKey::new("math.add"),
        label: Arc::<str>::from("Add"),
        enabled: false,
        template: None,
        payload: serde_json::Value::Null,
    }]);

    let items = build_insert_candidate_menu_items(&candidates);

    assert_eq!(items.len(), 2);
    assert!(matches!(
        items[0].action,
        NodeGraphContextMenuAction::InsertNodeCandidate(0)
    ));
    assert!(matches!(
        items[1].action,
        NodeGraphContextMenuAction::InsertNodeCandidate(1)
    ));
    assert!(items[0].enabled);
    assert!(!items[1].enabled);
    assert_eq!(items[0].label.as_ref(), "Reroute");
}
