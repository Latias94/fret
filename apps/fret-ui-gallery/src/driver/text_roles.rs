use fret_app::App;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::text as decl_text;
use std::sync::Arc;

pub(super) fn chrome_readout_text(
    cx: &mut ElementContext<'_, App>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_control_readout(cx, text)
}

pub(super) fn chrome_section_label(
    cx: &mut ElementContext<'_, App>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_section_chrome_label(cx, text)
}
