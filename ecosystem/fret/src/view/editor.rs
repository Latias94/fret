use std::sync::Arc;

use fret_ui::Invalidation;
use fret_ui::action::{ActionCx, OnActivate, UiActionHost};
use fret_ui_editor::controls::{
    EditorThemePresetPicker, TextField, TextFieldDraftController, TextFieldDraftSnapshot,
    TextFieldOptions, TextFieldOutcome,
};
use fret_ui_editor::theme::EditorThemePreset;

use super::{
    AppLocalStateExt as _, LocalState, LocalStateRawModelExt as _, RenderContextAccess,
    UiActionHostLocalStateTxnExt as _, activation::action_listener,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTextFieldOutcome {
    #[default]
    Idle,
    Committed,
    Canceled,
}

impl InspectorTextFieldOutcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Committed => "Committed",
            Self::Canceled => "Canceled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorTextFieldSnapshot {
    pub draft: TextFieldDraftSnapshot,
    pub outcome: InspectorTextFieldOutcome,
    pub status: String,
}

impl InspectorTextFieldSnapshot {
    pub fn committed(&self) -> &str {
        self.draft
            .committed()
            .expect("inspector snapshots normalize unbound drafts to a clean committed value")
    }

    pub fn is_dirty(&self) -> bool {
        self.draft.is_dirty()
    }

    pub fn draft_status_label(&self, committed_label: &str) -> String {
        match (&self.draft, self.outcome) {
            (TextFieldDraftSnapshot::Dirty { .. }, _) => {
                format!("Unsaved draft · {committed_label}")
            }
            (TextFieldDraftSnapshot::Unbound, _) => {
                format!("Draft unavailable · {committed_label}")
            }
            (_, InspectorTextFieldOutcome::Canceled) => {
                format!("Draft canceled · {committed_label}")
            }
            _ => format!("Clean draft · {committed_label}"),
        }
    }
}

/// App-facing binding for a buffered inspector text field plus its outcome/status readouts.
///
/// The committed value, last outcome, and status remain explicit `LocalState` handles. The
/// `TextFieldDraftController` stays inside the binding so app code can offer commit/discard
/// commands without naming raw models or `ModelStore`.
#[derive(Clone)]
pub struct InspectorTextFieldBinding {
    value: LocalState<String>,
    outcome: LocalState<InspectorTextFieldOutcome>,
    status: LocalState<String>,
    draft_controller: TextFieldDraftController,
    committed_status: Arc<str>,
    canceled_status: Arc<str>,
}

impl InspectorTextFieldBinding {
    pub fn new(
        app: &mut crate::app::App,
        value: impl Into<String>,
        initial_status: impl Into<String>,
    ) -> Self {
        Self {
            value: app.local_state(value.into()),
            outcome: app.local_state(InspectorTextFieldOutcome::Idle),
            status: app.local_state(initial_status.into()),
            draft_controller: TextFieldDraftController::new(),
            committed_status: Arc::from("Draft committed."),
            canceled_status: Arc::from("Draft discarded."),
        }
    }

    pub fn outcome_statuses(
        mut self,
        committed: impl Into<Arc<str>>,
        canceled: impl Into<Arc<str>>,
    ) -> Self {
        self.committed_status = committed.into();
        self.canceled_status = canceled.into();
        self
    }

    pub fn value_state(&self) -> &LocalState<String> {
        &self.value
    }

    pub fn outcome_state(&self) -> &LocalState<InspectorTextFieldOutcome> {
        &self.outcome
    }

    pub fn status_state(&self) -> &LocalState<String> {
        &self.status
    }

    pub fn text_field(&self, mut options: TextFieldOptions) -> TextField {
        options.draft_controller = Some(self.draft_controller.clone());
        self.value
            .editor_text_field()
            .on_outcome(Some(self.outcome_handler()))
            .options(options)
    }

    pub fn paint_snapshot<'a, Cx>(&self, cx: &mut Cx) -> InspectorTextFieldSnapshot
    where
        Cx: RenderContextAccess<'a, crate::app::App>,
    {
        let outcome = self.outcome.paint_value(cx);
        let status = self.status.paint_value(cx);
        let draft = match self.draft_controller.snapshot_in(cx, Invalidation::Paint) {
            TextFieldDraftSnapshot::Unbound => TextFieldDraftSnapshot::Clean {
                committed: self.value.paint_value(cx),
            },
            draft => draft,
        };
        InspectorTextFieldSnapshot {
            draft,
            outcome,
            status,
        }
    }

    /// Commit the bound draft while preserving the activating button as the focus owner.
    pub fn commit_activate(&self) -> OnActivate {
        let binding = self.clone();
        action_listener(move |host, action_cx| {
            if binding.draft_controller.commit_if_dirty(host, action_cx) {
                binding.apply_outcome(host, action_cx, InspectorTextFieldOutcome::Committed);
            }
        })
    }

    /// Discard the bound draft while preserving the activating button as the focus owner.
    pub fn discard_activate(&self) -> OnActivate {
        let binding = self.clone();
        action_listener(move |host, action_cx| {
            if binding.draft_controller.discard_if_dirty(host, action_cx) {
                binding.apply_outcome(host, action_cx, InspectorTextFieldOutcome::Canceled);
            }
        })
    }

