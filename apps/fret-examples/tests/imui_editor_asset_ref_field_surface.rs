#[test]
fn imui_editor_proof_demo_mounts_asset_ref_field_as_ui_shell() {
    let main_source = include_str!("../src/imui_editor_proof_demo.rs");
    let owner_source = include_str!("../src/imui_editor_proof_demo/asset_ref.rs");

    for needle in [
        "mod asset_ref;",
        "let editor_asset_slot_model = asset_ref::asset_slot_model(cx);",
        "let editor_asset_action_model = asset_ref::asset_action_model(cx);",
        "let show_asset_ref = material_show_all",
        "asset_ref::push_material_rows(",
    ] {
        assert!(
            main_source.contains(needle),
            "imui_editor_proof_demo should keep the AssetRefField proof wired through its owner module; missing `{needle}`"
        );
    }

    for needle in [
        "AssetRefField, AssetRefFieldOptions, AssetRefFieldValue",
        "OnAssetRefFieldAction",
        "fn asset_ref_value(path: &str) -> Option<AssetRefFieldValue>",
        "fn record_action(",
        "AssetRefField::new(asset_value)",
        "imui-editor-proof.editor.material.base-texture",
        "imui-editor-proof.editor.material.base-texture.value",
        "imui-editor-proof.editor.material.base-texture.choose",
        "imui-editor-proof.editor.material.base-texture.reveal",
        "imui-editor-proof.editor.material.base-texture.clear",
        "imui-editor-proof.editor.material.base-texture.action",
    ] {
        assert!(
            owner_source.contains(needle),
            "imui_editor_proof_demo asset-ref owner should keep the proof surface visible; missing `{needle}`"
        );
    }

    for unexpected in [
        "fret_ui_assets",
        "AssetCache",
        "ResolvedAsset",
        "QueryState",
    ] {
        assert!(
            !main_source.contains(unexpected) && !owner_source.contains(unexpected),
            "AssetRefField proof must stay caller-owned and avoid asset-system coupling; unexpected `{unexpected}`"
        );
    }
}
