use serde_json::Value;

const DATA_TABLE_TORTURE_SCROLL_REFRESH_REDIRECT: &str =
    include_str!("../../../tools/diag-scripts/ui-gallery-data-table-torture-scroll-refresh.json");

fn steps(script: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(script)
        .expect("valid diag script json")
        .get("steps")
        .and_then(Value::as_array)
        .cloned()
        .expect("diag script steps array")
}

fn step_type(step: &Value) -> Option<&str> {
    step.get("type").and_then(Value::as_str)
}

#[test]
fn data_table_torture_scroll_refresh_perf_script_starts_on_target_page_without_nav_search() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-data-table-torture-scroll-refresh.json"
    );
    let value = serde_json::from_str::<Value>(script).expect("valid data-table perf script");
    let meta = value.get("meta").expect("data-table perf script meta");
    let env = meta
        .get("env_defaults")
        .and_then(Value::as_object)
        .expect("data-table perf script env defaults");
    let steps = steps(script);

    assert_eq!(
        meta.get("name").and_then(Value::as_str),
        Some("ui-gallery-data-table-torture-scroll-refresh"),
        "data-table perf script should keep a stable promoted name",
    );
    assert_eq!(
        env.get("FRET_UI_GALLERY_START_PAGE")
            .and_then(Value::as_str),
        Some("data_table_torture"),
        "data-table perf script should start on data_table_torture",
    );

    let target_hints = meta
        .get("target_hints")
        .and_then(Value::as_array)
        .expect("data-table perf script target hints");
    assert!(
        target_hints.iter().any(|hint| {
            hint.as_str().is_some_and(|hint| {
                hint.contains("nav search and page-switch transitions do not dominate")
            })
        }),
        "data-table perf script should document why it starts directly on the target page",
    );

    assert!(
        script.contains("\"ui-gallery-page-data-table-torture\"")
            && script.contains("\"ui-gallery-data-table-torture-root\"")
            && script.contains("\"ui-gallery-data-table-row-0\""),
        "data-table perf script should wait for the page, root, and row anchors before measurement",
    );
    assert!(
        !script.contains("\"ui-gallery-nav-search\"")
            && !script.contains("\"ui-gallery-nav-data-table-torture\"")
            && !script.contains("\"text\": \"data_table_torture\""),
        "data-table perf script should not reintroduce nav-search/page-switch setup",
    );

    let reset_indices = steps
        .iter()
        .enumerate()
        .filter_map(|(idx, step)| (step_type(step) == Some("reset_diagnostics")).then_some(idx))
        .collect::<Vec<_>>();
    assert_eq!(
        reset_indices.len(),
        2,
        "data-table perf script should warm the page first, then reset diagnostics for the measured scroll window",
    );

    let capture_index = steps
        .iter()
        .position(|step| {
            step_type(step) == Some("capture_bundle")
                && step.get("label").and_then(Value::as_str)
                    == Some("ui-gallery-data-table-torture-scroll-refresh")
        })
        .expect("data-table perf script capture bundle step");
    assert!(
        reset_indices[1] < capture_index,
        "data-table perf script should capture after the second reset_diagnostics",
    );
}

#[test]
fn data_table_torture_scroll_refresh_legacy_redirect_points_at_the_promoted_script() {
    assert!(
        DATA_TABLE_TORTURE_SCROLL_REFRESH_REDIRECT.contains("\"kind\": \"script_redirect\"")
            && DATA_TABLE_TORTURE_SCROLL_REFRESH_REDIRECT.contains(
                "\"to\": \"tools/diag-scripts/ui-gallery/perf/ui-gallery-data-table-torture-scroll-refresh.json\""
            ),
        "legacy data-table scroll-refresh redirect should keep pointing at the promoted perf script",
    );
}
