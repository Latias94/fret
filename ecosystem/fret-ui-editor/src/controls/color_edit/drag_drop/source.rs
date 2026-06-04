use fret_core::Px;
use fret_ui::{ElementContext, Theme, UiHost};

mod handlers;

const DEFAULT_COLOR_DRAG_THRESHOLD_PX: f32 = 6.0;

pub(in crate::controls::color_edit) use handlers::install_color_drag_source;

pub(in crate::controls::color_edit) fn resolve_color_drag_threshold<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Px {
    let threshold = Theme::global(&*cx.app)
        .metric_by_key(fret_ui_kit::theme_tokens::metric::COMPONENT_IMUI_DRAG_THRESHOLD_PX)
        .unwrap_or(Px(DEFAULT_COLOR_DRAG_THRESHOLD_PX));
    if threshold.0.is_finite() {
        Px(threshold.0.max(0.0))
    } else {
        Px(DEFAULT_COLOR_DRAG_THRESHOLD_PX)
    }
}
