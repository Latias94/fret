use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    Size, TextBlobId, TextConstraints, TextOverflow, TextWrap, Transform2D,
};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::{UiHost, retained_bridge::*};

use crate::REROUTE_KIND;
use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, Graph, NodeId as GraphNodeId, NodeKindKey,
    PortDirection, PortId,
};
use crate::interaction::NodeGraphConnectionMode;
use crate::io::{NodeGraphInteractionState, NodeGraphViewState};
use crate::ops::{
    GraphFragment, GraphHistory, GraphOp, GraphOpBuilderExt, GraphTransaction, IdRemapSeed,
    IdRemapper, PasteTuning, apply_transaction,
};
use crate::profile::{ApplyPipelineError, apply_transaction_with_profile};
use crate::rules::{ConnectDecision, Diagnostic, DiagnosticSeverity, EdgeEndpoint};
use crate::runtime::callbacks::{
    ConnectDragKind, ConnectEnd, ConnectEndOutcome, ConnectStart, NodeDragEnd, NodeDragEndOutcome,
    NodeDragStart, ViewportMoveEnd, ViewportMoveEndOutcome, ViewportMoveKind, ViewportMoveStart,
};
use crate::runtime::callbacks::{NodeGraphCallbacks, connection_changes_from_transaction};
use crate::runtime::changes::NodeGraphChanges;
use crate::runtime::events::ViewChange;
use crate::runtime::store::NodeGraphStore;

use crate::ui::commands::{
    CMD_NODE_GRAPH_ACTIVATE, CMD_NODE_GRAPH_ALIGN_BOTTOM, CMD_NODE_GRAPH_ALIGN_CENTER_X,
    CMD_NODE_GRAPH_ALIGN_CENTER_Y, CMD_NODE_GRAPH_ALIGN_LEFT, CMD_NODE_GRAPH_ALIGN_RIGHT,
    CMD_NODE_GRAPH_ALIGN_TOP, CMD_NODE_GRAPH_COPY, CMD_NODE_GRAPH_CREATE_GROUP, CMD_NODE_GRAPH_CUT,
    CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_DISTRIBUTE_X, CMD_NODE_GRAPH_DISTRIBUTE_Y,
    CMD_NODE_GRAPH_DUPLICATE, CMD_NODE_GRAPH_FOCUS_NEXT, CMD_NODE_GRAPH_FOCUS_NEXT_EDGE,
    CMD_NODE_GRAPH_FOCUS_NEXT_PORT, CMD_NODE_GRAPH_FOCUS_PORT_DOWN, CMD_NODE_GRAPH_FOCUS_PORT_LEFT,
    CMD_NODE_GRAPH_FOCUS_PORT_RIGHT, CMD_NODE_GRAPH_FOCUS_PORT_UP, CMD_NODE_GRAPH_FOCUS_PREV,
    CMD_NODE_GRAPH_FOCUS_PREV_EDGE, CMD_NODE_GRAPH_FOCUS_PREV_PORT, CMD_NODE_GRAPH_FRAME_ALL,
    CMD_NODE_GRAPH_FRAME_SELECTION, CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT,
    CMD_NODE_GRAPH_GROUP_RENAME, CMD_NODE_GRAPH_GROUP_SEND_TO_BACK, CMD_NODE_GRAPH_INSERT_REROUTE,
    CMD_NODE_GRAPH_NUDGE_DOWN, CMD_NODE_GRAPH_NUDGE_DOWN_FAST, CMD_NODE_GRAPH_NUDGE_LEFT,
    CMD_NODE_GRAPH_NUDGE_LEFT_FAST, CMD_NODE_GRAPH_NUDGE_RIGHT, CMD_NODE_GRAPH_NUDGE_RIGHT_FAST,
    CMD_NODE_GRAPH_NUDGE_UP, CMD_NODE_GRAPH_NUDGE_UP_FAST, CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER,
    CMD_NODE_GRAPH_OPEN_INSERT_NODE, CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
    CMD_NODE_GRAPH_PASTE, CMD_NODE_GRAPH_REDO, CMD_NODE_GRAPH_RESET_VIEW,
    CMD_NODE_GRAPH_SELECT_ALL, CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE, CMD_NODE_GRAPH_UNDO,
    CMD_NODE_GRAPH_ZOOM_IN, CMD_NODE_GRAPH_ZOOM_OUT,
};
use crate::ui::presenter::{
    DefaultNodeGraphPresenter, EdgeRenderHint, EdgeRouteKind, InsertNodeCandidate,
    NodeGraphContextMenuAction, NodeGraphContextMenuItem, NodeGraphPresenter, NodeResizeHandleSet,
    PortAnchorHint,
};
use crate::ui::style::NodeGraphStyle;
use crate::ui::{
    FallbackMeasuredNodeGraphPresenter, GroupRenameOverlay, MeasuredGeometryStore,
    NodeGraphCanvasTransform, NodeGraphEdgeTypes, NodeGraphEditQueue, NodeGraphInternalsSnapshot,
    NodeGraphInternalsStore, NodeGraphOverlayState,
};

use super::paint::CanvasPaintCache;
use super::state::ViewportMoveDebounceState;

mod cancel;
mod context_menu;
mod cursor;
mod edge_drag;
mod edge_insert;
mod group_drag;
mod group_resize;
mod hover;
mod left_click;
mod marquee;
mod node_drag;
mod node_resize;
mod pan_zoom;
mod pending_drag;
mod pending_group_drag;
mod pending_group_resize;
mod pending_resize;
mod pending_wire_drag;
mod pointer_up;
mod right_click;
mod searcher;
mod sticky_wire;
mod threshold;
mod wire_drag;

use super::conversion;
use super::geometry::group_order;
use super::geometry::{CanvasGeometry, node_ports};
use super::route_math::{
    cubic_bezier, cubic_bezier_derivative, edge_route_end_tangent, edge_route_start_tangent,
    normal_from_tangent, wire_ctrl_points,
};
use super::searcher::{SEARCHER_MAX_VISIBLE_ROWS, SearcherRow, SearcherRowKind};
use super::snaplines::SnapGuides;
use super::spatial::CanvasSpatialIndex;
use super::state::{
    ContextMenuState, ContextMenuTarget, GeometryCache, GeometryCacheKey, InteractionState,
    InternalsCacheKey, MarqueeDrag, NodeResizeHandle, PanInertiaState, PasteSeries, PendingPaste,
    SearcherState, ToastState, ViewSnapshot, WireDrag, WireDragKind,
};
use super::workflow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlignDistributeMode {
    AlignLeft,
    AlignRight,
    AlignTop,
    AlignBottom,
    AlignCenterX,
    AlignCenterY,
    DistributeX,
    DistributeY,
}

#[derive(Debug, Clone, Copy)]
enum PortNavDir {
    Left,
    Right,
    Up,
    Down,
}

/// Retained node-graph canvas widget (MVP).
///
/// This draws nodes and wires and supports:
/// - pan (MMB drag, wheel without Ctrl),
/// - zoom (Ctrl+wheel, centered),
/// - node drag (LMB),
/// - connect ports (LMB drag pin -> pin).
pub struct NodeGraphCanvas {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    store: Option<Model<NodeGraphStore>>,
    store_rev: Option<u64>,
    presenter: Box<dyn NodeGraphPresenter>,
    edge_types: Option<NodeGraphEdgeTypes>,
    callbacks: Option<Box<dyn NodeGraphCallbacks>>,
    style: NodeGraphStyle,
    close_command: Option<CommandId>,

    auto_measured: Arc<MeasuredGeometryStore>,
    auto_measured_key: Option<(u64, u32)>,

    edit_queue: Option<Model<NodeGraphEditQueue>>,
    edit_queue_key: Option<u64>,
    overlays: Option<Model<NodeGraphOverlayState>>,

    measured_output: Option<Arc<MeasuredGeometryStore>>,
    measured_output_key: Option<GeometryCacheKey>,

    internals: Option<Arc<NodeGraphInternalsStore>>,
    internals_key: Option<InternalsCacheKey>,

    cached_pan: CanvasPoint,
    cached_zoom: f32,
    history: GraphHistory,
    geometry: GeometryCache,

    paint_cache: CanvasPaintCache,
    text_blobs: Vec<TextBlobId>,
    interaction: InteractionState,
}

