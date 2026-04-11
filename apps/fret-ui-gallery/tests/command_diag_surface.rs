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
