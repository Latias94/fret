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

#[test]
fn overlay_pointer_move_perf_cleanup_reenters_underlay_before_outside_press() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json"
    );
    let steps = steps(script);

    let capture_index = steps
        .iter()
        .position(|step| {
            step_type(step) == Some("capture_bundle")
                && step.get("label").and_then(Value::as_str)
                    == Some("ui-gallery-overlay-pointer-move-steady")
        })
        .expect("overlay pointer-move perf script should capture the steady-state bundle");

    let cleanup = &steps[capture_index + 1..];
    let reentry_index = cleanup
        .iter()
        .position(|step| {
            step_type(step) == Some("move_pointer")
                && test_id_target(step) == Some("ui-gallery-overlay-underlay")
        })
        .expect("overlay pointer-move perf cleanup should move the pointer back to the underlay");
    let click_index = cleanup
        .iter()
        .position(|step| {
            step_type(step) == Some("click")
                && test_id_target(step) == Some("ui-gallery-overlay-underlay")
        })
        .expect("overlay pointer-move perf cleanup should dismiss by clicking the underlay");

    assert!(
        reentry_index < click_index,
        "overlay pointer-move sweep can end outside the window; cleanup must re-enter the underlay before outside-press dismissal",
    );

    let wait_after_reentry = cleanup
        .get(reentry_index + 1)
        .expect("cleanup should wait one frame after pointer re-entry");
    assert_eq!(step_type(wait_after_reentry), Some("wait_frames"));
    assert_eq!(wait_after_reentry.get("n").and_then(Value::as_u64), Some(1));

    let dismissed_wait = cleanup[click_index + 1..]
        .iter()
        .find(|step| {
            step_type(step) == Some("wait_until")
                && step
                    .get("predicate")
                    .and_then(|predicate| predicate.get("target"))
                    .and_then(|target| target.get("id"))
                    .and_then(Value::as_str)
                    == Some("ui-gallery-popover-dismissed")
        })
        .expect("cleanup should wait for the popover dismissed flag after outside press");
    assert_eq!(
        dismissed_wait
            .get("predicate")
            .and_then(|predicate| predicate.get("kind"))
            .and_then(Value::as_str),
        Some("exists"),
    );
}
