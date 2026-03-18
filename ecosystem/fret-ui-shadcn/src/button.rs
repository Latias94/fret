use std::any::Any;
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
use fret_ui_kit::declarative::icon;
use fret_ui_kit::declarative::motion::{
    drive_tween_color_for_element, drive_tween_f32_for_element,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, MetricRef, OverrideSlot,
    PaddingRefinement, Radius, ShadowPreset, Size as ComponentSize, Space, WidgetStateProperty,
    WidgetStates, resolve_override_slot, ui,
};

use crate::overlay_motion;

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

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    // Tailwind default `ease-in-out`: cubic-bezier(0.4, 0, 0.2, 1)
    fret_ui_kit::headless::easing::CubicBezier::new(0.4, 0.0, 0.2, 1.0).sample(t)
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

fn button_padding_refinement(
    dir: crate::direction::LayoutDirection,
    size: ComponentSize,
    shrink_inline_start: bool,
    shrink_inline_end: bool,
) -> PaddingRefinement {
    let (base_x, compact_x, pad_y) = match size {
        ComponentSize::XSmall => (Space::N2, Space::N1p5, Space::N1),
        ComponentSize::Small => (Space::N3, Space::N2p5, Space::N1),
        ComponentSize::Medium => (Space::N4, Space::N3, Space::N2),
        ComponentSize::Large => (Space::N6, Space::N4, Space::N2),
    };

    let pad_inline_start = MetricRef::space(if shrink_inline_start {
        compact_x
    } else {
        base_x
    });
    let pad_inline_end = MetricRef::space(if shrink_inline_end { compact_x } else { base_x });
    let pad_y = MetricRef::space(pad_y);

    let (left, right) = match dir {
        crate::direction::LayoutDirection::Ltr => (pad_inline_start, pad_inline_end),
        crate::direction::LayoutDirection::Rtl => (pad_inline_end, pad_inline_start),
    };

    PaddingRefinement {
        top: Some(pad_y.clone()),
        right: Some(right),
        bottom: Some(pad_y),
        left: Some(left),
    }
}

pub struct Button {
    label: Arc<str>,
    a11y_label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    leading_children: Vec<AnyElement>,
    trailing_children: Vec<AnyElement>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    label_font_override: Option<FontId>,
    label_features_override: Vec<TextFontFeatureSetting>,
    label_axes_override: Vec<TextFontAxisSetting>,
    leading_icon_size: Option<Px>,
    content_justify: Justify,
    text_weight_override: Option<FontWeight>,
    command: Option<CommandId>,
    action_payload: Option<Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>>,
    on_activate: Option<OnActivate>,
    on_hover_change: Option<OnHoverChange>,
    toggle_model: Option<fret_runtime::Model<bool>>,
    disabled: bool,
    focusable: bool,
    test_id: Option<Arc<str>>,
    control_id: Option<ControlId>,
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
            .field("leading_children_len", &self.leading_children.len())
            .field("trailing_children_len", &self.trailing_children.len())
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
            leading_children: Vec::new(),
            trailing_children: Vec::new(),
            leading_icon: None,
            trailing_icon: None,
            label_font_override: None,
            label_features_override: Vec::new(),
            label_axes_override: Vec::new(),
            leading_icon_size: None,
            content_justify: Justify::Center,
            text_weight_override: None,
            command: None,
            action_payload: None,
            on_activate: None,
            on_hover_change: None,
            toggle_model: None,
            disabled: false,
            focusable: true,
            test_id: None,
            control_id: None,
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

    /// Binds this Button to a logical control id (similar to HTML `id`).
    ///
    /// When set, `Label::for_control(ControlId)` forwards focus to the button pressable. This is
    /// useful for "button-as-form-control" triggers (e.g. date picker / combobox triggers) where
    /// the field label should focus the trigger.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Appends one fully custom content node, replacing the default label row once any custom
    /// content is present.
    ///
    /// Prefer `leading_child(...)` / `trailing_child(...)` when you want to preserve the default
    /// label and slot layout.
    pub fn child(mut self, child: AnyElement) -> Self {
        self.children.push(child);
        self
    }

