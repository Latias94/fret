use fret_ui::element::{AnyElement, ScrollProps};
use fret_ui::{ElementCx, Theme, UiHost};

use crate::LayoutRefinement;
use crate::declarative::style;

/// Component-layer scroll helper (typed, declarative).
///
/// Fret treats scrolling as an explicit element (not a boolean overflow flag). This wrapper exists
/// to match gpui/tailwind ergonomics while keeping the runtime contract explicit.
pub fn overflow_scroll<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar: bool,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let layout = style::layout_style(theme, layout);
    cx.scroll(
        ScrollProps {
            layout,
            show_scrollbar,
            scroll_handle: None,
        },
        f,
    )
}

pub fn overflow_scrollbar<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    layout: LayoutRefinement,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    overflow_scroll(cx, layout, true, f)
}
