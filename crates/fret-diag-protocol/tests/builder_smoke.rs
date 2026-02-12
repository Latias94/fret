use fret_diag_protocol::builder::{ScriptV2Builder, role_and_name, test_id, text_composition_is};
use fret_diag_protocol::{
    UiActionScriptV2, UiOverlayPlacementTraceQueryV1, UiShortcutRoutingTraceQueryV1,
};

#[test]
fn builder_v2_roundtrip_smoke() {
    let script = ScriptV2Builder::new()
        .click(test_id("todo-input"))
        .type_text("hello")
        .ime_preedit("東", Some((0, 3)))
        .assert(text_composition_is(test_id("todo-input"), true))
        .ime_commit("東京")
        .press_key("enter")
        .wait_shortcut_routing_trace(
            UiShortcutRoutingTraceQueryV1 {
                outcome: Some("command_dispatched".to_string()),
                ..UiShortcutRoutingTraceQueryV1::default()
            },
            60,
        )
        .wait_overlay_placement_trace(UiOverlayPlacementTraceQueryV1::default(), 60)
        .wait_exists(test_id("todo-item-4-done"), 60)
        .assert_exists(test_id("todo-item-4-done"))
        .wait_exists(role_and_name("button", "Remove"), 60)
        .capture_bundle(Some("builder-smoke".to_string()))
        .build();

    assert_eq!(script.schema_version, 2);

    let value_1 = serde_json::to_value(&script).expect("script must serialize");
    let script_2: UiActionScriptV2 =
        serde_json::from_value(value_1.clone()).expect("script must parse after serialize");
    let value_2 = serde_json::to_value(&script_2).expect("script must serialize again");

    assert_eq!(value_1, value_2);
}
