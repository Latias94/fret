#[test]
fn overlay_outer_bounds_use_environment_snapshot_helpers() {
    let search_view = include_str!("../src/search_view.rs");
    assert!(
        search_view.contains("outer_bounds_with_window_margin_for_environment"),
        "search_view overlay should derive outer bounds from the committed environment snapshot"
    );

    let tooltip = include_str!("../src/tooltip.rs");
    assert!(
        tooltip.contains("outer_bounds_with_window_margin_for_environment"),
        "tooltip overlay should derive outer bounds from the committed environment snapshot"
    );
}
