use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use fret_canvas::view::{DEFAULT_WHEEL_ZOOM_BASE, DEFAULT_WHEEL_ZOOM_STEP, PanZoom2D};
use fret_canvas::wires as canvas_wires;
use fret_core::scene::{
    ColorSpace, DashPatternV1, GradientStop, LinearGradient, MAX_STOPS, Paint, PaintBindingV1,
    PaintEvalSpaceV1, TileMode,
};
use fret_core::{
    Color, DrawOrder, MouseButton, PathCommand, PathStyle, Point, Px, Rect, SemanticsRole,
    StrokeCapV1, StrokeJoinV1, StrokeStyleV2,
};
use fret_runtime::Model;
use fret_ui::canvas::{CanvasKey, CanvasPainter};
use fret_ui::element::{
    AnyElement, CanvasProps, Elements, Length, PointerRegionProps, SemanticsDecoration,
    SemanticsProps,
};
use fret_ui::{ElementContext, ElementContextAccess, GlobalElementId, Invalidation, UiHost};

use crate::core::Graph;
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::ui::canvas::{CanvasGeometry, CanvasSpatialDerived};
use crate::ui::declarative::view_reducer::{
    apply_pan_by_screen_delta, apply_zoom_about_screen_point, view_from_state,
};
use crate::ui::geometry_overrides::NodeGraphGeometryOverridesRef;
use crate::ui::paint_overrides::{NodeGraphPaintOverridesMap, NodeGraphPaintOverridesRef};
pub use crate::ui::portal_commands::{
    PortalCommandOutcome, PortalTextCommand, PortalTextStepMode, parse_portal_text_command,
    portal_cancel_text_command, portal_step_text_command, portal_step_text_command_with_mode,
    portal_submit_text_command,
};
use crate::ui::presenter::DefaultNodeGraphPresenter;
use crate::ui::style::NodeGraphStyle;
use crate::ui::{
    MeasuredGeometryStore, MeasuredNodeGraphPresenter, NodeGraphInternalsStore,
    NodeGraphPortalNodeLayout, NodeGraphPresenter, NodeGraphSurfaceBinding,
};

#[path = "paint_only/cache.rs"]
mod cache;
#[path = "paint_only/diag.rs"]
mod diag;
#[path = "paint_only/hover_anchor.rs"]
mod hover_anchor;
#[path = "paint_only/input_handlers.rs"]
mod input_handlers;
#[path = "paint_only/overlay_elements.rs"]
mod overlay_elements;
#[path = "paint_only/overlays.rs"]
mod overlays;
#[path = "paint_only/pointer_down.rs"]
mod pointer_down;
#[path = "paint_only/pointer_move.rs"]
mod pointer_move;
#[path = "paint_only/pointer_session.rs"]
mod pointer_session;
#[path = "paint_only/portal_measurement.rs"]
mod portal_measurement;
#[path = "paint_only/portals.rs"]
mod portals;
#[path = "paint_only/selection.rs"]
mod selection;
#[path = "paint_only/semantics.rs"]
mod semantics;
#[path = "paint_only/surface_content.rs"]
mod surface_content;
#[path = "paint_only/surface_frame.rs"]
mod surface_frame;
#[path = "paint_only/surface_math.rs"]
mod surface_math;
#[path = "paint_only/surface_models.rs"]
mod surface_models;
#[path = "paint_only/surface_shell.rs"]
mod surface_shell;
#[path = "paint_only/surface_state.rs"]
mod surface_state;
#[path = "paint_only/surface_support.rs"]
mod surface_support;
#[path = "paint_only/transactions.rs"]
mod transactions;

