use std::sync::Arc;

use fret_core::{Modifiers, MouseButton};
use fret_runtime::{CommandId, DefaultAction, Model};
use fret_ui::action::{
    ActionCx, ActivateReason, OnPressablePointerDown, PressablePointerDownResult,
    UiPointerActionHost,
};

use fret_ui_kit::dnd as ui_dnd;
use fret_ui_kit::headless::tab_strip_arbitration;

use crate::tab_drag::DRAG_KIND_WORKSPACE_TAB;

use super::consts::{TAB_CHROME_PAD_RIGHT, TAB_CLOSE_SIZE};
use super::drag_state::WorkspaceTabStripDragState;
use super::intent::{WorkspaceTabStripIntent, dispatch_intent};
use super::kernel::WorkspaceTabStripDropTarget;

pub(super) fn tab_pointer_down_handler(
    drag_model: Model<WorkspaceTabStripDragState>,
    tab_id: Arc<str>,
    tab_activate_command: CommandId,
    pane_activate_cmd: Option<CommandId>,
    tab_close_command: Option<CommandId>,
    show_close_button: bool,
    dnd: ui_dnd::DndServiceModel,
    dnd_scope: ui_dnd::DndScopeId,
) -> OnPressablePointerDown {
    Arc::new(
        move |host: &mut dyn UiPointerActionHost, acx: ActionCx, down| {
            host.prevent_default(DefaultAction::FocusOnPointerDown);

            if show_close_button
                && down.button == MouseButton::Left
                && down.modifiers == Modifiers::default()
            {
                // Prevent the tab pressable from arming when clicking the close affordance.
                //
                // Rationale: without this, the tab pressable can observe the pointer-down that
                // targets the nested close button pressable, leading to accidental activation or
                // DnD capture when the intent is "close without activation".
                if tab_strip_arbitration::tab_close_hit_test(
                    host.bounds(),
                    down.position_local,
                    TAB_CLOSE_SIZE,
                    TAB_CHROME_PAD_RIGHT,
                ) {
                    return PressablePointerDownResult::SkipDefault;
                }
            }

            match down.button {
                MouseButton::Middle => {
                    if let Some(cmd) = pane_activate_cmd.clone() {
                        host.record_pending_command_dispatch_source(
                            acx,
                            &cmd,
                            ActivateReason::Pointer,
                        );
                        dispatch_intent(host, acx.window, WorkspaceTabStripIntent::Activate(cmd));
                    }
                    if let Some(cmd) = tab_close_command.clone() {
                        host.record_pending_command_dispatch_source(
                            acx,
                            &cmd,
                            ActivateReason::Pointer,
                        );
                        dispatch_intent(host, acx.window, WorkspaceTabStripIntent::Close(cmd));
                        dispatch_intent(host, acx.window, WorkspaceTabStripIntent::RequestRedraw);
                    }
                    host.prevent_default(DefaultAction::FocusOnPointerDown);
                    return PressablePointerDownResult::SkipDefaultAndStopPropagation;
                }
                MouseButton::Right => {
                    if let Some(cmd) = pane_activate_cmd.clone() {
                        host.record_pending_command_dispatch_source(
                            acx,
                            &cmd,
                            ActivateReason::Pointer,
                        );
                        dispatch_intent(host, acx.window, WorkspaceTabStripIntent::Activate(cmd));
                    }
                    host.record_pending_command_dispatch_source(
                        acx,
                        &tab_activate_command,
                        ActivateReason::Pointer,
                    );
                    dispatch_intent(
                        host,
                        acx.window,
                        WorkspaceTabStripIntent::Activate(tab_activate_command.clone()),
                    );
                    dispatch_intent(host, acx.window, WorkspaceTabStripIntent::RequestRedraw);
                    host.prevent_default(DefaultAction::FocusOnPointerDown);
                    // Allow the surrounding context-menu trigger to observe the right-click.
                    return PressablePointerDownResult::Continue;
                }
                _ => {}
            }

            if down.button != MouseButton::Left {
                return PressablePointerDownResult::Continue;
            }
            if down.modifiers != Modifiers::default() {
                return PressablePointerDownResult::Continue;
            }

            host.capture_pointer();
            let _ = host.models_mut().update(&drag_model, |st| {
                st.pointer = Some(down.pointer_id);
                st.start_tick = down.tick_id;
                st.start_position = down.position;
                st.dragged_tab = Some(tab_id.clone());
                st.dragging = false;
                st.drop_target = WorkspaceTabStripDropTarget::None;
            });
            ui_dnd::clear_pointer_in_scope(
                host.models_mut(),
                &dnd,
                acx.window,
                DRAG_KIND_WORKSPACE_TAB,
                dnd_scope,
                down.pointer_id,
            );
            PressablePointerDownResult::Continue
        },
    )
}
