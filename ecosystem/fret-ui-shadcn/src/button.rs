use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextFontAxisSetting,
    TextFontFeatureSetting, TextStyle,
};
use fret_icons::IconId;
use fret_runtime::{CommandId, Effect};
use fret_ui::action::{OnActivate, OnHoverChange};
use fret_ui::element::{AnyElement, PressableA11y, PressableKeyActivation, PressableProps};
use fret_ui::{ElementContext, Theme, ThemeNamedColorKey, ThemeSnapshot, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, OverrideSlot, Radius,
    ShadowPreset, Size as ComponentSize, Space, WidgetStateProperty, WidgetStates,
    resolve_override_slot, ui,
};

#[derive(Debug, Clone, Default)]
pub struct ButtonStyle {
    pub background: OverrideSlot<ColorRef>,
    pub foreground: OverrideSlot<ColorRef>,
    pub border_color: OverrideSlot<ColorRef>,
}

impl ButtonStyle {
    pub fn background(mut self, background: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.background = Some(background);
        self
    }

    pub fn foreground(mut self, foreground: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.foreground = Some(foreground);
        self
    }

    pub fn border_color(mut self, border_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.background.is_some() {
            self.background = other.background;
        }
        if other.foreground.is_some() {
            self.foreground = other.foreground;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        self
    }
}

#[derive(Debug, Clone)]
pub enum ButtonRender {
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

fn contains_svg_icon_like(el: &AnyElement) -> bool {
    match &el.kind {
        fret_ui::element::ElementKind::SvgIcon(_) | fret_ui::element::ElementKind::Spinner(_) => {
            return true;
        }
        _ => {}
    }

    el.children.iter().any(contains_svg_icon_like)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    /// Upstream shadcn/ui v4: `size="xs"` (`h-6`, `text-xs`).
    Xs,
    Sm,
    Lg,
    Icon,
    /// Upstream shadcn/ui v4: `size="icon-xs"` (`size-6`).
    IconXs,
    IconSm,
    IconLg,
}

impl ButtonSize {
    fn component_size(self) -> ComponentSize {
        match self {
            Self::Default => ComponentSize::Medium,
            Self::Xs => ComponentSize::XSmall,
            Self::Sm => ComponentSize::Small,
            Self::Lg => ComponentSize::Large,
            Self::Icon => ComponentSize::Medium,
            Self::IconXs => ComponentSize::XSmall,
            Self::IconSm => ComponentSize::Small,
            Self::IconLg => ComponentSize::Large,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ButtonVariantStyle {
    pub background: WidgetStateProperty<ColorRef>,
    pub border_color: WidgetStateProperty<ColorRef>,
    pub foreground: WidgetStateProperty<ColorRef>,
}

fn token(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

pub(crate) fn variant_style(variant: ButtonVariant) -> ButtonVariantStyle {
    let transparent = ColorRef::Color(Color::TRANSPARENT);

    match variant {
        ButtonVariant::Default => ButtonVariantStyle {
            background: WidgetStateProperty::new(token("primary", ColorFallback::ThemeAccent))
                .when(
                    WidgetStates::HOVERED,
                    token(
                        "primary.hover.background",
                        ColorFallback::ThemeTokenAlphaMul {
                            key: "primary",
                            mul: 0.9,
                        },
                    ),
                )
                .when(
                    WidgetStates::ACTIVE,
                    token(
                        "primary.active.background",
                        ColorFallback::ThemeTokenAlphaMul {
                            key: "primary",
                            mul: 0.9,
                        },
                    ),
                ),
            border_color: WidgetStateProperty::new(transparent.clone()),
            foreground: WidgetStateProperty::new(token(
                "primary-foreground",
                ColorFallback::ThemeTextPrimary,
            )),
        },
        ButtonVariant::Destructive => ButtonVariantStyle {
            background: WidgetStateProperty::new(token(
                "component.button.destructive.bg",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "destructive",
                    mul: 1.0,
                },
            ))
            .when(
                WidgetStates::HOVERED,
                token(
                    "destructive.hover.background",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "destructive",
                        mul: 0.9,
                    },
                ),
            )
            .when(
                WidgetStates::ACTIVE,
                token(
                    "destructive.active.background",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "destructive",
                        mul: 0.9,
                    },
                ),
            ),
            border_color: WidgetStateProperty::new(transparent.clone()),
            // Upstream shadcn button uses `text-white` for destructive.
            foreground: WidgetStateProperty::new(ColorRef::Named(ThemeNamedColorKey::White)),
        },
        ButtonVariant::Secondary => ButtonVariantStyle {
            background: WidgetStateProperty::new(token(
                "secondary",
                ColorFallback::ThemePanelBackground,
            ))
            .when(
                WidgetStates::HOVERED,
                token(
                    "secondary.hover.background",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "secondary",
                        mul: 0.8,
                    },
                ),
            )
            .when(
                WidgetStates::ACTIVE,
                token(
                    "secondary.active.background",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "secondary",
                        mul: 0.8,
                    },
                ),
            ),
            border_color: WidgetStateProperty::new(transparent.clone()),
            foreground: WidgetStateProperty::new(token(
                "secondary-foreground",
                ColorFallback::ThemeTextPrimary,
            )),
        },
        ButtonVariant::Outline => ButtonVariantStyle {
            background: WidgetStateProperty::new(token(
                "component.button.outline.bg",
                ColorFallback::ThemeSurfaceBackground,
            ))
            .when(
                WidgetStates::HOVERED,
                token(
                    "component.button.outline.bg_hover",
                    ColorFallback::ThemeHoverBackground,
                ),
            )
            .when(
                WidgetStates::ACTIVE,
                token(
                    "component.button.outline.bg_hover",
                    ColorFallback::ThemeHoverBackground,
                ),
            ),
            border_color: WidgetStateProperty::new(token(
                "component.button.outline.border",
                ColorFallback::ThemePanelBorder,
            ))
            .when(
                WidgetStates::FOCUS_VISIBLE,
                token("ring", ColorFallback::ThemeFocusRing),
            ),
            foreground: WidgetStateProperty::new(token(
                "foreground",
                ColorFallback::ThemeTextPrimary,
            ))
            .when(
                WidgetStates::HOVERED,
                token("accent-foreground", ColorFallback::ThemeTextPrimary),
            )
            .when(
                WidgetStates::ACTIVE,
                token("accent-foreground", ColorFallback::ThemeTextPrimary),
            ),
        },
        ButtonVariant::Ghost => ButtonVariantStyle {
            background: WidgetStateProperty::new(transparent.clone())
                .when(
                    WidgetStates::HOVERED,
                    token("accent", ColorFallback::ThemeHoverBackground),
                )
                .when(
                    WidgetStates::ACTIVE,
                    token("accent", ColorFallback::ThemeHoverBackground),
                ),
            border_color: WidgetStateProperty::new(transparent.clone()),
            foreground: WidgetStateProperty::new(token(
                "foreground",
                ColorFallback::ThemeTextPrimary,
            )),
        },
        ButtonVariant::Link => ButtonVariantStyle {
            background: WidgetStateProperty::new(transparent.clone()),
            border_color: WidgetStateProperty::new(transparent.clone()),
            foreground: WidgetStateProperty::new(token("primary", ColorFallback::ThemeAccent)),
        },
    }
}

/// Upstream shadcn/ui `buttonVariants(...)` compat surface.
///
/// Upstream returns a Tailwind/CVA class string that can be applied to non-button nodes. In Fret we
/// expose the closest equivalent as mergeable refinements.
#[derive(Debug, Clone)]
pub struct ButtonVariants {
    pub chrome: ChromeRefinement,
    pub layout: LayoutRefinement,
}

fn button_variant_size_key(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Xs | ButtonSize::IconXs => "xs",
        ButtonSize::Sm | ButtonSize::IconSm => "sm",
        ButtonSize::Default | ButtonSize::Icon => "md",
        ButtonSize::Lg | ButtonSize::IconLg => "lg",
    }
}

