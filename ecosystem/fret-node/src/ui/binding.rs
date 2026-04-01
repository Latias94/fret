use fret_core::Rect;
use fret_runtime::{Model, ModelStore};
use fret_ui::action::UiActionHost;
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::core::{CanvasPoint, EdgeId, Graph, GroupId, NodeId};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::runtime::lookups::HandleConnection;
use crate::runtime::store::{DispatchOutcome, NodeGraphStore};

use super::controller::{
    NodeGraphController, NodeGraphControllerError, NodeGraphNodeConnectionsQuery,
    NodeGraphPortConnectionsQuery,
};
use super::declarative::NodeGraphSurfaceProps;
use super::viewport_options::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions};

/// Canonical app-facing binding bundle for the declarative node-graph surface.
///
/// This keeps the controller-first public story explicit while avoiding repeated
/// `graph + view_state + controller` triplets in app code.
#[derive(Debug, Clone)]
pub struct NodeGraphSurfaceBinding {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    controller: NodeGraphController,
}

impl NodeGraphSurfaceBinding {
    /// Creates the default store-backed binding for declarative node-graph surfaces.
    pub fn new(models: &mut ModelStore, graph: Graph, view_state: NodeGraphViewState) -> Self {
        let graph_model = models.insert(graph.clone());
        let view_state_model = models.insert(view_state.clone());
        let store = models.insert(NodeGraphStore::new(graph, view_state));
        Self::from_models(
            graph_model,
            view_state_model,
            NodeGraphController::new(store),
        )
    }

    /// Creates a declarative surface binding from an already-configured store.
    pub fn from_store(models: &mut ModelStore, store: NodeGraphStore) -> Self {
        let graph = store.graph().clone();
        let view_state = store.view_state().clone();
        let store_model = models.insert(store);
        let graph_model = models.insert(graph);
        let view_state_model = models.insert(view_state);
        Self::from_models(
            graph_model,
            view_state_model,
            NodeGraphController::new(store_model),
        )
    }

    /// Advanced seam for callers that already own explicit graph/view models.
    pub fn from_models(
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        controller: NodeGraphController,
    ) -> Self {
        Self {
            graph,
            view_state,
            controller,
        }
    }

    pub fn graph_model(&self) -> Model<Graph> {
        self.graph.clone()
    }

    pub fn view_state_model(&self) -> Model<NodeGraphViewState> {
        self.view_state.clone()
    }

    /// Advanced escape hatch for controller-only helpers not yet surfaced on the binding.
    pub fn controller(&self) -> NodeGraphController {
        self.controller.clone()
    }

    pub fn store_model(&self) -> Model<NodeGraphStore> {
        self.controller.store()
    }

    pub fn surface_props(&self) -> NodeGraphSurfaceProps {
        NodeGraphSurfaceProps::new(self.clone())
    }

    /// Reads the current viewport from the authoritative store-backed controller.
    pub fn viewport<H: UiHost>(&self, host: &H) -> (CanvasPoint, f32) {
        self.controller.viewport(host)
    }

    /// Clones the current graph snapshot from the authoritative store.
    pub fn graph_snapshot<H: UiHost>(&self, host: &H) -> Option<Graph> {
        self.controller.graph_snapshot(host)
    }

    /// Clones the current view-state snapshot from the authoritative store.
    pub fn view_state_snapshot<H: UiHost>(&self, host: &H) -> Option<NodeGraphViewState> {
        self.controller.view_state_snapshot(host)
    }

