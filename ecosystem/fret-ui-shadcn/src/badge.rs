use std::sync::Arc;

use fret_core::{Color, Point, Px, Transform2D};
use fret_ui::element::{AnyElement, ElementKind, LayoutStyle, Length, SpinnerProps, SvgIconProps};
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

    #[track_caller]
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
    theme.color_token("border")
}

fn fg_for(theme: &Theme, variant: BadgeVariant) -> Color {
    match variant {
        BadgeVariant::Default => theme.color_token("primary-foreground"),
        BadgeVariant::Secondary => theme.color_token("secondary-foreground"),
        // Upstream shadcn badge uses `text-white` for destructive.
        BadgeVariant::Destructive => theme
            .color_by_key("white")
            .unwrap_or_else(|| theme.color_token("destructive-foreground")),
        BadgeVariant::Outline => theme.color_token("foreground"),
    }
}

fn bg_for(theme: &Theme, variant: BadgeVariant) -> Option<Color> {
    match variant {
        BadgeVariant::Default => Some(theme.color_token("primary")),
        BadgeVariant::Secondary => Some(theme.color_token("secondary")),
        BadgeVariant::Destructive => Some(theme.color_token("destructive")),
        BadgeVariant::Outline => None,
    }
}

fn apply_badge_child_icon_size(mut element: AnyElement, icon_px: Px) -> AnyElement {
    fn set_layout_px(layout: &mut LayoutStyle, icon_px: Px) {
        layout.size.width = Length::Px(icon_px);
        layout.size.height = Length::Px(icon_px);
        layout.flex.shrink = 0.0;
    }

    match &mut element.kind {
        // Upstream enforces `[&>svg]:size-3` on direct children; apply to direct svg-ish children.
        ElementKind::SvgIcon(SvgIconProps { layout, .. }) => set_layout_px(layout, icon_px),
        ElementKind::Spinner(SpinnerProps { layout, .. }) => set_layout_px(layout, icon_px),
        ElementKind::VisualTransform(props) => {
            let (old_w, old_h) = match (props.layout.size.width, props.layout.size.height) {
                (Length::Px(w), Length::Px(h)) => (w, h),
                _ => (icon_px, icon_px),
            };
            let old_center = Point::new(Px(old_w.0 * 0.5), Px(old_h.0 * 0.5));

            set_layout_px(&mut props.layout, icon_px);
            let new_center = Point::new(Px(icon_px.0 * 0.5), Px(icon_px.0 * 0.5));

            // Keep the visual pivot stable when resizing a VisualTransform subtree (e.g. Spinner):
            // shift the transform so it rotates about the new center instead of the old one.
            let delta = Point::new(
                Px(new_center.x.0 - old_center.x.0),
                Px(new_center.y.0 - old_center.y.0),
            );
            let t = Transform2D::translation(delta);
            let t_inv = Transform2D::translation(Point::new(Px(-delta.x.0), Px(-delta.y.0)));
            props.transform = t * props.transform * t_inv;
        }
        _ => {}
    }

    element
}

fn apply_badge_inherited_fg(
    mut element: AnyElement,
    fg: Color,
    theme_fg: Color,
    theme_muted_fg: Color,
) -> AnyElement {
    match &mut element.kind {
        ElementKind::Text(props) => {
            if props.color.is_none() {
                props.color = Some(fg);
            }
        }
        ElementKind::SvgIcon(SvgIconProps { color, .. }) => {
            let is_default = *color
                == Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
            let should_override = is_default || *color == theme_fg || *color == theme_muted_fg;
            if *color != fg && should_override {
                *color = fg;
            }
        }
        ElementKind::Spinner(SpinnerProps { color, .. }) => {
            let should_override =
                color.is_none() || color.is_some_and(|c| c == theme_fg || c == theme_muted_fg);
            if should_override {
                *color = Some(fg);
            }
        }
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .map(|child| apply_badge_inherited_fg(child, fg, theme_fg, theme_muted_fg))
        .collect();
    element
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
    let theme_fg = theme.color_token("foreground");
    let theme_muted_fg = theme.color_by_key("muted-foreground").unwrap_or(theme_fg);

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
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.badge.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

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
            // Upstream shadcn badge enforces `[&>svg]:size-3` (12px) for direct svg children.
            let icon_px = Px(12.0);
            let mut content = Vec::with_capacity(children.len() + 1);

            let children = children
                .into_iter()
                .map(|child| apply_badge_child_icon_size(child, icon_px))
                .map(|child| apply_badge_inherited_fg(child, fg, theme_fg, theme_muted_fg))
                .collect::<Vec<_>>();
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