impl NodeGraphCanvas {
    const REROUTE_INPUTS: usize = 1;
    const REROUTE_OUTPUTS: usize = 1;
    const AUTO_PAN_TICK_HZ: f32 = 60.0;
    const AUTO_PAN_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::AUTO_PAN_TICK_HZ) as u64);
    const PAN_INERTIA_TICK_HZ: f32 = 60.0;
    const PAN_INERTIA_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::PAN_INERTIA_TICK_HZ) as u64);
    const VIEWPORT_MOVE_END_DEBOUNCE: Duration = Duration::from_millis(180);
    const EDGE_FOCUS_ANCHOR_SIZE_SCREEN: f32 = 16.0;
    const EDGE_FOCUS_ANCHOR_PAD_SCREEN: f32 = 1.0;
    const EDGE_FOCUS_ANCHOR_BORDER_SCREEN: f32 = 2.0;
    const EDGE_FOCUS_ANCHOR_OFFSET_SCREEN: f32 = 18.0;

    fn show_toast<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        severity: DiagnosticSeverity,
        message: impl Into<Arc<str>>,
    ) {
        if let Some(prev) = self.interaction.toast.take() {
            host.push_effect(Effect::CancelTimer { token: prev.timer });
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Duration::from_millis(2400),
            repeat: None,
        });

        self.interaction.toast = Some(ToastState {
            timer,
            severity,
            message: message.into(),
        });
    }

    fn edge_render_hint(&self, graph: &Graph, edge_id: EdgeId) -> EdgeRenderHint {
        let base = self.presenter.edge_render_hint(graph, edge_id, &self.style);
        if let Some(edge_types) = self.edge_types.as_ref() {
            edge_types.apply(graph, edge_id, &self.style, base)
        } else {
            base
        }
    }

    fn emit_graph_callbacks(&mut self, committed: &GraphTransaction, changes: &NodeGraphChanges) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };

        callbacks.on_graph_commit(committed, changes);
        if !changes.nodes.is_empty() {
            callbacks.on_nodes_change(&changes.nodes);
        }
        if !changes.edges.is_empty() {
            callbacks.on_edges_change(&changes.edges);
        }
        for change in connection_changes_from_transaction(committed) {
            callbacks.on_connection_change(change);
            match change {
                crate::runtime::callbacks::ConnectionChange::Connected(conn) => {
                    callbacks.on_connect(conn)
                }
                crate::runtime::callbacks::ConnectionChange::Disconnected(conn) => {
                    callbacks.on_disconnect(conn)
                }
                crate::runtime::callbacks::ConnectionChange::Reconnected { edge, from, to } => {
                    callbacks.on_reconnect(edge, from, to);
                    callbacks.on_edge_update(edge, from, to);
                }
            }
        }

        let deleted = crate::runtime::callbacks::delete_changes_from_transaction(committed);
        if !deleted.nodes.is_empty() {
            callbacks.on_nodes_delete(&deleted.nodes);
        }
        if !deleted.edges.is_empty() {
            callbacks.on_edges_delete(&deleted.edges);
        }
        if !deleted.groups.is_empty() {
            callbacks.on_groups_delete(&deleted.groups);
        }
        if !deleted.sticky_notes.is_empty() {
            callbacks.on_sticky_notes_delete(&deleted.sticky_notes);
        }
        if !deleted.nodes.is_empty()
            || !deleted.edges.is_empty()
            || !deleted.groups.is_empty()
            || !deleted.sticky_notes.is_empty()
        {
            callbacks.on_delete(deleted);
        }
    }

    fn drag_kind_from_wire_drag_kind(kind: &WireDragKind) -> ConnectDragKind {
        match kind {
            WireDragKind::New { from, bundle } => ConnectDragKind::New {
                from: *from,
                bundle: bundle.clone(),
            },
            WireDragKind::Reconnect {
                edge,
                endpoint,
                fixed,
            } => ConnectDragKind::Reconnect {
                edge: *edge,
                endpoint: *endpoint,
                fixed: *fixed,
            },
            WireDragKind::ReconnectMany { edges } => ConnectDragKind::ReconnectMany {
                edges: edges.clone(),
            },
        }
    }

    pub(super) fn emit_connect_start(&mut self, snapshot: &ViewSnapshot, kind: &WireDragKind) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        let ev = ConnectStart {
            kind: Self::drag_kind_from_wire_drag_kind(kind),
            mode: snapshot.interaction.connection_mode,
        };
        callbacks.on_connect_start(ev.clone());
        if matches!(
            kind,
            WireDragKind::Reconnect { .. } | WireDragKind::ReconnectMany { .. }
        ) {
            callbacks.on_reconnect_start(ev.clone());
            callbacks.on_edge_update_start(ev);
        }
    }

    pub(super) fn emit_connect_end(
        &mut self,
        mode: crate::interaction::NodeGraphConnectionMode,
        kind: &WireDragKind,
        target: Option<PortId>,
        outcome: ConnectEndOutcome,
    ) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        let ev = ConnectEnd {
            kind: Self::drag_kind_from_wire_drag_kind(kind),
            mode,
            target,
            outcome,
        };
        callbacks.on_connect_end(ev.clone());
        if matches!(
            kind,
            WireDragKind::Reconnect { .. } | WireDragKind::ReconnectMany { .. }
        ) {
            callbacks.on_reconnect_end(ev.clone());
            callbacks.on_edge_update_end(ev);
        }
    }

    pub(super) fn emit_move_start(&mut self, snapshot: &ViewSnapshot, kind: ViewportMoveKind) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_move_start(ViewportMoveStart {
            kind,
            pan: snapshot.pan,
            zoom: snapshot.zoom,
        });
    }

    pub(super) fn emit_move_end(
        &mut self,
        snapshot: &ViewSnapshot,
        kind: ViewportMoveKind,
        outcome: ViewportMoveEndOutcome,
    ) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_move_end(ViewportMoveEnd {
            kind,
            pan: snapshot.pan,
            zoom: snapshot.zoom,
            outcome,
        });
    }

    pub(super) fn emit_node_drag_start(&mut self, primary: GraphNodeId, nodes: &[GraphNodeId]) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_node_drag_start(NodeDragStart {
            primary,
            nodes: nodes.to_vec(),
        });
    }

    pub(super) fn emit_node_drag_end(
        &mut self,
        primary: GraphNodeId,
        nodes: &[GraphNodeId],
        outcome: NodeDragEndOutcome,
    ) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_node_drag_end(NodeDragEnd {
            primary,
            nodes: nodes.to_vec(),
            outcome,
        });
    }

    pub(super) fn emit_node_drag(&mut self, primary: GraphNodeId, nodes: &[GraphNodeId]) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_node_drag(primary, nodes);
    }

    fn emit_view_callbacks(&mut self, changes: &[ViewChange]) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        if changes.is_empty() {
            return;
        }
        callbacks.on_view_change(changes);
        for change in changes {
            match change {
                ViewChange::Viewport { pan, zoom } => {
                    callbacks.on_viewport_change(*pan, *zoom);
                    callbacks.on_move(*pan, *zoom);
                }
                ViewChange::Selection {
                    nodes,
                    edges,
                    groups,
                } => callbacks.on_selection_change(crate::runtime::callbacks::SelectionChange {
                    nodes: nodes.clone(),
                    edges: edges.clone(),
                    groups: groups.clone(),
                }),
            }
        }
    }

    fn toast_from_diagnostics(diags: &[Diagnostic]) -> Option<(DiagnosticSeverity, Arc<str>)> {
        let first = diags.first()?;
        if first.message.is_empty() {
            return None;
        }
        Some((first.severity, Arc::<str>::from(first.message.clone())))
    }

    fn repair_focused_edge_after_graph_change<H: UiHost>(
        &mut self,
        host: &mut H,
        preferred: Option<EdgeId>,
    ) {
        if preferred.is_none() && self.interaction.focused_edge.is_none() {
            return;
        }

        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.edges_focusable && !snapshot.interaction.edges_reconnectable {
            self.interaction.focused_edge = None;
            return;
        }

        let (edges, current_valid) = self
            .graph
            .read_ref(host, |g| {
                let mut edges: Vec<EdgeId> = g.edges.keys().copied().collect();
                edges.sort_unstable();
                let current = self.interaction.focused_edge;
                let current_valid = current.is_some_and(|id| g.edges.contains_key(&id));
                (edges, current_valid)
            })
            .ok()
            .unwrap_or_default();

        if edges.is_empty() {
            self.interaction.focused_edge = None;
            return;
        }

        if current_valid {
            return;
        }

        let base = preferred.or(self.interaction.focused_edge);
        let next = match base {
            Some(id) => match edges.binary_search(&id) {
                Ok(ix) => edges.get(ix).copied(),
                Err(ix) => edges.get(ix).copied().or_else(|| edges.first().copied()),
            },
            None => edges.first().copied(),
        };
        self.interaction.focused_edge = next;
    }

    fn draw_order_hash(ids: &[GraphNodeId]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        ids.hash(&mut hasher);
        hasher.finish()
    }

    fn canvas_geometry<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> Arc<CanvasGeometry> {
        let graph_rev = self.graph.revision(host).unwrap_or(0);
        let presenter_rev = self.presenter.geometry_revision();
        let key = GeometryCacheKey {
            graph_rev,
            zoom_bits: snapshot.zoom.to_bits(),
            draw_order_hash: Self::draw_order_hash(&snapshot.draw_order),
            presenter_rev,
        };

        if self.geometry.key != Some(key) {
            let style = self.style.clone();
            let draw_order = snapshot.draw_order.clone();
            let zoom = snapshot.zoom;
            let graph = self.graph.clone();
            let presenter = &mut *self.presenter;
            let (geom, index) = graph
                .read_ref(host, |graph| {
                    let geom = CanvasGeometry::build_with_presenter(
                        graph,
                        &draw_order,
                        &style,
                        zoom,
                        presenter,
                    );
                    let max_hit_pad_canvas = 96.0 / zoom.max(1.0e-6);
                    let index = CanvasSpatialIndex::build(graph, &geom, zoom, max_hit_pad_canvas);
                    (geom, index)
                })
                .ok()
                .unwrap_or_else(|| (CanvasGeometry::default(), CanvasSpatialIndex::empty()));
            self.geometry.key = Some(key);
            self.geometry.geom = Arc::new(geom);
            self.geometry.index = Arc::new(index);
        }

        self.geometry.geom.clone()
    }

    fn edge_focus_anchor_rect(center: Point, zoom: f32) -> Rect {
        let z = zoom.max(1.0e-6);
        let half = 0.5 * Self::EDGE_FOCUS_ANCHOR_SIZE_SCREEN / z;
        let pad = Self::EDGE_FOCUS_ANCHOR_PAD_SCREEN / z;
        let size = 2.0 * (half + pad);
        Rect::new(
            Point::new(Px(center.x.0 - half - pad), Px(center.y.0 - half - pad)),
            Size::new(Px(size), Px(size)),
        )
    }

    fn edge_focus_anchor_centers(
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
    ) -> (Point, Point) {
        fn norm_or_fallback(v: Point, fallback: Point) -> Point {
            let len = (v.x.0 * v.x.0 + v.y.0 * v.y.0).sqrt();
            if len.is_finite() && len > 1.0e-6 {
                return Point::new(Px(v.x.0 / len), Px(v.y.0 / len));
            }
            let len = (fallback.x.0 * fallback.x.0 + fallback.y.0 * fallback.y.0).sqrt();
            if len.is_finite() && len > 1.0e-6 {
                return Point::new(Px(fallback.x.0 / len), Px(fallback.y.0 / len));
            }
            Point::new(Px(1.0), Px(0.0))
        }

        let z = zoom.max(1.0e-6);
        let off = Self::EDGE_FOCUS_ANCHOR_OFFSET_SCREEN / z;
        let fallback = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));

        let start_dir = norm_or_fallback(edge_route_start_tangent(route, from, to, zoom), fallback);
        let end_dir = norm_or_fallback(edge_route_end_tangent(route, from, to, zoom), fallback);

        let start = Point::new(
            Px(from.x.0 + start_dir.x.0 * off),
            Px(from.y.0 + start_dir.y.0 * off),
        );
        let end = Point::new(
            Px(to.x.0 - end_dir.x.0 * off),
            Px(to.y.0 - end_dir.y.0 * off),
        );
        (start, end)
    }

    pub(super) fn canvas_derived<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> (Arc<CanvasGeometry>, Arc<CanvasSpatialIndex>) {
        let geom = self.canvas_geometry(host, snapshot);
        let index = self.geometry.index.clone();
        (geom, index)
    }

    pub fn new(graph: Model<Graph>, view_state: Model<NodeGraphViewState>) -> Self {
        let auto_measured = Arc::new(MeasuredGeometryStore::new());
        Self {
            graph,
            view_state,
            store: None,
            store_rev: None,
            presenter: Box::new(FallbackMeasuredNodeGraphPresenter::new(
                DefaultNodeGraphPresenter::default(),
                auto_measured.clone(),
            )),
            edge_types: None,
            callbacks: None,
            style: NodeGraphStyle::default(),
            close_command: None,
            auto_measured,
            auto_measured_key: None,
            edit_queue: None,
            edit_queue_key: None,
            overlays: None,
            measured_output: None,
            measured_output_key: None,
            internals: None,
            internals_key: None,
            cached_pan: CanvasPoint::default(),
            cached_zoom: 1.0,
            history: GraphHistory::default(),
            geometry: GeometryCache::default(),
            paint_cache: CanvasPaintCache::default(),
            text_blobs: Vec::new(),
            interaction: InteractionState::default(),
        }
    }

    pub fn with_presenter(mut self, presenter: impl NodeGraphPresenter + 'static) -> Self {
        self.presenter = Box::new(FallbackMeasuredNodeGraphPresenter::new(
            presenter,
            self.auto_measured.clone(),
        ));
        self
    }

    /// Attaches a B-layer `edgeTypes` registry to override edge render hints.
    pub fn with_edge_types(mut self, edge_types: NodeGraphEdgeTypes) -> Self {
        self.edge_types = Some(edge_types);
        self
    }

    /// Attaches B-layer callbacks for controlled/uncontrolled integrations (ReactFlow-style).
    ///
    /// This is a convenience surface: callbacks are invoked for commits originating from this
    /// canvas (including undo/redo).
    pub fn with_callbacks(mut self, callbacks: impl NodeGraphCallbacks) -> Self {
        self.callbacks = Some(Box::new(callbacks));
        self
    }

    /// Configures a store to receive derived measured geometry each frame.
    ///
    /// This is an output-only surface (similar to XyFlow "internals"):
    /// - the graph model stays pure data,
    /// - derived geometry is published for overlays/tooling,
    /// - the store is not consulted by this canvas unless you explicitly pass it into a presenter.
    pub fn with_measured_output_store(mut self, store: Arc<MeasuredGeometryStore>) -> Self {
        self.measured_output = Some(store);
        self.measured_output_key = None;
        self
    }

    pub fn with_internals_store(mut self, store: Arc<NodeGraphInternalsStore>) -> Self {
        self.internals = Some(store);
        self
    }

    pub fn with_style(mut self, style: NodeGraphStyle) -> Self {
        self.style = style;
        self.geometry.key = None;
        self
    }

    /// Adds a screen-space close button overlay that dispatches a command when clicked.
    ///
    /// This is intended for demos and tool windows; production apps typically wire close actions
    /// via docking/tab chrome instead.
    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = Some(command);
        self
    }

    /// Attaches a UI-side edit queue (`Model<NodeGraphEditQueue>`).
    ///
    /// The canvas drains queued transactions during layout and commits them through its normal
    /// apply + history pipeline (undo/redo).
    pub fn with_edit_queue(mut self, queue: Model<NodeGraphEditQueue>) -> Self {
        self.edit_queue = Some(queue);
        self.edit_queue_key = None;
        self
    }

    /// Attaches an overlay state model (`Model<NodeGraphOverlayState>`).
    pub fn with_overlay_state(mut self, overlays: Model<NodeGraphOverlayState>) -> Self {
        self.overlays = Some(overlays);
        self
    }

    /// Attaches a B-layer runtime store.
    ///
    /// When set, viewport and selection updates are written into the store and pulled back into
    /// `view_state` on demand.
    pub fn with_store(mut self, store: Model<NodeGraphStore>) -> Self {
        self.store = Some(store);
        self.store_rev = None;
        self
    }

    fn close_button_rect(pan: CanvasPoint, zoom: f32) -> Rect {
        let margin = 12.0 / zoom;
        let w = 64.0 / zoom;
        let h = 24.0 / zoom;
        Rect::new(
            Point::new(Px(-pan.x + margin), Px(-pan.y + margin)),
            Size::new(Px(w), Px(h)),
        )
    }

    fn rect_contains(rect: Rect, pos: Point) -> bool {
        pos.x.0 >= rect.origin.x.0
            && pos.y.0 >= rect.origin.y.0
            && pos.x.0 <= rect.origin.x.0 + rect.size.width.0
            && pos.y.0 <= rect.origin.y.0 + rect.size.height.0
    }

    fn resize_handle_rect(&self, node_rect: Rect, zoom: f32) -> Rect {
        self.node_resize_handle_rect(node_rect, NodeResizeHandle::BottomRight, zoom)
    }

    pub(crate) fn node_resize_handle_rect(
        &self,
        node_rect: Rect,
        handle: NodeResizeHandle,
        zoom: f32,
    ) -> Rect {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let min_size = 1.0 / zoom.max(1.0e-6);
        let size = (self.style.resize_handle_size / zoom).max(min_size);

        // Prevent resize handles from covering the full node for small nodes (which would make it
        // impossible to click/drag the node body without starting a resize).
        let max_w = (0.25 * node_rect.size.width.0.max(0.0)).max(min_size);
        let max_h = (0.25 * node_rect.size.height.0.max(0.0)).max(min_size);
        let size = size.min(max_w).min(max_h);

        let x0 = node_rect.origin.x.0;
        let y0 = node_rect.origin.y.0;
        let x1 = node_rect.origin.x.0 + node_rect.size.width.0;
        let y1 = node_rect.origin.y.0 + node_rect.size.height.0;

        let cx = x0 + 0.5 * (x1 - x0 - size);
        let cy = y0 + 0.5 * (y1 - y0 - size);

        let (x, y) = match handle {
            NodeResizeHandle::TopLeft => (x0, y0),
            NodeResizeHandle::Top => (cx, y0),
            NodeResizeHandle::TopRight => (x1 - size, y0),
            NodeResizeHandle::Right => (x1 - size, cy),
            NodeResizeHandle::BottomRight => (x1 - size, y1 - size),
            NodeResizeHandle::Bottom => (cx, y1 - size),
            NodeResizeHandle::BottomLeft => (x0, y1 - size),
            NodeResizeHandle::Left => (x0, cy),
        };

        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(size), Px(size)))
    }

    fn sync_view_state<H: UiHost>(&mut self, host: &mut H) -> ViewSnapshot {
        self.sync_view_state_from_store_if_needed(host);

        let mut snapshot = ViewSnapshot {
            pan: self.cached_pan,
            zoom: self.cached_zoom,
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            selected_groups: Vec::new(),
            draw_order: Vec::new(),
            group_draw_order: Vec::new(),
            interaction: NodeGraphInteractionState::default(),
        };

        let _ = self.view_state.read(host, |_host, s| {
            snapshot.pan = s.pan;
            snapshot.zoom = s.zoom;
            snapshot.selected_nodes = s.selected_nodes.clone();
            snapshot.selected_edges = s.selected_edges.clone();
            snapshot.selected_groups = s.selected_groups.clone();
            snapshot.draw_order = s.draw_order.clone();
            snapshot.group_draw_order = s.group_draw_order.clone();
            snapshot.interaction = s.interaction.clone();
        });

        let zoom = snapshot.zoom;
        if zoom.is_finite() && zoom > 0.0 {
            self.cached_zoom = zoom.clamp(self.style.min_zoom, self.style.max_zoom);
        } else {
            self.cached_zoom = 1.0;
        }
        self.cached_pan = snapshot.pan;
        snapshot.zoom = self.cached_zoom;
        snapshot.pan = self.cached_pan;

        snapshot
    }

    fn sync_view_state_from_store_if_needed<H: UiHost>(&mut self, host: &mut H) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let Some(rev) = store.revision(host) else {
            return;
        };
        if self.store_rev == Some(rev) {
            return;
        }
        self.store_rev = Some(rev);

        let Ok((next_view, next_graph)) =
            store.read_ref(host, |s| (s.view_state().clone(), s.graph().clone()))
        else {
            return;
        };
        let _ = self.graph.update(host, |g, _cx| {
            *g = next_graph;
        });
        let _ = self.view_state.update(host, |s, _cx| {
            *s = next_view;
        });
    }

    fn drain_edit_queue<H: UiHost>(&mut self, host: &mut H, window: Option<AppWindowId>) {
        let Some(queue) = self.edit_queue.as_ref() else {
            return;
        };
        let Some(rev) = queue.revision(host) else {
            return;
        };
        if self.edit_queue_key == Some(rev) {
            return;
        }
        self.edit_queue_key = Some(rev);

        let Ok(txs) = queue.update(host, |q, _cx| q.drain()) else {
            return;
        };
        for tx in txs {
            let _ = self.commit_transaction(host, window, &tx);
        }
    }

    fn update_auto_measured_node_sizes<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        let graph_rev = self.graph.revision(cx.app).unwrap_or(0);
        let scale_bits = cx.scale_factor.to_bits();
        let key = (graph_rev, scale_bits);
        if self.auto_measured_key == Some(key) {
            return;
        }
        self.auto_measured_key = Some(key);

        #[derive(Debug)]
        struct NodeMeasureInput {
            node: GraphNodeId,
            title: Arc<str>,
            inputs: Vec<Arc<str>>,
            outputs: Vec<Arc<str>>,
        }

        let presenter: &dyn NodeGraphPresenter = &*self.presenter;
        let Some(nodes) = self
            .graph
            .read_ref(cx.app, |graph| {
                let mut out: Vec<NodeMeasureInput> = Vec::new();
                out.reserve(graph.nodes.len());

                for node_id in graph.nodes.keys().copied() {
                    let title = presenter.node_title(graph, node_id);
                    let (inputs, outputs) = node_ports(graph, node_id);
                    let inputs = inputs
                        .into_iter()
                        .map(|p| presenter.port_label(graph, p))
                        .collect();
                    let outputs = outputs
                        .into_iter()
                        .map(|p| presenter.port_label(graph, p))
                        .collect();
                    out.push(NodeMeasureInput {
                        node: node_id,
                        title,
                        inputs,
                        outputs,
                    });
                }

                out
            })
            .ok()
        else {
            return;
        };

        let text_style = self.style.context_menu_text_style.clone();
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let node_pad = self.style.node_padding;
        let pin_gap = 8.0;
        let pin_r = self.style.pin_radius;
        let label_overhead = 2.0 * node_pad + 2.0 * (pin_r + pin_gap);

        let mut measured: Vec<(GraphNodeId, (f32, f32))> = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let title_w = if node.title.is_empty() {
                0.0
            } else {
                self.paint_cache
                    .text_metrics(cx.services, node.title.clone(), &text_style, constraints)
                    .size
                    .width
                    .0
            };
            let max_in = node
                .inputs
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    self.paint_cache
                        .text_metrics(cx.services, s.clone(), &text_style, constraints)
                        .size
                        .width
                        .0
                })
                .fold(0.0, f32::max);
            let max_out = node
                .outputs
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    self.paint_cache
                        .text_metrics(cx.services, s.clone(), &text_style, constraints)
                        .size
                        .width
                        .0
                })
                .fold(0.0, f32::max);

            let w_by_title = title_w + 2.0 * node_pad;
            let w_by_labels = max_in.max(max_out) + label_overhead;
            let w = self.style.node_width.max(w_by_title).max(w_by_labels);

            let rows = node.inputs.len().max(node.outputs.len()) as f32;
            let base = self.style.node_header_height + 2.0 * node_pad;
            let h = base + rows * self.style.pin_row_height;

            measured.push((node.node, (w, h)));
        }

        let keep: std::collections::BTreeSet<GraphNodeId> =
            measured.iter().map(|(n, _)| *n).collect();

        let _ = self
            .auto_measured
            .update_if_changed(|node_sizes, _anchors| {
                let mut changed = false;

                node_sizes.retain(|id, _| {
                    let ok = keep.contains(id);
                    if !ok {
                        changed = true;
                    }
                    ok
                });

                for (node, size) in &measured {
                    let needs = match node_sizes.get(node) {
                        Some(old) => (old.0 - size.0).abs() > 0.1 || (old.1 - size.1).abs() > 0.1,
                        None => true,
                    };
                    if needs {
                        node_sizes.insert(*node, *size);
                        changed = true;
                    }
                }

                changed
            });
    }

    fn update_internals_store<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        bounds: Rect,
        geom: &CanvasGeometry,
    ) {
        let Some(store) = self.internals.as_ref() else {
            return;
        };

        let graph_rev = self.graph.revision(host).unwrap_or(0);
        let presenter_rev = self.presenter.geometry_revision();
        let key = InternalsCacheKey {
            graph_rev,
            zoom_bits: snapshot.zoom.to_bits(),
            draw_order_hash: Self::draw_order_hash(&snapshot.draw_order),
            presenter_rev,
            pan_x_bits: snapshot.pan.x.to_bits(),
            pan_y_bits: snapshot.pan.y.to_bits(),
            bounds_x_bits: bounds.origin.x.0.to_bits(),
            bounds_y_bits: bounds.origin.y.0.to_bits(),
        };

        if self.internals_key == Some(key) {
            return;
        }
        self.internals_key = Some(key);

        let transform = NodeGraphCanvasTransform {
            bounds_origin: bounds.origin,
            pan: snapshot.pan,
            zoom: snapshot.zoom,
        };

        let mut next = NodeGraphInternalsSnapshot {
            transform,
            ..NodeGraphInternalsSnapshot::default()
        };

        for (&node, node_geom) in &geom.nodes {
            next.nodes_window
                .insert(node, transform.canvas_rect_to_window(node_geom.rect));
        }
        for (&port, handle) in &geom.ports {
            next.ports_window
                .insert(port, transform.canvas_rect_to_window(handle.bounds));
            next.port_centers_window
                .insert(port, transform.canvas_point_to_window(handle.center));
        }

        next.focused_node = self.interaction.focused_node;
        next.focused_port = self.interaction.focused_port;
        next.focused_edge = self.interaction.focused_edge;
        next.connecting = self.interaction.wire_drag.is_some();

        let style = self.style.clone();
        let focused_node = self.interaction.focused_node;
        let focused_port = self.interaction.focused_port;
        let focused_edge = self.interaction.focused_edge;
        let labels = self
            .graph
            .read_ref(host, |graph| {
                let node_label = focused_node
                    .and_then(|node| self.presenter.a11y_node_label(graph, node))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_node.map(|node| format!("{:?}", node)));

                let port_label = focused_port
                    .and_then(|port| self.presenter.a11y_port_label(graph, port))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_port.map(|port| format!("{:?}", port)));

                let edge_label = focused_edge
                    .and_then(|edge| self.presenter.a11y_edge_label(graph, edge, &style))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_edge.map(|edge| format!("{:?}", edge)));

                (node_label, port_label, edge_label)
            })
            .ok()
            .unwrap_or_default();

        next.a11y_focused_node_label = labels.0.clone().map(|label| format!("Node {}", label));
        next.a11y_focused_port_label = labels.1.clone().map(|label| format!("Port {}", label));
        next.a11y_focused_edge_label = labels.2.clone().map(|label| format!("Edge {}", label));
        next.a11y_active_descendant_label = next
            .a11y_focused_port_label
            .clone()
            .or_else(|| next.a11y_focused_edge_label.clone())
            .or_else(|| next.a11y_focused_node_label.clone());

        store.update(next);
    }

    fn update_measured_output_store(&mut self, zoom: f32, geom: &CanvasGeometry) {
        let Some(store) = self.measured_output.as_ref() else {
            return;
        };
        let Some(key) = self.geometry.key else {
            return;
        };
        if self.measured_output_key == Some(key) {
            return;
        }
        self.measured_output_key = Some(key);

        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let quant = |v: f32| {
            (v / crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX).round()
                * crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX
        };

        let mut node_sizes: Vec<(GraphNodeId, (f32, f32))> = Vec::with_capacity(geom.nodes.len());
        for (&node, node_geom) in &geom.nodes {
            let w = quant(node_geom.rect.size.width.0 * zoom);
            let h = quant(node_geom.rect.size.height.0 * zoom);
            node_sizes.push((node, (w, h)));
        }

        let mut port_anchors: Vec<(PortId, PortAnchorHint)> = Vec::with_capacity(geom.ports.len());
        for (&port, handle) in &geom.ports {
            let Some(node_geom) = geom.nodes.get(&handle.node) else {
                continue;
            };
            let ox = node_geom.rect.origin.x.0;
            let oy = node_geom.rect.origin.y.0;

            let cx = quant((handle.center.x.0 - ox) * zoom);
            let cy = quant((handle.center.y.0 - oy) * zoom);
            let bx = quant((handle.bounds.origin.x.0 - ox) * zoom);
            let by = quant((handle.bounds.origin.y.0 - oy) * zoom);
            let bw = quant(handle.bounds.size.width.0 * zoom);
            let bh = quant(handle.bounds.size.height.0 * zoom);

            let center = Point::new(Px(cx), Px(cy));
            let bounds = Rect::new(Point::new(Px(bx), Px(by)), Size::new(Px(bw), Px(bh)));
            port_anchors.push((port, PortAnchorHint { center, bounds }));
        }

        let _ = store.apply_exclusive_batch_if_changed(
            crate::ui::measured::MeasuredGeometryExclusiveBatch {
                node_sizes_px: node_sizes,
                port_anchors_px: port_anchors,
            },
            crate::ui::measured::MeasuredGeometryApplyOptions::default(),
        );
    }

    fn update_view_state<H: UiHost>(
        &mut self,
        host: &mut H,
        f: impl FnOnce(&mut NodeGraphViewState),
    ) {
        let before = if self.callbacks.is_some() {
            if let Some(store) = self.store.as_ref() {
                store.read_ref(host, |s| s.view_state().clone()).ok()
            } else {
                self.view_state.read_ref(host, |s| s.clone()).ok()
            }
        } else {
            None
        };

        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let style = self.style.clone();
        if let Some(store) = self.store.as_ref() {
            let _ = store.update(host, |store, _cx| {
                store.update_view_state(|s| {
                    f(s);

                    let zoom = if s.zoom.is_finite() && s.zoom > 0.0 {
                        s.zoom.clamp(style.min_zoom, style.max_zoom)
                    } else {
                        1.0
                    };
                    s.zoom = zoom;

                    if let Some(extent) = s.interaction.translate_extent {
                        s.pan = Self::clamp_pan_to_translate_extent(s.pan, zoom, bounds, extent);
                    }
                });
            });
        } else {
            let _ = self.view_state.update(host, |s, _cx| {
                f(s);

                let zoom = if s.zoom.is_finite() && s.zoom > 0.0 {
                    s.zoom.clamp(style.min_zoom, style.max_zoom)
                } else {
                    1.0
                };
                s.zoom = zoom;

                if let Some(extent) = s.interaction.translate_extent {
                    s.pan = Self::clamp_pan_to_translate_extent(s.pan, zoom, bounds, extent);
                }
            });
        }
        self.sync_view_state(host);

        if let Some(before) = before {
            let after = self.view_state.read_ref(host, |s| s.clone()).ok();
            if let Some(after) = after {
                let mut changes: Vec<ViewChange> = Vec::new();
                if before.pan != after.pan || (before.zoom - after.zoom).abs() > 1.0e-6 {
                    changes.push(ViewChange::Viewport {
                        pan: after.pan,
                        zoom: after.zoom,
                    });
                }
                if before.selected_nodes != after.selected_nodes
                    || before.selected_edges != after.selected_edges
                    || before.selected_groups != after.selected_groups
                {
                    changes.push(ViewChange::Selection {
                        nodes: after.selected_nodes.clone(),
                        edges: after.selected_edges.clone(),
                        groups: after.selected_groups.clone(),
                    });
                }
                self.emit_view_callbacks(&changes);
            }
        }
    }

    fn clamp_pan_to_translate_extent(
        pan: CanvasPoint,
        zoom: f32,
        bounds: Rect,
        extent: crate::core::CanvasRect,
    ) -> CanvasPoint {
        if !zoom.is_finite() || zoom <= 0.0 {
            return pan;
        }
        if !bounds.size.width.0.is_finite()
            || !bounds.size.height.0.is_finite()
            || bounds.size.width.0 <= 0.0
            || bounds.size.height.0 <= 0.0
        {
            return pan;
        }
        let ew = extent.size.width;
        let eh = extent.size.height;
        if !ew.is_finite() || !eh.is_finite() || ew <= 0.0 || eh <= 0.0 {
            return pan;
        }

        let view_w = bounds.size.width.0 / zoom;
        let view_h = bounds.size.height.0 / zoom;

        let min_x = extent.origin.x;
        let min_y = extent.origin.y;
        let max_x = extent.origin.x + (extent.size.width - view_w).max(0.0);
        let max_y = extent.origin.y + (extent.size.height - view_h).max(0.0);

        let view_origin_x = (-pan.x).clamp(min_x, max_x);
        let view_origin_y = (-pan.y).clamp(min_y, max_y);

        CanvasPoint {
            x: -view_origin_x,
            y: -view_origin_y,
        }
    }

    fn apply_ops<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        ops: Vec<GraphOp>,
    ) {
        let _ = self.apply_ops_result(host, window, ops);
    }

    fn apply_ops_result<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        ops: Vec<GraphOp>,
    ) -> bool {
        self.commit_ops(host, window, None, ops)
    }

    fn start_sticky_wire_drag_from_port<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        from: PortId,
        pos: Point,
    ) {
        self.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from,
                bundle: Vec::new(),
            },
            pos,
        });
        self.interaction.sticky_wire = true;
        self.interaction.sticky_wire_ignore_next_up = true;
        self.interaction.hover_port = None;
        self.interaction.hover_port_valid = false;
        self.interaction.hover_port_convertible = false;
        cx.capture_pointer(cx.node);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
    }

    fn restore_suspended_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: Option<PortId>,
        fallback_pos: Point,
    ) {
        if let Some(wire_drag) = self.interaction.suspended_wire_drag.take() {
            self.interaction.wire_drag = Some(wire_drag);
            self.interaction.sticky_wire = true;
            self.interaction.sticky_wire_ignore_next_up = true;
            self.interaction.hover_port = None;
            self.interaction.hover_port_valid = false;
            self.interaction.hover_port_convertible = false;
            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return;
        }

        if let Some(from) = fallback_from {
            self.start_sticky_wire_drag_from_port(cx, from, fallback_pos);
        }
    }

    fn apply_transaction_result<H: UiHost>(
        &mut self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<GraphTransaction, Vec<Diagnostic>> {
        if let Some(store) = self.store.as_ref() {
            if let Some(profile) = self.presenter.profile_mut() {
                match store.update(host, |store, _cx| {
                    store.dispatch_transaction_with_profile(tx, profile)
                }) {
                    Ok(Ok(outcome)) => {
                        self.sync_view_state(host);
                        self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                        return Ok(outcome.committed);
                    }
                    Ok(Err(err)) => match &err {
                        ApplyPipelineError::Rejected {
                            diagnostics: diags, ..
                        } => return Err(diags.clone()),
                        _ => {
                            return Err(vec![Diagnostic {
                                key: "tx.apply_failed".to_string(),
                                severity: DiagnosticSeverity::Error,
                                target: crate::rules::DiagnosticTarget::Graph,
                                message: err.to_string(),
                                fixes: Vec::new(),
                            }]);
                        }
                    },
                    Err(_) => {
                        return Err(vec![Diagnostic {
                            key: "tx.apply_failed".to_string(),
                            severity: DiagnosticSeverity::Error,
                            target: crate::rules::DiagnosticTarget::Graph,
                            message: "store unavailable".to_string(),
                            fixes: Vec::new(),
                        }]);
                    }
                }
            }

            match store.update(host, |store, _cx| store.dispatch_transaction(tx)) {
                Ok(Ok(outcome)) => {
                    self.sync_view_state(host);
                    self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                    return Ok(outcome.committed);
                }
                Ok(Err(err)) => {
                    let message = match &err {
                        crate::runtime::store::DispatchError::Apply(err) => match err {
                            ApplyPipelineError::Rejected {
                                diagnostics: diags, ..
                            } => return Err(diags.clone()),
                            _ => err.to_string(),
                        },
                        crate::runtime::store::DispatchError::Changes(err) => err.to_string(),
                    };
                    return Err(vec![Diagnostic {
                        key: "tx.apply_failed".to_string(),
                        severity: DiagnosticSeverity::Error,
                        target: crate::rules::DiagnosticTarget::Graph,
                        message,
                        fixes: Vec::new(),
                    }]);
                }
                Err(_) => {
                    return Err(vec![Diagnostic {
                        key: "tx.apply_failed".to_string(),
                        severity: DiagnosticSeverity::Error,
                        target: crate::rules::DiagnosticTarget::Graph,
                        message: "store unavailable".to_string(),
                        fixes: Vec::new(),
                    }]);
                }
            }
        }

        let Some(mut scratch) = self.graph.read_ref(host, |g| g.clone()).ok() else {
            return Err(vec![Diagnostic {
                key: "tx.graph_unavailable".to_string(),
                severity: DiagnosticSeverity::Error,
                target: crate::rules::DiagnosticTarget::Graph,
                message: "graph unavailable".to_string(),
                fixes: Vec::new(),
            }]);
        };

        let committed = if let Some(profile) = self.presenter.profile_mut() {
            match apply_transaction_with_profile(&mut scratch, profile, tx) {
                Ok(committed) => committed,
                Err(err) => match &err {
                    ApplyPipelineError::Rejected {
                        diagnostics: diags, ..
                    } => return Err(diags.clone()),
                    _ => {
                        return Err(vec![Diagnostic {
                            key: "tx.apply_failed".to_string(),
                            severity: DiagnosticSeverity::Error,
                            target: crate::rules::DiagnosticTarget::Graph,
                            message: err.to_string(),
                            fixes: Vec::new(),
                        }]);
                    }
                },
            }
        } else {
            match apply_transaction(&mut scratch, tx) {
                Ok(()) => GraphTransaction {
                    label: tx.label.clone(),
                    ops: tx.ops.clone(),
                },
                Err(err) => {
                    return Err(vec![Diagnostic {
                        key: "tx.apply_failed".to_string(),
                        severity: DiagnosticSeverity::Error,
                        target: crate::rules::DiagnosticTarget::Graph,
                        message: err.to_string(),
                        fixes: Vec::new(),
                    }]);
                }
            }
        };

        let _ = self.graph.update(host, |g, _cx| {
            *g = scratch;
        });

        let changes = NodeGraphChanges::from_transaction(&committed);
        self.emit_graph_callbacks(&committed, &changes);
        Ok(committed)
    }

    fn commit_ops<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        label: Option<&str>,
        ops: Vec<GraphOp>,
    ) -> bool {
        if ops.is_empty() {
            return true;
        }

        let tx = GraphTransaction {
            label: label.map(|s| s.to_string()),
            ops,
        };
        self.commit_transaction(host, window, &tx)
    }

    fn commit_transaction<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        tx: &GraphTransaction,
    ) -> bool {
        match self.apply_transaction_result(host, tx) {
            Ok(committed) => {
                if self.store.is_none() {
                    self.history.record(committed);
                }
                true
            }
            Err(diags) => {
                if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                    self.show_toast(host, window, sev, msg);
                }
                false
            }
        }
    }

    fn undo_last<H: UiHost>(&mut self, host: &mut H, window: Option<AppWindowId>) -> bool {
        let preferred_focus = self.interaction.focused_edge;

        if let Some(store) = self.store.as_ref() {
            if let Some(profile) = self.presenter.profile_mut() {
                match store.update(host, |store, _cx| store.undo_with_profile(profile)) {
                    Ok(Ok(Some(outcome))) => {
                        self.sync_view_state(host);
                        self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                        self.update_view_state(host, |s| {
                            s.selected_edges.clear();
                            s.selected_nodes.clear();
                            s.selected_groups.clear();
                        });
                        self.repair_focused_edge_after_graph_change(host, preferred_focus);
                        return true;
                    }
                    Ok(Ok(None)) => return false,
                    Ok(Err(err)) => {
                        self.show_toast(
                            host,
                            window,
                            DiagnosticSeverity::Error,
                            Arc::<str>::from(err.to_string()),
                        );
                        return false;
                    }
                    Err(_) => {
                        self.show_toast(
                            host,
                            window,
                            DiagnosticSeverity::Error,
                            Arc::<str>::from("store unavailable"),
                        );
                        return false;
                    }
                }
            }

            match store.update(host, |store, _cx| store.undo()) {
                Ok(Ok(Some(outcome))) => {
                    self.sync_view_state(host);
                    self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                    self.update_view_state(host, |s| {
                        s.selected_edges.clear();
                        s.selected_nodes.clear();
                        s.selected_groups.clear();
                    });
                    self.repair_focused_edge_after_graph_change(host, preferred_focus);
                    return true;
                }
                Ok(Ok(None)) => return false,
                Ok(Err(err)) => {
                    self.show_toast(
                        host,
                        window,
                        DiagnosticSeverity::Error,
                        Arc::<str>::from(err.to_string()),
                    );
                    return false;
                }
                Err(_) => {
                    self.show_toast(
                        host,
                        window,
                        DiagnosticSeverity::Error,
                        Arc::<str>::from("store unavailable"),
                    );
                    return false;
                }
            }
        }

        let mut history = std::mem::take(&mut self.history);
        let result = history.undo(|tx| self.apply_transaction_result(host, tx));
        self.history = history;

        match result {
            Ok(true) => {
                self.update_view_state(host, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                    s.selected_groups.clear();
                });
                self.repair_focused_edge_after_graph_change(host, preferred_focus);
                true
            }
            Ok(false) => false,
            Err(diags) => {
                if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                    self.show_toast(host, window, sev, msg);
                }
                false
            }
        }
    }

    fn redo_last<H: UiHost>(&mut self, host: &mut H, window: Option<AppWindowId>) -> bool {
        let preferred_focus = self.interaction.focused_edge;

        if let Some(store) = self.store.as_ref() {
            if let Some(profile) = self.presenter.profile_mut() {
                match store.update(host, |store, _cx| store.redo_with_profile(profile)) {
                    Ok(Ok(Some(outcome))) => {
                        self.sync_view_state(host);
                        self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                        self.update_view_state(host, |s| {
                            s.selected_edges.clear();
                            s.selected_nodes.clear();
                            s.selected_groups.clear();
                        });
                        self.repair_focused_edge_after_graph_change(host, preferred_focus);
                        return true;
                    }
                    Ok(Ok(None)) => return false,
                    Ok(Err(err)) => {
                        self.show_toast(
                            host,
                            window,
                            DiagnosticSeverity::Error,
                            Arc::<str>::from(err.to_string()),
                        );
                        return false;
                    }
                    Err(_) => {
                        self.show_toast(
                            host,
                            window,
                            DiagnosticSeverity::Error,
                            Arc::<str>::from("store unavailable"),
                        );
                        return false;
                    }
                }
            }

            match store.update(host, |store, _cx| store.redo()) {
                Ok(Ok(Some(outcome))) => {
                    self.sync_view_state(host);
                    self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                    self.update_view_state(host, |s| {
                        s.selected_edges.clear();
                        s.selected_nodes.clear();
                        s.selected_groups.clear();
                    });
                    self.repair_focused_edge_after_graph_change(host, preferred_focus);
                    return true;
                }
                Ok(Ok(None)) => return false,
                Ok(Err(err)) => {
                    self.show_toast(
                        host,
                        window,
                        DiagnosticSeverity::Error,
                        Arc::<str>::from(err.to_string()),
                    );
                    return false;
                }
                Err(_) => {
                    self.show_toast(
                        host,
                        window,
                        DiagnosticSeverity::Error,
                        Arc::<str>::from("store unavailable"),
                    );
                    return false;
                }
            }
        }

        let mut history = std::mem::take(&mut self.history);
        let result = history.redo(|tx| self.apply_transaction_result(host, tx));
        self.history = history;

        match result {
            Ok(true) => {
                self.update_view_state(host, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                    s.selected_groups.clear();
                });
                self.repair_focused_edge_after_graph_change(host, preferred_focus);
                true
            }
            Ok(false) => false,
            Err(diags) => {
                if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                    self.show_toast(host, window, sev, msg);
                }
                false
            }
        }
    }

    fn screen_to_canvas(bounds: Rect, screen: Point, pan: CanvasPoint, zoom: f32) -> CanvasPoint {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let sx = screen.x.0 - bounds.origin.x.0;
        let sy = screen.y.0 - bounds.origin.y.0;
        CanvasPoint {
            x: sx / zoom - pan.x,
            y: sy / zoom - pan.y,
        }
    }

    fn ensure_canvas_point_visible<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        point: CanvasPoint,
    ) {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
            return;
        }

        let margin_screen = 24.0f32;
        let margin = margin_screen / zoom;
        if !margin.is_finite() {
            return;
        }

        let view_w = bounds.size.width.0 / zoom;
        let view_h = bounds.size.height.0 / zoom;

        let view_min_x = -snapshot.pan.x;
        let view_min_y = -snapshot.pan.y;

        let mut pan = snapshot.pan;

        let min_x = view_min_x + margin;
        let max_x = view_min_x + view_w - margin;
        if point.x < min_x {
            pan.x = margin - point.x;
        } else if point.x > max_x {
            pan.x = (view_w - margin) - point.x;
        }

        let min_y = view_min_y + margin;
        let max_y = view_min_y + view_h - margin;
        if point.y < min_y {
            pan.y = margin - point.y;
        } else if point.y > max_y {
            pan.y = (view_h - margin) - point.y;
        }

        if pan != snapshot.pan {
            self.update_view_state(host, |s| {
                s.pan = pan;
            });
        }
    }

    fn next_paste_canvas_point(&mut self, bounds: Rect, snapshot: &ViewSnapshot) -> CanvasPoint {
        let zoom = if snapshot.zoom.is_finite() && snapshot.zoom > 0.0 {
            snapshot.zoom
        } else {
            1.0
        };

        let anchor = self.interaction.last_canvas_pos.unwrap_or_else(|| {
            let cx0 = bounds.origin.x.0 + 0.5 * bounds.size.width.0;
            let cy0 = bounds.origin.y.0 + 0.5 * bounds.size.height.0;
            let center = Point::new(Px(cx0), Px(cy0));
            Self::screen_to_canvas(bounds, center, snapshot.pan, zoom)
        });

        let (series, at) = PasteSeries::next(self.interaction.paste_series, anchor, zoom);
        self.interaction.paste_series = Some(series);
        at
    }

    fn copy_selection_to_clipboard<H: UiHost>(
        &mut self,
        host: &mut H,
        selected_nodes: &[GraphNodeId],
        selected_groups: &[crate::core::GroupId],
    ) {
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }
        let text = self
            .graph
            .read_ref(host, |graph| {
                let fragment = GraphFragment::from_selection(
                    graph,
                    selected_nodes.to_vec(),
                    selected_groups.to_vec(),
                );
                fragment.to_clipboard_text().unwrap_or_default()
            })
            .ok()
            .unwrap_or_default();
        if text.is_empty() {
            return;
        }
        host.push_effect(Effect::ClipboardSetText { text });
    }

    fn request_paste_at_canvas<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        at: CanvasPoint,
    ) {
        let Some(window) = window else {
            return;
        };

        let token = host.next_clipboard_token();
        self.interaction.pending_paste = Some(PendingPaste { token, at });
        host.push_effect(Effect::ClipboardGetText { window, token });
    }

    fn apply_paste_text<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        text: &str,
        at: CanvasPoint,
    ) {
        let fragment: GraphFragment = match GraphFragment::from_clipboard_text(text) {
            Ok(v) => v,
            Err(_) => {
                self.show_toast(
                    host,
                    window,
                    DiagnosticSeverity::Info,
                    "clipboard does not contain a fret-node fragment",
                );
                return;
            }
        };

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        for node in fragment.nodes.values() {
            min_x = min_x.min(node.pos.x);
            min_y = min_y.min(node.pos.y);
        }
        for group in fragment.groups.values() {
            min_x = min_x.min(group.rect.origin.x);
            min_y = min_y.min(group.rect.origin.y);
        }
        if !min_x.is_finite() || !min_y.is_finite() {
            return;
        }

        let tuning = PasteTuning {
            offset: CanvasPoint {
                x: at.x - min_x,
                y: at.y - min_y,
            },
        };
        let remapper = IdRemapper::new(IdRemapSeed::new_random());
        let tx = fragment.to_paste_transaction(&remapper, tuning);

        let new_nodes: Vec<GraphNodeId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddNode { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        let new_groups: Vec<crate::core::GroupId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddGroup { id, .. } => Some(*id),
                _ => None,
            })
            .collect();

        if !self.apply_ops_result(host, window, tx.ops) {
            return;
        }

        if !new_nodes.is_empty() || !new_groups.is_empty() {
            self.update_view_state(host, |s| {
                s.selected_edges.clear();
                s.selected_nodes = new_nodes.clone();
                s.selected_groups = new_groups.clone();
                for id in &new_nodes {
                    s.draw_order.retain(|x| x != id);
                    s.draw_order.push(*id);
                }
                for id in &new_groups {
                    s.group_draw_order.retain(|x| x != id);
                    s.group_draw_order.push(*id);
                }
            });
        }
    }

    fn duplicate_selection<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        selected_nodes: &[GraphNodeId],
        selected_groups: &[crate::core::GroupId],
    ) {
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }

        let fragment = self
            .graph
            .read_ref(host, |graph| {
                GraphFragment::from_selection(
                    graph,
                    selected_nodes.to_vec(),
                    selected_groups.to_vec(),
                )
            })
            .ok()
            .unwrap_or_default();

        let tuning = PasteTuning {
            offset: CanvasPoint { x: 24.0, y: 24.0 },
        };
        let remapper = IdRemapper::new(IdRemapSeed::new_random());
        let mut tx = fragment.to_paste_transaction(&remapper, tuning);
        tx.label = Some("Duplicate".to_string());

        let new_nodes: Vec<GraphNodeId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddNode { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        let new_groups: Vec<crate::core::GroupId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddGroup { id, .. } => Some(*id),
                _ => None,
            })
            .collect();

        if !self.commit_transaction(host, window, &tx) {
            return;
        }

        if !new_nodes.is_empty() || !new_groups.is_empty() {
            self.update_view_state(host, |s| {
                s.selected_edges.clear();
                s.selected_nodes = new_nodes.clone();
                s.selected_groups = new_groups.clone();
                for id in &new_nodes {
                    s.draw_order.retain(|x| x != id);
                    s.draw_order.push(*id);
                }
                for id in &new_groups {
                    s.group_draw_order.retain(|x| x != id);
                    s.group_draw_order.push(*id);
                }
            });
        }
    }

    fn delete_selection_ops(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        selected_nodes: &[GraphNodeId],
        selected_edges: &[EdgeId],
        selected_groups: &[crate::core::GroupId],
    ) -> Vec<GraphOp> {
        let mut ops: Vec<GraphOp> = Vec::new();
        let mut removed_edges: std::collections::BTreeSet<EdgeId> =
            std::collections::BTreeSet::new();

        let mut groups: Vec<crate::core::GroupId> = selected_groups.to_vec();
        groups.sort();
        for group_id in groups {
            if let Some(op) = graph.build_remove_group_op(group_id) {
                ops.push(op);
            }
        }

        let mut nodes: Vec<GraphNodeId> = selected_nodes.to_vec();
        nodes.sort();

        for node_id in nodes {
            if !Self::node_is_deletable(graph, interaction, node_id) {
                continue;
            }
            let Some(node) = graph.nodes.get(&node_id) else {
                continue;
            };

            let mut ports: Vec<(PortId, crate::core::Port)> = graph
                .ports
                .iter()
                .filter_map(|(port_id, port)| {
                    (port.node == node_id).then_some((*port_id, port.clone()))
                })
                .collect();
            ports.sort_by_key(|(id, _)| *id);

            let port_ids: std::collections::BTreeSet<PortId> =
                ports.iter().map(|(id, _)| *id).collect();

            let mut edges: Vec<(EdgeId, Edge)> = graph
                .edges
                .iter()
                .filter_map(|(edge_id, edge)| {
                    if port_ids.contains(&edge.from) || port_ids.contains(&edge.to) {
                        Some((*edge_id, edge.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            edges.sort_by_key(|(id, _)| *id);
            edges.retain(|(id, _)| removed_edges.insert(*id));

            ops.push(GraphOp::RemoveNode {
                id: node_id,
                node: node.clone(),
                ports,
                edges,
            });
        }

        let mut edges_sel: Vec<EdgeId> = selected_edges.to_vec();
        edges_sel.sort();
        for edge_id in edges_sel {
            if removed_edges.contains(&edge_id) {
                continue;
            }
            if !Self::edge_is_deletable(graph, interaction, edge_id) {
                continue;
            }
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            ops.push(GraphOp::RemoveEdge {
                id: edge_id,
                edge: edge.clone(),
            });
        }

        ops
    }

    fn removed_ids_from_ops(
        ops: &[GraphOp],
    ) -> (
        HashSet<GraphNodeId>,
        HashSet<EdgeId>,
        HashSet<crate::core::GroupId>,
    ) {
        let mut removed_nodes: HashSet<GraphNodeId> = HashSet::new();
        let mut removed_edges: HashSet<EdgeId> = HashSet::new();
        let mut removed_groups: HashSet<crate::core::GroupId> = HashSet::new();

        for op in ops {
            match op {
                GraphOp::RemoveNode { id, edges, .. } => {
                    removed_nodes.insert(*id);
                    for (edge_id, _) in edges {
                        removed_edges.insert(*edge_id);
                    }
                }
                GraphOp::RemoveEdge { id, .. } => {
                    removed_edges.insert(*id);
                }
                GraphOp::RemoveGroup { id, .. } => {
                    removed_groups.insert(*id);
                }
                _ => {}
            }
        }

        (removed_nodes, removed_edges, removed_groups)
    }

    fn nudge_selection_by_screen_delta<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        delta_screen_px: CanvasPoint,
    ) {
        let selected_nodes = snapshot.selected_nodes.clone();
        let selected_groups = snapshot.selected_groups.clone();
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }

        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }

        let mut delta = CanvasPoint {
            x: delta_screen_px.x / zoom,
            y: delta_screen_px.y / zoom,
        };
        if !delta.x.is_finite() || !delta.y.is_finite() {
            return;
        }

        if snapshot.interaction.snap_to_grid {
            if let Some(primary) = selected_nodes.first().copied() {
                let primary_start = self
                    .graph
                    .read_ref(host, |g| g.nodes.get(&primary).map(|n| n.pos))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let primary_target = CanvasPoint {
                    x: primary_start.x + delta.x,
                    y: primary_start.y + delta.y,
                };
                let snapped =
                    Self::snap_canvas_point(primary_target, snapshot.interaction.snap_grid);
                delta = CanvasPoint {
                    x: snapped.x - primary_start.x,
                    y: snapped.y - primary_start.y,
                };
            } else if let Some(primary) = selected_groups.first().copied() {
                let primary_start = self
                    .graph
                    .read_ref(host, |g| g.groups.get(&primary).map(|gr| gr.rect.origin))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let primary_target = CanvasPoint {
                    x: primary_start.x + delta.x,
                    y: primary_start.y + delta.y,
                };
                let snapped =
                    Self::snap_canvas_point(primary_target, snapshot.interaction.snap_grid);
                delta = CanvasPoint {
                    x: snapped.x - primary_start.x,
                    y: snapped.y - primary_start.y,
                };
            }
        }

        if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
            return;
        }

        let geom_for_extent = self.canvas_geometry(&*host, snapshot);
        let ops = self
            .graph
            .read_ref(host, |g| {
                let mut ops: Vec<GraphOp> = Vec::new();

                let selected_groups_set: std::collections::HashSet<crate::core::GroupId> =
                    selected_groups.iter().copied().collect();

                let mut moved_by_group: std::collections::HashSet<GraphNodeId> =
                    std::collections::HashSet::new();
                for (&node_id, node) in &g.nodes {
                    if let Some(parent) = node.parent
                        && selected_groups_set.contains(&parent)
                    {
                        moved_by_group.insert(node_id);
                    }
                }

                let mut moved_nodes: std::collections::BTreeSet<GraphNodeId> =
                    selected_nodes.iter().copied().collect();
                for id in &moved_by_group {
                    moved_nodes.insert(*id);
                }

                let mut groups_sorted = selected_groups.clone();
                groups_sorted.sort();
                for group_id in groups_sorted {
                    let Some(group) = g.groups.get(&group_id) else {
                        continue;
                    };
                    let from = group.rect;
                    let to = crate::core::CanvasRect {
                        origin: CanvasPoint {
                            x: from.origin.x + delta.x,
                            y: from.origin.y + delta.y,
                        },
                        size: from.size,
                    };
                    if from != to {
                        ops.push(GraphOp::SetGroupRect {
                            id: group_id,
                            from,
                            to,
                        });
                    }
                }

                for node_id in moved_nodes {
                    let Some(node) = g.nodes.get(&node_id) else {
                        continue;
                    };
                    let from = node.pos;
                    let mut to = CanvasPoint {
                        x: from.x + delta.x,
                        y: from.y + delta.y,
                    };

                    if !moved_by_group.contains(&node_id) {
                        let Some(node_geom) = geom_for_extent.nodes.get(&node_id) else {
                            continue;
                        };
                        let node_w = node_geom.rect.size.width.0;
                        let node_h = node_geom.rect.size.height.0;

                        if let Some(extent) = snapshot.interaction.node_extent {
                            let min_x = extent.origin.x;
                            let min_y = extent.origin.y;
                            let max_x = extent.origin.x + (extent.size.width - node_w).max(0.0);
                            let max_y = extent.origin.y + (extent.size.height - node_h).max(0.0);
                            to.x = to.x.clamp(min_x, max_x);
                            to.y = to.y.clamp(min_y, max_y);
                        }

                        if let Some(parent) = node.parent
                            && let Some(group) = g.groups.get(&parent)
                        {
                            let min_x = group.rect.origin.x;
                            let min_y = group.rect.origin.y;
                            let max_x =
                                group.rect.origin.x + (group.rect.size.width - node_w).max(0.0);
                            let max_y =
                                group.rect.origin.y + (group.rect.size.height - node_h).max(0.0);
                            to.x = to.x.clamp(min_x, max_x);
                            to.y = to.y.clamp(min_y, max_y);
                        }
                    }

                    if from != to {
                        ops.push(GraphOp::SetNodePos {
                            id: node_id,
                            from,
                            to,
                        });
                    }
                }

                ops
            })
            .ok()
            .unwrap_or_default();

        if ops.is_empty() {
            return;
        }

        let _ = self.commit_ops(host, window, Some("Nudge"), ops);
    }

    fn align_or_distribute_selection<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        mode: AlignDistributeMode,
    ) {
        let selected_nodes = snapshot.selected_nodes.clone();
        let selected_groups = snapshot.selected_groups.clone();
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }

        let geom = self.canvas_geometry(&*host, snapshot);

        let ops = self
            .graph
            .read_ref(host, |g| {
                #[derive(Clone, Copy)]
                enum ElementId {
                    Node(GraphNodeId),
                    Group(crate::core::GroupId),
                }

                #[derive(Clone, Copy)]
                struct Elem {
                    id: ElementId,
                    x: f32,
                    y: f32,
                    w: f32,
                    h: f32,
                }

                let selected_groups_set: std::collections::HashSet<crate::core::GroupId> =
                    selected_groups.iter().copied().collect();

                let mut moved_by_group: std::collections::HashSet<GraphNodeId> =
                    std::collections::HashSet::new();
                for (&node_id, node) in &g.nodes {
                    if let Some(parent) = node.parent
                        && selected_groups_set.contains(&parent)
                    {
                        moved_by_group.insert(node_id);
                    }
                }

                let mut elems: Vec<Elem> = Vec::new();
                for node_id in &selected_nodes {
                    let Some(node_geom) = geom.nodes.get(node_id) else {
                        continue;
                    };
                    elems.push(Elem {
                        id: ElementId::Node(*node_id),
                        x: node_geom.rect.origin.x.0,
                        y: node_geom.rect.origin.y.0,
                        w: node_geom.rect.size.width.0,
                        h: node_geom.rect.size.height.0,
                    });
                }
                for group_id in &selected_groups {
                    let Some(group) = g.groups.get(group_id) else {
                        continue;
                    };
                    elems.push(Elem {
                        id: ElementId::Group(*group_id),
                        x: group.rect.origin.x,
                        y: group.rect.origin.y,
                        w: group.rect.size.width,
                        h: group.rect.size.height,
                    });
                }

                if elems.len() < 2 {
                    return Vec::new();
                }

                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;
                for e in &elems {
                    min_x = min_x.min(e.x);
                    min_y = min_y.min(e.y);
                    max_x = max_x.max(e.x + e.w);
                    max_y = max_y.max(e.y + e.h);
                }
                if !min_x.is_finite()
                    || !min_y.is_finite()
                    || !max_x.is_finite()
                    || !max_y.is_finite()
                {
                    return Vec::new();
                }

                let target_left = min_x;
                let target_top = min_y;
                let target_right = max_x;
                let target_bottom = max_y;
                let target_center_x = 0.5 * (min_x + max_x);
                let target_center_y = 0.5 * (min_y + max_y);

                let mut ops: Vec<GraphOp> = Vec::new();

                let mut per_group_delta: std::collections::HashMap<
                    crate::core::GroupId,
                    CanvasPoint,
                > = std::collections::HashMap::new();
                let mut per_node_delta: std::collections::HashMap<GraphNodeId, CanvasPoint> =
                    std::collections::HashMap::new();

                match mode {
                    AlignDistributeMode::AlignLeft => {
                        for e in &elems {
                            let dx = target_left - e.x;
                            if dx.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignRight => {
                        for e in &elems {
                            let new_left = target_right - e.w;
                            let dx = new_left - e.x;
                            if dx.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignTop => {
                        for e in &elems {
                            let dy = target_top - e.y;
                            if dy.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignBottom => {
                        for e in &elems {
                            let new_top = target_bottom - e.h;
                            let dy = new_top - e.y;
                            if dy.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignCenterX => {
                        for e in &elems {
                            let cur = e.x + 0.5 * e.w;
                            let dx = target_center_x - cur;
                            if dx.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignCenterY => {
                        for e in &elems {
                            let cur = e.y + 0.5 * e.h;
                            let dy = target_center_y - cur;
                            if dy.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::DistributeX => {
                        if elems.len() < 3 {
                            return Vec::new();
                        }
                        let mut sorted = elems;
                        sorted.sort_by(|a, b| {
                            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        let first = sorted.first().copied().unwrap();
                        let last = sorted.last().copied().unwrap();
                        let c0 = first.x + 0.5 * first.w;
                        let c1 = last.x + 0.5 * last.w;
                        let span = c1 - c0;
                        if !span.is_finite() || span.abs() <= 1.0e-6 {
                            return Vec::new();
                        }
                        let step = span / (sorted.len() as f32 - 1.0);
                        for (ix, e) in sorted.iter().enumerate().skip(1).take(sorted.len() - 2) {
                            let desired = c0 + (ix as f32) * step;
                            let cur = e.x + 0.5 * e.w;
                            let dx = desired - cur;
                            if dx.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::DistributeY => {
                        if elems.len() < 3 {
                            return Vec::new();
                        }
                        let mut sorted = elems;
                        sorted.sort_by(|a, b| {
                            a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        let first = sorted.first().copied().unwrap();
                        let last = sorted.last().copied().unwrap();
                        let c0 = first.y + 0.5 * first.h;
                        let c1 = last.y + 0.5 * last.h;
                        let span = c1 - c0;
                        if !span.is_finite() || span.abs() <= 1.0e-6 {
                            return Vec::new();
                        }
                        let step = span / (sorted.len() as f32 - 1.0);
                        for (ix, e) in sorted.iter().enumerate().skip(1).take(sorted.len() - 2) {
                            let desired = c0 + (ix as f32) * step;
                            let cur = e.y + 0.5 * e.h;
                            let dy = desired - cur;
                            if dy.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                            }
                        }
                    }
                }

                // Apply group deltas first (and move their child nodes).
                let mut groups_sorted = selected_groups.clone();
                groups_sorted.sort();
                for group_id in groups_sorted {
                    let Some(delta) = per_group_delta.get(&group_id).copied() else {
                        continue;
                    };
                    let Some(group) = g.groups.get(&group_id) else {
                        continue;
                    };
                    let from = group.rect;
                    let to = crate::core::CanvasRect {
                        origin: CanvasPoint {
                            x: from.origin.x + delta.x,
                            y: from.origin.y + delta.y,
                        },
                        size: from.size,
                    };
                    if from != to {
                        ops.push(GraphOp::SetGroupRect {
                            id: group_id,
                            from,
                            to,
                        });
                    }

                    for (&node_id, node) in &g.nodes {
                        if node.parent != Some(group_id) {
                            continue;
                        }
                        let from = node.pos;
                        let to = CanvasPoint {
                            x: from.x + delta.x,
                            y: from.y + delta.y,
                        };
                        if from != to {
                            ops.push(GraphOp::SetNodePos {
                                id: node_id,
                                from,
                                to,
                            });
                        }
                    }
                }

                // Apply node deltas for nodes not moved by a selected group.
                let mut nodes_sorted = selected_nodes.clone();
                nodes_sorted.sort();
                for node_id in nodes_sorted {
                    if moved_by_group.contains(&node_id) {
                        continue;
                    }
                    let Some(delta) = per_node_delta.get(&node_id).copied() else {
                        continue;
                    };
                    let Some(node) = g.nodes.get(&node_id) else {
                        continue;
                    };
                    let from = node.pos;
                    let mut to = CanvasPoint {
                        x: from.x + delta.x,
                        y: from.y + delta.y,
                    };

                    // Reuse the same extent constraints as drag/nudge.
                    if let Some(node_geom) = geom.nodes.get(&node_id) {
                        let node_w = node_geom.rect.size.width.0;
                        let node_h = node_geom.rect.size.height.0;

                        if let Some(extent) = snapshot.interaction.node_extent {
                            let min_x = extent.origin.x;
                            let min_y = extent.origin.y;
                            let max_x = extent.origin.x + (extent.size.width - node_w).max(0.0);
                            let max_y = extent.origin.y + (extent.size.height - node_h).max(0.0);
                            to.x = to.x.clamp(min_x, max_x);
                            to.y = to.y.clamp(min_y, max_y);
                        }

                        if let Some(parent) = node.parent
                            && let Some(group) = g.groups.get(&parent)
                        {
                            let min_x = group.rect.origin.x;
                            let min_y = group.rect.origin.y;
                            let max_x =
                                group.rect.origin.x + (group.rect.size.width - node_w).max(0.0);
                            let max_y =
                                group.rect.origin.y + (group.rect.size.height - node_h).max(0.0);
                            to.x = to.x.clamp(min_x, max_x);
                            to.y = to.y.clamp(min_y, max_y);
                        }
                    }

                    if from != to {
                        ops.push(GraphOp::SetNodePos {
                            id: node_id,
                            from,
                            to,
                        });
                    }
                }

                ops
            })
            .ok()
            .unwrap_or_default();

        if ops.is_empty() {
            return;
        }

        let label = match mode {
            AlignDistributeMode::AlignLeft => "Align Left",
            AlignDistributeMode::AlignRight => "Align Right",
            AlignDistributeMode::AlignTop => "Align Top",
            AlignDistributeMode::AlignBottom => "Align Bottom",
            AlignDistributeMode::AlignCenterX => "Align Center X",
            AlignDistributeMode::AlignCenterY => "Align Center Y",
            AlignDistributeMode::DistributeX => "Distribute X",
            AlignDistributeMode::DistributeY => "Distribute Y",
        };
        let _ = self.commit_ops(host, window, Some(label), ops);
    }

    pub(super) fn snap_canvas_point(pos: CanvasPoint, grid: CanvasSize) -> CanvasPoint {
        fn snap_axis(value: f32, grid: f32) -> f32 {
            if !value.is_finite() {
                return value;
            }
            if !grid.is_finite() || grid <= 0.0 {
                return value;
            }
            (value / grid).round() * grid
        }

        CanvasPoint {
            x: snap_axis(pos.x, grid.width),
            y: snap_axis(pos.y, grid.height),
        }
    }

    pub(super) fn auto_pan_delta(snapshot: &ViewSnapshot, pos: Point, bounds: Rect) -> CanvasPoint {
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return CanvasPoint::default();
        }

        let margin_screen = snapshot.interaction.auto_pan.margin;
        let speed_screen_per_s = snapshot.interaction.auto_pan.speed;
        if !margin_screen.is_finite() || margin_screen <= 0.0 {
            return CanvasPoint::default();
        }
        if !speed_screen_per_s.is_finite() || speed_screen_per_s <= 0.0 {
            return CanvasPoint::default();
        }

        let viewport_w = bounds.size.width.0;
        let viewport_h = bounds.size.height.0;
        if !viewport_w.is_finite()
            || viewport_w <= 0.0
            || !viewport_h.is_finite()
            || viewport_h <= 0.0
        {
            return CanvasPoint::default();
        }

        let pan = snapshot.pan;
        let pos_screen_x = (pos.x.0 + pan.x) * zoom;
        let pos_screen_y = (pos.y.0 + pan.y) * zoom;

        let dist_left = pos_screen_x;
        let dist_right = viewport_w - pos_screen_x;
        let dist_top = pos_screen_y;
        let dist_bottom = viewport_h - pos_screen_y;

        let step_screen = speed_screen_per_s / Self::AUTO_PAN_TICK_HZ;
        let step_graph = step_screen / zoom;

        let mut delta_x = 0.0;
        let mut delta_y = 0.0;

        if dist_left.is_finite() && dist_left < margin_screen {
            let factor = ((margin_screen - dist_left) / margin_screen).clamp(0.0, 1.0);
            delta_x += step_graph * factor;
        }
        if dist_right.is_finite() && dist_right < margin_screen {
            let factor = ((margin_screen - dist_right) / margin_screen).clamp(0.0, 1.0);
            delta_x -= step_graph * factor;
        }
        if dist_top.is_finite() && dist_top < margin_screen {
            let factor = ((margin_screen - dist_top) / margin_screen).clamp(0.0, 1.0);
            delta_y += step_graph * factor;
        }
        if dist_bottom.is_finite() && dist_bottom < margin_screen {
            let factor = ((margin_screen - dist_bottom) / margin_screen).clamp(0.0, 1.0);
            delta_y -= step_graph * factor;
        }

        if !delta_x.is_finite() || !delta_y.is_finite() {
            return CanvasPoint::default();
        }

        CanvasPoint {
            x: delta_x,
            y: delta_y,
        }
    }

    fn wire_drag_suppresses_edge(kind: &WireDragKind, edge_id: EdgeId) -> bool {
        match kind {
            WireDragKind::Reconnect { edge, .. } => *edge == edge_id,
            WireDragKind::ReconnectMany { edges } => {
                edges.iter().any(|(edge, ..)| *edge == edge_id)
            }
            _ => false,
        }
    }

    // NOTE: Node bounds and port anchors must come from derived geometry (`CanvasGeometry`),
    // not ad-hoc layout guesses. See ADR 0135.

    fn rect_contains_point(rect: Rect, pos: Point) -> bool {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        pos.x.0 >= min_x && pos.x.0 <= max_x && pos.y.0 >= min_y && pos.y.0 <= max_y
    }

    fn distance_sq_point_to_rect(pos: Point, rect: Rect) -> f32 {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);

        let dx = if pos.x.0 < min_x {
            min_x - pos.x.0
        } else if pos.x.0 > max_x {
            pos.x.0 - max_x
        } else {
            0.0
        };
        let dy = if pos.y.0 < min_y {
            min_y - pos.y.0
        } else if pos.y.0 > max_y {
            pos.y.0 - max_y
        } else {
            0.0
        };

        dx * dx + dy * dy
    }

    fn hit_port(
        &self,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<PortId>,
    ) -> Option<PortId> {
        let r = self.style.pin_radius / zoom;
        if !r.is_finite() || r <= 0.0 {
            return None;
        }

        index.query_ports(pos, r, scratch);
        scratch.sort_unstable();
        scratch.dedup();

        let mut best: Option<(PortId, u32)> = None;
        for &port_id in scratch.iter() {
            let Some(handle) = geom.ports.get(&port_id) else {
                continue;
            };
            if !Self::rect_contains_point(handle.bounds, pos) {
                continue;
            }
            let rank = geom.node_rank.get(&handle.node).copied().unwrap_or(0);
            match best {
                Some((best_id, best_rank)) => {
                    if rank > best_rank || (rank == best_rank && port_id < best_id) {
                        best = Some((port_id, rank));
                    }
                }
                None => best = Some((port_id, rank)),
            }
        }

        best.map(|(id, _)| id)
    }

    fn pick_target_port(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        from: PortId,
        require_from_connectable_start: bool,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<PortId>,
    ) -> Option<PortId> {
        if require_from_connectable_start
            && !Self::port_is_connectable_start(graph, &snapshot.interaction, from)
        {
            return None;
        }

        let from_port = graph.ports.get(&from)?;
        let desired_dir = match from_port.dir {
            PortDirection::In => PortDirection::Out,
            PortDirection::Out => PortDirection::In,
        };

        match snapshot.interaction.connection_mode {
            NodeGraphConnectionMode::Strict => {
                let candidate = self.hit_port(geom, index, pos, zoom, scratch)?;
                let port = graph.ports.get(&candidate)?;
                (candidate != from
                    && port.dir == desired_dir
                    && Self::port_is_connectable_end(graph, &snapshot.interaction, candidate))
                .then_some(candidate)
            }
            NodeGraphConnectionMode::Loose => {
                let radius_screen = snapshot.interaction.connection_radius;
                if !radius_screen.is_finite() || radius_screen <= 0.0 {
                    let candidate = self.hit_port(geom, index, pos, zoom, scratch)?;
                    return (candidate != from
                        && Self::port_is_connectable_end(graph, &snapshot.interaction, candidate))
                    .then_some(candidate);
                }
                let r = radius_screen / zoom;
                let r2 = r * r;
                let eps = (1.0e-3 / zoom.max(1.0e-6)).max(1.0e-6);

                let mut best: Option<(PortId, f32, bool, u32)> = None;
                index.query_ports(pos, r, scratch);
                scratch.sort_unstable();
                scratch.dedup();
                for &port_id in scratch.iter() {
                    if port_id == from {
                        continue;
                    }
                    let Some(handle) = geom.ports.get(&port_id) else {
                        continue;
                    };
                    if !Self::port_is_connectable_end(graph, &snapshot.interaction, port_id) {
                        continue;
                    }
                    let d2 = Self::distance_sq_point_to_rect(pos, handle.bounds);
                    if d2 > r2 {
                        continue;
                    }
                    let prefers_opposite = handle.dir == desired_dir;
                    let rank = geom.node_rank.get(&handle.node).copied().unwrap_or(0);
                    match best {
                        Some((best_id, best_d2, best_prefers_opposite, best_rank)) => {
                            if d2 + eps < best_d2 {
                                best = Some((port_id, d2, prefers_opposite, rank));
                            } else if (d2 - best_d2).abs() <= eps {
                                if prefers_opposite != best_prefers_opposite {
                                    if prefers_opposite {
                                        best = Some((port_id, d2, prefers_opposite, rank));
                                    }
                                } else if rank > best_rank {
                                    best = Some((port_id, d2, prefers_opposite, rank));
                                } else if rank == best_rank && port_id < best_id {
                                    best = Some((port_id, d2, prefers_opposite, rank));
                                }
                            }
                        }
                        None => best = Some((port_id, d2, prefers_opposite, rank)),
                    }
                }

                best.map(|(id, _, _, _)| id)
            }
        }
    }

    fn hit_edge(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<EdgeId>,
    ) -> Option<EdgeId> {
        let hit_w =
            (snapshot.interaction.edge_interaction_width / zoom).max(self.style.wire_width / zoom);
        let threshold2 = hit_w * hit_w;

        index.query_edges(pos, hit_w, scratch);
        scratch.sort_unstable();
        scratch.dedup();

        let mut best: Option<(EdgeId, f32)> = None;
        for &edge_id in scratch.iter() {
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };

            let route = self.edge_render_hint(graph, edge_id).route;
            let d2 = match route {
                EdgeRouteKind::Bezier => wire_distance2(pos, from, to, zoom),
                EdgeRouteKind::Straight => dist2_point_to_segment(pos, from, to),
                EdgeRouteKind::Step => step_wire_distance2(pos, from, to),
            };
            if d2 <= threshold2 {
                match best {
                    Some((_id, best_d2)) if best_d2 <= d2 => {}
                    _ => best = Some((edge_id, d2)),
                }
            }
        }

        best.map(|(id, _)| id)
    }

    fn hit_edge_focus_anchor(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<EdgeId>,
    ) -> Option<(EdgeId, EdgeEndpoint, PortId)> {
        let z = zoom.max(1.0e-6);
        let half =
            (0.5 * Self::EDGE_FOCUS_ANCHOR_SIZE_SCREEN + Self::EDGE_FOCUS_ANCHOR_PAD_SCREEN) / z;
        let query_r = (half * 1.5).max(half);
        index.query_edges(pos, query_r, scratch);
        scratch.sort_unstable();
        scratch.dedup();

        let mut best: Option<(EdgeId, EdgeEndpoint, PortId, f32)> = None;

        for &edge_id in scratch.iter() {
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            let (allow_source, allow_target) =
                Self::edge_reconnectable_flags(edge, &snapshot.interaction);
            if !allow_source && !allow_target {
                continue;
            }
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };

            let route = self.edge_render_hint(graph, edge_id).route;
            let (a0, a1) = Self::edge_focus_anchor_centers(route, from, to, zoom);
            let r0 = Self::edge_focus_anchor_rect(a0, zoom);
            let r1 = Self::edge_focus_anchor_rect(a1, zoom);

            let mut consider =
                |center: Point, rect: Rect, endpoint: EdgeEndpoint, fixed: PortId| {
                    if !rect.contains(pos) {
                        return;
                    }
                    let dx = pos.x.0 - center.x.0;
                    let dy = pos.y.0 - center.y.0;
                    let d2 = dx * dx + dy * dy;
                    match best {
                        Some((_id, _ep, _fixed, best_d2)) if best_d2 <= d2 => {}
                        _ => best = Some((edge_id, endpoint, fixed, d2)),
                    }
                };

            if allow_source {
                consider(a0, r0, EdgeEndpoint::From, edge.to);
            }
            if allow_target {
                consider(a1, r1, EdgeEndpoint::To, edge.from);
            }
        }

        best.map(|(id, endpoint, fixed, _)| (id, endpoint, fixed))
    }

    fn clamp_context_menu_origin(
        &self,
        desired: Point,
        item_count: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let rect = context_menu_rect_at(&self.style, desired, item_count, snapshot.zoom);

        let viewport_w = bounds.size.width.0 / snapshot.zoom;
        let viewport_h = bounds.size.height.0 / snapshot.zoom;
        let viewport_origin_x = -snapshot.pan.x;
        let viewport_origin_y = -snapshot.pan.y;

        let min_x = viewport_origin_x;
        let min_y = viewport_origin_y;
        let max_x = viewport_origin_x + (viewport_w - rect.size.width.0).max(0.0);
        let max_y = viewport_origin_y + (viewport_h - rect.size.height.0).max(0.0);

        Point::new(
            Px(desired.x.0.clamp(min_x, max_x)),
            Px(desired.y.0.clamp(min_y, max_y)),
        )
    }

    fn clamp_searcher_origin(
        &self,
        desired: Point,
        visible_rows: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let rect = searcher_rect_at(&self.style, desired, visible_rows, snapshot.zoom);

        let viewport_w = bounds.size.width.0 / snapshot.zoom;
        let viewport_h = bounds.size.height.0 / snapshot.zoom;
        let viewport_origin_x = -snapshot.pan.x;
        let viewport_origin_y = -snapshot.pan.y;

        let min_x = viewport_origin_x;
        let min_y = viewport_origin_y;
        let max_x = viewport_origin_x + (viewport_w - rect.size.width.0).max(0.0);
        let max_y = viewport_origin_y + (viewport_h - rect.size.height.0).max(0.0);

        Point::new(
            Px(desired.x.0.clamp(min_x, max_x)),
            Px(desired.y.0.clamp(min_y, max_y)),
        )
    }

    fn node_default_size_for_ports(&self, inputs: usize, outputs: usize) -> (f32, f32) {
        let rows = inputs.max(outputs) as f32;
        let base = self.style.node_header_height + 2.0 * self.style.node_padding;
        let pin_area = rows * self.style.pin_row_height;
        (self.style.node_width, base + pin_area)
    }

    fn frame_nodes_in_view<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        bounds: Rect,
        node_ids: &[GraphNodeId],
    ) -> bool {
        if node_ids.is_empty() {
            self.show_toast(
                host,
                window,
                DiagnosticSeverity::Info,
                "no selection to frame",
            );
            return false;
        }

        #[derive(Debug, Clone, Copy)]
        struct NodeInfo {
            pos: CanvasPoint,
            w: f32,
            h: f32,
        }

        let infos: Vec<NodeInfo> = self
            .graph
            .read_ref(host, |graph| {
                let mut out: Vec<NodeInfo> = Vec::new();
                for id in node_ids {
                    let Some(node) = graph.nodes.get(id) else {
                        continue;
                    };
                    let (inputs, outputs) = node_ports(graph, *id);
                    let (w, h) = self.node_default_size_for_ports(inputs.len(), outputs.len());
                    out.push(NodeInfo {
                        pos: node.pos,
                        w,
                        h,
                    });
                }
                out
            })
            .ok()
            .unwrap_or_default();

        if infos.is_empty() {
            self.show_toast(
                host,
                window,
                DiagnosticSeverity::Info,
                "no selection to frame",
            );
            return false;
        }

        let viewport_w = bounds.size.width.0;
        let viewport_h = bounds.size.height.0;
        if !viewport_w.is_finite()
            || !viewport_h.is_finite()
            || viewport_w <= 1.0
            || viewport_h <= 1.0
        {
            return false;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;
        for n in &infos {
            min_x = min_x.min(n.pos.x);
            min_y = min_y.min(n.pos.y);
            max_x = max_x.max(n.pos.x);
            max_y = max_y.max(n.pos.y);
            max_w = max_w.max(n.w);
            max_h = max_h.max(n.h);
        }

        let spread_x = (max_x - min_x).max(0.0);
        let spread_y = (max_y - min_y).max(0.0);

        let margin = 48.0f32;
        let mut zoom_x = self.style.max_zoom;
        let mut zoom_y = self.style.max_zoom;
        if spread_x > 1.0e-3 {
            zoom_x = (viewport_w - max_w - 2.0 * margin) / spread_x;
        }
        if spread_y > 1.0e-3 {
            zoom_y = (viewport_h - max_h - 2.0 * margin) / spread_y;
        }

        let mut zoom = zoom_x.min(zoom_y);
        if !zoom.is_finite() {
            zoom = 1.0;
        }
        zoom = zoom.clamp(self.style.min_zoom, self.style.max_zoom);

        let mut rect_min_x = f32::INFINITY;
        let mut rect_min_y = f32::INFINITY;
        let mut rect_max_x = f32::NEG_INFINITY;
        let mut rect_max_y = f32::NEG_INFINITY;
        for n in &infos {
            let w = n.w / zoom;
            let h = n.h / zoom;
            rect_min_x = rect_min_x.min(n.pos.x);
            rect_min_y = rect_min_y.min(n.pos.y);
            rect_max_x = rect_max_x.max(n.pos.x + w);
            rect_max_y = rect_max_y.max(n.pos.y + h);
        }

        if !rect_min_x.is_finite()
            || !rect_min_y.is_finite()
            || !rect_max_x.is_finite()
            || !rect_max_y.is_finite()
        {
            return false;
        }

        let center_x = 0.5 * (rect_min_x + rect_max_x);
        let center_y = 0.5 * (rect_min_y + rect_max_y);

        let viewport_w_canvas = viewport_w / zoom;
        let viewport_h_canvas = viewport_h / zoom;
        let target_center_x = 0.5 * viewport_w_canvas;
        let target_center_y = 0.5 * viewport_h_canvas;

        let new_pan = CanvasPoint {
            x: target_center_x - center_x,
            y: target_center_y - center_y,
        };

        self.update_view_state(host, |s| {
            s.zoom = zoom;
            s.pan = new_pan;
        });

        true
    }

    fn reroute_pos_for_invoked_at(&self, invoked_at: Point) -> CanvasPoint {
        let (w, h) = self.node_default_size_for_ports(Self::REROUTE_INPUTS, Self::REROUTE_OUTPUTS);
        CanvasPoint {
            x: invoked_at.x.0 - 0.5 * w,
            y: invoked_at.y.0 - 0.5 * h,
        }
    }

    fn first_added_node_id(ops: &[GraphOp]) -> Option<GraphNodeId> {
        for op in ops {
            if let GraphOp::AddNode { id, .. } = op {
                return Some(*id);
            }
        }
        None
    }

    fn build_reroute_create_ops(at: CanvasPoint) -> Vec<GraphOp> {
        let node_id = GraphNodeId::new();
        let in_port_id = PortId::new();
        let out_port_id = PortId::new();

        let node = crate::core::Node {
            kind: NodeKindKey::new(REROUTE_KIND),
            kind_version: 1,
            pos: at,
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            size: None,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        };

        let in_port = crate::core::Port {
            node: node_id,
            key: crate::core::PortKey::new("in"),
            dir: PortDirection::In,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        };

        let out_port = crate::core::Port {
            node: node_id,
            key: crate::core::PortKey::new("out"),
            dir: PortDirection::Out,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        };

        vec![
            GraphOp::AddNode { id: node_id, node },
            GraphOp::AddPort {
                id: in_port_id,
                port: in_port,
            },
            GraphOp::AddPort {
                id: out_port_id,
                port: out_port,
            },
            GraphOp::SetNodePorts {
                id: node_id,
                from: Vec::new(),
                to: vec![in_port_id, out_port_id],
            },
        ]
    }

    fn create_group_at<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        at: CanvasPoint,
    ) {
        let size = crate::core::CanvasSize {
            width: 480.0,
            height: 320.0,
        };
        let origin = crate::core::CanvasPoint {
            x: at.x - 0.5 * size.width,
            y: at.y - 0.5 * size.height,
        };
        let group = crate::core::Group {
            title: "Group".to_string(),
            rect: crate::core::CanvasRect { origin, size },
            color: None,
        };
        let group_id = crate::core::GroupId::new();
        let ops = vec![GraphOp::AddGroup {
            id: group_id,
            group,
        }];
        if self.commit_ops(host, window, Some("Create Group"), ops) {
            self.update_view_state(host, |s| {
                s.selected_nodes.clear();
                s.selected_edges.clear();
                s.selected_groups.clear();
                s.selected_groups.push(group_id);
                s.group_draw_order.retain(|id| *id != group_id);
                s.group_draw_order.push(group_id);
            });
        }
    }

    fn record_recent_kind(&mut self, kind: &NodeKindKey) {
        const MAX_RECENT: usize = 20;

        self.interaction.recent_kinds.retain(|k| k != kind);
        self.interaction.recent_kinds.insert(0, kind.clone());
        if self.interaction.recent_kinds.len() > MAX_RECENT {
            self.interaction.recent_kinds.truncate(MAX_RECENT);
        }
    }

    fn searcher_is_selectable_row(row: &SearcherRow) -> bool {
        matches!(row.kind, SearcherRowKind::Candidate { .. }) && row.enabled
    }

    fn searcher_first_selectable_row(rows: &[SearcherRow]) -> usize {
        rows.iter()
            .position(Self::searcher_is_selectable_row)
            .unwrap_or(0)
    }

    fn rebuild_searcher_rows(searcher: &mut SearcherState) {
        let rows = match &searcher.target {
            ContextMenuTarget::ConnectionConvertPicker { .. } => {
                super::searcher::build_rows_flat(&searcher.candidates, &searcher.query)
            }
            _ => super::searcher::build_rows(
                &searcher.candidates,
                &searcher.query,
                &searcher.recent_kinds,
            ),
        };

        searcher.rows = rows;
        searcher.scroll = searcher.scroll.min(
            searcher
                .rows
                .len()
                .saturating_sub(SEARCHER_MAX_VISIBLE_ROWS),
        );
        searcher.active_row = Self::searcher_first_selectable_row(&searcher.rows)
            .min(searcher.rows.len().saturating_sub(1));
        Self::ensure_searcher_active_visible(searcher);
    }

    fn ensure_searcher_active_visible(searcher: &mut SearcherState) {
        let n = searcher.rows.len();
        if n == 0 {
            searcher.active_row = 0;
            searcher.scroll = 0;
            return;
        }

        let visible = SEARCHER_MAX_VISIBLE_ROWS.min(n);
        let max_scroll = n.saturating_sub(visible);
        searcher.scroll = searcher.scroll.min(max_scroll);

        if searcher.active_row < searcher.scroll {
            searcher.scroll = searcher.active_row;
        } else if searcher.active_row >= searcher.scroll + visible {
            searcher.scroll = (searcher.active_row + 1).saturating_sub(visible);
        }
        searcher.scroll = searcher.scroll.min(max_scroll);
    }

    fn try_activate_searcher_row<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        row_ix: usize,
    ) -> bool {
        let Some(searcher) = self.interaction.searcher.take() else {
            return false;
        };

        let Some(row) = searcher.rows.get(row_ix).cloned() else {
            self.interaction.searcher = Some(searcher);
            return false;
        };

        let SearcherRowKind::Candidate { candidate_ix } = row.kind else {
            self.interaction.searcher = Some(searcher);
            return false;
        };
        if !row.enabled {
            self.interaction.searcher = Some(searcher);
            return false;
        }

        let item = NodeGraphContextMenuItem {
            label: row.label,
            enabled: true,
            action: NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
        };
        self.activate_context_menu_item(
            cx,
            &searcher.target,
            searcher.invoked_at,
            item,
            &searcher.candidates,
        );
        true
    }

    fn open_insert_node_picker<H: UiHost>(&mut self, host: &mut H, at: CanvasPoint) {
        let candidates: Vec<InsertNodeCandidate> = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| presenter.list_insertable_nodes(graph))
                .ok()
                .unwrap_or_default()
        };

        let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
        menu_candidates.push(InsertNodeCandidate {
            kind: NodeKindKey::new(REROUTE_KIND),
            label: Arc::<str>::from("Reroute"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        });
        menu_candidates.extend(candidates);

        let snapshot = self.sync_view_state(host);
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let rows =
            super::searcher::build_rows(&menu_candidates, "", &self.interaction.recent_kinds);
        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
        let origin =
            self.clamp_searcher_origin(Point::new(Px(at.x), Px(at.y)), visible, bounds, &snapshot);
        let active_row = rows
            .iter()
            .position(|r| matches!(r.kind, SearcherRowKind::Candidate { .. }) && r.enabled)
            .unwrap_or(0);

        self.interaction.context_menu = None;
        self.interaction.searcher = Some(SearcherState {
            origin,
            invoked_at: Point::new(Px(at.x), Px(at.y)),
            target: ContextMenuTarget::BackgroundInsertNodePicker { at },
            query: String::new(),
            candidates: menu_candidates,
            recent_kinds: self.interaction.recent_kinds.clone(),
            rows,
            hovered_row: None,
            active_row,
            scroll: 0,
        });
    }

    fn open_connection_insert_node_picker<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        at: CanvasPoint,
    ) {
        let candidates: Vec<InsertNodeCandidate> = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| {
                    presenter.list_insertable_nodes_for_connection(graph, from)
                })
                .ok()
                .unwrap_or_default()
        };

        let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
        menu_candidates.push(InsertNodeCandidate {
            kind: NodeKindKey::new(REROUTE_KIND),
            label: Arc::<str>::from("Reroute"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        });
        menu_candidates.extend(candidates);

        let snapshot = self.sync_view_state(host);
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let rows =
            super::searcher::build_rows(&menu_candidates, "", &self.interaction.recent_kinds);
        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
        let origin =
            self.clamp_searcher_origin(Point::new(Px(at.x), Px(at.y)), visible, bounds, &snapshot);
        let active_row = rows
            .iter()
            .position(|r| matches!(r.kind, SearcherRowKind::Candidate { .. }) && r.enabled)
            .unwrap_or(0);

        self.interaction.context_menu = None;
        self.interaction.searcher = Some(SearcherState {
            origin,
            invoked_at: Point::new(Px(at.x), Px(at.y)),
            target: ContextMenuTarget::ConnectionInsertNodePicker { from, at },
            query: String::new(),
            candidates: menu_candidates,
            recent_kinds: self.interaction.recent_kinds.clone(),
            rows,
            hovered_row: None,
            active_row,
            scroll: 0,
        });
    }

    fn open_edge_insert_node_picker<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        edge: EdgeId,
        invoked_at: Point,
    ) {
        edge_insert::open_edge_insert_node_picker(self, host, window, edge, invoked_at);
    }

    fn activate_context_menu_item<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        menu_candidates: &[InsertNodeCandidate],
    ) {
        match (target, item.action) {
            (_, NodeGraphContextMenuAction::Command(command)) => {
                self.interaction.context_menu = None;
                if let ContextMenuTarget::Group(group_id) = target {
                    let group_id = *group_id;
                    self.update_view_state(cx.app, |s| {
                        s.selected_nodes.clear();
                        s.selected_edges.clear();
                        if !s.selected_groups.iter().any(|id| *id == group_id) {
                            s.selected_groups.clear();
                            s.selected_groups.push(group_id);
                        }
                    });
                }
                cx.dispatch_command(command);
            }
            (
                ContextMenuTarget::BackgroundInsertNodePicker { at },
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);

                let outcome = if candidate.kind.0 == REROUTE_KIND {
                    Some(Ok(Self::build_reroute_create_ops(*at)))
                } else {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            presenter.plan_create_node(graph, &candidate, *at)
                        })
                        .ok()
                };

                match outcome {
                    Some(Ok(ops)) => {
                        let node_id = Self::first_added_node_id(&ops);
                        if self.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                            if let Some(node_id) = node_id {
                                self.update_view_state(cx.app, |s| {
                                    s.selected_edges.clear();
                                    s.selected_groups.clear();
                                    s.selected_nodes.clear();
                                    s.selected_nodes.push(node_id);
                                    s.draw_order.retain(|id| *id != node_id);
                                    s.draw_order.push(node_id);
                                });
                            }
                        }
                    }
                    Some(Err(msg)) => {
                        self.show_toast(cx.app, cx.window, DiagnosticSeverity::Info, msg)
                    }
                    None => {}
                }
            }
            (
                ContextMenuTarget::ConnectionInsertNodePicker { from, at },
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                let mode = self.sync_view_state(cx.app).interaction.connection_mode;

                enum Outcome {
                    Apply(Vec<GraphOp>, Option<GraphNodeId>, Option<PortId>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                }

                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);

                let (outcome, toast) = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let insert_ops = if candidate.kind.0 == REROUTE_KIND {
                                Ok(Self::build_reroute_create_ops(*at))
                            } else {
                                presenter.plan_create_node(graph, &candidate, *at)
                            };

                            let insert_ops = match insert_ops {
                                Ok(ops) => ops,
                                Err(msg) => {
                                    return (Outcome::Reject(DiagnosticSeverity::Info, msg), None);
                                }
                            };

                            let planned = workflow::plan_wire_drop_insert(
                                presenter, graph, *from, mode, insert_ops,
                            );
                            let toast = planned.toast.clone();
                            (
                                Outcome::Apply(
                                    planned.ops,
                                    planned.created_node,
                                    planned.continue_from,
                                ),
                                toast,
                            )
                        })
                        .ok()
                        .unwrap_or((Outcome::Ignore, None))
                };

                match outcome {
                    Outcome::Apply(ops, created_node, continue_from) => {
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        if self.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                            if let Some(node_id) = created_node {
                                self.update_view_state(cx.app, |s| {
                                    s.selected_edges.clear();
                                    s.selected_groups.clear();
                                    s.selected_nodes.clear();
                                    s.selected_nodes.push(node_id);
                                    s.draw_order.retain(|id| *id != node_id);
                                    s.draw_order.push(node_id);
                                });
                            }
                            if let Some((sev, msg)) = toast {
                                self.show_toast(cx.app, cx.window, sev, msg);
                            }

                            if let Some(port) = continue_from {
                                self.interaction.suspended_wire_drag = None;
                                self.start_sticky_wire_drag_from_port(cx, port, resume_pos);
                            } else {
                                self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                            }
                        } else {
                            self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                        }
                    }
                    Outcome::Reject(sev, msg) => {
                        self.show_toast(cx.app, cx.window, sev, msg);
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                    }
                    Outcome::Ignore => {
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                    }
                }
            }
            (
                ContextMenuTarget::Edge(edge_id),
                NodeGraphContextMenuAction::OpenInsertNodePicker,
            ) => {
                edge_insert::open_edge_insert_context_menu(self, cx, *edge_id, invoked_at);
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::InsertReroute) => {
                let at = self.reroute_pos_for_invoked_at(invoked_at);
                let kind = NodeKindKey::new(REROUTE_KIND);

                let outcome = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let plan = presenter.plan_split_edge(graph, *edge_id, &kind, at);
                            match plan.decision {
                                ConnectDecision::Accept => Ok(plan.ops),
                                ConnectDecision::Reject => Err(plan.diagnostics),
                            }
                        })
                        .ok()
                };

                match outcome {
                    Some(Ok(ops)) => {
                        let node_id = Self::first_added_node_id(&ops);
                        self.apply_ops(cx.app, cx.window, ops);
                        if let Some(node_id) = node_id {
                            self.update_view_state(cx.app, |s| {
                                s.selected_edges.clear();
                                s.selected_groups.clear();
                                s.selected_nodes.clear();
                                s.selected_nodes.push(node_id);
                                s.draw_order.retain(|id| *id != node_id);
                                s.draw_order.push(node_id);
                            });
                        }
                    }
                    Some(Err(diags)) => {
                        let (sev, msg) =
                            Self::toast_from_diagnostics(&diags).unwrap_or_else(|| {
                                (
                                    DiagnosticSeverity::Error,
                                    Arc::<str>::from("failed to insert reroute"),
                                )
                            });
                        self.show_toast(cx.app, cx.window, sev, msg);
                    }
                    None => {}
                }
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::DeleteEdge) => {
                let remove_ops = {
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            graph
                                .edges
                                .get(edge_id)
                                .map(|edge| {
                                    vec![GraphOp::RemoveEdge {
                                        id: *edge_id,
                                        edge: edge.clone(),
                                    }]
                                })
                                .unwrap_or_default()
                        })
                        .ok()
                        .unwrap_or_default()
                };

                self.apply_ops(cx.app, cx.window, remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.retain(|id| *id != *edge_id);
                });
            }
            (
                ContextMenuTarget::EdgeInsertNodePicker(edge_id),
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                edge_insert::insert_node_on_edge(self, cx, *edge_id, invoked_at, candidate);
            }
            (
                ContextMenuTarget::ConnectionConvertPicker { from, to, at },
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                enum Outcome {
                    Apply(Vec<GraphOp>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                }

                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);

                let zoom = self.cached_zoom;
                let style = self.style.clone();

                let outcome = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let template = match &candidate.template {
                                Some(t) => t,
                                None => {
                                    return Outcome::Reject(
                                        DiagnosticSeverity::Error,
                                        Arc::<str>::from(
                                            "conversion candidate is missing template",
                                        ),
                                    );
                                }
                            };

                            let plan = conversion::plan_insert_conversion(
                                presenter, graph, &style, zoom, *from, *to, *at, template,
                            );
                            match plan.decision {
                                ConnectDecision::Accept => Outcome::Apply(plan.ops),
                                ConnectDecision::Reject => {
                                    Self::toast_from_diagnostics(&plan.diagnostics)
                                        .map(|(sev, msg)| Outcome::Reject(sev, msg))
                                        .unwrap_or(Outcome::Ignore)
                                }
                            }
                        })
                        .ok()
                        .unwrap_or(Outcome::Ignore)
                };

                match outcome {
                    Outcome::Apply(ops) => {
                        let node_id = Self::first_added_node_id(&ops);
                        self.apply_ops(cx.app, cx.window, ops);
                        self.interaction.suspended_wire_drag = None;
                        if let Some(node_id) = node_id {
                            self.update_view_state(cx.app, |s| {
                                s.selected_edges.clear();
                                s.selected_groups.clear();
                                s.selected_nodes.clear();
                                s.selected_nodes.push(node_id);
                                s.draw_order.retain(|id| *id != node_id);
                                s.draw_order.push(node_id);
                            });
                        }
                    }
                    Outcome::Reject(sev, msg) => {
                        self.show_toast(cx.app, cx.window, sev, msg);
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                    }
                    Outcome::Ignore => {
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                    }
                }
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::Custom(action_id)) => {
                let ops = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            presenter.on_edge_context_menu_action(graph, *edge_id, action_id)
                        })
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                };

                if !ops.is_empty() {
                    self.apply_ops(cx.app, cx.window, ops);
                }
            }
            _ => {}
        }
    }

    fn paint_context_menu<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        menu: &ContextMenuState,
        zoom: f32,
    ) {
        let rect = context_menu_rect_at(&self.style, menu.origin, menu.items.len(), zoom);
        let border_w = Px(1.0 / zoom);
        let radius = Px(self.style.context_menu_corner_radius / zoom);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(50),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(border_w),
            border_color: self.style.context_menu_border,
            corner_radii: Corners::all(radius),
        });

        let pad = self.style.context_menu_padding / zoom;
        let item_h = self.style.context_menu_item_height / zoom;
        let inner_x = rect.origin.x.0 + pad;
        let inner_y = rect.origin.y.0 + pad;
        let inner_w = (rect.size.width.0 - 2.0 * pad).max(0.0);

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let constraints = TextConstraints {
            max_width: Some(Px(inner_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor * zoom,
        };

        for (ix, item) in menu.items.iter().enumerate() {
            let item_rect = Rect::new(
                Point::new(Px(inner_x), Px(inner_y + ix as f32 * item_h)),
                Size::new(Px(inner_w), Px(item_h)),
            );

            let is_active = menu.active_item == ix;
            let is_hovered = menu.hovered_item == Some(ix);
            if (is_hovered || is_active) && item.enabled {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(51),
                    rect: item_rect,
                    background: self.style.context_menu_hover_background,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(4.0 / zoom)),
                });
            }

            let (blob, metrics) = self.paint_cache.text_blob(
                cx.services,
                item.label.clone(),
                &text_style,
                constraints,
            );

            let text_x = item_rect.origin.x;
            let inner_y =
                item_rect.origin.y.0 + (item_rect.size.height.0 - metrics.size.height.0) * 0.5;
            let text_y = Px(inner_y + metrics.baseline.0);
            let color = if item.enabled {
                self.style.context_menu_text
            } else {
                self.style.context_menu_text_disabled
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(52),
                origin: Point::new(text_x, text_y),
                text: blob,
                color,
            });
        }
    }

    fn paint_marquee<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        marquee: &MarqueeDrag,
        zoom: f32,
    ) {
        let rect = rect_from_points(marquee.start_pos, marquee.pos);
        let border_w = Px(self.style.marquee_border_width / zoom);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(49),
            rect,
            background: self.style.marquee_fill,
            border: Edges::all(border_w),
            border_color: self.style.marquee_border,
            corner_radii: Corners::all(Px(0.0)),
        });
    }

    fn paint_snap_guides<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        guides: &SnapGuides,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) {
        let w = Px((self.style.snapline_width / zoom).max(0.5 / zoom));
        let half = 0.5 * w.0;

        if let Some(x) = guides.x {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(48),
                rect: Rect::new(
                    Point::new(Px(x - half), Px(viewport_origin_y)),
                    Size::new(w, Px(viewport_h)),
                ),
                background: self.style.snapline_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        if let Some(y) = guides.y {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(48),
                rect: Rect::new(
                    Point::new(Px(viewport_origin_x), Px(y - half)),
                    Size::new(Px(viewport_w), w),
                ),
                background: self.style.snapline_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }
    }

    fn paint_searcher<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        searcher: &SearcherState,
        zoom: f32,
    ) {
        let visible_rows = searcher_visible_rows(searcher);
        let rect = searcher_rect_at(&self.style, searcher.origin, visible_rows, zoom);
        let border_w = Px(1.0 / zoom);
        let radius = Px(self.style.context_menu_corner_radius / zoom);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(55),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(border_w),
            border_color: self.style.context_menu_border,
            corner_radii: Corners::all(radius),
        });

        let pad = self.style.context_menu_padding / zoom;
        let item_h = self.style.context_menu_item_height / zoom;
        let inner_x = rect.origin.x.0 + pad;
        let inner_y = rect.origin.y.0 + pad;
        let inner_w = (rect.size.width.0 - 2.0 * pad).max(0.0);

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let constraints = TextConstraints {
            max_width: Some(Px(inner_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor * zoom,
        };

        let query_rect = Rect::new(
            Point::new(Px(inner_x), Px(inner_y)),
            Size::new(Px(inner_w), Px(item_h)),
        );
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(56),
            rect: query_rect,
            background: self.style.context_menu_hover_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(4.0 / zoom)),
        });

        let query_text = if searcher.query.is_empty() {
            Arc::<str>::from("Search...")
        } else {
            Arc::<str>::from(format!("Search: {}", searcher.query))
        };
        let (blob, metrics) =
            self.paint_cache
                .text_blob(cx.services, query_text, &text_style, constraints);
        let text_x = query_rect.origin.x;
        let text_y = Px(query_rect.origin.y.0
            + (query_rect.size.height.0 - metrics.size.height.0) * 0.5
            + metrics.baseline.0);
        let query_color = if searcher.query.is_empty() {
            self.style.context_menu_text_disabled
        } else {
            self.style.context_menu_text
        };
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(57),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: query_color,
        });

        let list_y0 = inner_y + item_h + pad;
        let start = searcher.scroll.min(searcher.rows.len());
        let end = (start + visible_rows).min(searcher.rows.len());
        for (slot, row_ix) in (start..end).enumerate() {
            let row = &searcher.rows[row_ix];
            let item_rect = Rect::new(
                Point::new(Px(inner_x), Px(list_y0 + slot as f32 * item_h)),
                Size::new(Px(inner_w), Px(item_h)),
            );

            let is_active = searcher.active_row == row_ix;
            let is_hovered = searcher.hovered_row == Some(row_ix);
            if (is_hovered || is_active) && Self::searcher_is_selectable_row(row) {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(56),
                    rect: item_rect,
                    background: self.style.context_menu_hover_background,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(4.0 / zoom)),
                });
            }

            let (blob, metrics) = self.paint_cache.text_blob(
                cx.services,
                row.label.clone(),
                &text_style,
                constraints,
            );

            let text_x = item_rect.origin.x;
            let text_y = Px(item_rect.origin.y.0
                + (item_rect.size.height.0 - metrics.size.height.0) * 0.5
                + metrics.baseline.0);
            let color = if row.enabled {
                self.style.context_menu_text
            } else {
                self.style.context_menu_text_disabled
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(57),
                origin: Point::new(text_x, text_y),
                text: blob,
                color,
            });
        }
    }

    fn paint_toast<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        toast: &ToastState,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_h: f32,
    ) {
        let margin = 12.0 / zoom;
        let pad = 10.0 / zoom;
        let max_w = 420.0 / zoom;

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let constraints = TextConstraints {
            max_width: Some(Px(max_w - 2.0 * pad)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor * zoom,
        };

        let (blob, metrics) = self.paint_cache.text_blob(
            cx.services,
            toast.message.clone(),
            &text_style,
            constraints,
        );

        let box_w = (metrics.size.width.0 + 2.0 * pad).clamp(120.0 / zoom, max_w);
        let box_h = metrics.size.height.0 + 2.0 * pad;

        let x = viewport_origin_x + margin;
        let y = viewport_origin_y + viewport_h - box_h - margin;
        let rect = Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(box_w), Px(box_h)));

        let border_color = match toast.severity {
            DiagnosticSeverity::Info => Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 1.0,
            },
            DiagnosticSeverity::Warning => Color {
                r: 0.95,
                g: 0.75,
                b: 0.20,
                a: 1.0,
            },
            DiagnosticSeverity::Error => Color {
                r: 0.90,
                g: 0.35,
                b: 0.35,
                a: 1.0,
            },
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(70),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(Px(1.0 / zoom)),
            border_color,
            corner_radii: Corners::all(Px(6.0 / zoom)),
        });

        let text_x = Px(rect.origin.x.0 + pad);
        let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(71),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: self.style.context_menu_text,
        });
    }

    fn paint_wire_drag_hint<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        wire_drag: &WireDrag,
        zoom: f32,
    ) {
        let text = match &wire_drag.kind {
            WireDragKind::New { bundle, .. } if bundle.len() > 1 => {
                Arc::<str>::from(format!("Bundle: {}", bundle.len()))
            }
            WireDragKind::ReconnectMany { edges } if edges.len() > 1 => {
                Arc::<str>::from(format!("Yank: {}", edges.len()))
            }
            _ => return,
        };

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let pad = 8.0 / zoom;
        let max_w = 220.0 / zoom;
        let constraints = TextConstraints {
            max_width: Some(Px(max_w - 2.0 * pad)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor * zoom,
        };

        let (blob, metrics) =
            self.paint_cache
                .text_blob(cx.services, text, &text_style, constraints);

        let box_w = (metrics.size.width.0 + 2.0 * pad).clamp(72.0 / zoom, max_w);
        let box_h = metrics.size.height.0 + 2.0 * pad;

        let offset_x = 14.0 / zoom;
        let offset_y = 12.0 / zoom;
        let rect = Rect::new(
            Point::new(
                Px(wire_drag.pos.x.0 + offset_x),
                Px(wire_drag.pos.y.0 + offset_y),
            ),
            Size::new(Px(box_w), Px(box_h)),
        );

        let border_color = if snapshot.interaction.connection_mode == NodeGraphConnectionMode::Loose
            && self.interaction.hover_port.is_some()
            && !self.interaction.hover_port_valid
        {
            if self.interaction.hover_port_convertible {
                Color {
                    r: 0.95,
                    g: 0.75,
                    b: 0.20,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.90,
                    g: 0.35,
                    b: 0.35,
                    a: 1.0,
                }
            }
        } else {
            self.style.context_menu_border
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(69),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(Px(1.0 / zoom)),
            border_color,
            corner_radii: Corners::all(Px(6.0 / zoom)),
        });

        let text_x = Px(rect.origin.x.0 + pad);
        let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(70),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: self.style.context_menu_text,
        });
    }

    fn yank_edges_from_port(graph: &Graph, port: PortId) -> Vec<(EdgeId, EdgeEndpoint, PortId)> {
        let Some(p) = graph.ports.get(&port) else {
            return Vec::new();
        };

        let mut out: Vec<(EdgeId, EdgeEndpoint, PortId)> = Vec::new();
        match p.dir {
            PortDirection::Out => {
                for (edge_id, edge) in &graph.edges {
                    if edge.from == port {
                        out.push((*edge_id, EdgeEndpoint::From, edge.to));
                    }
                }
            }
            PortDirection::In => {
                for (edge_id, edge) in &graph.edges {
                    if edge.to == port {
                        out.push((*edge_id, EdgeEndpoint::To, edge.from));
                    }
                }
            }
        }
        out
    }

    fn yank_reconnectable_edges_from_port<H: UiHost>(
        &self,
        host: &mut H,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> Vec<(EdgeId, EdgeEndpoint, PortId)> {
        if let Some(store) = self.store.as_ref() {
            if let Ok(out) = store.read_ref(host, |s| {
                use crate::runtime::lookups::ConnectionSide;

                let graph = s.graph();
                let Some(p) = graph.ports.get(&port) else {
                    return Vec::new();
                };

                let node = p.node;
                let side = ConnectionSide::from_port_dir(p.dir);
                let mut out: Vec<(EdgeId, EdgeEndpoint, PortId)> = Vec::new();

                let Some(conns) = s.lookups().connections_for_port(node, side, port) else {
                    return out;
                };

                for (edge_id, conn) in conns {
                    let (endpoint, fixed) = match p.dir {
                        PortDirection::Out => (EdgeEndpoint::From, conn.target_port),
                        PortDirection::In => (EdgeEndpoint::To, conn.source_port),
                    };
                    if Self::edge_endpoint_is_reconnectable(graph, interaction, *edge_id, endpoint)
                    {
                        out.push((*edge_id, endpoint, fixed));
                    }
                }

                out.sort_by_key(|(edge_id, _, _)| *edge_id);
                out
            }) {
                return out;
            }
        }

        self.graph
            .read_ref(host, |graph| {
                let mut edges = Self::yank_edges_from_port(graph, port);
                edges.retain(|(edge_id, endpoint, _fixed)| {
                    Self::edge_endpoint_is_reconnectable(graph, interaction, *edge_id, *endpoint)
                });
                edges
            })
            .ok()
            .unwrap_or_default()
    }

    fn edge_is_selectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
    ) -> bool {
        if !interaction.elements_selectable || !interaction.edges_selectable {
            return false;
        }
        let Some(edge) = graph.edges.get(&edge) else {
            return false;
        };
        edge.selectable.unwrap_or(true)
    }

    fn edge_is_deletable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
    ) -> bool {
        if !interaction.edges_deletable {
            return false;
        }
        let Some(edge) = graph.edges.get(&edge) else {
            return false;
        };
        edge.deletable.unwrap_or(true)
    }

    fn edge_reconnectable_flags(
        edge: &crate::core::Edge,
        interaction: &NodeGraphInteractionState,
    ) -> (bool, bool) {
        match edge.reconnectable {
            Some(crate::core::EdgeReconnectable::Bool(false)) => (false, false),
            Some(crate::core::EdgeReconnectable::Bool(true)) => (true, true),
            Some(crate::core::EdgeReconnectable::Endpoint(
                crate::core::EdgeReconnectableEndpoint::Source,
            )) => (true, false),
            Some(crate::core::EdgeReconnectable::Endpoint(
                crate::core::EdgeReconnectableEndpoint::Target,
            )) => (false, true),
            None => {
                let allow = interaction.edges_reconnectable;
                (allow, allow)
            }
        }
    }

    fn edge_endpoint_is_reconnectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
    ) -> bool {
        let Some(edge) = graph.edges.get(&edge) else {
            return false;
        };
        let (allow_source, allow_target) = Self::edge_reconnectable_flags(edge, interaction);
        match endpoint {
            EdgeEndpoint::From => allow_source,
            EdgeEndpoint::To => allow_target,
        }
    }

    fn node_is_selectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        if !interaction.elements_selectable {
            return false;
        }
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.selectable.unwrap_or(true)
    }

    fn node_is_draggable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        if !interaction.nodes_draggable {
            return false;
        }
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.draggable.unwrap_or(true)
    }

    fn port_is_connectable_base(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port) = graph.ports.get(&port) else {
            return false;
        };
        let node_connectable = Self::node_is_connectable(graph, interaction, port.node);
        port.connectable.unwrap_or(node_connectable)
    }

    fn port_is_connectable_start(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port_value) = graph.ports.get(&port) else {
            return false;
        };
        if !Self::port_is_connectable_base(graph, interaction, port) {
            return false;
        }
        port_value.connectable_start.unwrap_or(true)
    }

    fn port_is_connectable_end(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port_value) = graph.ports.get(&port) else {
            return false;
        };
        if !Self::port_is_connectable_base(graph, interaction, port) {
            return false;
        }
        port_value.connectable_end.unwrap_or(true)
    }

    fn node_is_connectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.connectable.unwrap_or(interaction.nodes_connectable)
    }

    fn node_is_deletable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        if !interaction.nodes_deletable {
            return false;
        }
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.deletable.unwrap_or(true)
    }

    fn should_add_bundle_port(
        graph: &Graph,
        from: PortId,
        bundle: &[PortId],
        candidate: PortId,
    ) -> bool {
        if candidate == from || bundle.contains(&candidate) {
            return false;
        }
        let Some(from_port) = graph.ports.get(&from) else {
            return false;
        };
        let Some(candidate_port) = graph.ports.get(&candidate) else {
            return false;
        };
        candidate_port.dir == from_port.dir
    }

    fn pick_reconnect_endpoint(
        &self,
        graph: &Graph,
        geom: &CanvasGeometry,
        edge_id: EdgeId,
        pos: Point,
        reconnect_radius_screen: f32,
        zoom: f32,
    ) -> Option<(EdgeEndpoint, PortId)> {
        let edge = graph.edges.get(&edge_id)?;

        let from_center = geom.port_center(edge.from);
        let to_center = geom.port_center(edge.to);

        let (from_center, to_center) = match (from_center, to_center) {
            (Some(a), Some(b)) => (a, b),
            _ => return None,
        };

        let d2_from = {
            let dx = pos.x.0 - from_center.x.0;
            let dy = pos.y.0 - from_center.y.0;
            dx * dx + dy * dy
        };
        let d2_to = {
            let dx = pos.x.0 - to_center.x.0;
            let dy = pos.y.0 - to_center.y.0;
            dx * dx + dy * dy
        };

        if reconnect_radius_screen.is_finite() && reconnect_radius_screen > 0.0 {
            let r = reconnect_radius_screen / zoom;
            let r2 = r * r;
            let min_d2 = d2_from.min(d2_to);
            if min_d2 > r2 {
                return None;
            }
        }

        if d2_from <= d2_to {
            Some((EdgeEndpoint::From, edge.to))
        } else {
            Some((EdgeEndpoint::To, edge.from))
        }
    }

    fn zoom_about_center_factor(&mut self, bounds: Rect, factor: f32) {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }

        let new_zoom = (zoom * factor).clamp(self.style.min_zoom, self.style.max_zoom);
        if (new_zoom - zoom).abs() <= 1.0e-6 {
            return;
        }

        let cx = 0.5 * bounds.size.width.0;
        let cy = 0.5 * bounds.size.height.0;
        let center_screen = (cx, cy);

        let pan_x = self.cached_pan.x;
        let pan_y = self.cached_pan.y;

        let g0_x = center_screen.0 / zoom - pan_x;
        let g0_y = center_screen.1 / zoom - pan_y;

        let new_pan_x = center_screen.0 / new_zoom - g0_x;
        let new_pan_y = center_screen.1 / new_zoom - g0_y;

        self.cached_pan = CanvasPoint {
            x: new_pan_x,
            y: new_pan_y,
        };
        self.cached_zoom = new_zoom;
    }

    fn zoom_about_pointer_factor(&mut self, position: Point, factor: f32) {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        if !position.x.0.is_finite() || !position.y.0.is_finite() {
            return;
        }

        let new_zoom = (zoom * factor).clamp(self.style.min_zoom, self.style.max_zoom);
        if (new_zoom - zoom).abs() <= 1.0e-6 {
            return;
        }

        let pan_x = self.cached_pan.x;
        let pan_y = self.cached_pan.y;

        // `position` is in the widget's local (canvas) coordinates.
        // Compute the pivot in screen coordinates (relative to bounds origin) to keep the
        // graph point under the cursor stable.
        let pivot_screen_x = (position.x.0 + pan_x) * zoom;
        let pivot_screen_y = (position.y.0 + pan_y) * zoom;

        let g0_x = pivot_screen_x / zoom - pan_x;
        let g0_y = pivot_screen_y / zoom - pan_y;

        let new_pan_x = pivot_screen_x / new_zoom - g0_x;
        let new_pan_y = pivot_screen_y / new_zoom - g0_y;

        self.cached_pan = CanvasPoint {
            x: new_pan_x,
            y: new_pan_y,
        };
        self.cached_zoom = new_zoom;
    }

    fn stop_auto_pan_timer<H: UiHost>(&mut self, host: &mut H) {
        let Some(timer) = self.interaction.auto_pan_timer.take() else {
            return;
        };
        host.push_effect(Effect::CancelTimer { token: timer });
    }

    fn stop_pan_inertia_timer<H: UiHost>(&mut self, host: &mut H) {
        let Some(inertia) = self.interaction.pan_inertia.take() else {
            return;
        };
        host.push_effect(Effect::CancelTimer {
            token: inertia.timer,
        });
    }

    fn bump_viewport_move_debounce<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        kind: ViewportMoveKind,
    ) {
        if let Some(prev) = self.interaction.viewport_move_debounce.take() {
            host.push_effect(Effect::CancelTimer { token: prev.timer });
            if prev.kind != kind {
                self.emit_move_end(snapshot, prev.kind, ViewportMoveEndOutcome::Ended);
                self.emit_move_start(snapshot, kind);
            }
        } else {
            self.emit_move_start(snapshot, kind);
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::VIEWPORT_MOVE_END_DEBOUNCE,
            repeat: None,
        });
        self.interaction.viewport_move_debounce = Some(ViewportMoveDebounceState { kind, timer });
    }

    fn pan_inertia_should_tick(&self) -> bool {
        if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
            return false;
        }
        if self.interaction.panning {
            return false;
        }
        self.interaction.pending_marquee.is_none()
            && self.interaction.marquee.is_none()
            && self.interaction.pending_node_drag.is_none()
            && self.interaction.node_drag.is_none()
            && self.interaction.pending_group_drag.is_none()
            && self.interaction.group_drag.is_none()
            && self.interaction.pending_group_resize.is_none()
            && self.interaction.group_resize.is_none()
            && self.interaction.pending_node_resize.is_none()
            && self.interaction.node_resize.is_none()
            && self.interaction.pending_wire_drag.is_none()
            && self.interaction.wire_drag.is_none()
            && self.interaction.edge_drag.is_none()
    }

    fn maybe_start_pan_inertia_timer<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.stop_pan_inertia_timer(host);

        let tuning = &snapshot.interaction.pan_inertia;
        if !tuning.enabled {
            return false;
        }

        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return false;
        }

        let mut velocity = self.interaction.pan_velocity;
        if !velocity.x.is_finite() || !velocity.y.is_finite() {
            return false;
        }

        let speed_screen = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt() * zoom;
        let min_speed = tuning.min_speed.max(0.0);
        if !speed_screen.is_finite() || speed_screen < min_speed {
            return false;
        }

        let max_speed = tuning.max_speed.max(min_speed);
        if max_speed.is_finite() && max_speed > 0.0 {
            let max_speed_canvas = max_speed / zoom;
            let speed_canvas = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
            if speed_canvas.is_finite() && speed_canvas > max_speed_canvas && speed_canvas > 0.0 {
                let scale = max_speed_canvas / speed_canvas;
                velocity.x *= scale;
                velocity.y *= scale;
            }
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::PAN_INERTIA_TICK_INTERVAL,
            repeat: Some(Self::PAN_INERTIA_TICK_INTERVAL),
        });
        self.interaction.pan_inertia = Some(PanInertiaState {
            timer,
            velocity,
            last_tick_at: Instant::now(),
        });
        true
    }

    fn ensure_auto_pan_timer_running<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) {
        if self.interaction.auto_pan_timer.is_some() {
            return;
        }
        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::AUTO_PAN_TICK_INTERVAL,
            repeat: Some(Self::AUTO_PAN_TICK_INTERVAL),
        });
        self.interaction.auto_pan_timer = Some(timer);
    }

    fn auto_pan_should_tick(&self, snapshot: &ViewSnapshot, bounds: Rect) -> bool {
        if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
            return false;
        }
        let Some(pos) = self.interaction.last_pos else {
            return false;
        };

        let wants_node_drag = snapshot.interaction.auto_pan.on_node_drag
            && (self.interaction.node_drag.is_some()
                || self.interaction.group_drag.is_some()
                || self.interaction.group_resize.is_some());
        let wants_connect =
            snapshot.interaction.auto_pan.on_connect && self.interaction.wire_drag.is_some();

        if !wants_node_drag && !wants_connect {
            return false;
        }

        let delta = Self::auto_pan_delta(snapshot, pos, bounds);
        delta.x != 0.0 || delta.y != 0.0
    }

    fn sync_auto_pan_timer<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        bounds: Rect,
    ) {
        if self.auto_pan_should_tick(snapshot, bounds) {
            self.ensure_auto_pan_timer_running(host, window);
        } else {
            self.stop_auto_pan_timer(host);
        }
    }

    fn focus_next_edge<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable
            || !snapshot.interaction.edges_selectable
            || !snapshot.interaction.edges_focusable
        {
            return false;
        }

        let mut edges: Vec<EdgeId> = self
            .graph
            .read_ref(host, |g| {
                g.edges
                    .keys()
                    .copied()
                    .filter(|id| Self::edge_is_selectable(g, &snapshot.interaction, *id))
                    .collect()
            })
            .ok()
            .unwrap_or_default();
        if edges.is_empty() {
            return false;
        }
        edges.sort_unstable();

        let current = self
            .interaction
            .focused_edge
            .or_else(|| snapshot.selected_edges.first().copied());

        let next = match current.and_then(|id| edges.iter().position(|e| *e == id)) {
            Some(ix) => {
                let len = edges.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                edges[next_ix]
            }
            None => {
                if forward {
                    edges[0]
                } else {
                    edges[edges.len() - 1]
                }
            }
        };

        self.interaction.focused_edge = Some(next);
        self.interaction.focused_node = None;
        self.interaction.focused_port = None;
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;
        self.update_view_state(host, |s| {
            s.selected_nodes.clear();
            s.selected_groups.clear();
            s.selected_edges.clear();
            s.selected_edges.push(next);
        });
        true
    }

    fn focus_next_node<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let ordered: Vec<GraphNodeId> = self
            .graph
            .read_ref(host, |g| {
                let mut out: Vec<GraphNodeId> = Vec::new();
                let mut used: HashSet<GraphNodeId> = HashSet::new();

                for id in &snapshot.draw_order {
                    if Self::node_is_selectable(g, &snapshot.interaction, *id) && used.insert(*id) {
                        out.push(*id);
                    }
                }

                let mut rest: Vec<GraphNodeId> = g
                    .nodes
                    .keys()
                    .copied()
                    .filter(|id| Self::node_is_selectable(g, &snapshot.interaction, *id))
                    .filter(|id| used.insert(*id))
                    .collect();
                rest.sort_unstable();
                out.extend(rest);
                out
            })
            .ok()
            .unwrap_or_default();

        if ordered.is_empty() {
            return false;
        }

        let current = self
            .interaction
            .focused_node
            .or_else(|| snapshot.selected_nodes.first().copied());

        let next = match current.and_then(|id| ordered.iter().position(|e| *e == id)) {
            Some(ix) => {
                let len = ordered.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                ordered[next_ix]
            }
            None => {
                if forward {
                    ordered[0]
                } else {
                    ordered[ordered.len() - 1]
                }
            }
        };

        self.interaction.focused_node = Some(next);
        self.interaction.focused_edge = None;
        self.interaction.focused_port = None;
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;
        self.update_view_state(host, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.clear();
            s.selected_nodes.push(next);
            s.draw_order.retain(|id| *id != next);
            s.draw_order.push(next);
        });
        true
    }

    fn refresh_focused_port_hints<H: UiHost>(&mut self, host: &mut H) {
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;

        let snapshot = self.sync_view_state(host);
        let mode = snapshot.interaction.connection_mode;

        let Some(target) = self.interaction.focused_port else {
            return;
        };
        let Some(wire_drag) = self.interaction.wire_drag.clone() else {
            return;
        };

        let presenter = &mut *self.presenter;
        let (valid, convertible) = self
            .graph
            .read_ref(host, |graph| {
                let mut scratch = graph.clone();

                let valid = match &wire_drag.kind {
                    WireDragKind::New { from, bundle } => {
                        let sources = if bundle.is_empty() {
                            std::slice::from_ref(from)
                        } else {
                            bundle.as_slice()
                        };
                        let mut any_accept = false;
                        for src in sources {
                            let plan = presenter.plan_connect(&scratch, *src, target, mode);
                            if plan.decision != ConnectDecision::Accept {
                                continue;
                            }
                            any_accept = true;
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                        }
                        any_accept
                    }
                    WireDragKind::Reconnect { edge, endpoint, .. } => matches!(
                        presenter
                            .plan_reconnect_edge(&scratch, *edge, *endpoint, target, mode)
                            .decision,
                        ConnectDecision::Accept
                    ),
                    WireDragKind::ReconnectMany { edges } => {
                        let mut any_accept = false;
                        for (edge, endpoint, _fixed) in edges {
                            let plan = presenter
                                .plan_reconnect_edge(&scratch, *edge, *endpoint, target, mode);
                            if plan.decision != ConnectDecision::Accept {
                                continue;
                            }
                            any_accept = true;
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                        }
                        any_accept
                    }
                };

                let convertible = if !valid {
                    match &wire_drag.kind {
                        WireDragKind::New { from, bundle } if bundle.len() <= 1 => {
                            conversion::is_convertible(presenter, &scratch, *from, target)
                        }
                        _ => false,
                    }
                } else {
                    false
                };

                (valid, convertible)
            })
            .ok()
            .unwrap_or((false, false));

        if self.interaction.wire_drag.is_some() && self.interaction.focused_port == Some(target) {
            self.interaction.focused_port_valid = valid;
            self.interaction.focused_port_convertible = convertible;
        }
    }

    fn focus_next_port<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let focused_node = self
            .interaction
            .focused_node
            .or_else(|| snapshot.selected_nodes.first().copied())
            .or_else(|| {
                self.graph
                    .read_ref(host, |g| g.nodes.keys().next().copied())
                    .ok()
                    .flatten()
            });

        let Some(focused_node) = focused_node else {
            return false;
        };

        let wire_dir = self.interaction.wire_drag.as_ref().and_then(|w| {
            let from_port = match &w.kind {
                WireDragKind::New { from, .. } => Some(*from),
                WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
                WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
            }?;
            self.graph
                .read_ref(host, |g| g.ports.get(&from_port).map(|p| p.dir))
                .ok()
                .flatten()
        });

        let ports = self
            .graph
            .read_ref(host, |g| {
                let (inputs, outputs) = node_ports(g, focused_node);
                let mut ports = Vec::with_capacity(inputs.len() + outputs.len());
                ports.extend(inputs);
                ports.extend(outputs);

                if let Some(wire_dir) = wire_dir {
                    let want = match wire_dir {
                        PortDirection::In => PortDirection::Out,
                        PortDirection::Out => PortDirection::In,
                    };
                    ports.retain(|id| g.ports.get(id).is_some_and(|p| p.dir == want));
                }

                ports
            })
            .ok()
            .unwrap_or_default();

        if ports.is_empty() {
            return false;
        }

        let current = self
            .interaction
            .focused_port
            .filter(|id| ports.iter().any(|p| *p == *id));

        let next = match current.and_then(|id| ports.iter().position(|p| *p == id)) {
            Some(ix) => {
                let len = ports.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                ports[next_ix]
            }
            None => {
                if forward {
                    ports[0]
                } else {
                    ports[ports.len() - 1]
                }
            }
        };

        self.interaction.focused_node = Some(focused_node);
        self.interaction.focused_edge = None;
        self.interaction.focused_port = Some(next);
        self.refresh_focused_port_hints(host);
        self.update_view_state(host, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.clear();
            s.selected_nodes.push(focused_node);
        });
        true
    }

    fn port_center_canvas<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        port: PortId,
    ) -> Option<CanvasPoint> {
        let (geom, _) = self.canvas_derived(&*host, snapshot);
        geom.ports.get(&port).map(|h| CanvasPoint {
            x: h.center.x.0,
            y: h.center.y.0,
        })
    }

    fn activate_focused_port<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let Some(port) = self
            .interaction
            .focused_port
            .or(self.interaction.hover_port)
        else {
            return false;
        };

        let pos = self
            .port_center_canvas(cx.app, snapshot, port)
            .map(|p| Point::new(Px(p.x), Px(p.y)))
            .or(self.interaction.last_pos)
            .unwrap_or_else(|| {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                Point::new(
                    Px(bounds.origin.x.0 + 0.5 * bounds.size.width.0),
                    Px(bounds.origin.y.0 + 0.5 * bounds.size.height.0),
                )
            });

        if self.interaction.wire_drag.is_none() {
            self.interaction.wire_drag = Some(WireDrag {
                kind: WireDragKind::New {
                    from: port,
                    bundle: Vec::new(),
                },
                pos,
            });
            self.interaction.click_connect = true;
            self.interaction.pending_wire_drag = None;
            self.interaction.suspended_wire_drag = None;
            self.interaction.sticky_wire = false;
            self.interaction.sticky_wire_ignore_next_up = false;
            self.interaction.focused_edge = None;
            self.interaction.focused_port = None;
            self.interaction.focused_port_valid = false;
            self.interaction.focused_port_convertible = false;
            self.interaction.hover_port = None;
            self.interaction.hover_port_valid = false;
            self.interaction.hover_port_convertible = false;
            return true;
        }

        if let Some(mut w) = self.interaction.wire_drag.take() {
            w.pos = pos;
            self.interaction.wire_drag = Some(w);
        }

        let _ = wire_drag::handle_wire_left_up_with_forced_target(
            self,
            cx,
            snapshot,
            snapshot.zoom,
            Some(port),
        );
        self.refresh_focused_port_hints(cx.app);
        true
    }

    fn focus_port_direction<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        dir: PortNavDir,
    ) -> bool {
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        if self.interaction.focused_port.is_none() {
            return self.focus_next_port(host, true);
        }

        let from_port = self.interaction.focused_port;
        let Some(from_port) = from_port else {
            return false;
        };

        let Some(from_center) = self.port_center_canvas(host, snapshot, from_port) else {
            return false;
        };

        let required_dir = self.interaction.wire_drag.as_ref().and_then(|w| {
            let from_port = match &w.kind {
                WireDragKind::New { from, .. } => Some(*from),
                WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
                WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
            }?;
            let dir = self
                .graph
                .read_ref(host, |g| g.ports.get(&from_port).map(|p| p.dir))
                .ok()
                .flatten()?;
            Some(match dir {
                PortDirection::In => PortDirection::Out,
                PortDirection::Out => PortDirection::In,
            })
        });

        let (geom, _) = self.canvas_derived(host, snapshot);
        let required_dir = required_dir;

        let best = self
            .graph
            .read_ref(host, |graph| {
                #[derive(Clone, Copy)]
                struct Rank {
                    angle: f32,
                    parallel: f32,
                    dist2: f32,
                    port: PortId,
                }

                let from = from_center;
                let mut best: Option<Rank> = None;

                for (&port, handle) in &geom.ports {
                    if port == from_port {
                        continue;
                    }

                    let Some(p) = graph.ports.get(&port) else {
                        continue;
                    };

                    if let Some(required_dir) = required_dir {
                        if p.dir != required_dir {
                            continue;
                        }
                    }

                    let dx = handle.center.x.0 - from.x;
                    let dy = handle.center.y.0 - from.y;
                    let (parallel, perp) = match dir {
                        PortNavDir::Left => (-dx, dy.abs()),
                        PortNavDir::Right => (dx, dy.abs()),
                        PortNavDir::Up => (-dy, dx.abs()),
                        PortNavDir::Down => (dy, dx.abs()),
                    };
                    if !parallel.is_finite() || !perp.is_finite() || parallel <= 1.0e-6 {
                        continue;
                    }

                    let angle = (perp / parallel).abs();
                    let dist2 = dx * dx + dy * dy;
                    if !angle.is_finite() || !dist2.is_finite() {
                        continue;
                    }

                    let cand = Rank {
                        angle,
                        parallel,
                        dist2,
                        port,
                    };

                    let better = match best {
                        None => true,
                        Some(best) => {
                            let by_angle = angle.total_cmp(&best.angle);
                            if by_angle != std::cmp::Ordering::Equal {
                                by_angle == std::cmp::Ordering::Less
                            } else {
                                let by_parallel = parallel.total_cmp(&best.parallel);
                                if by_parallel != std::cmp::Ordering::Equal {
                                    by_parallel == std::cmp::Ordering::Less
                                } else {
                                    let by_dist = dist2.total_cmp(&best.dist2);
                                    if by_dist != std::cmp::Ordering::Equal {
                                        by_dist == std::cmp::Ordering::Less
                                    } else {
                                        port < best.port
                                    }
                                }
                            }
                        }
                    };

                    if better {
                        best = Some(cand);
                    }
                }

                best.map(|r| r.port)
            })
            .ok()
            .flatten();

        let Some(next) = best else {
            return false;
        };

        let owner = self
            .graph
            .read_ref(host, |g| g.ports.get(&next).map(|p| p.node))
            .ok()
            .flatten();

        let Some(owner) = owner else {
            return false;
        };

        self.interaction.focused_node = Some(owner);
        self.interaction.focused_edge = None;
        self.interaction.focused_port = Some(next);
        self.refresh_focused_port_hints(host);
        self.update_view_state(host, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.clear();
            s.selected_nodes.push(owner);
            s.draw_order.retain(|id| *id != owner);
            s.draw_order.push(owner);
        });

        let snapshot = self.sync_view_state(host);
        if let Some(center) = self.port_center_canvas(host, &snapshot, next) {
            self.ensure_canvas_point_visible(host, &snapshot, center);
        }

        true
    }
}

