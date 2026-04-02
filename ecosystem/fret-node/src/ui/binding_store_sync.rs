use super::*;
use crate::io::NodeGraphEditorConfig;

impl NodeGraphSurfaceBinding {
    /// Re-syncs the graph/view mirrors from the authoritative store.
    pub fn sync_from_store<H: UiHost>(&self, host: &mut H) -> bool {
        let controller = self.controller();
        let graph_view_synced =
            controller.sync_models_from_store(host, &self.graph, &self.view_state);
        let config_synced = self.editor_config.as_ref().is_none_or(|editor_config| {
            controller.sync_editor_config_model_from_store(host, editor_config)
        });
        graph_view_synced && config_synced
    }

    /// Re-syncs the graph/view mirrors from the authoritative store.
    pub fn sync_from_store_action_host(&self, host: &mut dyn UiActionHost) -> bool {
        let controller = self.controller();
        let graph_view_synced =
            controller.sync_models_from_store_action_host(host, &self.graph, &self.view_state);
        let config_synced = self.editor_config.as_ref().is_none_or(|editor_config| {
            controller.sync_editor_config_model_from_store_action_host(host, editor_config)
        });
        graph_view_synced && config_synced
    }

    /// Dispatches a transaction and keeps the external graph/view mirrors in sync.
    pub fn dispatch_transaction<H: UiHost>(
        &self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        let outcome = self.controller().dispatch_transaction(host, tx)?;
        let _ = self.sync_from_store(host);
        Ok(outcome)
    }

    /// Dispatches a transaction from an object-safe action hook and keeps the external graph/view
    /// mirrors in sync.
    pub fn dispatch_transaction_action_host(
        &self,
        host: &mut dyn UiActionHost,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        let outcome = self
            .controller()
            .dispatch_transaction_action_host(host, tx)?;
        let _ = self.sync_from_store_action_host(host);
        Ok(outcome)
    }

    /// Submits a transaction and keeps the external graph/view mirrors in sync.
    pub fn submit_transaction<H: UiHost>(
        &self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller().submit_transaction(host, tx)?;
        let _ = self.sync_from_store(host);
        Ok(())
    }

    /// Submits a transaction from an object-safe action hook and keeps the external graph/view
    /// mirrors in sync.
    pub fn submit_transaction_action_host(
        &self,
        host: &mut dyn UiActionHost,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller().submit_transaction_action_host(host, tx)?;
        let _ = self.sync_from_store_action_host(host);
        Ok(())
    }

    /// Replaces the authoritative graph and keeps the external graph/view mirrors in sync.
    pub fn replace_graph<H: UiHost>(
        &self,
        host: &mut H,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller().replace_graph(host, graph)?;
        let _ = self.sync_from_store(host);
        Ok(())
    }

