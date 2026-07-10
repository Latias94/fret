use std::sync::{Arc, Mutex};

use super::super::{TextFieldBlurBehavior, TextFieldOptions};
use super::{
    BufferedTextFieldFocusPlan, BufferedTextFieldPendingBlurPlan, BufferedTextFieldState,
    TextFieldDraftController, TextFieldDraftSnapshot, plan_buffered_text_field_focus_transition,
    sync_buffered_text_field_session, sync_draft_from_model_when_session_inactive,
};
use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_runtime::ModelStore;
use fret_ui::action::{ActionCx, UiActionHost, UiFocusActionHost};
use fret_ui::{GlobalElementId, Invalidation, elements::with_element_cx};

#[derive(Default)]
struct FakeHost {
    models: ModelStore,
    redraws: Vec<AppWindowId>,
}

impl fret_runtime::ModelHost for FakeHost {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}

impl UiActionHost for FakeHost {
    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }

    fn push_effect(&mut self, _effect: fret_runtime::Effect) {}

    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraws.push(window);
    }

    fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
        fret_runtime::TimerToken(0)
    }

    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
        fret_runtime::ClipboardToken::default()
    }

    fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
        fret_runtime::ShareSheetToken::default()
    }
}

impl UiFocusActionHost for FakeHost {
    fn request_focus(&mut self, _target: GlobalElementId) {}
}

fn action_cx() -> ActionCx {
    ActionCx {
        window: AppWindowId::default(),
        target: GlobalElementId(1),
    }
}

fn bind_test_controller(
    host: &mut FakeHost,
    controller: &TextFieldDraftController,
) -> (
    fret_runtime::Model<String>,
    fret_runtime::Model<String>,
    Arc<Mutex<BufferedTextFieldState>>,
) {
    bind_test_controller_values(host, controller, "before", "after")
}

fn bind_test_controller_values(
    host: &mut FakeHost,
    controller: &TextFieldDraftController,
    committed: &str,
    draft: &str,
) -> (
    fret_runtime::Model<String>,
    fret_runtime::Model<String>,
    Arc<Mutex<BufferedTextFieldState>>,
) {
    let model = host.models.insert(committed.to_string());
    let draft = host.models.insert(draft.to_string());
    let buffered_state = Arc::new(Mutex::new(BufferedTextFieldState::default()));
    buffered_state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .session
        .begin(committed.to_string());

    controller.bind(model.clone(), draft.clone(), buffered_state.clone(), None);

    (model, draft, buffered_state)
}

fn bind_equal_test_controller(
    host: &mut FakeHost,
    controller: &TextFieldDraftController,
    text: &str,
) -> (
    fret_runtime::Model<String>,
    fret_runtime::Model<String>,
    Arc<Mutex<BufferedTextFieldState>>,
) {
    bind_test_controller_values(host, controller, text, text)
}

#[test]
fn focus_begin_starts_session_and_clears_pending_blur() {
    assert_eq!(
        plan_buffered_text_field_focus_transition(
            false,
            false,
            true,
            TextFieldBlurBehavior::Commit,
            true,
        ),
        BufferedTextFieldFocusPlan {
            begin_session: true,
            cancel_pending_blur: true,
            pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
        }
    );
}

#[test]
fn refocus_cancels_pending_blur_without_restarting_active_session() {
    assert_eq!(
        plan_buffered_text_field_focus_transition(
            false,
            true,
            true,
            TextFieldBlurBehavior::Commit,
            true,
        ),
        BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: true,
            pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
        }
    );
}

#[test]
fn blur_commit_arms_pending_commit() {
    assert_eq!(
        plan_buffered_text_field_focus_transition(
            true,
            true,
            false,
            TextFieldBlurBehavior::Commit,
            false,
        ),
        BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: false,
            pending_blur: BufferedTextFieldPendingBlurPlan::Arm(TextFieldBlurBehavior::Commit),
        }
    );
}

#[test]
fn blur_cancel_arms_pending_cancel() {
    assert_eq!(
        plan_buffered_text_field_focus_transition(
            true,
            true,
            false,
            TextFieldBlurBehavior::Cancel,
            false,
        ),
        BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: false,
            pending_blur: BufferedTextFieldPendingBlurPlan::Arm(TextFieldBlurBehavior::Cancel),
        }
    );
}