    /// Adds inline-start content while preserving the button's default label and slot layout.
    ///
    /// Prefer this for dynamic affordances such as `Spinner`, matching upstream
    /// `data-icon="inline-start"` compositions without forcing a full content override.
    pub fn leading_children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.leading_children = children.into_iter().collect();
        self
    }

    /// Appends one inline-start child while preserving the button's default label and slot layout.
    pub fn leading_child(mut self, child: AnyElement) -> Self {
        self.leading_children.push(child);
        self
    }

    /// Adds inline-end content while preserving the button's default label and slot layout.
    ///
    /// Prefer this for trailing affordances such as `Spinner`, matching upstream
    /// `data-icon="inline-end"` compositions without forcing a full content override.
    pub fn trailing_children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.trailing_children = children.into_iter().collect();
        self
    }

    /// Appends one inline-end child while preserving the button's default label and slot layout.
    pub fn trailing_child(mut self, child: AnyElement) -> Self {
        self.trailing_children.push(child);
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

    /// Bind a stable action ID to this button (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
    }

    /// Attach a payload for parameterized actions (ADR 0312).
    ///
    /// Notes:
    /// - Payload is transient and best-effort (window-scoped pending store + TTL).
    /// - Keymap/palette/menus remain unit-action surfaces in v2.
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`Button::action_payload`], but allows callers to compute the payload at activation time.
    pub fn action_payload_factory(
        mut self,
        payload: Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>,
    ) -> Self {
        self.action_payload = Some(payload);
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
            let dir = crate::direction::use_direction(cx, None);

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
            let content_fill_w = matches!(
                pressable_layout.size.width,
                fret_ui::element::Length::Px(_) | fret_ui::element::Length::Fill
            ) || pressable_layout.flex.grow > 0.0;
            let content_fill_h = matches!(
                pressable_layout.size.height,
                fret_ui::element::Length::Px(_) | fret_ui::element::Length::Fill
            );

            let command = self.command;
            let action_payload = self.action_payload;
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
            let has_a11y_label_override = self.a11y_label.is_some();
            let control_id = self.control_id.clone();
            let control_registry = control_id.as_ref().map(|_| control_registry_model(cx));
            let labelled_by_element = if has_a11y_label_override {
                None
            } else if let (Some(control_id), Some(control_registry)) =
                (control_id.as_ref(), control_registry.as_ref())
            {
                cx.app
                    .models()
                    .read(control_registry, |reg| {
                        reg.label_for(cx.window, control_id).map(|l| l.element)
                    })
                    .ok()
                    .flatten()
            } else {
                None
            };
            let described_by_element = if let (Some(control_id), Some(control_registry)) =
                (control_id.as_ref(), control_registry.as_ref())
            {
                cx.app
                    .models()
                    .read(control_registry, |reg| {
                        reg.described_by_for(cx.window, control_id)
                    })
                    .ok()
                    .flatten()
            } else {
                None
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
            let has_content_override = !self.children.is_empty();
            let has_inline_start_svg_like_children = !has_content_override
                && (self.leading_icon.is_some()
                    || self.leading_children.iter().any(contains_svg_icon_like));
            let has_inline_end_svg_like_children = !has_content_override
                && (self.trailing_icon.is_some()
                    || self.trailing_children.iter().any(contains_svg_icon_like));
            let has_override_svg_like_children =
                has_content_override && self.children.iter().any(contains_svg_icon_like);
            let is_icon = {
                let label_is_empty = self.label.is_empty();
                is_icon_button
                    || (label_is_empty
                        && (self.leading_icon.is_some()
                            || self.trailing_icon.is_some()
                            || !self.leading_children.is_empty()
                            || !self.trailing_children.is_empty()
                            || !self.children.is_empty()))
            };
            let has_svg_icon_like_children = !is_icon_button
                && (has_override_svg_like_children
                    || has_inline_start_svg_like_children
                    || has_inline_end_svg_like_children);
            let children = self.children;
            let leading_children = self.leading_children;
            let trailing_children = self.trailing_children;
            let visible_label = self.label;
            let leading_icon = self.leading_icon;
            let trailing_icon = self.trailing_icon;
            let leading_icon_size = self.leading_icon_size;
            let content_justify = self.content_justify;
            let label_font_override = self.label_font_override;
            let label_features_override = self.label_features_override;
            let label_axes_override = self.label_axes_override;
            let control_id_for_register = control_id.clone();
            let control_registry_for_register = control_registry.clone();
            let labelled_by_element_for_a11y = labelled_by_element;
            let described_by_element_for_a11y = described_by_element;

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                if let (Some(control_id), Some(control_registry)) = (
                    control_id_for_register.clone(),
                    control_registry_for_register.clone(),
                ) {
                    let entry = ControlEntry {
                        element: _id,
                        enabled: !disabled,
                        action: ControlAction::Noop,
                    };
                    let _ = cx.app.models_mut().update(&control_registry, |reg| {
                        reg.register_control(cx.window, cx.frame_id, control_id, entry);
                    });
                }

                if let Some(payload) = action_payload.clone() {
                    cx.pressable_dispatch_command_with_payload_factory_if_enabled_opt(
                        command.clone(),
                        payload,
                    );
                } else {
                    cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                }
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

                let duration = overlay_motion::shadcn_motion_duration_150(cx);
                let bg_motion = drive_tween_color_for_element(
                    cx,
                    _id,
                    "button.chrome.bg",
                    bg.resolve(&theme),
                    duration,
                    tailwind_transition_ease_in_out,
                );
                let fg_motion = drive_tween_color_for_element(
                    cx,
                    _id,
                    "button.content.fg",
                    fg.resolve(&theme),
                    duration,
                    tailwind_transition_ease_in_out,
                );
                let border_motion = drive_tween_color_for_element(
                    cx,
                    _id,
                    "button.chrome.border",
                    border_color.resolve(&theme),
                    duration,
                    tailwind_transition_ease_in_out,
                );

                let ring_alpha = drive_tween_f32_for_element(
                    cx,
                    _id,
                    "button.chrome.ring.alpha",
                    if states.contains(WidgetStates::FOCUS_VISIBLE) {
                        1.0
                    } else {
                        0.0
                    },
                    duration,
                    tailwind_transition_ease_in_out,
                );

                let bg = ColorRef::Color(bg_motion.value);
                let fg = ColorRef::Color(fg_motion.value);
                let border_color = ColorRef::Color(border_motion.value);

                let padding = if is_icon {
                    ChromeRefinement::default()
                } else {
                    // Upstream shadcn buttons compact only the occupied inline side when icon-like
                    // content is expressed through `data-icon="inline-start|inline-end"`. Keep
                    // the older "shrink both sides" fallback only for full content overrides.
                    let (shrink_inline_start, shrink_inline_end) = if has_content_override {
                        let shrink = has_svg_icon_like_children;
                        (shrink, shrink)
                    } else {
                        (
                            has_inline_start_svg_like_children,
                            has_inline_end_svg_like_children,
                        )
                    };

                    let mut chrome = ChromeRefinement::default();
                    chrome.padding = Some(button_padding_refinement(
                        dir,
                        size,
                        shrink_inline_start,
                        shrink_inline_end,
                    ));
                    chrome
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

                let mut focus_ring = decl_style::focus_ring(&theme, focus_radius);
                focus_ring.color.a = (focus_ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
                if let Some(offset_color) = focus_ring.offset_color {
                    focus_ring.offset_color = Some(Color {
                        a: (offset_color.a * ring_alpha.value).clamp(0.0, 1.0),
                        ..offset_color
                    });
                }

                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable,
                    focus_ring: Some(focus_ring),
                    focus_ring_always_paint: ring_alpha.animating,
                    key_activation: render_key_activation,
                    a11y: PressableA11y {
                        role: render_role,
                        label: if has_a11y_label_override || control_id.is_none() {
                            Some(a11y_label.clone())
                        } else {
                            None
                        },
                        test_id: test_id.clone(),
                        labelled_by_element: if has_a11y_label_override {
                            None
                        } else {
                            labelled_by_element_for_a11y.map(|id| id.0)
                        },
                        described_by_element: described_by_element_for_a11y.map(|id| id.0),
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
                        let mut leading_children = Some(leading_children);
                        let mut trailing_children = Some(trailing_children);
                        let content = if children.as_ref().is_some_and(|c| c.is_empty()) {
                            let mut inline_start = Vec::with_capacity(
                                usize::from(leading_icon.is_some())
                                    + leading_children.as_ref().map_or(0, Vec::len),
                            );
                            let mut inline_end = Vec::with_capacity(
                                trailing_children.as_ref().map_or(0, Vec::len)
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
                                let icon = icon::icon_with(cx, icon, Some(icon_px), None);
                                inline_start.push(crate::test_id::attach_test_id_suffix(
                                    icon,
                                    test_id.as_ref(),
                                    "icon",
                                ));
                            }
                            inline_start.extend(leading_children.take().unwrap_or_default());

                            let label = if !visible_label.is_empty() {
                                let mut label = ui::text(visible_label.clone())
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
                                Some(crate::test_id::attach_test_id_suffix(
                                    label,
                                    test_id.as_ref(),
                                    "label",
                                ))
                            } else {
                                None
                            };

                            inline_end.extend(trailing_children.take().unwrap_or_default());
                            if let Some(icon) = trailing_icon.clone() {
                                let icon = icon::icon_with(cx, icon, Some(icon_px), None);
                                inline_end.push(crate::test_id::attach_test_id_suffix(
                                    icon,
                                    test_id.as_ref(),
                                    "trailing-icon",
                                ));
                            }

                            match dir {
                                crate::direction::LayoutDirection::Ltr => {
                                    let mut content = inline_start;
                                    if let Some(label) = label {
                                        content.push(label);
                                    }
                                    content.extend(inline_end);
                                    content
                                }
                                crate::direction::LayoutDirection::Rtl => {
                                    inline_end.reverse();
                                    let mut content = inline_end;
                                    if let Some(label) = label {
                                        content.push(label);
                                    }
                                    inline_start.reverse();
                                    content.extend(inline_start);
                                    content
                                }
                            }
                        } else {
                            children.take().unwrap_or_default()
                        };

                        let mut builder = ui::h_row(move |_cx| content)
                            .justify(content_justify)
                            .items_center()
                            .gap(gap);
                        if content_fill_w || content_fill_h {
                            // Match shadcn/ui's `inline-flex items-center justify-center` behavior:
                            // when the control resolves to a definite box (fixed size, `w-full`, or
                            // flex-grow), let the content stack fill that box so cross-axis
                            // centering behaves like CSS flexbox even when padding/border leave a
                            // smaller content box (notably `outline`).
                            let mut layout = LayoutRefinement::default();
                            if content_fill_w {
                                layout = layout.w_full();
                            }
                            if content_fill_h {
                                layout = layout.h_full();
                            }
                            builder = builder.layout(layout);
                        }

                        vec![builder.into_element(cx)]
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
    use crate::spinner::Spinner;

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
    use std::time::Duration;

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

    fn color_eq_eps(a: Color, b: Color, eps: f32) -> bool {
        (a.r - b.r).abs() <= eps
            && (a.g - b.g).abs() <= eps
            && (a.b - b.b).abs() <= eps
            && (a.a - b.a).abs() <= eps
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

        crate::shadcn_themes::apply_shadcn_new_york(
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
    fn button_inline_slot_padding_compacts_only_the_occupied_inline_side() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let theme = Theme::global(&app).snapshot();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );

        let base_px = MetricRef::space(Space::N4).resolve(&theme);
        let compact_px = MetricRef::space(Space::N3).resolve(&theme);

        let leading =
            elements::with_element_cx(&mut app, window, bounds, "button-leading-slot", |cx| {
                Button::new("Generating")
                    .leading_children(vec![Spinner::new().speed(0.0).into_element(cx)])
                    .into_element(cx)
            });
        let leading_chrome = leading.children.first().expect("expected chrome container");
        let ElementKind::Container(leading_props) = &leading_chrome.kind else {
            panic!("expected chrome container");
        };
        assert_eq!(leading_props.padding.left, compact_px.into());
        assert_eq!(leading_props.padding.right, base_px.into());

        let trailing =
            elements::with_element_cx(&mut app, window, bounds, "button-trailing-slot", |cx| {
                Button::new("Downloading")
                    .trailing_children(vec![Spinner::new().speed(0.0).into_element(cx)])
                    .into_element(cx)
            });
        let trailing_chrome = trailing
            .children
            .first()
            .expect("expected chrome container");
        let ElementKind::Container(trailing_props) = &trailing_chrome.kind else {
            panic!("expected chrome container");
        };
        assert_eq!(trailing_props.padding.left, base_px.into());
        assert_eq!(trailing_props.padding.right, compact_px.into());

        let rtl =
            elements::with_element_cx(&mut app, window, bounds, "button-leading-slot-rtl", |cx| {
                crate::direction::with_direction_provider(
                    cx,
                    crate::direction::LayoutDirection::Rtl,
                    |cx| {
                        Button::new("جاري التحميل")
                            .leading_children(vec![Spinner::new().speed(0.0).into_element(cx)])
                            .into_element(cx)
                    },
                )
            });
        let rtl_chrome = rtl.children.first().expect("expected chrome container");
        let ElementKind::Container(rtl_props) = &rtl_chrome.kind else {
            panic!("expected chrome container");
        };
        assert_eq!(rtl_props.padding.left, base_px.into());
        assert_eq!(rtl_props.padding.right, compact_px.into());
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

    fn find_first_horizontal_content_layout(
        el: &AnyElement,
    ) -> Option<fret_ui::element::LayoutStyle> {
        match &el.kind {
            ElementKind::Row(props) => Some(props.layout),
            ElementKind::Flex(props) if props.direction == fret_core::Axis::Horizontal => {
                Some(props.layout)
            }
            _ => el
                .children
                .iter()
                .find_map(find_first_horizontal_content_layout),
        }
    }

    fn find_first_horizontal_content_element(el: &AnyElement) -> Option<&AnyElement> {
        match &el.kind {
            ElementKind::Row(_) => Some(el),
            ElementKind::Flex(props) if props.direction == fret_core::Axis::Horizontal => Some(el),
            _ => el
                .children
                .iter()
                .find_map(find_first_horizontal_content_element),
        }
    }

    fn collect_horizontal_content_texts(el: &AnyElement) -> Vec<String> {
        let content =
            find_first_horizontal_content_element(el).expect("expected horizontal content stack");
        content
            .children
            .iter()
            .filter_map(|child| match &child.kind {
                ElementKind::Text(props) => Some(props.text.to_string()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn button_inline_slots_preserve_label_and_flip_order_in_rtl() {
        let mut app = App::new();
        let window = AppWindowId::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );

        let ltr =
            elements::with_element_cx(&mut app, window, bounds, "button-inline-slots-ltr", |cx| {
                Button::new("Main")
                    .leading_children(vec![cx.text("start")])
                    .trailing_children(vec![cx.text("end")])
                    .into_element(cx)
            });
        assert_eq!(
            collect_horizontal_content_texts(&ltr),
            vec!["start", "Main", "end"]
        );

        let rtl =
            elements::with_element_cx(&mut app, window, bounds, "button-inline-slots-rtl", |cx| {
                crate::direction::with_direction_provider(
                    cx,
                    crate::direction::LayoutDirection::Rtl,
                    |cx| {
                        Button::new("Main")
                            .leading_children(vec![cx.text("start")])
                            .trailing_children(vec![cx.text("end")])
                            .into_element(cx)
                    },
                )
            });
        assert_eq!(
            collect_horizontal_content_texts(&rtl),
            vec!["end", "Main", "start"]
        );
    }

    #[test]
    fn button_single_child_helpers_append_without_replacing_existing_content() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(120.0)),
        );

        let _ = elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "button-single-child-helpers",
            |cx| {
                let button = Button::new("Hello")
                    .child(cx.text("custom-a"))
                    .child(cx.text("custom-b"))
                    .leading_child(cx.text("lead"))
                    .trailing_child(cx.text("trail"));

                assert_eq!(button.children.len(), 2);
                assert_eq!(button.leading_children.len(), 1);
                assert_eq!(button.trailing_children.len(), 1);

                button.into_element(cx)
            },
        );
    }

    #[test]
    fn button_content_row_fills_height_for_definite_controls() {
        let mut app = App::new();
        let window = AppWindowId::default();

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(160.0)),
        );

        let element =
            elements::with_element_cx(&mut app, window, bounds, "button-content-row-fill", |cx| {
                Button::new("Back")
                    .variant(ButtonVariant::Outline)
                    .leading_icon(IconId::new_static("lucide.arrow-left"))
                    .test_id("test-button-outline-back")
                    .into_element(cx)
            });

        let layout = find_first_horizontal_content_layout(&element)
            .expect("expected Button to render a horizontal content stack");
        assert_eq!(layout.size.height, Length::Fill);
        assert_eq!(layout.size.width, Length::Auto);
    }

    #[test]
    fn button_content_row_fills_width_when_button_fills() {
        let mut app = App::new();
        let window = AppWindowId::default();

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(160.0)),
        );

        let element = elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "button-content-row-fill-width",
            |cx| {
                Button::new("Back")
                    .variant(ButtonVariant::Outline)
                    .leading_icon(IconId::new_static("lucide.arrow-left"))
                    .refine_layout(LayoutRefinement::default().w_full())
                    .test_id("test-button-outline-back-w-full")
                    .into_element(cx)
            },
        );

        let layout = find_first_horizontal_content_layout(&element)
            .expect("expected Button to render a horizontal content stack");
        assert_eq!(layout.size.height, Length::Fill);
        assert_eq!(layout.size.width, Length::Fill);
    }

    #[test]
    fn outline_button_border_uses_ring_color_when_focused() {
        use std::cell::Cell;
        use std::rc::Rc;

        use fret_runtime::FrameId;
        use fret_ui::elements::GlobalElementId;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

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
        let base_border = variant_style(ButtonVariant::Outline)
            .border_color
            .resolve(WidgetStates::empty())
            .resolve(&Theme::global(&app).snapshot());

        let id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let border_color_out: Rc<Cell<Option<Color>>> = Rc::new(Cell::new(None));

        fn render_outline_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            id_out: Rc<Cell<Option<GlobalElementId>>>,
            border_color_out: Rc<Cell<Option<Color>>>,
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
                    let chrome = el
                        .children
                        .first()
                        .expect("expected pressable to contain chrome container");
                    let ElementKind::Container(props) = &chrome.kind else {
                        panic!("expected chrome container element");
                    };
                    border_color_out.set(props.border_color);
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
            border_color_out.clone(),
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

        // Second frame: re-render with focus-visible applied and ensure the border is transitioning.
        app.set_frame_id(FrameId(2));
        render_outline_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            id_out.clone(),
            border_color_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let border1 = border_color_out.get().expect("border color");
        assert!(
            !color_eq_eps(border1, base_border, 1e-6) && !color_eq_eps(border1, ring, 1e-6),
            "expected outline focus border to tween (intermediate), got border={border1:?} base={base_border:?} ring={ring:?}"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            render_outline_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                id_out.clone(),
                border_color_out.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let border_final = border_color_out.get().expect("border_final");
        assert!(
            color_eq_eps(border_final, ring, 1e-4),
            "expected outline focus border to settle to ring; got border={border_final:?} ring={ring:?}"
        );
    }

    #[test]
    fn button_focus_ring_alpha_tweens_in_and_out_like_a_transition() {
        use std::cell::Cell;
        use std::rc::Rc;

        use fret_runtime::FrameId;
        use fret_ui::elements::GlobalElementId;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let theme = Theme::global(&app).snapshot();
        let base_alpha = fret_ui_kit::declarative::style::focus_ring(&theme, Px(4.0))
            .color
            .a;

        let id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            id_out: Rc<Cell<Option<GlobalElementId>>>,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "button-focus-ring-transition",
                move |cx| {
                    let el = Button::new("Default").into_element(cx);
                    id_out.set(Some(el.id));

                    let ElementKind::Pressable(props) = &el.kind else {
                        panic!("expected button to render as a Pressable");
                    };
                    let alpha = props
                        .focus_ring
                        .as_ref()
                        .map(|ring| ring.color.a)
                        .unwrap_or(0.0);
                    ring_alpha_out.set(Some(alpha));
                    always_paint_out.set(Some(props.focus_ring_always_paint));
                    vec![el]
                },
            );
            ui.set_root(root);
        }

        // Frame 1: unfocused, ring should be fully hidden.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            id_out.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let id = id_out.get().expect("button element id");
        let node = elements::node_for_element(&mut app, window, id).expect("button node");

        assert!(
            ring_alpha_out.get().expect("alpha").abs() <= 1e-6,
            "expected initial ring alpha to be 0; got {:?}",
            ring_alpha_out.get()
        );
        assert_eq!(
            always_paint_out.get().expect("always paint"),
            false,
            "expected initial focus_ring_always_paint=false"
        );

        // Focus it and switch modality to keyboard (focus-visible).
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

        // Frame 2: focus-visible should start a tween (intermediate alpha) and paint-on-exit is ok.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            id_out.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let alpha2 = ring_alpha_out.get().expect("alpha2");
        assert!(
            alpha2 > 1e-6 && alpha2 < base_alpha - 1e-6,
            "expected ring alpha to tween in (intermediate); got alpha={alpha2} base_alpha={base_alpha}"
        );
        assert!(
            always_paint_out.get().expect("always paint2"),
            "expected focus_ring_always_paint while animating"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                id_out.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }
        let alpha_final = ring_alpha_out.get().expect("alpha_final");
        assert!(
            (alpha_final - base_alpha).abs() <= 1e-4,
            "expected ring alpha to settle to base; got alpha={alpha_final} base_alpha={base_alpha}"
        );
        assert!(
            !always_paint_out.get().expect("always paint final"),
            "expected focus_ring_always_paint=false once settled"
        );

        // Blur: should animate out and keep painting while alpha decreases.
        ui.set_focus(None);
        app.set_frame_id(FrameId(3 + settle));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            id_out.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let alpha_out = ring_alpha_out.get().expect("alpha_out");
        assert!(
            alpha_out > 1e-6 && alpha_out < base_alpha - 1e-6,
            "expected ring alpha to tween out (intermediate); got alpha={alpha_out} base_alpha={base_alpha}"
        );
        assert!(
            always_paint_out.get().expect("always paint out"),
            "expected focus_ring_always_paint while animating out"
        );

        for i in 0..settle {
            app.set_frame_id(FrameId(4 + settle + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                id_out.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let alpha_zero = ring_alpha_out.get().expect("alpha_zero");
        assert!(
            alpha_zero.abs() <= 1e-4,
            "expected ring alpha to settle to 0; got alpha={alpha_zero}"
        );
        assert!(
            !always_paint_out.get().expect("always paint after out"),
            "expected focus_ring_always_paint=false once ring alpha is 0"
        );
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
    fn button_hover_background_tweens_instead_of_snapping() {
        use std::cell::Cell;
        use std::rc::Rc;

        use fret_runtime::FrameId;
        use fret_ui::elements::GlobalElementId;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let button_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let bg_out: Rc<Cell<Option<Color>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            button_id: Rc<Cell<Option<GlobalElementId>>>,
            bg_out: Rc<Cell<Option<Color>>>,
        ) {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "button-hover-bg-tween",
                move |cx| {
                    let el = Button::new("Hello")
                        .variant(ButtonVariant::Default)
                        .test_id("test-button-hover-bg-tween")
                        .into_element(cx);
                    button_id.set(Some(el.id));

                    let chrome = el
                        .children
                        .first()
                        .expect("expected pressable to contain chrome container");
                    let ElementKind::Container(props) = &chrome.kind else {
                        panic!("expected chrome container element");
                    };
                    let bg = props
                        .background
                        .as_ref()
                        .copied()
                        .unwrap_or(Color::TRANSPARENT);
                    bg_out.set(Some(bg));

                    vec![el]
                },
            );
            ui.set_root(root);
        }

        let theme = Theme::global(&app).snapshot();
        let style = variant_style(ButtonVariant::Default);
        let base_bg = style
            .background
            .resolve(WidgetStates::empty())
            .resolve(&theme);
        let hover_bg = style
            .background
            .resolve(WidgetStates::HOVERED)
            .resolve(&theme);

        // Frame 1: baseline render.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            button_id.clone(),
            bg_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let bg0 = bg_out.get().expect("bg0");
        assert!(
            color_eq_eps(bg0, base_bg, 1e-6),
            "expected base background to match; got bg0={bg0:?} base={base_bg:?}"
        );

        let id = button_id.get().expect("button id");
        let node = elements::node_for_element(&mut app, window, id).expect("button node");
        let b = ui.debug_node_bounds(node).expect("button bounds");
        let center = Point::new(
            Px(b.origin.x.0 + b.size.width.0 * 0.5),
            Px(b.origin.y.0 + b.size.height.0 * 0.5),
        );

        // Hover to retarget the transition.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: hover is applied; the background should be in-between (not snapped).
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            button_id.clone(),
            bg_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let bg1 = bg_out.get().expect("bg1");
        assert!(
            !color_eq_eps(bg1, base_bg, 1e-6) && !color_eq_eps(bg1, hover_bg, 1e-6),
            "expected hover background to tween (intermediate), got bg1={bg1:?} base={base_bg:?} hover={hover_bg:?}"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                button_id.clone(),
                bg_out.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let bg_final = bg_out.get().expect("bg_final");
        assert!(
            color_eq_eps(bg_final, hover_bg, 1e-4),
            "expected hover background to settle; got bg={bg_final:?} hover={hover_bg:?}"
        );
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

        crate::shadcn_themes::apply_shadcn_new_york(
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
        crate::shadcn_themes::apply_shadcn_new_york(
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
        crate::shadcn_themes::apply_shadcn_new_york(
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
