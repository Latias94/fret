use std::sync::Arc;

use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_core::{FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};

#[derive(Debug, Clone)]
pub struct Kbd {
    text: Arc<str>,
}

impl Kbd {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        kbd(cx, self.text)
    }
}

pub fn kbd<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let text = text.into();
    let theme = Theme::global(&*cx.app).clone();

    let bg = theme
        .color_by_key("muted")
        .unwrap_or(theme.colors.panel_background);
    let border = theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border);

    let chrome = ChromeRefinement::default()
        .px(Space::N1p5)
        .py(Space::N0p5)
        .rounded(Radius::Sm)
        .border_1()
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border));

    let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());

    let fg = theme
        .color_by_key("muted.foreground")
        .or_else(|| theme.color_by_key("muted-foreground"))
        .unwrap_or(theme.colors.text_muted);

    let px = theme
        .metric_by_key("component.kbd.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.kbd.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);

    cx.container(props, |cx| {
        vec![cx.text_props(TextProps {
            layout: Default::default(),
            text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::MEDIUM,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })]
    })
}