pub fn button_variants(
    theme: &ThemeSnapshot,
    variant: ButtonVariant,
    size: ButtonSize,
) -> ButtonVariants {
    let style = variant_style(variant);
    let bg = style.background.resolve(WidgetStates::empty());
    let fg = style.foreground.resolve(WidgetStates::empty());
    let border = style.border_color.resolve(WidgetStates::empty());

    let chrome = ChromeRefinement::default()
        .rounded(Radius::Md)
        .bg(bg.clone())
        .text_color(fg.clone())
        .border_color(border.clone())
        .merge(if variant == ButtonVariant::Outline {
            ChromeRefinement::default().border_1()
        } else {
            ChromeRefinement::default()
        });

    let size_key = button_variant_size_key(size);
    let button_h = theme
        .metric_by_key(&format!("component.size.{size_key}.button.h"))
        .unwrap_or(Px(32.0));
    let icon = theme
        .metric_by_key(&format!("component.size.{size_key}.icon_button.size"))
        .unwrap_or(button_h);

    let mut layout = LayoutRefinement::default().flex_shrink_0();
    if matches!(
        size,
        ButtonSize::Icon | ButtonSize::IconXs | ButtonSize::IconSm | ButtonSize::IconLg
    ) {
        layout = layout.w_px(icon).h_px(icon).min_w(icon).min_h(icon);
    } else {
        layout = layout.h_px(button_h).min_h(button_h);
    }

    ButtonVariants { chrome, layout }
}

/// Upstream shadcn/ui compat alias for copy/paste parity.
#[allow(non_snake_case)]
pub fn buttonVariants(
    theme: &ThemeSnapshot,
    variant: ButtonVariant,
    size: ButtonSize,
) -> ButtonVariants {
    button_variants(theme, variant, size)
}

pub(crate) fn variant_colors(
    theme: &Theme,
    variant: ButtonVariant,
) -> (Color, Color, Color, Color, Color) {
    let style = variant_style(variant);

    let bg = style
        .background
        .resolve(WidgetStates::empty())
        .resolve(theme);
    let bg_hover = style
        .background
        .resolve(WidgetStates::HOVERED)
        .resolve(theme);
    let bg_active = style
        .background
        .resolve(WidgetStates::ACTIVE)
        .resolve(theme);
    let border = style
        .border_color
        .resolve(WidgetStates::empty())
        .resolve(theme);
    let fg = style
        .foreground
        .resolve(WidgetStates::empty())
        .resolve(theme);
    (bg, bg_hover, bg_active, border, fg)
}

pub(crate) fn button_text_style(theme: &Theme, size: ButtonSize) -> TextStyle {
    let px = size.component_size().control_text_px(theme);
    let line_height = theme.metric_token("font.line_height");

    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::MEDIUM;
    style
}

