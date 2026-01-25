use std::sync::Arc;

use fret_core::Color;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

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
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Badge {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
            children: Vec::new(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
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
        badge_with_patch(
            cx,
            self.label,
            self.variant,
            self.children,
            self.chrome,
            self.layout,
        )
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
    badge_with_patch(
        cx,
        label,
        variant,
        Vec::new(),
        ChromeRefinement::default(),
        LayoutRefinement::default(),
    )
}

fn badge_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: impl Into<Arc<str>>,
    variant: BadgeVariant,
    children: Vec<AnyElement>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
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
    chrome = chrome.merge(chrome_override);

    let fg = fg_for(&theme, variant);

    let props = decl_style::container_props(
        &theme,
        chrome,
        LayoutRefinement::default()
            .overflow_hidden()
            .merge(layout_override),
    );

    let text_px = theme
        .metric_by_key("component.badge.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.badge.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    cx.container(props, |cx| {
        let label = ui::text(cx, label)
            .text_size_px(text_px)
            .line_height_px(line_height)
            .font_semibold()
            .nowrap()
            .text_color(ColorRef::Color(fg))
            .h_px(MetricRef::Px(line_height))
            .into_element(cx);

        if children.is_empty() {
            vec![label]
        } else {
            let mut content = Vec::with_capacity(children.len() + 1);
            content.extend(children);
            content.push(label);

            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .justify_center()
                    .items_center()
                    .gap_x(Space::N1),
                |_cx| content,
            )]
        }
    })
}
