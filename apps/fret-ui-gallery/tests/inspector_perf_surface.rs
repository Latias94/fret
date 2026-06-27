use serde_json::Value;

#[test]
fn inspector_scroll_direct_entry_perf_script_starts_on_target_page_without_nav_search() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll-direct-entry.json"
    );
    let value = serde_json::from_str::<Value>(script).expect("valid inspector direct-entry script");
    let meta = value
        .get("meta")
        .expect("inspector direct-entry script meta");
    let env = meta
        .get("env_defaults")
        .and_then(Value::as_object)
        .expect("inspector direct-entry script env defaults");
    let steps = value
        .get("steps")
        .and_then(Value::as_array)
        .expect("inspector direct-entry script steps");

    assert_eq!(
        meta.get("name").and_then(Value::as_str),
        Some("ui-gallery-inspector-torture-scroll-direct-entry"),
        "inspector direct-entry perf script should keep a stable promoted name",
    );
    assert_eq!(
        env.get("FRET_UI_GALLERY_START_PAGE")
            .and_then(Value::as_str),
        Some("inspector_torture"),
        "inspector direct-entry perf script should start on inspector_torture",
    );
    assert_eq!(
        env.get("FRET_UI_GALLERY_VIEW_CACHE_SHELL")
            .and_then(Value::as_str),
        Some("1"),
        "inspector direct-entry perf script should default the sidebar shell cache so the direct-entry measurement stays on the stabilized shell contract",
    );
    assert_eq!(
        env.get("FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE")
            .and_then(Value::as_str),
        Some("0"),
        "inspector direct-entry perf script should keep the inspector keep-alive budget at 0 so the steady scroll surface does not get widened by retained-window stress",
    );
    assert!(
        !env.contains_key("FRET_UI_GALLERY_VIEW_CACHE"),
        "inspector direct-entry perf script should not silently enable global view-cache; shell cache policy is a separate measurement contract",
    );
    let target_hints = meta
        .get("target_hints")
        .and_then(Value::as_array)
        .expect("inspector direct-entry script target hints");
    assert!(
        target_hints.iter().any(|hint| {
            hint.as_str()
                .is_some_and(|hint| hint.contains("global view-cache remains opt-in"))
        }),
        "inspector direct-entry perf script should document that shell cache policy does not imply global view-cache activation",
    );
    assert!(
        target_hints.iter().any(|hint| {
            hint.as_str()
                .is_some_and(|hint| hint.contains("keep-alive budget at 0"))
        }),
        "inspector direct-entry perf script should document that the direct-entry surface keeps keep-alive at 0",
    );
    assert!(
        script.contains("\"ui-gallery-inspector-root\"")
            && script.contains("\"ui-gallery-inspector-row-0\"")
            && !script.contains("\"ui-gallery-inspector-row-0-label\""),
        "inspector direct-entry perf script should wait for both the inspector root and first row anchor",
    );
    assert!(
        script.contains("\"ui-gallery-inspector-row-0\"") && script.contains("\"type\": \"click\""),
        "inspector direct-entry perf script should click the first row before the measured scroll window so focus is already settled",
    );
    assert!(
        script.contains("\"kind\": \"focus_is\"")
            && script.contains("\"ui-gallery-inspector-root\""),
        "inspector direct-entry perf script should wait for the stable inspector root to become focused before the measured scroll window",
    );
    assert!(
        !script.contains("\"ui-gallery-nav-search\"")
            && !script.contains("\"ui-gallery-nav-inspector-torture\"")
            && !script.contains("\"text\": \"inspector torture\""),
        "inspector direct-entry perf script should not reintroduce nav-search/page-switch setup",
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
        "inspector direct-entry perf script should warm the page first, then reset diagnostics after the stable-focus settle window",
    );

    let focus_wait_index = steps
        .iter()
        .position(|step| {
            step.get("type").and_then(Value::as_str) == Some("wait_until")
                && step
                    .get("predicate")
                    .and_then(|predicate| predicate.get("kind"))
                    .and_then(Value::as_str)
                    == Some("focus_is")
                && step
                    .get("predicate")
                    .and_then(|predicate| predicate.get("target"))
                    .and_then(|target| target.get("id"))
                    .and_then(Value::as_str)
                    == Some("ui-gallery-inspector-root")
        })
        .expect("inspector direct-entry perf script focus wait step");
    let capture_index = steps
        .iter()
        .position(|step| {
            step.get("type").and_then(Value::as_str) == Some("capture_bundle")
                && step.get("label").and_then(Value::as_str)
                    == Some("ui-gallery-inspector-torture-scroll-direct-entry")
        })
        .expect("inspector direct-entry perf script capture bundle step");
    assert!(
        reset_indices[1] < capture_index,
        "inspector direct-entry perf script should capture after the second reset_diagnostics",
    );
    assert!(
        reset_indices[0] < focus_wait_index && focus_wait_index < reset_indices[1],
        "inspector direct-entry perf script should settle focus before the measured reset window",
    );
}

#[test]
fn inspector_scroll_perf_script_keeps_nav_transition_setup() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll.json"
    );
    let value = serde_json::from_str::<Value>(script).expect("valid inspector transition script");
    let meta = value.get("meta").expect("inspector transition script meta");
    let env = meta
        .get("env_defaults")
        .and_then(Value::as_object)
        .expect("inspector transition script env defaults");

    assert!(
        script.contains("\"ui-gallery-nav-search\"")
            && script.contains("\"ui-gallery-nav-inspector-torture\"")
            && script.contains("\"text\": \"inspector torture\""),
        "inspector transition perf script should keep the nav-search/page-switch setup steps",
    );
    assert_eq!(
        meta.get("name").and_then(Value::as_str),
        Some("ui-gallery-inspector-torture-scroll"),
        "inspector transition perf script should keep a stable promoted name",
    );
    assert_eq!(
        env.get("FRET_UI_GALLERY_START_PAGE")
            .and_then(Value::as_str),
        Some("intro"),
        "inspector transition perf script should start from the lightweight intro page instead of inheriting the diagnostics default overlay page",
    );
    let target_hints = meta
        .get("target_hints")
        .and_then(Value::as_array)
        .expect("inspector transition script target hints");
    assert!(
        target_hints.iter().any(|hint| {
            hint.as_str()
                .is_some_and(|hint| hint.contains("diagnostics default overlay page"))
        }),
        "inspector transition perf script should document why the explicit intro start page exists",
    );

    let steps = value
        .get("steps")
        .and_then(Value::as_array)
        .expect("inspector transition script steps");
    let click_index = steps
        .iter()
        .position(|step| {
            step.get("type").and_then(Value::as_str) == Some("click")
                && step
                    .get("target")
                    .and_then(|target| target.get("id"))
                    .and_then(Value::as_str)
                    == Some("ui-gallery-nav-inspector-torture")
        })
        .expect("inspector transition perf script nav click");
    let capture_index = steps
        .iter()
        .position(|step| {
            step.get("type").and_then(Value::as_str) == Some("capture_bundle")
                && step.get("label").and_then(Value::as_str)
                    == Some("ui-gallery-inspector-torture-scroll")
        })
        .expect("inspector transition perf script capture bundle step");
    assert!(
        click_index < capture_index,
        "inspector transition perf script should still capture after the nav click path",
    );
}
