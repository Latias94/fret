//! Measured geometry storage and presenter wrapper.
//!
//! This module provides a small, UI-only mechanism to feed measured node sizes and port handle
//! bounds back into the node graph canvas without coupling the canvas to a specific layout engine.
//!
//! Conceptually this is similar to ReactFlow/XyFlow "node internals.handleBounds": the graph model
//! remains pure data, while measured sizes and handle bounds live as derived editor internals.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::TypeDesc;
use crate::core::{CanvasPoint, EdgeId, Graph, NodeId, NodeKindKey, PortId};
use crate::ops::GraphOp;
use crate::profile::GraphProfile;
use crate::rules::{ConnectPlan, EdgeEndpoint, InsertNodeTemplate};
use crate::ui::presenter::{
    InsertNodeCandidate, NodeGraphContextMenuItem, NodeGraphPresenter, PortAnchorHint,
};
use crate::ui::style::NodeGraphStyle;

/// Thread-safe store for measured geometry hints.
///
/// Stored values are in screen-space logical pixels (px), consistent with `PortAnchorHint`.
#[derive(Debug, Default)]
pub struct MeasuredGeometryStore {
    revision: AtomicU64,
    node_sizes_px: RwLock<BTreeMap<NodeId, (f32, f32)>>,
    port_anchors_px: RwLock<BTreeMap<PortId, PortAnchorHint>>,
}

impl MeasuredGeometryStore {
    pub fn new() -> Self {
        Self {
            revision: AtomicU64::new(1),
            node_sizes_px: RwLock::new(BTreeMap::new()),
            port_anchors_px: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn bump_revision(&self) -> u64 {
        let old = self.revision.fetch_add(1, Ordering::Relaxed);
        old.wrapping_add(1)
    }

    pub fn update(
        &self,
        f: impl FnOnce(&mut BTreeMap<NodeId, (f32, f32)>, &mut BTreeMap<PortId, PortAnchorHint>),
    ) -> u64 {
        let mut node_sizes = self.node_sizes_px.write().expect("poisoned lock");
        let mut anchors = self.port_anchors_px.write().expect("poisoned lock");
        f(&mut node_sizes, &mut anchors);
        self.bump_revision()
    }

    pub fn update_if_changed(
        &self,
        f: impl FnOnce(&mut BTreeMap<NodeId, (f32, f32)>, &mut BTreeMap<PortId, PortAnchorHint>) -> bool,
    ) -> Option<u64> {
        let mut node_sizes = self.node_sizes_px.write().expect("poisoned lock");
        let mut anchors = self.port_anchors_px.write().expect("poisoned lock");
        let changed = f(&mut node_sizes, &mut anchors);
        changed.then(|| self.bump_revision())
    }

    pub fn node_size_px(&self, node: NodeId) -> Option<(f32, f32)> {
        self.node_sizes_px
            .read()
            .ok()
            .and_then(|m| m.get(&node).copied())
    }

    pub fn port_anchor_px(&self, port: PortId) -> Option<PortAnchorHint> {
        self.port_anchors_px
            .read()
            .ok()
            .and_then(|m| m.get(&port).copied())
    }
}

/// Presenter wrapper that consults measured geometry before delegating to an inner presenter.
///
/// This allows hosts to "push" measured handle bounds into the editor without changing the core
/// `NodeGraphCanvas` implementation.
pub struct MeasuredNodeGraphPresenter<P> {
    inner: P,
    measured: Arc<MeasuredGeometryStore>,
}

impl<P> MeasuredNodeGraphPresenter<P> {
    pub fn new(inner: P, measured: Arc<MeasuredGeometryStore>) -> Self {
        Self { inner, measured }
    }

    pub fn measured(&self) -> &Arc<MeasuredGeometryStore> {
        &self.measured
    }

    pub fn inner(&self) -> &P {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut P {
        &mut self.inner
    }
}

impl<P: NodeGraphPresenter> NodeGraphPresenter for MeasuredNodeGraphPresenter<P> {
    fn geometry_revision(&self) -> u64 {
        self.inner.geometry_revision().max(self.measured.revision())
    }

    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        self.inner.node_title(graph, node)
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        self.inner.port_label(graph, port)
    }

    fn node_body_label(&self, graph: &Graph, node: NodeId) -> Option<Arc<str>> {
        self.inner.node_body_label(graph, node)
    }

    fn port_color(&self, graph: &Graph, port: PortId, style: &NodeGraphStyle) -> fret_core::Color {
        self.inner.port_color(graph, port, style)
    }

    fn edge_color(&self, graph: &Graph, edge: EdgeId, style: &NodeGraphStyle) -> fret_core::Color {
        self.inner.edge_color(graph, edge, style)
    }

    fn node_size_hint_px(
        &mut self,
        graph: &Graph,
        node: NodeId,
        style: &NodeGraphStyle,
    ) -> Option<(f32, f32)> {
        self.measured
            .node_size_px(node)
            .or_else(|| self.inner.node_size_hint_px(graph, node, style))
    }

    fn port_anchor_hint(
        &mut self,
        graph: &Graph,
        node: NodeId,
        port: PortId,
        style: &NodeGraphStyle,
    ) -> Option<PortAnchorHint> {
        self.measured
            .port_anchor_px(port)
            .or_else(|| self.inner.port_anchor_hint(graph, node, port, style))
    }

    fn list_insertable_nodes(&mut self, graph: &Graph) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes(graph)
    }

    fn plan_create_node(
        &mut self,
        graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        self.inner.plan_create_node(graph, candidate, at)
    }

    fn list_insertable_nodes_for_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes_for_edge(graph, edge)
    }

