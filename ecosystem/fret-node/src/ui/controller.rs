use fret_core::Rect;
use fret_runtime::{Model, ModelStore};
use fret_ui::UiHost;
use fret_ui::action::UiActionHost;

use crate::core::{CanvasPoint, EdgeId, Graph, NodeId, PortId};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::runtime::fit_view::{FitViewComputeOptions, FitViewNodeInfo, compute_fit_view_target};
use crate::runtime::lookups::{ConnectionSide, HandleConnection};
use crate::runtime::store::{DispatchError, DispatchOutcome, NodeGraphStore};
use crate::runtime::utils::{get_connected_edges, get_incomers, get_outgoers};

use super::edit_queue::NodeGraphEditQueue;
use super::view_queue::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions, NodeGraphViewQueue};
use super::viewport_helper::pan_for_center;

const CONTROLLER_FIT_VIEW_MIN_ZOOM: f32 = 0.05;
const CONTROLLER_FIT_VIEW_MAX_ZOOM: f32 = 64.0;
const CONTROLLER_FIT_VIEW_MARGIN_PX_FALLBACK: f32 = 48.0;

#[derive(Debug, thiserror::Error)]
pub enum NodeGraphControllerError {
    #[error("store unavailable")]
    StoreUnavailable,
    #[error(transparent)]
    Dispatch(#[from] DispatchError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeGraphPortConnectionsQuery {
    pub node_id: NodeId,
    pub port_id: PortId,
    pub side: ConnectionSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeGraphNodeConnectionsQuery {
    pub node_id: NodeId,
    pub side: Option<ConnectionSide>,
    pub port_id: Option<PortId>,
}

#[derive(Debug, Clone)]
pub struct NodeGraphController {
    store: Model<NodeGraphStore>,
    edit_queue: Option<Model<NodeGraphEditQueue>>,
    view_queue: Option<Model<NodeGraphViewQueue>>,
}

impl NodeGraphController {
    pub fn new(store: Model<NodeGraphStore>) -> Self {
        Self {
            store,
            edit_queue: None,
            view_queue: None,
        }
    }

    pub(crate) fn bind_edit_queue_transport(mut self, queue: Model<NodeGraphEditQueue>) -> Self {
        self.edit_queue = Some(queue);
        self
    }

    pub(crate) fn bind_view_queue_transport(mut self, queue: Model<NodeGraphViewQueue>) -> Self {
        self.view_queue = Some(queue);
        self
    }

    pub fn store(&self) -> Model<NodeGraphStore> {
        self.store.clone()
    }

    pub(crate) fn transport_edit_queue(&self) -> Option<Model<NodeGraphEditQueue>> {
        self.edit_queue.clone()
    }

    pub(crate) fn transport_view_queue(&self) -> Option<Model<NodeGraphViewQueue>> {
        self.view_queue.clone()
    }

    pub fn viewport<H: UiHost>(&self, host: &H) -> (CanvasPoint, f32) {
        self.store
            .read_ref(host, |store| {
                let view = store.view_state();
                (view.pan, view.zoom)
            })
            .ok()
            .unwrap_or((CanvasPoint::default(), 1.0))
    }

    pub fn graph_snapshot<H: UiHost>(&self, host: &H) -> Option<Graph> {
        self.store
            .read_ref(host, |store| store.graph().clone())
            .ok()
    }

    pub fn view_state_snapshot<H: UiHost>(&self, host: &H) -> Option<NodeGraphViewState> {
        self.store
            .read_ref(host, |store| store.view_state().clone())
            .ok()
    }

    pub fn can_undo<H: UiHost>(&self, host: &H) -> bool {
        self.store
            .read_ref(host, |store| store.can_undo())
            .ok()
            .unwrap_or(false)
    }

    pub fn can_redo<H: UiHost>(&self, host: &H) -> bool {
        self.store
            .read_ref(host, |store| store.can_redo())
            .ok()
            .unwrap_or(false)
    }

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
        if self.edit_queue.is_none() {
            let _ = self.sync_models_from_store_in_models(host.models_mut(), graph, view_state);
        }
        Ok(())
    }

    pub fn submit_transaction_and_sync_graph_model<H: UiHost>(
        &self,
        host: &mut H,
        graph: &Model<Graph>,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.submit_transaction_in_models(host.models_mut(), tx)?;
        if self.edit_queue.is_none() {
            let _ = self.sync_graph_model_from_store_in_models(host.models_mut(), graph);
        }
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

    /// Replaces the entire document snapshot (graph + view state), clears history, and keeps
    /// bound graph/view models in sync.
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

    /// Replaces the entire document snapshot (graph + view state), clears history, and keeps
    /// bound graph/view models in sync.
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

    pub fn set_viewport<H: UiHost>(&self, host: &mut H, pan: CanvasPoint, zoom: f32) -> bool {
        self.set_viewport_with_options(host, pan, zoom, NodeGraphSetViewportOptions::default())
    }

    pub fn set_viewport_with_options<H: UiHost>(
        &self,
        host: &mut H,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        if let Some(queue) = self.view_queue.as_ref() {
            return queue
                .update(host, |queue, _cx| {
                    queue.push_set_viewport_with_options(pan, zoom, options);
                })
                .is_ok();
        }

        let (current_pan, current_zoom) = self.viewport(&*host);
        let pan = normalize_requested_pan(current_pan, pan);
        let zoom = normalize_requested_zoom(current_zoom, zoom, options.min_zoom, options.max_zoom);

        self.store
            .update(host, |store, _cx| {
                store.set_viewport(pan, zoom);
            })
            .is_ok()
    }

    pub fn set_center_in_bounds<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
    ) -> bool {
        self.set_center_in_bounds_with_options(
            host,
            bounds,
            center,
            None,
            NodeGraphSetViewportOptions::default(),
        )
    }

    pub fn set_center_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let (_, current_zoom) = self.viewport(&*host);
        let zoom = normalize_requested_zoom(
            current_zoom,
            zoom.unwrap_or(current_zoom),
            options.min_zoom,
            options.max_zoom,
        );
        let pan = pan_for_center(bounds, center, zoom);
        self.set_viewport_with_options(host, pan, zoom, options)
    }

