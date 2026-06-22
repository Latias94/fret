use serde_json::Value;

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

fn test_id_target(step: &Value) -> Option<&str> {
    step.get("target")
        .and_then(|target| target.get("id"))
        .and_then(Value::as_str)
}

fn contains_test_id(value: &Value, needle: &str) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            (key == "id" && value.as_str() == Some(needle)) || contains_test_id(value, needle)
        }),
        Value::Array(values) => values.iter().any(|value| contains_test_id(value, needle)),
        _ => false,
    }
}

fn contains_step_type(value: &Value, needle: &str) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            (key == "type" && value.as_str() == Some(needle)) || contains_step_type(value, needle)
        }),
        Value::Array(values) => values.iter().any(|value| contains_step_type(value, needle)),
        _ => false,
    }
}

fn first_step_index(steps: &[Value], expected_type: &str, expected_target: Option<&str>) -> usize {
    steps
        .iter()
        .position(|step| {
            step_type(step) == Some(expected_type)
                && expected_target.is_none_or(|target| test_id_target(step) == Some(target))
        })
        .unwrap_or_else(|| {
            panic!("missing step type `{expected_type}` target `{expected_target:?}`")
        })
}

fn assert_jump_input_seeded_before_jump_click(script_name: &str, steps: &[Value]) {
    let input_set = steps
        .iter()
        .position(|step| {
            step_type(step) == Some("set_text_value")
                && test_id_target(step) == Some("ui-gallery-virtual-list-jump-input")
                && step.get("text").and_then(Value::as_str) == Some("9000")
        })
        .unwrap_or_else(|| {
            panic!("{script_name} should deterministically set the jump input to row 9000")
        });
    let jump_click = first_step_index(steps, "click", Some("ui-gallery-virtual-list-jump-button"));

    assert!(
        input_set < jump_click,
        "{script_name} should set the jump input to row 9000 before clicking Jump",
    );
}

#[test]
fn virtual_list_torture_scripts_seed_jump_input_before_waiting_for_row_9000() {
    for (name, script) in [
        (
            "steady",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json"
            ),
        ),
        (
            "canonical",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture.json"
            ),
        ),
    ] {
        let steps = steps(script);
        assert_jump_input_seeded_before_jump_click(name, &steps);
    }
}

#[test]
fn virtual_list_steady_script_keeps_jump_input_setup_outside_perf_capture_window() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json"
    );
    let steps = steps(script);
    let reset = first_step_index(&steps, "reset_diagnostics", None);
    let jump_click = first_step_index(&steps, "click", Some("ui-gallery-virtual-list-jump-button"));
    let set_9000 = steps
        .iter()
        .position(|step| {
            step_type(step) == Some("set_text_value")
                && test_id_target(step) == Some("ui-gallery-virtual-list-jump-input")
                && step.get("text").and_then(Value::as_str) == Some("9000")
        })
        .expect("steady script should set the row 9000 target");

    assert!(
        set_9000 < reset && reset < jump_click,
        "steady perf script should prepare the jump input before reset_diagnostics, then measure jump/bottom behavior",
    );
}

#[test]
fn virtual_list_torture_correctness_scripts_target_inner_list_scroll_after_content_bypass() {
    for (name, script) in [
        (
            "retained selected action state bounce",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-retained-selected-action-state-bounce.json"
            ),
        ),
        (
            "retained collection metadata bounce",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-retained-collection-metadata-bounce.json"
            ),
        ),
        (
            "window boundary scroll",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-window-boundary-scroll.json"
            ),
        ),
        (
            "window boundary scroll retained",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-window-boundary-scroll-retained.json"
            ),
        ),
        (
            "small scroll no window shifts",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-small-scroll-no-window-shifts.json"
            ),
        ),
    ] {
        let parsed = serde_json::from_str::<Value>(script).expect("valid virtual-list script json");
        assert!(
            !contains_test_id(&parsed, "ui-gallery-content-viewport-virtual_list_torture"),
            "{name} should not target the removed outer content viewport"
        );
        assert!(
            !contains_step_type(&parsed, "wait_semantics_scroll_stable"),
            "{name} should not wait for ScrollArea semantics on the inner VirtualList"
        );
        assert!(
            !contains_test_id(&parsed, "ui-gallery-virtual-list-row-2-label.chrome"),
            "{name} should target direct row-action pressables, not removed chrome wrappers"
        );
        assert!(
            contains_test_id(&parsed, "ui-gallery-virtual-list-root"),
            "{name} should target the inner virtual-list scroll root"
        );
    }
}
