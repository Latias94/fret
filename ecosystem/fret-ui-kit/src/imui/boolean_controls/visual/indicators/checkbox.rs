use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::AnyElement;

use super::super::super::super::control_chrome::{self, ImUiControlPalette};

pub(in crate::imui::boolean_controls) fn checkbox_indicator<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    palette: ImUiControlPalette,
    value: bool,
) -> AnyElement {
    control_chrome::pill(
        cx,
        Arc::from(if value { "[x]" } else { "[ ]" }),
        if value {
            palette.accent_background
        } else {
            palette.subtle_background
        },
        if value {
            palette.accent_foreground
        } else {
            palette.muted_foreground
        },
    )
}