use self::cache::{
    DerivedGeometryCacheState, EdgePaintCacheState, NodePaintCacheState, NodeRectDraw,
    canvas_viewport_rect, declarative_presenter_revision, paint_debug_grid_cached,
    paint_edges_cached, paint_nodes_cached, sync_derived_cache, sync_edges_cache, sync_grid_cache,
    sync_nodes_cache,
};
#[cfg(test)]
use self::cache::{derived_geometry_cache_key, edges_cache_key, grid_cache_key, nodes_cache_key};
use self::diag::{
    DeclarativeDiagKeyAction, DeclarativeKeyboardZoomAction,
    handle_declarative_diag_key_action_host, handle_declarative_escape_key_action_host,
    handle_declarative_keyboard_zoom_action_host,
};
#[cfg(test)]
use self::hover_anchor::resolve_hover_tooltip_anchor;
use self::hover_anchor::{HoverAnchorStore, sync_hover_anchor_store_in_models};
use self::input_handlers::{
    KeyHandlerParams, PinchHandlerParams, PointerDownHandlerParams, PointerFinishHandlerParams,
    PointerMoveHandlerParams, WheelHandlerParams, build_key_down_capture_handler,
    build_pinch_handler, build_pointer_cancel_handler, build_pointer_down_handler,
    build_pointer_move_handler, build_pointer_up_handler, build_wheel_handler,
};
use self::overlays::{
    HoverTooltipOverlayParams, push_hover_tooltip_overlay_if_needed,
    push_marquee_overlay_if_active, push_overlay_layer_if_needed,
};
use self::pointer_down::{
    begin_left_pointer_down_action_host, begin_pan_pointer_down_action_host,
    read_left_pointer_down_snapshot_action_host,
};
use self::pointer_move::{
    MarqueePointerMoveOutcome, handle_marquee_pointer_move_action_host,
    handle_node_drag_pointer_move_action_host, update_hovered_node_pointer_move_action_host,
};
use self::pointer_session::{
    escape_cancel_declarative_interactions_action_host,
    handle_declarative_pointer_cancel_action_host, handle_declarative_pointer_up_action_host,
    invalidate_notify_and_redraw_pointer_action_host, notify_and_redraw_action_host,
};
use self::portal_measurement::{
    PortalBoundsStore, PortalMeasuredGeometryFlushOutcome, PortalMeasuredGeometryState,
    flush_portal_measured_geometry_state, record_portal_measured_node_size_in_state,
    sync_portal_canvas_bounds_in_models,
};
#[cfg(test)]
use self::portals::collect_portal_label_infos_for_visible_subset;
use self::portals::{apply_pending_fit_to_portals, host_visible_portal_labels};
use self::selection::{
    build_click_selection_preview_nodes, build_marquee_preview_selected_nodes,
    commit_marquee_selection_action_host, commit_pending_selection_action_host,
    effective_selected_nodes_for_paint,
};
use self::semantics::{
    SurfaceSemanticsParams, build_surface_semantics_value, collect_edge_paint_diagnostics,
    collect_portal_diagnostics,
};
use self::surface_content::{SurfaceRegionChildrenParams, build_surface_region_children};
use self::surface_frame::{
    PrepareSurfaceFrameParams, PreparedPaintOnlySurfaceFrame, prepare_surface_frame,
};
use self::surface_math::{
    hit_test_node_at_point, marquee_rect_screen, node_drag_commit_delta, node_drag_contains,
    node_drag_delta_canvas, pointer_crossed_threshold, quantize_f32, rect_approx_eq,
    rect_contains_rect, rect_from_points, rect_union, rects_intersect,
};
use self::surface_models::{PaintOnlySurfaceModels, use_paint_only_surface_models};
use self::surface_shell::{SurfaceShellParams, build_surface_shell};
use self::surface_state::{
    AuthoritativeSurfaceBoundarySnapshot, DragState, GridPaintCacheKeyV2, GridPaintCacheState,
    MarqueeDragState, NodeDragPhase, NodeDragState, PendingSelectionState, PortalDebugFlags,
};
use self::surface_support::{
    authoritative_surface_boundary_snapshot, mouse_buttons_contains,
    read_authoritative_graph_in_models, read_authoritative_view_state_action_host,
    read_authoritative_view_state_in_models, stable_hash_u64,
    sync_authoritative_surface_boundary_in_models, use_uncontrolled_model,
};
use self::transactions::{
    authoritative_graph_snapshot_action_host, build_node_drag_transaction,
    commit_graph_transaction, commit_node_drag_transaction, update_view_state_action_host,
    update_view_state_ui_host,
};

