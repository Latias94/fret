//! Slider pressable pointer interaction owner.

use std::sync::{Arc, Mutex};

use fret_core::{CursorIcon, MouseButton, Px};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, arm_numeric_text_entry_focus_handoff,
};

use super::super::model::SliderState;
use super::super::pointer::{
    begin_slider_drag, clear_slider_drag, enter_slider_typing, finish_slider_drag,
    is_slider_drag_pointer,
};
use super::super::value_math::value_from_slider_local_x;

pub(super) struct SliderInteractionHandlersArgs<T> {
    pub(super) state: Arc<Mutex<SliderState>>,
    pub(super) focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>>,
    pub(super) model: Model<T>,
    pub(super) interactive_enabled: bool,
    pub(super) allow_typing: bool,
    pub(super) min: f64,
    pub(super) max: f64,
    pub(super) clamp: bool,
    pub(super) step: Option<f64>,
    pub(super) show_value: bool,
    pub(super) value_width: Px,
    pub(super) frame_padding_left: Px,
    pub(super) frame_padding_right: Px,
    pub(super) thumb_d: Px,
}

pub(super) fn install_slider_interaction_handlers<T, H>(
    cx: &mut ElementContext<'_, H>,
    args: SliderInteractionHandlersArgs<T>,
) where
    T: DragValueScalar + Default,
    H: UiHost,
{
    let SliderInteractionHandlersArgs {
        state,
        focus_handoff,
        model,
        interactive_enabled,
        allow_typing,
        min,
        max,
        clamp,
        step,
        show_value,
        value_width,
        frame_padding_left,
        frame_padding_right,
        thumb_d,
    } = args;

    let state_for_down = state.clone();
    let focus_handoff_for_down = focus_handoff;
    let model_for_down = model.clone();
    cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
        if !interactive_enabled {
            return PressablePointerDownResult::Continue;
        }
        if allow_typing && down.button == MouseButton::Left && down.click_count >= 2 {
            let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
            enter_slider_typing(&mut st);
            {
                let mut handoff = focus_handoff_for_down
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                arm_numeric_text_entry_focus_handoff(&mut handoff);
            }
            host.request_redraw(action_cx.window);
            return PressablePointerDownResult::SkipDefaultAndStopPropagation;
        }

        if down.button != MouseButton::Left {
            return PressablePointerDownResult::Continue;
        }

        let bounds = host.bounds();
        let next = value_from_slider_local_x(
            min,
            max,
            clamp,
            step,
            down.position_local.x.0 as f64,
            bounds.size.width.0 as f64,
            show_value,
            value_width.0 as f64,
            frame_padding_left.0 as f64,
            frame_padding_right.0 as f64,
            thumb_d.0 as f64,
        );
        let next_t = T::from_f64(next);
        let _ = host.models_mut().update(&model_for_down, |v| *v = next_t);
        host.request_redraw(action_cx.window);

        host.set_cursor_icon(CursorIcon::ColResize);

        let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
        begin_slider_drag(&mut st, down.pointer_id);

        PressablePointerDownResult::Continue
    }));

    let state_for_move = state.clone();
    let model_for_move = model;
    cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
        let mut st_lock = state_for_move.lock().unwrap_or_else(|e| e.into_inner());
        if !is_slider_drag_pointer(&st_lock, mv.pointer_id) {
            return false;
        }

        // Best-effort cleanup when the pointer-up event is missed.
        if !mv.buttons.left {
            clear_slider_drag(&mut st_lock);
            return false;
        }

        let bounds = host.bounds();
        let next = value_from_slider_local_x(
            min,
            max,
            clamp,
            step,
            mv.position_local.x.0 as f64,
            bounds.size.width.0 as f64,
            show_value,
            value_width.0 as f64,
            frame_padding_left.0 as f64,
            frame_padding_right.0 as f64,
            thumb_d.0 as f64,
        );
        let next_t = T::from_f64(next);
        let _ = host.models_mut().update(&model_for_move, |v| *v = next_t);
        host.request_redraw(action_cx.window);
        true
    }));

    let state_for_up = state;
    cx.pressable_add_on_pointer_up(Arc::new(move |_host, _action_cx, up| {
        let mut st = state_for_up.lock().unwrap_or_else(|e| e.into_inner());
        finish_slider_drag(&mut st, up.pointer_id);
        PressablePointerUpResult::Continue
    }));
}