pub struct Button {
    label: Arc<str>,
    a11y_label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    label_font_override: Option<FontId>,
    label_features_override: Vec<TextFontFeatureSetting>,
    label_axes_override: Vec<TextFontAxisSetting>,
    leading_icon_size: Option<Px>,
    content_justify: Justify,
    text_weight_override: Option<FontWeight>,
    command: Option<CommandId>,
    on_activate: Option<OnActivate>,
    on_hover_change: Option<OnHoverChange>,
    toggle_model: Option<fret_runtime::Model<bool>>,
    disabled: bool,
    focusable: bool,
    test_id: Option<Arc<str>>,
    render: Option<ButtonRender>,
    variant: ButtonVariant,
    size: ButtonSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: ButtonStyle,
    border_override: Option<Edges>,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BorderWidthOverride {
    pub top: Option<Px>,
    pub right: Option<Px>,
    pub bottom: Option<Px>,
    pub left: Option<Px>,
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("label", &self.label)
            .field("a11y_label", &self.a11y_label)
            .field("children_len", &self.children.len())
            .field("leading_icon", &self.leading_icon)
            .field("trailing_icon", &self.trailing_icon)
            .field("command", &self.command)
            .field("on_activate", &self.on_activate.is_some())
            .field("on_hover_change", &self.on_hover_change.is_some())
            .field("toggle_model", &self.toggle_model.is_some())
            .field("disabled", &self.disabled)
            .field("focusable", &self.focusable)
            .field("test_id", &self.test_id)
            .field("render", &self.render)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("style", &self.style)
            .field("border_override", &self.border_override)
            .field("border_width_override", &self.border_width_override)
            .field("corner_radii_override", &self.corner_radii_override)
            .finish()
    }
}

