#[test]
fn public_docking_surface_prefers_dock_surface_entry_points() {
    let dock_mod = include_str!("../src/dock/mod.rs");
    let declarative = include_str!("../src/dock/declarative.rs");
    let lib = include_str!("../src/lib.rs");

    for symbol in [
        "DockSurface",
        "DockHostOptions",
        "DockPanel",
        "DockPanelElementRegistry",
        "DockViewportLayout",
        "DockViewportOverlayHooks",
        "DockingPolicy",
        "ViewportPanel",
    ] {
        assert!(
            lib.contains(symbol),
            "`lib.rs` should expose app-facing docking symbol `{symbol}`"
        );
    }

    for legacy in [
        "pub mod dock;",
        "pub mod runtime;",
        "DockPanelElementRegistryService",
        "DockViewportOverlayHooksService",
        "DockingPolicyService",
        "DockingRuntime",
        "dock_panel_element",
        "dock_space_element_from_registry",
        "handle_dock_op",
        "handle_dock_window_created",
        "handle_dock_before_close_window",
    ] {
        assert!(
            !lib.contains(legacy),
            "`lib.rs` must not expose legacy low-level docking surface `{legacy}`"
        );
    }
    assert!(
        lib.contains("pub mod advanced")
            && lib.contains("DockManager")
            && lib.contains("DockSurfaceDriver")
            && lib.contains("DockRuntimeCommand"),
        "advanced-only low-level driver and manager access should be explicit"
    );
    assert!(
        dock_mod.contains("DockPanelElementRegistryService")
            && dock_mod.contains("dock_space_element_from_registry"),
        "low-level host machinery may remain internal while the common root is DockSurface"
    );
    assert!(
        declarative.contains("cx.managed_surface(")
            || declarative.contains("cx.managed_surface_with_prepaint("),
        "low-level declarative dock-space host should be backed by ManagedSurface"
    );
    assert!(
        !declarative.contains("RetainedSubtreeProps")
            && !declarative.contains("UiTreeRetainedExt")
            && !declarative.contains("create_node_retained"),
        "declarative dock-space host must not grow retained bridge dependencies"
    );
}

#[test]
fn legacy_docking_entry_points_are_not_public() {
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
            "`dock/mod.rs` must not expose legacy docking symbol `{symbol}`"
        );
        assert!(
            !lib.contains(symbol),
            "`lib.rs` must not expose legacy docking symbol `{symbol}`"
        );
    }
}

#[test]
fn dock_surface_root_entry_point_is_public_without_internal_command_queue() {
    let lib = include_str!("../src/lib.rs");
    let runtime = include_str!("../src/runtime.rs");
    let commands = include_str!("../src/runtime/commands.rs");

    for symbol in ["DockSurface", "DockHostOptions"] {
        assert!(
            lib.contains(symbol),
            "`lib.rs` should expose app-facing docking facade symbol `{symbol}`"
        );
    }

    assert!(
        !lib.contains("pub use runtime::DockRuntimeCommand;"),
        "`DockRuntimeCommand` is a driver-tier type and must not be re-exported from the common root"
    );
    assert!(
        lib.contains("DockSurfaceDriver") && lib.contains("DockRuntimeCommand"),
        "`advanced` should name driver-tier surface and runtime command types explicitly"
    );
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
    for helper in [
        "request_float_panel_to_new_window",
        "request_float_tabs_to_new_window",
        "take_runtime_commands",
        "complete_queued_window_created",
        "handle_dock_op_with_runtime_commands",
        "handle_dock_op",
        "handle_dock_window_created",
        "handle_dock_before_close_window",
    ] {
        assert!(
            runtime.contains(&format!("pub(crate) fn {helper}")),
            "`runtime::{helper}` should stay crate-private; use `DockSurface` from outside the crate"
        );
        assert!(
            !runtime.contains(&format!("pub fn {helper}")),
            "`runtime::{helper}` must not be exposed as a public advanced API by accident"
        );
    }
}

fn impl_block<'a>(source: &'a str, header: &str) -> &'a str {
    let start = source
        .find(header)
        .unwrap_or_else(|| panic!("missing impl block `{header}`"));
    let open = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("missing impl body for `{header}`"));
    let mut depth = 0usize;

    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open + 1..open + offset];
                }
            }
            _ => {}
        }
    }

    panic!("unterminated impl block `{header}`");
}

fn public_function_signatures(block: &str) -> Vec<String> {
    let mut signatures = Vec::new();
    let mut current = String::new();
    let mut collecting = false;

    for line in block.lines() {
        let trimmed = line.trim();
        if !collecting && trimmed.starts_with("pub fn ") {
            collecting = true;
            current.clear();
        }
        if collecting {
            current.push_str(trimmed);
            current.push(' ');
            if trimmed.ends_with('{') || trimmed.ends_with(';') {
                signatures.push(current.trim().to_string());
                collecting = false;
            }
        }
    }

    signatures
}

