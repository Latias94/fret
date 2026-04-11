#[test]
fn overlay_narrow_header_sweep_covers_popover_sheet_drawer_and_alert_dialog() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-overlay-narrow-header-sweep.json"
    );

    for needle in [
        "\"ui-gallery-popover-demo-trigger\"",
        "\"ui-gallery-popover-demo-panel\"",
        "\"ui-gallery-overlay-narrow-sweep.popover\"",
        "\"ui-gallery-sheet-demo\"",
        "\"ui-gallery-sheet-demo-panel\"",
        "\"ui-gallery-overlay-narrow-sweep.sheet\"",
        "\"ui-gallery-drawer-demo-trigger\"",
        "\"ui-gallery-drawer-demo-content\"",
        "\"ui-gallery-overlay-narrow-sweep.drawer\"",
        "\"ui-gallery-alert-dialog-demo-trigger\"",
        "\"ui-gallery-alert-dialog-demo-content\"",
        "\"ui-gallery-overlay-narrow-sweep.alert-dialog\"",
        "\"bounds_within_window\"",
    ] {
        assert!(
            script.contains(needle),
            "overlay narrow sweep should keep the shared selector and label set stable; missing `{needle}`",
        );
    }
}

#[test]
fn overlay_narrow_header_sweep_waits_for_stable_bounds_before_screenshots() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-overlay-narrow-header-sweep.json"
    );

    assert_eq!(
        script.matches("\"type\": \"wait_bounds_stable\"").count(),
        4,
        "overlay narrow sweep should wait for stable bounds on popover, sheet, drawer, and alert dialog before capturing screenshots",
    );

    for needle in [
        "\"ui-gallery-popover-demo-panel\"",
        "\"ui-gallery-sheet-demo-panel\"",
        "\"ui-gallery-drawer-demo-content\"",
        "\"ui-gallery-alert-dialog-demo-content\"",
        "\"stable_frames\": 6",
        "\"max_move_px\": 1.0",
    ] {
        assert!(
            script.contains(needle),
            "overlay narrow sweep should keep the stable-bounds evidence anchors explicit; missing `{needle}`",
        );
    }
}
