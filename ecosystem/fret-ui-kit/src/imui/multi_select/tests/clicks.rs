use super::*;

#[test]
fn plain_click_replaces_selection_and_resets_anchor() {
    let keys = keys();
    let mut state = ImUiMultiSelectState::new(
        vec![keys[0].clone(), keys[2].clone()],
        Some(keys[2].clone()),
    );

    let changed = apply_click(&mut state, &keys, &keys[1], Modifiers::default());

    assert!(changed);
    assert_eq!(state.selected(), [keys[1].clone()]);
    assert_eq!(state.anchor(), Some(&keys[1]));
}

#[test]
fn primary_modifier_click_toggles_membership_in_collection_order() {
    let keys = keys();
    let mut state = ImUiMultiSelectState::new(
        vec![keys[0].clone(), keys[2].clone()],
        Some(keys[2].clone()),
    );

    let changed = apply_click(
        &mut state,
        &keys,
        &keys[1],
        Modifiers {
            meta: true,
            ..Default::default()
        },
    );

    assert!(changed);
    assert_eq!(
        state.selected(),
        [keys[0].clone(), keys[1].clone(), keys[2].clone()]
    );
    assert_eq!(state.anchor(), Some(&keys[1]));
}

#[test]
fn shift_click_selects_range_from_anchor_without_moving_anchor() {
    let keys = keys();
    let mut state = ImUiMultiSelectState::single(keys[1].clone());

    let changed = apply_click(
        &mut state,
        &keys,
        &keys[3],
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );

    assert!(changed);
    assert_eq!(
        state.selected(),
        [keys[1].clone(), keys[2].clone(), keys[3].clone()]
    );
    assert_eq!(state.anchor(), Some(&keys[1]));
}

#[test]
fn shift_click_without_anchor_falls_back_to_single_select() {
    let keys = keys();
    let mut state = ImUiMultiSelectState::<Arc<str>>::default();

    let changed = apply_click(
        &mut state,
        &keys,
        &keys[2],
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );

    assert!(changed);
    assert_eq!(state.selected(), [keys[2].clone()]);
    assert_eq!(state.anchor(), Some(&keys[2]));
}
