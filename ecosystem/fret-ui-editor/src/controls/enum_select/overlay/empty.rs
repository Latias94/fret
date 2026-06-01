//! Enum-select overlay empty-state rendering owner.

use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::readout::editor_popup_empty_text_props;

pub(in crate::controls::enum_select::overlay) fn enum_select_empty_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    row_height: Px,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    cx.text_props(editor_popup_empty_text_props(
        Arc::from("No matches"),
        editor_muted_foreground(theme),
        row_height,
    ))
}
