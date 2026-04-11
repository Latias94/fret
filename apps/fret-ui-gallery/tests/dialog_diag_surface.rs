#[test]
fn dialog_docs_demo_diag_script_waits_for_stable_overlay_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-docs-demo-open-screenshot.json"
    );

    for needle in [
        "\"ui-gallery-dialog-demo-content\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"stable_frames\": 6",
        "\"max_move_px\": 1.0",
        "\"ui-gallery-dialog-docs-demo-open-desktop\"",
    ] {
        assert!(
            script.contains(needle),
            "dialog docs demo diag script should wait for stable overlay bounds before screenshots; missing `{needle}`",
        );
    }
}
