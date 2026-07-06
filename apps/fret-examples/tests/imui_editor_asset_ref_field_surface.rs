#[test]
fn imui_editor_proof_demo_mounts_asset_ref_field_as_ui_shell() {
    let main_source = include_str!("../src/imui_editor_proof_demo.rs");
    let inspector_source = include_str!("../src/imui_editor_proof_demo/editor_inspector.rs");
    let material_router_source = include_str!("../src/imui_editor_proof_demo/editor_material.rs");
    let material_source = include_str!("../src/imui_editor_proof_demo/editor_material/surface.rs");
    let owner_source = include_str!("../src/imui_editor_proof_demo/asset_ref.rs");
    let model_owner_source = include_str!("../src/imui_editor_proof_demo/editor_model_owner.rs");
    let compact_owner_source: String = owner_source.split_whitespace().collect();

    for needle in ["mod asset_ref;", "render_editor_inspector_surface("] {
        assert!(
            main_source.contains(needle),
            "imui_editor_proof_demo should keep the AssetRefField proof reachable through the editor shell; missing `{needle}`"
        );
    }

    for needle in ["render_editor_material_surface(cx, panel_cx)"] {
        assert!(
            inspector_source.contains(needle),
            "editor inspector should route Material without eagerly wiring AssetRefField models; missing `{needle}`"
        );
    }

    for unexpected in [
        "use super::asset_ref;",
        "asset_slot: asset_ref::asset_slot_model(cx),",
        "asset_action: asset_ref::asset_action_model(cx),",
        "asset_slot: models.asset_slot.clone(),",
        "asset_action: models.asset_action.clone(),",
        "EditorMaterialModels {",
    ] {
        assert!(
            !inspector_source.contains(unexpected),
            "editor inspector should no longer own AssetRefField model wiring; unexpected `{unexpected}`"
        );
    }

    for needle in [
        "mod surface;",
        "pub use surface::{EditorMaterialModels, EditorMaterialSurface, render_editor_material_surface};",
    ] {
        assert!(
            material_router_source.contains(needle),
            "editor material router should only re-export the surface owner; missing `{needle}`"
        );
    }

    for needle in [
        "use super::super::asset_ref;",
        "fn editor_material_models(cx: &mut ElementContext<'_, KernelApp>) -> EditorMaterialModels",
        "asset_slot: asset_ref::asset_slot_model(cx),",
        "asset_action: asset_ref::asset_action_model(cx),",
        "let material_show_all = panel_cx.matches(\"material\");",
        "asset_ref: material_show_all",
        "let models = editor_material_models(cx);",
        "asset_ref::push_material_rows(",
        "models.asset_slot.clone(),",
        "models.asset_action.clone(),",
    ] {
        assert!(
            material_source.contains(needle),
            "editor material owner should own AssetRefField model wiring and route rows through asset_ref; missing `{needle}`"
        );
    }

    for needle in [
        "AssetRefField, AssetRefFieldOptions, AssetRefFieldValue",
        "OnAssetRefFieldAction",
        "fn asset_ref_value(path: &str) -> Option<AssetRefFieldValue>",
        "fn record_action(",
        "use super::editor_model_owner::EditorProofModelOwner;",
        "EditorProofModelOwner::new(host.models_mut()).record_asset_ref_action(",
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
    for needle in [
        "pub(super) struct EditorProofModelOwner<'a>",
        "pub(super) fn record_asset_ref_action(",
        "fn replace_string(",
    ] {
        assert!(
            model_owner_source.contains(needle),
            "imui_editor_proof_demo editor model owner should own asset-ref action writes; missing `{needle}`"
        );
    }

    for unexpected in ["host.models_mut().update", "models_mut().update"] {
        assert!(
            !owner_source.contains(unexpected),
            "AssetRefField proof should route model writes through EditorProofModelOwner; unexpected `{unexpected}`"
        );
    }

    assert!(
        compact_owner_source.contains("usefret_ui_kit::IntoUiElementas_;"),
        "asset-ref owner should import `IntoUiElement` explicitly",
    );
    assert!(
        !owner_source.contains("use fret::component::prelude::*;"),
        "asset-ref owner should not rely on the broad component prelude",
    );

    for unexpected in [
        "fret_ui_assets",
        "AssetCache",
        "ResolvedAsset",
        "QueryState",
    ] {
        assert!(
            !main_source.contains(unexpected)
                && !inspector_source.contains(unexpected)
                && !material_router_source.contains(unexpected)
                && !material_source.contains(unexpected)
                && !owner_source.contains(unexpected),
            "AssetRefField proof must stay caller-owned and avoid asset-system coupling; unexpected `{unexpected}`"
        );
    }
}
