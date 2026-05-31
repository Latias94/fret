use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiPointerActionHost};

use super::super::super::super::model::format_hex;
use super::{alpha_from_local_x, alpha_from_local_y};

pub(super) fn apply_alpha_bar_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    x: f32,
) {
    let width = host.bounds().size.width.0;
    let alpha = alpha_from_local_x(x, width);
    apply_alpha_value(host, action_cx, model, draft, error, alpha);
}

pub(super) fn apply_vertical_alpha_bar_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    y: f32,
) {
    let height = host.bounds().size.height.0;
    let alpha = alpha_from_local_y(y, height);
    apply_alpha_value(host, action_cx, model, draft, error, alpha);
}

fn apply_alpha_value(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    alpha: f32,
) {
    let mut next = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    next.a = alpha;
    let formatted = format_hex(next, true);

    let _ = host.models_mut().update(model, |c| *c = next);
    let _ = host
        .models_mut()
        .update(draft, |s| *s = formatted.as_ref().to_string());
    let _ = host.models_mut().update(error, |e| *e = None);
    host.request_redraw(action_cx.window);
}
