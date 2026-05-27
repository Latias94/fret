use std::path::{Path, PathBuf};

const LIB_RS: &str = include_str!("lib.rs");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const APP_RS: &str = include_str!("app.rs");
const ADVANCED_RS: &str = include_str!("advanced.rs");
const UI_BINDING_RS: &str = include_str!("ui/binding.rs");
const UI_BINDING_QUERIES_RS: &str = include_str!("ui/binding_queries.rs");
const UI_BINDING_STORE_SYNC_RS: &str = include_str!("ui/binding_store_sync.rs");
const UI_BINDING_VIEWPORT_RS: &str = include_str!("ui/binding_viewport.rs");
const UI_CANVAS_MOD_RS: &str = include_str!("ui/canvas/mod.rs");
const UI_CANVAS_GEOMETRY_MOD_RS: &str = include_str!("ui/canvas/geometry/mod.rs");
const UI_CANVAS_RESIZE_HANDLE_RS: &str = include_str!("ui/canvas/resize_handle.rs");
const UI_CANVAS_ROUTE_MATH_RS: &str = include_str!("ui/canvas/route_math.rs");
const FRET_CANVAS_INTERACTION_RESIZE_RS: &str =
    include_str!("../../fret-canvas/src/interaction/resize.rs");
const UI_CONTROLLER_RS: &str = include_str!("ui/controller.rs");
const UI_CONTROLLER_STORE_SYNC_RS: &str = include_str!("ui/controller_store_sync.rs");
const UI_CONTROLLER_UPDATES_RS: &str = include_str!("ui/controller_updates.rs");
const UI_CONTROLLER_VIEWPORT_RS: &str = include_str!("ui/controller_viewport.rs");
const UI_DECLARATIVE_MOD_RS: &str = include_str!("ui/declarative/mod.rs");
const UI_MOD_RS: &str = include_str!("ui/mod.rs");
const UI_OVERLAYS_MOD_RS: &str = include_str!("ui/overlays/mod.rs");
const UI_OVERLAY_TOOLBAR_POLICY_RS: &str = include_str!("ui/overlays/toolbar_policy.rs");
const UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS: &str =
    include_str!("ui/overlays/toolbar_layout_policy.rs");
const UI_OVERLAY_TOOLBARS_DECLARATIVE_RS: &str =
    include_str!("ui/overlays/toolbars_declarative.rs");
const UI_VIEWPORT_OPTIONS_RS: &str = include_str!("ui/viewport_options.rs");
const FRET_EXAMPLES_CARGO_TOML: &str = include_str!("../../../apps/fret-examples/Cargo.toml");
const FRET_EXAMPLES_LIB_RS: &str = include_str!("../../../apps/fret-examples/src/lib.rs");
const FRET_DEMO_CARGO_TOML: &str = include_str!("../../../apps/fret-demo/Cargo.toml");
const FRETBOARD_NATIVE_RS: &str = include_str!("../../../apps/fretboard/src/dev/native.rs");
const FRET_NODE_README_MD: &str = include_str!("../README.md");
const NODE_GRAPH_XYFLOW_GUIDE_MD: &str =
    include_str!("../../../docs/node-graph-how-to-build-like-xyflow.md");
const NODE_GRAPH_CONTROLLED_MODE_MD: &str =
    include_str!("../../../docs/node-graph-controlled-mode.md");
const NODE_GRAPH_DEMO_RS: &str = include_str!("../../../apps/fret-examples/src/node_graph_demo.rs");
const UI_GALLERY_CARGO_TOML: &str = include_str!("../../../apps/fret-ui-gallery/Cargo.toml");
const UI_GALLERY_NODE_GRAPH_CULL_TORTURE_RS: &str = include_str!(
    "../../../apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs"
);
const WORKFLOW_NODE_GRAPH_DEMO_RS: &str =
    include_str!("../../../apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs");

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