    fn plan_split_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        node_kind: &NodeKindKey,
        at: CanvasPoint,
    ) -> ConnectPlan {
        self.inner.plan_split_edge(graph, edge, node_kind, at)
    }

    fn plan_split_edge_candidate(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> ConnectPlan {
        self.inner
            .plan_split_edge_candidate(graph, edge, candidate, at)
    }

    fn fill_edge_context_menu(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
        out: &mut Vec<NodeGraphContextMenuItem>,
    ) {
        self.inner.fill_edge_context_menu(graph, edge, style, out)
    }

    fn on_edge_context_menu_action(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        action: u64,
    ) -> Option<Vec<GraphOp>> {
        self.inner.on_edge_context_menu_action(graph, edge, action)
    }

    fn plan_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
        self.inner.plan_connect(graph, a, b)
    }

    fn plan_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
    ) -> ConnectPlan {
        self.inner
            .plan_reconnect_edge(graph, edge, endpoint, new_port)
    }

    fn profile_mut(&mut self) -> Option<&mut dyn GraphProfile> {
        self.inner.profile_mut()
    }

    fn type_of_port(&self, graph: &Graph, port: PortId) -> Option<TypeDesc> {
        self.inner.type_of_port(graph, port)
    }

    fn can_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
        self.inner.can_connect(graph, a, b)
    }

    fn can_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
    ) -> ConnectPlan {
        self.inner
            .can_reconnect_edge(graph, edge, endpoint, new_port)
    }

    fn list_conversions(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
    ) -> Vec<InsertNodeTemplate> {
        self.inner.list_conversions(graph, from, to)
    }

    fn conversion_label(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        template: &InsertNodeTemplate,
    ) -> Arc<str> {
        self.inner.conversion_label(graph, from, to, template)
    }

    fn conversion_insert_position(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        default_at: CanvasPoint,
        template: &InsertNodeTemplate,
    ) -> CanvasPoint {
        self.inner
            .conversion_insert_position(graph, from, to, default_at, template)
    }
}

/// Presenter wrapper that uses measured geometry as a fallback.
///
/// This is useful for auto-measured internals maintained by the node-graph widget itself:
/// domain presenters keep full control, while the editor can still provide derived sizing/anchor
/// hints when the presenter returns `None`.
pub struct FallbackMeasuredNodeGraphPresenter<P> {
    inner: P,
    measured: Arc<MeasuredGeometryStore>,
}

