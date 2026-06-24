use serde_json::Value;

const CHROME_TORTURE_STEADY_DIRECT_ENTRY_SUITE: &str = include_str!(
    "../../../tools/diag-scripts/suites/perf-ui-gallery-chrome-torture-steady-direct-entry/suite.json"
);
const CHROME_TORTURE_STEADY_DIRECT_ENTRY_REDIRECT: &str =
    include_str!("../../../tools/diag-scripts/ui-gallery-chrome-torture-steady-direct-entry.json");

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

fn target_id(step: &Value) -> Option<&str> {
    step.get("target")
        .and_then(|target| target.get("id"))
        .and_then(Value::as_str)
}

#[test]
fn chrome_torture_steady_direct_entry_perf_script_starts_on_target_page_without_nav_search() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-steady-direct-entry.json"
    );
    let value = serde_json::from_str::<Value>(script).expect("valid chrome direct-entry script");
    let meta = value.get("meta").expect("chrome direct-entry script meta");
    let env = meta
        .get("env_defaults")
        .and_then(Value::as_object)
        .expect("chrome direct-entry script env defaults");
    let steps = steps(script);

    assert_eq!(
        meta.get("name").and_then(Value::as_str),
        Some("ui-gallery-chrome-torture-steady-direct-entry"),
        "chrome direct-entry perf script should keep a stable promoted name",
    );
    assert_eq!(
        env.get("FRET_UI_GALLERY_START_PAGE")
            .and_then(Value::as_str),
        Some("chrome_torture"),
        "chrome direct-entry perf script should start on chrome_torture",
    );

    let required_launch_features = meta
        .get("required_launch_features")
        .and_then(Value::as_array)
        .expect("chrome direct-entry script launch features");
    assert_eq!(
        required_launch_features
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>(),
        vec![
            "gallery-dev",
            "gallery-ai",
            "gallery-chart",
            "gallery-web-ime-harness"
        ],
        "chrome direct-entry perf script should keep the same launch feature contract as the steady chrome harness",
    );

    let target_hints = meta
        .get("target_hints")
        .and_then(Value::as_array)
        .expect("chrome direct-entry script target hints");
    assert!(
        target_hints.iter().any(|hint| {
            hint.as_str().is_some_and(|hint| {
                hint.contains("nav search and page-switch transitions do not dominate")
            })
        }),
        "chrome direct-entry perf script should document why it starts directly on the target page",
    );
    assert!(
        target_hints.iter().any(|hint| {
            hint.as_str()
                .is_some_and(|hint| hint.contains("steady direct-entry chrome surface"))
        }),
        "chrome direct-entry perf script should document the direct-entry steady measurement surface",
    );

    assert!(
        script.contains("\"ui-gallery-page-chrome-torture\"")
            && script.contains("\"ui-gallery-chrome-torture-root\"")
            && script.contains("\"ui-gallery-chrome-btn-1\"")
            && script.contains("\"ui-gallery-chrome-btn-2\"")
            && script.contains("\"ui-gallery-chrome-btn-3\""),
        "chrome direct-entry perf script should wait for the page and chrome anchors before the measured interaction window",
    );
    assert!(
        !script.contains("\"ui-gallery-nav-search\"")
            && !script.contains("\"ui-gallery-nav-chrome-torture\"")
            && !script.contains("\"text\": \"chrome_torture\""),
        "chrome direct-entry perf script should not reintroduce nav-search/page-switch setup",
    );

    let reset_indices = steps
        .iter()
        .enumerate()
        .filter_map(|(idx, step)| (step_type(step) == Some("reset_diagnostics")).then_some(idx))
        .collect::<Vec<_>>();
    assert_eq!(
        reset_indices.len(),
        2,
        "chrome direct-entry perf script should warm the page first, then reset diagnostics for the measured steady window",
    );

    let capture_index = steps
        .iter()
        .position(|step| {
            step_type(step) == Some("capture_bundle")
                && step.get("label").and_then(Value::as_str)
                    == Some("ui-gallery-chrome-torture-steady-direct-entry")
        })
        .expect("chrome direct-entry perf script capture bundle step");
    assert!(
        reset_indices[1] < capture_index,
        "chrome direct-entry perf script should capture after the second reset_diagnostics",
    );
}

#[test]
fn chrome_torture_transition_perf_script_keeps_nav_transition_setup() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-steady.json"
    );

    assert!(
        script.contains("\"ui-gallery-nav-search\"")
            && script.contains("\"ui-gallery-nav-chrome-torture\"")
            && script.contains("\"text\": \"chrome_torture\""),
        "chrome transition perf script should keep the nav-search/page-switch setup steps",
    );
    assert!(
        script.contains("\"ui-gallery-chrome-btn-1\"")
            && script.contains("\"label\": \"ui-gallery-chrome-torture-steady\""),
        "chrome transition perf script should keep the measured interaction and capture labels stable",
    );
}

#[test]
fn chrome_torture_steady_direct_entry_perf_suite_stays_promoted_in_registry() {
    let suite = serde_json::from_str::<Value>(CHROME_TORTURE_STEADY_DIRECT_ENTRY_SUITE)
        .expect("valid chrome direct-entry suite");
    let registry =
        serde_json::from_str::<Value>(include_str!("../../../tools/diag-scripts/index.json"))
            .expect("valid diag script registry");

    assert_eq!(
        suite.get("kind").and_then(Value::as_str),
        Some("diag_script_suite_manifest"),
        "chrome direct-entry perf suite should stay on the suite-manifest format",
    );

    let scripts = suite
        .get("scripts")
        .and_then(Value::as_array)
        .expect("chrome direct-entry perf suite scripts");
    assert_eq!(
        scripts.iter().filter_map(Value::as_str).collect::<Vec<_>>(),
        vec![
            "tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-steady-direct-entry.json"
        ],
        "chrome direct-entry perf suite should keep the promoted direct-entry script path stable",
    );

    let registry_entries = registry
        .get("scripts")
        .and_then(Value::as_array)
        .expect("diag script registry entries");
    let entry = registry_entries
        .iter()
        .find(|entry| {
            entry.get("path").and_then(Value::as_str)
                == Some("tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-steady-direct-entry.json")
        })
        .expect("registry should promote the chrome direct-entry perf script");

    assert!(
        entry
            .get("suite_memberships")
            .and_then(Value::as_array)
            .is_some_and(|suite_memberships| {
                suite_memberships.iter().any(|suite_name| {
                    suite_name.as_str()
                        == Some("perf-ui-gallery-chrome-torture-steady-direct-entry")
                })
            }),
        "registry should keep the chrome direct-entry perf suite membership",
    );
}

#[test]
fn chrome_torture_steady_direct_entry_legacy_redirect_points_at_the_promoted_script() {
    assert!(
        CHROME_TORTURE_STEADY_DIRECT_ENTRY_REDIRECT.contains(
            "\"kind\": \"script_redirect\""
        ) && CHROME_TORTURE_STEADY_DIRECT_ENTRY_REDIRECT.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-steady-direct-entry.json\""
        ),
        "legacy chrome direct-entry redirect should keep pointing at the promoted perf script",
    );
}
