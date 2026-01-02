use std::sync::Arc;

use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_core::{FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

#[derive(Debug, Clone)]
pub struct Alert {
    children: Vec<AnyElement>,
    variant: AlertVariant,
}

impl Alert {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            variant: AlertVariant::Default,
        }
    }

    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        alert(cx, self.variant, self.children)
    }
}

pub fn alert<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: AlertVariant,
    children: Vec<AnyElement>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let bg = theme
        .color_by_key("background")
        .unwrap_or(theme.colors.surface_background);
    let border = theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border);

    let destructive = theme
        .color_by_key("destructive")
        .unwrap_or(theme.colors.accent);

    let (bg, border) = match variant {
        AlertVariant::Default => (bg, border),
        AlertVariant::Destructive => (bg, destructive),
    };

    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .p(Space::N4)
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border)),
        LayoutRefinement::default(),
    );

    cx.container(props, move |_cx| children)
}

#[derive(Debug, Clone)]
pub struct AlertTitle {
    text: Arc<str>,
}

impl AlertTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let px = theme
            .metric_by_key("component.alert.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.alert.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
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
        })
    }
}

#[derive(Debug, Clone)]
pub struct AlertDescription {
    text: Arc<str>,
}

impl AlertDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);
        let px = theme
            .metric_by_key("component.alert.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.alert.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::NORMAL,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}
