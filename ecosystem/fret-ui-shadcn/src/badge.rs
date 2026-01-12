use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        badge(cx, self.label, self.variant)
    }
}

fn border_color(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn fg_for(theme: &Theme, variant: BadgeVariant) -> Color {
    match variant {
        BadgeVariant::Default => theme.color_required("primary-foreground"),
        BadgeVariant::Secondary => theme.color_required("secondary-foreground"),
        BadgeVariant::Destructive => theme.color_required("destructive-foreground"),
        BadgeVariant::Outline => theme.color_required("foreground"),
    }
}

fn bg_for(theme: &Theme, variant: BadgeVariant) -> Option<Color> {
    match variant {
        BadgeVariant::Default => Some(theme.color_required("primary")),
        BadgeVariant::Secondary => Some(theme.color_required("secondary")),
        BadgeVariant::Destructive => Some(theme.color_required("destructive")),
        BadgeVariant::Outline => None,
    }
}

pub fn badge<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: impl Into<Arc<str>>,
    variant: BadgeVariant,
) -> AnyElement {
    let label = label.into();
    let theme = Theme::global(&*cx.app).clone();

    let mut chrome = ChromeRefinement::default()
        .px(Space::N2)
        .py(Space::N0p5)
        .rounded(Radius::Full)
        .border_1()
        .border_color(ColorRef::Color(border_color(&theme)));
    if let Some(bg) = bg_for(&theme, variant) {
        chrome = chrome.bg(ColorRef::Color(bg));
    }

    let fg = fg_for(&theme, variant);

    let mut props = decl_style::container_props(
        &theme,
        chrome,
        LayoutRefinement::default().overflow_hidden(),
    );
    // Treat borders as part of the component's "outer size" to match the web box model:
    // shadcn uses `border` + `px-*`/`py-*` (border-box sizing). Our `ContainerProps.border` is a
    // paint-time concern, so we include the border thickness in layout padding.
    props.padding.top = Px((props.padding.top.0 + props.border.top.0).max(0.0));
    props.padding.right = Px((props.padding.right.0 + props.border.right.0).max(0.0));
    props.padding.bottom = Px((props.padding.bottom.0 + props.border.bottom.0).max(0.0));
    props.padding.left = Px((props.padding.left.0 + props.border.left.0).max(0.0));

    let text_px = theme
        .metric_by_key("component.badge.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.badge.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    let text_layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Auto,
            height: Length::Px(line_height),
            ..Default::default()
        },
        ..Default::default()
    };

    cx.container(props, |cx| {
        vec![cx.text_props(TextProps {
            layout: text_layout,
            text: label,
            style: Some(TextStyle {
                font: FontId::default(),
                size: text_px,
                weight: FontWeight::SEMIBOLD,
                slant: Default::default(),
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })]
    })
}
