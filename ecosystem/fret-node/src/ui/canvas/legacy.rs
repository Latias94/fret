use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, MouseButton, PathCommand,
    PathConstraints, PathStyle, Point, Px, Rect, SceneOp, Size, StrokeStyle, TextBlobId,
    TextConstraints, TextOverflow, TextWrap, Transform2D,
};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::{UiHost, retained_bridge::*};

use crate::REROUTE_KIND;
use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, Graph, NodeId as GraphNodeId, NodeKindKey,
    PortDirection, PortId,
};
use crate::io::{NodeGraphConnectionMode, NodeGraphInteractionState, NodeGraphViewState};
use crate::ops::{
    GraphFragment, GraphHistory, GraphOp, GraphTransaction, IdRemapSeed, IdRemapper, PasteTuning,
    apply_transaction,
};
use crate::profile::{ApplyPipelineError, apply_transaction_with_profile};
use crate::rules::{ConnectDecision, Diagnostic, DiagnosticSeverity, EdgeEndpoint};

use crate::ui::commands::{
    CMD_NODE_GRAPH_COPY, CMD_NODE_GRAPH_CUT, CMD_NODE_GRAPH_DELETE_SELECTION,
    CMD_NODE_GRAPH_DUPLICATE, CMD_NODE_GRAPH_FRAME_SELECTION, CMD_NODE_GRAPH_INSERT_REROUTE,
    CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER, CMD_NODE_GRAPH_OPEN_INSERT_NODE,
    CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE, CMD_NODE_GRAPH_PASTE, CMD_NODE_GRAPH_REDO,
    CMD_NODE_GRAPH_SELECT_ALL, CMD_NODE_GRAPH_UNDO,
};
use crate::ui::presenter::{
    DefaultNodeGraphPresenter, InsertNodeCandidate, NodeGraphContextMenuAction,
    NodeGraphContextMenuItem, NodeGraphPresenter, PortAnchorHint,
};
use crate::ui::style::NodeGraphStyle;
use crate::ui::{
    FallbackMeasuredNodeGraphPresenter, MeasuredGeometryStore, NodeGraphCanvasTransform,
    NodeGraphEditQueue, NodeGraphInternalsSnapshot, NodeGraphInternalsStore,
};

mod cancel;
mod context_menu;
mod edge_drag;
mod marquee;
mod node_drag;
mod pointer_up;
mod right_click;
mod searcher;
mod sticky_wire;
mod wire_drag;

use super::conversion;
use super::geometry::{CanvasGeometry, node_ports};
use super::searcher::{SEARCHER_MAX_VISIBLE_ROWS, SearcherRow, SearcherRowKind};
use super::snaplines::SnapGuides;
use super::spatial::CanvasSpatialIndex;
use super::state::{
    ContextMenuState, ContextMenuTarget, EdgeDrag, GeometryCache, GeometryCacheKey,
    InteractionState, InternalsCacheKey, MarqueeDrag, NodeDrag, PendingNodeDrag, PendingPaste,
    SearcherState, ToastState, ViewSnapshot, WireDrag, WireDragKind,
};
use super::workflow;

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
    presenter: Box<dyn NodeGraphPresenter>,
    style: NodeGraphStyle,
    close_command: Option<CommandId>,

    auto_measured: Arc<MeasuredGeometryStore>,
    auto_measured_key: Option<(u64, u32)>,

    edit_queue: Option<Model<NodeGraphEditQueue>>,
    edit_queue_key: Option<u64>,

    measured_output: Option<Arc<MeasuredGeometryStore>>,
    measured_output_key: Option<GeometryCacheKey>,

    internals: Option<Arc<NodeGraphInternalsStore>>,
    internals_key: Option<InternalsCacheKey>,

    cached_pan: CanvasPoint,
    cached_zoom: f32,
    history: GraphHistory,
    geometry: GeometryCache,

    wire_paths: Vec<fret_core::PathId>,
    text_blobs: Vec<TextBlobId>,
    interaction: InteractionState,
}

impl NodeGraphCanvas {
    const REROUTE_INPUTS: usize = 1;
    const REROUTE_OUTPUTS: usize = 1;
    const AUTO_PAN_TICK_HZ: f32 = 60.0;

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

