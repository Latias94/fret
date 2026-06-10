pub(super) fn assert_selection_owner_split(
    selection_source: &str,
    selection_projection_source: &str,
) {
    for needle in [
        "mod commands;",
        "mod context_menu;",
        "mod keyboard;",
        "mod projection;",
        "mod select_all;",
        "pub(super) use commands::{",
        "pub(super) use context_menu::proof_collection_context_menu_selection;",
        "pub(super) use keyboard::proof_collection_keyboard_selection;",
        "pub(super) use projection::{",
        "pub(super) use select_all::{",
        "pub(super) struct ProofCollectionKeyboardState",
    ] {
        assert!(
            selection_source.contains(needle),
            "the demo-local collection selection owner should keep pure selection state and command delegation explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(in super::super) fn proof_collection_assets_in_visible_order(",
        "pub(in super::super) fn proof_collection_selected_assets",
        "pub(in super::super) fn proof_collection_active_id(",
        "collect::<HashMap<_, _>>()",
        "selection.first_selected().cloned().filter(contains)",
        "collection_keys.first().cloned()",
    ] {
        assert!(
            selection_projection_source.contains(needle),
            "the demo-local collection selection projection owner should keep visible-order/selected/active projection explicit; missing `{needle}`"
        );
    }
}
