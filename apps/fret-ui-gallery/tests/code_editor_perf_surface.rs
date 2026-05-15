use serde_json::Value;

const OVERLAY_ENV: &str = "FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY";

fn assert_script_disables_torture_overlay(name: &str, script: &str) {
    let value = serde_json::from_str::<Value>(script).expect("valid diag script json");
    let overlay = value
        .get("meta")
        .and_then(|meta| meta.get("env_defaults"))
        .and_then(|env| env.get(OVERLAY_ENV))
        .and_then(Value::as_str);

    assert_eq!(
        overlay,
        Some("0"),
        "{name} should disable the diagnostic editor torture overlay for perf contract probes",
    );
}

#[test]
fn code_editor_perf_contract_scripts_disable_torture_overlay_by_default() {
    for (name, script) in [
        (
            "autoscroll steady",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json"
            ),
        ),
        (
            "autoscroll typical",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-typical.json"
            ),
        ),
        (
            "complex wheel",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json"
            ),
        ),
        (
            "resize jitter",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json"
            ),
        ),
    ] {
        assert_script_disables_torture_overlay(name, script);
    }
}
