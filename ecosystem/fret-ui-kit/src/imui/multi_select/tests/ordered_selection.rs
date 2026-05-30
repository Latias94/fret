use super::*;

#[test]
fn ordered_selection_normalizes_to_collection_order_and_deduplicates() {
    let keys = keys();
    let state = ImUiMultiSelectState::from_ordered_selection(
        &keys,
        vec![
            keys[2].clone(),
            keys[0].clone(),
            keys[2].clone(),
            Arc::from("external"),
        ],
        Some(keys[2].clone()),
    );

    assert_eq!(
        state.selected(),
        [keys[0].clone(), keys[2].clone(), Arc::from("external")]
    );
    assert_eq!(state.anchor(), Some(&keys[2]));
}

#[test]
fn ordered_selection_repairs_missing_anchor_to_first_selected_key() {
    let keys = keys();
    let state = ImUiMultiSelectState::from_ordered_selection(
        &keys,
        vec![keys[3].clone(), keys[1].clone()],
        Some(Arc::from("missing")),
    );

    assert_eq!(state.selected(), [keys[1].clone(), keys[3].clone()]);
    assert_eq!(state.anchor(), Some(&keys[1]));
}
