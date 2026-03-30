use std::any::Any;
use std::sync::Arc;

use fret_core::{
    AttributedText, Color, Corners, DecorationLineStyle, FontId, FontWeight, Point, Px,
    SemanticsRole, TextFontAxisSetting, TextFontFeatureSetting, TextPaintStyle, TextSpan,
    Transform2D, UnderlineStyle,
};
use fret_icons::IconId;
use fret_runtime::{ActionId, Effect};
use fret_ui::ThemeNamedColorKey;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ElementKind, LayoutStyle, Length, PressableA11y, PressableKeyActivation,
    PressableProps, SpinnerProps, SvgIconProps,
};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::motion::{
    drive_tween_color_for_element, drive_tween_f32_for_element,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, Radius, Space, ui};

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

/// Upstream shadcn/ui `badgeVariants(...)` compat surface.
///
/// Upstream returns a Tailwind/CVA class string. In Fret we expose the closest equivalent as
/// mergeable refinements.
#[derive(Debug, Clone)]
pub struct BadgeVariants {
    pub chrome: ChromeRefinement,
    pub layout: LayoutRefinement,
}

pub fn badge_variants(theme: &ThemeSnapshot, variant: BadgeVariant) -> BadgeVariants {
    let mut chrome = ChromeRefinement::default()
        .px(Space::N2)
        .py(Space::N0p5)
        .rounded(Radius::Full)
        .border_1();
    if let Some(bg) = bg_for(theme, variant) {
        chrome = chrome.bg(ColorRef::Color(bg));
    }
    chrome = match variant {
        BadgeVariant::Outline => chrome.border_color(ColorRef::Color(border_color(theme))),
        BadgeVariant::Default
        | BadgeVariant::Secondary
        | BadgeVariant::Destructive
        | BadgeVariant::Ghost
        | BadgeVariant::Link => chrome.border_color(ColorRef::Color(Color::TRANSPARENT)),
    };
    chrome = chrome.text_color(ColorRef::Color(fg_for(theme, variant)));

    let layout = LayoutRefinement::default().flex_shrink_0();
    BadgeVariants { chrome, layout }
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

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

pub struct Badge {
    label: Arc<str>,
    variant: BadgeVariant,
    render: Option<BadgeRender>,
    visited: bool,
    aria_invalid: bool,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    label_font_override: Option<FontId>,
    label_weight_override: Option<FontWeight>,
    label_features_override: Vec<TextFontFeatureSetting>,
    label_axes_override: Vec<TextFontAxisSetting>,
    children: Vec<AnyElement>,
    trailing_children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Badge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Badge")
            .field("label", &self.label.as_ref())
            .field("variant", &self.variant)
            .field("render", &self.render)
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
            .field("on_activate", &self.on_activate.is_some())
            .field("aria_invalid", &self.aria_invalid)
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
            aria_invalid: false,
            action: None,
            action_payload: None,
            on_activate: None,
            test_id: None,
            leading_icon: None,
            trailing_icon: None,
            label_font_override: None,
            label_weight_override: None,
            label_features_override: Vec::new(),
            label_axes_override: Vec::new(),
            children: Vec::new(),
            trailing_children: Vec::new(),
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

    pub fn label_font(mut self, font: FontId) -> Self {
        self.label_font_override = Some(font);
        self
    }

    pub fn label_font_monospace(self) -> Self {
        self.label_font(FontId::monospace())
    }

    pub fn label_tabular_nums(mut self) -> Self {
        self.label_features_override.push(TextFontFeatureSetting {
            tag: "tnum".into(),
            value: 1,
        });
        self
    }

    /// Adds extra inline content rendered before the badge label.
    ///
    /// This is the curated Fret mapping for upstream child compositions such as
    /// `data-icon="inline-start"`.
    pub fn leading_children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Backwards-compatible alias for [`Badge::leading_children`].
    pub fn children(self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.leading_children(children)
    }

    /// Adds extra inline content rendered after the badge label.
    ///
    /// This is useful for matching shadcn patterns like `Spinner data-icon="inline-end"` where the
    /// trailing affordance is not a static icon.
    pub fn trailing_children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.trailing_children = children.into_iter().collect();
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

    /// Apply the upstream `aria-invalid` error state chrome (focus ring color + border color).
    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    /// Overrides the badge label's font weight.
    ///
    /// Upstream shadcn badge defaults to `font-semibold`. Most Fret recipes map that to
    /// `FontWeight::MEDIUM`, but some consumers (e.g. AI Elements chain-of-thought search
    /// results) use `font-normal`.
    pub fn label_weight(mut self, weight: FontWeight) -> Self {
        self.label_weight_override = Some(weight);
        self
    }

    /// Bind a stable action ID to this badge (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized badge actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`Badge::action_payload`], but computes the payload lazily on activation.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
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
            self.aria_invalid,
            self.render,
            self.visited,
            self.action,
            self.action_payload,
            self.on_activate,
            self.test_id,
            self.leading_icon,
            self.trailing_icon,
            self.label_font_override,
            self.label_weight_override,
            self.label_features_override,
            self.label_axes_override,
            self.children,
            self.trailing_children,
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

fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha.clamp(0.0, 1.0);
    color
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

fn underline_rich_text(rich: AttributedText) -> AttributedText {
    let spans = rich
        .spans
        .iter()
        .map(|span| {
            let mut span = span.clone();
            if span.paint.underline.is_none() {
                span.paint.underline = Some(UnderlineStyle {
                    color: None,
                    style: DecorationLineStyle::Solid,
                });
            }
            span
        })
        .collect::<Vec<_>>();
    AttributedText::new(rich.text, Arc::from(spans.into_boxed_slice()))
}

fn apply_badge_hover_underline(mut element: AnyElement) -> AnyElement {
    element.children = element
        .children
        .into_iter()
        .map(apply_badge_hover_underline)
        .collect();

    match element.kind {
        ElementKind::Text(props) => {
            let text = props.text.clone();
            let mut span = TextSpan::new(text.as_ref().len());
            span.paint = TextPaintStyle {
                underline: Some(UnderlineStyle {
                    color: None,
                    style: DecorationLineStyle::Solid,
                }),
                ..Default::default()
            };
            let rich = AttributedText::new(text, Arc::from(vec![span].into_boxed_slice()));

            let mut styled = fret_ui::element::StyledTextProps::new(rich);
            styled.layout = props.layout;
            styled.style = props.style;
            styled.color = props.color;
            styled.wrap = props.wrap;
            styled.overflow = props.overflow;
            styled.align = props.align;
            styled.ink_overflow = props.ink_overflow;
            AnyElement::new(
                element.id,
                ElementKind::StyledText(styled),
                element.children,
            )
        }
        ElementKind::StyledText(mut props) => {
            props.rich = underline_rich_text(props.rich);
            AnyElement::new(element.id, ElementKind::StyledText(props), element.children)
        }
        ElementKind::SelectableText(mut props) => {
            props.rich = underline_rich_text(props.rich);
            AnyElement::new(
                element.id,
                ElementKind::SelectableText(props),
                element.children,
            )
        }
        kind => AnyElement::new(element.id, kind, element.children),
    }
}

fn link_hover_bg_for(theme: &ThemeSnapshot, variant: BadgeVariant) -> Option<Color> {
    match variant {
        BadgeVariant::Default | BadgeVariant::Secondary | BadgeVariant::Destructive => {
            bg_for(theme, variant).map(|bg| with_alpha(bg, 0.9))
        }
        BadgeVariant::Outline | BadgeVariant::Ghost => Some(theme.color_token("accent")),
        BadgeVariant::Link => None,
    }
}

fn link_hover_fg_for(theme: &ThemeSnapshot, variant: BadgeVariant, base_fg: Color) -> Color {
    match variant {
        BadgeVariant::Outline | BadgeVariant::Ghost => theme.color_token("accent-foreground"),
        BadgeVariant::Default
        | BadgeVariant::Secondary
        | BadgeVariant::Destructive
        | BadgeVariant::Link => base_fg,
    }
}

pub fn badge<H: UiHost, T>(label: T, variant: BadgeVariant) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    Badge::new(label).variant(variant)
}

fn badge_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: impl Into<Arc<str>>,
    variant: BadgeVariant,
    aria_invalid: bool,
    render: Option<BadgeRender>,
    visited: bool,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    label_font_override: Option<FontId>,
    label_weight_override: Option<FontWeight>,
    label_features_override: Vec<TextFontFeatureSetting>,
    label_axes_override: Vec<TextFontAxisSetting>,
    children: Vec<AnyElement>,
    trailing_children: Vec<AnyElement>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let label = label.into();
    let theme = Theme::global(&*cx.app).snapshot();

    let a11y_label = label.clone();
    let label_for_content = label.clone();

    // Upstream shadcn badge:
    // - uses `inline-flex ... shrink-0 w-fit whitespace-nowrap overflow-hidden`
    // - relies on `shrink-0` so badges don't collapse inside constrained flex rows.
    let pressable_layout = decl_style::layout_style(
        &theme,
        badge_variants(&theme, variant)
            .layout
            .merge(layout_override),
    );

    let mut chrome = badge_variants(&theme, variant).chrome;
    chrome = chrome.merge(chrome_override);

    let fg_ref = chrome
        .text_color
        .clone()
        .unwrap_or_else(|| ColorRef::Color(fg_for(&theme, variant)));
    let base_fg = fg_ref.resolve(&theme);
    let theme_fg = theme.color_token("foreground");
    let theme_muted_fg = theme.color_by_key("muted-foreground").unwrap_or(theme_fg);

    let mut chrome_props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
    chrome_props.layout.size = pressable_layout.size;
    chrome_props.layout.overflow = fret_ui::element::Overflow::Clip;
    let base_bg = chrome_props.background;

    let text_px = theme
        .metric_by_key("component.badge.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.badge.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    let build_content_children = move |cx: &mut ElementContext<'_, H>,
                                       resolved_fg_ref: ColorRef,
                                       resolved_fg: Color,
                                       hover_underline: bool| {
        current_color::scope_children(cx, resolved_fg_ref.clone(), |cx| {
            let mut label = ui::text(label_for_content.clone())
                .text_size_px(text_px)
                .fixed_line_box_px(line_height)
                .line_box_in_bounds()
                .font_weight(label_weight_override.unwrap_or(FontWeight::MEDIUM))
                .nowrap()
                .text_color(resolved_fg_ref.clone());

            if let Some(font) = label_font_override {
                label = label.font(font);
            }
            for feature in &label_features_override {
                label = label.font_feature(feature.tag.to_string(), feature.value);
            }
            for axis in &label_axes_override {
                label = label.font_axis(axis.tag.to_string(), axis.value);
            }

            let mut label = label.into_element(cx);
            if hover_underline {
                label = apply_badge_hover_underline(label);
            }

            // Upstream shadcn badge enforces `[&>svg]:size-3` (12px) for direct svg children.
            let icon_px = Px(12.0);
            let mut content = Vec::with_capacity(children.len() + 3);

            if let Some(icon) = leading_icon.clone() {
                content.push(decl_icon::icon_with(cx, icon, Some(icon_px), None));
            }

            let children = children
                .into_iter()
                .map(|child| apply_badge_child_icon_size(child, icon_px))
                .map(|child| apply_badge_inherited_fg(child, resolved_fg, theme_fg, theme_muted_fg))
                .map(|child| {
                    if hover_underline {
                        apply_badge_hover_underline(child)
                    } else {
                        child
                    }
                })
                .collect::<Vec<_>>();
            content.extend(children);
            content.push(label);

            let trailing_children = trailing_children
                .into_iter()
                .map(|child| apply_badge_child_icon_size(child, icon_px))
                .map(|child| apply_badge_inherited_fg(child, resolved_fg, theme_fg, theme_muted_fg))
                .map(|child| {
                    if hover_underline {
                        apply_badge_hover_underline(child)
                    } else {
                        child
                    }
                })
                .collect::<Vec<_>>();
            content.extend(trailing_children);

            if let Some(icon) = trailing_icon.clone() {
                content.push(decl_icon::icon_with(cx, icon, Some(icon_px), None));
            }

            vec![
                ui::h_row(|_cx| content)
                    .justify_center()
                    .items_center()
                    .gap(Space::N1)
                    .into_element(cx),
            ]
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

    let should_fallback_open_url = action.is_none() && on_activate.is_none();
    let (render_role, render_key_activation, render_on_activate) = match render {
        Some(BadgeRender::Link { href, target, rel }) => (
            Some(SemanticsRole::Link),
            PressableKeyActivation::EnterOnly,
            on_activate.or_else(|| {
                should_fallback_open_url.then(|| open_url_on_activate(href, target, rel))
            }),
        ),
        None => (None, PressableKeyActivation::EnterAndSpace, on_activate),
    };

    if render_role.is_some() || render_on_activate.is_some() || action.is_some() {
        let visited = visited && render_role == Some(SemanticsRole::Link);
        return control_chrome_pressable_with_id_props(cx, move |cx, st, id| {
            if let Some(payload) = action_payload.clone() {
                cx.pressable_dispatch_command_with_payload_factory_if_enabled_opt(
                    action.clone(),
                    payload,
                );
            } else {
                cx.pressable_dispatch_action_if_enabled_opt(action.clone());
            }
            if let Some(on_activate) = render_on_activate.clone() {
                cx.pressable_on_activate(on_activate);
            }

            let focus_visible =
                st.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
            let duration = crate::overlay_motion::shadcn_motion_duration_150(cx);
            let ease = crate::overlay_motion::shadcn_ease;
            let link_hovered = render_role == Some(SemanticsRole::Link) && st.hovered;

            let ring_color = theme.color_token("ring");
            let destructive = theme.color_token("destructive");

            let border_base = chrome_props.border_color.unwrap_or(Color::TRANSPARENT);
            let border_target = if aria_invalid {
                destructive
            } else if focus_visible {
                ring_color
            } else {
                border_base
            };

            let border_motion = drive_tween_color_for_element(
                cx,
                id,
                "badge-border-color",
                border_target,
                duration,
                ease,
            );

            let ring_alpha = drive_tween_f32_for_element(
                cx,
                id,
                "badge-ring-alpha",
                if focus_visible { 1.0 } else { 0.0 },
                duration,
                ease,
            );
            let target_fg = if link_hovered {
                link_hover_fg_for(&theme, variant, base_fg)
            } else {
                base_fg
            };
            let fg_motion = drive_tween_color_for_element(
                cx,
                id,
                "badge-foreground-color",
                target_fg,
                duration,
                ease,
            );
            let target_bg = if link_hovered {
                link_hover_bg_for(&theme, variant).or(base_bg)
            } else {
                base_bg
            }
            .unwrap_or(Color::TRANSPARENT);
            let bg_motion = drive_tween_color_for_element(
                cx,
                id,
                "badge-background-color",
                target_bg,
                duration,
                ease,
            );

            let mut ring = decl_style::focus_ring(&theme, focus_radius);
            if aria_invalid || variant == BadgeVariant::Destructive {
                ring.color = crate::theme_variants::invalid_control_ring_color(&theme, destructive);
            }
            ring.color.a = (ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
            if let Some(offset) = ring.offset_color {
                ring.offset_color = Some(Color {
                    a: (offset.a * ring_alpha.value).clamp(0.0, 1.0),
                    ..offset
                });
            }

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: true,
                focusable: true,
                focus_ring: Some(ring),
                focus_ring_always_paint: ring_alpha.animating,
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

            let mut chrome_props = chrome_props;
            chrome_props.border_color = Some(border_motion.value);
            chrome_props.background = (base_bg.is_some()
                || (link_hovered && link_hover_bg_for(&theme, variant).is_some())
                || bg_motion.animating)
                .then_some(bg_motion.value);
            let resolved_fg_ref = ColorRef::Color(fg_motion.value);
            let hover_underline = link_hovered && variant == BadgeVariant::Link;
            let content_children = move |cx: &mut ElementContext<'_, H>| {
                build_content_children(cx, resolved_fg_ref, fg_motion.value, hover_underline)
            };
            (pressable_props, chrome_props, content_children)
        });
    }

    let mut root_props = chrome_props;
    root_props.layout = pressable_layout;
    root_props.layout.overflow = fret_ui::element::Overflow::Clip;
    let mut out = cx.container(root_props, move |cx| {
        build_content_children(cx, fg_ref, base_fg, false)
    });
    if let Some(test_id) = test_id {
        out = out.test_id(test_id);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, FontWeight, Point, Rect, Size};
    fn blend_over(fg: Color, bg: Color) -> Color {
        let a = fg.a.clamp(0.0, 1.0);
        Color {
            r: fg.r * a + bg.r * (1.0 - a),
            g: fg.g * a + bg.g * (1.0 - a),
            b: fg.b * a + bg.b * (1.0 - a),
            a: 1.0,
        }
    }

    fn relative_luminance(c: Color) -> f32 {
        // Theme colors are stored in linear space, so we can use the WCAG coefficients directly.
        (0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b).clamp(0.0, 1.0)
    }

    fn contrast_ratio(a: Color, b: Color) -> f32 {
        let l1 = relative_luminance(a);
        let l2 = relative_luminance(b);
        let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
        (hi + 0.05) / (lo + 0.05)
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        )
    }

    fn collect_colors(
        el: &AnyElement,
        out_text: &mut Vec<(Arc<str>, Option<Color>)>,
        out_svg: &mut Vec<(Color, bool)>,
    ) {
        match &el.kind {
            ElementKind::Text(props) => out_text.push((props.text.clone(), props.color)),
            ElementKind::SvgIcon(SvgIconProps {
                color,
                inherit_color,
                ..
            }) => out_svg.push((*color, *inherit_color)),
            _ => {}
        }
        for child in &el.children {
            collect_colors(child, out_text, out_svg);
        }
    }

    fn find_inherited_foreground(el: &AnyElement) -> Option<Color> {
        if let Some(fg) = el.inherited_foreground {
            return Some(fg);
        }
        if let ElementKind::ForegroundScope(props) = &el.kind
            && let Some(fg) = props.foreground
        {
            return Some(fg);
        }
        for child in &el.children {
            if let Some(found) = find_inherited_foreground(child) {
                return Some(found);
            }
        }
        None
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

            let scope_fg = find_inherited_foreground(&el)
                .expect("expected badge root to carry inherited foreground");
            assert_eq!(
                scope_fg, expected_fg,
                "expected badge currentColor scope to resolve to variant fg"
            );

            assert!(
                texts
                    .iter()
                    .any(|(t, c)| t.as_ref() == "Verified" && *c == Some(expected_fg)),
                "expected badge label to resolve to variant fg"
            );
            assert!(
                icons.len() >= 2 && icons.iter().all(|(_, inherit)| *inherit),
                "expected badge icon(s) to inherit currentColor via inherited foreground"
            );
        });
    }

    fn find_text<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a fret_ui::element::TextProps> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => return Some(props),
            _ => {}
        }
        for child in &el.children {
            if let Some(found) = find_text(child, needle) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn badge_defaults_to_font_medium_and_shrink_0() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let el = Badge::new("Draft").into_element(cx);
            let ElementKind::Container(root) = &el.kind else {
                panic!("expected Badge root to be a Container, got {:?}", el.kind);
            };

            assert_eq!(
                root.layout.flex.shrink, 0.0,
                "expected shadcn Badge to default to shrink-0"
            );
            assert_eq!(
                root.layout.overflow,
                fret_ui::element::Overflow::Clip,
                "expected shadcn Badge to default to overflow-hidden (clip)"
            );

            let label = find_text(&el, "Draft").expect("badge label text element");
            let style = label.style.as_ref().expect("badge label has a text style");
            assert_eq!(
                style.weight,
                FontWeight::MEDIUM,
                "expected shadcn Badge label to use font-medium"
            );
        });
    }

    #[test]
    fn badge_leading_and_trailing_children_preserve_inline_order() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let el = Badge::new("Core")
                .leading_children([cx.text("Start")])
                .trailing_children([cx.text("End")])
                .into_element(cx);

            let mut texts = Vec::new();
            let mut icons = Vec::new();
            collect_colors(&el, &mut texts, &mut icons);

            let ordered = texts
                .into_iter()
                .map(|(text, _)| text.to_string())
                .collect::<Vec<_>>();
            assert_eq!(ordered, vec!["Start", "Core", "End"]);
            assert!(
                icons.is_empty(),
                "expected inline-order test to stay text-only"
            );
        });
    }

    #[test]
    fn destructive_badge_text_contrast_is_reasonable_in_zinc_dark() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Zinc,
            crate::shadcn_themes::ShadcnColorScheme::Dark,
        );

        let theme = Theme::global(&app);
        let snap = theme.snapshot();

        let fg = fg_for(&snap, BadgeVariant::Destructive);
        let bg = bg_for(&snap, BadgeVariant::Destructive).expect("missing destructive badge bg");
        let surface = snap.color_token("background");
        let bg_composited = blend_over(bg, surface);

        let ratio = contrast_ratio(fg, bg_composited);
        assert!(
            ratio >= 4.5,
            "expected destructive badge contrast >= 4.5, got {ratio:.2} (fg={:?} bg={:?} bg_composited={:?} surface={:?})",
            fg,
            bg,
            bg_composited,
            surface,
        );
    }

    #[derive(Default)]
    struct FakeServices;

    impl fret_core::TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn find_pressable_with_test_id<'a>(
        node: &'a AnyElement,
        test_id: &str,
    ) -> Option<&'a PressableProps> {
        match &node.kind {
            ElementKind::Pressable(props) => props
                .a11y
                .test_id
                .as_deref()
                .is_some_and(|id| id == test_id)
                .then_some(props),
            _ => node
                .children
                .iter()
                .find_map(|c| find_pressable_with_test_id(c, test_id)),
        }
    }

    fn color_eq_eps(a: Color, b: Color, eps: f32) -> bool {
        (a.r - b.r).abs() <= eps
            && (a.g - b.g).abs() <= eps
            && (a.b - b.b).abs() <= eps
            && (a.a - b.a).abs() <= eps
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct BadgeVisualState {
        background: Option<Color>,
        label_color: Option<Color>,
        label_underlined: bool,
    }

    fn find_label_visual(el: &AnyElement, needle: &str) -> Option<BadgeVisualState> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => {
                return Some(BadgeVisualState {
                    label_color: props.color,
                    label_underlined: false,
                    ..Default::default()
                });
            }
            ElementKind::StyledText(props) if props.rich.text.as_ref() == needle => {
                return Some(BadgeVisualState {
                    label_color: props.color,
                    label_underlined: props
                        .rich
                        .spans
                        .iter()
                        .any(|span| span.paint.underline.is_some()),
                    ..Default::default()
                });
            }
            ElementKind::SelectableText(props) if props.rich.text.as_ref() == needle => {
                return Some(BadgeVisualState {
                    label_color: props.color,
                    label_underlined: props
                        .rich
                        .spans
                        .iter()
                        .any(|span| span.paint.underline.is_some()),
                    ..Default::default()
                });
            }
            _ => {}
        }

        for child in &el.children {
            if let Some(found) = find_label_visual(child, needle) {
                return Some(found);
            }
        }
        None
    }

    fn capture_badge_visual_state(el: &AnyElement, label: &str) -> BadgeVisualState {
        let chrome = el
            .children
            .first()
            .expect("expected badge pressable to contain chrome");
        let ElementKind::Container(props) = &chrome.kind else {
            panic!("expected badge chrome container");
        };

        let mut state = find_label_visual(el, label).expect("badge label visual");
        state.background = props.background;
        state
    }

    #[test]
    fn badge_focus_ring_tweens_in_and_out_like_a_transition() {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        use fret_core::{Event, FrameId, KeyCode, Modifiers};
        use fret_ui::tree::UiTree;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(360.0), Px(160.0)),
        );
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;

        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) -> fret_core::NodeId {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "badge-focus-ring-tween",
                move |cx| {
                    let el = Badge::new("Draft")
                        .render(BadgeRender::Link {
                            href: Arc::from("https://example.com"),
                            target: None,
                            rel: None,
                        })
                        .test_id("badge")
                        .into_element(cx);

                    let badge = find_pressable_with_test_id(&el, "badge").expect("badge pressable");
                    let a = badge.focus_ring.map(|ring| ring.color.a).unwrap_or(0.0);
                    ring_alpha_out.set(Some(a));
                    always_paint_out.set(Some(badge.focus_ring_always_paint));

                    vec![el]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            root
        }

        // Frame 1: baseline render (no focus-visible), ring alpha should be 0.
        app.set_frame_id(FrameId(1));
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a0 = ring_alpha_out.get().expect("a0");
        assert!(
            a0.abs() <= 1e-6,
            "expected ring alpha to start at 0, got {a0}"
        );

        // Focus the badge and mark focus-visible via a navigation key.
        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable badge");
        ui.set_focus(Some(focusable));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        // Frame 2: ring should be in-between (not snapped).
        app.set_frame_id(FrameId(2));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a1 = ring_alpha_out.get().expect("a1");
        assert!(
            a1 > 0.0,
            "expected ring alpha to start animating in, got {a1}"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_focused = ring_alpha_out.get().expect("a_focused");
        assert!(
            a_focused > a1 + 1e-4,
            "expected ring alpha to increase over time, got a1={a1} a_focused={a_focused}"
        );

        // Blur and ensure ring animates out while still being painted.
        ui.set_focus(None);
        app.set_frame_id(FrameId(1000));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a_blur = ring_alpha_out.get().expect("a_blur");
        let always_paint = always_paint_out.get().expect("always_paint");
        assert!(
            a_blur > 0.0 && a_blur < a_focused,
            "expected ring alpha to be intermediate after blur, got a_blur={a_blur} a_focused={a_focused}"
        );
        assert!(
            always_paint,
            "expected focus ring to request painting while animating out"
        );

        for i in 0..settle {
            app.set_frame_id(FrameId(1001 + i));
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_final = ring_alpha_out.get().expect("a_final");
        let always_paint_final = always_paint_out.get().expect("always_paint_final");
        assert!(
            a_final.abs() <= 1e-4,
            "expected ring alpha to settle at 0, got {a_final}"
        );
        assert!(
            !always_paint_final,
            "expected focus ring to stop requesting painting after settling"
        );
    }

    #[test]
    fn outline_link_badge_hover_uses_accent_chrome_and_foreground() {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        use fret_core::{FrameId, MouseButtons, PointerType};
        use fret_ui::elements::GlobalElementId;
        use fret_ui::tree::UiTree;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(120.0)),
        );
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let badge_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let state_out: Rc<Cell<Option<BadgeVisualState>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            badge_id: Rc<Cell<Option<GlobalElementId>>>,
            state_out: Rc<Cell<Option<BadgeVisualState>>>,
        ) {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "badge-outline-link-hover",
                move |cx| {
                    let el = Badge::new("Outline")
                        .variant(BadgeVariant::Outline)
                        .render(BadgeRender::Link {
                            href: Arc::from("https://example.com"),
                            target: None,
                            rel: None,
                        })
                        .test_id("badge-outline-link-hover")
                        .into_element(cx);
                    badge_id.set(Some(el.id));
                    state_out.set(Some(capture_badge_visual_state(&el, "Outline")));
                    vec![el]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        }

        let theme = Theme::global(&app).snapshot();
        let expected_fg = theme.color_token("accent-foreground");
        let expected_bg = theme.color_token("accent");
        let base_fg = theme.color_token("foreground");

        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            badge_id.clone(),
            state_out.clone(),
        );

        let baseline = state_out.get().expect("baseline outline link badge state");
        assert!(
            color_eq_eps(
                baseline.background.unwrap_or(Color::TRANSPARENT),
                Color::TRANSPARENT,
                1e-6,
            ),
            "expected idle outline link badge to have a transparent background, got {:?}",
            baseline.background
        );
        assert!(
            color_eq_eps(
                baseline.label_color.expect("outline link label color"),
                base_fg,
                1e-6,
            ),
            "expected idle outline link badge fg to match theme foreground"
        );

        let id = badge_id.get().expect("outline link badge id");
        let node =
            fret_ui::elements::node_for_element(&mut app, window, id).expect("outline link node");
        let rect = ui
            .debug_node_bounds(node)
            .expect("outline link badge bounds");
        let center = Point::new(
            Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
            Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(2 + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                badge_id.clone(),
                state_out.clone(),
            );
        }

        let hovered = state_out.get().expect("hovered outline link badge state");
        assert!(
            color_eq_eps(
                hovered.background.expect("hovered outline link background"),
                expected_bg,
                1e-4,
            ),
            "expected hovered outline link badge bg to match accent"
        );
        assert!(
            color_eq_eps(
                hovered
                    .label_color
                    .expect("hovered outline link label color"),
                expected_fg,
                1e-4,
            ),
            "expected hovered outline link badge fg to match accent-foreground"
        );
        assert!(
            !hovered.label_underlined,
            "expected outline variant hover to recolor, not underline"
        );
    }

    #[test]
    fn link_badge_hover_underlines_label_when_rendered_as_link() {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        use fret_core::{FrameId, MouseButtons, PointerType};
        use fret_ui::elements::GlobalElementId;
        use fret_ui::tree::UiTree;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(280.0), Px(120.0)),
        );
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let badge_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let state_out: Rc<Cell<Option<BadgeVisualState>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            badge_id: Rc<Cell<Option<GlobalElementId>>>,
            state_out: Rc<Cell<Option<BadgeVisualState>>>,
        ) {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "badge-link-hover-underline",
                move |cx| {
                    let el = Badge::new("Open Link")
                        .variant(BadgeVariant::Link)
                        .render(BadgeRender::Link {
                            href: Arc::from("https://example.com"),
                            target: None,
                            rel: None,
                        })
                        .test_id("badge-link-hover-underline")
                        .into_element(cx);
                    badge_id.set(Some(el.id));
                    state_out.set(Some(capture_badge_visual_state(&el, "Open Link")));
                    vec![el]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        }

        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            badge_id.clone(),
            state_out.clone(),
        );

        let baseline = state_out.get().expect("baseline link badge state");
        assert!(
            !baseline.label_underlined,
            "expected idle link badge label to start without underline"
        );

        let id = badge_id.get().expect("link badge id");
        let node = fret_ui::elements::node_for_element(&mut app, window, id).expect("link node");
        let rect = ui.debug_node_bounds(node).expect("link badge bounds");
        let center = Point::new(
            Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
            Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(2 + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                badge_id.clone(),
                state_out.clone(),
            );
        }

        let hovered = state_out.get().expect("hovered link badge state");
        assert!(
            hovered.label_underlined,
            "expected link badge hover to underline the label"
        );
        assert!(
            color_eq_eps(
                hovered.background.unwrap_or(Color::TRANSPARENT),
                Color::TRANSPARENT,
                1e-6,
            ),
            "expected link badge hover to keep a transparent background"
        );
    }
}