fn source_without_tests(source: &str) -> &str {
    source.split("#[cfg(test)]").next().unwrap_or(source)
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("source directory should be readable");
    for entry in entries {
        let path = entry.expect("source entry should be readable").path();
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
fn retained_compatibility_surface_is_removed() {
    for (name, source) in [
        ("Cargo.toml", CARGO_TOML),
        ("ui/mod.rs", UI_MOD_RS),
        ("ui/canvas/mod.rs", UI_CANVAS_MOD_RS),
        ("ui/overlays/mod.rs", UI_OVERLAYS_MOD_RS),
    ] {
        for forbidden in [
            "compat-retained-canvas",
            "compat-retained-widgets",
            "compat_retained_canvas",
            "NodeGraphCanvasWith",
            "NodeGraphCanvasCommitOutcome",
            "NodeGraphCanvasMiddleware",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} must not retain `{forbidden}` after retained canvas exit"
            );
        }
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for removed in [
        "src/ui/compat_transport.rs",
        "src/ui/canvas/widget.rs",
        "src/ui/canvas/widget",
        "src/ui/canvas/state.rs",
        "src/ui/canvas/state",
        "src/ui/canvas/middleware.rs",
        "src/ui/canvas/middleware",
        "src/ui/canvas/paint.rs",
        "src/ui/canvas/paint",
        "src/ui/canvas/searcher.rs",
        "src/ui/canvas/searcher",
        "src/ui/canvas/workflow.rs",
        "src/ui/canvas/workflow",
    ] {
        assert!(
            !manifest_dir.join(removed).exists(),
            "retained canvas exit should delete {removed}"
        );
    }
}

#[test]
fn ui_sources_do_not_use_retained_canvas_compatibility() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_root = manifest_dir.join("src/ui");
    let mut files = Vec::new();
    collect_rs_files(&ui_root, &mut files);

    let forbidden_terms = [
        "compat-retained-canvas",
        "compat-retained-widgets",
        "compat_retained_canvas",
        "RetainedSubtreeProps",
        "UiTreeRetainedExt",
        "NodeGraphCanvas::",
        "struct NodeGraphCanvas {",
        "NodeGraphCanvasWith",
        "NodeGraphEditQueue",
        "NodeGraphViewQueue",
    ];

    let mut offenders = Vec::new();
    for path in files {
        let source =
            std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
        for forbidden in forbidden_terms {
            if source.contains(forbidden) {
                offenders.push(format!(
                    "{} contains `{forbidden}`",
                    source_rel_path(&path, &ui_root)
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "retained canvas compatibility terms must stay out of supported UI sources:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn resize_handle_vocabulary_lives_in_fret_canvas() {
    assert!(FRET_CANVAS_INTERACTION_RESIZE_RS.contains("pub enum ResizeHandle2D"));
    assert!(FRET_CANVAS_INTERACTION_RESIZE_RS.contains("pub struct ResizeHandleSet2D"));
    assert!(FRET_CANVAS_INTERACTION_RESIZE_RS.contains("pub const ALL: [Self; 8]"));
    assert!(FRET_CANVAS_INTERACTION_RESIZE_RS.contains("pub const fn affects_left"));

    assert!(UI_CANVAS_RESIZE_HANDLE_RS.contains("ResizeHandle2D as NodeResizeHandle"));
    assert!(UI_CANVAS_RESIZE_HANDLE_RS.contains("ResizeHandleSet2D as NodeResizeHandleSet"));
    assert!(!UI_CANVAS_RESIZE_HANDLE_RS.contains("pub enum NodeResizeHandle"));
    assert!(!UI_CANVAS_RESIZE_HANDLE_RS.contains("pub struct NodeResizeHandleSet"));
}

#[test]
fn public_node_graph_guides_teach_binding_first_surface() {
    assert!(FRET_NODE_README_MD.contains("## Recommended usage (declarative-first)"));
    assert!(FRET_NODE_README_MD.contains("NodeGraphSurfaceBinding"));
    assert!(FRET_NODE_README_MD.contains("node_graph_surface(...)"));
    assert!(!FRET_NODE_README_MD.contains("NodeGraphCanvas::new("));
    assert!(!FRET_NODE_README_MD.contains("NodeGraphCanvas::with_store"));

    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("## Recommended (binding-first) integration"));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("NodeGraphSurfaceBinding::new("));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("node_graph_surface(cx, surface.surface_props())"));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("NodeGraphController::new(surface.store_model())"));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("dispatch_transaction*"));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("fit_view_nodes_in_bounds*"));

    for forbidden in [
        "The UI consumes:",
        "`Model<Graph>` (for painting and hit-testing)",
        "`Model<NodeGraphViewState>` (pan/zoom/selection)",
        "optional `Model<NodeGraphStore>`",
        "NodeGraphCanvas::new(",
        "NodeGraphCanvas::with_store",
    ] {
        assert!(
            !NODE_GRAPH_XYFLOW_GUIDE_MD.contains(forbidden),
            "XyFlow-style guide must stay binding-first; found stale teaching text `{forbidden}`"
        );
    }
}

