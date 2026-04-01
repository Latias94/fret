use super::*;

impl NodeGraphController {
    pub fn dispatch_transaction<H: UiHost>(
        &self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        self.dispatch_transaction_in_models(host.models_mut(), tx)
    }

    pub fn dispatch_transaction_action_host(
        &self,
        host: &mut dyn UiActionHost,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        self.dispatch_transaction_in_models(host.models_mut(), tx)
    }

    pub fn submit_transaction<H: UiHost>(
        &self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.submit_transaction_in_models(host.models_mut(), tx)
    }

    pub fn submit_transaction_action_host(
        &self,
        host: &mut dyn UiActionHost,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.submit_transaction_in_models(host.models_mut(), tx)
    }

    pub fn submit_transaction_and_sync_models<H: UiHost>(
        &self,
        host: &mut H,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.submit_transaction_in_models(host.models_mut(), tx)?;
        let _ = self.sync_models_from_store_in_models(host.models_mut(), graph, view_state);
        Ok(())
    }

    pub fn submit_transaction_and_sync_graph_model<H: UiHost>(
        &self,
        host: &mut H,
        graph: &Model<Graph>,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.submit_transaction_in_models(host.models_mut(), tx)?;
        let _ = self.sync_graph_model_from_store_in_models(host.models_mut(), graph);
        Ok(())
    }

    pub fn sync_models_from_store<H: UiHost>(
        &self,
        host: &mut H,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
    ) -> bool {
        self.sync_models_from_store_in_models(host.models_mut(), graph, view_state)
    }

