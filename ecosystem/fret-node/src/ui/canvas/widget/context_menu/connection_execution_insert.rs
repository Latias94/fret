mod activate;
mod apply;
mod plan;
mod recovery;
mod retained_cx;

use super::connection_execution::ConnectionInsertMenuPlan;
use super::*;

pub(in crate::ui::canvas::widget) trait ConnectionInsertMenuCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn resume_connection_insert_wire_drag<M: NodeGraphCanvasMiddleware>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        fallback_from: PortId,
        invoked_at: Point,
        continue_from: Option<PortId>,
    );
    fn restore_connection_menu_wire_drag<M: NodeGraphCanvasMiddleware>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        fallback_from: PortId,
        invoked_at: Point,
    );
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn activate_connection_insert_picker_action<H: UiHost>(
        &mut self,
        cx: &mut impl ConnectionInsertMenuCx<H>,
        from: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        activate::activate_connection_insert_picker_action(
            self,
            cx,
            from,
            at,
            invoked_at,
            action,
            menu_candidates,
        )
    }

    #[cfg(test)]
    pub(super) fn plan_connection_insert_menu_candidate_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        from: PortId,
        at: CanvasPoint,
        mode: NodeGraphConnectionMode,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionInsertMenuPlan {
        plan::plan_connection_insert_menu_candidate_with_graph::<M>(
            presenter, graph, from, at, mode, candidate,
        )
    }

    pub(super) fn plan_connection_insert_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        at: CanvasPoint,
        mode: NodeGraphConnectionMode,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionInsertMenuPlan {
        plan::plan_connection_insert_menu_candidate(self, host, from, at, mode, candidate)
    }

    pub(super) fn apply_connection_insert_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut impl ConnectionInsertMenuCx<H>,
        fallback_from: PortId,
        invoked_at: Point,
        plan: ConnectionInsertMenuPlan,
    ) {
        apply::apply_connection_insert_menu_plan(self, cx, fallback_from, invoked_at, plan)
    }

    fn resume_connection_insert_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut impl ConnectionInsertMenuCx<H>,
        fallback_from: PortId,
        invoked_at: Point,
        continue_from: Option<PortId>,
    ) {
        recovery::resume_connection_insert_wire_drag(
            self,
            cx,
            fallback_from,
            invoked_at,
            continue_from,
        )
    }

    pub(super) fn restore_connection_menu_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut impl ConnectionInsertMenuCx<H>,
        fallback_from: PortId,
        invoked_at: Point,
    ) {
        recovery::restore_connection_menu_wire_drag(self, cx, fallback_from, invoked_at)
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::{ConnectionInsertMenuCx, ConnectionInsertMenuPlan};
    use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey, PortId};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ui::canvas::widget::NodeGraphCanvasWith;
    use crate::ui::canvas::widget::NoopNodeGraphCanvasMiddleware;
    use crate::ui::canvas::workflow;
    use crate::ui::presenter::{InsertNodeCandidate, NodeGraphContextMenuAction};
    use fret_core::{AppWindowId, Point, PointerId, Px};
    use fret_runtime::ui_host::{
        CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, DragKindId, DragSession, Effect, FrameId,
        ImageUploadToken, ModelHost, ModelId, ModelStore, ShareSheetToken, TickId, TimerToken,
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
    enum RecoveryCall {
        Resume {
            fallback_from: PortId,
            invoked_at: Point,
            continue_from: Option<PortId>,
        },
        Restore {
            fallback_from: PortId,
            invoked_at: Point,
        },
    }

    struct StubCx {
        host: TestHost,
        window: Option<AppWindowId>,
        recovery_calls: Vec<RecoveryCall>,
    }

    impl StubCx {
        fn new() -> Self {
            Self {
                host: TestHost::default(),
                window: Some(AppWindowId::default()),
                recovery_calls: Vec::new(),
            }
        }
    }

    impl ConnectionInsertMenuCx<TestHost> for StubCx {
        fn host(&mut self) -> &mut TestHost {
            &mut self.host
        }

        fn window(&self) -> Option<AppWindowId> {
            self.window
        }

        fn resume_connection_insert_wire_drag<
            M: crate::ui::canvas::widget::NodeGraphCanvasMiddleware,
        >(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<M>,
            fallback_from: PortId,
            invoked_at: Point,
            continue_from: Option<PortId>,
        ) {
            self.recovery_calls.push(RecoveryCall::Resume {
                fallback_from,
                invoked_at,
                continue_from,
            });
        }

        fn restore_connection_menu_wire_drag<
            M: crate::ui::canvas::widget::NodeGraphCanvasMiddleware,
        >(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<M>,
            fallback_from: PortId,
            invoked_at: Point,
        ) {
            self.recovery_calls.push(RecoveryCall::Restore {
                fallback_from,
                invoked_at,
            });
        }
    }

    fn test_canvas(cx: &mut StubCx) -> NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
        let graph = cx.host.models.insert(Graph::new(GraphId::new()));
        let view = cx.host.models.insert(NodeGraphViewState::default());
        let editor_config = cx.host.models.insert(NodeGraphEditorConfig::default());
        NodeGraphCanvasWith::new_with_middleware(
            graph,
            view,
            editor_config,
            NoopNodeGraphCanvasMiddleware,
        )
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

    #[test]
    fn connection_insert_action_with_missing_candidate_is_handled_without_side_effects() {
        let mut cx = StubCx::new();
        let mut canvas = test_canvas(&mut cx);

        let handled = canvas.activate_connection_insert_picker_action(
            &mut cx,
            PortId::new(),
            CanvasPoint { x: 10.0, y: 20.0 },
            Point::new(Px(1.0), Px(2.0)),
            NodeGraphContextMenuAction::InsertNodeCandidate(1),
            &[regular_candidate()],
        );

        assert!(handled);
        assert!(cx.recovery_calls.is_empty());
        assert!(canvas.interaction.toast.is_none());
        assert!(canvas.interaction.recent_kinds.is_empty());
    }

    #[test]
    fn connection_insert_action_ignores_non_candidate_actions() {
        let mut cx = StubCx::new();
        let mut canvas = test_canvas(&mut cx);

        let handled = canvas.activate_connection_insert_picker_action(
            &mut cx,
            PortId::new(),
            CanvasPoint { x: 10.0, y: 20.0 },
            Point::new(Px(1.0), Px(2.0)),
            NodeGraphContextMenuAction::OpenInsertNodePicker,
            &[regular_candidate()],
        );

        assert!(!handled);
        assert!(cx.recovery_calls.is_empty());
        assert!(canvas.interaction.toast.is_none());
        assert!(canvas.interaction.recent_kinds.is_empty());
    }

    #[test]
    fn connection_insert_action_records_candidate_and_restores_on_rejection() {
        let mut cx = StubCx::new();
        let mut canvas = test_canvas(&mut cx);
        let from = PortId::new();
        let invoked_at = Point::new(Px(1.0), Px(2.0));

        let handled = canvas.activate_connection_insert_picker_action(
            &mut cx,
            from,
            CanvasPoint { x: 10.0, y: 20.0 },
            invoked_at,
            NodeGraphContextMenuAction::InsertNodeCandidate(0),
            &[regular_candidate()],
        );

        assert!(handled);
        assert_eq!(
            canvas.interaction.recent_kinds,
            vec![NodeKindKey::new("regular")]
        );
        assert!(matches!(
            canvas.interaction.toast,
            Some(ref toast) if &*toast.message == "node insertion is not supported"
        ));
        assert_eq!(
            cx.recovery_calls,
            vec![RecoveryCall::Restore {
                fallback_from: from,
                invoked_at,
            }]
        );
    }

    #[test]
    fn connection_insert_apply_success_resumes_wire_drag() {
        let mut cx = StubCx::new();
        let mut canvas = test_canvas(&mut cx);
        let fallback_from = PortId::new();
        let continue_from = PortId::new();
        let invoked_at = Point::new(Px(3.0), Px(4.0));

        canvas.apply_connection_insert_menu_plan(
            &mut cx,
            fallback_from,
            invoked_at,
            ConnectionInsertMenuPlan::Apply(workflow::WireDropInsertPlan {
                ops: Vec::new(),
                created_node: None,
                continue_from: Some(continue_from),
                toast: None,
            }),
        );

        assert_eq!(
            cx.recovery_calls,
            vec![RecoveryCall::Resume {
                fallback_from,
                invoked_at,
                continue_from: Some(continue_from),
            }]
        );
    }

    #[test]
    fn connection_insert_apply_ignore_restores_wire_drag() {
        let mut cx = StubCx::new();
        let mut canvas = test_canvas(&mut cx);
        let fallback_from = PortId::new();
        let invoked_at = Point::new(Px(3.0), Px(4.0));

        canvas.apply_connection_insert_menu_plan(
            &mut cx,
            fallback_from,
            invoked_at,
            ConnectionInsertMenuPlan::Ignore,
        );

        assert_eq!(
            cx.recovery_calls,
            vec![RecoveryCall::Restore {
                fallback_from,
                invoked_at,
            }]
        );
    }
}
