mod activate;
mod apply;
mod plan;
mod retained_cx;

use super::connection_execution::ConnectionConversionMenuPlan;
use super::*;

pub(in crate::ui::canvas::widget) trait ConnectionConversionMenuCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn restore_connection_conversion_wire_drag<M: NodeGraphCanvasMiddleware>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        fallback_from: PortId,
        invoked_at: Point,
    );
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[cfg(test)]
    pub(super) fn plan_connection_conversion_menu_candidate_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        style: &NodeGraphStyle,
        zoom: f32,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionConversionMenuPlan {
        plan::plan_connection_conversion_menu_candidate_with_graph::<M>(
            presenter, graph, style, zoom, from, to, at, candidate,
        )
    }

    pub(super) fn plan_connection_conversion_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionConversionMenuPlan {
        plan::plan_connection_conversion_menu_candidate(self, host, from, to, at, candidate)
    }

    pub(super) fn activate_connection_conversion_picker_action<H: UiHost>(
        &mut self,
        cx: &mut impl ConnectionConversionMenuCx<H>,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        activate::activate_connection_conversion_picker_action(
            self,
            cx,
            from,
            to,
            at,
            invoked_at,
            action,
            menu_candidates,
        )
    }

    pub(super) fn apply_connection_conversion_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut impl ConnectionConversionMenuCx<H>,
        fallback_from: PortId,
        invoked_at: Point,
        plan: ConnectionConversionMenuPlan,
    ) {
        apply::apply_connection_conversion_menu_plan(self, cx, fallback_from, invoked_at, plan)
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::{ConnectionConversionMenuCx, ConnectionConversionMenuPlan};
    use crate::core::{CanvasPoint, Graph, GraphId, Node, NodeId, NodeKindKey, PortId};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ops::GraphOp;
    use crate::ui::canvas::state::{WireDrag, WireDragKind};
    use crate::ui::canvas::widget::NodeGraphCanvasWith;
    use crate::ui::canvas::widget::NoopNodeGraphCanvasMiddleware;
    use crate::ui::presenter::{InsertNodeCandidate, NodeGraphContextMenuAction};
    use fret_core::{AppWindowId, Point, PointerId, Px};
    use fret_runtime::ui_host::{
        CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, DragKindId, DragSession, Effect, FrameId,
        ImageUploadToken, Model, ModelHost, ModelId, ModelStore, ShareSheetToken, TickId,
        TimerToken,
    };
    use serde_json::Value;

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

    #[derive(Debug, PartialEq)]
    struct RestoreCall {
        fallback_from: PortId,
        invoked_at: Point,
    }

    struct StubCx {
        host: TestHost,
        window: Option<AppWindowId>,
        restore_calls: Vec<RestoreCall>,
    }

    impl StubCx {
        fn new() -> Self {
            Self {
                host: TestHost::default(),
                window: Some(AppWindowId::default()),
                restore_calls: Vec::new(),
            }
        }
    }

    impl ConnectionConversionMenuCx<TestHost> for StubCx {
        fn host(&mut self) -> &mut TestHost {
            &mut self.host
        }

        fn window(&self) -> Option<AppWindowId> {
            self.window
        }

        fn restore_connection_conversion_wire_drag<
            M: crate::ui::canvas::widget::NodeGraphCanvasMiddleware,
        >(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<M>,
            fallback_from: PortId,
            invoked_at: Point,
        ) {
            self.restore_calls.push(RestoreCall {
                fallback_from,
                invoked_at,
            });
        }
    }

    struct TestCanvas {
        canvas: NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
        graph: Model<Graph>,
        view: Model<NodeGraphViewState>,
    }

    fn test_canvas(cx: &mut StubCx) -> TestCanvas {
        let graph = cx.host.models.insert(Graph::new(GraphId::new()));
        let view = cx.host.models.insert(NodeGraphViewState::default());
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
        }
    }

    fn regular_candidate() -> InsertNodeCandidate {
        InsertNodeCandidate {
            kind: NodeKindKey::new("regular"),
            label: Arc::<str>::from("Regular"),
            enabled: true,
            template: None,
            payload: Value::Null,
        }
    }

    fn test_node(kind: &str) -> Node {
        Node {
            kind: NodeKindKey::new(kind),
            kind_version: 0,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
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

    fn suspend_wire_drag(
        canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
        from: PortId,
        pos: Point,
    ) {
        canvas.interaction.suspended_wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from,
                bundle: Vec::new(),
            },
            pos,
        });
    }

    fn selected_nodes(cx: &StubCx, view: &Model<NodeGraphViewState>) -> Vec<NodeId> {
        view.read_ref(&cx.host, |state| state.selected_nodes.clone())
            .expect("test view state should be readable")
    }

    #[test]
    fn connection_conversion_action_with_missing_candidate_is_handled_without_side_effects() {
        let mut cx = StubCx::new();
        let mut test = test_canvas(&mut cx);

        let handled = test.canvas.activate_connection_conversion_picker_action(
            &mut cx,
            PortId::new(),
            PortId::new(),
            CanvasPoint { x: 10.0, y: 20.0 },
            Point::new(Px(1.0), Px(2.0)),
            NodeGraphContextMenuAction::InsertNodeCandidate(1),
            &[regular_candidate()],
        );

        assert!(handled);
        assert!(cx.restore_calls.is_empty());
        assert!(test.canvas.interaction.toast.is_none());
        assert!(test.canvas.interaction.recent_kinds.is_empty());
    }

    #[test]
    fn connection_conversion_action_ignores_non_candidate_actions() {
        let mut cx = StubCx::new();
        let mut test = test_canvas(&mut cx);

        let handled = test.canvas.activate_connection_conversion_picker_action(
            &mut cx,
            PortId::new(),
            PortId::new(),
            CanvasPoint { x: 10.0, y: 20.0 },
            Point::new(Px(1.0), Px(2.0)),
            NodeGraphContextMenuAction::OpenInsertNodePicker,
            &[regular_candidate()],
        );

        assert!(!handled);
        assert!(cx.restore_calls.is_empty());
        assert!(test.canvas.interaction.toast.is_none());
        assert!(test.canvas.interaction.recent_kinds.is_empty());
    }

    #[test]
    fn connection_conversion_action_records_candidate_and_restores_on_rejection() {
        let mut cx = StubCx::new();
        let mut test = test_canvas(&mut cx);
        let from = PortId::new();
        let to = PortId::new();
        let invoked_at = Point::new(Px(1.0), Px(2.0));

        let handled = test.canvas.activate_connection_conversion_picker_action(
            &mut cx,
            from,
            to,
            CanvasPoint { x: 10.0, y: 20.0 },
            invoked_at,
            NodeGraphContextMenuAction::InsertNodeCandidate(0),
            &[regular_candidate()],
        );

        assert!(handled);
        assert_eq!(
            test.canvas.interaction.recent_kinds,
            vec![NodeKindKey::new("regular")]
        );
        assert!(matches!(
            test.canvas.interaction.toast,
            Some(ref toast) if &*toast.message == "conversion candidate is missing template"
        ));
        assert_eq!(
            cx.restore_calls,
            vec![RestoreCall {
                fallback_from: from,
                invoked_at,
            }]
        );
    }

    #[test]
    fn connection_conversion_apply_success_clears_suspended_drag_and_selects_node() {
        let mut cx = StubCx::new();
        let mut test = test_canvas(&mut cx);
        let fallback_from = PortId::new();
        let invoked_at = Point::new(Px(3.0), Px(4.0));
        let inserted = NodeId::new();
        suspend_wire_drag(&mut test.canvas, fallback_from, invoked_at);

        test.canvas.apply_connection_conversion_menu_plan(
            &mut cx,
            fallback_from,
            invoked_at,
            ConnectionConversionMenuPlan::Apply(vec![GraphOp::AddNode {
                id: inserted,
                node: test_node("inserted"),
            }]),
        );

        assert!(cx.restore_calls.is_empty());
        assert!(test.canvas.interaction.suspended_wire_drag.is_none());
        assert_eq!(selected_nodes(&cx, &test.view), vec![inserted]);
        assert!(
            test.graph
                .read_ref(&cx.host, |graph| graph.nodes.contains_key(&inserted))
                .expect("test graph should be readable")
        );
    }

    #[test]
    fn connection_conversion_apply_ignore_restores_wire_drag() {
        let mut cx = StubCx::new();
        let mut test = test_canvas(&mut cx);
        let fallback_from = PortId::new();
        let invoked_at = Point::new(Px(3.0), Px(4.0));

        test.canvas.apply_connection_conversion_menu_plan(
            &mut cx,
            fallback_from,
            invoked_at,
            ConnectionConversionMenuPlan::Ignore,
        );

        assert_eq!(
            cx.restore_calls,
            vec![RestoreCall {
                fallback_from,
                invoked_at,
            }]
        );
    }
}
