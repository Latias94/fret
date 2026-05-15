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
