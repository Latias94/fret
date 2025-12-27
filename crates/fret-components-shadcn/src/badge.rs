use std::sync::Arc;

use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_core::Color;
use fret_core::{FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, TextProps};
use fret_ui::{ElementCx, Theme, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
}

#[derive(Debug, Clone)]
pub struct Badge {
    label: Arc<str>,
    variant: BadgeVariant,
}

impl Badge {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        badge(cx, self.label, self.variant)
    }
}

fn border_color(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border)
}

fn fg_for(theme: &Theme, variant: BadgeVariant) -> Color {
    match variant {
        BadgeVariant::Default => theme
            .color_by_key("primary-foreground")
            .or_else(|| theme.color_by_key("primary.foreground"))
            .unwrap_or(theme.colors.text_primary),
        BadgeVariant::Secondary => theme
            .color_by_key("secondary-foreground")
            .or_else(|| theme.color_by_key("secondary.foreground"))
            .unwrap_or(theme.colors.text_primary),
        BadgeVariant::Destructive => theme
            .color_by_key("destructive-foreground")
            .or_else(|| theme.color_by_key("destructive.foreground"))
            .unwrap_or(theme.colors.text_primary),
        BadgeVariant::Outline => theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary),
    }
}

fn bg_for(theme: &Theme, variant: BadgeVariant) -> Option<Color> {
    match variant {
        BadgeVariant::Default => Some(theme.color_by_key("primary").unwrap_or(theme.colors.accent)),
        BadgeVariant::Secondary => Some(
            theme
                .color_by_key("secondary")
                .unwrap_or(theme.colors.panel_background),
        ),
        BadgeVariant::Destructive => Some(
            theme
                .color_by_key("destructive")
                .unwrap_or(theme.colors.accent),
        ),
        BadgeVariant::Outline => None,
    }
}

pub fn badge<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    label: impl Into<Arc<str>>,
    variant: BadgeVariant,
) -> AnyElement {
    let label = label.into();
    let theme = Theme::global(&*cx.app).clone();

    let mut chrome = ChromeRefinement::default()
        .px(Space::N2p5)
        .py(Space::N0p5)
        .rounded(Radius::Full)
        .border_1()
        .border_color(ColorRef::Color(border_color(&theme)));
    if let Some(bg) = bg_for(&theme, variant) {
        chrome = chrome.bg(ColorRef::Color(bg));
    }

    let fg = fg_for(&theme, variant);

    let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());

    let text_px = theme
        .metric_by_key("component.badge.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.badge.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);

    cx.container(props, |cx| {
        vec![cx.text_props(TextProps {
            layout: Default::default(),
            text: label,
            style: Some(TextStyle {
                font: FontId::default(),
                size: text_px,
                weight: FontWeight::SEMIBOLD,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })]
    })
}
