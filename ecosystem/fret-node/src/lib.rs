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
    use std::path::{Path, PathBuf};

    const LIB_RS: &str = include_str!("lib.rs");
    const CARGO_TOML: &str = include_str!("../Cargo.toml");
    const APP_RS: &str = include_str!("app.rs");
    const ADVANCED_RS: &str = include_str!("advanced.rs");
    const UI_BINDING_RS: &str = include_str!("ui/binding.rs");
    const UI_BINDING_QUERIES_RS: &str = include_str!("ui/binding_queries.rs");
    const UI_BINDING_STORE_SYNC_RS: &str = include_str!("ui/binding_store_sync.rs");
    const UI_BINDING_VIEWPORT_RS: &str = include_str!("ui/binding_viewport.rs");
    const UI_CANVAS_RS: &str = include_str!("ui/canvas/widget/widget_surface.rs");
    const UI_CANVAS_BUILDERS_RS: &str = include_str!("ui/canvas/widget/widget_surface/builders.rs");
    const UI_CONTROLLER_RS: &str = include_str!("ui/controller.rs");
    const UI_CONTROLLER_UPDATES_RS: &str = include_str!("ui/controller_updates.rs");
    const UI_CONTROLLER_VIEWPORT_RS: &str = include_str!("ui/controller_viewport.rs");
    const UI_DECLARATIVE_MOD_RS: &str = include_str!("ui/declarative/mod.rs");
    const UI_EDITORS_MOD_RS: &str = include_str!("ui/editors/mod.rs");
    const UI_EDITOR_PORTAL_NUMBER_RS: &str = include_str!("ui/editors/portal_number.rs");
    const UI_EDITOR_PORTAL_TEXT_RS: &str = include_str!("ui/editors/portal_text.rs");
    const UI_MOD_RS: &str = include_str!("ui/mod.rs");
    const UI_OVERLAYS_MOD_RS: &str = include_str!("ui/overlays/mod.rs");
    const UI_OVERLAY_CONTROLS_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/controls_declarative.rs");
    const UI_OVERLAY_CONTROLS_HOST_POLICY_RS: &str =
        include_str!("ui/overlays/controls_host_policy.rs");
    const UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS: &str =
        include_str!("ui/overlays/controls_interaction_policy.rs");
    const UI_OVERLAY_CONTROLS_PAINT_PLAN_RS: &str =
        include_str!("ui/overlays/controls_paint_plan.rs");
    const UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/blackboard_declarative.rs");
    const UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS: &str =
        include_str!("ui/overlays/blackboard_interaction_policy.rs");
    const UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS: &str =
        include_str!("ui/overlays/blackboard_paint_plan.rs");
    const UI_OVERLAY_MINIMAP_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/minimap_declarative.rs");
    const UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS: &str =
        include_str!("ui/overlays/minimap_interaction_policy.rs");
    const UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS: &str =
        include_str!("ui/overlays/toolbar_layout_policy.rs");
    const UI_OVERLAY_TOOLBARS_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/toolbars_declarative.rs");
    const UI_OVERLAY_RENAME_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/rename_declarative.rs");
    const UI_OVERLAY_RENAME_COMMAND_RS: &str = include_str!("ui/overlays/rename_command.rs");
    const UI_OVERLAY_RENAME_LIFECYCLE_RS: &str = include_str!("ui/overlays/rename_lifecycle.rs");
    const UI_VIEWPORT_OPTIONS_RS: &str = include_str!("ui/viewport_options.rs");
    const UI_CANVAS_WIDGET_PAINT_INVALIDATION_RS: &str =
        include_str!("ui/canvas/widget/paint_invalidation.rs");
    const UI_CANVAS_WIDGET_REDRAW_REQUEST_RS: &str =
        include_str!("ui/canvas/widget/redraw_request.rs");
    const UI_CANVAS_WIDGET_TAIL_RS: &str = include_str!("ui/canvas/widget/widget_tail.rs");
    const UI_CANVAS_WIDGET_WIRE_DRAG_COMMIT_CX_RS: &str =
        include_str!("ui/canvas/widget/wire_drag/commit_cx.rs");
    const UI_CANVAS_WIDGET_POINTER_UP_FINISH_RS: &str =
        include_str!("ui/canvas/widget/pointer_up_finish.rs");
    const UI_CANVAS_WIDGET_POINTER_UP_SESSION_CLEANUP_RS: &str =
        include_str!("ui/canvas/widget/pointer_up_session/cleanup.rs");
    const UI_CANVAS_WIDGET_STICKY_WIRE_CONNECT_FINISH_RS: &str =
        include_str!("ui/canvas/widget/sticky_wire_connect/finish.rs");
    const UI_VIEW_QUEUE_RS: &str = include_str!("ui/canvas/widget/view_queue.rs");
    const FRET_EXAMPLES_CARGO_TOML: &str = include_str!("../../../apps/fret-examples/Cargo.toml");
    const FRET_EXAMPLES_LIB_RS: &str = include_str!("../../../apps/fret-examples/src/lib.rs");
    const FRET_DEMO_CARGO_TOML: &str = include_str!("../../../apps/fret-demo/Cargo.toml");
    const FRETBOARD_NATIVE_RS: &str = include_str!("../../../apps/fretboard/src/dev/native.rs");
    const NODE_GRAPH_DEMO_RS: &str =
        include_str!("../../../apps/fret-examples/src/node_graph_demo.rs");
    const UI_GALLERY_CARGO_TOML: &str = include_str!("../../../apps/fret-ui-gallery/Cargo.toml");
    const UI_GALLERY_NODE_GRAPH_CULL_TORTURE_RS: &str = include_str!(
        "../../../apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs"
    );
    const WORKFLOW_NODE_GRAPH_DEMO_RS: &str = include_str!(
        "../../../apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs"
    );

    fn public_surface() -> &'static str {
        LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
    }

    fn binding_surface() -> String {
        [
            UI_BINDING_RS,
            UI_BINDING_QUERIES_RS,
            UI_BINDING_STORE_SYNC_RS,
            UI_BINDING_VIEWPORT_RS,
        ]
        .join("\n")
    }

    fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("source directory should be readable") {
            let entry = entry.expect("source directory entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }

    fn source_rel_path(path: &Path, root: &Path) -> String {
        let rel = path
            .strip_prefix(root)
            .expect("source file should be under scan root")
            .to_string_lossy()
            .replace('\\', "/");
        format!("src/ui/{rel}")
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
        assert!(!CARGO_TOML.contains("compat-retained-bridge"));
        assert!(CARGO_TOML.contains(
            "compat-retained-canvas = [\"fret-ui\", \"fret-ui/unstable-retained-bridge\"]"
        ));
        assert!(
            !UI_DECLARATIVE_MOD_RS.contains("compat_retained")
                && !UI_DECLARATIVE_MOD_RS.contains("node_graph_surface_compat_retained")
                && !UI_DECLARATIVE_MOD_RS.contains("NodeGraphSurfaceCompatRetainedProps"),
            "`fret-node` declarative surface must not expose a retained-subtree compatibility entry point"
        );
        assert!(
            !UI_MOD_RS.contains("node_graph_surface_compat_retained")
                && !UI_MOD_RS.contains("NodeGraphSurfaceCompatRetainedProps"),
            "`fret-node::ui` must not re-export retained-subtree declarative compatibility"
        );
        assert!(
            !UI_DECLARATIVE_MOD_RS.contains("RetainedSubtreeProps")
                && !UI_MOD_RS.contains("RetainedSubtreeProps"),
            "retained subtree compatibility must stay out of the public declarative node graph path"
        );
    }

    #[test]
    fn retained_canvas_tail_policy_helpers_stay_off_retained_bridge() {
        let tail_policy_sources = [
            UI_CANVAS_WIDGET_PAINT_INVALIDATION_RS,
            UI_CANVAS_WIDGET_REDRAW_REQUEST_RS,
            UI_CANVAS_WIDGET_TAIL_RS,
            UI_CANVAS_WIDGET_WIRE_DRAG_COMMIT_CX_RS,
            UI_CANVAS_WIDGET_POINTER_UP_FINISH_RS,
            UI_CANVAS_WIDGET_POINTER_UP_SESSION_CLEANUP_RS,
            UI_CANVAS_WIDGET_STICKY_WIRE_CONNECT_FINISH_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !tail_policy_sources.contains(forbidden),
                "canvas widget tail policy helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
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
        let binding_surface = binding_surface();
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes("));
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_action_host("));
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_with_options("));
        assert!(
            !UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_with_options_action_host(")
        );
        assert!(!binding_surface.contains("pub fn fit_view_nodes("));
        assert!(UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_in_bounds<"));
        assert!(UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_canvas_rect_in_bounds<"));
        assert!(binding_surface.contains("pub fn fit_view_nodes_in_bounds<"));
        assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds<"));
    }

    #[test]
    fn binding_surface_covers_instance_style_viewport_helpers() {
        let binding_surface = binding_surface();
        assert!(binding_surface.contains("pub fn set_viewport_with_options<"));
        assert!(binding_surface.contains("pub fn set_viewport_with_options_action_host("));
        assert!(binding_surface.contains("pub fn set_center_in_bounds<"));
        assert!(binding_surface.contains("pub fn set_center_in_bounds_action_host("));
        assert!(binding_surface.contains("pub fn set_center_in_bounds_with_options<"));
        assert!(binding_surface.contains("pub fn set_center_in_bounds_with_options_action_host("));
        assert!(binding_surface.contains("pub fn fit_view_nodes_in_bounds_with_options<"));
        assert!(
            binding_surface.contains("pub fn fit_view_nodes_in_bounds_with_options_action_host(")
        );
        assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds<"));
        assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_action_host("));
        assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_with_options<"));
        assert!(
            binding_surface.contains("pub fn fit_canvas_rect_in_bounds_with_options_action_host(")
        );
        assert!(binding_surface.contains("pub fn screen_to_canvas<"));
        assert!(binding_surface.contains("pub fn canvas_to_screen<"));
    }

    #[test]
    fn binding_surface_covers_instance_style_sync_and_history_helpers() {
        let binding_surface = binding_surface();
        assert!(binding_surface.contains(
            "pub struct NodeGraphSurfaceBinding {\n    graph: Model<Graph>,\n    view_state: Model<NodeGraphViewState>,\n    editor_config: Model<NodeGraphEditorConfig>,\n    store: Model<NodeGraphStore>,\n    internals: Arc<NodeGraphInternalsStore>,\n}"
        ));
        assert!(binding_surface.contains("pub fn from_models_and_controller("));
        assert!(!binding_surface.contains("pub fn from_models_and_controller_with_editor_config("));
        assert!(!binding_surface.contains("pub fn from_models("));
        assert!(binding_surface.contains("pub fn dispatch_transaction<"));
        assert!(binding_surface.contains("pub fn dispatch_transaction_action_host("));
        assert!(binding_surface.contains("pub fn submit_transaction<"));
        assert!(binding_surface.contains("pub fn submit_transaction_action_host("));
        assert!(binding_surface.contains("pub fn update_node<"));
        assert!(binding_surface.contains("pub fn update_node_action_host<"));
        assert!(binding_surface.contains("pub fn update_edge<"));
        assert!(binding_surface.contains("pub fn update_edge_action_host<"));
        assert!(binding_surface.contains("FnOnce(&mut NodeGraphNodeUpdate)"));
        assert!(binding_surface.contains("FnOnce(&mut NodeGraphEdgeUpdate)"));
        assert!(binding_surface.contains("pub fn store_model(&self) -> Model<NodeGraphStore> {"));
        assert!(
            binding_surface
                .contains("pub fn internals_store(&self) -> Arc<NodeGraphInternalsStore> {")
        );
        assert!(!binding_surface.contains("pub fn controller(&self) -> NodeGraphController {"));
        assert!(binding_surface.contains("pub fn replace_graph_action_host("));
        assert!(binding_surface.contains("pub fn replace_document_action_host("));
        assert!(binding_surface.contains("pub fn replace_view_state_action_host("));
        assert!(binding_surface.contains("pub fn set_selection_action_host("));
        assert!(binding_surface.contains("pub fn undo_action_host("));
        assert!(binding_surface.contains("pub fn redo_action_host("));
    }

    #[test]
    fn update_helpers_hide_structural_fields_behind_explicit_transactions() {
        assert!(UI_CONTROLLER_UPDATES_RS.contains("pub struct NodeGraphNodeUpdate"));
        assert!(UI_CONTROLLER_UPDATES_RS.contains("pub struct NodeGraphEdgeUpdate"));
        assert!(!UI_CONTROLLER_UPDATES_RS.contains("pub ports:"));
        assert!(!UI_CONTROLLER_UPDATES_RS.contains("pub from:"));
        assert!(!UI_CONTROLLER_UPDATES_RS.contains("pub to:"));
        assert!(UI_CONTROLLER_UPDATES_RS.contains("Use explicit transactions for port"));
        assert!(UI_CONTROLLER_UPDATES_RS.contains("Use explicit transactions for reconnects"));
    }

    #[test]
    fn root_ui_surface_re_exports_store_first_viewport_option_types_but_not_raw_view_queue_module()
    {
        assert!(!UI_MOD_RS.contains("mod view_queue;"));
        assert!(UI_MOD_RS.contains("mod viewport_options;"));
        assert!(!UI_MOD_RS.contains("pub mod view_queue;"));
        assert!(UI_MOD_RS.contains(
            "pub use viewport_options::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions};"
        ));
    }

    #[test]
    fn public_viewport_option_surface_stays_store_first() {
        assert!(UI_VIEWPORT_OPTIONS_RS.contains("pub struct NodeGraphFitViewOptions"));
        assert!(UI_VIEWPORT_OPTIONS_RS.contains("pub struct NodeGraphSetViewportOptions"));
        assert!(!UI_VIEWPORT_OPTIONS_RS.contains("duration_ms"));
        assert!(!UI_VIEWPORT_OPTIONS_RS.contains("interpolate"));
        assert!(!UI_VIEWPORT_OPTIONS_RS.contains("ease"));
        assert!(UI_VIEW_QUEUE_RS.contains("pub(crate) struct NodeGraphViewQueueFitViewOptions"));
        assert!(
            UI_VIEW_QUEUE_RS.contains("pub(crate) struct NodeGraphViewQueueSetViewportOptions")
        );
        assert!(UI_VIEW_QUEUE_RS.contains("duration_ms"));
        assert!(UI_VIEW_QUEUE_RS.contains("interpolate"));
        assert!(UI_VIEW_QUEUE_RS.contains("ease"));
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("duration_ms"));
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("interpolate"));
        assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("ease"));
    }

    #[test]
    fn retained_widget_compat_island_stays_crate_private_and_controller_bound() {
        assert!(UI_MOD_RS.contains("mod canvas;"));
        assert!(!UI_MOD_RS.contains("pub mod canvas;"));
        assert!(!UI_MOD_RS.contains("pub use canvas::NodeGraphCanvas"));
        assert!(!UI_MOD_RS.contains("pub(crate) use canvas::NodeGraphCanvas"));
        assert!(!UI_MOD_RS.contains("pub use canvas::NodeGraphCanvasWith"));
        assert!(!UI_MOD_RS.contains("pub(crate) use canvas::NodeGraphCanvasWith"));
        assert!(!UI_MOD_RS.contains("pub mod a11y;"));
        assert!(!UI_MOD_RS.contains("pub mod editor;"));
        assert!(!UI_MOD_RS.contains("pub mod editors;"));
        assert!(!UI_MOD_RS.contains("pub mod overlays;"));
        assert!(!UI_MOD_RS.contains("pub mod panel;"));
        assert!(!UI_MOD_RS.contains("pub mod portal;"));
        assert!(!UI_MOD_RS.contains("pub use editor::NodeGraphEditor"));
        assert!(!UI_MOD_RS.contains("pub use panel::{NodeGraphPanel"));
        assert!(!UI_MOD_RS.contains("pub use portal::{"));
        assert!(!UI_MOD_RS.contains("pub use overlays::{"));
        assert!(UI_MOD_RS.contains("#[cfg(all(test, feature = \"compat-retained-canvas\"))]"));
        assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod editor;"));
        assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod panel;"));

        assert!(UI_CANVAS_RS.contains(
            "pub fn new(\n        graph: Model<Graph>,\n        view_state: Model<NodeGraphViewState>,\n        editor_config: Model<NodeGraphEditorConfig>,\n    ) -> Self {"
        ));
        assert!(UI_CANVAS_BUILDERS_RS.contains(
            "pub fn with_controller(mut self, controller: NodeGraphController) -> Self {"
        ));
        assert!(!UI_CANVAS_BUILDERS_RS.contains("with_editor_config_model("));
        assert!(UI_CANVAS_BUILDERS_RS.contains("pub(crate) fn with_view_queue("));
        assert!(UI_CANVAS_BUILDERS_RS.contains("retained compatibility plumbing"));
        assert!(UI_CANVAS_BUILDERS_RS.contains("declarative node graph surface"));

        assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod portal;"));
    }

    #[test]
    fn retained_bridge_source_usage_stays_on_the_migration_ledger() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let mut files = Vec::new();
        collect_rs_files(&ui_root, &mut files);

        let allowed_exact = ["src/ui/canvas/widget.rs"];
        let allowed_prefixes = ["src/ui/canvas/widget/"];
        let retained_terms = [
            "use fret_ui::retained_bridge",
            "use fret_ui::{UiHost, retained_bridge",
            "fret_ui::retained_bridge::",
            "RetainedSubtreeProps",
            "UiTreeRetainedExt",
        ];

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if !retained_terms.iter().any(|term| source.contains(term)) {
                continue;
            }

            let rel = source_rel_path(&path, &ui_root);
            let allowed = allowed_exact.contains(&rel.as_str())
                || allowed_prefixes
                    .iter()
                    .any(|prefix| rel.starts_with(prefix));
            if !allowed {
                offenders.push(rel);
            }
        }

        assert!(
            offenders.is_empty(),
            "retained bridge source usage must stay on the explicit compat-retained-canvas migration ledger:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn overlay_policy_modules_compile_without_retained_canvas_compat() {
        assert!(UI_MOD_RS.contains("mod overlays;"));
        assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod overlays;"));
        assert!(UI_MOD_RS.contains("mod screen_space_placement;"));
        assert!(
            !UI_MOD_RS.contains(
                "#[cfg(feature = \"compat-retained-canvas\")]\nmod screen_space_placement;"
            )
        );

        for module in [
            "mod blackboard_declarative;",
            "mod blackboard_interaction_policy;",
            "mod blackboard_layout;",
            "mod blackboard_paint_plan;",
            "mod blackboard_policy;",
            "mod controls_declarative;",
            "mod controls_host_policy;",
            "mod controls_interaction_policy;",
            "mod controls_layout;",
            "mod controls_paint_plan;",
            "mod controls_policy;",
            "mod minimap_drag_policy;",
            "mod minimap_interaction_policy;",
            "mod minimap_declarative;",
            "mod minimap_navigation_policy;",
            "mod minimap_policy;",
            "mod minimap_projection;",
            "mod panel_item_state;",
            "mod panel_navigation_policy;",
            "mod panel_pointer_policy;",
            "mod rename_command;",
            "mod rename_host_layout;",
            "mod rename_lifecycle;",
            "mod rename_declarative;",
            "mod rename_policy;",
            "mod toolbar_layout_policy;",
            "mod toolbar_policy;",
            "mod toolbars_declarative;",
        ] {
            assert!(
                UI_OVERLAYS_MOD_RS.contains(module),
                "overlay policy module should compile outside compat-retained-canvas: {module}"
            );
        }

        assert!(!UI_OVERLAYS_MOD_RS.contains("mod panel_button_paint;"));
    }

    #[test]
    fn editor_chrome_compiles_without_retained_canvas_compat() {
        assert!(UI_MOD_RS.contains("mod editors;"));
        assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod editors;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod chrome;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod portal_command_policy;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod portal_command_session;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod portal_number;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod portal_text;"));
        assert!(
            !UI_EDITORS_MOD_RS
                .contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod portal_number;")
        );
        assert!(
            !UI_EDITORS_MOD_RS
                .contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod portal_text;")
        );

        for retained_bridge_free_editor in [UI_EDITOR_PORTAL_NUMBER_RS, UI_EDITOR_PORTAL_TEXT_RS] {
            assert!(
                !retained_bridge_free_editor.contains("retained_bridge")
                    && !retained_bridge_free_editor.contains("CommandCx"),
                "default portal editor modules must not depend on retained bridge command adapters"
            );
        }
    }

    #[test]
    fn default_overlay_policy_surfaces_stay_off_retained_bridge() {
        assert!(
            !UI_OVERLAY_CONTROLS_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_CONTROLS_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_CONTROLS_DECLARATIVE_RS.contains("Widget<"),
            "declarative controls composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_CONTROLS_HOST_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_CONTROLS_HOST_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_CONTROLS_HOST_POLICY_RS.contains("Widget<"),
            "default controls host policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS.contains("Widget<"),
            "default controls interaction policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_CONTROLS_PAINT_PLAN_RS.contains("retained_bridge")
                && !UI_OVERLAY_CONTROLS_PAINT_PLAN_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_CONTROLS_PAINT_PLAN_RS.contains("Widget<"),
            "default controls paint plan must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS.contains("Widget<"),
            "declarative blackboard composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS.contains("Widget<"),
            "default blackboard interaction policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS.contains("retained_bridge")
                && !UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS.contains("Widget<"),
            "default blackboard paint plan must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_MINIMAP_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_MINIMAP_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_MINIMAP_DECLARATIVE_RS.contains("Widget<"),
            "declarative minimap composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS.contains("Widget<"),
            "default minimap interaction policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS.contains("Widget<"),
            "default toolbar layout policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("Widget<"),
            "declarative toolbar composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_RENAME_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_RENAME_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_RENAME_DECLARATIVE_RS.contains("Widget<"),
            "declarative rename composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_RENAME_COMMAND_RS.contains("retained_bridge")
                && !UI_OVERLAY_RENAME_COMMAND_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_RENAME_COMMAND_RS.contains("Widget<"),
            "default rename command/session policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_RENAME_LIFECYCLE_RS.contains("retained_bridge")
                && !UI_OVERLAY_RENAME_LIFECYCLE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_RENAME_LIFECYCLE_RS.contains("Widget<"),
            "default rename lifecycle policy must not take a retained dependency"
        );
    }

    #[test]
    fn workflow_gallery_surface_stays_binding_first_for_viewport_controls() {
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.observe(cx);"));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("LayoutQueryRegionProps::default()"));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("node_graph_surface(cx, props)"));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.set_viewport_action_host("));
        assert!(
            WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.fit_view_nodes_in_bounds_action_host(")
        );
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.fit_view_nodes_in_bounds(cx.app,"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("RetainedSubtreeProps"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("retained_bridge"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphCanvas::new"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphEditor::new"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("create_node_retained"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("retained_subtree"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphController"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.controller()"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains(".with_controller(binding.controller())"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphViewQueue"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("bind_controller_view_queue_transport"));
        assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("use fret_node::ui::advanced::{"));
    }

    #[test]
    fn first_party_gallery_node_graph_pages_stay_off_retained_canvas() {
        assert!(
            UI_GALLERY_CARGO_TOML
                .contains("fret-node = { path = \"../../ecosystem/fret-node\", optional = true }")
        );
        assert!(!UI_GALLERY_CARGO_TOML.contains("fret-node/compat-retained-canvas"));
        assert!(!UI_GALLERY_CARGO_TOML.contains(
            "fret-node = { path = \"../../ecosystem/fret-node\", optional = true, features = [\"compat-retained-canvas\"] }"
        ));

        for source in [
            WORKFLOW_NODE_GRAPH_DEMO_RS,
            UI_GALLERY_NODE_GRAPH_CULL_TORTURE_RS,
        ] {
            assert!(source.contains("NodeGraphSurfaceBinding::new("));
            assert!(source.contains("node_graph_surface"));
            assert!(!source.contains("RetainedSubtreeProps"));
            assert!(!source.contains("retained_bridge"));
            assert!(!source.contains("NodeGraphCanvas::new"));
            assert!(!source.contains("NodeGraphEditor::new"));
            assert!(!source.contains("create_node_retained"));
            assert!(!source.contains("retained_subtree"));
        }
    }

    #[test]
    fn first_party_node_graph_demos_stay_declarative_only() {
        for source in [
            FRET_EXAMPLES_CARGO_TOML,
            FRET_EXAMPLES_LIB_RS,
            FRET_DEMO_CARGO_TOML,
            FRETBOARD_NATIVE_RS,
            NODE_GRAPH_DEMO_RS,
        ] {
            assert!(!source.contains("node-graph-demos-legacy"));
            assert!(!source.contains("fret-node/compat-retained-canvas"));
            assert!(!source.contains("node_graph_legacy_demo"));
            assert!(!source.contains("node_graph_domain_demo"));
            assert!(!source.contains("imui_node_graph_demo"));
            assert!(!source.contains("node_graph_tuning_overlay"));
            assert!(!source.contains("RetainedSubtreeProps"));
            assert!(!source.contains("retained_bridge"));
            assert!(!source.contains("NodeGraphCanvas::new"));
            assert!(!source.contains("NodeGraphEditor::new"));
            assert!(!source.contains("create_node_retained"));
            assert!(!source.contains("retained_subtree"));
        }
        assert!(FRET_EXAMPLES_CARGO_TOML.contains("node-graph-demos = []"));
        assert!(
            FRET_DEMO_CARGO_TOML
                .contains("node-graph-demos = [\"fret-examples/node-graph-demos\"]")
        );
        assert!(FRET_EXAMPLES_LIB_RS.contains("pub mod node_graph_demo;"));
        assert!(NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
        assert!(NODE_GRAPH_DEMO_RS.contains("node_graph_surface_in(cx, props)"));
        assert!(!NODE_GRAPH_DEMO_RS.contains("NodeGraphController"));
        assert!(!NODE_GRAPH_DEMO_RS.contains("binding.controller()"));
    }
}
