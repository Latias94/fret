use super::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::core::{Graph, GraphId, NodeKindKey};
use crate::ui::DefaultNodeGraphPresenter;
use fret_core::{AppWindowId, Point, PointerId};
use fret_runtime::{
    ClipboardToken, CommandRegistry, DragKindId, DragSession, Effect, FrameId, ImageUploadToken,
    ModelHost, ModelId, ModelStore, ShareSheetToken, TickId, TimerToken,
};
use fret_runtime::{CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost};

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
}

impl StubCx {
    fn new() -> Self {
        Self {
            host: TestHost::default(),
            window: Some(AppWindowId::default()),
        }
    }
}

impl BackgroundInsertMenuCx<TestHost> for StubCx {
    fn host(&mut self) -> &mut TestHost {
        &mut self.host
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }
}

fn test_canvas(host: &mut TestHost) -> NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let editor_config = host.models.insert(NodeGraphEditorConfig::default());
    NodeGraphCanvasWith::new_with_middleware(
        graph,
        view,
        editor_config,
        NoopNodeGraphCanvasMiddleware,
    )
}

#[test]
fn background_insert_menu_plan_surfaces_create_node_errors() {
    let mut presenter = DefaultNodeGraphPresenter::default();
    let graph = Graph::new(GraphId::new());
    let candidate = super::test_support::regular_candidate();
    let plan =
        plan::plan_background_insert_menu_candidate_with_graph::<NoopNodeGraphCanvasMiddleware>(
            &mut presenter,
            &graph,
            &candidate,
            CanvasPoint { x: 10.0, y: 20.0 },
        );

    assert!(matches!(
        plan,
        BackgroundInsertMenuPlan::Reject(DiagnosticSeverity::Info, ref msg)
            if &**msg == "node insertion is not supported"
    ));
}

#[test]
fn background_insert_action_with_missing_candidate_is_handled_without_side_effects() {
    let mut cx = StubCx::new();
    let mut canvas = test_canvas(&mut cx.host);

    let handled = activate::activate_background_context_action(
        &mut canvas,
        &mut cx,
        CanvasPoint { x: 10.0, y: 20.0 },
        NodeGraphContextMenuAction::InsertNodeCandidate(1),
        &[super::test_support::regular_candidate()],
    );

    assert!(handled);
    assert!(canvas.interaction.toast.is_none());
    assert!(canvas.interaction.recent_kinds.is_empty());
}

#[test]
fn background_insert_action_ignores_non_candidate_actions() {
    let mut cx = StubCx::new();
    let mut canvas = test_canvas(&mut cx.host);

    let handled = activate::activate_background_context_action(
        &mut canvas,
        &mut cx,
        CanvasPoint { x: 10.0, y: 20.0 },
        NodeGraphContextMenuAction::OpenInsertNodePicker,
        &[super::test_support::regular_candidate()],
    );

    assert!(!handled);
    assert!(canvas.interaction.toast.is_none());
    assert!(canvas.interaction.recent_kinds.is_empty());
}

#[test]
fn background_insert_action_records_candidate_and_surfaces_rejection_toast() {
    let mut cx = StubCx::new();
    let mut canvas = test_canvas(&mut cx.host);

    let handled = activate::activate_background_context_action(
        &mut canvas,
        &mut cx,
        CanvasPoint { x: 10.0, y: 20.0 },
        NodeGraphContextMenuAction::InsertNodeCandidate(0),
        &[super::test_support::regular_candidate()],
    );

    assert!(handled);
    assert_eq!(
        canvas.interaction.recent_kinds,
        vec![NodeKindKey::new("regular")]
    );
    let toast = canvas
        .interaction
        .toast
        .as_ref()
        .expect("rejection should surface a toast");
    assert_eq!(toast.severity, DiagnosticSeverity::Info);
    assert_eq!(toast.message.as_ref(), "node insertion is not supported");
}
