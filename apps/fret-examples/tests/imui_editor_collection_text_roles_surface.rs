#[test]
fn imui_editor_proof_collection_fixed_text_uses_shared_roles() {
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/models.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/readouts.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename.rs"),
    );

    for needle in [
        "proof_compact_readout_element",
        "proof_section_chrome_label",
        "fn proof_collection_readout_text(",
        "fn proof_collection_section_label(",
        "proof_collection_section_label(\n        ui,\n        \"Collection-first asset browser proof\",",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_assets_line(&collection_assets),",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_visible_order_line(&collection_assets),",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_select_all_line(),",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_rename_line(),",
        "proof_collection_readout_text(\n        ui,\n        proof_collection_command_status_line(&collection_command_status),",
        "ui.text_wrapped(\n                                                                    \"Inline rename stays app-owned: Enter commits; Escape or blur cancels without widening shared IMUI helpers.\",",
        "proof_collection_readout_text(\n                                                                ui,\n                                                                format!(",
        "\"imui-editor-proof.authoring.imui.collection.asset.metadata\"",
        "proof_collection_readout_text(\n                                                                ui,\n                                                                asset.path.clone(),",
        "\"imui-editor-proof.authoring.imui.collection.asset.path\"",
        "proof_collection_readout_text(\n            ui,\n            format!(",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.selection-readout\"",
        "proof_collection_readout_text(\n        ui,\n        visible_collection_status,",
        "\"imui-editor-proof.authoring.imui.collection.drop-status-readout\"",
    ] {
        assert!(
            source.contains(needle),
            "collection proof fixed text should use shared text-role helpers; missing `{needle}`"
        );
    }

    for needle in [
        "ui.text(\"Collection-first asset browser proof\");",
        "ui.text(proof_collection_assets_line(&collection_assets));",
        "ui.text(proof_collection_visible_order_line(&collection_assets));",
        "ui.text(proof_collection_select_all_line());",
        "ui.text(proof_collection_rename_line());",
        "ui.text(proof_collection_command_status_line(",
        "ui.text(\n                                                                    \"Inline rename stays app-owned: Enter commits; Escape or blur cancels without widening shared IMUI helpers.\",",
        "ui.text(format!(\n                                                                \"{} | {} KiB\",",
        "ui.text(asset.path.clone());",
        "ui.text(format!(\n            \"Selection: {} item(s)\",",
        "ui.text(visible_collection_status);",
    ] {
        assert!(
            !source.contains(needle),
            "collection proof fixed text should not use bare IMUI text; unexpected `{needle}`"
        );
    }
}
