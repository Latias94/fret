use std::sync::Arc;

use fret::AppComponentCx;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::{UiHost, action};
use fret_ui_editor::composites::PropertyGridRowCx;
use fret_ui_editor::controls::{
    AssetRefField, AssetRefFieldOptions, AssetRefFieldValue, FieldStatus, OnAssetRefFieldAction,
};
use fret_ui_kit::IntoUiElement as _;

use super::editor_model_owner::EditorProofModelOwner;

pub(super) const DEFAULT_ASSET: &str = "textures/default/basecolor.ktx2";
const CHOSEN_ASSET: &str = "textures/props/brushed-metal-albedo.ktx2";

pub(super) fn asset_slot_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    super::named_demo_state(cx, "imui_editor_proof_demo.model.asset_slot", |cx| {
        cx.app.models_mut().insert(DEFAULT_ASSET.to_string())
    })
}

pub(super) fn asset_action_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    super::named_demo_state(cx, "imui_editor_proof_demo.model.asset_action", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

pub(super) fn push_material_rows(
    rows: &mut Vec<AnyElement>,
    cx: &mut AppComponentCx<'_>,
    row_cx: &PropertyGridRowCx,
    asset_slot_model: Model<String>,
    asset_action_model: Model<String>,
) {
    let assigned_asset = super::editor_string_model_readout(cx, &asset_slot_model);
    let asset_value = asset_ref_value(&assigned_asset);
    let choose_slot_model = asset_slot_model.clone();
    let choose_action_model = asset_action_model.clone();
    let reveal_slot_model = asset_slot_model.clone();
    let reveal_action_model = asset_action_model.clone();
    let clear_slot_model = asset_slot_model.clone();
    let clear_action_model = asset_action_model.clone();

    let on_choose: OnAssetRefFieldAction = Arc::new(move |host, action_cx| {
        record_action(
            host,
            action_cx,
            &choose_slot_model,
            &choose_action_model,
            "Chose alternate base texture",
            Some(CHOSEN_ASSET),
        );
    });
    let on_reveal: OnAssetRefFieldAction = Arc::new(move |host, action_cx| {
        record_action(
            host,
            action_cx,
            &reveal_slot_model,
            &reveal_action_model,
            "Reveal requested",
            None,
        );
    });
    let on_clear: OnAssetRefFieldAction = Arc::new(move |host, action_cx| {
        record_action(
            host,
            action_cx,
            &clear_slot_model,
            &clear_action_model,
            "Cleared base texture",
            Some(""),
        );
    });

    rows.push(row_cx.row(
        cx,
        |cx| row_cx.label_text(cx, "Base texture"),
        |cx| {
            AssetRefField::new(asset_value)
                .options(AssetRefFieldOptions {
                    test_id: Some(Arc::from("imui-editor-proof.editor.material.base-texture")),
                    value_test_id: Some(Arc::from(
                        "imui-editor-proof.editor.material.base-texture.value",
                    )),
                    choose_test_id: Some(Arc::from(
                        "imui-editor-proof.editor.material.base-texture.choose",
                    )),
                    reveal_test_id: Some(Arc::from(
                        "imui-editor-proof.editor.material.base-texture.reveal",
                    )),
                    clear_test_id: Some(Arc::from(
                        "imui-editor-proof.editor.material.base-texture.clear",
                    )),
                    status: Some(FieldStatus::Dirty),
                    on_choose: Some(on_choose),
                    on_reveal: Some(on_reveal),
                    on_clear: Some(on_clear),
                    ..Default::default()
                })
                .into_element(cx)
        },
    ));

    let asset_action = super::editor_string_model_readout(cx, &asset_action_model);
    rows.push(row_cx.row(
        cx,
        |cx| row_cx.label_text(cx, "Texture action"),
        move |cx| {
            let readout = if asset_action.trim().is_empty() {
                "Idle".to_string()
            } else {
                asset_action.clone()
            };
            super::proof_compact_readout(
                cx,
                readout,
                Some(Arc::from(
                    "imui-editor-proof.editor.material.base-texture.action",
                )),
            )
        },
    ));
}

fn asset_ref_value(path: &str) -> Option<AssetRefFieldValue> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let label = trimmed
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .filter(|label| !label.is_empty())
        .unwrap_or(trimmed);
    Some(
        AssetRefFieldValue::new(label)
            .path(trimmed)
            .icon(fret_icons::ids::ui::FILE),
    )
}

fn record_action(
    host: &mut dyn action::UiActionHost,
    _action_cx: action::ActionCx,
    asset_slot_model: &Model<String>,
    action_model: &Model<String>,
    action_label: &'static str,
    next_asset: Option<&'static str>,
) {
    EditorProofModelOwner::new(host.models_mut()).record_asset_ref_action(
        asset_slot_model,
        action_model,
        action_label,
        next_asset,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_ref_value_uses_filename_label_without_asset_semantics() {
        let value = asset_ref_value(DEFAULT_ASSET).expect("default asset should map to a value");
        assert_eq!(value.label.as_ref(), "basecolor.ktx2");
        assert_eq!(value.path.as_deref(), Some(DEFAULT_ASSET));
    }

    #[test]
    fn blank_asset_ref_value_is_empty_slot() {
        assert!(asset_ref_value("").is_none());
        assert!(asset_ref_value("   ").is_none());
    }
}