#[test]
fn raw_transport_surface_stays_removed() {
    assert!(!UI_MOD_RS.contains("mod compat_transport;"));
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
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_with_options_action_host("));
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
    assert!(binding_surface.contains("pub fn fit_view_nodes_in_bounds_with_options_action_host("));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds<"));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_action_host("));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_with_options<"));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_with_options_action_host("));
    assert!(binding_surface.contains("pub fn screen_to_canvas<"));
    assert!(binding_surface.contains("pub fn canvas_to_screen<"));
}

#[test]
fn binding_surface_covers_instance_style_sync_and_history_helpers() {
    let binding_surface = binding_surface();
    assert!(binding_surface.contains(
        "struct NodeGraphSurfaceMirrors {\n    graph: Model<Graph>,\n    view_state: Model<NodeGraphViewState>,\n    editor_config: Model<NodeGraphEditorConfig>,\n}"
    ));
    assert!(binding_surface.contains(
        "pub struct NodeGraphSurfaceBinding {\n    mirrors: NodeGraphSurfaceMirrors,\n    store: Model<NodeGraphStore>,\n    internals: Arc<NodeGraphInternalsStore>,\n}"
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
        binding_surface.contains("pub fn internals_store(&self) -> Arc<NodeGraphInternalsStore> {")
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
fn controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper() {
    assert!(NODE_GRAPH_CONTROLLED_MODE_MD.contains("### Current replace policy"));
    assert!(NODE_GRAPH_CONTROLLED_MODE_MD.contains("**full replace first**"));
    assert!(NODE_GRAPH_CONTROLLED_MODE_MD.contains("NodeGraphSurfaceBinding::replace_document("));
    assert!(
        NODE_GRAPH_CONTROLLED_MODE_MD
            .contains("NodeGraphController::replace_document_and_sync_models(")
    );
    assert!(NODE_GRAPH_CONTROLLED_MODE_MD.contains("replace_graph(...)"));
    assert!(
        NODE_GRAPH_CONTROLLED_MODE_MD.contains(
            "Diff-first replace helpers remain intentionally deferred until we have a concrete"
        ),
        "controlled-mode docs must keep the public helper decision explicit"
    );

    let binding_surface = binding_surface();
    let controlled_sync_sources =
        [binding_surface.as_str(), UI_CONTROLLER_STORE_SYNC_RS].join("\n");

    assert!(controlled_sync_sources.contains("pub fn replace_graph<"));
    assert!(controlled_sync_sources.contains("pub fn replace_document<"));
    assert!(controlled_sync_sources.contains("pub fn replace_graph_and_sync_models<"));
    assert!(controlled_sync_sources.contains("pub fn replace_document_and_sync_models<"));
    assert!(
        !controlled_sync_sources.contains("graph_diff"),
        "public controlled sync helpers should not hide diff-first replace semantics"
    );

    for forbidden in [
        "pub fn replace_graph_with_diff",
        "pub fn replace_document_with_diff",
        "pub fn replace_graph_diff",
        "pub fn replace_document_diff",
        "pub fn apply_graph_diff",
        "pub fn sync_graph_diff",
    ] {
        assert!(
            !controlled_sync_sources.contains(forbidden),
            "diff-first controlled sync remains deferred; found `{forbidden}`"
        );
    }
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
fn root_ui_surface_re_exports_store_first_viewport_option_types_without_raw_view_queue() {
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
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("duration_ms"));
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("interpolate"));
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("ease"));
}

#[test]
fn pure_geometry_and_route_math_helpers_are_supported_without_compat_gating() {
    for (name, source) in [
        ("ui/canvas/mod.rs", UI_CANVAS_MOD_RS),
        ("ui/canvas/geometry/mod.rs", UI_CANVAS_GEOMETRY_MOD_RS),
        ("ui/canvas/route_math.rs", UI_CANVAS_ROUTE_MATH_RS),
    ] {
        assert!(
            !source.contains("compat-retained-canvas"),
            "{name} should not gate pure canvas helpers behind retained compatibility"
        );
    }
    assert!(UI_CANVAS_MOD_RS.contains("mod geometry;"));
    assert!(UI_CANVAS_MOD_RS.contains("mod route_math;"));
    assert!(UI_CANVAS_MOD_RS.contains("mod spatial;"));
    assert!(UI_CANVAS_GEOMETRY_MOD_RS.contains("pub(crate) struct CanvasGeometry"));
    assert!(UI_CANVAS_ROUTE_MATH_RS.contains("mod route_math_curve;"));
    assert!(UI_CANVAS_ROUTE_MATH_RS.contains("mod route_math_tangent;"));
}

#[test]
fn overlay_menu_toolbar_policy_ownership_stays_on_named_seams() {
    assert!(UI_OVERLAYS_MOD_RS.contains("mod toolbar_policy;"));
    assert!(UI_OVERLAYS_MOD_RS.contains("mod toolbar_layout_policy;"));
    assert!(UI_OVERLAYS_MOD_RS.contains("mod toolbars_declarative;"));
    for required in [
        "pub enum NodeGraphToolbarVisibility",
        "pub enum NodeGraphToolbarPosition",
        "pub enum NodeGraphToolbarAlign",
        "pub enum NodeGraphToolbarSize",
        "resolve_node_toolbar_window_target",
        "resolve_edge_toolbar_window_target",
    ] {
        assert!(
            UI_OVERLAY_TOOLBAR_POLICY_RS.contains(required),
            "toolbar public policy surface should stay in toolbar_policy.rs: {required}"
        );
    }
    assert!(
        UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS.contains("visible_toolbar_anchor"),
        "toolbar layout math should stay in toolbar_layout_policy.rs"
    );
    for forbidden in [
        "pub enum NodeGraphToolbarVisibility",
        "pub enum NodeGraphToolbarPosition",
        "pub enum NodeGraphToolbarAlign",
        "pub enum NodeGraphToolbarSize",
        "fn resolve_node_toolbar_window_target",
        "fn resolve_edge_toolbar_window_target",
    ] {
        assert!(
            !source_without_tests(UI_OVERLAY_TOOLBARS_DECLARATIVE_RS).contains(forbidden),
            "declarative toolbar composition should consume toolbar policy, not own it: {forbidden}"
        );
    }
    assert!(
        UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("use super::toolbar_policy::{"),
        "declarative toolbar composition should import the policy seam"
    );
    assert!(
        UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("use super::toolbar_layout_policy::{"),
        "declarative toolbar composition should import the layout-policy seam"
    );
}

#[test]
fn workflow_gallery_surface_stays_binding_first_for_viewport_controls() {
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.observe(cx);"));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("LayoutQueryRegionProps::default()"));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("node_graph_surface(cx, props)"));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.set_viewport_action_host("));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.fit_view_nodes_in_bounds_action_host("));
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
        FRET_DEMO_CARGO_TOML.contains("node-graph-demos = [\"fret-examples/node-graph-demos\"]")
    );
    assert!(FRET_EXAMPLES_LIB_RS.contains("pub mod node_graph_demo;"));
    assert!(NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
    assert!(NODE_GRAPH_DEMO_RS.contains("node_graph_surface_in(cx, props)"));
    assert!(!NODE_GRAPH_DEMO_RS.contains("NodeGraphController"));
    assert!(!NODE_GRAPH_DEMO_RS.contains("binding.controller()"));
}
