use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

use fret_core::{AppWindowId, Point, Px, Rect, Size, TextBlobId};
use fret_runtime::ui_host::{
    CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
};
use fret_runtime::{
    ClipboardToken, CommandRegistry, DragKindId, DragSession, DragSessionId, Effect, FrameId,
};
use fret_runtime::{ModelHost, ModelStore, TickId, TimerToken};
use serde_json::Value;

use crate::core::{
    CanvasPoint, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey, Port, PortCapacity,
    PortDirection, PortId, PortKey, PortKind,
};

#[derive(Default)]
pub(super) struct NullServices;

impl fret_core::TextService for NullServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: fret_core::TextConstraints,
    ) -> (TextBlobId, fret_core::TextMetrics) {
        (
            TextBlobId::default(),
            fret_core::TextMetrics {
                size: Size::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for NullServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for NullServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

#[derive(Default)]
pub(super) struct TestUiHostImpl {
    pub(super) globals: HashMap<TypeId, Box<dyn Any>>,
    pub(super) models: ModelStore,
    pub(super) commands: CommandRegistry,
    pub(super) redraw: HashSet<AppWindowId>,
    pub(super) effects: Vec<Effect>,
    pub(super) drag: Option<DragSession>,
    pub(super) tick_id: TickId,
    pub(super) frame_id: FrameId,
    pub(super) next_timer_token: u64,
    pub(super) next_clipboard_token: u64,
    pub(super) next_image_upload_token: u64,
}

impl GlobalsHost for TestUiHostImpl {
    fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
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

        // Avoid aliasing `&mut self` by temporarily removing the value.
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
    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw.insert(window);
    }

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

    fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
        self.drag.as_ref().is_some_and(|d| predicate(d))
    }

    fn find_drag_pointer_id(
        &self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<fret_core::PointerId> {
        self.drag
            .as_ref()
            .filter(|d| predicate(d))
            .map(|d| d.pointer_id)
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

pub(super) fn event_cx<'a>(
    host: &'a mut TestUiHostImpl,
    services: &'a mut NullServices,
    bounds: Rect,
    prevented_default_actions: &'a mut fret_runtime::DefaultActionSet,
) -> fret_ui::retained_bridge::EventCx<'a, TestUiHostImpl> {
    fret_ui::retained_bridge::EventCx {
        app: host,
        services,
        node: fret_core::NodeId::default(),
        layer_root: None,
        window: None,
        input_ctx: fret_runtime::InputContext::default(),
        pointer_id: None,
        prevented_default_actions,
        children: &[],
        focus: None,
        captured: None,
        bounds,
        invalidations: Vec::new(),
        requested_focus: None,
        requested_capture: None,
        requested_cursor: None,
        notify_requested: false,
        notify_requested_location: None,
        stop_propagation: false,
    }
}

pub(super) fn command_cx<'a>(
    host: &'a mut TestUiHostImpl,
    services: &'a mut NullServices,
    tree: &'a mut fret_ui::UiTree<TestUiHostImpl>,
) -> fret_ui::retained_bridge::CommandCx<'a, TestUiHostImpl> {
    fret_ui::retained_bridge::CommandCx {
        app: host,
        services,
        tree,
        node: fret_core::NodeId::default(),
        window: None,
        input_ctx: fret_runtime::InputContext::default(),
        focus: None,
        invalidations: Vec::new(),
        requested_focus: None,
        stop_propagation: false,
    }
}

fn test_node(
    kind: NodeKindKey,
    pos: CanvasPoint,
    size: Option<CanvasSize>,
    ports: Vec<PortId>,
) -> Node {
    Node {
        kind,
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

fn test_data_port(node: NodeId, key: &str, dir: PortDirection, capacity: PortCapacity) -> Port {
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

pub(super) fn make_test_graph_two_nodes() -> (Graph, NodeId, NodeId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();

    graph.nodes.insert(
        a,
        test_node(
            kind.clone(),
            CanvasPoint { x: 0.0, y: 0.0 },
            None,
            Vec::new(),
        ),
    );
    graph.nodes.insert(
        b,
        test_node(kind, CanvasPoint { x: 10.0, y: 0.0 }, None, Vec::new()),
    );

    (graph, a, b)
}

pub(super) fn make_test_graph_two_nodes_with_size() -> (Graph, NodeId, NodeId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();

    graph.nodes.insert(
        a,
        test_node(
            kind.clone(),
            CanvasPoint { x: 0.0, y: 0.0 },
            Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            Vec::new(),
        ),
    );
    graph.nodes.insert(
        b,
        test_node(
            kind,
            CanvasPoint { x: 10.0, y: 5.0 },
            Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            Vec::new(),
        ),
    );

    (graph, a, b)
}

pub(super) fn make_test_graph_two_nodes_with_ports()
-> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let a_in = PortId::new();
    let a_out = PortId::new();
    graph.nodes.insert(
        a,
        test_node(
            kind.clone(),
            CanvasPoint { x: 0.0, y: 0.0 },
            None,
            vec![a_in, a_out],
        ),
    );
    graph.ports.insert(
        a_in,
        test_data_port(a, "in", PortDirection::In, PortCapacity::Single),
    );
    graph.ports.insert(
        a_out,
        test_data_port(a, "out", PortDirection::Out, PortCapacity::Multi),
    );

    let b = NodeId::new();
    let b_in = PortId::new();
    graph.nodes.insert(
        b,
        test_node(kind, CanvasPoint { x: 200.0, y: 0.0 }, None, vec![b_in]),
    );
    graph.ports.insert(
        b_in,
        test_data_port(b, "in", PortDirection::In, PortCapacity::Single),
    );

    (graph, a, a_in, a_out, b, b_in)
}

pub(super) fn make_test_graph_two_nodes_with_ports_spaced_x(
    dx: f32,
) -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
    let (mut graph, a, a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
    graph
        .nodes
        .entry(b)
        .and_modify(|n| n.pos = CanvasPoint { x: dx, y: 0.0 });
    (graph, a, a_in, a_out, b, b_in)
}

pub(super) fn read_node_pos(
    host: &mut TestUiHostImpl,
    model: &fret_runtime::Model<Graph>,
    id: NodeId,
) -> CanvasPoint {
    model
        .read_ref(host, |g| g.nodes.get(&id).map(|n| n.pos))
        .ok()
        .flatten()
        .unwrap_or_default()
}