#[test]
fn dock_surface_common_signatures_do_not_expose_driver_tier_types() {
    let facade = include_str!("../src/facade.rs");
    let dock_surface_signatures =
        public_function_signatures(impl_block(facade, "impl DockSurface"));

    for forbidden in [
        "DockGraph",
        "DockNodeId",
        "DockOp",
        "CreateWindowRequest",
        "DockRuntimeCommand",
    ] {
        assert!(
            dock_surface_signatures
                .iter()
                .all(|signature| !signature.contains(forbidden)),
            "`DockSurface` common facade signature must not expose driver-tier type `{forbidden}`; signatures: {dock_surface_signatures:#?}"
        );
    }

    let driver_signatures =
        public_function_signatures(impl_block(facade, "impl DockSurfaceDriver"));
    assert!(
        driver_signatures
            .iter()
            .any(|signature| signature.contains("DockOp")),
        "`DockSurfaceDriver` should own operation-routing signatures"
    );
    assert!(
        driver_signatures
            .iter()
            .any(|signature| signature.contains("DockRuntimeCommand")),
        "`DockSurfaceDriver` should own runtime-command signatures"
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
            source.contains("DockSurface"),
            "`{name}` should route common docking setup through DockSurface"
        );
        assert!(
            source.contains("DockPanelElementRegistry"),
            "`{name}` should provide declarative dock panel roots"
        );
        assert!(
            source.contains("dock_space_declarative_with(") || source.contains(".host("),
            "`{name}` should mount the declarative dock-space host"
        );

        for forbidden in [
            "DockPanelElementRegistryService",
            "DockViewportOverlayHooksService",
            "DockingPolicyService",
            "DockingRuntime::new",
            "dock_space_element_from_registry",
            "fret_docking::runtime",
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
                "`{name}` must not teach legacy low-level docking entry point `{forbidden}`"
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
        "DockSurface",
        "impl DockPanelElementRegistry",
        "DockManager::default",
        "request_dock_invalidation",
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
            "DockPanelElementRegistryService",
            "DockingRuntime::new",
            "dock_runtime::handle_dock_op",
            "dock_runtime::handle_dock_window_created",
            "dock_runtime::handle_dock_before_close_window",
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
                "`{name}` must not teach legacy low-level docking entry point `{forbidden}`"
            );
        }
    }
}

#[test]
fn public_root_legacy_low_level_surface_is_removed_after_dock_surface_lands() {
    let lib = include_str!("../src/lib.rs");
    let manager = include_str!("../src/dock/manager.rs");
    let facade = include_str!("../src/facade.rs");

    for symbol in [
        "pub mod dock;",
        "pub mod runtime;",
        "DockPanelElementRegistryService",
        "DockViewportOverlayHooksService",
        "DockingPolicyService",
        "DockingRuntime",
        "handle_dock_op",
        "handle_dock_window_created",
        "handle_dock_before_close_window",
    ] {
        assert!(
            !lib.contains(symbol),
            "`lib.rs` must not expose legacy low-level docking surface symbol `{symbol}`"
        );
    }
    assert!(
        lib.contains("pub mod advanced") && lib.contains("DockManager"),
        "low-level graph access should be explicit under `advanced`, not taught as the common root"
    );
    assert!(
        manager.contains("pub workspace: DockWorkspace")
            && manager.contains("pub(crate) presentation: DockPresentationState"),
        "`DockManager` should coordinate explicit workspace and transient presentation owners"
    );
    let manager_struct = manager
        .split("pub struct DockManager")
        .nth(1)
        .and_then(|rest| rest.split("}").next())
        .expect("DockManager struct is present");
    for mixed_field in [
        "pub graph: DockGraph",
        "pub panels: HashMap",
        "hover:",
        "viewport_layouts:",
        "dock_space_nodes:",
    ] {
        assert!(
            !manager_struct.contains(mixed_field),
            "`DockManager` must not expose old mixed state field `{mixed_field}` directly"
        );
    }
    assert!(
        manager.contains("pub struct DockWorkspace")
            && manager.contains("pub struct DockPanelCatalog")
            && manager.contains("DuplicatePanelKey"),
        "durable graph and panel catalog authority should live behind explicit workspace/catalog types"
    );
    assert!(
        lib.contains("DockWorkspace")
            && lib.contains("DockPanelCatalog")
            && lib.contains("DockPanelCatalogError"),
        "advanced low-level access should name workspace/catalog types explicitly"
    );
    assert!(
        !facade.contains("pub struct DockingRuntime") && !facade.contains("impl DockingRuntime"),
        "`DockingRuntime` thin adapter should be deleted once DockSurface owns callbacks"
    );
}
