#[test]
fn imui_editor_proof_demo_keeps_collection_box_select_app_owned_and_explicit() {
    let source = include_str!("../src/imui_editor_proof_demo/collection.rs");

    for needle in [
        "Background drag now draws a marquee and updates grid selection app-locally while shared helper widening stays deferred until another first-party proof surface exists.",
        "const PROOF_COLLECTION_BOX_SELECT_DRAG_THRESHOLD_PX: f32 = 6.0;",
        "struct ProofCollectionBoxSelectSession {",
        "struct ProofCollectionBoxSelectState {",
        "fn proof_collection_box_select_selection(",
        "fn proof_collection_box_select_active_rect(",
        "imui_editor_proof_demo.model.authoring_parity.collection_box_select",
        "props.capture_phase_pointer_moves = true;",
        "cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {",
        "cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {",
        "cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {",
        "host.capture_pointer();",
        "host.release_pointer_capture();",
        "\"imui-editor-proof.authoring.imui.collection.box-select.scope\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.marquee\"",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the box-select proof explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_box_select",
        "pub fn collection_box_select",
        "struct ImUiCollectionBoxSelect",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the new slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