    pub fn fit_view_nodes<H: UiHost>(&self, host: &mut H, nodes: Vec<NodeId>) -> bool {
        self.fit_view_nodes_with_options(host, nodes, NodeGraphFitViewOptions::default())
    }

    pub fn fit_view_nodes_with_options<H: UiHost>(
        &self,
        host: &mut H,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        let Some(queue) = self.view_queue.as_ref() else {
            return false;
        };
        queue
            .update(host, |queue, _cx| {
                queue.push_frame_nodes_with_options(nodes, options);
            })
            .is_ok()
    }

    pub fn fit_view_nodes_in_bounds<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        nodes: Vec<NodeId>,
    ) -> bool {
        self.fit_view_nodes_in_bounds_with_options(
            host,
            bounds,
            nodes,
            NodeGraphFitViewOptions::default(),
        )
    }

    pub fn fit_view_nodes_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        let target = self
            .store
            .read_ref(&*host, |store| {
                let interaction = &store.view_state().interaction;
                let node_origin = interaction.node_origin.normalized();
                let padding = normalize_fit_view_padding(
                    options.padding.unwrap_or(interaction.frame_view_padding),
                );
                let (min_zoom, max_zoom) =
                    normalize_fit_view_zoom_bounds(options.min_zoom, options.max_zoom);

                let infos: Vec<FitViewNodeInfo> = nodes
                    .iter()
                    .filter_map(|id| {
                        let entry = store.lookups().node_lookup.get(id)?;
                        if entry.hidden && !options.include_hidden_nodes {
                            return None;
                        }
                        let size = entry.size?;
                        if !size.width.is_finite()
                            || !size.height.is_finite()
                            || size.width <= 0.0
                            || size.height <= 0.0
                        {
                            return None;
                        }
                        Some(FitViewNodeInfo {
                            pos: entry.pos,
                            size_px: (size.width, size.height),
                        })
                    })
                    .collect();

                compute_fit_view_target(
                    &infos,
                    FitViewComputeOptions {
                        viewport_width_px: bounds.size.width.0,
                        viewport_height_px: bounds.size.height.0,
                        node_origin: (node_origin.x, node_origin.y),
                        padding,
                        margin_px_fallback: CONTROLLER_FIT_VIEW_MARGIN_PX_FALLBACK,
                        min_zoom,
                        max_zoom,
                    },
                )
            })
            .ok()
            .flatten();

        let Some((pan, zoom)) = target else {
            return false;
        };

        self.set_viewport_with_options(
            host,
            pan,
            zoom,
            NodeGraphSetViewportOptions {
                min_zoom: options.min_zoom,
                max_zoom: options.max_zoom,
                duration_ms: options.duration_ms,
                interpolate: options.interpolate,
                ease: options.ease,
            },
        )
    }

    pub fn outgoers<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<NodeId> {
        self.store
            .read_ref(host, |store| get_outgoers(store.lookups(), node))
            .ok()
            .unwrap_or_default()
    }

    pub fn incomers<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<NodeId> {
        self.store
            .read_ref(host, |store| get_incomers(store.lookups(), node))
            .ok()
            .unwrap_or_default()
    }

    pub fn connected_edges<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<EdgeId> {
        self.store
            .read_ref(host, |store| get_connected_edges(store.lookups(), node))
            .ok()
            .unwrap_or_default()
    }

    pub fn port_connections<H: UiHost>(
        &self,
        host: &H,
        query: NodeGraphPortConnectionsQuery,
    ) -> Vec<HandleConnection> {
        self.store
            .read_ref(host, |store| {
                sorted_connections(store.lookups().connections_for_port(
                    query.node_id,
                    query.side,
                    query.port_id,
                ))
            })
            .ok()
            .unwrap_or_default()
    }

    pub fn node_connections<H: UiHost>(
        &self,
        host: &H,
        query: NodeGraphNodeConnectionsQuery,
    ) -> Vec<HandleConnection> {
        self.store
            .read_ref(host, |store| {
                let lookups = store.lookups();
                let connections = match (query.side, query.port_id) {
                    (Some(side), Some(port_id)) => {
                        lookups.connections_for_port(query.node_id, side, port_id)
                    }
                    (Some(side), None) => lookups.connections_for_node_side(query.node_id, side),
                    (None, Some(port_id)) => store
                        .graph()
                        .ports
                        .get(&port_id)
                        .filter(|port| port.node == query.node_id)
                        .and_then(|port| {
                            lookups.connections_for_port(
                                query.node_id,
                                ConnectionSide::from_port_dir(port.dir),
                                port_id,
                            )
                        }),
                    (None, None) => lookups.connections_for_node(query.node_id),
                };
                sorted_connections(connections)
            })
            .ok()
            .unwrap_or_default()
    }

    fn submit_transaction_in_models(
        &self,
        models: &mut ModelStore,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        if let Some(queue) = self.edit_queue.as_ref() {
            return match models.update(queue, |edit_queue| edit_queue.push(tx.clone())) {
                Ok(()) => Ok(()),
                Err(_) => Err(NodeGraphControllerError::StoreUnavailable),
            };
        }

        self.dispatch_transaction_in_models(models, tx).map(|_| ())
    }

    fn dispatch_transaction_in_models(
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

fn normalize_requested_pan(current_pan: CanvasPoint, requested_pan: CanvasPoint) -> CanvasPoint {
    if requested_pan.x.is_finite() && requested_pan.y.is_finite() {
        requested_pan
    } else {
        current_pan
    }
}

fn normalize_requested_zoom(
    current_zoom: f32,
    requested_zoom: f32,
    min_zoom: Option<f32>,
    max_zoom: Option<f32>,
) -> f32 {
    let mut min_zoom = min_zoom
        .filter(|zoom| zoom.is_finite() && *zoom > 0.0)
        .unwrap_or(f32::MIN_POSITIVE);
    let mut max_zoom = max_zoom
        .filter(|zoom| zoom.is_finite() && *zoom > 0.0)
        .unwrap_or(f32::MAX);
    if min_zoom > max_zoom {
        std::mem::swap(&mut min_zoom, &mut max_zoom);
    }

    let base = if requested_zoom.is_finite() && requested_zoom > 0.0 {
        requested_zoom
    } else if current_zoom.is_finite() && current_zoom > 0.0 {
        current_zoom
    } else {
        1.0
    };

    base.clamp(min_zoom, max_zoom)
}

fn sorted_connections(
    connections: Option<&std::collections::HashMap<EdgeId, HandleConnection>>,
) -> Vec<HandleConnection> {
    let Some(connections) = connections else {
        return Vec::new();
    };
    let mut out: Vec<_> = connections.values().copied().collect();
    out.sort_by_key(|connection| connection.edge);
    out
}

fn normalize_fit_view_padding(padding: f32) -> f32 {
    if padding.is_finite() {
        padding.clamp(0.0, 0.45)
    } else {
        0.0
    }
}

fn normalize_fit_view_zoom_bounds(min_zoom: Option<f32>, max_zoom: Option<f32>) -> (f32, f32) {
    let mut min_zoom = min_zoom
        .filter(|zoom| zoom.is_finite() && *zoom > 0.0)
        .unwrap_or(CONTROLLER_FIT_VIEW_MIN_ZOOM);
    let mut max_zoom = max_zoom
        .filter(|zoom| zoom.is_finite() && *zoom > 0.0)
        .unwrap_or(CONTROLLER_FIT_VIEW_MAX_ZOOM);
    if min_zoom > max_zoom {
        std::mem::swap(&mut min_zoom, &mut max_zoom);
    }
    (min_zoom, max_zoom)
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::ui_host::{
        CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, DragKindId, DragSession, DragSessionId, Effect, FrameId,
        ModelHost, ModelStore, ShareSheetToken, TickId, TimerToken,
    };
    use serde_json::Value;

    use super::{
        CONTROLLER_FIT_VIEW_MARGIN_PX_FALLBACK, CONTROLLER_FIT_VIEW_MAX_ZOOM,
        CONTROLLER_FIT_VIEW_MIN_ZOOM, NodeGraphController, NodeGraphNodeConnectionsQuery,
        NodeGraphPortConnectionsQuery,
    };
    use crate::core::{
        CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey,
        Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };
    use crate::io::NodeGraphViewState;
    use crate::ops::{GraphOp, GraphTransaction};
    use crate::runtime::fit_view::{
        FitViewComputeOptions, FitViewNodeInfo, compute_fit_view_target,
    };
    use crate::runtime::lookups::{ConnectionSide, HandleConnection};
    use crate::runtime::store::NodeGraphStore;
    use crate::ui::NodeGraphSurfaceBinding;
    use crate::ui::edit_queue::NodeGraphEditQueue;
    use crate::ui::view_queue::{
        NodeGraphFitViewOptions, NodeGraphSetViewportOptions, NodeGraphViewQueue,
        NodeGraphViewRequest,
    };

    #[derive(Default)]
    struct TestUiHostImpl {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        effects: Vec<Effect>,
        drag: Option<DragSession>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestUiHostImpl {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|value| value.downcast_ref::<T>())
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

    impl fret_ui::action::UiActionHost for TestUiHostImpl {
        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn request_redraw(&mut self, _window: AppWindowId) {}

        fn next_timer_token(&mut self) -> TimerToken {
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            TimerToken(self.next_timer_token)
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            ClipboardToken(self.next_clipboard_token)
        }

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
            ShareSheetToken(self.next_share_sheet_token)
        }
    }

    impl EffectSink for TestUiHostImpl {
        fn request_redraw(&mut self, _window: AppWindowId) {}

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

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
            ShareSheetToken(self.next_share_sheet_token)
        }

        fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            fret_runtime::ImageUploadToken(self.next_image_upload_token)
        }
    }

    impl DragHost for TestUiHostImpl {
        fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&DragSession> {
            self.drag
                .as_ref()
                .filter(|drag| drag.pointer_id == pointer_id)
        }

        fn drag_mut(&mut self, pointer_id: fret_core::PointerId) -> Option<&mut DragSession> {
            self.drag
                .as_mut()
                .filter(|drag| drag.pointer_id == pointer_id)
        }

        fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
            if self.drag(pointer_id).is_some() {
                self.drag = None;
            }
        }

        fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
            self.drag.as_ref().is_some_and(|drag| predicate(drag))
        }

        fn find_drag_pointer_id(
            &self,
            mut predicate: impl FnMut(&DragSession) -> bool,
        ) -> Option<fret_core::PointerId> {
            self.drag
                .as_ref()
                .filter(|drag| predicate(drag))
                .map(|drag| drag.pointer_id)
        }

        fn cancel_drag_sessions(
            &mut self,
            mut predicate: impl FnMut(&DragSession) -> bool,
        ) -> Vec<fret_core::PointerId> {
            let Some(drag) = self.drag.as_ref() else {
                return Vec::new();
            };
            if !predicate(drag) {
                return Vec::new();
            }
            let pointer_id = drag.pointer_id;
            self.drag = None;
            vec![pointer_id]
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.drag = Some(DragSession::new(
                DragSessionId(1),
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ));
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.drag = Some(DragSession::new_cross_window(
                DragSessionId(1),
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ));
        }
    }

    fn test_node(pos: CanvasPoint, ports: Vec<PortId>) -> Node {
        test_node_with_size(pos, None, ports)
    }

    fn test_node_with_size(pos: CanvasPoint, size: Option<CanvasSize>, ports: Vec<PortId>) -> Node {
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos,
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size,
            hidden: false,
            collapsed: false,
            ports,
            data: Value::Null,
        }
    }

    fn test_port(node: NodeId, key: &str, dir: PortDirection, capacity: PortCapacity) -> Port {
        Port {
            node,
            key: PortKey::new(key),
            dir,
            kind: PortKind::Data,
            capacity,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        }
    }

    fn make_test_graph_two_nodes() -> (Graph, NodeId, NodeId) {
        let mut graph = Graph::new(GraphId::new());
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        graph.nodes.insert(
            node_a,
            test_node(CanvasPoint { x: 0.0, y: 0.0 }, Vec::new()),
        );
        graph.nodes.insert(
            node_b,
            test_node(CanvasPoint { x: 10.0, y: 0.0 }, Vec::new()),
        );
        (graph, node_a, node_b)
    }

    fn make_test_graph_two_nodes_with_ports() -> (Graph, NodeId, PortId, NodeId, PortId) {
        let mut graph = Graph::new(GraphId::new());
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        let a_out = PortId::new();
        let b_in = PortId::new();
        graph.nodes.insert(
            node_a,
            test_node(CanvasPoint { x: 0.0, y: 0.0 }, vec![a_out]),
        );
        graph.nodes.insert(
            node_b,
            test_node(CanvasPoint { x: 200.0, y: 0.0 }, vec![b_in]),
        );
        graph.ports.insert(
            a_out,
            test_port(node_a, "out", PortDirection::Out, PortCapacity::Multi),
        );
        graph.ports.insert(
            b_in,
            test_port(node_b, "in", PortDirection::In, PortCapacity::Single),
        );
        (graph, node_a, a_out, node_b, b_in)
    }

    #[test]
    fn controller_dispatch_transaction_and_sync_models_updates_bound_models() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, node_a, _node_b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value.clone());
        let view = host.models.insert(NodeGraphViewState::default());
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store.clone());

        let from = graph
            .read_ref(&host, |graph| graph.nodes.get(&node_a).map(|node| node.pos))
            .ok()
            .flatten()
            .expect("start node position");
        let to = CanvasPoint {
            x: from.x + 24.0,
            y: from.y + 8.0,
        };
        let mut tx = GraphTransaction::new().with_label("Move Node");
        tx.push(GraphOp::SetNodePos {
            id: node_a,
            from,
            to,
        });

        let outcome = controller
            .dispatch_transaction_and_sync_models(&mut host, &graph, &view, &tx)
            .expect("dispatch transaction through controller");

        assert_eq!(outcome.committed.label.as_deref(), Some("Move Node"));
        let graph_pos = graph
            .read_ref(&host, |graph| graph.nodes.get(&node_a).map(|node| node.pos))
            .ok()
            .flatten()
            .expect("graph node position");
        let store_pos = store
            .read_ref(&host, |store| {
                store.graph().nodes.get(&node_a).map(|node| node.pos)
            })
            .ok()
            .flatten()
            .expect("store node position");
        assert_eq!(graph_pos, to);
        assert_eq!(store_pos, to);
    }

    #[test]
    fn controller_replace_graph_and_sync_models_action_host_updates_bound_models() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, node_a, node_b) = make_test_graph_two_nodes();
        let mut initial_view = NodeGraphViewState::default();
        initial_view.selected_nodes = vec![node_b];
        let graph = host.models.insert(graph_value.clone());
        let view = host.models.insert(initial_view.clone());
        let store = host
            .models
            .insert(NodeGraphStore::new(graph_value, initial_view));
        let controller = NodeGraphController::new(store.clone());

        let mut replacement = Graph::new(
            graph
                .read_ref(&host, |value| value.graph_id)
                .ok()
                .expect("graph id"),
        );
        replacement.nodes.insert(
            node_a,
            graph
                .read_ref(&host, |value| value.nodes[&node_a].clone())
                .ok()
                .expect("node a"),
        );

        controller
            .replace_graph_and_sync_models_action_host(
                &mut host,
                &graph,
                &view,
                replacement.clone(),
            )
            .expect("replace graph through controller");

        let model_nodes = graph
            .read_ref(&host, |value| {
                value.nodes.keys().copied().collect::<Vec<_>>()
            })
            .ok()
            .expect("graph model nodes");
        let store_nodes = store
            .read_ref(&host, |value| {
                value.graph().nodes.keys().copied().collect::<Vec<_>>()
            })
            .ok()
            .expect("store nodes");
        let model_selection = view
            .read_ref(&host, |state| state.selected_nodes.clone())
            .ok()
            .expect("view selection");
        let store_selection = store
            .read_ref(&host, |value| value.view_state().selected_nodes.clone())
            .ok()
            .expect("store selection");

        assert_eq!(model_nodes, vec![node_a]);
        assert_eq!(store_nodes, vec![node_a]);
        assert!(model_selection.is_empty());
        assert!(store_selection.is_empty());
    }

    #[test]
    fn controller_replace_document_and_sync_models_action_host_resets_history() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, node_a, node_b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value.clone());
        let view = host.models.insert(NodeGraphViewState::default());
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store.clone());

        let from = graph
            .read_ref(&host, |value| value.nodes.get(&node_a).map(|node| node.pos))
            .ok()
            .flatten()
            .expect("node pos");
        let mut tx = GraphTransaction::new().with_label("Move Node");
        tx.push(GraphOp::SetNodePos {
            id: node_a,
            from,
            to: CanvasPoint {
                x: from.x + 20.0,
                y: from.y + 10.0,
            },
        });
        controller
            .dispatch_transaction_and_sync_models(&mut host, &graph, &view, &tx)
            .expect("seed history");
        assert!(controller.can_undo(&host));

        let mut next_graph = Graph::new(
            graph
                .read_ref(&host, |value| value.graph_id)
                .ok()
                .expect("graph id"),
        );
        next_graph.nodes.insert(
            node_b,
            graph
                .read_ref(&host, |value| value.nodes[&node_b].clone())
                .ok()
                .expect("node b"),
        );
        let mut next_view = NodeGraphViewState::default();
        next_view.pan = CanvasPoint { x: 88.0, y: 21.0 };
        next_view.zoom = 1.75;
        next_view.selected_nodes = vec![node_a];

        controller
            .replace_document_and_sync_models_action_host(
                &mut host, &graph, &view, next_graph, next_view,
            )
            .expect("replace document through controller");

        let model_nodes = graph
            .read_ref(&host, |value| {
                value.nodes.keys().copied().collect::<Vec<_>>()
            })
            .ok()
            .expect("graph model nodes");
        let (model_pan, model_zoom, model_selection) = view
            .read_ref(&host, |state| {
                (state.pan, state.zoom, state.selected_nodes.clone())
            })
            .ok()
            .expect("view model");
        let (store_pan, store_zoom, store_selection, can_undo, can_redo) = store
            .read_ref(&host, |value| {
                (
                    value.view_state().pan,
                    value.view_state().zoom,
                    value.view_state().selected_nodes.clone(),
                    value.can_undo(),
                    value.can_redo(),
                )
            })
            .ok()
            .expect("store state");

        assert_eq!(model_nodes, vec![node_b]);
        assert_eq!(model_pan, CanvasPoint { x: 88.0, y: 21.0 });
        assert_eq!(model_zoom, 1.75);
        assert!(model_selection.is_empty());
        assert_eq!(store_pan, CanvasPoint { x: 88.0, y: 21.0 });
        assert_eq!(store_zoom, 1.75);
        assert!(store_selection.is_empty());
        assert!(!can_undo);
        assert!(!can_redo);
    }

    #[test]
    fn controller_replace_view_state_and_sync_model_action_host_updates_bound_view_model() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let view = host.models.insert(NodeGraphViewState::default());
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store.clone());
        let mut next_view = NodeGraphViewState::default();
        next_view.pan = CanvasPoint { x: 12.0, y: 34.0 };
        next_view.zoom = 1.5;

        controller
            .replace_view_state_and_sync_model_action_host(&mut host, &view, next_view.clone())
            .expect("replace view-state through controller");

        let model_view = view.read_ref(&host, |state| state.clone()).ok().unwrap();
        let store_view = store
            .read_ref(&host, |store| store.view_state().clone())
            .ok()
            .unwrap();
        assert_eq!(model_view.pan, next_view.pan);
        assert_eq!(model_view.zoom, next_view.zoom);
        assert_eq!(store_view.pan, next_view.pan);
        assert_eq!(store_view.zoom, next_view.zoom);
    }

    #[test]
    fn controller_set_selection_and_sync_view_model_action_host_updates_bound_view_model() {
        let mut host = TestUiHostImpl::default();
        let (mut graph, node_a, a_out, node_b, b_in) = make_test_graph_two_nodes_with_ports();
        let edge = EdgeId::new();
        graph.edges.insert(
            edge,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        let view = host.models.insert(NodeGraphViewState::default());
        let store = host
            .models
            .insert(NodeGraphStore::new(graph, NodeGraphViewState::default()));
        let controller = NodeGraphController::new(store.clone());

        controller
            .set_selection_and_sync_view_model_action_host(
                &mut host,
                &view,
                vec![node_a, node_b],
                vec![edge],
                Vec::new(),
            )
            .expect("set selection through controller");

        let model_view = view.read_ref(&host, |state| state.clone()).ok().unwrap();
        let store_view = store
            .read_ref(&host, |store| store.view_state().clone())
            .ok()
            .unwrap();
        assert_eq!(model_view.selected_nodes, vec![node_a, node_b]);
        assert_eq!(model_view.selected_edges, vec![edge]);
        assert_eq!(store_view.selected_nodes, vec![node_a, node_b]);
        assert_eq!(store_view.selected_edges, vec![edge]);
    }

    #[test]
    fn controller_set_viewport_uses_queue_when_present() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let queue = host.models.insert(NodeGraphViewQueue::default());
        let controller =
            NodeGraphController::new(store.clone()).bind_view_queue_transport(queue.clone());

        assert!(controller.set_viewport_with_options(
            &mut host,
            CanvasPoint { x: 10.0, y: 20.0 },
            1.5,
            NodeGraphSetViewportOptions {
                duration_ms: Some(0),
                ..NodeGraphSetViewportOptions::default()
            },
        ));

        let pending = queue
            .read_ref(&host, |queue| queue.pending.clone())
            .ok()
            .unwrap_or_default();
        assert_eq!(pending.len(), 1);
        let NodeGraphViewRequest::SetViewport { pan, zoom, .. } = pending[0].clone() else {
            panic!("expected queued SetViewport request");
        };
        assert_eq!(pan, CanvasPoint { x: 10.0, y: 20.0 });
        assert!((zoom - 1.5).abs() <= 1.0e-6);

        assert_eq!(controller.viewport(&host), (CanvasPoint::default(), 1.0));
    }

    #[test]
    fn controller_set_viewport_falls_back_to_store_without_queue() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store);

        assert!(controller.set_viewport(&mut host, CanvasPoint { x: 3.0, y: 4.0 }, 2.0,));
        assert_eq!(
            controller.viewport(&host),
            (CanvasPoint { x: 3.0, y: 4.0 }, 2.0)
        );
    }

    #[test]
    fn controller_set_viewport_fallback_clamps_zoom_options() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store);

        assert!(controller.set_viewport_with_options(
            &mut host,
            CanvasPoint { x: 3.0, y: 4.0 },
            2.0,
            NodeGraphSetViewportOptions {
                max_zoom: Some(1.25),
                ..NodeGraphSetViewportOptions::default()
            },
        ));
        assert_eq!(
            controller.viewport(&host),
            (CanvasPoint { x: 3.0, y: 4.0 }, 1.25)
        );
    }

    #[test]
    fn controller_set_center_in_bounds_falls_back_to_store_without_queue() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        assert!(controller.set_center_in_bounds_with_options(
            &mut host,
            bounds,
            CanvasPoint { x: 10.0, y: 20.0 },
            Some(2.0),
            NodeGraphSetViewportOptions::default(),
        ));
        assert_eq!(
            controller.viewport(&host),
            (CanvasPoint { x: 190.0, y: 130.0 }, 2.0)
        );
    }

    #[test]
    fn controller_fit_view_nodes_in_bounds_falls_back_to_store_without_queue() {
        let mut host = TestUiHostImpl::default();
        let mut graph = Graph::new(GraphId::new());
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        graph.nodes.insert(
            node_a,
            test_node_with_size(
                CanvasPoint { x: 0.0, y: 0.0 },
                Some(CanvasSize {
                    width: 100.0,
                    height: 100.0,
                }),
                Vec::new(),
            ),
        );
        graph.nodes.insert(
            node_b,
            test_node_with_size(
                CanvasPoint { x: 200.0, y: 0.0 },
                Some(CanvasSize {
                    width: 100.0,
                    height: 100.0,
                }),
                Vec::new(),
            ),
        );
        let store = host
            .models
            .insert(NodeGraphStore::new(graph, NodeGraphViewState::default()));
        let controller = NodeGraphController::new(store);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let options = NodeGraphFitViewOptions {
            min_zoom: Some(CONTROLLER_FIT_VIEW_MIN_ZOOM),
            max_zoom: Some(CONTROLLER_FIT_VIEW_MAX_ZOOM),
            padding: Some(0.0),
            ..NodeGraphFitViewOptions::default()
        };

        assert!(controller.fit_view_nodes_in_bounds_with_options(
            &mut host,
            bounds,
            vec![node_a, node_b],
            options.clone(),
        ));

        let expected = compute_fit_view_target(
            &[
                FitViewNodeInfo {
                    pos: CanvasPoint { x: 0.0, y: 0.0 },
                    size_px: (100.0, 100.0),
                },
                FitViewNodeInfo {
                    pos: CanvasPoint { x: 200.0, y: 0.0 },
                    size_px: (100.0, 100.0),
                },
            ],
            FitViewComputeOptions {
                viewport_width_px: 800.0,
                viewport_height_px: 600.0,
                node_origin: (0.0, 0.0),
                padding: options.padding.unwrap_or(0.0),
                margin_px_fallback: CONTROLLER_FIT_VIEW_MARGIN_PX_FALLBACK,
                min_zoom: CONTROLLER_FIT_VIEW_MIN_ZOOM,
                max_zoom: CONTROLLER_FIT_VIEW_MAX_ZOOM,
            },
        )
        .expect("fit-view target");
        let (pan, zoom) = controller.viewport(&host);

        assert!((pan.x - expected.0.x).abs() <= 1.0e-6);
        assert!((pan.y - expected.0.y).abs() <= 1.0e-6);
        assert!((zoom - expected.1).abs() <= 1.0e-6);
    }

    #[test]
    fn controller_fit_view_nodes_in_bounds_uses_queue_when_present() {
        let mut host = TestUiHostImpl::default();
        let mut graph = Graph::new(GraphId::new());
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        graph.nodes.insert(
            node_a,
            test_node_with_size(
                CanvasPoint { x: 0.0, y: 0.0 },
                Some(CanvasSize {
                    width: 100.0,
                    height: 100.0,
                }),
                Vec::new(),
            ),
        );
        graph.nodes.insert(
            node_b,
            test_node_with_size(
                CanvasPoint { x: 200.0, y: 0.0 },
                Some(CanvasSize {
                    width: 100.0,
                    height: 100.0,
                }),
                Vec::new(),
            ),
        );
        let store = host
            .models
            .insert(NodeGraphStore::new(graph, NodeGraphViewState::default()));
        let queue = host.models.insert(NodeGraphViewQueue::default());
        let controller = NodeGraphController::new(store).bind_view_queue_transport(queue.clone());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let options = NodeGraphFitViewOptions {
            min_zoom: Some(CONTROLLER_FIT_VIEW_MIN_ZOOM),
            max_zoom: Some(CONTROLLER_FIT_VIEW_MAX_ZOOM),
            padding: Some(0.0),
            duration_ms: Some(0),
            ..NodeGraphFitViewOptions::default()
        };

        assert!(controller.fit_view_nodes_in_bounds_with_options(
            &mut host,
            bounds,
            vec![node_a, node_b],
            options.clone(),
        ));

        let pending = queue
            .read_ref(&host, |queue| queue.pending.clone())
            .ok()
            .unwrap_or_default();
        assert_eq!(pending.len(), 1);
        let NodeGraphViewRequest::SetViewport { pan, zoom, .. } = pending[0].clone() else {
            panic!("expected queued SetViewport request");
        };

        let expected = compute_fit_view_target(
            &[
                FitViewNodeInfo {
                    pos: CanvasPoint { x: 0.0, y: 0.0 },
                    size_px: (100.0, 100.0),
                },
                FitViewNodeInfo {
                    pos: CanvasPoint { x: 200.0, y: 0.0 },
                    size_px: (100.0, 100.0),
                },
            ],
            FitViewComputeOptions {
                viewport_width_px: 800.0,
                viewport_height_px: 600.0,
                node_origin: (0.0, 0.0),
                padding: options.padding.unwrap_or(0.0),
                margin_px_fallback: CONTROLLER_FIT_VIEW_MARGIN_PX_FALLBACK,
                min_zoom: CONTROLLER_FIT_VIEW_MIN_ZOOM,
                max_zoom: CONTROLLER_FIT_VIEW_MAX_ZOOM,
            },
        )
        .expect("fit-view target");

        assert!((pan.x - expected.0.x).abs() <= 1.0e-6);
        assert!((pan.y - expected.0.y).abs() <= 1.0e-6);
        assert!((zoom - expected.1).abs() <= 1.0e-6);
    }

    #[test]
    fn controller_queries_use_store_lookups() {
        let mut host = TestUiHostImpl::default();
        let (mut graph, node_a, a_out, node_b, b_in) = make_test_graph_two_nodes_with_ports();
        let edge = EdgeId::new();
        graph.edges.insert(
            edge,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        let store = host
            .models
            .insert(NodeGraphStore::new(graph, NodeGraphViewState::default()));
        let controller = NodeGraphController::new(store);
        let expected = HandleConnection {
            edge,
            source_node: node_a,
            source_port: a_out,
            target_node: node_b,
            target_port: b_in,
            kind: EdgeKind::Data,
        };

        assert_eq!(controller.outgoers(&host, node_a), vec![node_b]);
        assert_eq!(controller.incomers(&host, node_b), vec![node_a]);
        assert_eq!(controller.connected_edges(&host, node_a), vec![edge]);
        assert_eq!(
            controller.port_connections(
                &host,
                NodeGraphPortConnectionsQuery {
                    node_id: node_a,
                    port_id: a_out,
                    side: ConnectionSide::Source,
                },
            ),
            vec![expected],
        );
        assert_eq!(
            controller.node_connections(
                &host,
                NodeGraphNodeConnectionsQuery {
                    node_id: node_a,
                    side: Some(ConnectionSide::Source),
                    port_id: None,
                },
            ),
            vec![expected],
        );
        assert_eq!(
            controller.node_connections(
                &host,
                NodeGraphNodeConnectionsQuery {
                    node_id: node_a,
                    side: None,
                    port_id: Some(a_out),
                },
            ),
            vec![expected],
        );
        assert_eq!(
            controller.node_connections(
                &host,
                NodeGraphNodeConnectionsQuery {
                    node_id: node_b,
                    side: Some(ConnectionSide::Target),
                    port_id: Some(b_in),
                },
            ),
            vec![expected],
        );
        assert!(
            controller
                .node_connections(
                    &host,
                    NodeGraphNodeConnectionsQuery {
                        node_id: node_a,
                        side: Some(ConnectionSide::Target),
                        port_id: Some(a_out),
                    },
                )
                .is_empty()
        );
    }

    #[test]
    fn binding_query_helpers_proxy_controller_queries() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, node_a, a_out, node_b, b_in) = make_test_graph_two_nodes_with_ports();
        let edge = EdgeId::new();
        graph_value.edges.insert(
            edge,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        let graph = host.models.insert(graph_value.clone());
        let view = host.models.insert(NodeGraphViewState::default());
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let binding =
            NodeGraphSurfaceBinding::from_models(graph, view, NodeGraphController::new(store));
        let expected = HandleConnection {
            edge,
            source_node: node_a,
            source_port: a_out,
            target_node: node_b,
            target_port: b_in,
            kind: EdgeKind::Data,
        };

        assert_eq!(binding.outgoers(&host, node_a), vec![node_b]);
        assert_eq!(binding.incomers(&host, node_b), vec![node_a]);
        assert_eq!(binding.connected_edges(&host, node_a), vec![edge]);
        assert_eq!(
            binding.port_connections(
                &host,
                NodeGraphPortConnectionsQuery {
                    node_id: node_a,
                    port_id: a_out,
                    side: ConnectionSide::Source,
                },
            ),
            vec![expected],
        );
        assert_eq!(
            binding.node_connections(
                &host,
                NodeGraphNodeConnectionsQuery {
                    node_id: node_b,
                    side: Some(ConnectionSide::Target),
                    port_id: Some(b_in),
                },
            ),
            vec![expected],
        );
    }

    #[test]
    fn controller_submit_transaction_uses_edit_queue_when_present() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, node_a, _node_b) = make_test_graph_two_nodes();
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let edits = host.models.insert(NodeGraphEditQueue::default());
        let controller =
            NodeGraphController::new(store.clone()).bind_edit_queue_transport(edits.clone());
        let tx = GraphTransaction {
            label: Some("Move Node".to_string()),
            ops: vec![GraphOp::SetNodePos {
                id: node_a,
                from: CanvasPoint { x: 0.0, y: 0.0 },
                to: CanvasPoint { x: 42.0, y: 24.0 },
            }],
        };

        controller.submit_transaction(&mut host, &tx).unwrap();

        let queued = edits
            .read_ref(&host, |queue| queue.pending.clone())
            .ok()
            .unwrap_or_default();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].label.as_deref(), Some("Move Node"));

        let store_pos = store
            .read_ref(&host, |store| {
                store.graph().nodes.get(&node_a).map(|node| node.pos)
            })
            .ok()
            .flatten()
            .expect("store node pos");
        assert_eq!(store_pos, CanvasPoint { x: 0.0, y: 0.0 });
    }

    #[test]
    fn controller_submit_transaction_and_sync_graph_model_dispatches_without_edit_queue() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, node_a, _node_b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value.clone());
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store);
        let tx = GraphTransaction {
            label: Some("Move Node".to_string()),
            ops: vec![GraphOp::SetNodePos {
                id: node_a,
                from: CanvasPoint { x: 0.0, y: 0.0 },
                to: CanvasPoint { x: 84.0, y: 12.0 },
            }],
        };

        controller
            .submit_transaction_and_sync_graph_model(&mut host, &graph, &tx)
            .unwrap();

        let graph_pos = graph
            .read_ref(&host, |graph| graph.nodes.get(&node_a).map(|node| node.pos))
            .ok()
            .flatten()
            .expect("graph node pos");
        assert_eq!(graph_pos, CanvasPoint { x: 84.0, y: 12.0 });
    }

    #[test]
    fn controller_undo_and_redo_sync_models_without_edit_queue() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, node_a, _node_b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value.clone());
        let initial_view = NodeGraphViewState::default();
        let view_state = host.models.insert(initial_view.clone());
        let store = host
            .models
            .insert(NodeGraphStore::new(graph_value, initial_view));
        let controller = NodeGraphController::new(store.clone());
        let tx = GraphTransaction {
            label: Some("Move Node".to_string()),
            ops: vec![GraphOp::SetNodePos {
                id: node_a,
                from: CanvasPoint { x: 0.0, y: 0.0 },
                to: CanvasPoint { x: 64.0, y: 16.0 },
            }],
        };

        controller
            .dispatch_transaction_and_sync_models(&mut host, &graph, &view_state, &tx)
            .unwrap();
        assert!(controller.can_undo(&host));

        let undo = controller
            .undo_and_sync_models(&mut host, &graph, &view_state)
            .unwrap()
            .expect("did undo");
        assert!(!undo.committed.ops.is_empty());
        assert!(controller.can_redo(&host));

        let graph_pos = graph
            .read_ref(&host, |graph| graph.nodes.get(&node_a).map(|node| node.pos))
            .ok()
            .flatten()
            .expect("graph node pos after undo");
        let store_pos = store
            .read_ref(&host, |store| {
                store.graph().nodes.get(&node_a).map(|node| node.pos)
            })
            .ok()
            .flatten()
            .expect("store node pos after undo");
        assert_eq!(graph_pos, CanvasPoint { x: 0.0, y: 0.0 });
        assert_eq!(store_pos, CanvasPoint { x: 0.0, y: 0.0 });

        let redo = controller
            .redo_and_sync_models(&mut host, &graph, &view_state)
            .unwrap()
            .expect("did redo");
        assert!(!redo.committed.ops.is_empty());

        let graph_pos = graph
            .read_ref(&host, |graph| graph.nodes.get(&node_a).map(|node| node.pos))
            .ok()
            .flatten()
            .expect("graph node pos after redo");
        let store_pos = store
            .read_ref(&host, |store| {
                store.graph().nodes.get(&node_a).map(|node| node.pos)
            })
            .ok()
            .flatten()
            .expect("store node pos after redo");
        assert_eq!(graph_pos, CanvasPoint { x: 64.0, y: 16.0 });
        assert_eq!(store_pos, CanvasPoint { x: 64.0, y: 16.0 });
    }
}