    pub fn sync_models_from_store_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
    ) -> bool {
        self.sync_models_from_store_in_models(host.models_mut(), graph, view_state)
    }

    pub fn dispatch_transaction_and_sync_models<H: UiHost>(
        &self,
        host: &mut H,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        let outcome = self.dispatch_transaction_in_models(host.models_mut(), tx)?;
        let _ = self.sync_models_from_store_in_models(host.models_mut(), graph, view_state);
        Ok(outcome)
    }

    pub fn dispatch_transaction_and_sync_models_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        let outcome = self.dispatch_transaction_in_models(host.models_mut(), tx)?;
        let _ = self.sync_models_from_store_in_models(host.models_mut(), graph, view_state);
        Ok(outcome)
    }

    pub fn undo<H: UiHost>(
        &self,
        host: &mut H,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        self.undo_in_models(host.models_mut())
    }

    pub fn undo_action_host(
        &self,
        host: &mut dyn UiActionHost,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        self.undo_in_models(host.models_mut())
    }

    pub fn undo_and_sync_models<H: UiHost>(
        &self,
        host: &mut H,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        let outcome = self.undo_in_models(host.models_mut())?;
        if outcome.is_some() {
            let _ = self.sync_models_from_store_in_models(host.models_mut(), graph, view_state);
        }
        Ok(outcome)
    }

    pub fn undo_and_sync_models_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        let outcome = self.undo_in_models(host.models_mut())?;
        if outcome.is_some() {
            let _ = self.sync_models_from_store_in_models(host.models_mut(), graph, view_state);
        }
        Ok(outcome)
    }

    pub fn redo<H: UiHost>(
        &self,
        host: &mut H,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        self.redo_in_models(host.models_mut())
    }

    pub fn redo_action_host(
        &self,
        host: &mut dyn UiActionHost,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        self.redo_in_models(host.models_mut())
    }

    pub fn redo_and_sync_models<H: UiHost>(
        &self,
        host: &mut H,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        let outcome = self.redo_in_models(host.models_mut())?;
        if outcome.is_some() {
            let _ = self.sync_models_from_store_in_models(host.models_mut(), graph, view_state);
        }
        Ok(outcome)
    }

    pub fn redo_and_sync_models_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        let outcome = self.redo_in_models(host.models_mut())?;
        if outcome.is_some() {
            let _ = self.sync_models_from_store_in_models(host.models_mut(), graph, view_state);
        }
        Ok(outcome)
    }

    pub fn replace_graph<H: UiHost>(
        &self,
        host: &mut H,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_graph_in_models(host.models_mut(), graph)
    }

    pub fn replace_document<H: UiHost>(
        &self,
        host: &mut H,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_document_in_models(host.models_mut(), graph, view_state)
    }

    pub fn replace_document_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_document_in_models(host.models_mut(), graph, view_state)
    }

    pub fn replace_graph_and_sync_models<H: UiHost>(
        &self,
        host: &mut H,
        graph_model: &Model<Graph>,
        view_state_model: &Model<NodeGraphViewState>,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_graph_in_models(host.models_mut(), graph)?;
        let _ =
            self.sync_models_from_store_in_models(host.models_mut(), graph_model, view_state_model);
        Ok(())
    }

    pub fn replace_graph_and_sync_models_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph_model: &Model<Graph>,
        view_state_model: &Model<NodeGraphViewState>,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_graph_in_models(host.models_mut(), graph)?;
        let _ =
            self.sync_models_from_store_in_models(host.models_mut(), graph_model, view_state_model);
        Ok(())
    }

    pub fn replace_document_and_sync_models<H: UiHost>(
        &self,
        host: &mut H,
        graph_model: &Model<Graph>,
        view_state_model: &Model<NodeGraphViewState>,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_document_in_models(host.models_mut(), graph, view_state)?;
        let _ =
            self.sync_models_from_store_in_models(host.models_mut(), graph_model, view_state_model);
        Ok(())
    }

    pub fn replace_document_and_sync_models_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph_model: &Model<Graph>,
        view_state_model: &Model<NodeGraphViewState>,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_document_in_models(host.models_mut(), graph, view_state)?;
        let _ =
            self.sync_models_from_store_in_models(host.models_mut(), graph_model, view_state_model);
        Ok(())
    }

    pub fn replace_view_state<H: UiHost>(
        &self,
        host: &mut H,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_view_state_in_models(host.models_mut(), view_state)
    }

    pub fn replace_view_state_action_host(
        &self,
        host: &mut dyn UiActionHost,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_view_state_in_models(host.models_mut(), view_state)
    }

    pub fn sync_view_state_model_from_store<H: UiHost>(
        &self,
        host: &mut H,
        view_state: &Model<NodeGraphViewState>,
    ) -> bool {
        self.sync_view_state_model_from_store_in_models(host.models_mut(), view_state)
    }

    pub fn sync_view_state_model_from_store_action_host(
        &self,
        host: &mut dyn UiActionHost,
        view_state: &Model<NodeGraphViewState>,
    ) -> bool {
        self.sync_view_state_model_from_store_in_models(host.models_mut(), view_state)
    }

    pub fn sync_graph_model_from_store<H: UiHost>(
        &self,
        host: &mut H,
        graph: &Model<Graph>,
    ) -> bool {
        self.sync_graph_model_from_store_in_models(host.models_mut(), graph)
    }

    pub fn sync_graph_model_from_store_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: &Model<Graph>,
    ) -> bool {
        self.sync_graph_model_from_store_in_models(host.models_mut(), graph)
    }

    pub fn replace_view_state_and_sync_model<H: UiHost>(
        &self,
        host: &mut H,
        view_state_model: &Model<NodeGraphViewState>,
        next_view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_view_state_in_models(host.models_mut(), next_view_state)?;
        let _ =
            self.sync_view_state_model_from_store_in_models(host.models_mut(), view_state_model);
        Ok(())
    }

    pub fn replace_view_state_and_sync_model_action_host(
        &self,
        host: &mut dyn UiActionHost,
        view_state_model: &Model<NodeGraphViewState>,
        next_view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_view_state_in_models(host.models_mut(), next_view_state)?;
        let _ =
            self.sync_view_state_model_from_store_in_models(host.models_mut(), view_state_model);
        Ok(())
    }

    pub fn set_selection<H: UiHost>(
        &self,
        host: &mut H,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        groups: Vec<crate::core::GroupId>,
    ) -> Result<(), NodeGraphControllerError> {
        self.set_selection_in_models(host.models_mut(), nodes, edges, groups)
    }

    pub fn set_selection_action_host(
        &self,
        host: &mut dyn UiActionHost,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        groups: Vec<crate::core::GroupId>,
    ) -> Result<(), NodeGraphControllerError> {
        self.set_selection_in_models(host.models_mut(), nodes, edges, groups)
    }

    pub fn set_selection_and_sync_view_model<H: UiHost>(
        &self,
        host: &mut H,
        view_state_model: &Model<NodeGraphViewState>,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        groups: Vec<crate::core::GroupId>,
    ) -> Result<(), NodeGraphControllerError> {
        self.set_selection_in_models(host.models_mut(), nodes, edges, groups)?;
        let _ =
            self.sync_view_state_model_from_store_in_models(host.models_mut(), view_state_model);
        Ok(())
    }

    pub fn set_selection_and_sync_view_model_action_host(
        &self,
        host: &mut dyn UiActionHost,
        view_state_model: &Model<NodeGraphViewState>,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        groups: Vec<crate::core::GroupId>,
    ) -> Result<(), NodeGraphControllerError> {
        self.set_selection_in_models(host.models_mut(), nodes, edges, groups)?;
        let _ =
            self.sync_view_state_model_from_store_in_models(host.models_mut(), view_state_model);
        Ok(())
    }

    fn submit_transaction_in_models(
        &self,
        models: &mut ModelStore,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.dispatch_transaction_in_models(models, tx).map(|_| ())
    }

    pub(super) fn dispatch_transaction_in_models(
        &self,
        models: &mut ModelStore,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        match models.update(&self.store, |store| store.dispatch_transaction(tx)) {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(err)) => Err(NodeGraphControllerError::Dispatch(err)),
            Err(_) => Err(NodeGraphControllerError::StoreUnavailable),
        }
    }

    fn undo_in_models(
        &self,
        models: &mut ModelStore,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        match models.update(&self.store, NodeGraphStore::undo) {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(err)) => Err(NodeGraphControllerError::Dispatch(err)),
            Err(_) => Err(NodeGraphControllerError::StoreUnavailable),
        }
    }

    fn redo_in_models(
        &self,
        models: &mut ModelStore,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        match models.update(&self.store, NodeGraphStore::redo) {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(err)) => Err(NodeGraphControllerError::Dispatch(err)),
            Err(_) => Err(NodeGraphControllerError::StoreUnavailable),
        }
    }

    fn sync_graph_model_from_store_in_models(
        &self,
        models: &mut ModelStore,
        graph: &Model<Graph>,
    ) -> bool {
        let Ok(next_graph) = models.read(&self.store, |store| store.graph().clone()) else {
            return false;
        };

        models
            .update(graph, |graph_model| {
                *graph_model = next_graph;
            })
            .is_ok()
    }

    fn sync_models_from_store_in_models(
        &self,
        models: &mut ModelStore,
        graph: &Model<Graph>,
        view_state: &Model<NodeGraphViewState>,
    ) -> bool {
        let Ok((next_view_state, next_graph)) = models.read(&self.store, |store| {
            (store.view_state().clone(), store.graph().clone())
        }) else {
            return false;
        };

        let graph_synced = models
            .update(graph, |graph| {
                *graph = next_graph;
            })
            .is_ok();
        let view_synced = models
            .update(view_state, |state| {
                *state = next_view_state;
            })
            .is_ok();
        graph_synced && view_synced
    }

    fn sync_view_state_model_from_store_in_models(
        &self,
        models: &mut ModelStore,
        view_state: &Model<NodeGraphViewState>,
    ) -> bool {
        let Ok(next_view_state) = models.read(&self.store, |store| store.view_state().clone())
        else {
            return false;
        };

        models
            .update(view_state, |state| {
                *state = next_view_state;
            })
            .is_ok()
    }

    fn replace_graph_in_models(
        &self,
        models: &mut ModelStore,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        match models.update(&self.store, |store| store.replace_graph(graph)) {
            Ok(()) => Ok(()),
            Err(_) => Err(NodeGraphControllerError::StoreUnavailable),
        }
    }

    fn replace_document_in_models(
        &self,
        models: &mut ModelStore,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        match models.update(&self.store, |store| {
            store.replace_graph(graph);
            store.replace_view_state(view_state);
            store.clear_history();
        }) {
            Ok(()) => Ok(()),
            Err(_) => Err(NodeGraphControllerError::StoreUnavailable),
        }
    }

    fn replace_view_state_in_models(
        &self,
        models: &mut ModelStore,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        match models.update(&self.store, |store| store.replace_view_state(view_state)) {
            Ok(()) => Ok(()),
            Err(_) => Err(NodeGraphControllerError::StoreUnavailable),
        }
    }

    fn set_selection_in_models(
        &self,
        models: &mut ModelStore,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        groups: Vec<crate::core::GroupId>,
    ) -> Result<(), NodeGraphControllerError> {
        match models.update(&self.store, |store| {
            store.set_selection(nodes, edges, groups)
        }) {
            Ok(()) => Ok(()),
            Err(_) => Err(NodeGraphControllerError::StoreUnavailable),
        }
    }
}
