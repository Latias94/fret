#[test]
fn drawer_responsive_dialog_diag_script_waits_for_stable_overlay_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-responsive-dialog-smoke.json"
    );

    for needle in [
        "\"ui-gallery-drawer-responsive-desktop-content\"",
        "\"ui-gallery-drawer-responsive-mobile-content\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"stable_frames\": 6",
        "\"max_move_px\": 1.0",
        "\"ui-gallery-drawer-responsive-dialog-desktop-open\"",
        "\"ui-gallery-drawer-responsive-dialog-mobile-open\"",
        "\"ui-gallery-drawer-responsive-dialog-smoke\"",
    ] {
        assert!(
            script.contains(needle),
            "drawer responsive dialog diag script should wait for stable overlay bounds before screenshots; missing `{needle}`",
        );
    }
}