#[test]
fn blur_preserve_draft_clears_pending_blur_without_arming_timer() {
    assert_eq!(
        plan_buffered_text_field_focus_transition(
            true,
            true,
            false,
            TextFieldBlurBehavior::PreserveDraft,
            true,
        ),
        BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: true,
            pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
        }
    );
}

#[test]
fn active_unfocused_session_keeps_existing_pending_blur_state() {
    assert_eq!(
        plan_buffered_text_field_focus_transition(
            false,
            true,
            false,
            TextFieldBlurBehavior::Commit,
            true,
        ),
        BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: false,
            pending_blur: BufferedTextFieldPendingBlurPlan::Keep,
        }
    );
}

#[test]
fn inactive_unfocused_state_clears_stale_pending_blur() {
    assert_eq!(
        plan_buffered_text_field_focus_transition(
            false,
            false,
            false,
            TextFieldBlurBehavior::Commit,
            true,
        ),
        BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: true,
            pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
        }
    );
}

#[test]
fn text_field_defaults_to_stable_line_boxes() {
    assert!(TextFieldOptions::default().stable_line_boxes);
}

#[test]
fn draft_controller_commit_uses_bound_buffered_session() {
    let mut host = FakeHost::default();
    let controller = TextFieldDraftController::new();
    let (model, _draft, buffered_state) = bind_test_controller(&mut host, &controller);

    assert!(controller.is_bound());
    assert!(controller.commit(&mut host, action_cx()));

    assert_eq!(host.models.get_cloned(&model).as_deref(), Some("after"));
    assert!(
        !buffered_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .session
            .is_active()
    );
    assert_eq!(host.redraws, vec![AppWindowId::default()]);
}

#[test]
fn draft_controller_discard_reverts_bound_buffered_session() {
    let mut host = FakeHost::default();
    let controller = TextFieldDraftController::new();
    let (model, draft, buffered_state) = bind_test_controller(&mut host, &controller);

    assert!(controller.discard(&mut host, action_cx()));

    assert_eq!(host.models.get_cloned(&model).as_deref(), Some("before"));
    assert_eq!(host.models.get_cloned(&draft).as_deref(), Some("before"));
    assert!(
        !buffered_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .session
            .is_active()
    );
    assert_eq!(host.redraws, vec![AppWindowId::default()]);
}

#[test]
fn draft_controller_unbound_actions_are_noops() {
    let mut host = FakeHost::default();
    let controller = TextFieldDraftController::new();

    assert!(!controller.is_bound());
    assert_eq!(controller.snapshot(&host), TextFieldDraftSnapshot::Unbound);
    assert!(!controller.commit(&mut host, action_cx()));
    assert!(!controller.commit_if_dirty(&mut host, action_cx()));
    assert!(!controller.discard(&mut host, action_cx()));
    assert!(!controller.discard_if_dirty(&mut host, action_cx()));
    assert!(host.redraws.is_empty());
}

#[test]
fn draft_controller_snapshot_distinguishes_clean_and_dirty_sessions() {
    let mut host = FakeHost::default();
    let controller = TextFieldDraftController::new();
    let (_model, draft, _buffered_state) =
        bind_equal_test_controller(&mut host, &controller, "before");

    assert_eq!(
        controller.snapshot(&host),
        TextFieldDraftSnapshot::Clean {
            committed: "before".to_string(),
        }
    );
    assert_eq!(controller.snapshot(&host).committed(), Some("before"));
    assert!(!controller.commit_if_dirty(&mut host, action_cx()));
    assert!(!controller.discard_if_dirty(&mut host, action_cx()));
    assert!(host.redraws.is_empty());

    host.models
        .update(&draft, |draft| *draft = "after".to_string())
        .expect("update draft");
    assert_eq!(
        controller.snapshot(&host),
        TextFieldDraftSnapshot::Dirty {
            committed: "before".to_string(),
            draft: "after".to_string(),
        }
    );
    assert_eq!(controller.snapshot(&host).committed(), Some("before"));

    assert!(controller.commit_if_dirty(&mut host, action_cx()));
    assert_eq!(
        controller.snapshot(&host),
        TextFieldDraftSnapshot::Clean {
            committed: "after".to_string(),
        }
    );
}

