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

#[test]
fn first_party_docking_examples_use_declarative_entry_points() {
    let examples = [
        (
            "docking_demo",
            include_str!("../../../apps/fret-examples/src/docking_demo.rs"),
        ),
        (
            "container_queries_docking_demo",
            include_str!("../../../apps/fret-examples/src/container_queries_docking_demo.rs"),
        ),
        (
            "docking_arbitration_demo",
            include_str!("../../../apps/fret-examples/src/docking_arbitration_demo.rs"),
        ),
        (
            "imui_editor_proof_demo",
            include_str!("../../../apps/fret-examples/src/imui_editor_proof_demo.rs"),
        ),
        (
            "cookbook_docking_basics",
            include_str!("../../../apps/fret-cookbook/examples/docking_basics.rs"),
        ),
    ];

    for (name, source) in examples {
        assert!(
            source.contains("DockPanelElementRegistry")
                || source.contains("DockPanelElementRegistryService"),
            "`{name}` should register declarative dock panel roots"
        );
        assert!(
            source.contains("dock_space_element_from_registry(")
                || source.contains("dock_space_declarative_with("),
            "`{name}` should mount the declarative dock-space host"
        );

        for forbidden in [
            "DockPanelFactory",
            "DockPanelFactoryCx",
            "DockPanelRegistryBuilder",
            "DockPanelRegistryService",
            "DockSpace::new",
            "create_dock_space_node",
            "mount_dock_space",
            "render_and_bind_dock_panels",
            "dock_space_with(",
            "DockSpaceImUiOptions",
        ] {
            assert!(
                !source.contains(forbidden),
                "`{name}` must not teach retained docking entry point `{forbidden}`"
            );
        }
    }
}