impl<H: UiHost> Widget<H> for NodeGraphCanvas {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.paint_cache.clear(services);
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        let snapshot = self.sync_view_state(cx.app);
        if cx.input_ctx.focus_is_text_input && command.as_str().starts_with("node_graph.") {
            return false;
        }

        match command.as_str() {
            CMD_NODE_GRAPH_OPEN_INSERT_NODE => {
                let at = self
                    .interaction
                    .last_canvas_pos
                    .or_else(|| {
                        let bounds = self.interaction.last_bounds?;
                        let cx0 = bounds.origin.x.0 + 0.5 * bounds.size.width.0;
                        let cy0 = bounds.origin.y.0 + 0.5 * bounds.size.height.0;
                        let center = Point::new(Px(cx0), Px(cy0));
                        Some(Self::screen_to_canvas(
                            bounds,
                            center,
                            snapshot.pan,
                            snapshot.zoom,
                        ))
                    })
                    .unwrap_or_default();
                self.open_insert_node_picker(cx.app, at);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_CREATE_GROUP => {
                let at = self.interaction.last_canvas_pos.unwrap_or_default();
                self.create_group_at(cx.app, cx.window, at);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT => {
                self.interaction.context_menu = None;
                self.interaction.searcher = None;
                let groups = snapshot.selected_groups.clone();
                if groups.is_empty() {
                    return true;
                }
                self.update_view_state(cx.app, |s| {
                    let mut selected_in_order: Vec<crate::core::GroupId> = Vec::new();
                    for id in &s.group_draw_order {
                        if groups.contains(id) {
                            selected_in_order.push(*id);
                        }
                    }
                    for id in &groups {
                        if !selected_in_order.contains(id) {
                            selected_in_order.push(*id);
                        }
                    }
                    s.group_draw_order.retain(|id| !groups.contains(id));
                    s.group_draw_order.extend(selected_in_order);
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_GROUP_SEND_TO_BACK => {
                self.interaction.context_menu = None;
                self.interaction.searcher = None;
                let groups = snapshot.selected_groups.clone();
                if groups.is_empty() {
                    return true;
                }
                self.update_view_state(cx.app, |s| {
                    let mut selected_in_order: Vec<crate::core::GroupId> = Vec::new();
                    for id in &s.group_draw_order {
                        if groups.contains(id) {
                            selected_in_order.push(*id);
                        }
                    }
                    for id in &groups {
                        if !selected_in_order.contains(id) {
                            selected_in_order.push(*id);
                        }
                    }
                    s.group_draw_order.retain(|id| !groups.contains(id));
                    let mut next = selected_in_order;
                    next.extend_from_slice(&s.group_draw_order);
                    s.group_draw_order = next;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_GROUP_RENAME => {
                self.interaction.context_menu = None;
                self.interaction.searcher = None;
                let Some(overlays) = self.overlays.clone() else {
                    self.show_toast(
                        cx.app,
                        cx.window,
                        DiagnosticSeverity::Info,
                        "group rename overlay not configured",
                    );
                    return true;
                };
                let Some(group) = snapshot.selected_groups.last().copied() else {
                    return true;
                };
                let invoked_at = self
                    .interaction
                    .last_pos
                    .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));
                let _ = overlays.update(cx.app, |s, _cx| {
                    s.group_rename = Some(GroupRenameOverlay {
                        group,
                        invoked_at_window: invoked_at,
                    });
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE => {
                if snapshot.selected_edges.len() != 1 {
                    return true;
                }
                let edge = snapshot.selected_edges[0];
                let invoked_at = self
                    .interaction
                    .last_pos
                    .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));
                self.open_edge_insert_node_picker(cx.app, cx.window, edge, invoked_at);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_INSERT_REROUTE => {
                if snapshot.selected_edges.len() != 1 {
                    return true;
                }
                let edge_id = snapshot.selected_edges[0];
                let invoked_at = self
                    .interaction
                    .last_pos
                    .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));
                let at = self.reroute_pos_for_invoked_at(invoked_at);

                let outcome = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let plan = presenter.plan_split_edge(
                                graph,
                                edge_id,
                                &NodeKindKey::new(REROUTE_KIND),
                                at,
                            );
                            match plan.decision {
                                ConnectDecision::Accept => Ok(plan.ops),
                                ConnectDecision::Reject => Err(plan.diagnostics),
                            }
                        })
                        .ok()
                };

                match outcome {
                    Some(Ok(ops)) => {
                        let node_id = Self::first_added_node_id(&ops);
                        if self.commit_ops(cx.app, cx.window, Some("Insert Reroute"), ops) {
                            if let Some(node_id) = node_id {
                                self.update_view_state(cx.app, |s| {
                                    s.selected_edges.clear();
                                    s.selected_nodes.clear();
                                    s.selected_nodes.push(node_id);
                                    s.draw_order.retain(|id| *id != node_id);
                                    s.draw_order.push(node_id);
                                });
                            }
                        }
                    }
                    Some(Err(diags)) => {
                        if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                            self.show_toast(cx.app, cx.window, sev, msg);
                        }
                    }
                    None => {}
                }

                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER => {
                let Some(ctx0) = self.interaction.last_conversion.clone() else {
                    self.show_toast(
                        cx.app,
                        cx.window,
                        DiagnosticSeverity::Info,
                        "no recent conversion candidates",
                    );
                    return true;
                };

                let rows = super::searcher::build_rows_flat(&ctx0.candidates, "");
                let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                let origin = self.clamp_searcher_origin(
                    Point::new(Px(ctx0.at.x), Px(ctx0.at.y)),
                    visible,
                    bounds,
                    &snapshot,
                );
                let active_row =
                    Self::searcher_first_selectable_row(&rows).min(rows.len().saturating_sub(1));

                self.interaction.context_menu = None;
                self.interaction.searcher = Some(SearcherState {
                    origin,
                    invoked_at: Point::new(Px(ctx0.at.x), Px(ctx0.at.y)),
                    target: ContextMenuTarget::ConnectionConvertPicker {
                        from: ctx0.from,
                        to: ctx0.to,
                        at: ctx0.at,
                    },
                    query: String::new(),
                    candidates: ctx0.candidates,
                    recent_kinds: self.interaction.recent_kinds.clone(),
                    rows,
                    hovered_row: None,
                    active_row,
                    scroll: 0,
                });

                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_FRAME_SELECTION => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                let did =
                    self.frame_nodes_in_view(cx.app, cx.window, bounds, &snapshot.selected_nodes);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FRAME_ALL => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                let nodes = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        graph.nodes.keys().copied().collect::<Vec<_>>()
                    })
                    .ok()
                    .unwrap_or_default();
                let did = self.frame_nodes_in_view(cx.app, cx.window, bounds, &nodes);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_RESET_VIEW => {
                self.update_view_state(cx.app, |s| {
                    s.pan = CanvasPoint::default();
                    s.zoom = 1.0;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ZOOM_IN => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                self.zoom_about_center_factor(bounds, 1.2);
                let pan = self.cached_pan;
                let zoom = self.cached_zoom;
                self.update_view_state(cx.app, |s| {
                    s.pan = pan;
                    s.zoom = zoom;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ZOOM_OUT => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                self.zoom_about_center_factor(bounds, 1.0 / 1.2);
                let pan = self.cached_pan;
                let zoom = self.cached_zoom;
                self.update_view_state(cx.app, |s| {
                    s.pan = pan;
                    s.zoom = zoom;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE => {
                let next = match snapshot.interaction.connection_mode {
                    NodeGraphConnectionMode::Strict => NodeGraphConnectionMode::Loose,
                    NodeGraphConnectionMode::Loose => NodeGraphConnectionMode::Strict,
                };

                self.update_view_state(cx.app, |s| {
                    s.interaction.connection_mode = next;
                });
                self.show_toast(
                    cx.app,
                    cx.window,
                    DiagnosticSeverity::Info,
                    match next {
                        NodeGraphConnectionMode::Strict => "connection mode: strict",
                        NodeGraphConnectionMode::Loose => "connection mode: loose",
                    },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_UNDO => {
                let did = self.undo_last(cx.app, cx.window);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_REDO => {
                let did = self.redo_last(cx.app, cx.window);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_NEXT => {
                let did = self.focus_next_node(cx.app, true);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PREV => {
                let did = self.focus_next_node(cx.app, false);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_NEXT_EDGE => {
                let did = self.focus_next_edge(cx.app, true);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PREV_EDGE => {
                let did = self.focus_next_edge(cx.app, false);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_NEXT_PORT => {
                let did = self.focus_next_port(cx.app, true);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PREV_PORT => {
                let did = self.focus_next_port(cx.app, false);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PORT_LEFT => {
                let did = self.focus_port_direction(cx.app, &snapshot, PortNavDir::Left);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PORT_RIGHT => {
                let did = self.focus_port_direction(cx.app, &snapshot, PortNavDir::Right);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PORT_UP => {
                let did = self.focus_port_direction(cx.app, &snapshot, PortNavDir::Up);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PORT_DOWN => {
                let did = self.focus_port_direction(cx.app, &snapshot, PortNavDir::Down);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_ACTIVATE => {
                let did = self.activate_focused_port(cx, &snapshot);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_SELECT_ALL => {
                if !snapshot.interaction.elements_selectable {
                    return true;
                }
                let (nodes, groups, edges) = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        let nodes = graph
                            .nodes
                            .keys()
                            .copied()
                            .filter(|id| {
                                Self::node_is_selectable(graph, &snapshot.interaction, *id)
                            })
                            .collect::<Vec<_>>();
                        let groups = graph.groups.keys().copied().collect::<Vec<_>>();
                        let edges = if snapshot.interaction.edges_selectable {
                            graph
                                .edges
                                .keys()
                                .copied()
                                .filter(|id| {
                                    Self::edge_is_selectable(graph, &snapshot.interaction, *id)
                                })
                                .collect::<Vec<_>>()
                        } else {
                            Vec::new()
                        };
                        (nodes, groups, edges)
                    })
                    .ok()
                    .unwrap_or_default();

                self.interaction.focused_edge = None;
                self.interaction.focused_node = None;
                self.interaction.focused_port = None;
                self.interaction.focused_port_valid = false;
                self.interaction.focused_port_convertible = false;
                self.update_view_state(cx.app, |s| {
                    s.selected_nodes = nodes;
                    s.selected_groups = groups;
                    s.selected_edges = edges;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_COPY => {
                self.copy_selection_to_clipboard(
                    cx.app,
                    &snapshot.selected_nodes,
                    &snapshot.selected_groups,
                );
                true
            }
            CMD_NODE_GRAPH_CUT => {
                self.copy_selection_to_clipboard(
                    cx.app,
                    &snapshot.selected_nodes,
                    &snapshot.selected_groups,
                );

                let selected_nodes = snapshot.selected_nodes.clone();
                let selected_edges = snapshot.selected_edges.clone();
                let selected_groups = snapshot.selected_groups.clone();
                let remove_ops = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        Self::delete_selection_ops(
                            graph,
                            &snapshot.interaction,
                            &selected_nodes,
                            &selected_edges,
                            &selected_groups,
                        )
                    })
                    .ok()
                    .unwrap_or_default();
                if remove_ops.is_empty() {
                    return true;
                }
                let (removed_nodes, removed_edges, removed_groups) =
                    Self::removed_ids_from_ops(&remove_ops);
                let _ = self.commit_ops(cx.app, cx.window, Some("Cut"), remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.retain(|id| !removed_edges.contains(id));
                    s.selected_nodes.retain(|id| !removed_nodes.contains(id));
                    s.selected_groups.retain(|id| !removed_groups.contains(id));
                });

                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_PASTE => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                let at = self.next_paste_canvas_point(bounds, &snapshot);
                self.request_paste_at_canvas(cx.app, cx.window, at);
                true
            }
            CMD_NODE_GRAPH_DUPLICATE => {
                self.duplicate_selection(
                    cx.app,
                    cx.window,
                    &snapshot.selected_nodes,
                    &snapshot.selected_groups,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_DELETE_SELECTION => {
                let preferred_focus = self
                    .interaction
                    .focused_edge
                    .or_else(|| snapshot.selected_edges.first().copied());
                let selected_edges = snapshot.selected_edges.clone();
                let selected_nodes = snapshot.selected_nodes.clone();
                let selected_groups = snapshot.selected_groups.clone();
                if selected_edges.is_empty()
                    && selected_nodes.is_empty()
                    && selected_groups.is_empty()
                {
                    return true;
                }

                let remove_ops = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        Self::delete_selection_ops(
                            graph,
                            &snapshot.interaction,
                            &selected_nodes,
                            &selected_edges,
                            &selected_groups,
                        )
                    })
                    .ok()
                    .unwrap_or_default();

                if remove_ops.is_empty() {
                    return true;
                }
                let (removed_nodes, removed_edges, removed_groups) =
                    Self::removed_ids_from_ops(&remove_ops);
                let _ = self.commit_ops(cx.app, cx.window, Some("Delete Selection"), remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.retain(|id| !removed_edges.contains(id));
                    s.selected_nodes.retain(|id| !removed_nodes.contains(id));
                    s.selected_groups.retain(|id| !removed_groups.contains(id));
                });
                self.repair_focused_edge_after_graph_change(cx.app, preferred_focus);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_LEFT => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: -1.0, y: 0.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_RIGHT => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 1.0, y: 0.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_UP => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 0.0, y: -1.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_DOWN => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 0.0, y: 1.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_LEFT_FAST => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: -10.0, y: 0.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_RIGHT_FAST => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 10.0, y: 0.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_UP_FAST => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 0.0, y: -10.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_DOWN_FAST => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 0.0, y: 10.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_LEFT => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignLeft,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_RIGHT => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignRight,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_TOP => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignTop,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_BOTTOM => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignBottom,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_CENTER_X => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignCenterX,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_CENTER_Y => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignCenterY,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_DISTRIBUTE_X => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::DistributeX,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_DISTRIBUTE_Y => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::DistributeY,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            _ => false,
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        self.interaction.last_bounds = Some(cx.bounds);
        let snapshot = self.sync_view_state(cx.app);

        cx.set_role(fret_core::SemanticsRole::Viewport);
        cx.set_focusable(true);
        cx.set_label(self.presenter.a11y_canvas_label().as_ref());

        let active_descendant = match (
            self.interaction.focused_port.is_some(),
            self.interaction.focused_edge.is_some(),
            self.interaction.focused_node.is_some(),
        ) {
            (true, _, _) => cx.children.get(0).copied(),
            (false, true, _) => cx.children.get(1).copied(),
            (false, false, true) => cx.children.get(2).copied(),
            _ => None,
        };
        cx.set_active_descendant(active_descendant);

        let (focused_node, focused_port, focused_edge) = (
            self.interaction.focused_node,
            self.interaction.focused_port,
            self.interaction.focused_edge,
        );

        let style = self.style.clone();
        let value = self
            .graph
            .read_ref(cx.app, |graph| {
                let mut parts: Vec<String> = Vec::new();
                parts.push(format!("zoom {:.3}", snapshot.zoom));
                parts.push(format!(
                    "selected nodes {}, edges {}, groups {}",
                    snapshot.selected_nodes.len(),
                    snapshot.selected_edges.len(),
                    snapshot.selected_groups.len(),
                ));

                if self.interaction.wire_drag.is_some() {
                    parts.push("connecting".to_string());
                }

                if let Some(node) = focused_node {
                    if let Some(label) = self.presenter.a11y_node_label(graph, node) {
                        parts.push(format!("focused node {}", label));
                    } else {
                        parts.push(format!("focused node {:?}", node));
                    }
                }

                if let Some(port) = focused_port {
                    if let Some(label) = self.presenter.a11y_port_label(graph, port) {
                        parts.push(format!("focused port {}", label));
                    } else {
                        parts.push(format!("focused port {:?}", port));
                    }
                }

                if let Some(edge) = focused_edge {
                    if let Some(label) = self.presenter.a11y_edge_label(graph, edge, &style) {
                        parts.push(format!("focused edge {}", label));
                    } else {
                        parts.push(format!("focused edge {:?}", edge));
                    }
                }

                parts.join("; ")
            })
            .ok()
            .unwrap_or_else(|| format!("zoom {:.3}", snapshot.zoom));

        cx.set_value(value);
    }

    fn render_transform(&self, bounds: Rect) -> Option<Transform2D> {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return None;
        }
        let pan = Point::new(Px(self.cached_pan.x), Px(self.cached_pan.y));
        Some(
            Transform2D::translation(bounds.origin)
                .compose(Transform2D::scale_uniform(zoom))
                .compose(Transform2D::translation(pan)),
        )
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.view_state, Invalidation::Layout);
        if let Some(queue) = self.edit_queue.as_ref() {
            cx.observe_model(queue, Invalidation::Layout);
        }
        for &child in cx.children {
            cx.layout_in(child, cx.bounds);
        }
        self.interaction.last_bounds = Some(cx.bounds);
        self.sync_view_state(cx.app);
        self.drain_edit_queue(cx.app, cx.window);
        self.update_auto_measured_node_sizes(cx);
        cx.available
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let snapshot = self.sync_view_state(cx.app);
        self.interaction.last_bounds = Some(cx.bounds);
        let zoom = snapshot.zoom;

        match event {
            Event::ClipboardText { token, text } => {
                let Some(pending) = self.interaction.pending_paste.take() else {
                    return;
                };
                if pending.token != *token {
                    self.interaction.pending_paste = Some(pending);
                    return;
                }
                self.apply_paste_text(cx.app, cx.window, text, pending.at);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            Event::ClipboardTextUnavailable { token } => {
                if let Some(pending) = &self.interaction.pending_paste
                    && pending.token == *token
                {
                    self.interaction.pending_paste = None;
                    self.show_toast(
                        cx.app,
                        cx.window,
                        DiagnosticSeverity::Info,
                        "clipboard text unavailable",
                    );
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::WindowFocusChanged(false) => {
                if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
                    return;
                }

                cancel::handle_escape_cancel(self, cx);
                self.interaction.pan_activation_key_held = false;
                self.interaction.multi_selection_active = false;
                return;
            }
            Event::PointerCancel(_) => {
                cancel::cancel_active_gestures(self, cx);
                return;
            }
            Event::Timer { token } => {
                if self
                    .interaction
                    .toast
                    .as_ref()
                    .is_some_and(|t| t.timer == *token)
                {
                    self.interaction.toast = None;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if self
                    .interaction
                    .pan_inertia
                    .as_ref()
                    .is_some_and(|i| i.timer == *token)
                {
                    let tuning = snapshot.interaction.pan_inertia.clone();
                    let zoom = snapshot.zoom;
                    let before = snapshot.pan;

                    let Some(mut inertia) = self.interaction.pan_inertia.take() else {
                        return;
                    };
                    let timer = inertia.timer;
                    let mut end_move = false;

                    if !tuning.enabled
                        || !self.pan_inertia_should_tick()
                        || !zoom.is_finite()
                        || zoom <= 0.0
                        || !tuning.decay_per_s.is_finite()
                        || tuning.decay_per_s <= 0.0
                    {
                        cx.app.push_effect(Effect::CancelTimer { token: timer });
                        end_move = true;
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        if end_move {
                            let snap = self.sync_view_state(cx.app);
                            self.emit_move_end(
                                &snap,
                                ViewportMoveKind::PanInertia,
                                ViewportMoveEndOutcome::Ended,
                            );
                        }
                        return;
                    }

                    let now = Instant::now();
                    let dt = (now - inertia.last_tick_at).as_secs_f32().clamp(0.0, 0.2);
                    inertia.last_tick_at = now;

                    if dt > 0.0 {
                        let dx = inertia.velocity.x * dt;
                        let dy = inertia.velocity.y * dt;
                        self.update_view_state(cx.app, |s| {
                            s.pan.x += dx;
                            s.pan.y += dy;
                        });
                    }

                    let after = self.sync_view_state(cx.app).pan;
                    let moved_x = after.x - before.x;
                    let moved_y = after.y - before.y;
                    let moved = (moved_x * moved_x + moved_y * moved_y).sqrt();

                    let decay = (-tuning.decay_per_s * dt).exp();
                    inertia.velocity.x *= decay;
                    inertia.velocity.y *= decay;

                    let speed_screen = (inertia.velocity.x * inertia.velocity.x
                        + inertia.velocity.y * inertia.velocity.y)
                        .sqrt()
                        * zoom;
                    let min_speed = tuning.min_speed.max(0.0);

                    if moved <= 1.0e-6
                        || !speed_screen.is_finite()
                        || speed_screen <= min_speed
                        || !inertia.velocity.x.is_finite()
                        || !inertia.velocity.y.is_finite()
                    {
                        cx.app.push_effect(Effect::CancelTimer { token: timer });
                        end_move = true;
                    } else {
                        self.interaction.pan_inertia = Some(inertia);
                    }

                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    if end_move {
                        let snap = self.sync_view_state(cx.app);
                        self.emit_move_end(
                            &snap,
                            ViewportMoveKind::PanInertia,
                            ViewportMoveEndOutcome::Ended,
                        );
                    }
                    return;
                }

                if self.interaction.auto_pan_timer == Some(*token) {
                    if !self.auto_pan_should_tick(&snapshot, cx.bounds) {
                        self.stop_auto_pan_timer(cx.app);
                        return;
                    }

                    let pos = self.interaction.last_pos.unwrap_or_default();
                    let mods = self.interaction.last_modifiers;
                    let zoom = snapshot.zoom;

                    if self.interaction.wire_drag.is_some() {
                        let _ =
                            wire_drag::handle_wire_drag_move(self, cx, &snapshot, pos, mods, zoom);
                    } else if self.interaction.node_drag.is_some() {
                        let _ =
                            node_drag::handle_node_drag_move(self, cx, &snapshot, pos, mods, zoom);
                    } else if self.interaction.group_drag.is_some() {
                        let _ = group_drag::handle_group_drag_move(
                            self, cx, &snapshot, pos, mods, zoom,
                        );
                    } else if self.interaction.group_resize.is_some() {
                        let _ = group_resize::handle_group_resize_move(
                            self, cx, &snapshot, pos, mods, zoom,
                        );
                    }

                    let snapshot = self.sync_view_state(cx.app);
                    self.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }

                if self
                    .interaction
                    .viewport_move_debounce
                    .as_ref()
                    .is_some_and(|s| s.timer == *token)
                {
                    let Some(state) = self.interaction.viewport_move_debounce.take() else {
                        return;
                    };
                    let snapshot = self.sync_view_state(cx.app);
                    self.emit_move_end(&snapshot, state.kind, ViewportMoveEndOutcome::Ended);
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                if cx.input_ctx.focus_is_text_input {
                    return;
                }

                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);

                if *key == fret_core::KeyCode::Escape {
                    if searcher::handle_searcher_escape(self, cx)
                        || context_menu::handle_context_menu_escape(self, cx)
                    {
                        return;
                    }
                    cancel::handle_escape_cancel(self, cx);
                    return;
                }

                if searcher::handle_searcher_key_down(self, cx, *key, *modifiers)
                    || context_menu::handle_context_menu_key_down(self, cx, *key)
                {
                    return;
                }

                if modifiers.ctrl || modifiers.meta {
                    if !snapshot.interaction.disable_keyboard_a11y
                        && *key == fret_core::KeyCode::Tab
                    {
                        let cmd = if modifiers.shift {
                            CMD_NODE_GRAPH_FOCUS_PREV_EDGE
                        } else {
                            CMD_NODE_GRAPH_FOCUS_NEXT_EDGE
                        };
                        cx.dispatch_command(CommandId::from(cmd));
                        cx.stop_propagation();
                        return;
                    }

                    match *key {
                        fret_core::KeyCode::KeyA => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_SELECT_ALL));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyZ => {
                            let cmd = if modifiers.shift {
                                CMD_NODE_GRAPH_REDO
                            } else {
                                CMD_NODE_GRAPH_UNDO
                            };
                            cx.dispatch_command(CommandId::from(cmd));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyY => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_REDO));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyC => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_COPY));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyX => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_CUT));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyV => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_PASTE));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyD => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_DUPLICATE));
                            cx.stop_propagation();
                            return;
                        }
                        _ => {}
                    }
                }

                if !snapshot.interaction.disable_keyboard_a11y
                    && *key == fret_core::KeyCode::Tab
                    && !modifiers.ctrl
                    && !modifiers.meta
                    && !modifiers.alt
                    && !modifiers.alt_gr
                {
                    if self.interaction.searcher.is_some()
                        || self.interaction.context_menu.is_some()
                    {
                        return;
                    }

                    let cmd = if modifiers.shift {
                        CMD_NODE_GRAPH_FOCUS_PREV
                    } else {
                        CMD_NODE_GRAPH_FOCUS_NEXT
                    };
                    cx.dispatch_command(CommandId::from(cmd));
                    cx.stop_propagation();
                    return;
                }

                if !modifiers.ctrl && !modifiers.meta && !modifiers.alt && !modifiers.alt_gr {
                    if snapshot.interaction.space_to_pan
                        && self.interaction.searcher.is_none()
                        && self.interaction.context_menu.is_none()
                    {
                        if let Some(crate::io::NodeGraphKeyCode(key_code)) =
                            snapshot.interaction.pan_activation_key_code
                        {
                            if *key == key_code && !self.interaction.pan_activation_key_held {
                                self.interaction.pan_activation_key_held = true;
                                cx.request_redraw();
                                cx.invalidate_self(Invalidation::Paint);
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }
                }

                if matches!(
                    key,
                    fret_core::KeyCode::ArrowLeft
                        | fret_core::KeyCode::ArrowRight
                        | fret_core::KeyCode::ArrowUp
                        | fret_core::KeyCode::ArrowDown
                ) && !modifiers.ctrl
                    && !modifiers.meta
                    && !modifiers.alt
                    && !modifiers.alt_gr
                {
                    if snapshot.interaction.disable_keyboard_a11y {
                        return;
                    }

                    if snapshot.selected_nodes.is_empty() && snapshot.selected_groups.is_empty() {
                        return;
                    }

                    let cmd = match (*key, modifiers.shift) {
                        (fret_core::KeyCode::ArrowLeft, false) => CMD_NODE_GRAPH_NUDGE_LEFT,
                        (fret_core::KeyCode::ArrowRight, false) => CMD_NODE_GRAPH_NUDGE_RIGHT,
                        (fret_core::KeyCode::ArrowUp, false) => CMD_NODE_GRAPH_NUDGE_UP,
                        (fret_core::KeyCode::ArrowDown, false) => CMD_NODE_GRAPH_NUDGE_DOWN,
                        (fret_core::KeyCode::ArrowLeft, true) => CMD_NODE_GRAPH_NUDGE_LEFT_FAST,
                        (fret_core::KeyCode::ArrowRight, true) => CMD_NODE_GRAPH_NUDGE_RIGHT_FAST,
                        (fret_core::KeyCode::ArrowUp, true) => CMD_NODE_GRAPH_NUDGE_UP_FAST,
                        (fret_core::KeyCode::ArrowDown, true) => CMD_NODE_GRAPH_NUDGE_DOWN_FAST,
                        _ => return,
                    };
                    cx.dispatch_command(CommandId::from(cmd));
                    cx.stop_propagation();
                    return;
                }

                if !snapshot.interaction.delete_key.matches(*key) {
                    return;
                }

                cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION));
                cx.stop_propagation();
                return;
            }
            Event::KeyUp { key, .. } => {
                let Some(crate::io::NodeGraphKeyCode(key_code)) =
                    snapshot.interaction.pan_activation_key_code
                else {
                    return;
                };
                if *key == key_code && self.interaction.pan_activation_key_held {
                    self.interaction.pan_activation_key_held = false;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                return;
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                if self.interaction.pan_inertia.is_some() {
                    self.stop_pan_inertia_timer(cx.app);
                    self.emit_move_end(
                        &snapshot,
                        ViewportMoveKind::PanInertia,
                        ViewportMoveEndOutcome::Ended,
                    );
                }
                self.interaction.last_pos = Some(*position);
                self.interaction.last_modifiers = *modifiers;
                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);
                self.interaction.last_canvas_pos = Some(CanvasPoint {
                    x: position.x.0,
                    y: position.y.0,
                });

                if searcher::handle_searcher_pointer_down(self, cx, *position, *button, zoom) {
                    return;
                }

                if *button == MouseButton::Left {
                    if let Some(command) = self.close_command.clone() {
                        let rect = Self::close_button_rect(snapshot.pan, zoom);
                        if Self::rect_contains(rect, *position) {
                            cx.dispatch_command(command);
                            cx.stop_propagation();
                            return;
                        }
                    }
                }

                if *button == MouseButton::Left
                    && *click_count == 2
                    && snapshot.interaction.zoom_on_double_click
                    && self.interaction.searcher.is_none()
                    && self.interaction.context_menu.is_none()
                {
                    let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
                    let is_background = self
                        .graph
                        .read_ref(cx.app, |graph| {
                            let mut scratch_ports: Vec<PortId> = Vec::new();
                            let mut scratch_edges: Vec<EdgeId> = Vec::new();

                            if self
                                .hit_port(
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_ports,
                                )
                                .is_some()
                            {
                                return false;
                            }
                            if self
                                .hit_edge_focus_anchor(
                                    graph,
                                    &snapshot,
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_edges,
                                )
                                .is_some()
                            {
                                return false;
                            }
                            if geom.nodes.values().any(|ng| ng.rect.contains(*position)) {
                                return false;
                            }
                            if self
                                .hit_edge(
                                    graph,
                                    &snapshot,
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_edges,
                                )
                                .is_some()
                            {
                                return false;
                            }
                            !graph.groups.values().any(|group| {
                                group_resize::group_rect_to_px(group.rect).contains(*position)
                            })
                        })
                        .unwrap_or(false);

                    if is_background {
                        if let Some(state) = self.interaction.viewport_move_debounce.take() {
                            cx.app
                                .push_effect(Effect::CancelTimer { token: state.timer });
                            self.emit_move_end(
                                &snapshot,
                                state.kind,
                                ViewportMoveEndOutcome::Ended,
                            );
                        }

                        self.emit_move_start(&snapshot, ViewportMoveKind::ZoomDoubleClick);
                        let factor = if modifiers.shift { 0.5 } else { 2.0 };
                        self.zoom_about_pointer_factor(*position, factor);
                        let pan = self.cached_pan;
                        let zoom = self.cached_zoom;
                        self.update_view_state(cx.app, |s| {
                            s.pan = pan;
                            s.zoom = zoom;
                        });
                        let snap = self.sync_view_state(cx.app);
                        self.emit_move_end(
                            &snap,
                            ViewportMoveKind::ZoomDoubleClick,
                            ViewportMoveEndOutcome::Ended,
                        );
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }
                }

                if self.interaction.context_menu.is_some()
                    && context_menu::handle_context_menu_pointer_down(
                        self, cx, *position, *button, zoom,
                    )
                {
                    return;
                }

                if *button == MouseButton::Right {
                    cancel::cancel_active_gestures(self, cx);
                    if snapshot.interaction.pan_on_drag.right {
                        self.interaction.pending_right_click =
                            Some(super::state::PendingRightClick {
                                start_pos: *position,
                            });
                        cx.capture_pointer(cx.node);
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }
                }

                if sticky_wire::handle_sticky_wire_pointer_down(
                    self, cx, &snapshot, *position, *button, zoom,
                ) {
                    return;
                }

                if *button == MouseButton::Left
                    && snapshot.interaction.space_to_pan
                    && self.interaction.pan_activation_key_held
                    && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
                {
                    let _ = pan_zoom::begin_panning(
                        self,
                        cx,
                        &snapshot,
                        *position,
                        fret_core::MouseButton::Left,
                    );
                    return;
                }

                if *button == MouseButton::Middle && snapshot.interaction.pan_on_drag.middle {
                    let _ = pan_zoom::begin_panning(
                        self,
                        cx,
                        &snapshot,
                        *position,
                        fret_core::MouseButton::Middle,
                    );
                    return;
                }

                if *button == MouseButton::Right
                    && right_click::handle_right_click_pointer_down(
                        self, cx, &snapshot, *position, zoom,
                    )
                {
                    return;
                }

                if *button != MouseButton::Left {
                    return;
                }

                let _ = left_click::handle_left_click_pointer_down(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                );
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position,
                buttons,
                modifiers,
                ..
            }) => {
                self.interaction.last_modifiers = *modifiers;
                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);

                // The runtime may occasionally miss a corresponding `PointerEvent::Up` (e.g. when
                // releasing outside of the window / losing capture). Infer the release from the
                // current button state and synthesize an "up" so we can finish the interaction
                // through the canonical pointer-up code path (commit, cancel, inertia, etc.).
                if self.interaction.panning {
                    let should_end = match self.interaction.panning_button {
                        Some(fret_core::MouseButton::Middle) => !buttons.middle,
                        Some(fret_core::MouseButton::Left) => !buttons.left,
                        Some(fret_core::MouseButton::Right) => !buttons.right,
                        _ => false,
                    };
                    if should_end {
                        let snapshot = self.sync_view_state(cx.app);
                        let button = self
                            .interaction
                            .panning_button
                            .unwrap_or(fret_core::MouseButton::Middle);
                        let _ = pointer_up::handle_pointer_up(
                            self,
                            cx,
                            &snapshot,
                            *position,
                            button,
                            1,
                            *modifiers,
                            snapshot.zoom,
                        );
                        return;
                    }
                }

                if snapshot.interaction.pan_on_drag.right
                    && buttons.right
                    && self.interaction.panning_button.is_none()
                    && let Some(pending) = self.interaction.pending_right_click
                {
                    let click_distance = snapshot.interaction.pane_click_distance.max(0.0);
                    let threshold = click_distance / zoom;
                    let dx = position.x.0 - pending.start_pos.x.0;
                    let dy = position.y.0 - pending.start_pos.y.0;
                    if click_distance == 0.0 || (dx * dx + dy * dy) > threshold * threshold {
                        self.interaction.pending_right_click = None;
                        let _ = pan_zoom::begin_panning(
                            self,
                            cx,
                            &snapshot,
                            *position,
                            fret_core::MouseButton::Right,
                        );
                        return;
                    }
                }

                let has_left_interaction = self.interaction.pending_marquee.is_some()
                    || self.interaction.marquee.is_some()
                    || self.interaction.pending_node_drag.is_some()
                    || self.interaction.node_drag.is_some()
                    || self.interaction.pending_group_drag.is_some()
                    || self.interaction.group_drag.is_some()
                    || self.interaction.pending_group_resize.is_some()
                    || self.interaction.group_resize.is_some()
                    || self.interaction.pending_node_resize.is_some()
                    || self.interaction.node_resize.is_some()
                    || self.interaction.pending_wire_drag.is_some()
                    || self.interaction.wire_drag.is_some()
                    || self.interaction.edge_drag.is_some();

                if has_left_interaction && !buttons.left {
                    let snapshot = self.sync_view_state(cx.app);
                    let _ = pointer_up::handle_pointer_up(
                        self,
                        cx,
                        &snapshot,
                        *position,
                        fret_core::MouseButton::Left,
                        1,
                        *modifiers,
                        snapshot.zoom,
                    );
                    return;
                }

                if self.interaction.last_pos.is_none() {
                    self.interaction.last_pos = Some(*position);
                    self.interaction.last_modifiers = *modifiers;
                    self.interaction.last_canvas_pos = Some(CanvasPoint {
                        x: position.x.0,
                        y: position.y.0,
                    });
                    return;
                }
                self.interaction.last_pos = Some(*position);
                self.interaction.last_modifiers = *modifiers;
                self.interaction.last_canvas_pos = Some(CanvasPoint {
                    x: position.x.0,
                    y: position.y.0,
                });

                cursor::update_cursors(self, cx, &snapshot, *position, zoom);

                if pan_zoom::handle_panning_move(self, cx, &snapshot, *position) {
                    // keep going to sync auto-pan timer
                } else if marquee::handle_marquee_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_group_drag::handle_pending_group_drag_move(
                    self, cx, &snapshot, *position, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if group_drag::handle_group_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_group_resize::handle_pending_group_resize_move(
                    self, cx, &snapshot, *position, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if group_resize::handle_group_resize_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_drag::handle_pending_node_drag_move(
                    self, cx, &snapshot, *position, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_resize::handle_pending_node_resize_move(
                    self, cx, &snapshot, *position, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_wire_drag::handle_pending_wire_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if node_resize::handle_node_resize_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if node_drag::handle_node_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if wire_drag::handle_wire_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if edge_drag::handle_edge_drag_move(self, cx, &snapshot, *position, zoom) {
                    // keep going to sync auto-pan timer
                } else if searcher::handle_searcher_pointer_move(self, cx, *position, zoom) {
                    // keep going to sync auto-pan timer
                } else if context_menu::handle_context_menu_pointer_move(self, cx, *position, zoom)
                {
                    // keep going to sync auto-pan timer
                } else {
                    hover::update_hover_edge(self, cx, &snapshot, *position, zoom);
                }

                let snapshot = self.sync_view_state(cx.app);
                self.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                self.interaction.last_modifiers = *modifiers;
                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);

                if *button == MouseButton::Right
                    && snapshot.interaction.pan_on_drag.right
                    && let Some(pending) = self.interaction.pending_right_click.take()
                {
                    let click_distance = snapshot.interaction.pane_click_distance.max(0.0);
                    let threshold = click_distance / zoom;
                    let dx = position.x.0 - pending.start_pos.x.0;
                    let dy = position.y.0 - pending.start_pos.y.0;
                    let is_click =
                        click_distance == 0.0 || (dx * dx + dy * dy) <= threshold * threshold;

                    cx.release_pointer_capture();
                    if is_click {
                        right_click::handle_right_click_pointer_down(
                            self, cx, &snapshot, *position, zoom,
                        );
                    }
                    return;
                }
                if pointer_up::handle_pointer_up(
                    self,
                    cx,
                    &snapshot,
                    *position,
                    *button,
                    *click_count,
                    *modifiers,
                    zoom,
                ) {
                    return;
                }
            }
            Event::Pointer(fret_core::PointerEvent::Wheel {
                position,
                delta,
                modifiers,
                ..
            }) => {
                if self.interaction.pan_inertia.is_some() {
                    self.stop_pan_inertia_timer(cx.app);
                    self.emit_move_end(
                        &snapshot,
                        ViewportMoveKind::PanInertia,
                        ViewportMoveEndOutcome::Ended,
                    );
                }
                self.interaction.last_modifiers = *modifiers;
                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);
                if searcher::handle_searcher_wheel(self, cx, *delta, *modifiers, zoom) {
                    return;
                }

                let zoom_active = snapshot
                    .interaction
                    .zoom_activation_key
                    .is_pressed(*modifiers);
                if snapshot.interaction.zoom_on_scroll && zoom_active {
                    self.bump_viewport_move_debounce(
                        cx.app,
                        cx.window,
                        &snapshot,
                        ViewportMoveKind::ZoomWheel,
                    );
                    let speed = snapshot.interaction.zoom_on_scroll_speed.max(0.0);
                    let delta_screen_y = delta.y.0 * zoom * speed;
                    let zoom_speed = 0.0015;
                    let factor = (1.0 + (-delta_screen_y * zoom_speed)).clamp(0.2, 5.0);
                    self.zoom_about_pointer_factor(*position, factor);
                    let pan = self.cached_pan;
                    let zoom = self.cached_zoom;
                    self.update_view_state(cx.app, |s| {
                        s.pan = pan;
                        s.zoom = zoom;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                } else if snapshot.interaction.pan_on_scroll
                    || (snapshot.interaction.space_to_pan
                        && self.interaction.pan_activation_key_held)
                {
                    self.bump_viewport_move_debounce(
                        cx.app,
                        cx.window,
                        &snapshot,
                        ViewportMoveKind::PanScroll,
                    );
                    let mode = snapshot.interaction.pan_on_scroll_mode;
                    let speed = snapshot.interaction.pan_on_scroll_speed.max(0.0);
                    let dy_for_shift = delta.y.0;

                    let mut dx = delta.x.0;
                    let mut dy = delta.y.0;
                    match mode {
                        crate::io::NodeGraphPanOnScrollMode::Free => {}
                        crate::io::NodeGraphPanOnScrollMode::Horizontal => {
                            dy = 0.0;
                        }
                        crate::io::NodeGraphPanOnScrollMode::Vertical => {
                            dx = 0.0;
                        }
                    }

                    if cx.input_ctx.platform != fret_runtime::Platform::Macos
                        && modifiers.shift
                        && !matches!(mode, crate::io::NodeGraphPanOnScrollMode::Vertical)
                    {
                        dx = dy_for_shift;
                        dy = 0.0;
                    }
                    self.update_view_state(cx.app, |s| {
                        s.pan.x += dx * speed;
                        s.pan.y += dy * speed;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::Pointer(fret_core::PointerEvent::PinchGesture {
                position, delta, ..
            }) => {
                if self.interaction.pan_inertia.is_some() {
                    self.stop_pan_inertia_timer(cx.app);
                    self.emit_move_end(
                        &snapshot,
                        ViewportMoveKind::PanInertia,
                        ViewportMoveEndOutcome::Ended,
                    );
                }
                if !snapshot.interaction.zoom_on_pinch {
                    return;
                }
                if !delta.is_finite() {
                    return;
                }

                self.bump_viewport_move_debounce(
                    cx.app,
                    cx.window,
                    &snapshot,
                    ViewportMoveKind::ZoomPinch,
                );

                let speed = snapshot.interaction.zoom_on_pinch_speed.max(0.0);
                let delta = (*delta).clamp(-0.95, 10.0);
                let factor = (1.0 + delta * speed).max(0.01);
                self.zoom_about_pointer_factor(*position, factor);
                let pan = self.cached_pan;
                let zoom = self.cached_zoom;
                self.update_view_state(cx.app, |s| {
                    s.pan = pan;
                    s.zoom = zoom;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
        let snapshot = self.sync_view_state(cx.app);

        self.paint_cache.begin_frame();
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        let zoom = snapshot.zoom;
        let pan = snapshot.pan;

        let viewport_w = cx.bounds.size.width.0 / zoom;
        let viewport_h = cx.bounds.size.height.0 / zoom;
        let viewport_origin_x = -pan.x;
        let viewport_origin_y = -pan.y;
        let render_cull_rect = {
            let margin_screen = self.style.render_cull_margin_px;
            if !margin_screen.is_finite()
                || margin_screen <= 0.0
                || !viewport_w.is_finite()
                || !viewport_h.is_finite()
                || viewport_w <= 0.0
                || viewport_h <= 0.0
            {
                None
            } else {
                let margin = margin_screen / zoom;
                Some(inflate_rect(
                    Rect::new(
                        Point::new(Px(viewport_origin_x), Px(viewport_origin_y)),
                        Size::new(Px(viewport_w), Px(viewport_h)),
                    ),
                    margin,
                ))
            }
        };

        cx.scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(viewport_origin_x), Px(viewport_origin_y)),
                Size::new(Px(viewport_w), Px(viewport_h)),
            ),
        });

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(
                Point::new(Px(viewport_origin_x), Px(viewport_origin_y)),
                Size::new(Px(viewport_w), Px(viewport_h)),
            ),
            background: self.style.background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        let spacing = self.style.grid_spacing;
        if spacing.is_finite() && spacing > 1.0e-3 {
            let major_every = self.style.grid_major_every.max(1) as i64;
            let thickness = Px((1.0 / zoom).max(0.25 / zoom));

            let x0 = (viewport_origin_x / spacing).floor() as i64;
            let x1 = ((viewport_origin_x + viewport_w) / spacing).ceil() as i64;
            for ix in x0..=x1 {
                let x = ix as f32 * spacing;
                let color = if ix.rem_euclid(major_every) == 0 {
                    self.style.grid_major_color
                } else {
                    self.style.grid_minor_color
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(x - 0.5 * thickness.0), Px(viewport_origin_y)),
                        Size::new(thickness, Px(viewport_h)),
                    ),
                    background: color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            let y0 = (viewport_origin_y / spacing).floor() as i64;
            let y1 = ((viewport_origin_y + viewport_h) / spacing).ceil() as i64;
            for iy in y0..=y1 {
                let y = iy as f32 * spacing;
                let color = if iy.rem_euclid(major_every) == 0 {
                    self.style.grid_major_color
                } else {
                    self.style.grid_minor_color
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(viewport_origin_x), Px(y - 0.5 * thickness.0)),
                        Size::new(Px(viewport_w), thickness),
                    ),
                    background: color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }

        #[derive(Debug, Default)]
        struct RenderData {
            groups: Vec<(Rect, Arc<str>, bool)>,
            edges: Vec<EdgeRender>,
            nodes: Vec<(
                GraphNodeId,
                Rect,
                bool,
                Arc<str>,
                Option<Arc<str>>,
                usize,
                NodeResizeHandleSet,
            )>,
            pins: Vec<(PortId, Rect, Color)>,
            port_labels: HashMap<PortId, PortLabelRender>,
            port_centers: HashMap<PortId, Point>,
        }

        #[derive(Debug, Clone)]
        struct EdgeRender {
            id: EdgeId,
            from: Point,
            to: Point,
            color: Color,
            hint: EdgeRenderHint,
            selected: bool,
            hovered: bool,
        }

        #[derive(Debug, Clone)]
        struct PortLabelRender {
            label: Arc<str>,
            dir: PortDirection,
            max_width: Px,
        }

        let hovered_edge = self.interaction.hover_edge;
        let hovered_port = self.interaction.hover_port;
        let hovered_port_valid = self.interaction.hover_port_valid;
        let hovered_port_convertible = self.interaction.hover_port_convertible;
        let focused_port = self.interaction.focused_port;
        let focused_port_valid = self.interaction.focused_port_valid;
        let focused_port_convertible = self.interaction.focused_port_convertible;
        let wire_drag = self.interaction.wire_drag.clone();
        let marked_ports: HashSet<PortId> = match wire_drag.as_ref().map(|w| &w.kind) {
            Some(WireDragKind::New { bundle, .. }) if bundle.len() > 1 => {
                bundle.iter().copied().collect()
            }
            Some(WireDragKind::ReconnectMany { edges }) if edges.len() > 1 => edges
                .iter()
                .map(|(_edge, _endpoint, fixed)| *fixed)
                .collect(),
            _ => HashSet::new(),
        };

        let geom = self.canvas_geometry(&*cx.app, &snapshot);
        self.update_measured_output_store(snapshot.zoom, &geom);
        self.update_internals_store(&*cx.app, &snapshot, cx.bounds, &geom);
        let render = {
            let selected: HashSet<GraphNodeId> = snapshot.selected_nodes.iter().copied().collect();
            let selected_edges: HashSet<EdgeId> = snapshot.selected_edges.iter().copied().collect();
            let selected_groups: HashSet<crate::core::GroupId> =
                snapshot.selected_groups.iter().copied().collect();
            let this = &*self;
            let geom = geom.clone();
            let presenter: &dyn NodeGraphPresenter = &*this.presenter;
            let cull = render_cull_rect;
            this.graph
                .read_ref(cx.app, |graph| {
                    let mut out = RenderData::default();

                    let geom = geom.as_ref();
                    let node_pad = this.style.node_padding;
                    let pin_gap = 8.0;
                    let pin_r = this.style.pin_radius;
                    let label_overhead = 2.0 * node_pad + 2.0 * (pin_r + pin_gap);

                    let order = group_order(graph, &snapshot.group_draw_order);
                    for group_id in order {
                        let Some(group) = graph.groups.get(&group_id) else {
                            continue;
                        };
                        let rect = Rect::new(
                            Point::new(Px(group.rect.origin.x), Px(group.rect.origin.y)),
                            Size::new(Px(group.rect.size.width), Px(group.rect.size.height)),
                        );
                        if cull.is_some_and(|c| !rects_intersect(rect, c)) {
                            continue;
                        }
                        out.groups.push((
                            rect,
                            Arc::<str>::from(group.title.clone()),
                            selected_groups.contains(&group_id),
                        ));
                    }

                    let mut visible_nodes: HashSet<GraphNodeId> = HashSet::new();
                    if let Some(c) = cull {
                        for (&node, node_geom) in &geom.nodes {
                            if rects_intersect(node_geom.rect, c) {
                                visible_nodes.insert(node);
                            }
                        }
                    }

                    for node in geom.order.iter().copied() {
                        let Some(node_geom) = geom.nodes.get(&node) else {
                            continue;
                        };
                        if cull.is_some() && !visible_nodes.contains(&node) {
                            continue;
                        }
                        let is_selected = selected.contains(&node);
                        let title = presenter.node_title(graph, node);
                        let (inputs, outputs) = node_ports(graph, node);
                        let pin_rows = inputs.len().max(outputs.len());
                        let body = presenter.node_body_label(graph, node);
                        let resize_handles =
                            presenter.node_resize_handles(graph, node, &this.style);
                        out.nodes.push((
                            node,
                            node_geom.rect,
                            is_selected,
                            title,
                            body,
                            pin_rows,
                            resize_handles,
                        ));
                    }

                    for (&port_id, handle) in &geom.ports {
                        out.port_centers.insert(port_id, handle.center);
                        if cull.is_some() && !visible_nodes.contains(&handle.node) {
                            continue;
                        }
                        let max_w = graph
                            .ports
                            .get(&port_id)
                            .and_then(|p| geom.nodes.get(&p.node))
                            .map(|node| {
                                let screen_w = node.rect.size.width.0 * zoom;
                                let screen_max = (screen_w - label_overhead).max(0.0);
                                Px(screen_max / zoom)
                            })
                            .unwrap_or_else(|| {
                                let screen_max = (this.style.node_width - label_overhead).max(0.0);
                                Px(screen_max / zoom)
                            });
                        out.port_labels.insert(
                            port_id,
                            PortLabelRender {
                                label: presenter.port_label(graph, port_id),
                                dir: handle.dir,
                                max_width: max_w,
                            },
                        );
                        let color = presenter.port_color(graph, port_id, &this.style);
                        out.pins.push((port_id, handle.bounds, color));
                    }

                    for (&edge_id, edge) in &graph.edges {
                        if this
                            .interaction
                            .wire_drag
                            .as_ref()
                            .is_some_and(|w| Self::wire_drag_suppresses_edge(&w.kind, edge_id))
                        {
                            continue;
                        }
                        let Some(from) = out.port_centers.get(&edge.from).copied() else {
                            continue;
                        };
                        let Some(to) = out.port_centers.get(&edge.to).copied() else {
                            continue;
                        };
                        let hint = this.edge_render_hint(graph, edge_id).normalized();
                        if let Some(c) = cull {
                            let pad = (snapshot
                                .interaction
                                .edge_interaction_width
                                .max(this.style.wire_width * this.style.wire_width_selected_mul)
                                .max(this.style.wire_width * this.style.wire_width_hover_mul))
                                / zoom;
                            let bounds = edge_bounds_rect(hint.route, from, to, zoom, pad);
                            if !rects_intersect(bounds, c) {
                                continue;
                            }
                        }
                        let mut color = presenter.edge_color(graph, edge_id, &this.style);
                        if let Some(override_color) = hint.color {
                            color = override_color;
                        }
                        let selected = selected_edges.contains(&edge_id);
                        let hovered = hovered_edge == Some(edge_id);
                        out.edges.push(EdgeRender {
                            id: edge_id,
                            from,
                            to,
                            color,
                            hint,
                            selected,
                            hovered,
                        });
                    }

                    out
                })
                .unwrap_or_default()
        };

        let edge_anchor_target_id = self
            .interaction
            .focused_edge
            .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]))
            .filter(|edge_id| {
                self.graph
                    .read_ref(cx.app, |g| {
                        let edge = g.edges.get(edge_id)?;
                        let (allow_source, allow_target) =
                            Self::edge_reconnectable_flags(edge, &snapshot.interaction);
                        Some(allow_source || allow_target)
                    })
                    .ok()
                    .flatten()
                    .unwrap_or(false)
            });
        let edge_anchor_target: Option<(EdgeRouteKind, Point, Point, Color)> =
            edge_anchor_target_id.and_then(|id| {
                render
                    .edges
                    .iter()
                    .find(|e| e.id == id)
                    .map(|e| (e.hint.route, e.from, e.to, e.color))
            });

        // Groups render under edges and nodes (container frames).
        if !render.groups.is_empty() {
            let mut group_text_style = self.style.context_menu_text_style.clone();
            group_text_style.size = Px(group_text_style.size.0 / zoom);
            if let Some(lh) = group_text_style.line_height.as_mut() {
                lh.0 /= zoom;
            }

            let group_pad = 10.0 / zoom;
            let group_corner = Px(10.0 / zoom);
            for (rect, title, selected) in &render.groups {
                let border_color = if *selected {
                    self.style.node_border_selected
                } else {
                    self.style.group_border
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: *rect,
                    background: self.style.group_background,
                    border: Edges::all(Px(1.0 / zoom)),
                    border_color,
                    corner_radii: Corners::all(group_corner),
                });

                if !title.is_empty() {
                    let max_w = (rect.size.width.0 - 2.0 * group_pad).max(0.0);
                    let constraints = TextConstraints {
                        max_width: Some(Px(max_w)),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        scale_factor: cx.scale_factor * zoom,
                    };
                    let (blob, metrics) = self.paint_cache.text_blob(
                        cx.services,
                        title.clone(),
                        &group_text_style,
                        constraints,
                    );

                    let text_x = Px(rect.origin.x.0 + group_pad);
                    let text_y = Px(rect.origin.y.0 + group_pad + metrics.baseline.0);
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(1),
                        origin: Point::new(text_x, text_y),
                        text: blob,
                        color: self.style.context_menu_text,
                    });
                }
            }
        }

        #[derive(Debug, Clone)]
        struct EdgePaint {
            from: Point,
            to: Point,
            color: Color,
            width: f32,
            route: EdgeRouteKind,
            start_marker: Option<crate::ui::presenter::EdgeMarker>,
            end_marker: Option<crate::ui::presenter::EdgeMarker>,
        }

        let mut edges_normal: Vec<EdgePaint> = Vec::new();
        let mut edges_selected: Vec<EdgePaint> = Vec::new();
        let mut edges_hovered: Vec<EdgePaint> = Vec::new();
        let mut edge_labels: Vec<(Point, Point, EdgeRouteKind, Arc<str>, bool, bool)> = Vec::new();

        for edge in render.edges {
            let mut width = self.style.wire_width * edge.hint.width_mul.max(0.0);
            if edge.selected {
                width *= self.style.wire_width_selected_mul;
            }
            if edge.hovered {
                width *= self.style.wire_width_hover_mul;
            }

            let route = edge.hint.route;
            if let Some(label) = edge.hint.label.as_ref().filter(|s| !s.is_empty()) {
                edge_labels.push((
                    edge.from,
                    edge.to,
                    route,
                    label.clone(),
                    edge.selected,
                    edge.hovered,
                ));
            }

            let paint = EdgePaint {
                from: edge.from,
                to: edge.to,
                color: edge.color,
                width,
                route,
                start_marker: edge.hint.start_marker.clone(),
                end_marker: edge.hint.end_marker.clone(),
            };

            if edge.hovered {
                edges_hovered.push(paint);
            } else if edge.selected {
                edges_selected.push(paint);
            } else {
                edges_normal.push(paint);
            }
        }

        for edge in edges_normal
            .into_iter()
            .chain(edges_selected)
            .chain(edges_hovered)
        {
            let path = self.paint_cache.wire_path(
                cx.services,
                edge.route,
                edge.from,
                edge.to,
                zoom,
                cx.scale_factor,
                edge.width,
            );

            if let Some(path) = path {
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color: edge.color,
                });
            }

            if let Some(marker) = edge.end_marker.as_ref() {
                if let Some(path) = self.paint_cache.edge_end_marker_path(
                    cx.services,
                    edge.route,
                    edge.from,
                    edge.to,
                    zoom,
                    cx.scale_factor,
                    marker,
                    self.style.pin_radius,
                ) {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color: edge.color,
                    });
                }
            }

            if let Some(marker) = edge.start_marker.as_ref() {
                if let Some(path) = self.paint_cache.edge_start_marker_path(
                    cx.services,
                    edge.route,
                    edge.from,
                    edge.to,
                    zoom,
                    cx.scale_factor,
                    marker,
                    self.style.pin_radius,
                ) {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color: edge.color,
                    });
                }
            }
        }

        self.paint_cache.prune(cx.services, 300, 30_000);

        if !edge_labels.is_empty() {
            let pad_screen = 6.0;
            let corner_screen = 8.0;
            let offset_screen = 10.0;

            let mut edge_text_style = self.style.context_menu_text_style.clone();
            edge_text_style.size = Px(edge_text_style.size.0 / zoom);
            if let Some(lh) = edge_text_style.line_height.as_mut() {
                lh.0 /= zoom;
            }

            for (from, to, route, label, _selected, _hovered) in edge_labels {
                let (pos, normal) = match route {
                    EdgeRouteKind::Bezier => {
                        let (c1, c2) = wire_ctrl_points(from, to, zoom);
                        let p = cubic_bezier(from, c1, c2, to, 0.5);
                        let d = cubic_bezier_derivative(from, c1, c2, to, 0.5);
                        (p, normal_from_tangent(d))
                    }
                    EdgeRouteKind::Straight => {
                        let p = Point::new(
                            Px(0.5 * (from.x.0 + to.x.0)),
                            Px(0.5 * (from.y.0 + to.y.0)),
                        );
                        let d = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));
                        (p, normal_from_tangent(d))
                    }
                    EdgeRouteKind::Step => {
                        let mx = 0.5 * (from.x.0 + to.x.0);
                        let p = Point::new(Px(mx), Px(0.5 * (from.y.0 + to.y.0)));
                        (p, Point::new(Px(0.0), Px(-1.0)))
                    }
                };

                let z = zoom.max(1.0e-6);
                let off = offset_screen / z;
                let anchor = Point::new(
                    Px(pos.x.0 + normal.x.0 * off),
                    Px(pos.y.0 + normal.y.0 * off),
                );

                let max_w = 220.0 / z;
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    scale_factor: cx.scale_factor * zoom,
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    cx.services,
                    label.clone(),
                    &edge_text_style,
                    constraints,
                );

                let pad = pad_screen / z;
                let w = metrics.size.width.0.max(0.0);
                let h = metrics.size.height.0.max(0.0);
                let rect = Rect::new(
                    Point::new(
                        Px(anchor.x.0 - 0.5 * w - pad),
                        Px(anchor.y.0 - 0.5 * h - pad),
                    ),
                    Size::new(Px(w + 2.0 * pad), Px(h + 2.0 * pad)),
                );

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(2),
                    rect,
                    background: self.style.context_menu_background,
                    border: Edges::all(Px(1.0 / z)),
                    border_color: self.style.context_menu_border,
                    corner_radii: Corners::all(Px(corner_screen / z)),
                });

                let text_x = Px(rect.origin.x.0 + pad);
                let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(2),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }
        }

        if let Some(w) = &self.interaction.wire_drag {
            let focused_target =
                focused_port.filter(|_| focused_port_valid || focused_port_convertible);
            let to = hovered_port
                .filter(|_| hovered_port_valid || hovered_port_convertible)
                .or(focused_target)
                .and_then(|port| render.port_centers.get(&port).copied())
                .unwrap_or(w.pos);
            let color =
                if hovered_port.is_some() && !hovered_port_valid && !hovered_port_convertible {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 0.95,
                    }
                } else if hovered_port.is_some() && hovered_port_convertible && !hovered_port_valid
                {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 0.95,
                    }
                } else if focused_port.is_some()
                    && !focused_port_valid
                    && !focused_port_convertible
                    && hovered_port.is_none()
                {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 0.95,
                    }
                } else if focused_port.is_some()
                    && focused_port_convertible
                    && !focused_port_valid
                    && hovered_port.is_none()
                {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 0.95,
                    }
                } else {
                    self.style.wire_color_preview
                };

            let mut draw_preview = |from: Point| {
                if let Some(path) = self.paint_cache.wire_path(
                    cx.services,
                    EdgeRouteKind::Bezier,
                    from,
                    to,
                    zoom,
                    cx.scale_factor,
                    self.style.wire_width,
                ) {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            };

            match &w.kind {
                WireDragKind::New { from, bundle } => {
                    let ports = if bundle.is_empty() {
                        std::slice::from_ref(from)
                    } else {
                        bundle.as_slice()
                    };
                    for port in ports {
                        if let Some(from) = render.port_centers.get(port).copied() {
                            draw_preview(from);
                        }
                    }
                }
                WireDragKind::Reconnect { fixed, .. } => {
                    if let Some(from) = render.port_centers.get(fixed).copied() {
                        draw_preview(from);
                    }
                }
                WireDragKind::ReconnectMany { edges } => {
                    for (_edge, _endpoint, fixed) in edges {
                        if let Some(from) = render.port_centers.get(fixed).copied() {
                            draw_preview(from);
                        }
                    }
                }
            }
        }

        let mut node_text_style = self.style.context_menu_text_style.clone();
        node_text_style.size = Px(node_text_style.size.0 / zoom);
        if let Some(lh) = node_text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let corner = Px(8.0 / zoom);
        let title_pad = self.style.node_padding / zoom;
        let title_h = self.style.node_header_height / zoom;

        for (node, rect, is_selected, title, body, pin_rows, resize_handles) in &render.nodes {
            let rect = *rect;
            let border_color = if *is_selected {
                self.style.node_border_selected
            } else {
                self.style.node_border
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: self.style.node_background,
                border: Edges::all(Px(1.0 / zoom)),
                border_color,
                corner_radii: Corners::all(corner),
            });

            let show_resize_handle = *is_selected
                && (self
                    .interaction
                    .node_resize
                    .as_ref()
                    .is_some_and(|r| r.node == *node)
                    || self
                        .interaction
                        .last_pos
                        .is_some_and(|p| Self::rect_contains(rect, p)));
            if show_resize_handle {
                for handle in NodeResizeHandle::ALL {
                    if !resize_handles.contains(handle) {
                        continue;
                    }
                    let rect = self.node_resize_handle_rect(rect, handle, zoom);
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect,
                        background: self.style.resize_handle_background,
                        border: Edges::all(Px(1.0 / zoom)),
                        border_color: self.style.resize_handle_border,
                        corner_radii: Corners::all(Px(2.0 / zoom)),
                    });
                }
            }

            if !title.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor * zoom,
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    cx.services,
                    title.clone(),
                    &node_text_style,
                    constraints,
                );

                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
                let text_y = Px(inner_y + metrics.baseline.0);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }

            if let Some(body) = body
                && !body.is_empty()
            {
                let pin_rows = (*pin_rows).max(0) as f32;
                let body_top = rect.origin.y.0
                    + (self.style.node_header_height
                        + self.style.node_padding
                        + pin_rows * self.style.pin_row_height
                        + self.style.node_padding)
                        / zoom;

                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor * zoom,
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    cx.services,
                    body.clone(),
                    &node_text_style,
                    constraints,
                );

                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = body_top + metrics.baseline.0;
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, Px(inner_y)),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }
        }

        let pin_r = self.style.pin_radius / zoom;
        let pin_gap = 8.0 / zoom;

        for (port_id, info) in &render.port_labels {
            let Some(center) = render.port_centers.get(port_id).copied() else {
                continue;
            };
            let port_constraints = TextConstraints {
                max_width: Some(info.max_width),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor * zoom,
            };
            let (blob, metrics) = self.paint_cache.text_blob(
                cx.services,
                info.label.clone(),
                &node_text_style,
                port_constraints,
            );

            let y = Px(center.y.0 - 0.5 * metrics.size.height.0 + metrics.baseline.0);
            let x = match info.dir {
                PortDirection::In => Px(center.x.0 + pin_r + pin_gap),
                PortDirection::Out => Px(center.x.0 - pin_r - pin_gap - metrics.size.width.0),
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(4),
                origin: Point::new(x, y),
                text: blob,
                color: self.style.context_menu_text,
            });
        }

        for (port_id, rect, color) in render.pins {
            if marked_ports.contains(&port_id) {
                let pad = 5.0 / zoom;
                let marker_rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                    Size::new(
                        Px(rect.size.width.0 + 2.0 * pad),
                        Px(rect.size.height.0 + 2.0 * pad),
                    ),
                );
                let r = Px(0.5 * marker_rect.size.width.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: marker_rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(Px(1.0 / zoom)),
                    border_color: Color {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                        a: 0.55,
                    },
                    corner_radii: Corners::all(r),
                });
            }

            if hovered_port == Some(port_id) {
                let border_color = if hovered_port_valid {
                    color
                } else if hovered_port_convertible {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 1.0,
                    }
                } else {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 1.0,
                    }
                };
                let pad = 2.0 / zoom;
                let hover_rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                    Size::new(
                        Px(rect.size.width.0 + 2.0 * pad),
                        Px(rect.size.height.0 + 2.0 * pad),
                    ),
                );
                let r = Px(0.5 * hover_rect.size.width.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: hover_rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(Px(2.0 / zoom)),
                    border_color,
                    corner_radii: Corners::all(r),
                });
            }

            if hovered_port != Some(port_id) && focused_port == Some(port_id) {
                let border_color = if self.interaction.wire_drag.is_some() {
                    if focused_port_valid {
                        color
                    } else if focused_port_convertible {
                        Color {
                            r: 0.95,
                            g: 0.75,
                            b: 0.20,
                            a: 1.0,
                        }
                    } else {
                        Color {
                            r: 0.90,
                            g: 0.35,
                            b: 0.35,
                            a: 1.0,
                        }
                    }
                } else {
                    self.style.node_border_selected
                };

                let pad = 2.0 / zoom;
                let hover_rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                    Size::new(
                        Px(rect.size.width.0 + 2.0 * pad),
                        Px(rect.size.height.0 + 2.0 * pad),
                    ),
                );
                let r = Px(0.5 * hover_rect.size.width.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: hover_rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(Px(2.0 / zoom)),
                    border_color,
                    corner_radii: Corners::all(r),
                });
            }

            let r = Px(0.5 * rect.size.width.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect,
                background: color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(r),
            });
        }

        if let Some((route, from, to, color)) = edge_anchor_target {
            let (a0, a1) = Self::edge_focus_anchor_centers(route, from, to, zoom);
            let target_edge_id = edge_anchor_target_id;
            let (allow_from, allow_to) = target_edge_id
                .and_then(|edge_id| {
                    self.graph
                        .read_ref(cx.app, |g| {
                            let edge = g.edges.get(&edge_id)?;
                            Some(Self::edge_reconnectable_flags(edge, &snapshot.interaction))
                        })
                        .ok()
                        .flatten()
                })
                .unwrap_or((false, false));

            let z = zoom.max(1.0e-6);
            let border_base = Px(Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN / z);
            let anchor_color = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: 0.95,
            };
            let fill_color = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: 0.15,
            };

            for (endpoint, center) in [(EdgeEndpoint::From, a0), (EdgeEndpoint::To, a1)] {
                if (endpoint == EdgeEndpoint::From && !allow_from)
                    || (endpoint == EdgeEndpoint::To && !allow_to)
                {
                    continue;
                }
                let rect = Self::edge_focus_anchor_rect(center, zoom);
                let r = Px(0.5 * rect.size.width.0);
                let hovered = self
                    .interaction
                    .hover_edge_anchor
                    .is_some_and(|(edge, ep)| Some(edge) == target_edge_id && ep == endpoint);
                let active = self
                    .interaction
                    .wire_drag
                    .as_ref()
                    .is_some_and(|w| match &w.kind {
                        WireDragKind::Reconnect {
                            edge, endpoint: ep, ..
                        } => Some(*edge) == target_edge_id && *ep == endpoint,
                        _ => false,
                    });

                let border = if active {
                    Px((Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN + 1.0) / z)
                } else if hovered {
                    Px((Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN + 0.5) / z)
                } else {
                    border_base
                };

                let background = if active {
                    Color {
                        a: (fill_color.a + 0.20).min(1.0),
                        ..fill_color
                    }
                } else if hovered {
                    Color {
                        a: (fill_color.a + 0.10).min(1.0),
                        ..fill_color
                    }
                } else {
                    fill_color
                };

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(6),
                    rect,
                    background,
                    border: Edges::all(border),
                    border_color: anchor_color,
                    corner_radii: Corners::all(r),
                });
            }
        }

        if self.close_command.is_some() {
            let rect = Self::close_button_rect(snapshot.pan, zoom);
            let hovered = self
                .interaction
                .last_pos
                .is_some_and(|p| Self::rect_contains(rect, p));

            let background = if hovered {
                self.style.context_menu_hover_background
            } else {
                self.style.context_menu_background
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(60),
                rect,
                background,
                border: Edges::all(Px(1.0 / zoom)),
                border_color: self.style.context_menu_border,
                corner_radii: Corners::all(Px(6.0 / zoom)),
            });

            let mut text_style = self.style.context_menu_text_style.clone();
            text_style.size = Px(text_style.size.0 / zoom);
            if let Some(lh) = text_style.line_height.as_mut() {
                lh.0 /= zoom;
            }
            let pad = 10.0 / zoom;
            let constraints = TextConstraints {
                max_width: Some(Px((rect.size.width.0 - 2.0 * pad).max(0.0))),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor * zoom,
            };
            let (blob, metrics) =
                self.paint_cache
                    .text_blob(cx.services, "Close", &text_style, constraints);

            let text_x = Px(rect.origin.x.0 + pad);
            let inner_y = rect.origin.y.0 + (rect.size.height.0 - metrics.size.height.0) * 0.5;
            let text_y = Px(inner_y + metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(61),
                origin: Point::new(text_x, text_y),
                text: blob,
                color: self.style.context_menu_text,
            });
        }

        if let Some(wire_drag) = wire_drag {
            self.paint_wire_drag_hint(cx, &snapshot, &wire_drag, zoom);
        }

        if let Some(marquee) = self.interaction.marquee.clone() {
            self.paint_marquee(cx, &marquee, zoom);
        }

        if let Some(guides) = self.interaction.snap_guides {
            self.paint_snap_guides(
                cx,
                &guides,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_w,
                viewport_h,
            );
        }

        if let Some(searcher) = self.interaction.searcher.clone() {
            self.paint_searcher(cx, &searcher, zoom);
        }

        if let Some(menu) = self.interaction.context_menu.clone() {
            self.paint_context_menu(cx, &menu, zoom);
        }

        if let Some(toast) = self.interaction.toast.clone() {
            self.paint_toast(
                cx,
                &toast,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_h,
            );
        }

        cx.scene.push(SceneOp::PopClip);
    }
}

