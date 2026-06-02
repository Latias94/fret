//! Color-edit swatch popup activation owner.

use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};

use super::super::ColorEditPopupOptions;

pub(super) struct ColorSwatchActivateInput {
    pub(super) model: Model<Color>,
    pub(super) open: Model<bool>,
    pub(super) copy_menu_open: Model<bool>,
    pub(super) reference: Model<Option<Color>>,
    pub(super) popup_has_visible_content: bool,
    pub(super) popup_options: ColorEditPopupOptions,
}

pub(super) fn color_swatch_activate(input: ColorSwatchActivateInput) -> OnActivate {
    let ColorSwatchActivateInput {
        model,
        open,
        copy_menu_open,
        reference,
        popup_has_visible_content,
        popup_options,
    } = input;

    Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
        if !popup_has_visible_content {
            return;
        }
        let prev = host.models_mut().get_copied(&open).unwrap_or(false);
        let opening = !prev;
        if opening && popup_options.side_preview.shows_original() {
            let current = host
                .models_mut()
                .get_copied(&model)
                .unwrap_or(Color::TRANSPARENT);
            let _ = host
                .models_mut()
                .update(&reference, |reference| *reference = Some(current));
        }
        let _ = host.models_mut().update(&open, |v| *v = opening);
        let _ = host.models_mut().update(&copy_menu_open, |v| *v = false);
        host.request_redraw(action_cx.window);
    })
}
