use std::sync::Arc;

use fret_core::{Color, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;

use super::super::model::format_hex;
use super::super::{ColorEditEyedropperRequest, OnColorEditEyedropper};
use super::options::option_button;

pub(super) fn color_eyedropper_action<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    frame_current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    on_eyedropper: OnColorEditEyedropper,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let row_height = {
        let theme = Theme::global(&*cx.app);
        EditorDensity::resolve(theme).row_height
    };
    let activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let current = host
                .models_mut()
                .get_copied(&model)
                .unwrap_or(frame_current);
            let request = ColorEditEyedropperRequest::new(current, show_alpha);
            let sampled = on_eyedropper(host, action_cx, request);

            if let Some(sampled) = sampled {
                let next = request.apply_sample(sampled);
                let formatted = format_hex(next, show_alpha);
                let _ = host.models_mut().update(&model, |color| *color = next);
                let _ = host
                    .models_mut()
                    .update(&draft, |text| *text = formatted.as_ref().to_string());
                let _ = host.models_mut().update(&error, |value| *value = None);
            }

            host.request_redraw(action_cx.window);
        });

    option_button(
        cx,
        "Eyedropper",
        SemanticsRole::Button,
        false,
        enabled,
        row_height,
        test_id,
        activate,
    )
}
