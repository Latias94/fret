use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::CommandId;
use fret_ui::action::UiActionHost;

use crate::commands::{
    tab_move_active_after_command, tab_move_active_before_command, tab_pin_command,
    tab_unpin_command,
};
use crate::tab_drag::WorkspaceTabInsertionSide;

#[derive(Debug, Clone)]
pub(super) enum WorkspaceTabStripIntent {
    Activate(CommandId),
    Close(CommandId),
    ReorderActive {
        target_tab_id: Arc<str>,
        side: WorkspaceTabInsertionSide,
    },
    SetPinned {
        tab_id: Arc<str>,
        pinned: bool,
    },
    RequestRedraw,
}

pub(super) fn dispatch_intent<H: UiActionHost + ?Sized>(
    host: &mut H,
    window: AppWindowId,
    intent: WorkspaceTabStripIntent,
) {
    match intent {
        WorkspaceTabStripIntent::Activate(cmd) | WorkspaceTabStripIntent::Close(cmd) => {
            host.dispatch_command(Some(window), cmd);
        }
        WorkspaceTabStripIntent::ReorderActive {
            target_tab_id,
            side,
        } => {
            let cmd = match side {
                WorkspaceTabInsertionSide::Before => {
                    tab_move_active_before_command(target_tab_id.as_ref())
                }
                WorkspaceTabInsertionSide::After => {
                    tab_move_active_after_command(target_tab_id.as_ref())
                }
            };
            if let Some(cmd) = cmd {
                host.dispatch_command(Some(window), cmd);
            }
        }
        WorkspaceTabStripIntent::SetPinned { tab_id, pinned } => {
            let cmd = if pinned {
                tab_pin_command(tab_id.as_ref())
            } else {
                tab_unpin_command(tab_id.as_ref())
            };
            if let Some(cmd) = cmd {
                host.dispatch_command(Some(window), cmd);
            }
        }
        WorkspaceTabStripIntent::RequestRedraw => host.request_redraw(window),
    }
}
