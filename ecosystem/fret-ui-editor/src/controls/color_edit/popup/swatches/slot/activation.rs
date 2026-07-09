use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};

use super::super::super::super::model::{color_from_rgb_preserving_alpha, format_hex};

pub(super) struct PresetSwatchActivateArgs {
    pub(super) model: Model<Color>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) open: Model<bool>,
    pub(super) color: Color,
    pub(super) rgb: u32,
    pub(super) show_alpha: bool,
}

#[allow(clippy::arc_with_non_send_sync)]
pub(super) fn preset_swatch_on_activate(args: PresetSwatchActivateArgs) -> OnActivate {
    let PresetSwatchActivateArgs {
        model,
        draft,
        error,
        open,
        color,
        rgb,
        show_alpha,
    } = args;

    Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
        let current = host.models_mut().get_copied(&model).unwrap_or(color);
        let color = color_from_rgb_preserving_alpha(rgb, current.a);
        let formatted = format_hex(color, show_alpha);
        let _ = host.models_mut().update(&model, |c| *c = color);
        let _ = host
            .models_mut()
            .update(&draft, |s| *s = formatted.as_ref().to_string());
        let _ = host.models_mut().update(&error, |e| *e = None);
        let _ = host.models_mut().update(&open, |v| *v = false);
        host.request_redraw(action_cx.window);
    })
}