impl<P> FallbackMeasuredNodeGraphPresenter<P> {
    pub fn new(inner: P, measured: Arc<MeasuredGeometryStore>) -> Self {
        Self { inner, measured }
    }
}

impl<P: NodeGraphPresenter> NodeGraphPresenter for FallbackMeasuredNodeGraphPresenter<P> {
    fn geometry_revision(&self) -> u64 {
        self.inner.geometry_revision().max(self.measured.revision())
    }

    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        self.inner.node_title(graph, node)
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        self.inner.port_label(graph, port)
    }

    fn node_body_label(&self, graph: &Graph, node: NodeId) -> Option<Arc<str>> {
        self.inner.node_body_label(graph, node)
    }

    fn port_color(&self, graph: &Graph, port: PortId, style: &NodeGraphStyle) -> fret_core::Color {
        self.inner.port_color(graph, port, style)
    }

    fn edge_color(&self, graph: &Graph, edge: EdgeId, style: &NodeGraphStyle) -> fret_core::Color {
        self.inner.edge_color(graph, edge, style)
    }

    fn node_size_hint_px(
        &mut self,
        graph: &Graph,
        node: NodeId,
        style: &NodeGraphStyle,
    ) -> Option<(f32, f32)> {
        self.inner
            .node_size_hint_px(graph, node, style)
            .or_else(|| self.measured.node_size_px(node))
    }

    fn port_anchor_hint(
        &mut self,
        graph: &Graph,
        node: NodeId,
        port: PortId,
        style: &NodeGraphStyle,
    ) -> Option<PortAnchorHint> {
        self.inner
            .port_anchor_hint(graph, node, port, style)
            .or_else(|| self.measured.port_anchor_px(port))
    }

    fn list_insertable_nodes(&mut self, graph: &Graph) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes(graph)
    }

    fn plan_create_node(
        &mut self,
        graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        self.inner.plan_create_node(graph, candidate, at)
    }

    fn list_insertable_nodes_for_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes_for_edge(graph, edge)
    }

    fn plan_split_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        node_kind: &NodeKindKey,
        at: CanvasPoint,
    ) -> ConnectPlan {
        self.inner.plan_split_edge(graph, edge, node_kind, at)
    }

    fn plan_split_edge_candidate(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> ConnectPlan {
        self.inner
            .plan_split_edge_candidate(graph, edge, candidate, at)
    }

    fn fill_edge_context_menu(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
        out: &mut Vec<NodeGraphContextMenuItem>,
    ) {
        self.inner.fill_edge_context_menu(graph, edge, style, out)
    }

    fn on_edge_context_menu_action(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        action: u64,
    ) -> Option<Vec<GraphOp>> {
        self.inner.on_edge_context_menu_action(graph, edge, action)
    }

    fn plan_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
        self.inner.plan_connect(graph, a, b)
    }

    fn plan_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
    ) -> ConnectPlan {
        self.inner
            .plan_reconnect_edge(graph, edge, endpoint, new_port)
    }

    fn profile_mut(&mut self) -> Option<&mut dyn GraphProfile> {
        self.inner.profile_mut()
    }

    fn type_of_port(&self, graph: &Graph, port: PortId) -> Option<TypeDesc> {
        self.inner.type_of_port(graph, port)
    }

    fn can_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
        self.inner.can_connect(graph, a, b)
    }

    fn can_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
    ) -> ConnectPlan {
        self.inner
            .can_reconnect_edge(graph, edge, endpoint, new_port)
    }

    fn list_conversions(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
    ) -> Vec<InsertNodeTemplate> {
        self.inner.list_conversions(graph, from, to)
    }

    fn conversion_label(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        template: &InsertNodeTemplate,
    ) -> Arc<str> {
        self.inner.conversion_label(graph, from, to, template)
    }

    fn conversion_insert_position(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        default_at: CanvasPoint,
        template: &InsertNodeTemplate,
    ) -> CanvasPoint {
        self.inner
            .conversion_insert_position(graph, from, to, default_at, template)
    }
}
