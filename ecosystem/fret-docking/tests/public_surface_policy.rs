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
fn dock_surface_root_entry_point_is_public_without_internal_command_queue() {
    let lib = include_str!("../src/lib.rs");
    let runtime = include_str!("../src/runtime.rs");
    let commands = include_str!("../src/runtime/commands.rs");

    for symbol in ["DockSurface", "DockHostOptions", "DockRuntimeCommand"] {
        assert!(
            lib.contains(symbol),
            "`lib.rs` should expose app-facing docking facade symbol `{symbol}`"
        );
    }

    assert!(
        !lib.contains("DockRuntimeCommandQueue"),
        "`DockRuntimeCommandQueue` is storage detail and must not be part of the crate root API"
    );
    assert!(
        runtime.contains("pub use commands::DockRuntimeCommand;")
            && !runtime.contains("DockRuntimeCommandQueue};"),
        "`runtime.rs` should export the command result type without exporting queue storage"
    );
    assert!(
        commands.contains("pub(super) struct DockRuntimeCommandQueue"),
        "`DockRuntimeCommandQueue` should remain internal to the runtime implementation"
    );
}

#[test]
fn first_party_docking_examples_use_current_declarative_entry_points() {
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

    let imui_demo = include_str!("../../../apps/fret-examples/src/imui_editor_proof_demo.rs");
    let imui_workbench =
        include_str!("../../../apps/fret-examples/src/imui_editor_proof_demo/workbench_shell.rs");

    for symbol in [
        "workbench_shell::install_dock_panel_registry(app)",
        "workbench_shell::ensure_dock_graph(cx.app, cx.window)",
        "dock_space_declarative_with(",
    ] {
        assert!(
            imui_demo.contains(symbol),
            "`imui_editor_proof_demo` should route declarative docking setup through workbench shell symbol `{symbol}`"
        );
    }
    for symbol in [
        "DockPanelElementRegistryService",
        "impl DockPanelElementRegistry",
        "DockManager::default",
    ] {
        assert!(
            imui_workbench.contains(symbol),
            "`imui_editor_proof_demo/workbench_shell.rs` should own declarative docking setup symbol `{symbol}`"
        );
    }
    for source in [
        ("imui_editor_proof_demo", imui_demo),
        ("imui_editor_proof_demo/workbench_shell", imui_workbench),
    ] {
        let (name, source) = source;
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

#[test]
fn public_root_legacy_low_level_surface_is_characterized_until_dock_surface_lands() {
    let lib = include_str!("../src/lib.rs");
    let dock_mod = include_str!("../src/dock/mod.rs");
    let manager = include_str!("../src/dock/manager.rs");
    let facade = include_str!("../src/facade.rs");

    for symbol in [
        "pub mod dock;",
        "pub mod runtime;",
        "DockManager",
        "DockPanelElementRegistryService",
        "DockViewportOverlayHooksService",
        "DockingPolicyService",
        "DockingRuntime",
        "handle_dock_op",
        "handle_dock_window_created",
        "handle_dock_before_close_window",
    ] {
        assert!(
            lib.contains(symbol),
            "`lib.rs` currently exposes legacy low-level docking surface symbol `{symbol}`; U3/U8 should replace this characterization with facade policy"
        );
    }
    assert!(
        dock_mod.contains("DockPanelContentService"),
        "`dock/mod.rs` currently exposes DockPanelContentService through the public dock module"
    );
    assert!(
        manager.contains("pub graph: DockGraph") && manager.contains("pub panels: HashMap"),
        "`DockManager` currently exposes graph and panel state directly"
    );
    assert!(
        facade.contains("crate::runtime::handle_dock_op")
            && facade.contains("crate::runtime::handle_dock_window_created")
            && facade.contains("crate::runtime::handle_dock_before_close_window"),
        "`DockingRuntime` is currently a thin adapter over free runtime handlers"
    );
}
