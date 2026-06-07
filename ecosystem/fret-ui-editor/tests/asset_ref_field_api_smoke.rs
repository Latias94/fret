use std::sync::Arc;

use fret_ui_editor::controls::{
    AssetRefField, AssetRefFieldOptions, AssetRefFieldValue, FieldStatus, OnAssetRefFieldAction,
};

#[allow(dead_code)]
fn asset_ref_field_accepts_ui_only_action_slots() {
    let value = AssetRefFieldValue::new("Base Color")
        .path("textures/default/basecolor.ktx2")
        .icon(fret_icons::ids::ui::FILE);
    let choose: OnAssetRefFieldAction = Arc::new(|_host, _action_cx| {});

    let _field = AssetRefField::new(Some(value)).options(AssetRefFieldOptions {
        test_id: Some(Arc::from("tests.asset_ref.base_color")),
        value_test_id: Some(Arc::from("tests.asset_ref.base_color.value")),
        choose_test_id: Some(Arc::from("tests.asset_ref.base_color.choose")),
        reveal_test_id: Some(Arc::from("tests.asset_ref.base_color.reveal")),
        clear_test_id: Some(Arc::from("tests.asset_ref.base_color.clear")),
        status: Some(FieldStatus::Dirty),
        on_choose: Some(choose),
        ..Default::default()
    });
}

#[test]
fn asset_ref_value_display_is_label_plus_optional_path() {
    let value = AssetRefFieldValue::new("Base Color").path("textures/default/basecolor.ktx2");
    assert_eq!(
        value.display_text().as_ref(),
        "Base Color - textures/default/basecolor.ktx2"
    );

    let label_only = AssetRefFieldValue::new("Base Color").path("Base Color");
    assert_eq!(label_only.display_text().as_ref(), "Base Color");
}

#[test]
fn asset_ref_options_default_to_caller_owned_actions() {
    let options = AssetRefFieldOptions::default();
    assert!(options.enabled);
    assert!(options.on_choose.is_none());
    assert!(options.on_reveal.is_none());
    assert!(options.on_clear.is_none());
    assert!(options.status.is_none());
    assert_eq!(options.placeholder.as_ref(), "No asset assigned");
}