    /// Observes the external graph/view mirrors that the declarative surface keeps in sync.
    pub fn observe<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
    }

    /// Re-syncs the graph/view mirrors from the authoritative store.
    pub fn sync_from_store<H: UiHost>(&self, host: &mut H) -> bool {
        self.controller
            .sync_models_from_store(host, &self.graph, &self.view_state)
    }

    /// Re-syncs the graph/view mirrors from the authoritative store.
    pub fn sync_from_store_action_host(&self, host: &mut dyn UiActionHost) -> bool {
        self.controller
            .sync_models_from_store_action_host(host, &self.graph, &self.view_state)
    }

    /// Dispatches a transaction and keeps the external graph/view mirrors in sync.
    pub fn dispatch_transaction<H: UiHost>(
        &self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        self.controller.dispatch_transaction_and_sync_models(
            host,
            &self.graph,
            &self.view_state,
            tx,
        )
    }

    /// Dispatches a transaction from an object-safe action hook and keeps the external graph/view
    /// mirrors in sync.
    pub fn dispatch_transaction_action_host(
        &self,
        host: &mut dyn UiActionHost,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        self.controller
            .dispatch_transaction_and_sync_models_action_host(
                host,
                &self.graph,
                &self.view_state,
                tx,
            )
    }

    /// Submits a transaction and keeps the external graph/view mirrors in sync.
    pub fn submit_transaction<H: UiHost>(
        &self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller
            .submit_transaction_and_sync_models(host, &self.graph, &self.view_state, tx)
    }

    /// Submits a transaction from an object-safe action hook and keeps the external graph/view
    /// mirrors in sync.
    pub fn submit_transaction_action_host(
        &self,
        host: &mut dyn UiActionHost,
        tx: &GraphTransaction,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller.submit_transaction_action_host(host, tx)?;
        let _ = self.sync_from_store_action_host(host);
        Ok(())
    }

    /// Replaces the authoritative graph and keeps the external graph/view mirrors in sync.
    pub fn replace_graph<H: UiHost>(
        &self,
        host: &mut H,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller
            .replace_graph_and_sync_models(host, &self.graph, &self.view_state, graph)
    }

    /// Replaces the authoritative graph from an object-safe action hook and keeps the external
    /// graph/view mirrors in sync.
    pub fn replace_graph_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller.replace_graph_and_sync_models_action_host(
            host,
            &self.graph,
            &self.view_state,
            graph,
        )
    }

    /// Replaces the entire document snapshot (graph + view state), clears history, and keeps the
    /// external graph/view mirrors in sync.
    pub fn replace_document<H: UiHost>(
        &self,
        host: &mut H,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller.replace_document_and_sync_models(
            host,
            &self.graph,
            &self.view_state,
            graph,
            view_state,
        )
    }

    /// Replaces the entire document snapshot from an object-safe action hook, clears history, and
    /// keeps the external graph/view mirrors in sync.
    pub fn replace_document_action_host(
        &self,
        host: &mut dyn UiActionHost,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller
            .replace_document_and_sync_models_action_host(
                host,
                &self.graph,
                &self.view_state,
                graph,
                view_state,
            )
    }

    /// Replaces the authoritative view state and keeps the external view model in sync.
    pub fn replace_view_state<H: UiHost>(
        &self,
        host: &mut H,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller
            .replace_view_state_and_sync_model(host, &self.view_state, view_state)
    }

    /// Replaces the authoritative view state from an object-safe action hook and keeps the
    /// external view model in sync.
    pub fn replace_view_state_action_host(
        &self,
        host: &mut dyn UiActionHost,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller
            .replace_view_state_and_sync_model_action_host(host, &self.view_state, view_state)
    }

    /// Replaces the authoritative selection and keeps the external view model in sync.
    pub fn set_selection<H: UiHost>(
        &self,
        host: &mut H,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        groups: Vec<GroupId>,
    ) -> Result<(), NodeGraphControllerError> {
        self.controller.set_selection_and_sync_view_model(
            host,
            &self.view_state,
            nodes,
            edges,
            groups,
        )
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
        self.controller
            .set_selection_and_sync_view_model_action_host(
                host,
                &self.view_state,
                nodes,
                edges,
                groups,
            )
    }

    /// Applies a viewport change through the bound controller.
    pub fn set_viewport<H: UiHost>(&self, host: &mut H, pan: CanvasPoint, zoom: f32) -> bool {
        let applied = self.controller.set_viewport(host, pan, zoom);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Applies a viewport change with store-first viewport options through the bound controller.
    pub fn set_viewport_with_options<H: UiHost>(
        &self,
        host: &mut H,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let applied = self
            .controller
            .set_viewport_with_options(host, pan, zoom, options);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Applies a viewport change from an object-safe action hook.
    pub fn set_viewport_action_host(
        &self,
        host: &mut dyn UiActionHost,
        pan: CanvasPoint,
        zoom: f32,
    ) -> bool {
        let applied = self.controller.set_viewport_action_host(host, pan, zoom);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Applies a viewport change with store-first viewport options from an object-safe action hook.
    pub fn set_viewport_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let applied = self
            .controller
            .set_viewport_with_options_action_host(host, pan, zoom, options);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Centers a canvas point inside explicit bounds through the bound controller.
    pub fn set_center_in_bounds<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
    ) -> bool {
        let applied = self.controller.set_center_in_bounds(host, bounds, center);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Centers a canvas point inside explicit bounds from an object-safe action hook.
    pub fn set_center_in_bounds_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        center: CanvasPoint,
    ) -> bool {
        let applied = self
            .controller
            .set_center_in_bounds_action_host(host, bounds, center);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Centers a canvas point inside explicit bounds with store-first viewport options.
    pub fn set_center_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let applied = self
            .controller
            .set_center_in_bounds_with_options(host, bounds, center, zoom, options);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Centers a canvas point inside explicit bounds with store-first viewport options from an
    /// object-safe action hook.
    pub fn set_center_in_bounds_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let applied = self
            .controller
            .set_center_in_bounds_with_options_action_host(host, bounds, center, zoom, options);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Fits the viewport to the given nodes inside explicit bounds.
    pub fn fit_view_nodes_in_bounds<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        nodes: Vec<NodeId>,
    ) -> bool {
        let applied = self
            .controller
            .fit_view_nodes_in_bounds(host, bounds, nodes);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Fits the viewport to the given nodes inside explicit bounds with store-first fit-view
    /// options.
    pub fn fit_view_nodes_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        let applied = self
            .controller
            .fit_view_nodes_in_bounds_with_options(host, bounds, nodes, options);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Fits the viewport to the given nodes inside explicit bounds from an object-safe action
    /// hook.
    pub fn fit_view_nodes_in_bounds_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        nodes: Vec<NodeId>,
    ) -> bool {
        let applied = self
            .controller
            .fit_view_nodes_in_bounds_action_host(host, bounds, nodes);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Fits the viewport to the given nodes inside explicit bounds with store-first fit-view
    /// options from an object-safe action hook.
    pub fn fit_view_nodes_in_bounds_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        let applied = self
            .controller
            .fit_view_nodes_in_bounds_with_options_action_host(host, bounds, nodes, options);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Returns the outgoing neighbor node ids for the given node.
    pub fn outgoers<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<NodeId> {
        self.controller.outgoers(host, node)
    }

    /// Returns the incoming neighbor node ids for the given node.
    pub fn incomers<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<NodeId> {
        self.controller.incomers(host, node)
    }

    /// Returns the edge ids incident to the given node.
    pub fn connected_edges<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<EdgeId> {
        self.controller.connected_edges(host, node)
    }

    /// Returns handle-level connections for the given node-side/port query.
    pub fn port_connections<H: UiHost>(
        &self,
        host: &H,
        query: NodeGraphPortConnectionsQuery,
    ) -> Vec<HandleConnection> {
        self.controller.port_connections(host, query)
    }

    /// Returns node-level connections for the given query.
    pub fn node_connections<H: UiHost>(
        &self,
        host: &H,
        query: NodeGraphNodeConnectionsQuery,
    ) -> Vec<HandleConnection> {
        self.controller.node_connections(host, query)
    }

    /// Returns whether the bound store currently has undo history.
    pub fn can_undo<H: UiHost>(&self, host: &H) -> bool {
        self.controller.can_undo(host)
    }

    /// Returns whether the bound store currently has redo history.
    pub fn can_redo<H: UiHost>(&self, host: &H) -> bool {
        self.controller.can_redo(host)
    }

    /// Undoes the last committed transaction and keeps the external graph/view mirrors in sync.
    pub fn undo<H: UiHost>(
        &self,
        host: &mut H,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        self.controller
            .undo_and_sync_models(host, &self.graph, &self.view_state)
    }

    /// Undoes the last committed transaction from an object-safe action hook and keeps the external
    /// graph/view mirrors in sync.
    pub fn undo_action_host(
        &self,
        host: &mut dyn UiActionHost,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        self.controller
            .undo_and_sync_models_action_host(host, &self.graph, &self.view_state)
    }

    /// Redoes the last undone transaction and keeps the external graph/view mirrors in sync.
    pub fn redo<H: UiHost>(
        &self,
        host: &mut H,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        self.controller
            .redo_and_sync_models(host, &self.graph, &self.view_state)
    }

    /// Redoes the last undone transaction from an object-safe action hook and keeps the external
    /// graph/view mirrors in sync.
    pub fn redo_action_host(
        &self,
        host: &mut dyn UiActionHost,
    ) -> Result<Option<DispatchOutcome>, NodeGraphControllerError> {
        self.controller
            .redo_and_sync_models_action_host(host, &self.graph, &self.view_state)
    }

    fn sync_view_state_after_viewport_update<H: UiHost>(
        &self,
        host: &mut H,
        applied: bool,
    ) -> bool {
        if applied {
            let _ = self
                .controller
                .sync_view_state_model_from_store(host, &self.view_state);
        }
        applied
    }

    fn sync_view_state_after_viewport_update_action_host(
        &self,
        host: &mut dyn UiActionHost,
        applied: bool,
    ) -> bool {
        if applied {
            let _ = self
                .controller
                .sync_view_state_model_from_store_action_host(host, &self.view_state);
        }
        applied
    }
}

#[cfg(test)]
mod tests {
    use super::NodeGraphSurfaceBinding;
    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey, StickyNote,
        StickyNoteId,
    };
    use crate::io::NodeGraphViewState;
    use crate::ops::{GraphOp, GraphTransaction};
    use crate::runtime::store::NodeGraphStore;
    use fret_core::AppWindowId;
    use fret_runtime::{Effect, ModelStore, TimerToken};
    use serde_json::Value;

    #[derive(Default)]
    struct TestActionHost {
        models: ModelStore,
        effects: Vec<Effect>,
        next_token: u64,
    }

    impl fret_ui::action::UiActionHost for TestActionHost {
        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn request_redraw(&mut self, _window: AppWindowId) {}

        fn next_timer_token(&mut self) -> TimerToken {
            self.next_token = self.next_token.saturating_add(1);
            TimerToken(self.next_token)
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.next_token = self.next_token.saturating_add(1);
            fret_runtime::ClipboardToken(self.next_token)
        }

        fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
            self.next_token = self.next_token.saturating_add(1);
            fret_runtime::ShareSheetToken(self.next_token)
        }
    }

    fn test_node(pos: CanvasPoint) -> Node {
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
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        }
    }

    #[test]
    fn new_binding_seeds_graph_view_and_store_models() {
        let mut models = ModelStore::default();
        let graph = Graph::new(GraphId::from_u128(0x9001));
        let view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 12.0, y: 34.0 },
            zoom: 1.5,
            ..NodeGraphViewState::default()
        };

        let binding = NodeGraphSurfaceBinding::new(&mut models, graph.clone(), view_state.clone());

        let graph_id = models
            .read(&binding.graph_model(), |value| value.graph_id)
            .unwrap();
        let pan = models
            .read(&binding.view_state_model(), |value| value.pan)
            .unwrap();
        let zoom = models
            .read(&binding.store_model(), |store| store.view_state().zoom)
            .unwrap();

        assert_eq!(graph_id, graph.graph_id);
        assert_eq!(pan, view_state.pan);
        assert_eq!(zoom, view_state.zoom);
    }

    #[test]
    fn from_store_clones_initial_store_state_into_surface_models() {
        let mut models = ModelStore::default();
        let graph = Graph::new(GraphId::from_u128(0x9002));
        let view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 88.0, y: 55.0 },
            zoom: 0.75,
            ..NodeGraphViewState::default()
        };
        let store = NodeGraphStore::new(graph.clone(), view_state.clone());

        let binding = NodeGraphSurfaceBinding::from_store(&mut models, store);

        let graph_id = models
            .read(&binding.graph_model(), |value| value.graph_id)
            .unwrap();
        let pan = models
            .read(&binding.view_state_model(), |value| value.pan)
            .unwrap();
        let zoom = models
            .read(&binding.store_model(), |store| store.view_state().zoom)
            .unwrap();

        assert_eq!(graph_id, graph.graph_id);
        assert_eq!(pan, view_state.pan);
        assert_eq!(zoom, view_state.zoom);
    }

    #[test]
    fn dispatch_transaction_action_host_and_history_sync_bound_models() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x9003)),
            NodeGraphViewState::default(),
        );

        let note_id = StickyNoteId::from_u128(0x9004);
        let note = StickyNote {
            text: "note".into(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 10.0, y: 20.0 },
                size: CanvasSize {
                    width: 120.0,
                    height: 80.0,
                },
            },
            color: Some("amber".into()),
        };
        let mut tx = GraphTransaction::new();
        tx.push(GraphOp::AddStickyNote {
            id: note_id,
            note: note.clone(),
        });

        let outcome = binding
            .dispatch_transaction_action_host(&mut host, &tx)
            .expect("dispatch succeeds");
        assert_eq!(outcome.committed.ops.len(), 1);
        assert!(matches!(
            outcome.committed.ops.first(),
            Some(GraphOp::AddStickyNote { id, .. }) if *id == note_id
        ));
        assert!(outcome.changes.nodes.is_empty());
        assert!(outcome.changes.edges.is_empty());
        assert!(
            host.models
                .read(&binding.graph_model(), |graph| graph
                    .sticky_notes
                    .contains_key(&note_id))
                .expect("graph model readable")
        );

        let undo = binding.undo_action_host(&mut host).expect("undo succeeds");
        assert!(undo.is_some());
        assert!(
            !host
                .models
                .read(&binding.graph_model(), |graph| graph
                    .sticky_notes
                    .contains_key(&note_id))
                .expect("graph model readable")
        );

        let redo = binding.redo_action_host(&mut host).expect("redo succeeds");
        assert!(redo.is_some());
        assert!(
            host.models
                .read(&binding.graph_model(), |graph| graph
                    .sticky_notes
                    .contains_key(&note_id))
                .expect("graph model readable")
        );
    }

    #[test]
    fn replace_graph_and_selection_action_host_sync_bound_models() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x9005)),
            NodeGraphViewState::default(),
        );

        let node_id = NodeId::from_u128(0x9006);
        let mut graph = Graph::new(GraphId::from_u128(0x9005));
        graph
            .nodes
            .insert(node_id, test_node(CanvasPoint { x: 64.0, y: 96.0 }));

        binding
            .replace_graph_action_host(&mut host, graph)
            .expect("replace graph succeeds");
        assert!(
            host.models
                .read(&binding.graph_model(), |value| value
                    .nodes
                    .contains_key(&node_id))
                .expect("graph model readable")
        );

        binding
            .set_selection_action_host(&mut host, vec![node_id], Vec::new(), Vec::new())
            .expect("set selection succeeds");
        assert_eq!(
            host.models
                .read(&binding.view_state_model(), |state| state
                    .selected_nodes
                    .clone())
                .expect("view model readable"),
            vec![node_id]
        );
    }

    #[test]
    fn replace_view_state_action_host_syncs_bound_view_model() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x9007)),
            NodeGraphViewState::default(),
        );

        let next = NodeGraphViewState {
            pan: CanvasPoint { x: 21.0, y: 34.0 },
            zoom: 1.75,
            ..NodeGraphViewState::default()
        };

        binding
            .replace_view_state_action_host(&mut host, next.clone())
            .expect("replace view state succeeds");

        let (pan, zoom) = host
            .models
            .read(&binding.view_state_model(), |state| (state.pan, state.zoom))
            .expect("view model readable");
        assert_eq!(pan, next.pan);
        assert_eq!(zoom, next.zoom);
    }
}
