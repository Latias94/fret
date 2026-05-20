mod custom_action;
mod delete;
mod open_insert;
mod reroute;
mod retained_cx;

use crate::ui::canvas::widget::*;

pub(in crate::ui::canvas::widget) trait EdgeContextActionCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn open_edge_insert_context_menu<M: NodeGraphCanvasMiddleware>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        edge_id: EdgeId,
        invoked_at: Point,
    );
}

#[derive(Debug)]
enum EdgeContextActionRoute {
    Ignore,
    OpenInsertNodePicker { edge_id: EdgeId, invoked_at: Point },
    InsertReroute { edge_id: EdgeId, invoked_at: Point },
    DeleteEdge { edge_id: EdgeId },
    Custom { edge_id: EdgeId, action_id: u64 },
}

fn edge_context_action_route(
    edge_id: EdgeId,
    invoked_at: Point,
    action: NodeGraphContextMenuAction,
) -> EdgeContextActionRoute {
    match action {
        NodeGraphContextMenuAction::OpenInsertNodePicker => {
            EdgeContextActionRoute::OpenInsertNodePicker {
                edge_id,
                invoked_at,
            }
        }
        NodeGraphContextMenuAction::InsertReroute => EdgeContextActionRoute::InsertReroute {
            edge_id,
            invoked_at,
        },
        NodeGraphContextMenuAction::DeleteEdge => EdgeContextActionRoute::DeleteEdge { edge_id },
        NodeGraphContextMenuAction::Custom(action_id) => {
            EdgeContextActionRoute::Custom { edge_id, action_id }
        }
        NodeGraphContextMenuAction::Command(_)
        | NodeGraphContextMenuAction::InsertNodeCandidate(_) => EdgeContextActionRoute::Ignore,
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn activate_edge_context_action<H: UiHost>(
        &mut self,
        cx: &mut impl EdgeContextActionCx<H>,
        edge_id: EdgeId,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    ) -> bool {
        match edge_context_action_route(edge_id, invoked_at, action) {
            EdgeContextActionRoute::OpenInsertNodePicker {
                edge_id,
                invoked_at,
            } => {
                open_insert::open_edge_insert_context_menu(self, cx, edge_id, invoked_at);
                true
            }
            EdgeContextActionRoute::InsertReroute {
                edge_id,
                invoked_at,
            } => {
                reroute::insert_edge_reroute(self, cx, edge_id, invoked_at);
                true
            }
            EdgeContextActionRoute::DeleteEdge { edge_id } => {
                delete::delete_edge(self, cx, edge_id);
                true
            }
            EdgeContextActionRoute::Custom { edge_id, action_id } => {
                custom_action::apply_custom_edge_context_action(self, cx, edge_id, action_id);
                true
            }
            EdgeContextActionRoute::Ignore => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::Arc;

    use super::{EdgeContextActionCx, EdgeContextActionRoute, edge_context_action_route};
    use crate::REROUTE_KIND;
    use crate::core::{
        CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
        PortCapacity, PortDirection, PortId, PortKey,
    };
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ops::GraphOp;
    use crate::ui::canvas::widget::{
        NodeGraphCanvasMiddleware, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    };
    use crate::ui::presenter::{NodeGraphContextMenuAction, NodeGraphPresenter};
    use fret_core::{AppWindowId, Point, PointerId, Px};
    use fret_runtime::ui_host::{
        CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
    };
    use fret_runtime::{
        ClipboardToken, CommandId, CommandRegistry, DragKindId, DragSession, Effect, FrameId,
        ImageUploadToken, Model, ModelHost, ModelId, ModelStore, ShareSheetToken, TickId,
        TimerToken,
    };

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        effects: Vec<Effect>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestHost {
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
            let existing = self.globals.remove(&type_id);
            let mut value = existing
                .and_then(|value| value.downcast::<T>().ok().map(|value| *value))
                .unwrap_or_else(init);
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for TestHost {
        fn take_changed_models(&mut self) -> Vec<ModelId> {
            self.models.take_changed_models()
        }
    }

    impl CommandsHost for TestHost {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for TestHost {
        fn request_redraw(&mut self, _window: AppWindowId) {}

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl TimeHost for TestHost {
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

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            ImageUploadToken(self.next_image_upload_token)
        }
    }

    impl DragHost for TestHost {
        fn drag(&self, _pointer_id: PointerId) -> Option<&DragSession> {
            None
        }

        fn drag_mut(&mut self, _pointer_id: PointerId) -> Option<&mut DragSession> {
            None
        }

        fn cancel_drag(&mut self, _pointer_id: PointerId) {}

        fn any_drag_session(&self, _predicate: impl FnMut(&DragSession) -> bool) -> bool {
            false
        }

        fn find_drag_pointer_id(
            &self,
            _predicate: impl FnMut(&DragSession) -> bool,
        ) -> Option<PointerId> {
            None
        }

        fn cancel_drag_sessions(
            &mut self,
            _predicate: impl FnMut(&DragSession) -> bool,
        ) -> Vec<PointerId> {
            Vec::new()
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }
    }

    struct StubCx {
        host: TestHost,
        window: Option<AppWindowId>,
        opened_edge_insert: Vec<(EdgeId, Point)>,
    }

    impl StubCx {
        fn new() -> Self {
            Self {
                host: TestHost::default(),
                window: Some(AppWindowId::default()),
                opened_edge_insert: Vec::new(),
            }
        }
    }

    impl EdgeContextActionCx<TestHost> for StubCx {
        fn host(&mut self) -> &mut TestHost {
            &mut self.host
        }

        fn window(&self) -> Option<AppWindowId> {
            self.window
        }

        fn open_edge_insert_context_menu<M: NodeGraphCanvasMiddleware>(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<M>,
            edge_id: EdgeId,
            invoked_at: Point,
        ) {
            self.opened_edge_insert.push((edge_id, invoked_at));
        }
    }

    struct TestCanvas {
        canvas: NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
        graph: Model<Graph>,
        view: Model<NodeGraphViewState>,
        edge_id: EdgeId,
        to_port: PortId,
    }

    fn make_node(kind: &str, x: f32) -> Node {
        Node {
            kind: NodeKindKey::new(kind),
            kind_version: 0,
            pos: CanvasPoint { x, y: 0.0 },
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
            data: serde_json::Value::Null,
        }
    }

    fn make_port(node: NodeId, key: &str, dir: PortDirection) -> Port {
        Port {
            node,
            key: PortKey::new(key),
            dir,
            kind: crate::core::PortKind::Data,
            capacity: match dir {
                PortDirection::In => PortCapacity::Single,
                PortDirection::Out => PortCapacity::Multi,
            },
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        }
    }

    fn graph_with_edge() -> (Graph, EdgeId, PortId) {
        let mut graph = Graph::new(GraphId::new());
        let from_node = NodeId::new();
        let to_node = NodeId::new();
        let from_port = PortId::new();
        let to_port = PortId::new();
        let mut source = make_node("demo.source", 0.0);
        let mut target = make_node("demo.target", 220.0);
        source.ports.push(from_port);
        target.ports.push(to_port);
        graph.nodes.insert(from_node, source);
        graph.nodes.insert(to_node, target);
        graph
            .ports
            .insert(from_port, make_port(from_node, "out", PortDirection::Out));
        graph
            .ports
            .insert(to_port, make_port(to_node, "in", PortDirection::In));

        let edge_id = EdgeId::new();
        graph.edges.insert(
            edge_id,
            Edge {
                kind: EdgeKind::Data,
                from: from_port,
                to: to_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        (graph, edge_id, to_port)
    }

    fn test_canvas(cx: &mut StubCx) -> TestCanvas {
        let (graph_value, edge_id, to_port) = graph_with_edge();
        let graph = cx.host.models.insert(graph_value);
        let mut view_state = NodeGraphViewState::default();
        view_state.selected_edges.push(edge_id);
        let view = cx.host.models.insert(view_state);
        let editor_config = cx.host.models.insert(NodeGraphEditorConfig::default());
        let canvas = NodeGraphCanvasWith::new_with_middleware(
            graph.clone(),
            view.clone(),
            editor_config,
            NoopNodeGraphCanvasMiddleware,
        );

        TestCanvas {
            canvas,
            graph,
            view,
            edge_id,
            to_port,
        }
    }

    fn graph_snapshot(cx: &StubCx, graph: &Model<Graph>) -> Graph {
        graph
            .read_ref(&cx.host, Clone::clone)
            .expect("test graph should be readable")
    }

    fn view_snapshot(cx: &StubCx, view: &Model<NodeGraphViewState>) -> NodeGraphViewState {
        view.read_ref(&cx.host, Clone::clone)
            .expect("test view should be readable")
    }

    #[derive(Default)]
    struct CustomPresenter {
        calls: Rc<RefCell<Vec<(EdgeId, u64)>>>,
    }

    impl NodeGraphPresenter for CustomPresenter {
        fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
            graph
                .nodes
                .get(&node)
                .map(|node| Arc::<str>::from(node.kind.0.clone()))
                .unwrap_or_else(|| Arc::<str>::from("<missing node>"))
        }

        fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
            graph
                .ports
                .get(&port)
                .map(|port| Arc::<str>::from(port.key.0.clone()))
                .unwrap_or_else(|| Arc::<str>::from("<missing port>"))
        }

        fn on_edge_context_menu_action(
            &mut self,
            graph: &Graph,
            edge: EdgeId,
            action: u64,
        ) -> Option<Vec<GraphOp>> {
            self.calls.borrow_mut().push((edge, action));
            let from = graph.edges.get(&edge)?.selectable;
            Some(vec![GraphOp::SetEdgeSelectable {
                id: edge,
                from,
                to: Some(false),
            }])
        }
    }

    #[test]
    fn edge_action_routes_preserve_edge_id_and_invocation_point() {
        let edge_id = EdgeId::new();
        let invoked_at = Point::new(Px(120.0), Px(48.0));

        let open_insert = edge_context_action_route(
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::OpenInsertNodePicker,
        );
        let reroute = edge_context_action_route(
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::InsertReroute,
        );

        assert!(matches!(
            open_insert,
            EdgeContextActionRoute::OpenInsertNodePicker {
                edge_id: route_edge,
                invoked_at: route_invoked_at,
            } if route_edge == edge_id && route_invoked_at == invoked_at
        ));
        assert!(matches!(
            reroute,
            EdgeContextActionRoute::InsertReroute {
                edge_id: route_edge,
                invoked_at: route_invoked_at,
            } if route_edge == edge_id && route_invoked_at == invoked_at
        ));
    }

    #[test]
    fn delete_and_custom_edge_actions_route_to_edge_specific_executors() {
        let edge_id = EdgeId::new();
        let invoked_at = Point::new(Px(10.0), Px(20.0));

        let delete_route =
            edge_context_action_route(edge_id, invoked_at, NodeGraphContextMenuAction::DeleteEdge);
        let custom_route =
            edge_context_action_route(edge_id, invoked_at, NodeGraphContextMenuAction::Custom(7));

        assert!(matches!(
            delete_route,
            EdgeContextActionRoute::DeleteEdge { edge_id: route_edge } if route_edge == edge_id
        ));
        assert!(matches!(
            custom_route,
            EdgeContextActionRoute::Custom {
                edge_id: route_edge,
                action_id: 7,
            } if route_edge == edge_id
        ));
    }

    #[test]
    fn non_edge_actions_are_ignored_by_edge_executor() {
        let edge_id = EdgeId::new();
        let invoked_at = Point::new(Px(1.0), Px(2.0));

        let command_route = edge_context_action_route(
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::Command(CommandId::from("demo.command")),
        );
        let insert_candidate_route = edge_context_action_route(
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::InsertNodeCandidate(0),
        );

        assert!(matches!(command_route, EdgeContextActionRoute::Ignore));
        assert!(matches!(
            insert_candidate_route,
            EdgeContextActionRoute::Ignore
        ));
    }

    #[test]
    fn open_insert_action_delegates_to_context_adapter() {
        let mut cx = StubCx::new();
        let TestCanvas {
            mut canvas,
            edge_id,
            ..
        } = test_canvas(&mut cx);
        let invoked_at = Point::new(Px(12.0), Px(24.0));

        let handled = canvas.activate_edge_context_action(
            &mut cx,
            edge_id,
            invoked_at,
            NodeGraphContextMenuAction::OpenInsertNodePicker,
        );

        assert!(handled);
        assert_eq!(cx.opened_edge_insert, vec![(edge_id, invoked_at)]);
    }

    #[test]
    fn delete_edge_action_removes_edge_and_selection() {
        let mut cx = StubCx::new();
        let TestCanvas {
            mut canvas,
            graph,
            view,
            edge_id,
            ..
        } = test_canvas(&mut cx);

        let handled = canvas.activate_edge_context_action(
            &mut cx,
            edge_id,
            Point::new(Px(1.0), Px(2.0)),
            NodeGraphContextMenuAction::DeleteEdge,
        );

        assert!(handled);
        assert!(!graph_snapshot(&cx, &graph).edges.contains_key(&edge_id));
        assert!(!view_snapshot(&cx, &view).selected_edges.contains(&edge_id));
    }

    #[test]
    fn insert_reroute_action_splits_edge_and_selects_inserted_node() {
        let mut cx = StubCx::new();
        let TestCanvas {
            mut canvas,
            graph,
            view,
            edge_id,
            to_port,
        } = test_canvas(&mut cx);

        let handled = canvas.activate_edge_context_action(
            &mut cx,
            edge_id,
            Point::new(Px(64.0), Px(32.0)),
            NodeGraphContextMenuAction::InsertReroute,
        );

        let graph = graph_snapshot(&cx, &graph);
        let view = view_snapshot(&cx, &view);
        let selected_node = view
            .selected_nodes
            .first()
            .copied()
            .expect("reroute insert should select the inserted node");

        assert!(handled);
        assert_eq!(graph.edges.len(), 2);
        assert_ne!(
            graph
                .edges
                .get(&edge_id)
                .expect("original edge should be preserved")
                .to,
            to_port
        );
        assert_eq!(
            graph
                .nodes
                .get(&selected_node)
                .expect("selected node should exist")
                .kind,
            NodeKindKey::new(REROUTE_KIND)
        );
    }

    #[test]
    fn custom_edge_action_applies_presenter_ops() {
        let mut cx = StubCx::new();
        let TestCanvas {
            mut canvas,
            graph,
            edge_id,
            ..
        } = test_canvas(&mut cx);
        let calls = Rc::<RefCell<Vec<(EdgeId, u64)>>>::default();
        canvas = canvas.with_presenter(CustomPresenter {
            calls: calls.clone(),
        });

        let handled = canvas.activate_edge_context_action(
            &mut cx,
            edge_id,
            Point::new(Px(1.0), Px(2.0)),
            NodeGraphContextMenuAction::Custom(42),
        );

        let graph = graph_snapshot(&cx, &graph);
        assert!(handled);
        assert_eq!(&*calls.borrow(), &[(edge_id, 42)]);
        assert_eq!(
            graph
                .edges
                .get(&edge_id)
                .expect("edge should still exist")
                .selectable,
            Some(false)
        );
    }

    #[test]
    fn ignored_edge_actions_are_side_effect_free() {
        let mut cx = StubCx::new();
        let TestCanvas {
            mut canvas,
            graph,
            edge_id,
            ..
        } = test_canvas(&mut cx);
        let before = graph_snapshot(&cx, &graph).edges.len();

        let command_handled = canvas.activate_edge_context_action(
            &mut cx,
            edge_id,
            Point::new(Px(1.0), Px(2.0)),
            NodeGraphContextMenuAction::Command(CommandId::from("demo.command")),
        );
        let candidate_handled = canvas.activate_edge_context_action(
            &mut cx,
            edge_id,
            Point::new(Px(1.0), Px(2.0)),
            NodeGraphContextMenuAction::InsertNodeCandidate(0),
        );

        assert!(!command_handled);
        assert!(!candidate_handled);
        assert!(cx.opened_edge_insert.is_empty());
        assert_eq!(graph_snapshot(&cx, &graph).edges.len(), before);
    }
}
