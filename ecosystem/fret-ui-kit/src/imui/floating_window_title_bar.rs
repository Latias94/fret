use std::sync::Arc;

use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::element::AnyElement;

mod behavior;
mod row;

pub(in crate::imui) use row::floating_window_title_bar_row;

pub(super) fn floating_window_close_glyph_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    crate::declarative::text::text_chrome_glyph(cx, Arc::<str>::from("\u{00D7}"))
}

#[cfg(test)]
mod tests;
