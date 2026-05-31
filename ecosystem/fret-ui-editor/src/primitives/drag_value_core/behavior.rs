use std::sync::{Arc, Mutex};

use fret_core::{KeyCode, MouseButton};
use fret_ui::action::{
    ActionCx, PressablePointerDownResult, PressablePointerUpResult, UiActionHost,
};
use fret_ui::{ElementContext, UiHost};

use super::super::constrain_numeric_value;
use super::DragValueScalar;
use super::options::DragValueCoreOptions;
use super::state::{DragState, DragValueCoreMoveAction, resolve_scrub_multiplier};

pub(super) fn install_drag_value_core_behavior<T, H>(
    cx: &mut ElementContext<'_, H>,
    state: Arc<Mutex<DragState<T>>>,
    opts: DragValueCoreOptions,
    on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static>,
    on_commit: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    on_cancel: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
) where
    T: DragValueScalar + Default,
    H: UiHost,
{
    let pressable_id = cx.root_id();
    let opts_for_down = opts;
    let state_for_down = state.clone();
    cx.pressable_add_on_pointer_down(Arc::new(move |host, _action_cx, down| {
        if down.button != MouseButton::Left {
            return PressablePointerDownResult::Continue;
        }

        if !opts_for_down.scrub_on_double_click && down.click_count >= 2 {
            return PressablePointerDownResult::SkipDefaultAndStopPropagation;
        }

        // Own focus for the active scrub session so Escape cancel routes to this control even when
        // the gesture started from a pointer-only interaction.
        host.request_focus(pressable_id);
        host.capture_pointer();

        let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
        st.begin_session(down.pointer_id, down.position_local);
        PressablePointerDownResult::Continue
    }));

    let opts_for_move = opts;
    let state_for_move = state.clone();
    let on_change_live_for_move = on_change_live.clone();
    let on_commit_for_move = on_commit.clone();
    let on_cancel_for_move = on_cancel.clone();
    cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
        let action = {
            let mut st = state_for_move.lock().unwrap_or_else(|e| e.into_inner());
            if !st.armed || st.pointer_id != Some(mv.pointer_id) {
                return false;
            }

            // Best-effort cleanup for unexpected end-of-stream: if the runtime reports no pressed
            // left button while armed, treat it like pointer up/cancel to avoid stuck sessions.
            if !mv.buttons.left {
                let was_dragging = st.dragging;
                st.clear_pointer_session();
                if st.session.is_active() {
                    if was_dragging {
                        let edited = st.commit_session();
                        DragValueCoreMoveAction::Commit { edited }
                    } else {
                        let pre = st.cancel_session();
                        DragValueCoreMoveAction::Cancel(pre)
                    }
                } else {
                    DragValueCoreMoveAction::None
                }
            } else {
                if !st.dragging {
                    let dx = mv.position_local.x.0 - st.down_pos.x.0;
                    let dy = mv.position_local.y.0 - st.down_pos.y.0;
                    let dist2 = (dx as f64) * (dx as f64) + (dy as f64) * (dy as f64);
                    let th = opts_for_move.drag_threshold.0 as f64;
                    if dist2 < th * th {
                        return false;
                    }

                    st.dragging = true;
                    // Reset the delta origin when crossing the threshold to avoid an initial jump.
                    st.start_x = mv.position_local.x.0 as f64;
                    st.down_pos = mv.position_local;
                }

                let delta_x = mv.position_local.x.0 as f64 - st.start_x;
                let multiplier = resolve_scrub_multiplier(
                    mv.modifiers,
                    opts_for_move.slow_multiplier,
                    opts_for_move.fast_multiplier,
                );
                let delta = delta_x * opts_for_move.scrub_speed * multiplier;
                let next = constrain_numeric_value(
                    opts_for_move.constraints,
                    T::from_f64(st.start_value.to_f64() + delta),
                );
                if st.apply_live_value(next) {
                    DragValueCoreMoveAction::Live(next)
                } else {
                    DragValueCoreMoveAction::None
                }
            }
        };

        match action {
            DragValueCoreMoveAction::None => false,
            DragValueCoreMoveAction::Live(next) => {
                (on_change_live_for_move)(host, action_cx, next);
                true
            }
            DragValueCoreMoveAction::Commit { edited } => {
                host.release_pointer_capture();
                if edited && let Some(cb) = on_commit_for_move.as_ref() {
                    cb(host, action_cx);
                }
                host.request_redraw(action_cx.window);
                true
            }
            DragValueCoreMoveAction::Cancel(pre) => {
                host.release_pointer_capture();
                if let Some(pre) = pre {
                    (on_change_live_for_move)(host, action_cx, pre);
                }
                if let Some(cb) = on_cancel_for_move.as_ref() {
                    cb(host, action_cx);
                }
                host.request_redraw(action_cx.window);
                true
            }
        }
    }));

    let state_for_up = state.clone();
    let on_commit_for_up = on_commit;
    cx.pressable_add_on_pointer_up(Arc::new(move |host, action_cx, up| {
        if up.button != MouseButton::Left {
            return PressablePointerUpResult::Continue;
        }

        let mut st = state_for_up.lock().unwrap_or_else(|e| e.into_inner());
        if st.pointer_id.is_some() && st.pointer_id != Some(up.pointer_id) {
            return PressablePointerUpResult::Continue;
        }
        let was_dragging = st.dragging;
        st.clear_pointer_session();
        let edited = st.commit_session();
        host.release_pointer_capture();
        if was_dragging
            && edited
            && let Some(cb) = on_commit_for_up.as_ref()
        {
            cb(host, action_cx);
        }
        PressablePointerUpResult::Continue
    }));

    let state_for_key = state;
    let on_change_live_for_key = on_change_live;
    let on_cancel_for_key = on_cancel;
    cx.key_add_on_key_down_capture_for(
        cx.root_id(),
        Arc::new(move |host, action_cx, key| {
            if key.key != KeyCode::Escape {
                return false;
            }

            let mut st = state_for_key.lock().unwrap_or_else(|e| e.into_inner());
            if !st.session.is_active() {
                return false;
            }

            st.clear_pointer_session();
            if let Some(pre) = st.cancel_session() {
                (on_change_live_for_key)(host, action_cx, pre);
            }
            if let Some(cb) = on_cancel_for_key.as_ref() {
                cb(host, action_cx);
            }
            true
        }),
    );
}
