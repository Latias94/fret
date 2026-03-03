use fret_diag_protocol::{UiActionScriptV2, UiActionStepV2};

#[test]
fn capture_layout_sidecar_roundtrips() {
    let script = UiActionScriptV2 {
        schema_version: 2,
        meta: None,
        steps: vec![UiActionStepV2::CaptureLayoutSidecar {
            label: Some("layout-sidecar".to_string()),
            root_label_filter: Some("CodeEditor".to_string()),
        }],
    };

    let json = serde_json::to_string_pretty(&script).expect("serialize script");
    let parsed: UiActionScriptV2 = serde_json::from_str(&json).expect("deserialize script");
    assert_eq!(parsed.schema_version, 2);
    assert_eq!(parsed.steps.len(), 1);
    assert_eq!(
        serde_json::to_value(&parsed).unwrap(),
        serde_json::to_value(&script).unwrap()
    );
}
