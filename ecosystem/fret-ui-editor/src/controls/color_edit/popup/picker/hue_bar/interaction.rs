use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiPointerActionHost};

use super::super::super::super::model::{hsv_from_color, hue_from_local_y};
use super::super::apply_hsv_color;

pub(super) fn apply_hue_bar_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    y: f32,
) {
    let height = host.bounds().size.height.0;
    let current = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    let mut next_hsv = hsv_from_color(current);
    next_hsv.hue = hue_from_local_y(y, height);
    apply_hsv_color(
        host, action_cx, model, draft, error, show_alpha, current, next_hsv,
    );
}