    /// Replaces the authoritative graph from an object-safe action hook and keeps the external
    /// graph/view mirrors in sync.
    pub fn replace_graph_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller().replace_graph_action_host(host, graph)?;
        let _ = self.sync_from_store_action_host(host);
        Ok(())
    }

    /// Replaces the entire document snapshot (graph + view state), clears history, and keeps the
    /// external graph/view mirrors in sync.
    pub fn replace_document<H: UiHost>(
        &self,
        host: &mut H,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        let editor_config = self
            .editor_config
            .as_ref()
            .and_then(|editor_config| editor_config.read_ref(host, |config| config.clone()).ok())
            .unwrap_or_default();
        self.controller().replace_document_with_editor_config(
            host,
            graph,
            view_state,
            editor_config,
        )?;
        let _ = self.sync_from_store(host);
        Ok(())
    }

    /// Replaces the entire document snapshot from an object-safe action hook, clears history, and
    /// keeps the external graph/view mirrors in sync.
    pub fn replace_document_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        let editor_config = if let Some(editor_config) = self.editor_config.as_ref() {
            host.models_mut()
                .read(editor_config, |config| config.clone())
                .ok()
                .unwrap_or_default()
        } else {
            NodeGraphEditorConfig::default()
        };
        self.controller()
            .replace_document_with_editor_config_action_host(
                host,
                graph,
                view_state,
                editor_config,
            )?;
        let _ = self.sync_from_store_action_host(host);
        Ok(())
    }

    /// Replaces the authoritative view state and keeps the external view model in sync.
    pub fn replace_view_state<H: UiHost>(
        &self,
        host: &mut H,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller().replace_view_state(host, view_state)?;
        let _ = self.sync_from_store(host);
        Ok(())
    }

    /// Replaces the authoritative view state from an object-safe action hook and keeps the
    /// external view model in sync.
    pub fn replace_view_state_action_host(
        &self,
        host: &mut dyn UiActionHost,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller()
            .replace_view_state_action_host(host, view_state)?;
        let _ = self.sync_from_store_action_host(host);
        Ok(())
    }

    /// Replaces the authoritative selection and keeps the external view model in sync.
    pub fn set_selection<H: UiHost>(
        &self,
        host: &mut H,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        groups: Vec<GroupId>,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller()
            .set_selection(host, nodes, edges, groups)?;
        let _ = self.sync_from_store(host);
        Ok(())
    }

    /// Replaces the authoritative selection from an object-safe action hook and keeps the external
    /// view model in sync.
    pub fn set_selection_action_host(
        &self,
        host: &mut dyn UiActionHost,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        groups: Vec<GroupId>,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller()
            .set_selection_action_host(host, nodes, edges, groups)?;
        let _ = self.sync_from_store_action_host(host);
        Ok(())
    }

    /// Undoes the last committed transaction and keeps the external graph/view mirrors in sync.
    pub fn undo<H: UiHost>(
        &self,
        host: &mut H,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        let outcome = self.controller().undo(host)?;
        if outcome.is_some() {
            let _ = self.sync_from_store(host);
        }
        Ok(outcome)
    }

    /// Undoes the last committed transaction from an object-safe action hook and keeps the external
    /// graph/view mirrors in sync.
    pub fn undo_action_host(
        &self,
        host: &mut dyn UiActionHost,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        let outcome = self.controller().undo_action_host(host)?;
        if outcome.is_some() {
            let _ = self.sync_from_store_action_host(host);
        }
        Ok(outcome)
    }

    /// Redoes the last undone transaction and keeps the external graph/view mirrors in sync.
    pub fn redo<H: UiHost>(
        &self,
        host: &mut H,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        let outcome = self.controller().redo(host)?;
        if outcome.is_some() {
            let _ = self.sync_from_store(host);
        }
        Ok(outcome)
    }

    /// Redoes the last undone transaction from an object-safe action hook and keeps the external
    /// graph/view mirrors in sync.
    pub fn redo_action_host(
        &self,
        host: &mut dyn UiActionHost,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        let outcome = self.controller().redo_action_host(host)?;
        if outcome.is_some() {
            let _ = self.sync_from_store_action_host(host);
        }
        Ok(outcome)
    }

    /// Applies a non-structural node update and keeps the external graph/view mirrors in sync.
    pub fn update_node<H: UiHost, F>(
        &self,
        host: &mut H,
        node_id: NodeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphNodeUpdate),
    {
        let outcome = self.controller().update_node(host, node_id, update)?;
        let _ = self.sync_from_store(host);
        Ok(outcome)
    }

    /// Applies a non-structural node update from an object-safe action hook and keeps the external
    /// graph/view mirrors in sync.
    pub fn update_node_action_host<F>(
        &self,
        host: &mut dyn UiActionHost,
        node_id: NodeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphNodeUpdate),
    {
        let outcome = self
            .controller()
            .update_node_action_host(host, node_id, update)?;
        let _ = self.sync_from_store_action_host(host);
        Ok(outcome)
    }

    /// Applies an edge update and keeps the external graph/view mirrors in sync.
    pub fn update_edge<H: UiHost, F>(
        &self,
        host: &mut H,
        edge_id: EdgeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphEdgeUpdate),
    {
        let outcome = self.controller().update_edge(host, edge_id, update)?;
        let _ = self.sync_from_store(host);
        Ok(outcome)
    }

    /// Applies an edge update from an object-safe action hook and keeps the external graph/view
    /// mirrors in sync.
    pub fn update_edge_action_host<F>(
        &self,
        host: &mut dyn UiActionHost,
        edge_id: EdgeId,
        update: F,
    ) -> Result<DispatchOutcome, NodeGraphControllerError>
    where
        F: FnOnce(&mut NodeGraphEdgeUpdate),
    {
        let outcome = self
            .controller()
            .update_edge_action_host(host, edge_id, update)?;
        let _ = self.sync_from_store_action_host(host);
        Ok(outcome)
    }
}
