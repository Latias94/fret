#[test]
fn imui_editor_proof_demo_mounts_asset_ref_field_as_ui_shell() {
    let source = include_str!("../src/imui_editor_proof_demo.rs");

    for needle in [
        "AssetRefField, AssetRefFieldOptions, AssetRefFieldValue",
        "OnAssetRefFieldAction",
        "fn editor_asset_ref_value(path: &str) -> Option<AssetRefFieldValue>",
        "fn record_editor_asset_ref_action(",
        "let show_asset_ref = material_show_all",
        "AssetRefField::new(asset_value)",
        "imui-editor-proof.editor.material.base-texture",
        "imui-editor-proof.editor.material.base-texture.value",
        "imui-editor-proof.editor.material.base-texture.choose",
        "imui-editor-proof.editor.material.base-texture.reveal",
        "imui-editor-proof.editor.material.base-texture.clear",
        "imui-editor-proof.editor.material.base-texture.action",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the AssetRefField proof surface visible; missing `{needle}`"
        );
    }

    for unexpected in [
        "fret_ui_assets",
        "AssetCache",
        "ResolvedAsset",
        "QueryState",
    ] {
        assert!(
            !source.contains(unexpected),
            "AssetRefField proof must stay caller-owned and avoid asset-system coupling; unexpected `{unexpected}`"
        );
    }
}
