//! Node graph substrate for Fret.
//!
//! This crate provides a long-lived, serializable graph model with typed connections and
//! editor-grade contracts (migrations, diagnostics, deterministic persistence).
//!
//! UI integration is optional and lives behind the default `fret-ui` feature.

#![deny(unsafe_code)]

/// Reserved builtin node kind for a schema-less wire reroute node.
pub const REROUTE_KIND: &str = "fret.reroute";

pub mod core;
pub mod interaction;
pub mod io;
#[cfg(feature = "kit")]
pub mod kit;
pub mod ops;
pub mod profile;
pub mod rules;
pub mod runtime;
pub mod schema;
pub mod types;

#[cfg(feature = "fret-ui")]
pub mod ui;

pub use core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group, GroupId,
    Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
    StickyNote, StickyNoteId, Symbol, SymbolId,
};
pub use interaction::{
    NodeGraphConnectionMode, NodeGraphDragHandleMode, NodeGraphModifierKey,
    NodeGraphZoomActivationKey,
};
pub use rules::{ConnectPlan, Diagnostic, DiagnosticSeverity};
pub use types::{TypeDesc, TypeVarId};

#[cfg(feature = "app-integration")]
pub mod advanced;
#[cfg(feature = "app-integration")]
pub mod app;

#[cfg(test)]
mod surface_policy_tests {
    const LIB_RS: &str = include_str!("lib.rs");
    const CARGO_TOML: &str = include_str!("../Cargo.toml");
    const APP_RS: &str = include_str!("app.rs");
    const ADVANCED_RS: &str = include_str!("advanced.rs");
    const COMPAT_RETAINED_RS: &str = include_str!("ui/declarative/compat_retained.rs");
    const UI_BINDING_RS: &str = include_str!("ui/binding.rs");
    const UI_CONTROLLER_RS: &str = include_str!("ui/controller.rs");
    const UI_CONTROLLER_VIEWPORT_RS: &str = include_str!("ui/controller_viewport.rs");
    const UI_MOD_RS: &str = include_str!("ui/mod.rs");
    const MINIMAP_RS: &str = include_str!("ui/overlays/minimap.rs");
    const NODE_GRAPH_DOMAIN_DEMO_RS: &str =
        include_str!("../../../apps/fret-examples/src/node_graph_domain_demo.rs");
    const NODE_GRAPH_LEGACY_DEMO_RS: &str =
        include_str!("../../../apps/fret-examples/src/node_graph_legacy_demo.rs");
    const WORKFLOW_NODE_GRAPH_DEMO_RS: &str = include_str!(
        "../../../apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs"
    );

    fn public_surface() -> &'static str {
        LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
    }

    #[test]
    fn app_integration_stays_under_explicit_app_module() {
        let public_surface = public_surface();
        assert!(public_surface.contains("pub mod app;"));
        assert!(public_surface.contains("pub mod advanced;"));
        assert!(!public_surface.contains("pub use app::"));
        assert!(!public_surface.contains("pub use advanced::"));
        assert!(!public_surface.contains("pub fn install("));
        assert!(!public_surface.contains("pub fn install_with_ui_services("));
        assert!(APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
        assert!(!APP_RS.contains("install_with_ui_services"));
        assert!(ADVANCED_RS.contains("pub fn install_with_ui_services("));
    }

    #[test]
    fn retained_compatibility_surface_stays_declarative_only() {
        let public_surface = public_surface();
        assert!(!public_surface.contains("pub mod imui;"));
        assert!(!CARGO_TOML.contains("\nimui = ["));
        assert!(!CARGO_TOML.contains("fret-authoring"));
        assert!(COMPAT_RETAINED_RS.contains("This is a **compatibility** surface:"));
        assert!(COMPAT_RETAINED_RS.contains("delete-planned"));
        assert!(
            COMPAT_RETAINED_RS
                .contains("keeps retained authoring out of the downstream API surface")
        );
    }

    #[test]
    fn raw_transport_surface_stays_crate_internal() {
        assert!(!UI_MOD_RS.contains("pub mod advanced;"));
        assert!(!UI_MOD_RS.contains("pub mod edit_queue;"));
        assert!(!UI_MOD_RS.contains("NodeGraphEditQueue"));
        assert!(!UI_MOD_RS.contains("bind_controller_edit_queue_transport"));
        assert!(!UI_MOD_RS.contains("NodeGraphViewQueue"));
    }

    #[test]
    fn controller_surface_stays_store_first_without_embedded_transport_state() {
        assert!(!UI_CONTROLLER_RS.contains("edit_queue: Option<"));
        assert!(!UI_CONTROLLER_RS.contains("view_queue: Option<"));
        assert!(!UI_CONTROLLER_RS.contains("bind_edit_queue_transport"));
        assert!(!UI_CONTROLLER_RS.contains("bind_view_queue_transport"));
        assert!(!UI_CONTROLLER_RS.contains("transport_edit_queue"));
        assert!(!UI_CONTROLLER_RS.contains("transport_view_queue"));
    }

    #[test]
    fn fit_view_surface_stays_bounds_first() {
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes("));
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_action_host("));
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_with_options("));
        assert!(
            !UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_with_options_action_host(")
        );
        assert!(!UI_BINDING_RS.contains("pub fn fit_view_nodes("));
        assert!(UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_in_bounds<"));
        assert!(UI_BINDING_RS.contains("pub fn fit_view_nodes_in_bounds<"));
    }

    #[test]
    fn root_ui_surface_re_exports_viewport_option_types_but_not_raw_view_queue_module() {
        assert!(UI_MOD_RS.contains("mod view_queue;"));
        assert!(!UI_MOD_RS.contains("pub mod view_queue;"));
        assert!(UI_MOD_RS.contains(
            "pub use view_queue::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions};"
        ));
    }

    #[test]
    fn workflow_gallery_surface_stays_binding_first_for_viewport_controls() {
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.set_viewport_action_host("));
        assert!(
            WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.fit_view_nodes_in_bounds_action_host(")
        );
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphViewQueue"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("bind_controller_view_queue_transport"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("use fret_node::ui::advanced::{"));
    }

    #[test]
    fn first_party_demos_stay_controller_first_for_edit_commits() {
        for source in [NODE_GRAPH_DOMAIN_DEMO_RS, NODE_GRAPH_LEGACY_DEMO_RS] {
            assert!(!source.contains("NodeGraphEditQueue"));
            assert!(!source.contains("bind_controller_edit_queue_transport"));
            assert!(!source.contains("use fret_node::ui::advanced::{"));
        }
        assert!(NODE_GRAPH_DOMAIN_DEMO_RS.contains("let controller = NodeGraphController::new("));
        assert!(NODE_GRAPH_LEGACY_DEMO_RS.contains("submit_transaction_and_sync_graph_model("));
        assert!(NODE_GRAPH_LEGACY_DEMO_RS.contains("replace_document_and_sync_models("));
    }

    #[test]
    fn minimap_navigation_surface_stays_controller_or_default_only() {
        assert!(!MINIMAP_RS.contains("NodeGraphMiniMapNavigationBinding::ViewQueue"));
        assert!(!MINIMAP_RS.contains("ViewQueue(Model<NodeGraphViewQueue>)"));
        assert!(!MINIMAP_RS.contains("pub(crate) fn with_view_queue("));
        assert!(MINIMAP_RS.contains("NodeGraphMiniMapNavigationBinding::Controller(controller)"));
    }
}
