use std::sync::Arc;

use fret_core::{AppWindowId, NodeId};
use fret_runtime::CommandId;
use fret_ui::{EventCx, UiHost};

use fret_components_ui::{DialogAction, DialogRequest};

/// AlertDialog request (shadcn-style).
///
/// This is a policy wrapper over the standard dialog overlay: it provides a canonical "cancel +
/// confirm action" shape matching shadcn's AlertDialog behavior.
#[derive(Debug, Clone)]
pub struct AlertDialogRequest {
    pub owner: NodeId,
    pub title: Arc<str>,
    pub description: Arc<str>,
    pub cancel_label: Arc<str>,
    pub action_label: Arc<str>,
    pub action_command: CommandId,
    pub default_action: AlertDialogDefaultAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertDialogDefaultAction {
    Action,
    Cancel,
}

impl Default for AlertDialogDefaultAction {
    fn default() -> Self {
        Self::Action
    }
}

impl AlertDialogRequest {
    pub fn new(
        owner: NodeId,
        title: impl Into<Arc<str>>,
        description: impl Into<Arc<str>>,
        action_label: impl Into<Arc<str>>,
        action_command: CommandId,
    ) -> Self {
        Self {
            owner,
            title: title.into(),
            description: description.into(),
            cancel_label: Arc::from("Cancel"),
            action_label: action_label.into(),
            action_command,
            default_action: AlertDialogDefaultAction::Action,
        }
    }

    pub fn cancel_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn default_action(mut self, default_action: AlertDialogDefaultAction) -> Self {
        self.default_action = default_action;
        self
    }

    pub fn into_dialog_request(self) -> DialogRequest {
        let cancel = DialogAction::cancel(self.cancel_label);
        let action = DialogAction::new(self.action_label, self.action_command);

        DialogRequest {
            owner: self.owner,
            title: self.title,
            message: self.description,
            actions: vec![cancel, action],
            default_action: Some(match self.default_action {
                AlertDialogDefaultAction::Cancel => 0,
                AlertDialogDefaultAction::Action => 1,
            }),
            // shadcn semantics: cancel closes without a side-effect command by default.
            cancel_command: None,
        }
    }
}

/// Opens a window-scoped alert dialog by setting a `DialogRequest` and dispatching `dialog.open`.
///
/// This uses the standard dialog overlay installed by `fret_components_ui::WindowOverlays`.
pub fn open_alert_dialog<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    request: AlertDialogRequest,
) {
    fret_components_ui::dialog::open_dialog(cx, window, request.into_dialog_request());
}
