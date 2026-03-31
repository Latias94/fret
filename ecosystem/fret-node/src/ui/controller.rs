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

#[path = "controller_queries.rs"]
mod controller_queries;
#[path = "controller_store_sync.rs"]
mod controller_store_sync;
#[path = "controller_viewport.rs"]
mod controller_viewport;

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
    fn controller_set_viewport_action_host_falls_back_to_store_without_queue() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store);

        assert!(controller.set_viewport_action_host(
            &mut host,
            CanvasPoint { x: -12.0, y: 8.0 },
            1.75,
        ));
        assert_eq!(
            controller.viewport(&host),
            (CanvasPoint { x: -12.0, y: 8.0 }, 1.75)
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
    fn controller_set_center_in_bounds_uses_bound_queue_and_current_zoom_when_omitted() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let mut store_view = NodeGraphViewState::default();
        store_view.zoom = 2.0;
        let store = host
            .models
            .insert(NodeGraphStore::new(graph_value, store_view));
        let queue = host.models.insert(NodeGraphViewQueue::default());
        let controller = NodeGraphController::new(store).bind_view_queue_transport(queue.clone());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        assert!(controller.set_center_in_bounds_with_options(
            &mut host,
            bounds,
            CanvasPoint { x: 10.0, y: 20.0 },
            None,
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
        assert!((zoom - 2.0).abs() <= 1.0e-6);
        assert!((pan.x - (800.0 / 4.0 - 10.0)).abs() <= 1.0e-6);
        assert!((pan.y - (600.0 / 4.0 - 20.0)).abs() <= 1.0e-6);
    }

    #[test]
    fn controller_set_center_in_bounds_honors_explicit_zoom_override_when_queue_bound() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let queue = host.models.insert(NodeGraphViewQueue::default());
        let controller = NodeGraphController::new(store).bind_view_queue_transport(queue.clone());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        assert!(controller.set_center_in_bounds_with_options(
            &mut host,
            bounds,
            CanvasPoint { x: 10.0, y: 20.0 },
            Some(4.0),
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
        assert!((zoom - 4.0).abs() <= 1.0e-6);
        assert!((pan.x - (800.0 / 8.0 - 10.0)).abs() <= 1.0e-6);
        assert!((pan.y - (600.0 / 8.0 - 20.0)).abs() <= 1.0e-6);
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
    fn controller_fit_view_nodes_in_bounds_action_host_falls_back_to_store_without_queue() {
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

        assert!(
            controller.fit_view_nodes_in_bounds_with_options_action_host(
                &mut host,
                bounds,
                vec![node_a, node_b],
                options.clone(),
            )
        );

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
