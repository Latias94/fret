use std::sync::Arc;

use fret_core::{Modifiers, MouseButton};
use fret_runtime::{CommandId, DefaultAction, Model};
use fret_ui::action::{
    ActionCx, ActivateReason, OnPressablePointerDown, OnPressablePointerUp,
    PressablePointerDownResult, PressablePointerUpResult, UiPointerActionHost,
};

use fret_ui_kit::dnd as ui_dnd;
use fret_ui_kit::headless::tab_strip_arbitration;

use crate::tab_drag::DRAG_KIND_WORKSPACE_TAB;

use super::consts::{TAB_CHROME_PAD_RIGHT, TAB_CLOSE_CLICK_SLOP, TAB_CLOSE_SIZE};
use super::drag_state::WorkspaceTabStripClosePress;
use super::drag_state::WorkspaceTabStripDragState;
use super::intent::{WorkspaceTabStripIntent, dispatch_intent};
use super::kernel::WorkspaceTabStripDropTarget;

pub(super) fn tab_pointer_down_handler(
    drag_model: Model<WorkspaceTabStripDragState>,
    tab_id: Arc<str>,
    tab_activate_command: CommandId,
    pane_activate_cmd: Option<CommandId>,
    tab_close_command: Option<CommandId>,
    close_visible: bool,
    dnd: ui_dnd::DndServiceModel,
    dnd_scope: ui_dnd::DndScopeId,
) -> OnPressablePointerDown {
    Arc::new(
        move |host: &mut dyn UiPointerActionHost, acx: ActionCx, down| {
            host.prevent_default(DefaultAction::FocusOnPointerDown);

            if down.button == MouseButton::Left
                && down.modifiers == Modifiers::default()
                && close_visible
                && let Some(tab_close_command) = tab_close_command.clone()
                && tab_strip_arbitration::tab_close_hit_test(
                    host.bounds(),
                    down.position,
                    TAB_CLOSE_SIZE,
                    TAB_CHROME_PAD_RIGHT,
                )
            {
                host.capture_pointer();
                let close_command = tab_close_command.clone();
                let _ = host.models_mut().update(&drag_model, |st| {
                    st.pointer = None;
                    st.dragged_tab = None;
                    st.dragging = false;
                    st.drop_target = WorkspaceTabStripDropTarget::None;
                    st.close_press = Some(WorkspaceTabStripClosePress {
                        pointer_id: down.pointer_id,
                        start_position: down.position,
                        start_position_window: down.position_window,
                        close_command,
                        pane_activate_cmd: pane_activate_cmd.clone(),
                    });
                });
                ui_dnd::clear_pointer_in_scope(
                    host.models_mut(),
                    &dnd,
                    acx.window,
                    DRAG_KIND_WORKSPACE_TAB,
                    dnd_scope,
                    down.pointer_id,
                );
                return PressablePointerDownResult::SkipDefaultAndStopPropagation;
            }

            // If a nested close affordance already armed a `close_press` for this pointer, do not
            // arm the tab pressable for activation or DnD (avoid clearing the close press state).
            if down.button == MouseButton::Left && down.modifiers == Modifiers::default() {
                let mut close_armed = false;
                let _ = host.models_mut().read(&drag_model, |st| {
                    close_armed = st
                        .close_press
                        .as_ref()
                        .is_some_and(|p| p.pointer_id == down.pointer_id);
                });
                if close_armed {
                    return PressablePointerDownResult::SkipDefaultAndStopPropagation;
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
                st.start_position_window = down.position_window;
                st.dragged_tab = Some(tab_id.clone());
                st.dragging = false;
                st.drop_target = WorkspaceTabStripDropTarget::None;
                st.close_press = None;
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

pub(super) fn tab_close_pointer_down_handler(
    drag_model: Model<WorkspaceTabStripDragState>,
    pane_activate_cmd: Option<CommandId>,
    tab_close_command: CommandId,
    dnd: ui_dnd::DndServiceModel,
    dnd_scope: ui_dnd::DndScopeId,
) -> OnPressablePointerDown {
    Arc::new(
        move |host: &mut dyn UiPointerActionHost, acx: ActionCx, down| {
            host.prevent_default(DefaultAction::FocusOnPointerDown);

            if down.button != MouseButton::Left || down.modifiers != Modifiers::default() {
                return PressablePointerDownResult::Continue;
            }

            host.capture_pointer();
            let close_command = tab_close_command.clone();
            let _ = host.models_mut().update(&drag_model, |st| {
                st.pointer = None;
                st.dragged_tab = None;
                st.dragging = false;
                st.drop_target = WorkspaceTabStripDropTarget::None;
                st.close_press = Some(WorkspaceTabStripClosePress {
                    pointer_id: down.pointer_id,
                    start_position: down.position,
                    start_position_window: down.position_window,
                    close_command,
                    pane_activate_cmd: pane_activate_cmd.clone(),
                });
            });
            ui_dnd::clear_pointer_in_scope(
                host.models_mut(),
                &dnd,
                acx.window,
                DRAG_KIND_WORKSPACE_TAB,
                dnd_scope,
                down.pointer_id,
            );

            PressablePointerDownResult::SkipDefaultAndStopPropagation
        },
    )
}

pub(super) fn tab_pointer_up_handler(
    drag_model: Model<WorkspaceTabStripDragState>,
) -> OnPressablePointerUp {
    Arc::new(
        move |host: &mut dyn UiPointerActionHost, acx: ActionCx, up| {
            if up.button != MouseButton::Left || up.modifiers != Modifiers::default() {
                let _ = host.models_mut().update(&drag_model, |st| {
                    if st
                        .close_press
                        .as_ref()
                        .is_some_and(|p| p.pointer_id == up.pointer_id)
                    {
                        st.close_press = None;
                    }
                });
                return PressablePointerUpResult::Continue;
            }

            let mut press: Option<WorkspaceTabStripClosePress> = None;
            let _ = host.models_mut().update(&drag_model, |st| {
                if st
                    .close_press
                    .as_ref()
                    .is_some_and(|p| p.pointer_id == up.pointer_id)
                {
                    press = st.close_press.take();
                }
            });

            let Some(press) = press else {
                return PressablePointerUpResult::Continue;
            };

            let start = press.start_position_window.unwrap_or(press.start_position);
            let end = up.position_window.unwrap_or(up.position);
            let within_slop =
                tab_strip_arbitration::pointer_move_within_slop(start, end, TAB_CLOSE_CLICK_SLOP);
            if !within_slop {
                return PressablePointerUpResult::Continue;
            }

            if let Some(cmd) = press.pane_activate_cmd {
                host.record_pending_command_dispatch_source(acx, &cmd, ActivateReason::Pointer);
                host.dispatch_command(Some(acx.window), cmd);
            }

            host.record_pending_command_dispatch_source(
                acx,
                &press.close_command,
                ActivateReason::Pointer,
            );
            host.dispatch_command(Some(acx.window), press.close_command);
            host.request_redraw(acx.window);
            PressablePointerUpResult::SkipActivate
        },
    )
}