#[test]
fn draft_controller_snapshot_in_reads_through_element_context() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(48.0)));
    let model = app.models_mut().insert(String::from("before"));
    let draft = app.models_mut().insert(String::from("after"));
    let buffered_state = Arc::new(Mutex::new(BufferedTextFieldState::default()));
    buffered_state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .session
        .begin(String::from("before"));
    let controller = TextFieldDraftController::new();
    controller.bind(model, draft, buffered_state, None);

    with_element_cx(
        &mut app,
        window,
        bounds,
        "draft-controller-snapshot-test",
        |cx| {
            assert_eq!(
                controller.snapshot_in(cx, Invalidation::Paint),
                TextFieldDraftSnapshot::Dirty {
                    committed: "before".to_string(),
                    draft: "after".to_string(),
                }
            );
        },
    );
}

#[test]
fn draft_controller_snapshot_is_clean_after_discard() {
    let mut host = FakeHost::default();
    let controller = TextFieldDraftController::new();
    let (_model, _draft, _buffered_state) = bind_test_controller(&mut host, &controller);

    assert!(matches!(
        controller.snapshot(&host),
        TextFieldDraftSnapshot::Dirty { .. }
    ));
    assert!(controller.discard_if_dirty(&mut host, action_cx()));
    assert_eq!(
        controller.snapshot(&host),
        TextFieldDraftSnapshot::Clean {
            committed: "before".to_string(),
        }
    );
}

#[test]
fn draft_controller_commit_skips_noop_model_write_when_text_matches() {
    let mut host = FakeHost::default();
    let controller = TextFieldDraftController::new();
    let (model, draft, _buffered_state) =
        bind_equal_test_controller(&mut host, &controller, "same");
    let model_revision = host.models.revision(&model);
    let draft_revision = host.models.revision(&draft);
    let _ = host.models.take_changed_models();

    assert!(controller.commit(&mut host, action_cx()));

    assert_eq!(model_revision, host.models.revision(&model));
    assert_eq!(draft_revision, host.models.revision(&draft));
    assert!(host.models.take_changed_models().is_empty());
    assert_eq!(host.redraws, vec![AppWindowId::default()]);
}

#[test]
fn draft_controller_discard_skips_noop_model_write_when_text_matches() {
    let mut host = FakeHost::default();
    let controller = TextFieldDraftController::new();
    let (model, draft, _buffered_state) =
        bind_equal_test_controller(&mut host, &controller, "same");
    let model_revision = host.models.revision(&model);
    let draft_revision = host.models.revision(&draft);
    let _ = host.models.take_changed_models();

    assert!(controller.discard(&mut host, action_cx()));

    assert_eq!(model_revision, host.models.revision(&model));
    assert_eq!(draft_revision, host.models.revision(&draft));
    assert!(host.models.take_changed_models().is_empty());
    assert_eq!(host.redraws, vec![AppWindowId::default()]);
}

#[test]
fn inactive_draft_sync_skips_write_when_text_is_unchanged() {
    let mut host = App::new();
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(48.0)));
    let draft = host.models_mut().insert(String::from("same"));
    let buffered_state = Arc::new(Mutex::new(BufferedTextFieldState::default()));
    let _ = host.take_changed_models();

    with_element_cx(
        &mut host,
        window,
        bounds,
        "buffered-draft-sync-test",
        |cx| {
            sync_draft_from_model_when_session_inactive(cx, &draft, &buffered_state, "same");
        },
    );

    assert!(host.take_changed_models().is_empty());
    assert_eq!(host.models().get_cloned(&draft).as_deref(), Some("same"));
}

#[test]
fn focus_begin_skips_write_when_text_matches() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(48.0)));
    let draft = app.models_mut().insert(String::from("same"));
    let buffered_state = Arc::new(Mutex::new(BufferedTextFieldState::default()));
    let _ = app.take_changed_models();

    with_element_cx(
        &mut app,
        window,
        bounds,
        "buffered-focus-begin-test",
        |cx| {
            sync_buffered_text_field_session(
                cx,
                GlobalElementId(1),
                true,
                "same",
                &draft,
                &buffered_state,
                TextFieldBlurBehavior::Commit,
            );
        },
    );

    assert!(app.take_changed_models().is_empty());
    assert_eq!(app.models().get_cloned(&draft).as_deref(), Some("same"));
    assert!(
        buffered_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .session
            .is_active()
    );
}
