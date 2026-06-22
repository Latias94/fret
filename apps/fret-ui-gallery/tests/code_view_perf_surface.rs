use serde_json::Value;

#[test]
fn code_view_mount_direct_entry_perf_script_starts_on_target_page_without_nav_search() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount-direct-entry.json"
    );
    let value = serde_json::from_str::<Value>(script).expect("valid diag script json");
    let meta = value.get("meta").expect("direct-entry perf script meta");
    let env = meta
        .get("env_defaults")
        .and_then(Value::as_object)
        .expect("direct-entry perf script env defaults");
    let steps = value
        .get("steps")
        .and_then(Value::as_array)
        .expect("direct-entry perf script steps");

    assert_eq!(
        meta.get("name").and_then(Value::as_str),
        Some("ui-gallery-code-view-torture-mount-direct-entry"),
        "direct-entry perf script should keep a stable promoted name",
    );
    assert_eq!(
        env.get("FRET_UI_GALLERY_START_PAGE")
            .and_then(Value::as_str),
        Some("code_view_torture"),
        "direct-entry perf script should start on code_view_torture",
    );
    assert!(
        script.contains("\"ui-gallery-page-code-view-torture\"")
            && script.contains("\"ui-gallery-code-view-root\""),
        "direct-entry perf script should wait for both the page root and code-view root anchors",
    );
    assert!(
        !script.contains("\"ui-gallery-nav-search\"")
            && !script.contains("\"ui-gallery-nav-code-view-torture\"")
            && !script.contains("\"text\": \"code_view_torture\""),
        "direct-entry perf script should not reintroduce nav-search/page-switch setup",
    );

    let reset_indices = steps
        .iter()
        .enumerate()
        .filter_map(|(idx, step)| {
            (step.get("type").and_then(Value::as_str) == Some("reset_diagnostics")).then_some(idx)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        reset_indices.len(),
        2,
        "direct-entry perf script should warm the page first, then reset diagnostics for the measured mount window",
    );

    let capture_index = steps
        .iter()
        .position(|step| {
            step.get("type").and_then(Value::as_str) == Some("capture_bundle")
                && step.get("label").and_then(Value::as_str)
                    == Some("ui-gallery-code-view-torture-mount-direct-entry")
        })
        .expect("direct-entry perf script capture bundle step");
    assert!(
        reset_indices[1] < capture_index,
        "direct-entry perf script should capture after the second reset_diagnostics",
    );
}

#[test]
fn code_view_mount_perf_script_keeps_nav_transition_setup() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount.json"
    );
    let value = serde_json::from_str::<Value>(script).expect("valid transition perf script json");
    let meta = value.get("meta").expect("transition perf script meta");
    let env = meta
        .get("env_defaults")
        .and_then(Value::as_object)
        .expect("transition perf script env defaults");

    assert!(
        script.contains("\"ui-gallery-nav-search\"")
            && script.contains("\"ui-gallery-nav-code-view-torture\"")
            && script.contains("\"text\": \"code_view_torture\""),
        "transition perf script should keep the nav-search/page-switch setup steps",
    );
    assert_eq!(
        meta.get("name").and_then(Value::as_str),
        Some("ui-gallery-code-view-torture-mount"),
        "transition perf script should keep a stable promoted name",
    );
    assert_eq!(
        env.get("FRET_UI_GALLERY_START_PAGE")
            .and_then(Value::as_str),
        Some("intro"),
        "transition perf script should start from the lightweight intro page instead of inheriting the diagnostics default overlay page",
    );
    let target_hints = meta
        .get("target_hints")
        .and_then(Value::as_array)
        .expect("transition perf script target hints");
    assert!(
        target_hints.iter().any(|hint| {
            hint.as_str()
                .is_some_and(|hint| hint.contains("diagnostics default overlay page"))
        }),
        "transition perf script should document why the explicit intro start page exists",
    );

    let steps = value
        .get("steps")
        .and_then(Value::as_array)
        .expect("transition perf script steps");
    let click_index = steps
        .iter()
        .position(|step| {
            step.get("type").and_then(Value::as_str) == Some("click_stable")
                && step
                    .get("target")
                    .and_then(|target| target.get("id"))
                    .and_then(Value::as_str)
                    == Some("ui-gallery-nav-code-view-torture")
        })
        .expect("transition perf script nav click");
    let capture_index = steps
        .iter()
        .position(|step| {
            step.get("type").and_then(Value::as_str) == Some("capture_bundle")
                && step.get("label").and_then(Value::as_str)
                    == Some("ui-gallery-code-view-torture-mount")
        })
        .expect("transition perf script capture bundle step");
    assert!(
        click_index < capture_index,
        "transition perf script should still capture after the nav click path",
    );
}