fn context_menu_rect_at(
    style: &NodeGraphStyle,
    origin: Point,
    item_count: usize,
    zoom: f32,
) -> Rect {
    let w = style.context_menu_width / zoom;
    let item_h = style.context_menu_item_height / zoom;
    let pad = style.context_menu_padding / zoom;
    let h = (2.0 * pad + item_h * item_count.max(1) as f32).max(item_h + 2.0 * pad);
    Rect::new(origin, Size::new(Px(w), Px(h)))
}

fn searcher_visible_rows(searcher: &SearcherState) -> usize {
    searcher
        .rows
        .len()
        .saturating_sub(searcher.scroll)
        .min(SEARCHER_MAX_VISIBLE_ROWS)
}

fn searcher_rect_at(style: &NodeGraphStyle, origin: Point, row_count: usize, zoom: f32) -> Rect {
    let w = style.context_menu_width / zoom;
    let item_h = style.context_menu_item_height / zoom;
    let pad = style.context_menu_padding / zoom;

    let list_rows = row_count.max(1) as f32;
    let h = 3.0 * pad + item_h * (1.0 + list_rows);
    Rect::new(origin, Size::new(Px(w), Px(h)))
}

fn hit_searcher_row(
    style: &NodeGraphStyle,
    searcher: &SearcherState,
    pos: Point,
    zoom: f32,
) -> Option<usize> {
    let visible = searcher_visible_rows(searcher);
    let rect = searcher_rect_at(style, searcher.origin, visible, zoom);
    if !rect.contains(pos) {
        return None;
    }

    let pad = style.context_menu_padding / zoom;
    let item_h = style.context_menu_item_height / zoom;

    let list_top = rect.origin.y.0 + pad + item_h + pad;
    let y = pos.y.0 - list_top;
    if y < 0.0 {
        return None;
    }

    let slot = (y / item_h).floor() as isize;
    if slot < 0 {
        return None;
    }
    let slot = slot as usize;
    if slot >= visible {
        return None;
    }

    let row_ix = searcher.scroll + slot;
    (row_ix < searcher.rows.len()).then_some(row_ix)
}

