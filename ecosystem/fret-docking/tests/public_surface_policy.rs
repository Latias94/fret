#[test]
fn public_docking_surface_prefers_declarative_entry_points() {
    let dock_mod = include_str!("../src/dock/mod.rs");
    let declarative = include_str!("../src/dock/declarative.rs");
    let lib = include_str!("../src/lib.rs");

    for symbol in [
        "DockPanelElement",
        "DockPanelElementRegistry",
        "DockPanelElementRegistryService",
        "DockSpaceElementOptions",
        "dock_panel_element",
        "dock_space_element",
        "dock_space_element_from_registry",
    ] {
        assert!(
            dock_mod.contains(symbol),
            "`dock/mod.rs` should re-export public declarative docking symbol `{symbol}`"
        );
        assert!(
            lib.contains(symbol),
            "`lib.rs` should expose public declarative docking symbol `{symbol}`"
        );
    }

    assert!(
        declarative.contains("cx.managed_surface(")
            || declarative.contains("cx.managed_surface_with_prepaint("),
        "public declarative dock-space entry point should be backed by ManagedSurface"
    );
    assert!(
        !declarative.contains("RetainedSubtreeProps")
            && !declarative.contains("UiTreeRetainedExt")
            && !declarative.contains("create_node_retained"),
        "public declarative dock-space entry point must not grow retained bridge dependencies"
    );
}

#[test]
fn retained_docking_entry_points_are_not_public() {
    let dock_mod = include_str!("../src/dock/mod.rs");
    let lib = include_str!("../src/lib.rs");

    for symbol in [
        "create_dock_space_node",
        "create_dock_space_node_with_test_id",
        "mount_dock_space",
        "mount_dock_space_with_test_id",
        "render_and_bind_dock_panels",
        "DockPanelFactory",
        "DockPanelRegistry",
        "DockPanelRegistryService",
    ] {
        assert!(
            !dock_mod.contains(symbol),
            "`dock/mod.rs` must not expose retained docking symbol `{symbol}`"
        );
        assert!(
            !lib.contains(symbol),
            "`lib.rs` must not expose retained docking symbol `{symbol}`"
        );
    }
}
