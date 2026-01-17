use std::sync::Arc;

use fret_core::{FontId, FontWeight, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

use crate::layout as shadcn_layout;

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
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Alert {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            variant: AlertVariant::Default,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        alert_with_patch(cx, self.variant, self.children, self.chrome, self.layout)
    }
}

pub fn alert<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: AlertVariant,
    children: Vec<AnyElement>,
) -> AnyElement {
    alert_with_patch(
        cx,
        variant,
        children,
        ChromeRefinement::default(),
        LayoutRefinement::default(),
    )
}

fn alert_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: AlertVariant,
    children: Vec<AnyElement>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let bg = theme.color_required("background");
    let border = theme.color_required("border");
    let destructive = theme.color_required("destructive");

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
            .border_color(ColorRef::Color(border))
            .merge(chrome_override),
        LayoutRefinement::default().merge(layout_override),
    );

    let container = shadcn_layout::container_vstack_gap(cx, props, Space::N1p5, children);
    cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Alert,
            ..Default::default()
        },
        move |_cx| vec![container],
    )
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
        let fg = theme.color_required("foreground");
        let px = theme
            .metric_by_key("component.alert.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.alert.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
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
        let fg = theme.color_required("muted-foreground");
        let px = theme
            .metric_by_key("component.alert.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.alert.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}
