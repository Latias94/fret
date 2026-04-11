use serde_json::Value;

fn parsed_steps(script: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(script)
        .unwrap()
        .get("steps")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn first_step_index_by_type(steps: &[Value], expected_type: &str) -> usize {
    steps
        .iter()
        .position(|step| step.get("type").and_then(Value::as_str) == Some(expected_type))
        .unwrap_or_else(|| panic!("missing step type `{expected_type}`"))
}

fn has_wait_until_runner_accessibility_activated(steps: &[Value]) -> bool {
    steps.iter().any(|step| {
        step.get("type").and_then(Value::as_str) == Some("wait_until")
            && step
                .get("predicate")
                .and_then(|predicate| predicate.get("kind"))
                .and_then(Value::as_str)
                == Some("runner_accessibility_activated")
    })
}

#[test]
fn command_dialog_shortcut_diag_script_scrolls_to_the_basic_trigger_before_opening() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-a11y-command-dialog-shortcut-primary.json"
    );

    for needle in [
        "\"ui-gallery-command-basic-trigger.chrome\"",
        "\"ui-gallery-command-basic-input\"",
        "\"type\": \"scroll_into_view\"",
        "\"type\": \"click_stable\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"Command palette\"",
        "\"ui-gallery-a11y-command-dialog-shortcut-primary\"",
    ] {
        assert!(
            script.contains(needle),
            "command dialog diag script should keep the trigger scroll/open chain stable; missing `{needle}`",
        );
    }
}

#[test]
fn command_dialog_ax_activated_diag_script_reuses_the_stable_basic_trigger_open_chain() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/command/a11y-ui-gallery-command-dialog-shortcut-primary-ax-activated.json"
    );

    for needle in [
        "\"inject_runner_accessibility_activation\"",
        "\"runner_accessibility_activated\"",
        "\"ui-gallery-command-basic-trigger.chrome\"",
        "\"ui-gallery-command-basic-input\"",
        "\"type\": \"scroll_into_view\"",
        "\"type\": \"click_stable\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"Command palette\"",
        "\"ui-gallery-a11y-command-dialog-shortcut-primary-ax-activated\"",
    ] {
        assert!(
            script.contains(needle),
            "command accessibility diag script should keep the same stable open-chain after the accessibility precondition; missing `{needle}`",
        );
    }
}

#[test]
fn command_palette_ax_activated_diag_script_injects_runner_accessibility_before_shortcut() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/command/a11y-ui-gallery-command-palette-shortcut-primary-ax-activated.json"
    );

    for needle in [
        "\"inject_runner_accessibility_activation\"",
        "\"runner_accessibility_activated\"",
        "\"press_shortcut\"",
        "\"primary+p\"",
        "\"Command palette\"",
        "\"ui-gallery-command-palette-shortcut-primary-ax-activated\"",
    ] {
        assert!(
            script.contains(needle),
            "command palette accessibility diag script should inject runner accessibility before exercising the shortcut; missing `{needle}`",
        );
    }
}

#[test]
fn command_accessibility_diag_scripts_inject_before_interaction_and_skip_host_wait() {
    for (name, script, first_interaction_step) in [
        (
            "command-dialog",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/command/a11y-ui-gallery-command-dialog-shortcut-primary-ax-activated.json"
            ),
            "click",
        ),
        (
            "command-palette",
            include_str!(
                "../../../tools/diag-scripts/ui-gallery/command/a11y-ui-gallery-command-palette-shortcut-primary-ax-activated.json"
            ),
            "press_shortcut",
        ),
    ] {
        let steps = parsed_steps(script);
        let inject_index =
            first_step_index_by_type(&steps, "inject_runner_accessibility_activation");
        let interaction_index = first_step_index_by_type(&steps, first_interaction_step);
        let runner_assert_index = steps
            .iter()
            .position(|step| {
                step.get("type").and_then(Value::as_str) == Some("assert")
                    && step
                        .get("predicate")
                        .and_then(|predicate| predicate.get("kind"))
                        .and_then(Value::as_str)
                        == Some("runner_accessibility_activated")
            })
            .unwrap_or_else(|| panic!("missing runner accessibility assertion for `{name}`"));

        assert!(
            inject_index < interaction_index,
            "`{name}` should inject accessibility activation before the first interaction step",
        );
        assert!(
            inject_index < runner_assert_index,
            "`{name}` should inject accessibility activation before asserting the diagnostic evidence",
        );
        assert!(
            !has_wait_until_runner_accessibility_activated(&steps),
            "`{name}` should not wait for host accessibility activation in launch mode",
        );
    }
}
