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

#[test]
fn material_recipes_resolve_layout_direction_through_material_context() {
    let sources = [
        ("autocomplete.rs", include_str!("../src/autocomplete.rs")),
        ("dropdown_menu.rs", include_str!("../src/dropdown_menu.rs")),
        ("search_view.rs", include_str!("../src/search_view.rs")),
        ("tooltip.rs", include_str!("../src/tooltip.rs")),
    ];

    for (name, source) in sources {
        assert!(
            !source.contains("use_direction_in_scope(cx, None)"),
            "{name} should not bypass Material context for layout direction"
        );
        assert!(
            source.contains("material_layout_direction_in_scope(cx)"),
            "{name} should use the Material-facing layout direction helper"
        );
    }
}