    fn toast_from_diagnostics(diags: &[Diagnostic]) -> Option<(DiagnosticSeverity, Arc<str>)> {
        let first = diags.first()?;
        if first.message.is_empty() {
            return None;
        }
        Some((first.severity, Arc::<str>::from(first.message.clone())))
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
            presenter: Box::new(FallbackMeasuredNodeGraphPresenter::new(
                DefaultNodeGraphPresenter::default(),
                auto_measured.clone(),
            )),
            style: NodeGraphStyle::default(),
            close_command: None,
            auto_measured,
            auto_measured_key: None,
            edit_queue: None,
            edit_queue_key: None,
            measured_output: None,
            measured_output_key: None,
            internals: None,
            internals_key: None,
            cached_pan: CanvasPoint::default(),
            cached_zoom: 1.0,
            history: GraphHistory::default(),
            geometry: GeometryCache::default(),
            wire_paths: Vec::new(),
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

    fn sync_view_state<H: UiHost>(&mut self, host: &mut H) -> ViewSnapshot {
        let mut snapshot = ViewSnapshot {
            pan: self.cached_pan,
            zoom: self.cached_zoom,
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            draw_order: Vec::new(),
            interaction: NodeGraphInteractionState::default(),
        };

        let _ = self.view_state.read(host, |_host, s| {
            snapshot.pan = s.pan;
            snapshot.zoom = s.zoom;
            snapshot.selected_nodes = s.selected_nodes.clone();
            snapshot.selected_edges = s.selected_edges.clone();
            snapshot.draw_order = s.draw_order.clone();
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
                cx.services
                    .text()
                    .measure(node.title.as_ref(), &text_style, constraints)
                    .size
                    .width
                    .0
            };
            let max_in = node
                .inputs
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    cx.services
                        .text()
                        .measure(s.as_ref(), &text_style, constraints)
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
                    cx.services
                        .text()
                        .measure(s.as_ref(), &text_style, constraints)
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
        let quant = |v: f32| (v / 0.25).round() * 0.25;

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

        let keep_nodes: std::collections::BTreeSet<GraphNodeId> =
            node_sizes.iter().map(|(id, _)| *id).collect();
        let keep_ports: std::collections::BTreeSet<PortId> =
            port_anchors.iter().map(|(id, _)| *id).collect();

        let _ = store.update_if_changed(|sizes, anchors| {
            let mut changed = false;

            sizes.retain(|id, _| {
                let ok = keep_nodes.contains(id);
                if !ok {
                    changed = true;
                }
                ok
            });
            anchors.retain(|id, _| {
                let ok = keep_ports.contains(id);
                if !ok {
                    changed = true;
                }
                ok
            });

            for (node, size) in &node_sizes {
                let needs = match sizes.get(node) {
                    Some(old) => (old.0 - size.0).abs() > 0.25 || (old.1 - size.1).abs() > 0.25,
                    None => true,
                };
                if needs {
                    sizes.insert(*node, *size);
                    changed = true;
                }
            }
            for (port, hint) in &port_anchors {
                let needs = match anchors.get(port) {
                    Some(old) => {
                        (old.center.x.0 - hint.center.x.0).abs() > 0.25
                            || (old.center.y.0 - hint.center.y.0).abs() > 0.25
                            || (old.bounds.origin.x.0 - hint.bounds.origin.x.0).abs() > 0.25
                            || (old.bounds.origin.y.0 - hint.bounds.origin.y.0).abs() > 0.25
                            || (old.bounds.size.width.0 - hint.bounds.size.width.0).abs() > 0.25
                            || (old.bounds.size.height.0 - hint.bounds.size.height.0).abs() > 0.25
                    }
                    None => true,
                };
                if needs {
                    anchors.insert(*port, *hint);
                    changed = true;
                }
            }

            changed
        });
    }

    fn update_view_state<H: UiHost>(
        &mut self,
        host: &mut H,
        f: impl FnOnce(&mut NodeGraphViewState),
    ) {
        let _ = self.view_state.update(host, |s, _cx| {
            f(s);
        });
        self.sync_view_state(host);
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

    fn apply_transaction_result<H: UiHost>(
        &mut self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<GraphTransaction, Vec<Diagnostic>> {
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
                self.history.record(committed);
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
        let mut history = std::mem::take(&mut self.history);
        let result = history.undo(|tx| self.apply_transaction_result(host, tx));
        self.history = history;

        match result {
            Ok(true) => {
                self.update_view_state(host, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                });
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
        let mut history = std::mem::take(&mut self.history);
        let result = history.redo(|tx| self.apply_transaction_result(host, tx));
        self.history = history;

        match result {
            Ok(true) => {
                self.update_view_state(host, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                });
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

    fn copy_selected_nodes_to_clipboard<H: UiHost>(
        &mut self,
        host: &mut H,
        selected_nodes: &[GraphNodeId],
    ) {
        if selected_nodes.is_empty() {
            return;
        }
        let nodes: Vec<GraphNodeId> = selected_nodes.to_vec();
        let text = self
            .graph
            .read_ref(host, |graph| {
                let fragment = GraphFragment::from_nodes(graph, nodes);
                match serde_json::to_string(&fragment) {
                    Ok(json) => format!("fret-node.fragment.v1\n{json}"),
                    Err(_) => String::new(),
                }
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
        let payload = text.strip_prefix("fret-node.fragment.v1\n").unwrap_or(text);
        let fragment: GraphFragment = match serde_json::from_str(payload) {
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

        if !self.apply_ops_result(host, window, tx.ops) {
            return;
        }

        if !new_nodes.is_empty() {
            self.update_view_state(host, |s| {
                s.selected_edges.clear();
                s.selected_nodes = new_nodes.clone();
                for id in &new_nodes {
                    s.draw_order.retain(|x| x != id);
                    s.draw_order.push(*id);
                }
            });
        }
    }

    fn duplicate_selection<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        selected_nodes: &[GraphNodeId],
    ) {
        if selected_nodes.is_empty() {
            return;
        }

        let fragment = self
            .graph
            .read_ref(host, |graph| {
                GraphFragment::from_nodes(graph, selected_nodes.to_vec())
            })
            .ok()
            .unwrap_or_default();

        let tuning = PasteTuning {
            offset: CanvasPoint { x: 24.0, y: 24.0 },
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

        if !self.apply_ops_result(host, window, tx.ops) {
            return;
        }

        if !new_nodes.is_empty() {
            self.update_view_state(host, |s| {
                s.selected_edges.clear();
                s.selected_nodes = new_nodes.clone();
                for id in &new_nodes {
                    s.draw_order.retain(|x| x != id);
                    s.draw_order.push(*id);
                }
            });
        }
    }

    fn delete_selection_ops(
        graph: &Graph,
        selected_nodes: &[GraphNodeId],
        selected_edges: &[EdgeId],
    ) -> Vec<GraphOp> {
        let mut ops: Vec<GraphOp> = Vec::new();
        let mut removed_edges: std::collections::BTreeSet<EdgeId> =
            std::collections::BTreeSet::new();

        let mut nodes: Vec<GraphNodeId> = selected_nodes.to_vec();
        nodes.sort();

        for node_id in nodes {
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

    fn node_order(&self, graph: &Graph, snapshot: &ViewSnapshot) -> Vec<GraphNodeId> {
        let mut seen: HashSet<GraphNodeId> = HashSet::new();
        let mut out: Vec<GraphNodeId> = Vec::new();

        for id in &snapshot.draw_order {
            if graph.nodes.contains_key(id) && seen.insert(*id) {
                out.push(*id);
            }
        }

        for id in graph.nodes.keys() {
            if seen.insert(*id) {
                out.push(*id);
            }
        }

        out
    }

    fn node_ports<'a>(&self, graph: &'a Graph, node: GraphNodeId) -> (Vec<PortId>, Vec<PortId>) {
        let Some(n) = graph.nodes.get(&node) else {
            return (Vec::new(), Vec::new());
        };
        let mut inputs: Vec<PortId> = Vec::new();
        let mut outputs: Vec<PortId> = Vec::new();
        for port_id in &n.ports {
            let Some(p) = graph.ports.get(port_id) else {
                continue;
            };
            match p.dir {
                PortDirection::In => inputs.push(*port_id),
                PortDirection::Out => outputs.push(*port_id),
            }
        }
        (inputs, outputs)
    }

    fn node_height(&self, inputs: usize, outputs: usize, zoom: f32) -> f32 {
        let rows = inputs.max(outputs) as f32;
        let base = self.style.node_header_height + 2.0 * self.style.node_padding;
        let pin_area = rows * self.style.pin_row_height;
        (base + pin_area) / zoom
    }

    fn node_rect(&self, graph: &Graph, node: GraphNodeId, zoom: f32) -> Option<Rect> {
        let n = graph.nodes.get(&node)?;
        let (inputs, outputs) = self.node_ports(graph, node);
        let w = self.style.node_width / zoom;
        let h = self.node_height(inputs.len(), outputs.len(), zoom);
        Some(Rect::new(
            Point::new(Px(n.pos.x), Px(n.pos.y)),
            Size::new(Px(w), Px(h)),
        ))
    }

    fn port_center(
        &self,
        graph: &Graph,
        node: GraphNodeId,
        port: PortId,
        zoom: f32,
    ) -> Option<Point> {
        let rect = self.node_rect(graph, node, zoom)?;
        let (inputs, outputs) = self.node_ports(graph, node);
        let Some(p) = graph.ports.get(&port) else {
            return None;
        };

        let row = match p.dir {
            PortDirection::In => inputs.iter().position(|id| *id == port)?,
            PortDirection::Out => outputs.iter().position(|id| *id == port)?,
        } as f32;

        let x = match p.dir {
            PortDirection::In => rect.origin.x.0,
            PortDirection::Out => rect.origin.x.0 + rect.size.width.0,
        };

        let y = rect.origin.y.0
            + (self.style.node_header_height + self.style.node_padding) / zoom
            + (row + 0.5) * (self.style.pin_row_height / zoom);

        Some(Point::new(Px(x), Px(y)))
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

        let r2 = r * r;
        let mut best: Option<(PortId, f32, u32)> = None;
        let eps = (1.0e-3 / zoom.max(1.0e-6)).max(1.0e-6);
        for &port_id in scratch.iter() {
            let Some(center) = geom.port_center(port_id) else {
                continue;
            };
            let dx = center.x.0 - pos.x.0;
            let dy = center.y.0 - pos.y.0;
            let d2 = dx * dx + dy * dy;
            if d2 > r2 {
                continue;
            }
            let rank = geom
                .ports
                .get(&port_id)
                .and_then(|h| geom.node_rank.get(&h.node).copied())
                .unwrap_or(0);
            match best {
                Some((_id, best_d2, best_rank)) => {
                    if d2 + eps < best_d2 {
                        best = Some((port_id, d2, rank));
                    } else if (d2 - best_d2).abs() <= eps && rank > best_rank {
                        best = Some((port_id, d2, rank));
                    }
                }
                None => best = Some((port_id, d2, rank)),
            }
        }

        best.map(|(id, _, _)| id)
    }

    fn pick_target_port(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        from: PortId,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<PortId>,
    ) -> Option<PortId> {
        let from_port = graph.ports.get(&from)?;
        let desired_dir = match from_port.dir {
            PortDirection::In => PortDirection::Out,
            PortDirection::Out => PortDirection::In,
        };

        match snapshot.interaction.connection_mode {
            NodeGraphConnectionMode::Strict => {
                let candidate = self.hit_port(geom, index, pos, zoom, scratch)?;
                let port = graph.ports.get(&candidate)?;
                (candidate != from && port.dir == desired_dir).then_some(candidate)
            }
            NodeGraphConnectionMode::Loose => {
                let radius_screen = snapshot.interaction.connection_radius;
                if !radius_screen.is_finite() || radius_screen <= 0.0 {
                    let candidate = self.hit_port(geom, index, pos, zoom, scratch)?;
                    let port = graph.ports.get(&candidate)?;
                    return (candidate != from && port.dir == desired_dir).then_some(candidate);
                }
                let r = radius_screen / zoom;
                let r2 = r * r;
                let eps = (1.0e-3 / zoom.max(1.0e-6)).max(1.0e-6);

                let mut best: Option<(PortId, f32, bool, u32)> = None;
                index.query_ports(pos, r, scratch);
                for &port_id in scratch.iter() {
                    if port_id == from {
                        continue;
                    }
                    let Some(handle) = geom.ports.get(&port_id) else {
                        continue;
                    };
                    let center = handle.center;
                    let dx = center.x.0 - pos.x.0;
                    let dy = center.y.0 - pos.y.0;
                    let d2 = dx * dx + dy * dy;
                    if d2 > r2 {
                        continue;
                    }
                    let prefer_opposite = handle.dir == desired_dir;
                    let rank = geom.node_rank.get(&handle.node).copied().unwrap_or(0);
                    match best {
                        Some((_id, best_d2, best_prefer, best_rank)) => {
                            if d2 + eps < best_d2 {
                                best = Some((port_id, d2, prefer_opposite, rank));
                            } else if (d2 - best_d2).abs() <= eps {
                                if prefer_opposite && !best_prefer {
                                    best = Some((port_id, d2, prefer_opposite, rank));
                                } else if prefer_opposite == best_prefer && rank > best_rank {
                                    best = Some((port_id, d2, prefer_opposite, rank));
                                }
                            }
                        }
                        None => best = Some((port_id, d2, prefer_opposite, rank)),
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

            let d2 = wire_distance2(pos, from, to, zoom);
            if d2 <= threshold2 {
                match best {
                    Some((_id, best_d2)) if best_d2 <= d2 => {}
                    _ => best = Some((edge_id, d2)),
                }
            }
        }

        best.map(|(id, _)| id)
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
            ty: None,
            data: serde_json::Value::Null,
        };

        let out_port = crate::core::Port {
            node: node_id,
            key: crate::core::PortKey::new("out"),
            dir: PortDirection::Out,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Multi,
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
        let candidates: Vec<InsertNodeCandidate> = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| {
                    presenter.list_insertable_nodes_for_edge(graph, edge)
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

        let rows =
            super::searcher::build_rows(&menu_candidates, "", &self.interaction.recent_kinds);

        if rows.is_empty() {
            self.show_toast(
                host,
                window,
                DiagnosticSeverity::Info,
                "no insertable nodes for edge",
            );
            return;
        }

        let snapshot = self.sync_view_state(host);
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
        let origin = self.clamp_searcher_origin(invoked_at, visible, bounds, &snapshot);
        let active_row = rows
            .iter()
            .position(|r| matches!(r.kind, SearcherRowKind::Candidate { .. }) && r.enabled)
            .unwrap_or(0);

        self.interaction.context_menu = None;
        self.interaction.searcher = Some(SearcherState {
            origin,
            invoked_at,
            target: ContextMenuTarget::EdgeInsertNodePicker(edge),
            query: String::new(),
            candidates: menu_candidates,
            recent_kinds: self.interaction.recent_kinds.clone(),
            rows,
            hovered_row: None,
            active_row,
            scroll: 0,
        });
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
                                presenter, graph, *from, insert_ops,
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
                        if self.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                            if let Some(node_id) = created_node {
                                self.update_view_state(cx.app, |s| {
                                    s.selected_edges.clear();
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
                                self.interaction.wire_drag = Some(WireDrag {
                                    kind: WireDragKind::New {
                                        from: port,
                                        bundle: Vec::new(),
                                    },
                                    pos: invoked_at,
                                });
                                self.interaction.sticky_wire = true;
                                self.interaction.sticky_wire_ignore_next_up = true;
                                self.interaction.hover_port = None;
                                self.interaction.hover_port_valid = false;
                                self.interaction.hover_port_convertible = false;
                                cx.capture_pointer(cx.node);
                            }
                        }
                    }
                    Outcome::Reject(sev, msg) => self.show_toast(cx.app, cx.window, sev, msg),
                    Outcome::Ignore => {}
                }
            }
            (
                ContextMenuTarget::Edge(edge_id),
                NodeGraphContextMenuAction::OpenInsertNodePicker,
            ) => {
                let candidates: Vec<InsertNodeCandidate> = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            presenter.list_insertable_nodes_for_edge(graph, *edge_id)
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

                let mut items: Vec<NodeGraphContextMenuItem> = Vec::new();
                for (ix, c) in menu_candidates.iter().enumerate() {
                    items.push(NodeGraphContextMenuItem {
                        label: c.label.clone(),
                        enabled: c.enabled,
                        action: NodeGraphContextMenuAction::InsertNodeCandidate(ix),
                    });
                }

                let snapshot = self.sync_view_state(cx.app);
                let origin =
                    self.clamp_context_menu_origin(invoked_at, items.len(), cx.bounds, &snapshot);

                let active_item = items.iter().position(|it| it.enabled).unwrap_or(0);
                self.interaction.context_menu = Some(ContextMenuState {
                    origin,
                    invoked_at,
                    target: ContextMenuTarget::EdgeInsertNodePicker(*edge_id),
                    items,
                    candidates: menu_candidates,
                    hovered_item: None,
                    active_item,
                    typeahead: String::new(),
                });
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
                enum Outcome {
                    Apply(Vec<GraphOp>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                }

                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);

                let outcome = {
                    let at = if candidate.kind.0 == REROUTE_KIND {
                        self.reroute_pos_for_invoked_at(invoked_at)
                    } else {
                        CanvasPoint {
                            x: invoked_at.x.0,
                            y: invoked_at.y.0,
                        }
                    };

                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let plan = presenter
                                .plan_split_edge_candidate(graph, *edge_id, &candidate, at);
                            match plan.decision {
                                ConnectDecision::Accept => Outcome::Apply(plan.ops),
                                ConnectDecision::Reject => {
                                    Self::toast_from_diagnostics(&plan.diagnostics)
                                        .map(|(sev, msg)| Outcome::Reject(sev, msg))
                                        .unwrap_or_else(|| {
                                            Outcome::Reject(
                                                DiagnosticSeverity::Error,
                                                Arc::<str>::from(format!(
                                                    "node insertion was rejected: {}",
                                                    candidate.kind.0
                                                )),
                                            )
                                        })
                                }
                            }
                        })
                        .ok()
                        .unwrap_or(Outcome::Ignore)
                };

                match outcome {
                    Outcome::Apply(ops) => {
                        let select_node = candidate.kind.0 == REROUTE_KIND;
                        let node_id = select_node
                            .then(|| Self::first_added_node_id(&ops))
                            .flatten();
                        self.apply_ops(cx.app, cx.window, ops);
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
                    Outcome::Reject(sev, msg) => self.show_toast(cx.app, cx.window, sev, msg),
                    Outcome::Ignore => {}
                }
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
                                None => return Outcome::Ignore,
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
                    Outcome::Reject(sev, msg) => self.show_toast(cx.app, cx.window, sev, msg),
                    Outcome::Ignore => {}
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

            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(item.label.as_ref(), &text_style, constraints);
            self.text_blobs.push(blob);

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
            cx.services
                .text()
                .prepare(query_text.as_ref(), &text_style, constraints);
        self.text_blobs.push(blob);
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

            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(row.label.as_ref(), &text_style, constraints);
            self.text_blobs.push(blob);

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

        let (blob, metrics) =
            cx.services
                .text()
                .prepare(toast.message.as_ref(), &text_style, constraints);
        self.text_blobs.push(blob);

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

        let (blob, metrics) = cx
            .services
            .text()
            .prepare(text.as_ref(), &text_style, constraints);
        self.text_blobs.push(blob);

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
        edge_id: EdgeId,
        pos: Point,
        reconnect_radius_screen: f32,
        zoom: f32,
    ) -> Option<(EdgeEndpoint, PortId)> {
        let edge = graph.edges.get(&edge_id)?;

        let from_center = {
            let from = graph.ports.get(&edge.from)?;
            self.port_center(graph, from.node, edge.from, zoom)
        };
        let to_center = {
            let to = graph.ports.get(&edge.to)?;
            self.port_center(graph, to.node, edge.to, zoom)
        };

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

    fn hit_node(
        &self,
        graph: &Graph,
        pos: Point,
        node_order: &[GraphNodeId],
        zoom: f32,
    ) -> Option<GraphNodeId> {
        for node in node_order.iter().rev() {
            let Some(rect) = self.node_rect(graph, *node, zoom) else {
                continue;
            };
            if rect.contains(pos) {
                return Some(*node);
            }
        }
        None
    }

    fn prepare_wire_path(
        services: &mut dyn fret_core::UiServices,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        width_px: f32,
    ) -> Option<fret_core::PathId> {
        let dx = to.x.0 - from.x.0;
        let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
        let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
        let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
        let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);

        let commands = [
            PathCommand::MoveTo(from),
            PathCommand::CubicTo {
                ctrl1: c1,
                ctrl2: c2,
                to,
            },
        ];

        let (id, _metrics) = services.path().prepare(
            &commands,
            PathStyle::Stroke(StrokeStyle {
                width: Px(width_px / zoom),
            }),
            PathConstraints {
                scale_factor: scale_factor * zoom,
            },
        );

        Some(id)
    }

    fn zoom_about_center(&mut self, bounds: Rect, delta_y: f32) {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }

        let speed = 0.0015;
        let factor = (1.0 + (-delta_y * speed)).clamp(0.2, 5.0);
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
}

impl<H: UiHost> Widget<H> for NodeGraphCanvas {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for id in self.wire_paths.drain(..) {
            services.path().release(id);
        }
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
                let at = self.interaction.last_canvas_pos.unwrap_or_default();
                self.open_insert_node_picker(cx.app, at);
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
            CMD_NODE_GRAPH_SELECT_ALL => {
                let nodes = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        graph.nodes.keys().copied().collect::<Vec<_>>()
                    })
                    .ok()
                    .unwrap_or_default();
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes = nodes;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_COPY => {
                self.copy_selected_nodes_to_clipboard(cx.app, &snapshot.selected_nodes);
                true
            }
            CMD_NODE_GRAPH_CUT => {
                self.copy_selected_nodes_to_clipboard(cx.app, &snapshot.selected_nodes);

                let selected_nodes = snapshot.selected_nodes.clone();
                let selected_edges = snapshot.selected_edges.clone();
                let remove_ops = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        Self::delete_selection_ops(graph, &selected_nodes, &selected_edges)
                    })
                    .ok()
                    .unwrap_or_default();
                let _ = self.commit_ops(cx.app, cx.window, Some("Cut"), remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                });

                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_PASTE => {
                let at = self.interaction.last_canvas_pos.unwrap_or_default();
                self.request_paste_at_canvas(cx.app, cx.window, at);
                true
            }
            CMD_NODE_GRAPH_DUPLICATE => {
                self.duplicate_selection(cx.app, cx.window, &snapshot.selected_nodes);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_DELETE_SELECTION => {
                let selected_edges = snapshot.selected_edges.clone();
                let selected_nodes = snapshot.selected_nodes.clone();
                if selected_edges.is_empty() && selected_nodes.is_empty() {
                    return true;
                }

                let remove_ops = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        Self::delete_selection_ops(graph, &selected_nodes, &selected_edges)
                    })
                    .ok()
                    .unwrap_or_default();

                let _ = self.commit_ops(cx.app, cx.window, Some("Delete Selection"), remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            _ => false,
        }
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
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                if modifiers.ctrl || modifiers.meta {
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

                if !matches!(
                    key,
                    fret_core::KeyCode::Delete | fret_core::KeyCode::Backspace
                ) {
                    return;
                }

                cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION));
                cx.stop_propagation();
                return;
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button,
                modifiers,
                ..
            }) => {
                self.interaction.last_pos = Some(*position);
                self.interaction.last_canvas_pos = Some(Self::screen_to_canvas(
                    cx.bounds,
                    *position,
                    snapshot.pan,
                    zoom,
                ));

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

                if self.interaction.context_menu.is_some()
                    && context_menu::handle_context_menu_pointer_down(
                        self, cx, *position, *button, zoom,
                    )
                {
                    return;
                }

                if sticky_wire::handle_sticky_wire_pointer_down(
                    self, cx, &snapshot, *position, *button, zoom,
                ) {
                    return;
                }

                if *button == MouseButton::Middle {
                    self.interaction.hover_edge = None;
                    self.interaction.pending_node_drag = None;
                    self.interaction.node_drag = None;
                    self.interaction.wire_drag = None;
                    self.interaction.edge_drag = None;
                    self.interaction.panning = true;
                    cx.capture_pointer(cx.node);
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
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

                self.interaction.hover_edge = None;

                #[derive(Debug, Clone, Copy)]
                enum Hit {
                    Port(PortId),
                    Node(GraphNodeId, Rect),
                    Edge(EdgeId),
                    Background,
                }

                let hit = {
                    let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            let mut scratch_ports: Vec<PortId> = Vec::new();
                            let mut scratch_edges: Vec<EdgeId> = Vec::new();
                            if let Some(port) = this.hit_port(
                                geom.as_ref(),
                                index.as_ref(),
                                *position,
                                zoom,
                                &mut scratch_ports,
                            ) {
                                return Hit::Port(port);
                            }
                            let order = this.node_order(graph, &snapshot);
                            let Some(node) = this.hit_node(graph, *position, &order, zoom) else {
                                if let Some(edge) = this.hit_edge(
                                    graph,
                                    &snapshot,
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_edges,
                                ) {
                                    return Hit::Edge(edge);
                                }
                                return Hit::Background;
                            };
                            let Some(rect) = this.node_rect(graph, node, zoom) else {
                                return Hit::Background;
                            };
                            Hit::Node(node, rect)
                        })
                        .unwrap_or(Hit::Background)
                };

                match hit {
                    Hit::Port(port) => {
                        self.interaction.pending_node_drag = None;
                        self.interaction.node_drag = None;
                        self.interaction.edge_drag = None;
                        self.interaction.pending_marquee = None;
                        self.interaction.marquee = None;
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;
                        let yank = (modifiers.ctrl || modifiers.meta).then(|| {
                            let this = &*self;
                            this.graph
                                .read_ref(cx.app, |graph| Self::yank_edges_from_port(graph, port))
                                .ok()
                                .unwrap_or_default()
                        });

                        let kind = match yank {
                            Some(edges) if edges.len() > 1 => WireDragKind::ReconnectMany { edges },
                            Some(mut edges) if edges.len() == 1 => {
                                let (edge, endpoint, fixed) = edges.remove(0);
                                WireDragKind::Reconnect {
                                    edge,
                                    endpoint,
                                    fixed,
                                }
                            }
                            _ => WireDragKind::New {
                                from: port,
                                bundle: vec![port],
                            },
                        };

                        self.interaction.wire_drag = Some(WireDrag {
                            kind,
                            pos: *position,
                        });
                        cx.capture_pointer(cx.node);
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    Hit::Node(node, rect) => {
                        self.interaction.pending_node_drag = None;
                        self.interaction.node_drag = None;
                        self.interaction.wire_drag = None;
                        self.interaction.edge_drag = None;
                        self.interaction.pending_marquee = None;
                        self.interaction.marquee = None;
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;
                        let offset = Point::new(
                            Px(position.x.0 - rect.origin.x.0),
                            Px(position.y.0 - rect.origin.y.0),
                        );
                        let already_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
                        let multi_mod = modifiers.shift || modifiers.ctrl || modifiers.meta;

                        self.update_view_state(cx.app, |s| {
                            s.selected_edges.clear();
                            if multi_mod {
                                if let Some(ix) = s.selected_nodes.iter().position(|id| *id == node)
                                {
                                    s.selected_nodes.remove(ix);
                                } else {
                                    s.selected_nodes.push(node);
                                }
                            } else if !s.selected_nodes.iter().any(|id| *id == node) {
                                s.selected_nodes.clear();
                                s.selected_nodes.push(node);
                            }
                            s.draw_order.retain(|id| *id != node);
                            s.draw_order.push(node);
                        });

                        if !multi_mod {
                            let nodes_for_drag = (already_selected
                                && snapshot.selected_nodes.len() > 1)
                                .then(|| snapshot.selected_nodes.clone())
                                .unwrap_or_else(|| vec![node]);
                            self.interaction.pending_node_drag = Some(PendingNodeDrag {
                                primary: node,
                                nodes: nodes_for_drag,
                                grab_offset: offset,
                                start_pos: *position,
                            });
                            cx.capture_pointer(cx.node);
                        }

                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    Hit::Edge(edge) => {
                        self.interaction.pending_node_drag = None;
                        self.interaction.node_drag = None;
                        self.interaction.wire_drag = None;
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;
                        let multi = modifiers.ctrl || modifiers.meta;
                        self.update_view_state(cx.app, |s| {
                            s.selected_nodes.clear();
                            if multi {
                                if let Some(ix) = s.selected_edges.iter().position(|id| *id == edge)
                                {
                                    s.selected_edges.remove(ix);
                                } else {
                                    s.selected_edges.push(edge);
                                }
                            } else {
                                s.selected_edges.clear();
                                s.selected_edges.push(edge);
                            }
                        });
                        self.interaction.edge_drag = Some(EdgeDrag {
                            edge,
                            start_pos: *position,
                        });
                        cx.capture_pointer(cx.node);
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    Hit::Background => {
                        self.interaction.edge_drag = None;
                        self.interaction.pending_node_drag = None;
                        self.interaction.node_drag = None;
                        self.interaction.wire_drag = None;
                        self.interaction.pending_marquee = None;
                        self.interaction.marquee = None;
                        self.interaction.hover_port = None;
                        self.interaction.hover_port_valid = false;
                        self.interaction.hover_port_convertible = false;
                        marquee::begin_background_marquee(
                            self, cx, &snapshot, *position, *modifiers,
                        );
                    }
                }
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position,
                modifiers,
                ..
            }) => {
                let Some(last) = self.interaction.last_pos else {
                    self.interaction.last_pos = Some(*position);
                    self.interaction.last_canvas_pos = Some(Self::screen_to_canvas(
                        cx.bounds,
                        *position,
                        snapshot.pan,
                        zoom,
                    ));
                    return;
                };
                let delta = Point::new(Px(position.x.0 - last.x.0), Px(position.y.0 - last.y.0));
                self.interaction.last_pos = Some(*position);
                self.interaction.last_canvas_pos = Some(Self::screen_to_canvas(
                    cx.bounds,
                    *position,
                    snapshot.pan,
                    zoom,
                ));

                if self.close_command.is_some()
                    && self.interaction.node_drag.is_none()
                    && self.interaction.wire_drag.is_none()
                    && self.interaction.edge_drag.is_none()
                    && !self.interaction.panning
                {
                    let rect = Self::close_button_rect(snapshot.pan, zoom);
                    if Self::rect_contains(rect, *position) {
                        cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
                    }
                }

                if self.interaction.panning {
                    self.update_view_state(cx.app, |s| {
                        s.pan.x += delta.x.0;
                        s.pan.y += delta.y.0;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if marquee::handle_marquee_move(self, cx, &snapshot, *position, *modifiers, zoom) {
                    return;
                }

                if self.interaction.node_drag.is_none() {
                    if let Some(pending) = self.interaction.pending_node_drag.clone() {
                        let threshold_screen = snapshot.interaction.node_drag_threshold.max(0.0);
                        let threshold_graph = threshold_screen / zoom;
                        let dx = position.x.0 - pending.start_pos.x.0;
                        let dy = position.y.0 - pending.start_pos.y.0;
                        if threshold_graph <= 0.0
                            || dx * dx + dy * dy >= threshold_graph * threshold_graph
                        {
                            self.interaction.pending_node_drag = None;
                            let start_nodes = self
                                .graph
                                .read_ref(cx.app, |g| {
                                    pending
                                        .nodes
                                        .iter()
                                        .copied()
                                        .filter_map(|id| g.nodes.get(&id).map(|n| (id, n.pos)))
                                        .collect::<Vec<_>>()
                                })
                                .ok()
                                .unwrap_or_default();
                            self.interaction.node_drag = Some(NodeDrag {
                                primary: pending.primary,
                                nodes: start_nodes,
                                grab_offset: pending.grab_offset,
                                start_pos: pending.start_pos,
                            });
                        } else {
                            return;
                        }
                    }
                }

                if node_drag::handle_node_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    return;
                }

                if wire_drag::handle_wire_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    return;
                }

                if edge_drag::handle_edge_drag_move(self, cx, &snapshot, *position, zoom) {
                    return;
                }

                if searcher::handle_searcher_pointer_move(self, cx, *position, zoom) {
                    return;
                }

                if context_menu::handle_context_menu_pointer_move(self, cx, *position, zoom) {
                    return;
                }

                let new_hover = {
                    let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
                    let this = &*self;
                    let index = index.clone();
                    this.graph
                        .read_ref(cx.app, |graph| {
                            let mut scratch: Vec<EdgeId> = Vec::new();
                            this.hit_edge(
                                graph,
                                &snapshot,
                                geom.as_ref(),
                                index.as_ref(),
                                *position,
                                zoom,
                                &mut scratch,
                            )
                        })
                        .ok()
                        .flatten()
                };

                if self.interaction.hover_edge != new_hover {
                    self.interaction.hover_edge = new_hover;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position, button, ..
            }) => {
                if pointer_up::handle_pointer_up(self, cx, &snapshot, *position, *button, zoom) {
                    return;
                }
            }
            Event::Pointer(fret_core::PointerEvent::Wheel {
                delta, modifiers, ..
            }) => {
                if searcher::handle_searcher_wheel(self, cx, *delta, *modifiers, zoom) {
                    return;
                }

                if modifiers.ctrl || modifiers.meta {
                    let delta_screen_y = delta.y.0 * zoom;
                    self.zoom_about_center(cx.bounds, delta_screen_y);
                    let pan = self.cached_pan;
                    let zoom = self.cached_zoom;
                    self.update_view_state(cx.app, |s| {
                        s.pan = pan;
                        s.zoom = zoom;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                } else {
                    self.update_view_state(cx.app, |s| {
                        s.pan.x += delta.x.0;
                        s.pan.y += delta.y.0;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
        let snapshot = self.sync_view_state(cx.app);

        for id in self.wire_paths.drain(..) {
            cx.services.path().release(id);
        }
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        let zoom = snapshot.zoom;
        let pan = snapshot.pan;

        let viewport_w = cx.bounds.size.width.0 / zoom;
        let viewport_h = cx.bounds.size.height.0 / zoom;
        let viewport_origin_x = -pan.x;
        let viewport_origin_y = -pan.y;

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
            edges: Vec<(EdgeId, Point, Point, Color, bool, bool)>,
            nodes: Vec<(GraphNodeId, Rect, bool, Arc<str>, Option<Arc<str>>, usize)>,
            pins: Vec<(PortId, Rect, Color)>,
            port_labels: HashMap<PortId, PortLabelRender>,
            port_centers: HashMap<PortId, Point>,
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
            let this = &*self;
            let geom = geom.clone();
            let presenter: &dyn NodeGraphPresenter = &*this.presenter;
            this.graph
                .read_ref(cx.app, |graph| {
                    let mut out = RenderData::default();

                    let geom = geom.as_ref();
                    let node_pad = this.style.node_padding;
                    let pin_gap = 8.0;
                    let pin_r = this.style.pin_radius;
                    let label_overhead = 2.0 * node_pad + 2.0 * (pin_r + pin_gap);

                    for node in geom.order.iter().copied() {
                        let Some(node_geom) = geom.nodes.get(&node) else {
                            continue;
                        };
                        let is_selected = selected.contains(&node);
                        let title = presenter.node_title(graph, node);
                        let (inputs, outputs) = node_ports(graph, node);
                        let pin_rows = inputs.len().max(outputs.len());
                        let body = presenter.node_body_label(graph, node);
                        out.nodes
                            .push((node, node_geom.rect, is_selected, title, body, pin_rows));
                    }

                    for (&port_id, handle) in &geom.ports {
                        out.port_centers.insert(port_id, handle.center);
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
                        let color = presenter.edge_color(graph, edge_id, &this.style);
                        out.edges.push((
                            edge_id,
                            from,
                            to,
                            color,
                            selected_edges.contains(&edge_id),
                            hovered_edge == Some(edge_id),
                        ));
                    }

                    out
                })
                .unwrap_or_default()
        };

        let mut edges_normal: Vec<(Point, Point, Color, f32)> = Vec::new();
        let mut edges_selected: Vec<(Point, Point, Color, f32)> = Vec::new();
        let mut edges_hovered: Vec<(Point, Point, Color, f32)> = Vec::new();

        for (_edge_id, from, to, color, selected, hovered) in render.edges {
            let mut width = self.style.wire_width;
            if selected {
                width *= self.style.wire_width_selected_mul;
            }
            if hovered {
                width *= self.style.wire_width_hover_mul;
            }

            if hovered {
                edges_hovered.push((from, to, color, width));
            } else if selected {
                edges_selected.push((from, to, color, width));
            } else {
                edges_normal.push((from, to, color, width));
            }
        }

        for (from, to, color, width) in edges_normal
            .into_iter()
            .chain(edges_selected)
            .chain(edges_hovered)
        {
            if let Some(path) =
                Self::prepare_wire_path(cx.services, from, to, zoom, cx.scale_factor, width)
            {
                self.wire_paths.push(path);
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color,
                });
            }
        }

        if let Some(w) = &self.interaction.wire_drag {
            let to = hovered_port
                .filter(|_| hovered_port_valid || hovered_port_convertible)
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
                } else {
                    self.style.wire_color_preview
                };

            let mut draw_preview = |from: Point| {
                if let Some(path) = Self::prepare_wire_path(
                    cx.services,
                    from,
                    to,
                    zoom,
                    cx.scale_factor,
                    self.style.wire_width,
                ) {
                    self.wire_paths.push(path);
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

        for (_node, rect, is_selected, title, body, pin_rows) in &render.nodes {
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

            if !title.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor * zoom,
                };
                let (blob, metrics) =
                    cx.services
                        .text()
                        .prepare(title.as_ref(), &node_text_style, constraints);
                self.text_blobs.push(blob);

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
                let (blob, metrics) =
                    cx.services
                        .text()
                        .prepare(body.as_ref(), &node_text_style, constraints);
                self.text_blobs.push(blob);

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
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(info.label.as_ref(), &node_text_style, port_constraints);
            self.text_blobs.push(blob);

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
            let (blob, metrics) = cx
                .services
                .text()
                .prepare("Close", &text_style, constraints);
            self.text_blobs.push(blob);

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

fn wire_ctrl_points(from: Point, to: Point, zoom: f32) -> (Point, Point) {
    let dx = to.x.0 - from.x.0;
    let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
    let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
    let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
    let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);
    (c1, c2)
}

fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    let w0 = mt2 * mt;
    let w1 = 3.0 * mt2 * t;
    let w2 = 3.0 * mt * t2;
    let w3 = t2 * t;

    Point::new(
        Px(w0 * p0.x.0 + w1 * p1.x.0 + w2 * p2.x.0 + w3 * p3.x.0),
        Px(w0 * p0.y.0 + w1 * p1.y.0 + w2 * p2.y.0 + w3 * p3.y.0),
    )
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
    use serde_json::Value;

    use crate::core::{
        CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
        PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };
    use crate::rules::EdgeEndpoint;

    use super::NodeGraphCanvas;

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
            },
        );
        graph.edges.insert(
            e2,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in2,
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
}
