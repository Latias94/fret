use std::sync::Arc;

use fret_core::{Color, Corners, Point, Px, SemanticsRole, Transform2D};
use fret_icons::IconId;
use fret_runtime::Effect;
use fret_ui::ThemeNamedColorKey;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ElementKind, LayoutStyle, Length, PressableA11y, PressableKeyActivation,
    PressableProps, SpinnerProps, SvgIconProps,
};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

#[derive(Debug, Clone)]
pub enum BadgeRender {
    Link {
        href: Arc<str>,
        target: Option<Arc<str>>,
        rel: Option<Arc<str>>,
    },
}

fn open_url_on_activate(
    url: Arc<str>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
) -> OnActivate {
    Arc::new(move |host, _acx, _reason| {
        host.push_effect(Effect::OpenUrl {
            url: url.to_string(),
            target: target.as_ref().map(|v| v.to_string()),
            rel: rel.as_ref().map(|v| v.to_string()),
        });
    })
}

pub struct Badge {
    label: Arc<str>,
    variant: BadgeVariant,
    render: Option<BadgeRender>,
    visited: bool,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Badge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Badge")
            .field("label", &self.label.as_ref())
            .field("variant", &self.variant)
            .field("render", &self.render)
            .field("on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .field("children_len", &self.children.len())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Badge {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
            render: None,
            visited: false,
            on_activate: None,
            test_id: None,
            leading_icon: None,
            trailing_icon: None,
            children: Vec::new(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Adds a leading icon rendered under the badge's `currentColor` scope.
    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Adds a trailing icon rendered under the badge's `currentColor` scope.
    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn render(mut self, render: BadgeRender) -> Self {
        self.render = Some(render);
        self
    }

    /// Marks the badge's link as visited (when rendered as a link).
    pub fn visited(mut self, visited: bool) -> Self {
        self.visited = visited;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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
            self.render,
            self.visited,
            self.on_activate,
            self.test_id,
            self.leading_icon,
            self.trailing_icon,
            self.children,
            self.chrome,
            self.layout,
        )
    }
}

fn border_color(theme: &ThemeSnapshot) -> Color {
    theme.color_token("border")
}

fn fg_for(theme: &ThemeSnapshot, variant: BadgeVariant) -> Color {
    match variant {
        BadgeVariant::Default => theme.color_token("primary-foreground"),
        BadgeVariant::Secondary => theme.color_token("secondary-foreground"),
        // Upstream shadcn badge uses `text-white` for destructive.
        BadgeVariant::Destructive => theme.named_color(ThemeNamedColorKey::White),
        BadgeVariant::Outline => theme.color_token("foreground"),
        BadgeVariant::Ghost => theme.color_token("foreground"),
        BadgeVariant::Link => theme.color_token("primary"),
    }
}

fn bg_for(theme: &ThemeSnapshot, variant: BadgeVariant) -> Option<Color> {
    match variant {
        BadgeVariant::Default => Some(theme.color_token("primary")),
        BadgeVariant::Secondary => Some(theme.color_token("secondary")),
        BadgeVariant::Destructive => Some(
            theme
                .color_by_key("component.badge.destructive.bg")
                .unwrap_or_else(|| theme.color_token("destructive")),
        ),
        BadgeVariant::Outline => None,
        BadgeVariant::Ghost => None,
        BadgeVariant::Link => None,
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
        None,
        false,
        None,
        None,
        None,
        None,
        Vec::new(),
        ChromeRefinement::default(),
        LayoutRefinement::default(),
    )
}

fn badge_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: impl Into<Arc<str>>,
    variant: BadgeVariant,
    render: Option<BadgeRender>,
    visited: bool,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    children: Vec<AnyElement>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let label = label.into();
    let theme = Theme::global(&*cx.app).snapshot();

    let a11y_label = label.clone();
    let label_for_content = label.clone();

    let pressable_layout =
        decl_style::layout_style(&theme, LayoutRefinement::default().merge(layout_override));

    let mut chrome = ChromeRefinement::default()
        .px(Space::N2)
        .py(Space::N0p5)
        .rounded(Radius::Full)
        .border_1();
    if let Some(bg) = bg_for(&theme, variant) {
        chrome = chrome.bg(ColorRef::Color(bg));
    }
    let chrome = match variant {
        BadgeVariant::Outline => chrome.border_color(ColorRef::Color(border_color(&theme))),
        BadgeVariant::Default
        | BadgeVariant::Secondary
        | BadgeVariant::Destructive
        | BadgeVariant::Ghost
        | BadgeVariant::Link => chrome.border_color(ColorRef::Color(Color::TRANSPARENT)),
    };
    let chrome = chrome.merge(chrome_override);

    let fg = fg_for(&theme, variant);
    let theme_fg = theme.color_token("foreground");
    let theme_muted_fg = theme.color_by_key("muted-foreground").unwrap_or(theme_fg);

