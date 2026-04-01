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
    NodeGraphController, NodeGraphControllerError, NodeGraphEdgeUpdate,
    NodeGraphNodeConnectionsQuery, NodeGraphNodeUpdate, NodeGraphPortConnectionsQuery,
};
use super::declarative::NodeGraphSurfaceProps;
use super::viewport_options::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions};

#[path = "binding_queries.rs"]
mod binding_queries;
#[path = "binding_store_sync.rs"]
mod binding_store_sync;
#[path = "binding_viewport.rs"]
mod binding_viewport;

/// Canonical app-facing binding bundle for the declarative node-graph surface.
///
/// This keeps the controller-first public story explicit while avoiding repeated
/// `graph + view_state + controller` triplets in app code.
#[derive(Debug, Clone)]
pub struct NodeGraphSurfaceBinding {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    store: Model<NodeGraphStore>,
}

impl NodeGraphSurfaceBinding {
    /// Creates the default store-backed binding for declarative node-graph surfaces.
    pub fn new(models: &mut ModelStore, graph: Graph, view_state: NodeGraphViewState) -> Self {
        let graph_model = models.insert(graph.clone());
        let view_state_model = models.insert(view_state.clone());
        let store = models.insert(NodeGraphStore::new(graph, view_state));
        Self::from_models_and_controller(
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
        Self::from_models_and_controller(
            graph_model,
            view_state_model,
            NodeGraphController::new(store_model),
        )
    }

    /// Advanced seam for callers that already own explicit graph/view mirrors and controller state.
    pub fn from_models_and_controller(
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        controller: NodeGraphController,
    ) -> Self {
        Self {
            graph,
            view_state,
            store: controller.store(),
        }
    }

    pub fn graph_model(&self) -> Model<Graph> {
        self.graph.clone()
    }

    pub fn view_state_model(&self) -> Model<NodeGraphViewState> {
        self.view_state.clone()
    }

    /// Advanced lower-level seam for callers that need explicit controller ownership.
    ///
    /// Typical app code should stay on the binding surface. Retained/compat composition can derive
    /// a fresh `NodeGraphController` explicitly from this store handle.
    pub fn store_model(&self) -> Model<NodeGraphStore> {
        self.store.clone()
    }

    pub fn surface_props(&self) -> NodeGraphSurfaceProps {
        NodeGraphSurfaceProps::new(self.clone())
    }

    fn controller(&self) -> NodeGraphController {
        NodeGraphController::new(self.store.clone())
    }

    /// Observes the external graph/view mirrors that the declarative surface keeps in sync.
    pub fn observe<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
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
