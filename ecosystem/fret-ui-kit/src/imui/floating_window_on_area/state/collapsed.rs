use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{FloatingWindowOptions, KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED};

pub(super) fn read_floating_window_collapsed<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    options: FloatingWindowOptions,
) -> bool {
    let collapsed_model = super::super::super::float_window_collapsed_model_for(cx, window_id);
    if options.inputs_enabled
        && options.collapsible
        && cx.take_transient_for(window_id, KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED)
    {
        let _ = cx.app.models_mut().update(&collapsed_model, |v| {
            *v = !*v;
        });
    }
    cx.read_model(&collapsed_model, fret_ui::Invalidation::Paint, |_app, v| *v)
        .unwrap_or(false)
}
