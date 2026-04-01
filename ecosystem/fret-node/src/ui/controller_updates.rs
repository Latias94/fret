use super::*;

/// Editable node snapshot used by `update_node*`.
///
/// Structural port ownership/order is intentionally excluded. Use explicit transactions for port
/// graph edits.
#[derive(Debug, Clone)]
pub struct NodeGraphNodeUpdate {
    pub kind: NodeKindKey,
    pub kind_version: u32,
    pub pos: CanvasPoint,
    pub selectable: Option<bool>,
    pub draggable: Option<bool>,
    pub connectable: Option<bool>,
    pub deletable: Option<bool>,
    pub parent: Option<GroupId>,
    pub extent: Option<NodeExtent>,
    pub expand_parent: Option<bool>,
    pub size: Option<CanvasSize>,
    pub hidden: bool,
    pub collapsed: bool,
    pub data: Value,
}

impl From<&Node> for NodeGraphNodeUpdate {
    fn from(node: &Node) -> Self {
        Self {
            kind: node.kind.clone(),
            kind_version: node.kind_version,
            pos: node.pos,
            selectable: node.selectable,
            draggable: node.draggable,
            connectable: node.connectable,
            deletable: node.deletable,
            parent: node.parent,
            extent: node.extent,
            expand_parent: node.expand_parent,
            size: node.size,
            hidden: node.hidden,
            collapsed: node.collapsed,
            data: node.data.clone(),
        }
    }
}

impl NodeGraphNodeUpdate {
    fn apply_to_node(self, node: &mut Node) {
        node.kind = self.kind;
        node.kind_version = self.kind_version;
        node.pos = self.pos;
        node.selectable = self.selectable;
        node.draggable = self.draggable;
        node.connectable = self.connectable;
        node.deletable = self.deletable;
        node.parent = self.parent;
        node.extent = self.extent;
        node.expand_parent = self.expand_parent;
        node.size = self.size;
        node.hidden = self.hidden;
        node.collapsed = self.collapsed;
        node.data = self.data;
    }
}

/// Editable edge snapshot used by `update_edge*`.
///
/// Structural endpoint wiring is intentionally excluded. Use explicit transactions for reconnects.
#[derive(Debug, Clone)]
pub struct NodeGraphEdgeUpdate {
    pub kind: EdgeKind,
    pub selectable: Option<bool>,
    pub deletable: Option<bool>,
    pub reconnectable: Option<EdgeReconnectable>,
}

impl From<&Edge> for NodeGraphEdgeUpdate {
    fn from(edge: &Edge) -> Self {
        Self {
            kind: edge.kind,
            selectable: edge.selectable,
            deletable: edge.deletable,
            reconnectable: edge.reconnectable,
        }
    }
}

impl NodeGraphEdgeUpdate {
    fn apply_to_edge(self, edge: &mut Edge) {
        edge.kind = self.kind;
        edge.selectable = self.selectable;
        edge.deletable = self.deletable;
        edge.reconnectable = self.reconnectable;
    }
}

impl NodeGraphController {
    /// Applies a non-structural node update through the authoritative store transaction path.
    ///
    /// This helper is intended for XyFlow-style ergonomic node updates without bypassing
    /// transaction/history/validation. Structural port edits stay on explicit transactions.
    pub fn update_node<H: UiHost, F>(
        &self,
        host: &mut H,
        node_id: NodeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphNodeUpdate),
    {
        self.update_node_in_models(host.models_mut(), node_id, update)
    }

    /// Applies a non-structural node update from an object-safe action hook.
    pub fn update_node_action_host<F>(
        &self,
        host: &mut dyn UiActionHost,
        node_id: NodeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphNodeUpdate),
    {
        self.update_node_in_models(host.models_mut(), node_id, update)
    }

    /// Applies an edge update through the authoritative store transaction path.
    pub fn update_edge<H: UiHost, F>(
        &self,
        host: &mut H,
        edge_id: EdgeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphEdgeUpdate),
    {
        self.update_edge_in_models(host.models_mut(), edge_id, update)
    }

    /// Applies an edge update from an object-safe action hook.
    pub fn update_edge_action_host<F>(
        &self,
        host: &mut dyn UiActionHost,
        edge_id: EdgeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphEdgeUpdate),
    {
        self.update_edge_in_models(host.models_mut(), edge_id, update)
    }

    fn update_node_in_models<F>(
        &self,
        models: &mut ModelStore,
        node_id: NodeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphNodeUpdate),
    {
        let tx = models
            .read(&self.store, move |store| {
                build_update_node_transaction(store.graph(), node_id, update)
            })
            .map_err(|_| NodeGraphControllerError::StoreUnavailable)??;
        self.dispatch_transaction_in_models(models, &tx)
    }

    fn update_edge_in_models<F>(
        &self,
        models: &mut ModelStore,
        edge_id: EdgeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphEdgeUpdate),
    {
        let tx = models
            .read(&self.store, move |store| {
                build_update_edge_transaction(store.graph(), edge_id, update)
            })
            .map_err(|_| NodeGraphControllerError::StoreUnavailable)??;
        self.dispatch_transaction_in_models(models, &tx)
    }
}

fn build_update_node_transaction<F>(
    graph: &Graph,
    node_id: NodeId,
    update: F,
) -> Result<GraphTransaction, NodeGraphControllerError>
where
    F: FnOnce(&mut NodeGraphNodeUpdate),
{
    let Some(original_node) = graph.nodes.get(&node_id) else {
        return Err(NodeGraphControllerError::NodeNotFound(node_id));
    };

    let mut draft = NodeGraphNodeUpdate::from(original_node);
    update(&mut draft);

    let mut next = graph.clone();
    let node = next
        .nodes
        .get_mut(&node_id)
        .expect("node existence already checked");
    draft.apply_to_node(node);

    let mut tx = graph_diff(graph, &next);
    if !tx.is_empty() {
        tx.label = Some("Update Node".to_string());
    }
    Ok(tx)
}

fn build_update_edge_transaction<F>(
    graph: &Graph,
    edge_id: EdgeId,
    update: F,
) -> Result<GraphTransaction, NodeGraphControllerError>
where
    F: FnOnce(&mut NodeGraphEdgeUpdate),
{
    let Some(original_edge) = graph.edges.get(&edge_id) else {
        return Err(NodeGraphControllerError::EdgeNotFound(edge_id));
    };

    let mut draft = NodeGraphEdgeUpdate::from(original_edge);
    update(&mut draft);

    let mut next = graph.clone();
    let edge = next
        .edges
        .get_mut(&edge_id)
        .expect("edge existence already checked");
    draft.apply_to_edge(edge);

    let mut tx = graph_diff(graph, &next);
    if !tx.is_empty() {
        tx.label = Some("Update Edge".to_string());
    }
    Ok(tx)
}