#[cfg(test)]
use self::diag::{DeclarativeDiagViewPreset, apply_declarative_diag_view_preset_action_host};
#[cfg(test)]
use self::diag::{
    build_diag_normalize_visible_node_transaction, build_diag_nudge_visible_node_transaction,
};
#[cfg(test)]
use self::pointer_down::{LeftPointerDownOutcome, LeftPointerDownSnapshot};

#[cfg(test)]
use self::pointer_move::NodeDragPointerMoveOutcome;
#[cfg(test)]
use self::pointer_session::{
    LeftPointerReleaseOutcome, NodeDragReleaseOutcome, complete_left_pointer_release_action_host,
    complete_node_drag_release_action_host, handle_marquee_left_pointer_release_action_host,
    handle_node_drag_left_pointer_release_action_host,
    handle_pending_selection_left_pointer_release_action_host,
    pointer_cancel_declarative_interactions_action_host,
};

#[derive(Debug, Clone, Copy)]
pub struct NodeGraphWheelZoomConfig {
    pub base: f32,
    pub step: f32,
    pub speed: f32,
}

impl Default for NodeGraphWheelZoomConfig {
    fn default() -> Self {
        Self {
            base: DEFAULT_WHEEL_ZOOM_BASE,
            step: DEFAULT_WHEEL_ZOOM_STEP,
            speed: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeGraphVisibleSubsetPortalConfig {
    /// When true, host a lightweight screen-space portal layer for the visible node subset.
    pub enabled: bool,
    /// Upper bound on the number of hosted node portals in a single frame.
    pub max_nodes: usize,
}

impl Default for NodeGraphVisibleSubsetPortalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_nodes: 32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeGraphDiagnosticsConfig {
    /// Enables Ctrl/Cmd-based declarative diagnostics hotkeys.
    pub key_actions_enabled: bool,
    /// Enables diagnostics-only hover tooltip overlays.
    pub hover_tooltip_enabled: bool,
}

/// Per-node portal renderer for the declarative visible-subset portal layer.
///
/// This is the default-path replacement for the retained `NodeGraphPortalHost` renderer callback.
/// It receives a graph snapshot plus the screen-space layout for the currently visible node and
/// returns regular declarative elements. Returning an empty list falls back to the built-in portal
/// label chrome.
pub trait NodeGraphDeclarativePortalRenderer<H: UiHost> {
    fn render_portal(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        graph: &Graph,
        layout: NodeGraphPortalNodeLayout,
    ) -> Elements;
}

/// Command adapter for declarative portal subtrees.
///
/// This is the default-path replacement for the retained `NodeGraphPortalHost::command` adapter.
/// It consumes the shared portal command protocol and returns a policy outcome; the surface owns
/// binding submission, focus return, redraw, and notify side effects.
pub trait NodeGraphDeclarativePortalCommandHandler {
    fn handle_portal_command(
        &mut self,
        host: &mut dyn fret_ui::action::UiFocusActionHost,
        cx: fret_ui::action::ActionCx,
        graph: &Graph,
        command: PortalTextCommand,
    ) -> PortalCommandOutcome;
}

pub type NodeGraphDeclarativePortalCommandHandlerRef =
    Rc<RefCell<dyn NodeGraphDeclarativePortalCommandHandler>>;

#[derive(Clone)]
pub struct NodeGraphSurfaceProps {
    binding: NodeGraphSurfaceBinding,

    pub pointer_region: PointerRegionProps,
    pub canvas: CanvasProps,

    /// Optional geometry overrides for UI-only per-entity sizing/hit-testing.
    ///
    /// Changes must bump `revision()` on the provider (see `ui/geometry_overrides.rs`).
    pub geometry_overrides: Option<NodeGraphGeometryOverridesRef>,

    /// Optional paint-only overrides for UI-only per-node/per-edge styling.
    ///
    /// Changes must bump `revision()` on the provider (see `ui/paint_overrides.rs`).
    pub paint_overrides: Option<NodeGraphPaintOverridesRef>,

    /// Optional measured geometry store shared with declarative portal subtrees.
    ///
    /// When present, derived geometry is built through `MeasuredNodeGraphPresenter`, and hosted
    /// portal subtrees may publish measured node sizes back into the same store.
    pub measured_geometry: Option<Arc<MeasuredGeometryStore>>,

    /// Optional UI-only internals store used to expose active-descendant semantics and editor
    /// overlay geometry without serializing that derived state into graph assets.
    pub a11y_internals: Option<Arc<NodeGraphInternalsStore>>,

    /// Visible-subset portal hosting policy for the declarative editor surface.
    pub portal_hosting: NodeGraphVisibleSubsetPortalConfig,

    /// Optional command handler for declarative portal subtrees.
    pub portal_command_handler: Option<NodeGraphDeclarativePortalCommandHandlerRef>,

    /// Declarative diagnostics policy for debug-only shortcuts and overlays.
    pub diagnostics: NodeGraphDiagnosticsConfig,

    /// Cull margin in screen pixels around the viewport.
    pub cull_margin_screen_px: f32,

    pub pan_button: MouseButton,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub wheel_zoom: NodeGraphWheelZoomConfig,
    pub pinch_zoom_speed: f32,

    pub test_id: Option<Arc<str>>,
}

impl NodeGraphSurfaceProps {
    pub fn new(binding: NodeGraphSurfaceBinding) -> Self {
        let mut pointer_region = PointerRegionProps::default();
        pointer_region.layout.size.width = Length::Fill;
        pointer_region.layout.size.height = Length::Fill;
        let a11y_internals = Some(binding.internals_store());

        Self {
            binding,
            pointer_region,
            canvas: CanvasProps::default(),
            geometry_overrides: None,
            paint_overrides: None,
            measured_geometry: None,
            a11y_internals,
            portal_hosting: NodeGraphVisibleSubsetPortalConfig::default(),
            portal_command_handler: None,
            diagnostics: NodeGraphDiagnosticsConfig::default(),
            cull_margin_screen_px: 256.0,
            pan_button: MouseButton::Middle,
            min_zoom: 0.05,
            max_zoom: 64.0,
            wheel_zoom: NodeGraphWheelZoomConfig::default(),
            pinch_zoom_speed: 1.0,
            test_id: None,
        }
    }

    pub fn binding(&self) -> NodeGraphSurfaceBinding {
        self.binding.clone()
    }
}

/// Paint-only declarative node-graph surface skeleton.
///
/// Notes:
/// - This is intentionally a minimal “M1 skeleton” surface: pan/zoom + a simple grid.
/// - It does **not** yet host node/edge portals or full editor interaction policy.
/// - Escape cancel clears the local pan-drag state, but cannot currently release pointer capture
///   from a key hook (see workstream contract gap log).
#[track_caller]
pub fn node_graph_surface<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphSurfaceProps,
) -> AnyElement {
    node_graph_surface_impl(cx, props, None)
}

/// Declarative node-graph surface with custom visible-subset portal subtree hosting.
///
/// The renderer is invoked only for visible nodes selected by `portal_hosting`. Its output is
/// mounted through the same screen-space portal layer as the built-in lightweight labels and is
/// keyed by `(node id, node kind, node kind_version)` so local element state follows the retained
/// portal lifecycle contract.
#[track_caller]
pub fn node_graph_surface_with_portal_renderer<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphSurfaceProps,
    portal_renderer: &mut dyn NodeGraphDeclarativePortalRenderer<H>,
) -> AnyElement {
    node_graph_surface_impl(cx, props, Some(portal_renderer))
}

fn node_graph_surface_impl<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphSurfaceProps,
    portal_renderer: Option<&mut dyn NodeGraphDeclarativePortalRenderer<H>>,
) -> AnyElement {
    let NodeGraphSurfaceProps {
        binding,
        pointer_region,
        canvas,
        geometry_overrides,
        paint_overrides,
        measured_geometry,
        a11y_internals,
        portal_hosting,
        portal_command_handler,
        diagnostics,
        cull_margin_screen_px,
        pan_button,
        min_zoom,
        max_zoom,
        wheel_zoom,
        pinch_zoom_speed,
        test_id,
    } = props;

    let PaintOnlySurfaceModels {
        drag,
        marquee_drag,
        node_drag,
        pending_selection,
        hovered_node,
        hit_scratch,
        diag_paint_overrides,
        diag_paint_overrides_enabled,
        grid_cache,
        derived_cache,
        edges_cache,
        nodes_cache,
        portal_bounds_store,
        portal_measured_geometry_state,
        portal_debug_flags,
        hover_anchor_store,
        authoritative_surface_boundary,
    } = use_paint_only_surface_models(cx);

    let prepared_frame = prepare_surface_frame(
        cx,
        PrepareSurfaceFrameParams {
            binding: &binding,
            surface_models: &PaintOnlySurfaceModels {
                drag: drag.clone(),
                marquee_drag: marquee_drag.clone(),
                node_drag: node_drag.clone(),
                pending_selection: pending_selection.clone(),
                hovered_node: hovered_node.clone(),
                hit_scratch: hit_scratch.clone(),
                diag_paint_overrides: diag_paint_overrides.clone(),
                diag_paint_overrides_enabled: diag_paint_overrides_enabled.clone(),
                grid_cache: grid_cache.clone(),
                derived_cache: derived_cache.clone(),
                edges_cache: edges_cache.clone(),
                nodes_cache: nodes_cache.clone(),
                portal_bounds_store: portal_bounds_store.clone(),
                portal_measured_geometry_state: portal_measured_geometry_state.clone(),
                portal_debug_flags: portal_debug_flags.clone(),
                hover_anchor_store: hover_anchor_store.clone(),
                authoritative_surface_boundary: authoritative_surface_boundary.clone(),
            },
            geometry_overrides: geometry_overrides.clone(),
            paint_overrides: paint_overrides.clone(),
            measured_geometry: measured_geometry.clone(),
            diagnostics,
            cull_margin_screen_px,
            test_id,
        },
    );

    let view_for_paint = prepared_frame.view_for_paint;
    let theme = prepared_frame.theme;
    let style_tokens = prepared_frame.style_tokens;
    let diagnostics = prepared_frame.diagnostics;
    let diag_paint_overrides_value = prepared_frame.diag_paint_overrides_value;
    let paint_overrides_ref = prepared_frame.paint_overrides_ref;
    let panning = prepared_frame.panning;
    let marquee_value = prepared_frame.marquee_value;
    let marquee_active = prepared_frame.marquee_active;
    let node_drag_value = prepared_frame.node_drag_value;
    let node_dragging = prepared_frame.node_dragging;
    let grid_cache_value = prepared_frame.grid_cache_value;
    let derived_cache_value = prepared_frame.derived_cache_value;
    let nodes_cache_value = prepared_frame.nodes_cache_value;
    let edges_cache_value = prepared_frame.edges_cache_value;
    let hovered_node_value = prepared_frame.hovered_node_value;
    let effective_selected_nodes = prepared_frame.effective_selected_nodes;
    let portals_disabled = prepared_frame.portals_disabled;
    let semantics_value = prepared_frame.semantics_value;
    let test_id = prepared_frame.test_id;

    let a11y_snapshot = a11y_internals
        .as_ref()
        .map(|internals| internals.a11y_snapshot());
    let active_descendant_key = a11y_snapshot
        .as_ref()
        .and_then(|snapshot| active_descendant_key_from_a11y(snapshot));
    let active_descendant_element = Rc::new(Cell::new(None::<GlobalElementId>));
    let active_descendant_element_for_children = active_descendant_element.clone();

    let surface = cx.semantics_with_id(
        SemanticsProps {
            role: SemanticsRole::Viewport,
            label: Some(Arc::from("Node Graph Canvas")),
            test_id: Some(test_id.clone()),
            value: Some(semantics_value.clone()),
            // Make the surface focusable so keyboard actions can route here after pointer-down.
            focusable: true,
            ..Default::default()
        },
        move |cx, _element| {
            if let Some(handler) = portal_command_handler.clone() {
                let binding_for_availability = binding.clone();
                let binding_for_command = binding.clone();
                cx.action_route_fallback_root(_element);
                cx.command_on_command_availability_for(
                    _element,
                    Arc::new(move |host, _cx, command| {
                        let Some(command) = parse_portal_text_command(&command) else {
                            return fret_ui::CommandAvailability::NotHandled;
                        };
                        let node = match command {
                            PortalTextCommand::Submit { node }
                            | PortalTextCommand::Cancel { node }
                            | PortalTextCommand::Step { node, .. } => node,
                        };
                        let has_node = host
                            .models_mut()
                            .read(&binding_for_availability.store_model(), |store| {
                                store.graph().nodes.contains_key(&node)
                            })
                            .ok()
                            .unwrap_or(false);
                        if has_node {
                            fret_ui::CommandAvailability::Available
                        } else {
                            fret_ui::CommandAvailability::NotHandled
                        }
                    }),
                );

                cx.command_on_command_for(
                    _element,
                    Arc::new(move |host, action_cx, command| {
                        let Some(command) = parse_portal_text_command(&command) else {
                            return false;
                        };
                        let graph = host
                            .models_mut()
                            .read(&binding_for_command.store_model(), |store| {
                                store.graph().clone()
                            })
                            .ok()
                            .unwrap_or_default();
                        let outcome = handler
                            .borrow_mut()
                            .handle_portal_command(host, action_cx, &graph, command);

                        match outcome {
                            PortalCommandOutcome::NotHandled => false,
                            PortalCommandOutcome::Handled => {
                                host.request_focus(action_cx.target);
                                host.request_redraw(action_cx.window);
                                host.notify(action_cx);
                                true
                            }
                            PortalCommandOutcome::Commit(tx) => {
                                let _ = commit_graph_transaction(host, &binding_for_command, &tx);
                                host.request_focus(action_cx.target);
                                host.request_redraw(action_cx.window);
                                host.notify(action_cx);
                                true
                            }
                        }
                    }),
                );
            }

            let shell = build_surface_shell(
                cx,
                _element,
                SurfaceShellParams {
                    binding: binding.clone(),
                    pointer_region,
                    canvas,
                    measured_geometry_present: measured_geometry.is_some(),
                    portal_hosting,
                    cull_margin_screen_px,
                    pan_button,
                    min_zoom,
                    max_zoom,
                    wheel_zoom,
                    pinch_zoom_speed,
                    portal_renderer,
                    surface_models: PaintOnlySurfaceModels {
                        drag: drag.clone(),
                        marquee_drag: marquee_drag.clone(),
                        node_drag: node_drag.clone(),
                        pending_selection: pending_selection.clone(),
                        hovered_node: hovered_node.clone(),
                        hit_scratch: hit_scratch.clone(),
                        diag_paint_overrides: diag_paint_overrides.clone(),
                        diag_paint_overrides_enabled: diag_paint_overrides_enabled.clone(),
                        grid_cache: grid_cache.clone(),
                        derived_cache: derived_cache.clone(),
                        edges_cache: edges_cache.clone(),
                        nodes_cache: nodes_cache.clone(),
                        portal_bounds_store: portal_bounds_store.clone(),
                        portal_measured_geometry_state: portal_measured_geometry_state.clone(),
                        portal_debug_flags: portal_debug_flags.clone(),
                        hover_anchor_store: hover_anchor_store.clone(),
                        authoritative_surface_boundary: authoritative_surface_boundary.clone(),
                    },
                    prepared_frame: PreparedPaintOnlySurfaceFrame {
                        view_for_paint,
                        theme: theme.clone(),
                        style_tokens: style_tokens.clone(),
                        diagnostics,
                        diag_paint_overrides_value: diag_paint_overrides_value.clone(),
                        paint_overrides_ref: paint_overrides_ref.clone(),
                        panning,
                        marquee_value: marquee_value.clone(),
                        marquee_active,
                        node_drag_value: node_drag_value.clone(),
                        node_dragging,
                        grid_cache_value: grid_cache_value.clone(),
                        derived_cache_value: derived_cache_value.clone(),
                        nodes_cache_value: nodes_cache_value.clone(),
                        edges_cache_value: edges_cache_value.clone(),
                        hovered_node_value,
                        effective_selected_nodes: effective_selected_nodes.clone(),
                        portals_disabled,
                        semantics_value: semantics_value.clone(),
                        test_id: test_id.clone(),
                    },
                },
            );
            build_surface_semantics_children(
                cx,
                shell,
                a11y_snapshot.as_ref(),
                active_descendant_key,
                &active_descendant_element_for_children,
            )
        },
    );
    match active_descendant_element.get() {
        Some(element) => surface
            .attach_semantics(SemanticsDecoration::default().active_descendant_element(element.0)),
        None => surface,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeGraphA11yDescendantKey {
    FocusedPort,
    FocusedEdge,
    FocusedNode,
}

fn active_descendant_key_from_a11y(
    snapshot: &crate::ui::internals::NodeGraphA11ySnapshot,
) -> Option<NodeGraphA11yDescendantKey> {
    if snapshot.focused_port.is_some() {
        Some(NodeGraphA11yDescendantKey::FocusedPort)
    } else if snapshot.focused_edge.is_some() {
        Some(NodeGraphA11yDescendantKey::FocusedEdge)
    } else if snapshot.focused_node.is_some() {
        Some(NodeGraphA11yDescendantKey::FocusedNode)
    } else {
        None
    }
}

fn build_surface_semantics_children<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    mut shell: Vec<AnyElement>,
    a11y: Option<&crate::ui::internals::NodeGraphA11ySnapshot>,
    active_descendant_key: Option<NodeGraphA11yDescendantKey>,
    active_descendant_element: &Rc<Cell<Option<GlobalElementId>>>,
) -> Vec<AnyElement> {
    if let Some(a11y) = a11y {
        shell.extend(build_a11y_descendant_semantics(
            cx,
            a11y,
            active_descendant_key,
            active_descendant_element,
        ));
    }
    shell
}

fn build_a11y_descendant_semantics<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    a11y: &crate::ui::internals::NodeGraphA11ySnapshot,
    active_descendant_key: Option<NodeGraphA11yDescendantKey>,
    active_descendant_element: &Rc<Cell<Option<GlobalElementId>>>,
) -> Vec<AnyElement> {
    [
        (
            NodeGraphA11yDescendantKey::FocusedPort,
            a11y.focused_port_label.clone().or_else(|| {
                a11y.focused_port
                    .map(|port| format!("Focused port {port:?}"))
            }),
        ),
        (
            NodeGraphA11yDescendantKey::FocusedEdge,
            a11y.focused_edge_label.clone().or_else(|| {
                a11y.focused_edge
                    .map(|edge| format!("Focused edge {edge:?}"))
            }),
        ),
        (
            NodeGraphA11yDescendantKey::FocusedNode,
            a11y.focused_node_label.clone().or_else(|| {
                a11y.focused_node
                    .map(|node| format!("Focused node {node:?}"))
            }),
        ),
    ]
    .into_iter()
    .filter_map(|(key, label)| {
        label.map(|label| {
            let active_descendant_element = active_descendant_element.clone();
            let mut props = SemanticsProps {
                role: SemanticsRole::Generic,
                label: Some(Arc::<str>::from(label)),
                ..Default::default()
            };
            props.layout.size.width = Length::Px(Px(0.0));
            props.layout.size.height = Length::Px(Px(0.0));
            cx.keyed(key, move |cx| {
                cx.semantics_with_id(props, |_cx, id| {
                    if active_descendant_key == Some(key) {
                        active_descendant_element.set(Some(id));
                    }
                    Vec::new()
                })
            })
        })
    })
    .collect()
}

#[cfg(test)]
#[path = "paint_only/tests.rs"]
mod tests;

/// Capability-first adapter for [`node_graph_surface`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn node_graph_surface_in<'a, H: UiHost + 'static + 'a, Cx>(
    cx: &mut Cx,
    props: NodeGraphSurfaceProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    node_graph_surface(cx.elements(), props)
}

/// Capability-first adapter for [`node_graph_surface_with_portal_renderer`].
#[track_caller]
pub fn node_graph_surface_with_portal_renderer_in<'a, H: UiHost + 'static + 'a, Cx>(
    cx: &mut Cx,
    props: NodeGraphSurfaceProps,
    portal_renderer: &mut dyn NodeGraphDeclarativePortalRenderer<H>,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    node_graph_surface_with_portal_renderer(cx.elements(), props, portal_renderer)
}
