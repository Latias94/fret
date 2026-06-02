use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::AnyElement;

use super::super::control_chrome::{self, ImUiControlPalette};

mod indicators;

pub(super) use indicators::{checkbox_indicator, radio_indicator, switch_state_badge};

pub(super) fn boolean_label<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    label: Arc<str>,
    palette: ImUiControlPalette,
) -> AnyElement {
    control_chrome::fill_text(cx, label, palette.foreground)
}