fn rect_from_points(a: Point, b: Point) -> Rect {
    let min_x = a.x.0.min(b.x.0);
    let min_y = a.y.0.min(b.y.0);
    let max_x = a.x.0.max(b.x.0);
    let max_y = a.y.0.max(b.y.0);
    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

fn rect_union(a: Rect, b: Rect) -> Rect {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = a.origin.x.0 + a.size.width.0;
    let ay1 = a.origin.y.0 + a.size.height.0;

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = b.origin.x.0 + b.size.width.0;
    let by1 = b.origin.y.0 + b.size.height.0;

    let min_x = ax0.min(bx0);
    let min_y = ay0.min(by0);
    let max_x = ax1.max(bx1);
    let max_y = ay1.max(by1);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = a.origin.x.0 + a.size.width.0;
    let ay1 = a.origin.y.0 + a.size.height.0;

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = b.origin.x.0 + b.size.width.0;
    let by1 = b.origin.y.0 + b.size.height.0;

    ax0 <= bx1 && ax1 >= bx0 && ay0 <= by1 && ay1 >= by0
}

fn inflate_rect(rect: Rect, margin: f32) -> Rect {
    if !margin.is_finite() || margin <= 0.0 {
        return rect;
    }
    Rect::new(
        Point::new(Px(rect.origin.x.0 - margin), Px(rect.origin.y.0 - margin)),
        Size::new(
            Px(rect.size.width.0 + 2.0 * margin),
            Px(rect.size.height.0 + 2.0 * margin),
        ),
    )
}

fn edge_bounds_rect(route: EdgeRouteKind, from: Point, to: Point, zoom: f32, pad: f32) -> Rect {
    let mut min_x = from.x.0.min(to.x.0);
    let mut min_y = from.y.0.min(to.y.0);
    let mut max_x = from.x.0.max(to.x.0);
    let mut max_y = from.y.0.max(to.y.0);

    if route == EdgeRouteKind::Bezier {
        let (c1, c2) = wire_ctrl_points(from, to, zoom);
        min_x = min_x.min(c1.x.0).min(c2.x.0);
        min_y = min_y.min(c1.y.0).min(c2.y.0);
        max_x = max_x.max(c1.x.0).max(c2.x.0);
        max_y = max_y.max(c1.y.0).max(c2.y.0);
    }

    let pad = if pad.is_finite() { pad.max(0.0) } else { 0.0 };

    Rect::new(
        Point::new(Px(min_x - pad), Px(min_y - pad)),
        Size::new(
            Px((max_x - min_x) + 2.0 * pad),
            Px((max_y - min_y) + 2.0 * pad),
        ),
    )
}

fn hit_context_menu_item(
    style: &NodeGraphStyle,
    menu: &ContextMenuState,
    pos: Point,
    zoom: f32,
) -> Option<usize> {
    let rect = context_menu_rect_at(style, menu.origin, menu.items.len(), zoom);
    if !rect.contains(pos) {
        return None;
    }

    let pad = style.context_menu_padding / zoom;
    let item_h = style.context_menu_item_height / zoom;
    let inner_top = rect.origin.y.0 + pad;
    let y = pos.y.0 - inner_top;
    if y < 0.0 {
        return None;
    }

    let ix = (y / item_h).floor() as isize;
    if ix < 0 {
        return None;
    }
    let ix = ix as usize;
    (ix < menu.items.len()).then_some(ix)
}

fn wire_distance2(p: Point, from: Point, to: Point, zoom: f32) -> f32 {
    let (c1, c2) = wire_ctrl_points(from, to, zoom);

    let steps: usize = 24;
    let mut best = f32::INFINITY;

    let mut prev = from;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let cur = cubic_bezier(from, c1, c2, to, t);
        best = best.min(dist2_point_to_segment(p, prev, cur));
        prev = cur;
    }

    best
}