    pub fn status_activate(&self, status: impl Into<String>) -> OnActivate {
        let binding = self.clone();
        let status = status.into();
        action_listener(move |host, action_cx| {
            let changed = host.local_state_txn(|tx| tx.set(&binding.status, status.clone()));
            if changed {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
        })
    }

    fn outcome_handler(&self) -> fret_ui_editor::controls::OnTextFieldOutcome {
        let binding = self.clone();
        Arc::new(move |host, action_cx, outcome| {
            let outcome = match outcome {
                TextFieldOutcome::Committed => InspectorTextFieldOutcome::Committed,
                TextFieldOutcome::Canceled => InspectorTextFieldOutcome::Canceled,
            };
            binding.apply_outcome(host, action_cx, outcome);
        })
    }

    fn apply_outcome(
        &self,
        host: &mut dyn UiActionHost,
        action_cx: ActionCx,
        outcome: InspectorTextFieldOutcome,
    ) {
        let status = match outcome {
            InspectorTextFieldOutcome::Idle => return,
            InspectorTextFieldOutcome::Committed => self.committed_status.as_ref(),
            InspectorTextFieldOutcome::Canceled => self.canceled_status.as_ref(),
        };
        let changed = host.local_state_txn(|tx| {
            let outcome_changed = tx.set(&self.outcome, outcome);
            let status_changed = tx.set(&self.status, status.to_string());
            outcome_changed || status_changed
        });
        if changed {
            host.notify(action_cx);
        }
    }
}

pub trait TextFieldLocalStateExt {
    fn editor_text_field(&self) -> TextField;
}

impl TextFieldLocalStateExt for LocalState<String> {
    fn editor_text_field(&self) -> TextField {
        TextField::new(self.clone_model())
    }
}

pub trait EditorThemePresetPickerLocalStateExt {
    fn editor_theme_preset_picker(&self) -> EditorThemePresetPicker;
}

impl EditorThemePresetPickerLocalStateExt for LocalState<EditorThemePreset> {
    fn editor_theme_preset_picker(&self) -> EditorThemePresetPicker {
        EditorThemePresetPicker::new(self.clone_model())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::AppLocalStateTxnExt as _;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::GlobalElementId;
    use fret_ui::action::{ActionCx, ActivateReason, UiActionHostAdapter};
    use fret_ui::elements::with_element_cx;

    fn action_cx() -> ActionCx {
        ActionCx {
            window: AppWindowId::default(),
            target: GlobalElementId(7),
        }
    }

    #[test]
    fn inspector_binding_updates_outcome_and_status_without_model_store_at_callsite() {
        let mut app = crate::app::App::new();
        let binding = InspectorTextFieldBinding::new(&mut app, "before", "Ready")
            .outcome_statuses("Committed status", "Canceled status");
        let cx = action_cx();

        {
            let mut host = UiActionHostAdapter { app: &mut app };
            binding.apply_outcome(&mut host, cx, InspectorTextFieldOutcome::Committed);
        }
        app.local_state_txn(|tx| {
            assert_eq!(
                tx.value(binding.outcome_state()),
                InspectorTextFieldOutcome::Committed
            );
            assert_eq!(tx.value(binding.status_state()), "Committed status");
        });

        {
            let mut host = UiActionHostAdapter { app: &mut app };
            binding.apply_outcome(&mut host, cx, InspectorTextFieldOutcome::Canceled);
        }
        app.local_state_txn(|tx| {
            assert_eq!(
                tx.value(binding.outcome_state()),
                InspectorTextFieldOutcome::Canceled
            );
            assert_eq!(tx.value(binding.status_state()), "Canceled status");
        });
    }

    #[test]
    fn inspector_binding_first_snapshot_is_clean_before_text_field_binds() {
        let mut app = crate::app::App::new();
        let binding = InspectorTextFieldBinding::new(&mut app, "before", "Ready");
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(48.0)));

        with_element_cx(
            &mut app,
            AppWindowId::default(),
            bounds,
            "inspector-binding-first-snapshot-test",
            |cx| {
                let snapshot = binding.paint_snapshot(cx);
                assert_eq!(
                    snapshot.draft,
                    TextFieldDraftSnapshot::Clean {
                        committed: "before".to_string(),
                    }
                );
                assert_eq!(snapshot.committed(), "before");
            },
        );
    }

    #[test]
    fn inspector_binding_status_action_keeps_summary_feedback_local() {
        let mut app = crate::app::App::new();
        let binding = InspectorTextFieldBinding::new(&mut app, "before", "Ready");
        let activate = binding.status_activate("Copied summary");
        let cx = action_cx();
        let mut host = UiActionHostAdapter { app: &mut app };

        activate(&mut host, cx, ActivateReason::Keyboard);
        app.local_state_txn(|tx| {
            assert_eq!(tx.value(binding.status_state()), "Copied summary");
        });
    }

    #[test]
    fn inspector_snapshot_labels_dirty_and_canceled_states() {
        let dirty = InspectorTextFieldSnapshot {
            draft: TextFieldDraftSnapshot::Dirty {
                committed: "before".to_string(),
                draft: "after".to_string(),
            },
            outcome: InspectorTextFieldOutcome::Idle,
            status: "Ready".to_string(),
        };
        assert_eq!(dirty.committed(), "before");
        assert!(dirty.is_dirty());
        assert_eq!(
            dirty.draft_status_label("1 line committed"),
            "Unsaved draft · 1 line committed"
        );

        let canceled = InspectorTextFieldSnapshot {
            draft: TextFieldDraftSnapshot::Clean {
                committed: "before".to_string(),
            },
            outcome: InspectorTextFieldOutcome::Canceled,
            ..dirty
        };
        assert!(!canceled.is_dirty());
        assert_eq!(
            canceled.draft_status_label("1 line committed"),
            "Draft canceled · 1 line committed"
        );
    }
}
