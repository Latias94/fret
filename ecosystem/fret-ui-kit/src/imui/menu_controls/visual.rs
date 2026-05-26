use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

pub(super) fn menu_item_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
) -> AnyElement {
    crate::declarative::text::text_list_row_label(cx, label)
}

pub(super) fn menu_item_shortcut_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    shortcut: Arc<str>,
) -> AnyElement {
    crate::declarative::text::text_control_readout(cx, shortcut)
}

pub(super) fn menu_item_indicator_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    indicator: Arc<str>,
) -> AnyElement {
    crate::declarative::text::text_chrome_glyph(cx, indicator)
}
