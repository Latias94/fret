use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActivateReason, OnActivate};

use crate::theme::EditorThemePreset;

#[allow(clippy::arc_with_non_send_sync)]
pub(super) fn theme_preset_row_activate(
    model: Model<EditorThemePreset>,
    preset: EditorThemePreset,
) -> OnActivate {
    Arc::new(move |host, action_cx, _reason: ActivateReason| {
        let _ = host.models_mut().update(&model, |value| *value = preset);
        host.request_redraw(action_cx.window);
    })
}
