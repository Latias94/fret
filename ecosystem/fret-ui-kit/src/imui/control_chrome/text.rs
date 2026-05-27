use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, UiHost};

use super::{CONTROL_RADIUS, ImUiControlPalette};

pub(in crate::imui) fn control_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    color: Color,
) -> AnyElement {
    crate::declarative::text::text_button_label(cx, text).inherit_foreground(color)
}

pub(in crate::imui) fn fill_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    color: Color,
) -> AnyElement {
    crate::declarative::text::text_control_label(cx, text).inherit_foreground(color)
}

pub(in crate::imui) fn caption_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    palette: ImUiControlPalette,
) -> AnyElement {
    fill_text(cx, text, palette.muted_foreground)
}

pub(in crate::imui) fn pill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    bg: Color,
    fg: Color,
) -> AnyElement {
    let mut chrome = ContainerProps::default();
    chrome.padding = Edges {
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();
    chrome.background = Some(bg);
    chrome.corner_radii = Corners::all(CONTROL_RADIUS);

    cx.container(chrome, move |cx| vec![control_text(cx, text, fg)])
}
