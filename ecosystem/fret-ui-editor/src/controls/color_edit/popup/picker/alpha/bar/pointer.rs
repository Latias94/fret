use std::sync::Arc;

use fret_core::{Color, MouseButton};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::{ElementContext, UiHost};

use super::super::interaction::{apply_alpha_bar_position, apply_vertical_alpha_bar_position};

pub(super) fn install_vertical_alpha_bar_pointer_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
) {
    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
        if down.button != MouseButton::Left {
            return PressablePointerDownResult::Continue;
        }
        apply_vertical_alpha_bar_position(
            host,
            action_cx,
            &model_for_down,
            &draft_for_down,
            &error_for_down,
            down.position_local.y.0,
        );
        host.capture_pointer();
        PressablePointerDownResult::Continue
    }));

    cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
        if !mv.buttons.left {
            host.release_pointer_capture();
            return false;
        }
        apply_vertical_alpha_bar_position(
            host,
            action_cx,
            &model_for_move,
            &draft_for_move,
            &error_for_move,
            mv.position_local.y.0,
        );
        true
    }));
    install_alpha_bar_pointer_release(cx);
}

pub(super) fn install_alpha_bar_pointer_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
) {
    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
        if down.button != MouseButton::Left {
            return PressablePointerDownResult::Continue;
        }
        apply_alpha_bar_position(
            host,
            action_cx,
            &model_for_down,
            &draft_for_down,
            &error_for_down,
            down.position_local.x.0,
        );
        host.capture_pointer();
        PressablePointerDownResult::Continue
    }));

    cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
        if !mv.buttons.left {
            host.release_pointer_capture();
            return false;
        }
        apply_alpha_bar_position(
            host,
            action_cx,
            &model_for_move,
            &draft_for_move,
            &error_for_move,
            mv.position_local.x.0,
        );
        true
    }));
    install_alpha_bar_pointer_release(cx);
}

fn install_alpha_bar_pointer_release<H: UiHost>(cx: &mut ElementContext<'_, H>) {
    cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
        host.release_pointer_capture();
        PressablePointerUpResult::Continue
    }));
}