    let mut chrome_props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
    chrome_props.layout.size = pressable_layout.size;

    let text_px = theme
        .metric_by_key("component.badge.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.badge.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    let content_children = move |cx: &mut ElementContext<'_, H>| {
        current_color::scope_children(cx, ColorRef::Color(fg), |cx| {
            let label = ui::text(cx, label_for_content.clone())
                .text_size_px(text_px)
                .fixed_line_box_px(line_height)
                .line_box_in_bounds()
                .font_semibold()
                .nowrap()
                .text_color(ColorRef::Color(fg))
                .into_element(cx);

            if children.is_empty() && leading_icon.is_none() && trailing_icon.is_none() {
                return vec![label];
            }

            // Upstream shadcn badge enforces `[&>svg]:size-3` (12px) for direct svg children.
            let icon_px = Px(12.0);
            let mut content = Vec::with_capacity(children.len() + 3);

            if let Some(icon) = leading_icon.clone() {
                content.push(decl_icon::icon_with(
                    cx,
                    icon,
                    Some(icon_px),
                    Some(ColorRef::Color(fg)),
                ));
            }

            let children = children
                .into_iter()
                .map(|child| apply_badge_child_icon_size(child, icon_px))
                .map(|child| apply_badge_inherited_fg(child, fg, theme_fg, theme_muted_fg))
                .collect::<Vec<_>>();
            content.extend(children);
            content.push(label);

            if let Some(icon) = trailing_icon.clone() {
                content.push(decl_icon::icon_with(
                    cx,
                    icon,
                    Some(icon_px),
                    Some(ColorRef::Color(fg)),
                ));
            }

            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .justify_center()
                    .items_center()
                    .gap_x(Space::N1),
                |_cx| content,
            )]
        })
    };

    let focus_radius = {
        let Corners {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        } = chrome_props.corner_radii;
        Px(top_left
            .0
            .max(top_right.0)
            .max(bottom_right.0)
            .max(bottom_left.0))
    };

    let (render_role, render_key_activation, render_on_activate) = match render {
        Some(BadgeRender::Link { href, target, rel }) => (
            Some(SemanticsRole::Link),
            PressableKeyActivation::EnterOnly,
            on_activate.or_else(|| Some(open_url_on_activate(href, target, rel))),
        ),
        None => (None, PressableKeyActivation::EnterAndSpace, on_activate),
    };

    if render_role.is_some() || render_on_activate.is_some() {
        let visited = visited && render_role == Some(SemanticsRole::Link);
        return control_chrome_pressable_with_id_props(cx, move |cx, _st, _id| {
            if let Some(on_activate) = render_on_activate.clone() {
                cx.pressable_on_activate(on_activate);
            }

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: true,
                focusable: true,
                focus_ring: Some(decl_style::focus_ring(&theme, focus_radius)),
                key_activation: render_key_activation,
                a11y: PressableA11y {
                    role: render_role,
                    label: Some(a11y_label.clone()),
                    test_id: test_id.clone(),
                    visited,
                    ..Default::default()
                },
                ..Default::default()
            };

            let chrome_props = chrome_props;
            (pressable_props, chrome_props, content_children)
        });
    }

    let mut root_props = chrome_props;
    root_props.layout = pressable_layout;
    let mut out = cx.container(root_props, content_children);
    if let Some(test_id) = test_id {
        out = out.test_id(test_id);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        )
    }

    fn collect_colors(
        el: &AnyElement,
        out_text: &mut Vec<(Arc<str>, Option<Color>)>,
        out_svg: &mut Vec<Color>,
    ) {
        match &el.kind {
            ElementKind::Text(props) => out_text.push((props.text.clone(), props.color)),
            ElementKind::SvgIcon(SvgIconProps { color, .. }) => out_svg.push(*color),
            _ => {}
        }
        for child in &el.children {
            collect_colors(child, out_text, out_svg);
        }
    }

    #[test]
    fn badge_leading_icon_and_label_follow_variant_fg() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let theme = Theme::global(&*cx.app).snapshot();
            let expected_fg = fg_for(&theme, BadgeVariant::Default);

            let el = Badge::new("Verified")
                .leading_icon(IconId::new_static("lucide.check"))
                .trailing_icon(IconId::new_static("lucide.arrow-right"))
                .into_element(cx);

            let mut texts = Vec::new();
            let mut icons = Vec::new();
            collect_colors(&el, &mut texts, &mut icons);

            assert!(
                texts
                    .iter()
                    .any(|(t, c)| t.as_ref() == "Verified" && *c == Some(expected_fg)),
                "expected badge label to resolve to variant fg"
            );
            assert!(
                icons.len() >= 2 && icons.iter().all(|c| *c == expected_fg),
                "expected badge icon(s) to resolve to variant fg"
            );
        });
    }
}