impl Button {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label,
            a11y_label: None,
            children: Vec::new(),
            leading_icon: None,
            trailing_icon: None,
            label_font_override: None,
            label_features_override: Vec::new(),
            label_axes_override: Vec::new(),
            leading_icon_size: None,
            content_justify: Justify::Center,
            text_weight_override: None,
            command: None,
            on_activate: None,
            on_hover_change: None,
            toggle_model: None,
            disabled: false,
            focusable: true,
            test_id: None,
            render: None,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            chrome: ChromeRefinement::default(),
            // Match shadcn/ui `Button` base class `shrink-0`: buttons should not collapse when used
            // inside wrapping control rows.
            layout: fret_ui_kit::LayoutRefinement::default().flex_shrink_0(),
            style: ButtonStyle::default(),
            border_override: None,
            border_width_override: BorderWidthOverride::default(),
            corner_radii_override: None,
        }
    }

    /// Overrides the semantic label (useful for icon-only buttons).
    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn label_font(mut self, font: FontId) -> Self {
        self.label_font_override = Some(font);
        self
    }

    pub fn label_font_monospace(self) -> Self {
        self.label_font(FontId::monospace())
    }

    pub fn label_font_feature(mut self, tag: impl Into<String>, value: u32) -> Self {
        self.label_features_override.push(TextFontFeatureSetting {
            tag: tag.into().into(),
            value,
        });
        self
    }

    pub fn label_font_axis(mut self, tag: impl Into<String>, value: f32) -> Self {
        self.label_axes_override.push(TextFontAxisSetting {
            tag: tag.into().into(),
            value,
        });
        self
    }

    /// Enables OpenType tabular numbers (`font-variant-numeric: tabular-nums`) for the default label text.
    pub fn label_tabular_nums(self) -> Self {
        self.label_font_feature("tnum", 1)
    }

    /// Adds a leading icon rendered under the button's `currentColor` scope.
    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Adds a trailing icon rendered under the button's `currentColor` scope.
    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Shorthand for an icon-only button content slot.
    ///
    /// Note: this does not set `size=Icon*`; callers should still pick an icon size variant.
    pub fn icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self.trailing_icon = None;
        self
    }

    pub fn leading_icon_size(mut self, size: Px) -> Self {
        self.leading_icon_size = Some(size);
        self
    }

    pub fn content_justify(mut self, justify: Justify) -> Self {
        self.content_justify = justify;
        self
    }

    pub fn content_justify_start(self) -> Self {
        self.content_justify(Justify::Start)
    }

    pub fn text_weight(mut self, weight: FontWeight) -> Self {
        self.text_weight_override = Some(weight);
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn on_hover_change(mut self, on_hover_change: OnHoverChange) -> Self {
        self.on_hover_change = Some(on_hover_change);
        self
    }

    pub fn render(mut self, render: ButtonRender) -> Self {
        self.render = Some(render);
        self
    }

    pub fn toggle_model(mut self, model: fret_runtime::Model<bool>) -> Self {
        self.toggle_model = Some(model);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn refine_layout(mut self, layout: fret_ui_kit::LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn border_width_override(mut self, border: BorderWidthOverride) -> Self {
        self.border_width_override = border;
        self
    }

    pub fn border_top_width_override(mut self, border: Px) -> Self {
        self.border_width_override.top = Some(border);
        self
    }

    pub fn border_right_width_override(mut self, border: Px) -> Self {
        self.border_width_override.right = Some(border);
        self
    }

    pub fn border_bottom_width_override(mut self, border: Px) -> Self {
        self.border_width_override.bottom = Some(border);
        self
    }

    pub fn border_left_width_override(mut self, border: Px) -> Self {
        self.border_width_override.left = Some(border);
        self
    }

    /// Overrides per-edge border widths (in px) for this button's chrome.
    ///
    /// This is primarily used by shadcn recipes like `button-group` (`border-l-0`).
    pub fn border_override(mut self, border: Edges) -> Self {
        self.border_override = Some(border);
        self
    }

    /// Overrides per-corner radii (in px) for this button's chrome.
    ///
    /// This is primarily used by shadcn recipes like `button-group` (`rounded-l-none`,
    /// `rounded-r-none`).
    pub fn corner_radii_override(mut self, corners: Corners) -> Self {
        self.corner_radii_override = Some(corners);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();

            let variant_style = variant_style(self.variant);
            let has_outline_shadow = self.variant == ButtonVariant::Outline;

            let size = self.size.component_size();
            // shadcn/ui v4 buttons use `rounded-md` across sizes (including `sm` and `icon`), so
            // we intentionally pin the default radius to `metric.radius.md` rather than scaling
            // with `ComponentSize`.
            let radius = theme.metric_token("metric.radius.md");
            let border_w = if self.variant == ButtonVariant::Outline {
                Px(1.0)
            } else {
                Px(0.0)
            };

            let mut base_layout = self.layout;
            let test_id = self.test_id.clone();
            let is_icon_button = matches!(
                self.size,
                ButtonSize::Icon | ButtonSize::IconXs | ButtonSize::IconSm | ButtonSize::IconLg
            );
            if is_icon_button {
                let icon = size.icon_button_size(Theme::global(&*cx.app));
                let has_explicit_w = base_layout
                    .size
                    .as_ref()
                    .and_then(|s| s.width.as_ref())
                    .is_some();
                let has_explicit_h = base_layout
                    .size
                    .as_ref()
                    .and_then(|s| s.height.as_ref())
                    .is_some();

                // shadcn/ui v4 `size=icon` uses Tailwind `size-*` (a fixed square), not
                // `min-width/min-height`. Using an explicit width/height avoids relying on flexbox
                // min-size behavior and makes icon buttons match web goldens 1:1.
                if !has_explicit_w {
                    base_layout = base_layout.w_px(icon).min_w(icon);
                }
                if !has_explicit_h {
                    base_layout = base_layout.h_px(icon).min_h(icon);
                }
            } else {
                let min_h = size.button_h(Theme::global(&*cx.app));

                // shadcn/ui v4 buttons use Tailwind `h-*` to pin the border-box height across
                // variants (including `outline`). Using `min-height` alone allows chrome padding +
                // border to grow the control, which diverges from web goldens under constrained
                // viewports (available-height clamps depend on the trigger bounds).
                let has_explicit_h = base_layout
                    .size
                    .as_ref()
                    .and_then(|s| s.height.as_ref())
                    .is_some();
                if !has_explicit_h {
                    base_layout = base_layout.h_px(min_h);
                }

                base_layout = base_layout.min_h(min_h);
            }

            let pressable_layout = decl_style::layout_style(&theme, base_layout);

            let command = self.command;
            let on_activate = self.on_activate;
            let on_hover_change = self.on_hover_change;
            let toggle_model = self.toggle_model;
            let should_fallback_open_url = command.is_none() && on_activate.is_none();
            let (render_role, render_key_activation, render_on_activate) = match self.render {
                Some(ButtonRender::Link { href, target, rel }) => (
                    Some(SemanticsRole::Link),
                    PressableKeyActivation::EnterOnly,
                    should_fallback_open_url.then(|| open_url_on_activate(href, target, rel)),
                ),
                None => (None, PressableKeyActivation::EnterAndSpace, None),
            };
            let a11y_label = self.label.clone();
            let a11y_label = self
                .a11y_label
                .clone()
                .unwrap_or_else(|| a11y_label.clone());
            let disabled_explicit = self.disabled;
            let disabled = disabled_explicit
                || command
                    .as_ref()
                    .is_some_and(|cmd| !cx.command_is_enabled(cmd));
            // Upstream: disabled `<button disabled>` is not focusable.
            let focusable = self.focusable && !disabled;
            let user_chrome = self.chrome;
            let user_bg_override = user_chrome.background.is_some();
            let user_border_override = user_chrome.border_color.is_some();
            let style_override = self.style;
            let border_override = self.border_override;
            let border_width_override = self.border_width_override;
            let corner_radii_override = self.corner_radii_override;
            let text_style = button_text_style(Theme::global(&*cx.app), self.size);
            let text_px = text_style.size;
            let text_weight = self.text_weight_override.unwrap_or(text_style.weight);
            let text_line_height = text_style
                .line_height
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            let is_icon = {
                let label_is_empty = self.label.is_empty();
                is_icon_button
                    || (label_is_empty
                        && (self.leading_icon.is_some()
                            || self.trailing_icon.is_some()
                            || !self.children.is_empty()))
            };
            let has_svg_icon_like_children = !is_icon_button
                && (self.children.iter().any(contains_svg_icon_like)
                    || self.leading_icon.is_some()
                    || self.trailing_icon.is_some());
            let children = self.children;
            let visible_label = self.label;
            let leading_icon = self.leading_icon;
            let trailing_icon = self.trailing_icon;
            let leading_icon_size = self.leading_icon_size;
            let content_justify = self.content_justify;
            let label_font_override = self.label_font_override;
            let label_features_override = self.label_features_override;
            let label_axes_override = self.label_axes_override;

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_if_enabled_opt(command);
                if let Some(on_activate) = on_activate.clone() {
                    cx.pressable_on_activate(on_activate);
                } else if let Some(on_activate) = render_on_activate.clone() {
                    cx.pressable_on_activate(on_activate);
                }
                if let Some(on_hover_change) = on_hover_change.clone() {
                    cx.pressable_on_hover_change(on_hover_change);
                }
                if let Some(model) = toggle_model {
                    cx.pressable_toggle_bool(&model);
                }

                let states = WidgetStates::from_pressable(cx, st, !disabled);

                let bg = resolve_override_slot(
                    style_override.background.as_ref(),
                    &variant_style.background,
                    states,
                );
                let fg = resolve_override_slot(
                    style_override.foreground.as_ref(),
                    &variant_style.foreground,
                    states,
                );
                let border_color = resolve_override_slot(
                    style_override.border_color.as_ref(),
                    &variant_style.border_color,
                    states,
                );

                let padding = if is_icon {
                    ChromeRefinement::default()
                } else {
                    // shadcn/ui: `has-[>svg]:px-*` uses tighter horizontal padding when an icon is present.
                    let shrink_px = has_svg_icon_like_children;
                    match size {
                        ComponentSize::Small => ChromeRefinement::default()
                            .px(if shrink_px { Space::N2p5 } else { Space::N3 })
                            .py(Space::N1),
                        ComponentSize::Medium => ChromeRefinement::default()
                            .px(if shrink_px { Space::N3 } else { Space::N4 })
                            .py(Space::N2),
                        ComponentSize::Large => ChromeRefinement::default()
                            .px(if shrink_px { Space::N4 } else { Space::N6 })
                            .py(Space::N2),
                        ComponentSize::XSmall => {
                            ChromeRefinement::default().px(Space::N2).py(Space::N1)
                        }
                    }
                };

                let mut chrome = padding.merge(
                    ChromeRefinement::default()
                        .radius(radius)
                        .border_width(border_w),
                );
                if has_outline_shadow {
                    chrome.shadow = Some(ShadowPreset::Xs);
                }

                if !user_bg_override {
                    chrome.background = Some(bg);
                }
                if !user_border_override {
                    chrome.border_color = Some(border_color);
                }
                chrome = chrome.merge(user_chrome.clone());

                let mut chrome_props =
                    decl_style::container_props(&theme, chrome, LayoutRefinement::default());
                chrome_props.layout.size = pressable_layout.size;
                if let Some(border) = border_override {
                    chrome_props.border = border;
                }
                if let Some(border) = border_width_override.top {
                    chrome_props.border.top = border;
                }
                if let Some(border) = border_width_override.right {
                    chrome_props.border.right = border;
                }
                if let Some(border) = border_width_override.bottom {
                    chrome_props.border.bottom = border;
                }
                if let Some(border) = border_width_override.left {
                    chrome_props.border.left = border;
                }
                if let Some(corners) = corner_radii_override {
                    chrome_props.corner_radii = corners;
                }

                let focus_radius = {
                    let corners = chrome_props.corner_radii;
                    let mut max = corners.top_left.0;
                    max = max.max(corners.top_right.0);
                    max = max.max(corners.bottom_right.0);
                    max = max.max(corners.bottom_left.0);
                    Px(max)
                };

                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable,
                    focus_ring: Some(decl_style::focus_ring(&theme, focus_radius)),
                    key_activation: render_key_activation,
                    a11y: PressableA11y {
                        role: render_role,
                        label: Some(a11y_label.clone()),
                        test_id: test_id.clone(),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let content_children = move |cx: &mut ElementContext<'_, H>| {
                    current_color::scope_children(cx, fg.clone(), |cx| {
                        let gap = if is_icon {
                            Space::N0
                        } else {
                            match size {
                                ComponentSize::Small | ComponentSize::XSmall => Space::N1p5,
                                ComponentSize::Medium | ComponentSize::Large => Space::N2,
                            }
                        };

                        let mut children = Some(children);
                        let content = if children.as_ref().is_some_and(|c| c.is_empty()) {
                            let mut content = Vec::with_capacity(
                                usize::from(leading_icon.is_some())
                                    + usize::from(!visible_label.is_empty())
                                    + usize::from(trailing_icon.is_some()),
                            );

                            let icon_px = leading_icon_size.unwrap_or_else(|| match size {
                                // Upstream shadcn/ui v4:
                                // - `xs` / `icon-xs` => `size-3` (12px)
                                // - `sm` => `size-3.5` (14px)
                                // - default => `size-4` (16px)
                                ComponentSize::XSmall => Px(12.0),
                                ComponentSize::Small => Px(14.0),
                                ComponentSize::Medium | ComponentSize::Large => Px(16.0),
                            });
                            if let Some(icon) = leading_icon.clone() {
                                let icon = crate::icon::icon_with(cx, icon, Some(icon_px), None);
                                content.push(crate::test_id::attach_test_id_suffix(
                                    icon,
                                    test_id.as_ref(),
                                    "icon",
                                ));
                            }

                            if !visible_label.is_empty() {
                                let mut label = ui::text(cx, visible_label.clone())
                                    .text_size_px(text_px)
                                    .fixed_line_box_px(text_line_height)
                                    .line_box_in_bounds()
                                    .font_weight(text_weight)
                                    .nowrap()
                                    .text_color(fg.clone());
                                if let Some(font) = label_font_override {
                                    label = label.font(font);
                                }
                                for feature in &label_features_override {
                                    label =
                                        label.font_feature(feature.tag.to_string(), feature.value);
                                }
                                for axis in &label_axes_override {
                                    label = label.font_axis(axis.tag.to_string(), axis.value);
                                }
                                let label = label.into_element(cx);
                                content.push(crate::test_id::attach_test_id_suffix(
                                    label,
                                    test_id.as_ref(),
                                    "label",
                                ));
                            }

                            if let Some(icon) = trailing_icon.clone() {
                                let icon = crate::icon::icon_with(cx, icon, Some(icon_px), None);
                                content.push(crate::test_id::attach_test_id_suffix(
                                    icon,
                                    test_id.as_ref(),
                                    "trailing-icon",
                                ));
                            }

                            content
                        } else {
                            children.take().unwrap_or_default()
                        };

                        let mut props = fret_ui_kit::declarative::stack::HStackProps::default()
                            .justify(content_justify)
                            .items_center()
                            .gap_x(gap);
                        if is_icon {
                            // shadcn/ui icon buttons are `inline-flex items-center justify-center`
                            // in a fixed square. Ensure the content stack stretches so the icon
                            // is centered within that box.
                            props = props.layout(LayoutRefinement::default().w_full().h_full());
                        }

                        vec![fret_ui_kit::declarative::stack::hstack(cx, props, |_cx| {
                            content
                        })]
                    })
                };

                (pressable_props, chrome_props, content_children)
            });

            if disabled {
                cx.opacity(0.5, |_cx| vec![pressable])
            } else {
                pressable
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, MouseButton, MouseButtons, PathCommand, PathConstraints, PathId, PathMetrics,
        PathService, PathStyle, Point, Px, Rect, Scene, SceneOp, Size as CoreSize, SvgId,
        SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{
        CommandMeta, CommandScope, WindowCommandActionAvailabilityService,
        WindowCommandEnabledService, WindowCommandGatingService, WindowCommandGatingSnapshot,
    };
    use fret_ui::SvgSource;
    use fret_ui::Theme;
    use fret_ui::element::{ContainerProps, ElementKind, LayoutStyle, Length, SizeStyle};
    use fret_ui::elements;
    use fret_ui::tree::UiTree;
    use std::collections::HashMap;

    fn blend_over(fg: Color, bg: Color) -> Color {
        let a = fg.a.clamp(0.0, 1.0);
        Color {
            r: fg.r * a + bg.r * (1.0 - a),
            g: fg.g * a + bg.g * (1.0 - a),
            b: fg.b * a + bg.b * (1.0 - a),
            a: 1.0,
        }
    }

    fn alpha_mul(mut c: Color, mul: f32) -> Color {
        c.a *= mul;
        c
    }

    fn relative_luminance(c: Color) -> f32 {
        // The theme pipeline stores colors in linear space, so we can use the WCAG coefficients
        // directly.
        (0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b).clamp(0.0, 1.0)
    }

    fn contrast_ratio(a: Color, b: Color) -> f32 {
        let l1 = relative_luminance(a);
        let l2 = relative_luminance(b);
        let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
        (hi + 0.05) / (lo + 0.05)
    }

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: CoreSize::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
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

    #[test]
    fn outline_icon_button_shadow_and_ring_follow_rounded_full() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "outline-icon-button-shadow-and-ring-follow-rounded-full",
            |cx| {
                let el = Button::new("Next")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::IconSm)
                    .test_id("test-outline-icon-button")
                    .refine_style(ChromeRefinement::default().rounded(fret_ui_kit::Radius::Full))
                    .into_element(cx);
                let ElementKind::Pressable(pressable) = &el.kind else {
                    panic!("expected pressable root, got {:?}", el.kind);
                };
                let ring = pressable.focus_ring.as_ref().expect("focus ring");
                assert!(
                    ring.corner_radii.top_left.0 >= 900.0,
                    "expected rounded-full focus ring, got {:?}",
                    ring.corner_radii
                );

                let chrome = el.children.first().expect("chrome child");
                let ElementKind::Container(chrome_props) = &chrome.kind else {
                    panic!("expected chrome container, got {:?}", chrome.kind);
                };
                let shadow = chrome_props.shadow.as_ref().expect("outline shadow");
                assert!(
                    shadow.corner_radii.top_left.0 >= 900.0,
                    "expected rounded-full shadow, got {:?}",
                    shadow.corner_radii
                );

                vec![el]
            },
        );
        ui.set_root(root);
    }

    #[test]
    fn disabled_button_emits_opacity_stack_ops() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(200.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "disabled-button-emits-opacity-stack-ops",
            |cx| vec![Button::new("Hello").disabled(true).into_element(cx)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert!(matches!(
            scene.ops().first(),
            Some(SceneOp::PushOpacity { opacity }) if (*opacity - 0.5).abs() < 1e-6
        ));
        assert!(matches!(scene.ops().last(), Some(SceneOp::PopOpacity)));
    }

    #[test]
    fn disabled_button_is_not_focusable() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(200.0), Px(80.0)),
        );

        elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "disabled-button-focusable",
            |cx| {
                let el = Button::new("Hello").disabled(true).into_element(cx);
                let ElementKind::Opacity(root) = &el.kind else {
                    panic!(
                        "expected disabled Button to be wrapped in Opacity, got {:?}",
                        el.kind
                    );
                };
                assert!(
                    (root.opacity - 0.5).abs() < 1e-6,
                    "expected disabled Button to apply opacity 0.5"
                );

                let child = el.children.first().expect("opacity child");
                let ElementKind::Pressable(pressable) = &child.kind else {
                    panic!(
                        "expected Opacity child to be a Pressable, got {:?}",
                        child.kind
                    );
                };
                assert!(
                    !pressable.focusable,
                    "expected disabled Button to be non-focusable"
                );
            },
        );
    }

    #[test]
    fn icon_button_sizes_apply_min_dimensions() {
        let mut app = App::new();
        let window = AppWindowId::default();

        let (expected_sm, expected_md, expected_lg) = {
            let theme = Theme::global(&app);
            (
                fret_ui_kit::Size::Small.icon_button_size(theme),
                fret_ui_kit::Size::Medium.icon_button_size(theme),
                fret_ui_kit::Size::Large.icon_button_size(theme),
            )
        };

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );

        let icon_stub = |cx: &mut fret_ui::ElementContext<'_, App>| {
            cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(1.0)),
                            height: Length::Px(Px(1.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )
        };

        for (size, expected) in [
            (ButtonSize::IconSm, expected_sm),
            (ButtonSize::Icon, expected_md),
            (ButtonSize::IconLg, expected_lg),
        ] {
            let element =
                elements::with_element_cx(&mut app, window, bounds, "icon-button-size", |cx| {
                    Button::new("Icon button")
                        .size(size)
                        .children(vec![icon_stub(cx)])
                        .into_element(cx)
                });

            let ElementKind::Pressable(props) = &element.kind else {
                panic!("expected icon button to render as a Pressable");
            };

            assert_eq!(props.layout.size.min_width, Some(Length::Px(expected)));
            assert_eq!(props.layout.size.min_height, Some(Length::Px(expected)));
        }
    }

    #[test]
    fn button_padding_x_compacts_when_icon_present() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let theme = Theme::global(&app).snapshot();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );

        let icon_bytes: &'static [u8] =
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"></svg>"#;

        for (size, expected_px) in [
            (
                ButtonSize::Sm,
                fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N2p5).resolve(&theme),
            ),
            (
                ButtonSize::Default,
                fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N3).resolve(&theme),
            ),
            (
                ButtonSize::Lg,
                fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N4).resolve(&theme),
            ),
        ] {
            let element =
                elements::with_element_cx(&mut app, window, bounds, "button-padding", |cx| {
                    let icon = cx.svg_icon_props(fret_ui::element::SvgIconProps::new(
                        SvgSource::Static(icon_bytes),
                    ));
                    let text = cx.text("Label");

                    Button::new("Icon label")
                        .size(size)
                        .children(vec![icon, text])
                        .into_element(cx)
                });

            let ElementKind::Pressable(_props) = &element.kind else {
                panic!("expected button to render as a Pressable");
            };
            let Some(chrome) = element.children.first() else {
                panic!("expected button pressable to contain chrome container");
            };
            let ElementKind::Container(props) = &chrome.kind else {
                panic!("expected chrome container");
            };

            assert_eq!(props.padding.left, expected_px.into());
            assert_eq!(props.padding.right, expected_px.into());
        }
    }

    #[test]
    fn button_default_layout_does_not_flex_shrink() {
        let mut app = App::new();
        let window = AppWindowId::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );

        let element =
            elements::with_element_cx(&mut app, window, bounds, "button-flex-shrink", |cx| {
                Button::new("Default").into_element(cx)
            });

        let ElementKind::Pressable(props) = &element.kind else {
            panic!("expected button to render as a Pressable");
        };

        assert_eq!(props.layout.flex.shrink, 0.0);
    }

    #[test]
    fn outline_button_border_uses_ring_color_when_focused() {
        use std::cell::Cell;
        use std::rc::Rc;

        use fret_runtime::FrameId;
        use fret_ui::elements::GlobalElementId;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let ring = Theme::global(&app).snapshot().color_token("ring");

        let id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

        fn render_outline_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            id_out: Rc<Cell<Option<GlobalElementId>>>,
            expect_border_color: Option<Color>,
        ) {
            // Keep the render closure's callsite stable across frames so element identity is
            // stable under `#[track_caller]`-anchored IDs.
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "outline-button-focus-border",
                move |cx| {
                    let el = Button::new("Outline")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx);
                    id_out.set(Some(el.id));
                    if let Some(ring) = expect_border_color {
                        let chrome = el
                            .children
                            .first()
                            .expect("expected pressable to contain chrome container");
                        let ElementKind::Container(props) = &chrome.kind else {
                            panic!("expected chrome container element");
                        };
                        assert_eq!(props.border_color, Some(ring));
                    }
                    vec![el]
                },
            );
            ui.set_root(root);
        }

        // First frame: render once to obtain the element id and map to a node.
        app.set_frame_id(FrameId(1));
        render_outline_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            id_out.clone(),
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let id = id_out.get().expect("button element id");
        let node =
            elements::node_for_element(&mut app, window, id).expect("button node id resolved");
        ui.set_focus(Some(node));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        // Second frame: re-render with focus applied and capture the element tree.
        app.set_frame_id(FrameId(2));
        render_outline_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            id_out.clone(),
            Some(ring),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    #[test]
    fn command_gating_button_is_disabled_by_window_command_enabled_service() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let cmd = CommandId::from("test.disabled-command");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Disabled Command").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandEnabledService::default());
        app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
            svc.set_enabled(window, cmd.clone(), false);
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "command-gating-button-enabled-service",
            |cx| {
                vec![
                    Button::new("Hello")
                        .on_click(cmd.clone())
                        .test_id("disabled-button")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("disabled-button"))
            .expect("expected a semantics node for the button test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn command_gating_button_is_disabled_when_widget_action_is_unavailable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), false);
                svc.set_snapshot(window, snapshot);
            },
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "command-gating-button-action-availability",
            |cx| {
                vec![
                    Button::new("Hello")
                        .on_click(cmd.clone())
                        .test_id("disabled-button")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("disabled-button"))
            .expect("expected a semantics node for the button test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn command_gating_button_prefers_window_command_gating_snapshot_when_present() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), true);
                svc.set_snapshot(window, snapshot);
            },
        );

        app.set_global(WindowCommandGatingService::default());
        app.with_global_mut(WindowCommandGatingService::default, |svc, app| {
            let input_ctx = crate::command_gating::default_input_context(app);
            let enabled_overrides: HashMap<CommandId, bool> = HashMap::new();
            let mut availability: HashMap<CommandId, bool> = HashMap::new();
            availability.insert(cmd.clone(), false);
            let _token = svc.push_snapshot(
                window,
                WindowCommandGatingSnapshot::new(input_ctx, enabled_overrides)
                    .with_action_availability(Some(Arc::new(availability))),
            );
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "command-gating-button-gating-snapshot",
            |cx| {
                vec![
                    Button::new("Hello")
                        .on_click(cmd.clone())
                        .test_id("disabled-button")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("disabled-button"))
            .expect("expected a semantics node for the button test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn button_clears_hover_and_active_visuals_after_click_and_pointer_leave() {
        fn overlap_area(a: Rect, b: Rect) -> f32 {
            let ax0 = a.origin.x.0;
            let ay0 = a.origin.y.0;
            let ax1 = ax0 + a.size.width.0;
            let ay1 = ay0 + a.size.height.0;

            let bx0 = b.origin.x.0;
            let by0 = b.origin.y.0;
            let bx1 = bx0 + b.size.width.0;
            let by1 = by0 + b.size.height.0;

            let x0 = ax0.max(bx0);
            let y0 = ay0.max(by0);
            let x1 = ax1.min(bx1);
            let y1 = ay1.min(by1);

            let w = (x1 - x0).max(0.0);
            let h = (y1 - y0).max(0.0);
            w * h
        }

        fn assert_color_close(label: &str, actual: Color, expected: Color, eps: f32) {
            let dr = (actual.r - expected.r).abs();
            let dg = (actual.g - expected.g).abs();
            let db = (actual.b - expected.b).abs();
            let da = (actual.a - expected.a).abs();
            assert!(
                dr <= eps && dg <= eps && db <= eps && da <= eps,
                "{label}: expected rgba({:.3},{:.3},{:.3},{:.3}) got rgba({:.3},{:.3},{:.3},{:.3})",
                expected.r,
                expected.g,
                expected.b,
                expected.a,
                actual.r,
                actual.g,
                actual.b,
                actual.a
            );
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "button-clears-hover-and-active-after-click",
            |cx| {
                vec![
                    Button::new("Continue")
                        .test_id("continue-button")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("continue-button"))
            .expect("missing semantics node for continue button");
        let button_bounds = button.bounds;

        let inside = Point::new(
            Px(button_bounds.origin.x.0 + button_bounds.size.width.0 * 0.5),
            Px(button_bounds.origin.y.0 + button_bounds.size.height.0 * 0.5),
        );
        let outside = Point::new(
            Px(button_bounds.origin.x.0 + button_bounds.size.width.0 + 80.0),
            inside.y,
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: inside,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: inside,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: inside,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let (expected_bg, _expected_bg_hover, _expected_bg_active, _border, _fg) = {
            let theme = Theme::global(&app);
            variant_colors(theme, ButtonVariant::Default)
        };

        let mut best_quad: Option<(Rect, Color, f32)> = None;
        for op in scene.ops() {
            let SceneOp::Quad {
                rect, background, ..
            } = op
            else {
                continue;
            };
            let fret_core::Paint::Solid(bg_color) = background.paint else {
                continue;
            };
            if bg_color.a < 0.5 {
                continue;
            }
            let score = overlap_area(*rect, button_bounds);
            if score <= 0.0 {
                continue;
            }
            let replace = best_quad.is_none_or(|(_, _, best)| score > best);
            if replace {
                best_quad = Some((*rect, bg_color, score));
            }
        }

        let (_rect, actual_bg, _score) = best_quad.expect("missing painted quad for button");
        assert_color_close(
            "default button background after pointer leave",
            actual_bg,
            expected_bg,
            0.02,
        );
    }

    #[test]
    fn destructive_button_text_contrast_is_reasonable_in_zinc_dark() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Zinc,
            crate::shadcn_themes::ShadcnColorScheme::Dark,
        );

        let theme = Theme::global(&app);
        let snap = theme.snapshot();

        let (bg, _bg_hover, _bg_active, _border, fg) =
            variant_colors(theme, ButtonVariant::Destructive);
        let surface = snap.color_token("background");
        let bg_composited = blend_over(bg, surface);

        let ratio = contrast_ratio(fg, bg_composited);
        assert!(
            ratio >= 4.5,
            "expected destructive button contrast >= 4.5, got {ratio:.2} (fg={:?} bg={:?} bg_composited={:?} surface={:?})",
            fg,
            bg,
            bg_composited,
            surface,
        );
    }

    #[test]
    fn disabled_destructive_button_text_contrast_is_reasonable_in_zinc_dark() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Zinc,
            crate::shadcn_themes::ShadcnColorScheme::Dark,
        );

        let theme = Theme::global(&app);
        let snap = theme.snapshot();

        let (bg, _bg_hover, _bg_active, _border, fg) =
            variant_colors(theme, ButtonVariant::Destructive);
        let surface = snap.color_token("background");

        // Disabled buttons are wrapped in an opacity node (`disabled:opacity-50` upstream).
        // Model that as group alpha applied when compositing each pixel over the surface.
        let disabled_opacity = 0.5;
        let text_pixel = blend_over(alpha_mul(fg, disabled_opacity), surface);
        let bg_pixel = blend_over(alpha_mul(bg, disabled_opacity), surface);

        let ratio = contrast_ratio(text_pixel, bg_pixel);
        assert!(
            ratio >= 3.0,
            "expected disabled destructive button contrast >= 3.0, got {ratio:.2} (text={:?} bg={:?} surface={:?})",
            text_pixel,
            bg_pixel,
            surface,
        );
    }
}
