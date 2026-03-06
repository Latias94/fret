use fret_runtime::{Model, ModelStore};
use fret_ui::UiHost;
use fret_ui::action::UiActionHost;

use crate::core::{CanvasPoint, EdgeId, Graph, NodeId};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::runtime::store::{DispatchError, DispatchOutcome, NodeGraphStore};
use crate::runtime::utils::{get_connected_edges, get_incomers, get_outgoers};

use super::view_queue::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions, NodeGraphViewQueue};

#[derive(Debug, thiserror::Error)]
pub enum NodeGraphControllerError {
    #[error("store unavailable")]
    StoreUnavailable,
    #[error(transparent)]
    Dispatch(#[from] DispatchError),
}

#[derive(Debug, Clone)]
pub struct NodeGraphController {
    store: Model<NodeGraphStore>,
    view_queue: Option<Model<NodeGraphViewQueue>>,
}

impl NodeGraphController {
    pub fn new(store: Model<NodeGraphStore>) -> Self {
        Self {
            store,
            view_queue: None,
        }
    }

    pub fn with_view_queue(mut self, queue: Model<NodeGraphViewQueue>) -> Self {
        self.view_queue = Some(queue);
        self
    }

    pub fn store(&self) -> Model<NodeGraphStore> {
        self.store.clone()
    }

    pub fn view_queue(&self) -> Option<Model<NodeGraphViewQueue>> {
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

    pub fn replace_graph<H: UiHost>(
        &self,
        host: &mut H,
        graph: Graph,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_graph_in_models(host.models_mut(), graph)
    }

    pub fn replace_view_state<H: UiHost>(
        &self,
        host: &mut H,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.replace_view_state_in_models(host.models_mut(), view_state)
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

        self.store
            .update(host, |store, _cx| {
                store.set_viewport(pan, zoom);
            })
            .is_ok()
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
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{AppWindowId, Point};
    use fret_runtime::ui_host::{
        CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, DragKindId, DragSession, DragSessionId, Effect, FrameId,
        ModelHost, ModelStore, ShareSheetToken, TickId, TimerToken,
    };
    use serde_json::Value;

    use super::NodeGraphController;
    use crate::core::{
        CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
        PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };
    use crate::io::NodeGraphViewState;
    use crate::ops::{GraphOp, GraphTransaction};
    use crate::runtime::store::NodeGraphStore;
    use crate::ui::{NodeGraphSetViewportOptions, NodeGraphViewQueue, NodeGraphViewRequest};

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
    fn controller_set_viewport_uses_queue_when_present() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let queue = host.models.insert(NodeGraphViewQueue::default());
        let controller = NodeGraphController::new(store.clone()).with_view_queue(queue.clone());

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

        assert_eq!(controller.outgoers(&host, node_a), vec![node_b]);
        assert_eq!(controller.incomers(&host, node_b), vec![node_a]);
        assert_eq!(controller.connected_edges(&host, node_a), vec![edge]);
    }
}
