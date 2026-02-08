use fret_diag_protocol::{DiagTransportMessageV1, UiActionScriptV1, UiActionScriptV2};

fn assert_script_v1_roundtrip(json: &str) {
    let script_1: UiActionScriptV1 = serde_json::from_str(json).expect("script v1 must parse");
    assert_eq!(script_1.schema_version, 1);

    let value_1 = serde_json::to_value(&script_1).expect("script v1 must serialize");
    let script_2: UiActionScriptV1 =
        serde_json::from_value(value_1.clone()).expect("script v1 must parse after serialize");
    let value_2 = serde_json::to_value(&script_2).expect("script v1 must serialize again");

    assert_eq!(value_1, value_2);
}

fn assert_script_v2_roundtrip(json: &str) {
    let script_1: UiActionScriptV2 = serde_json::from_str(json).expect("script v2 must parse");
    assert_eq!(script_1.schema_version, 2);

    let value_1 = serde_json::to_value(&script_1).expect("script v2 must serialize");
    let script_2: UiActionScriptV2 =
        serde_json::from_value(value_1.clone()).expect("script v2 must parse after serialize");
    let value_2 = serde_json::to_value(&script_2).expect("script v2 must serialize again");

    assert_eq!(value_1, value_2);
}

#[test]
fn script_v1_roundtrip_todo_baseline() {
    assert_script_v1_roundtrip(include_str!(
        "../../../tools/diag-scripts/todo-baseline.json"
    ));
}

#[test]
fn script_v2_roundtrip_command_palette_shortcut_primary() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-command-palette-shortcut-primary.json"
    ));
}

#[test]
fn script_v2_roundtrip_chart_torture_pan_zoom() {
    assert_script_v2_roundtrip(include_str!(
        "../../../tools/diag-scripts/ui-gallery-chart-torture-pan-zoom.json"
    ));
}

#[test]
fn transport_message_roundtrip_envelope() {
    let message_1 = DiagTransportMessageV1 {
        schema_version: 1,
        r#type: "hello".to_string(),
        session_id: None,
        request_id: Some(1),
        payload: serde_json::json!({"client_kind":"tooling","capabilities":["inspect"]}),
    };

    let value_1 = serde_json::to_value(&message_1).expect("message must serialize");
    let message_2: DiagTransportMessageV1 =
        serde_json::from_value(value_1.clone()).expect("message must parse");
    let value_2 = serde_json::to_value(&message_2).expect("message must serialize again");

    assert_eq!(value_1, value_2);
}
