use std::sync::{Arc, Mutex};

use fret_core::{Color, MouseButton};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, PressablePointerDownResult, PressablePointerUpResult, UiPointerActionHost,
};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::model::{
    HueWheelDragTarget, hsv_from_color, hsv_with_hue_wheel_position,
    hue_wheel_target_from_local_position,
};
use super::super::apply_hsv_color;

pub(super) fn install_hue_wheel_pointer_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
) {
    let drag_target = Arc::new(Mutex::new(None::<HueWheelDragTarget>));
    let target_for_down = Arc::clone(&drag_target);
    let target_for_move = Arc::clone(&drag_target);
    let target_for_up = Arc::clone(&drag_target);

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

        let current = host
            .models_mut()
            .get_copied(&model_for_down)
            .unwrap_or(Color::TRANSPARENT);
        let hsv = hsv_from_color(current);
        let bounds = host.bounds();
        let target = hue_wheel_target_from_local_position(
            hsv,
            down.position_local.x.0,
            down.position_local.y.0,
            bounds.size.width.0,
            bounds.size.height.0,
        );
        let Some(target) = target else {
            return PressablePointerDownResult::Continue;
        };

        if let Ok(mut slot) = target_for_down.lock() {
            *slot = Some(target);
        }
        apply_hue_wheel_position(
            host,
            action_cx,
            &model_for_down,
            &draft_for_down,
            &error_for_down,
            show_alpha,
            target,
            down.position_local.x.0,
            down.position_local.y.0,
        );
        host.capture_pointer();
        PressablePointerDownResult::Continue
    }));

    cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
        if !mv.buttons.left {
            if let Ok(mut slot) = target_for_move.lock() {
                *slot = None;
            }
            host.release_pointer_capture();
            return false;
        }

        let target = target_for_move.lock().ok().and_then(|slot| *slot);
        let Some(target) = target else {
            return false;
        };
        apply_hue_wheel_position(
            host,
            action_cx,
            &model_for_move,
            &draft_for_move,
            &error_for_move,
            show_alpha,
            target,
            mv.position_local.x.0,
            mv.position_local.y.0,
        );
        true
    }));

    cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
        if let Ok(mut slot) = target_for_up.lock() {
            *slot = None;
        }
        host.release_pointer_capture();
        PressablePointerUpResult::Continue
    }));
}

#[allow(clippy::too_many_arguments)]
fn apply_hue_wheel_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    target: HueWheelDragTarget,
    x: f32,
    y: f32,
) {
    let bounds = host.bounds();
    let current = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    let current_hsv = hsv_from_color(current);
    let next_hsv = hsv_with_hue_wheel_position(
        current_hsv,
        x,
        y,
        bounds.size.width.0,
        bounds.size.height.0,
        target,
    );
    apply_hsv_color(
        host, action_cx, model, draft, error, show_alpha, current, next_hsv,
    );
}