fn step_wire_distance2(p: Point, from: Point, to: Point) -> f32 {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let p1 = Point::new(Px(mx), from.y);
    let p2 = Point::new(Px(mx), to.y);
    let d0 = dist2_point_to_segment(p, from, p1);
    let d1 = dist2_point_to_segment(p, p1, p2);
    let d2 = dist2_point_to_segment(p, p2, to);
    d0.min(d1).min(d2)
}

fn dist2_point_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let apx = p.x.0 - a.x.0;
    let apy = p.y.0 - a.y.0;
    let abx = b.x.0 - a.x.0;
    let aby = b.y.0 - a.y.0;

    let ab2 = abx * abx + aby * aby;
    if ab2 <= 1.0e-9 {
        return apx * apx + apy * apy;
    }

    let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
    let cx = a.x.0 + t * abx;
    let cy = a.y.0 + t * aby;
    let dx = p.x.0 - cx;
    let dy = p.y.0 - cy;
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButton, MouseButtons, Point, PointerEvent, Px, Rect,
        Size, TextBlobId,
    };
    use fret_runtime::CommandId;
    use fret_runtime::ui_host::{
        CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
    };
    use fret_runtime::{ClipboardToken, CommandRegistry, DragKind, DragSession, Effect, FrameId};
    use fret_runtime::{ModelHost, ModelStore, TickId, TimerToken};
    use fret_ui::retained_bridge::Widget as _;
    use serde_json::Value;
    use std::any::{Any, TypeId};
    use std::collections::{HashMap, HashSet};
    use std::time::Instant;

    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group,
        GroupId, Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey,
        PortKind,
    };

    mod callbacks_conformance;
    mod connect_conformance;
    mod connection_mode_conformance;
    mod hit_testing_conformance;
    mod interaction_conformance;
    mod perf_cache;
    mod portal_conformance;
    mod portal_keyboard_conformance;
    mod portal_pointer_passthrough_conformance;

    #[test]
    fn inflate_rect_expands_by_margin() {
        let rect = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let inflated = super::inflate_rect(rect, 5.0);
        assert_eq!(
            inflated,
            Rect::new(Point::new(Px(5.0), Px(15.0)), Size::new(Px(40.0), Px(50.0)))
        );
    }

    #[test]
    fn edge_bounds_rect_applies_padding() {
        let from = Point::new(Px(10.0), Px(10.0));
        let to = Point::new(Px(30.0), Px(20.0));
        let bounds = super::edge_bounds_rect(
            crate::ui::presenter::EdgeRouteKind::Straight,
            from,
            to,
            1.0,
            2.0,
        );
        assert_eq!(
            bounds,
            Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(24.0), Px(14.0)))
        );
    }

    #[test]
    fn middle_mouse_panning_tracks_screen_delta_under_render_transform() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let mut snapshot = canvas.sync_view_state(cx.app);
        assert_eq!(snapshot.zoom, 1.0);
        assert_eq!(snapshot.pan, CanvasPoint::default());

        canvas.interaction.panning = true;
        canvas.interaction.pan_last_sample_at = Some(Instant::now());
        canvas.interaction.pan_last_screen_pos = None;

        let screen_positions = [
            Point::new(Px(100.0), Px(100.0)),
            Point::new(Px(140.0), Px(100.0)),
            Point::new(Px(190.0), Px(100.0)),
        ];

        for screen in screen_positions {
            let zoom = snapshot.zoom;
            let pan = snapshot.pan;
            let local = Point::new(
                Px((screen.x.0 - bounds.origin.x.0) / zoom - pan.x),
                Px((screen.y.0 - bounds.origin.y.0) / zoom - pan.y),
            );
            assert!(super::pan_zoom::handle_panning_move(
                &mut canvas,
                &mut cx,
                &snapshot,
                local,
            ));
            snapshot = canvas.sync_view_state(cx.app);
        }

        let expected_pan_x = screen_positions.last().unwrap().x.0 - screen_positions[0].x.0;
        assert!((snapshot.pan.x - expected_pan_x).abs() <= 1.0e-3);
        assert!((snapshot.pan.y - 0.0).abs() <= 1.0e-3);
    }

    #[test]
    fn space_to_pan_starts_left_mouse_panning_and_updates_viewport() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let mut snapshot = canvas.sync_view_state(cx.app);
        assert!(snapshot.interaction.space_to_pan);
        assert_eq!(snapshot.zoom, 1.0);
        assert_eq!(snapshot.pan, CanvasPoint::default());

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(canvas.interaction.pan_activation_key_held);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(100.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(canvas.interaction.panning);
        assert_eq!(canvas.interaction.panning_button, Some(MouseButton::Left));
        assert!(canvas.interaction.pending_marquee.is_none());
        assert!(canvas.interaction.marquee.is_none());

        let screen_positions = [
            Point::new(Px(100.0), Px(100.0)),
            Point::new(Px(140.0), Px(100.0)),
            Point::new(Px(190.0), Px(100.0)),
        ];
        for screen in screen_positions {
            let zoom = snapshot.zoom;
            let pan = snapshot.pan;
            let local = Point::new(
                Px((screen.x.0 - bounds.origin.x.0) / zoom - pan.x),
                Px((screen.y.0 - bounds.origin.y.0) / zoom - pan.y),
            );
            canvas.event(
                &mut cx,
                &Event::Pointer(PointerEvent::Move {
                    position: local,
                    buttons: MouseButtons {
                        left: true,
                        ..MouseButtons::default()
                    },
                    modifiers: Modifiers::default(),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
            snapshot = canvas.sync_view_state(cx.app);
        }

        let expected_pan_x = screen_positions.last().unwrap().x.0 - screen_positions[0].x.0;
        assert!((snapshot.pan.x - expected_pan_x).abs() <= 1.0e-3);
        assert!((snapshot.pan.y - 0.0).abs() <= 1.0e-3);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Up {
                position: *screen_positions.last().unwrap(),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(!canvas.interaction.panning);

        canvas.event(
            &mut cx,
            &Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );
        assert!(!canvas.interaction.pan_activation_key_held);
        assert_eq!(canvas.history.undo_len(), 0);
    }

    #[test]
    fn pan_activation_key_code_must_match_to_enable_space_to_pan() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.space_to_pan = true;
            s.interaction.pan_activation_key_code =
                Some(crate::io::NodeGraphKeyCode(fret_core::KeyCode::KeyP));
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let _snapshot = canvas.sync_view_state(cx.app);
        assert!(!canvas.interaction.pan_activation_key_held);

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(
            !canvas.interaction.pan_activation_key_held,
            "Space should not activate panning when pan_activation_key_code is KeyP"
        );

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::KeyP,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(canvas.interaction.pan_activation_key_held);

        canvas.event(
            &mut cx,
            &Event::KeyUp {
                key: fret_core::KeyCode::KeyP,
                modifiers: Modifiers::default(),
            },
        );
        assert!(!canvas.interaction.pan_activation_key_held);
    }

    #[test]
    fn pan_activation_key_code_none_disables_space_to_pan_activation() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.space_to_pan = true;
            s.interaction.pan_activation_key_code = None;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let _snapshot = canvas.sync_view_state(cx.app);

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(
            !canvas.interaction.pan_activation_key_held,
            "pan_activation_key_code=None should disable activation"
        );
    }

    #[test]
    fn pan_on_scroll_mode_horizontal_ignores_vertical_wheel_delta() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.pan_on_scroll = true;
            s.interaction.pan_on_scroll_speed = 1.0;
            s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Horizontal;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let before = canvas.sync_view_state(cx.app).pan;
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after = canvas.sync_view_state(cx.app).pan;
        assert_eq!(before, after);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(80.0), Px(0.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after2 = canvas.sync_view_state(cx.app).pan;
        assert!((after2.x - after.x - 80.0).abs() <= 1.0e-3);
        assert!((after2.y - after.y).abs() <= 1.0e-3);
    }

    #[test]
    fn pan_on_scroll_shift_maps_vertical_wheel_to_horizontal_on_windows() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.pan_on_scroll = true;
            s.interaction.pan_on_scroll_speed = 1.0;
            s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Free;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);
        cx.input_ctx.platform = fret_runtime::Platform::Windows;

        let before = canvas.sync_view_state(cx.app).pan;
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after = canvas.sync_view_state(cx.app).pan;
        assert!((after.x - before.x - 120.0).abs() <= 1.0e-3);
        assert!((after.y - before.y).abs() <= 1.0e-3);
    }

    #[test]
    fn space_enables_pan_on_scroll_even_when_pan_on_scroll_is_disabled() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.pan_on_scroll = false;
            s.interaction.pan_on_scroll_speed = 1.0;
            s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Free;
            s.interaction.zoom_on_scroll = false;
            s.interaction.space_to_pan = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let before = canvas.sync_view_state(cx.app).pan;
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after = canvas.sync_view_state(cx.app).pan;
        assert_eq!(before, after);

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(canvas.interaction.pan_activation_key_held);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after2 = canvas.sync_view_state(cx.app).pan;
        assert!((after2.y - after.y - 120.0).abs() <= 1.0e-3);
    }

    #[test]
    fn pinch_gesture_zooms_in_about_pointer() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_pinch = true;
            s.interaction.zoom_on_pinch_speed = 1.0;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let pos = Point::new(Px(100.0), Px(100.0));
        let before = canvas.sync_view_state(cx.app);
        assert_eq!(before.zoom, 1.0);
        assert_eq!(before.pan, CanvasPoint::default());

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::PinchGesture {
                position: pos,
                delta: 1.0,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert!((after.zoom - 2.0).abs() <= 1.0e-6);
        assert!((after.pan.x - -50.0).abs() <= 1.0e-3);
        assert!((after.pan.y - -50.0).abs() <= 1.0e-3);
    }

    #[test]
    fn pinch_gesture_respects_toggle() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_pinch = false;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::PinchGesture {
                position: Point::new(Px(100.0), Px(100.0)),
                delta: 1.0,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert_eq!(after.zoom, 1.0);
        assert_eq!(after.pan, CanvasPoint::default());
    }

    #[test]
    fn wheel_zoom_zooms_about_pointer() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_scroll = true;
            s.interaction.zoom_on_scroll_speed = 1.0;
            s.interaction.zoom_activation_key = crate::io::NodeGraphZoomActivationKey::None;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let pos = Point::new(Px(100.0), Px(100.0));
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                position: pos,
                delta: Point::new(Px(0.0), Px(-120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert!((after.zoom - 1.18).abs() <= 1.0e-4);
        assert!((after.pan.x - -15.254).abs() <= 1.0e-3);
        assert!((after.pan.y - -15.254).abs() <= 1.0e-3);
    }

    #[test]
    fn delete_key_defaults_to_backspace() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());
        let mut canvas = NodeGraphCanvas::new(graph, view);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        {
            let mut cx = event_cx(&mut host, &mut services, bounds);
            cx.window = Some(AppWindowId::default());
            canvas.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Delete,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }
        assert!(
            host.effects.is_empty(),
            "Delete should not dispatch delete-selection by default (XYFlow default is Backspace)"
        );

        {
            let mut cx = event_cx(&mut host, &mut services, bounds);
            cx.window = Some(AppWindowId::default());
            canvas.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Backspace,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }
        assert!(
            host.effects.iter().any(|e| matches!(
                e,
                Effect::Command { command, .. }
                    if *command == CommandId::from(crate::ui::commands::CMD_NODE_GRAPH_DELETE_SELECTION)
            )),
            "Backspace should dispatch delete-selection by default"
        );
    }

    #[test]
    fn disable_keyboard_a11y_does_not_block_delete_shortcut() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.disable_keyboard_a11y = true;
            s.interaction.delete_key = crate::io::NodeGraphDeleteKey::BackspaceOrDelete;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();

        {
            let mut cx = event_cx(&mut host, &mut services, bounds);
            cx.window = Some(AppWindowId::default());
            canvas.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Backspace,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }

        assert!(
            host.effects.iter().any(|e| matches!(
                e,
                Effect::Command { command, .. }
                    if *command == CommandId::from(crate::ui::commands::CMD_NODE_GRAPH_DELETE_SELECTION)
            )),
            "delete shortcut should still work when disable_keyboard_a11y is enabled (XYFlow parity)"
        );
    }

    #[test]
    fn disable_keyboard_a11y_blocks_tab_focus_traversal() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.disable_keyboard_a11y = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();

        {
            let mut cx = event_cx(&mut host, &mut services, bounds);
            cx.window = Some(AppWindowId::default());
            canvas.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Tab,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }

        assert!(
            host.effects.is_empty(),
            "Tab focus traversal should not dispatch focus commands when disable_keyboard_a11y is enabled"
        );
    }

    #[test]
    fn double_click_background_zooms_in_about_pointer() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_double_click = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let pos = Point::new(Px(600.0), Px(500.0));
        let before = canvas.sync_view_state(cx.app);
        assert_eq!(before.zoom, 1.0);
        assert_eq!(before.pan, CanvasPoint::default());

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert!((after.zoom - 2.0).abs() <= 1.0e-6);
        assert!((after.pan.x - -300.0).abs() <= 1.0e-3);
        assert!((after.pan.y - -250.0).abs() <= 1.0e-3);
        assert!(canvas.interaction.pending_marquee.is_none());
        assert!(canvas.interaction.marquee.is_none());
    }

    #[test]
    fn shift_double_click_background_zooms_out_about_pointer() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_double_click = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let pos = Point::new(Px(600.0), Px(500.0));
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert!((after.zoom - 0.5).abs() <= 1.0e-6);
        assert!((after.pan.x - 600.0).abs() <= 1.0e-3);
        assert!((after.pan.y - 500.0).abs() <= 1.0e-3);
    }
    use crate::rules::EdgeEndpoint;
    use crate::ui::commands::{
        CMD_NODE_GRAPH_ACTIVATE, CMD_NODE_GRAPH_ALIGN_LEFT, CMD_NODE_GRAPH_DELETE_SELECTION,
        CMD_NODE_GRAPH_FOCUS_NEXT, CMD_NODE_GRAPH_FOCUS_NEXT_PORT, CMD_NODE_GRAPH_FOCUS_PORT_LEFT,
        CMD_NODE_GRAPH_FOCUS_PORT_RIGHT, CMD_NODE_GRAPH_FOCUS_PREV, CMD_NODE_GRAPH_FOCUS_PREV_PORT,
        CMD_NODE_GRAPH_NUDGE_RIGHT, CMD_NODE_GRAPH_SELECT_ALL,
    };

    use super::super::state::{NodeDrag, ViewSnapshot, WireDrag, WireDragKind};
    use super::NodeGraphCanvas;

    #[derive(Default)]
    struct NullServices;

    impl fret_core::TextService for NullServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &fret_core::TextStyle,
            _constraints: fret_core::TextConstraints,
        ) -> (TextBlobId, fret_core::TextMetrics) {
            (
                TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl fret_core::PathService for NullServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for NullServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct TestUiHostImpl {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        redraw: HashSet<AppWindowId>,
        effects: Vec<Effect>,
        drag: Option<DragSession>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestUiHostImpl {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|b| b.downcast_ref::<T>())
        }

        fn global_mut<T: Any>(&mut self) -> Option<&mut T> {
            self.globals
                .get_mut(&TypeId::of::<T>())
                .and_then(|b| b.downcast_mut::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            if !self.globals.contains_key(&type_id) {
                self.globals.insert(type_id, Box::new(init()));
            }

            // Avoid aliasing `&mut self` by temporarily removing the value.
            let boxed = self
                .globals
                .remove(&type_id)
                .expect("global must exist")
                .downcast::<T>()
                .ok()
                .expect("global has wrong type");
            let mut value = *boxed;

            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestUiHostImpl {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for TestUiHostImpl {
        fn take_changed_models(&mut self) -> Vec<fret_runtime::ModelId> {
            self.models.take_changed_models()
        }
    }

    impl CommandsHost for TestUiHostImpl {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for TestUiHostImpl {
        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraw.insert(window);
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl TimeHost for TestUiHostImpl {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_timer_token(&mut self) -> TimerToken {
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            TimerToken(self.next_timer_token)
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            ClipboardToken(self.next_clipboard_token)
        }

        fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            fret_runtime::ImageUploadToken(self.next_image_upload_token)
        }
    }

    impl DragHost for TestUiHostImpl {
        fn drag(&self) -> Option<&DragSession> {
            self.drag.as_ref()
        }

        fn drag_mut(&mut self) -> Option<&mut DragSession> {
            self.drag.as_mut()
        }

        fn cancel_drag(&mut self) {
            self.drag = None;
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            _kind: DragKind,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            _kind: DragKind,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }
    }

    fn event_cx<'a>(
        host: &'a mut TestUiHostImpl,
        services: &'a mut NullServices,
        bounds: Rect,
    ) -> fret_ui::retained_bridge::EventCx<'a, TestUiHostImpl> {
        fret_ui::retained_bridge::EventCx {
            app: host,
            services,
            node: fret_core::NodeId::default(),
            window: None,
            input_ctx: fret_runtime::InputContext::default(),
            children: &[],
            focus: None,
            captured: None,
            bounds,
            invalidations: Vec::new(),
            requested_focus: None,
            requested_capture: None,
            requested_cursor: None,
            stop_propagation: false,
        }
    }

    fn command_cx<'a>(
        host: &'a mut TestUiHostImpl,
        services: &'a mut NullServices,
        tree: &'a mut fret_ui::UiTree<TestUiHostImpl>,
    ) -> fret_ui::retained_bridge::CommandCx<'a, TestUiHostImpl> {
        fret_ui::retained_bridge::CommandCx {
            app: host,
            services,
            tree,
            node: fret_core::NodeId::default(),
            window: None,
            input_ctx: fret_runtime::InputContext::default(),
            focus: None,
            invalidations: Vec::new(),
            requested_focus: None,
            stop_propagation: false,
        }
    }

    fn make_test_graph_two_nodes() -> (Graph, NodeId, NodeId) {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();

        graph.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 10.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        (graph, a, b)
    }

    fn make_test_graph_two_nodes_with_size() -> (Graph, NodeId, NodeId) {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();

        graph.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 10.0, y: 5.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        (graph, a, b)
    }

    fn make_test_graph_two_nodes_with_ports() -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let a_in = PortId::new();
        let a_out = PortId::new();
        graph.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![a_in, a_out],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            a_in,
            Port {
                node: a,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            a_out,
            Port {
                node: a,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        let b = NodeId::new();
        let b_in = PortId::new();
        graph.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 200.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![b_in],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            b_in,
            Port {
                node: b,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        (graph, a, a_in, a_out, b, b_in)
    }

    fn make_test_graph_two_nodes_with_ports_spaced_x(
        dx: f32,
    ) -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
        let (mut graph, a, a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
        graph
            .nodes
            .entry(b)
            .and_modify(|n| n.pos = CanvasPoint { x: dx, y: 0.0 });
        (graph, a, a_in, a_out, b, b_in)
    }

    fn read_node_pos(
        host: &mut TestUiHostImpl,
        model: &fret_runtime::Model<Graph>,
        id: NodeId,
    ) -> CanvasPoint {
        model
            .read_ref(host, |g| g.nodes.get(&id).map(|n| n.pos))
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    #[test]
    fn distance_sq_point_to_rect_is_zero_inside_and_positive_outside() {
        let rect = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(100.0), Px(50.0)),
        );
        let inside = Point::new(Px(50.0), Px(40.0));
        assert!(NodeGraphCanvas::rect_contains_point(rect, inside));
        assert_eq!(
            NodeGraphCanvas::distance_sq_point_to_rect(inside, rect),
            0.0
        );

        let outside = Point::new(Px(0.0), Px(0.0));
        assert!(!NodeGraphCanvas::rect_contains_point(rect, outside));
        assert!(NodeGraphCanvas::distance_sq_point_to_rect(outside, rect) > 0.0);
    }

    #[test]
    fn yank_edges_from_port_returns_all_incident_edges() {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let n1 = NodeId::new();
        let p_out = PortId::new();
        graph.nodes.insert(
            n1,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![p_out],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_out,
            Port {
                node: n1,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        let n2 = NodeId::new();
        let p_in1 = PortId::new();
        let p_in2 = PortId::new();
        graph.nodes.insert(
            n2,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![p_in1, p_in2],
                data: Value::Null,
            },
        );
        for (id, key) in [(p_in1, "in1"), (p_in2, "in2")] {
            graph.ports.insert(
                id,
                Port {
                    node: n2,
                    key: PortKey::new(key),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: None,
                    data: Value::Null,
                },
            );
        }

        let e1 = EdgeId::new();
        let e2 = EdgeId::new();
        graph.edges.insert(
            e1,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in1,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            e2,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in2,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let from_edges = NodeGraphCanvas::yank_edges_from_port(&graph, p_out);
        assert_eq!(from_edges.len(), 2);
        assert!(from_edges.contains(&(e1, EdgeEndpoint::From, p_in1)));
        assert!(from_edges.contains(&(e2, EdgeEndpoint::From, p_in2)));

        let to_edges = NodeGraphCanvas::yank_edges_from_port(&graph, p_in1);
        assert_eq!(to_edges, vec![(e1, EdgeEndpoint::To, p_out)]);
    }

    #[test]
    fn should_add_bundle_port_requires_same_side_and_dedupes() {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");
        let n1 = NodeId::new();

        let p_out1 = PortId::new();
        let p_out2 = PortId::new();
        let p_in = PortId::new();

        graph.nodes.insert(
            n1,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![p_out1, p_out2, p_in],
                data: Value::Null,
            },
        );

        graph.ports.insert(
            p_out1,
            Port {
                node: n1,
                key: PortKey::new("out1"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_out2,
            Port {
                node: n1,
                key: PortKey::new("out2"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_in,
            Port {
                node: n1,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        assert!(NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[p_out1],
            p_out2
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[p_out2],
            p_out2
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[],
            p_out1
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[],
            p_in
        ));
    }

    #[test]
    fn node_drag_records_single_history_entry_for_multi_node_move() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
        let snapshot = canvas.sync_view_state(&mut host);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        canvas.interaction.node_drag = Some(NodeDrag {
            primary: a,
            node_ids: vec![a, b],
            nodes: vec![
                (a, CanvasPoint { x: 0.0, y: 0.0 }),
                (b, CanvasPoint { x: 10.0, y: 0.0 }),
            ],
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_pos: Point::new(Px(0.0), Px(0.0)),
        });

        for pos in [
            Point::new(Px(20.0), Px(5.0)),
            Point::new(Px(40.0), Px(10.0)),
            Point::new(Px(60.0), Px(10.0)),
        ] {
            let did = super::node_drag::handle_node_drag_move(
                &mut canvas,
                &mut cx,
                &snapshot,
                pos,
                fret_core::Modifiers::default(),
                snapshot.zoom,
            );
            assert!(did);
            assert_eq!(canvas.history.undo_len(), 0);
        }

        let did_up = super::pointer_up::handle_pointer_up(
            &mut canvas,
            &mut cx,
            &snapshot,
            Point::new(Px(60.0), Px(10.0)),
            fret_core::MouseButton::Left,
            1,
            fret_core::Modifiers::default(),
            snapshot.zoom,
        );
        assert!(did_up);
        assert_eq!(canvas.history.undo_len(), 1);

        assert!(canvas.undo_last(&mut host, None));
        assert_eq!(
            read_node_pos(&mut host, &graph, a),
            CanvasPoint { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            read_node_pos(&mut host, &graph, b),
            CanvasPoint { x: 10.0, y: 0.0 }
        );
    }

    #[test]
    fn connect_bundle_records_single_history_entry() {
        let mut host = TestUiHostImpl::default();
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let n1 = NodeId::new();
        let out1 = PortId::new();
        let out2 = PortId::new();
        graph.nodes.insert(
            n1,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![out1, out2],
                data: Value::Null,
            },
        );
        for (id, key) in [(out1, "out1"), (out2, "out2")] {
            graph.ports.insert(
                id,
                Port {
                    node: n1,
                    key: PortKey::new(key),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Multi,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: None,
                    data: Value::Null,
                },
            );
        }

        let n2 = NodeId::new();
        let inn = PortId::new();
        graph.nodes.insert(
            n2,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 100.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![inn],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            inn,
            Port {
                node: n2,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        let graph_model = host.models.insert(graph);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph_model.clone(), view);
        let snapshot: ViewSnapshot = canvas.sync_view_state(&mut host);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: out1,
                bundle: vec![out1, out2],
            },
            pos: Point::new(Px(0.0), Px(0.0)),
        });

        let did = super::wire_drag::handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot,
            snapshot.zoom,
            Some(inn),
        );
        assert!(did);
        assert_eq!(canvas.history.undo_len(), 1);
        let edges_len = graph_model
            .read_ref(&mut host, |g| g.edges.len())
            .unwrap_or(0);
        assert_eq!(edges_len, 2);

        assert!(canvas.undo_last(&mut host, None));
        let edges_len = graph_model
            .read_ref(&mut host, |g| g.edges.len())
            .unwrap_or(0);
        assert_eq!(edges_len, 0);
    }

    #[test]
    fn nudge_moves_selection_and_records_history_entry() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT)));
        assert_eq!(canvas.history.undo_len(), 1);
        assert_eq!(read_node_pos(&mut host, &graph, a).x, 1.0);
        assert_eq!(read_node_pos(&mut host, &graph, b).x, 11.0);

        assert!(canvas.undo_last(&mut host, None));
        assert_eq!(
            read_node_pos(&mut host, &graph, a),
            CanvasPoint { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            read_node_pos(&mut host, &graph, b),
            CanvasPoint { x: 10.0, y: 0.0 }
        );
    }

    #[test]
    fn select_all_selects_nodes_groups_and_edges_and_respects_edge_selectable() {
        let mut host = TestUiHostImpl::default();

        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();
        graph.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 10.0, y: 0.0 },
                selectable: Some(false),
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        let p_out = PortId::new();
        let p_in = PortId::new();
        graph.ports.insert(
            p_out,
            Port {
                node: a,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_in,
            Port {
                node: b,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.nodes.get_mut(&a).unwrap().ports.push(p_out);
        graph.nodes.get_mut(&b).unwrap().ports.push(p_in);

        let e_ok = EdgeId::new();
        let e_no = EdgeId::new();
        graph.edges.insert(
            e_ok,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            e_no,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in,
                selectable: Some(false),
                deletable: None,
                reconnectable: None,
            },
        );

        let g0 = GroupId::new();
        graph.groups.insert(
            g0,
            Group {
                title: "Group".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 80.0,
                    },
                },
                color: None,
            },
        );

        let graph = host.models.insert(graph);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.interaction.elements_selectable = true;
            s.interaction.edges_selectable = true;
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        canvas.interaction.focused_edge = Some(e_ok);
        canvas.interaction.focused_node = Some(a);
        canvas.interaction.focused_port = Some(p_out);
        canvas.interaction.focused_port_valid = true;
        canvas.interaction.focused_port_convertible = true;

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_SELECT_ALL)));

        assert!(canvas.interaction.focused_edge.is_none());
        assert!(canvas.interaction.focused_node.is_none());
        assert!(canvas.interaction.focused_port.is_none());
        assert!(!canvas.interaction.focused_port_valid);
        assert!(!canvas.interaction.focused_port_convertible);

        let mut selected_nodes = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        selected_nodes.sort();
        assert_eq!(selected_nodes, vec![a]);

        let mut selected_groups = view
            .read_ref(&host, |s| s.selected_groups.clone())
            .unwrap_or_default();
        selected_groups.sort();
        assert_eq!(selected_groups, vec![g0]);

        let mut selected_edges = view
            .read_ref(&host, |s| s.selected_edges.clone())
            .unwrap_or_default();
        selected_edges.sort();
        assert_eq!(selected_edges, vec![e_ok]);
    }

    #[test]
    fn delete_selection_respects_node_deletable_and_keeps_undeletable_selected() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, a, b) = make_test_graph_two_nodes();
        graph_value
            .nodes
            .get_mut(&a)
            .expect("node must exist")
            .deletable = Some(false);
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.interaction.nodes_deletable = true;
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION)));

        assert!(
            graph
                .read_ref(&mut host, |g| g.nodes.contains_key(&a))
                .unwrap_or(false)
        );
        assert!(
            !graph
                .read_ref(&mut host, |g| g.nodes.contains_key(&b))
                .unwrap_or(true)
        );

        let selected_nodes = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected_nodes, vec![a]);
        assert_eq!(canvas.history.undo_len(), 1);
    }

    #[test]
    fn delete_selection_respects_edge_deletable_and_keeps_undeletable_selected() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();

        let edge = EdgeId::new();
        graph_value.edges.insert(
            edge,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: Some(false),
                reconnectable: None,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes.clear();
            s.selected_groups.clear();
            s.selected_edges = vec![edge];
            s.interaction.edges_deletable = true;
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION)));

        assert_eq!(graph.read_ref(&mut host, |g| g.edges.len()).unwrap_or(0), 1);
        let selected_edges = view
            .read_ref(&host, |s| s.selected_edges.clone())
            .unwrap_or_default();
        assert_eq!(selected_edges, vec![edge]);
        assert_eq!(canvas.history.undo_len(), 0);
    }

    #[test]
    fn align_left_moves_selected_nodes_and_records_history_entry() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes_with_size();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ALIGN_LEFT)));
        assert_eq!(canvas.history.undo_len(), 1);
        assert_eq!(read_node_pos(&mut host, &graph, a).x, 0.0);
        assert_eq!(read_node_pos(&mut host, &graph, b).x, 0.0);

        assert!(canvas.undo_last(&mut host, None));
        assert_eq!(
            read_node_pos(&mut host, &graph, a),
            CanvasPoint { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            read_node_pos(&mut host, &graph, b),
            CanvasPoint { x: 10.0, y: 5.0 }
        );
    }

    #[test]
    fn focus_next_cycles_nodes_and_updates_selection() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.draw_order = vec![a, b];
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
        }
        let selected = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected, vec![a]);

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
        }
        let selected = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected, vec![b]);

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PREV)));
        }
        let selected = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected, vec![a]);
    }

    #[test]
    fn focus_next_skips_unselectable_nodes() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, a, b) = make_test_graph_two_nodes();
        graph_value
            .nodes
            .get_mut(&a)
            .expect("node exists")
            .selectable = Some(false);

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.interaction.elements_selectable = true;
            s.draw_order = vec![a, b];
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
        let selected = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected, vec![b]);
    }

    #[test]
    fn focus_next_port_cycles_ports_within_focused_node() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a];
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
        }
        assert_eq!(canvas.interaction.focused_port, Some(a_in));

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
        }
        assert_eq!(canvas.interaction.focused_port, Some(a_out));

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PREV_PORT)));
        }
        assert_eq!(canvas.interaction.focused_port, Some(a_in));
    }

    #[test]
    fn focus_next_port_filters_by_wire_direction() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a];
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: a_out,
                bundle: Vec::new(),
            },
            pos: Point::new(Px(0.0), Px(0.0)),
        });

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
        assert_eq!(canvas.interaction.focused_port, Some(a_in));
    }

    #[test]
    fn activate_starts_and_commits_wire_drag() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
        let graph_model = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph_model.clone(), view);
        canvas.sync_view_state(&mut host);

        canvas.interaction.focused_port = Some(a_out);

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ACTIVATE)));
        }
        assert!(canvas.interaction.wire_drag.is_some());
        assert!(canvas.interaction.click_connect);
        assert!(canvas.interaction.focused_port.is_none());

        canvas.interaction.focused_port = Some(b_in);
        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ACTIVATE)));
        }
        assert!(canvas.interaction.wire_drag.is_none());

        let edges_len = graph_model
            .read_ref(&mut host, |g| g.edges.len())
            .unwrap_or(0);
        assert_eq!(edges_len, 1);
    }

    #[test]
    fn focus_port_right_moves_to_neighbor_node() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _a_in, a_out, b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(500.0);
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        canvas.interaction.focused_port = Some(a_out);
        canvas.interaction.focused_node = None;

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PORT_RIGHT)));
        assert_eq!(canvas.interaction.focused_node, Some(b));
        assert_eq!(canvas.interaction.focused_port, Some(b_in));
    }

    #[test]
    fn focus_port_left_moves_back() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, _a_in, a_out, _b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(500.0);
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        canvas.interaction.focused_port = Some(b_in);

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PORT_LEFT)));
        assert_eq!(canvas.interaction.focused_node, Some(a));
        assert_eq!(canvas.interaction.focused_port, Some(a_out));
    }
}
