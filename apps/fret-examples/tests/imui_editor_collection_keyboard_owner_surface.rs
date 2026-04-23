#[test]
fn imui_editor_proof_demo_keeps_collection_keyboard_owner_app_owned_and_explicit() {
    let source = include_str!("../src/imui_editor_proof_demo/collection.rs");

    for needle in [
        "struct ProofCollectionKeyboardState {",
        "fn proof_collection_active_line(",
        "fn proof_collection_keyboard_selection(",
        "fn proof_collection_keyboard_next_index(",
        "fn proof_collection_keyboard_move_selection(",
        "imui_editor_proof_demo.model.authoring_parity.collection_keyboard",
        "cx.key_on_key_down_for(scope_id, Arc::new(move |host, acx, down| {",
        "host.request_focus(acx.target);",
        "state.active_id = next_selection.selected.first().cloned();",
        "state.active_id = None;",
        "\"Active tile: none. Click background to focus the collection scope, then use Arrow/Home/End to drive selection app-locally.\"",
        "\"Active tile: {}. Shift+Arrow/Home/End extends from the current anchor; Escape clears the selection without widening shared IMUI helper ownership.\"",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection keyboard-owner proof explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_keyboard_owner",
        "pub fn collection_keyboard_owner",
        "pub fn set_next_collection_shortcut",
        "SetNextItemShortcut",
        "SetItemKeyOwner",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the keyboard-owner slice is a shared helper or generic key-owner facade; unexpected `{needle}`"
        );
    }
}
