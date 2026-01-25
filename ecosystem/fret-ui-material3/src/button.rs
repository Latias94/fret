use std::sync::Arc;

use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, PressableA11y, PressableProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::stack::{HStackProps, hstack};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Radius, Space,
    WidgetStateProperty, WidgetStates, ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Filled,
    Outlined,
    Text,
}

#[derive(Debug, Clone, Default)]
pub struct ButtonStyle {
    pub background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub foreground: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub border_color: Option<WidgetStateProperty<Option<ColorRef>>>,
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

fn token(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

fn default_style_for_variant(
    variant: ButtonVariant,
) -> (
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
) {
    match variant {
        ButtonVariant::Filled => (
            WidgetStateProperty::new(Some(token(
                "material3.button.filled.container",
                ColorFallback::ThemeAccent,
            )))
            .when(
                WidgetStates::DISABLED,
                Some(token(
                    "material3.button.filled.disabled.container",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "primary",
                        mul: 0.38,
                    },
                )),
            ),
            WidgetStateProperty::new(Some(token(
                "material3.button.filled.label",
                ColorFallback::ThemeTextPrimary,
            )))
            .when(
                WidgetStates::DISABLED,
                Some(token(
                    "material3.button.filled.disabled.label",
                    ColorFallback::ThemeTextDisabled,
                )),
            ),
            WidgetStateProperty::new(None),
        ),
        ButtonVariant::Outlined => (
            WidgetStateProperty::new(None)
                .when(
                    WidgetStates::HOVERED,
                    Some(token(
                        "material3.button.state_layer.hover",
                        ColorFallback::ThemeTokenAlphaMul {
                            key: "accent",
                            mul: 0.08,
                        },
                    )),
                )
                .when(
                    WidgetStates::ACTIVE,
                    Some(token(
                        "material3.button.state_layer.pressed",
                        ColorFallback::ThemeTokenAlphaMul {
                            key: "accent",
                            mul: 0.12,
                        },
                    )),
                ),
            WidgetStateProperty::new(Some(token(
                "material3.button.outlined.label",
                ColorFallback::ThemeAccent,
            )))
            .when(
                WidgetStates::DISABLED,
                Some(token(
                    "material3.button.outlined.disabled.label",
                    ColorFallback::ThemeTextDisabled,
                )),
            ),
            WidgetStateProperty::new(Some(token(
                "material3.button.outlined.outline",
                ColorFallback::ThemePanelBorder,
            )))
            .when(
                WidgetStates::FOCUS_VISIBLE,
                Some(token(
                    "material3.button.outlined.focus.outline",
                    ColorFallback::ThemeAccent,
                )),
            )
            .when(
                WidgetStates::DISABLED,
                Some(token(
                    "material3.button.outlined.disabled.outline",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "border",
                        mul: 0.38,
                    },
                )),
            ),
        ),
        ButtonVariant::Text => (
            WidgetStateProperty::new(None)
                .when(
                    WidgetStates::HOVERED,
                    Some(token(
                        "material3.button.state_layer.hover",
                        ColorFallback::ThemeTokenAlphaMul {
                            key: "accent",
                            mul: 0.08,
                        },
                    )),
                )
                .when(
                    WidgetStates::ACTIVE,
                    Some(token(
                        "material3.button.state_layer.pressed",
                        ColorFallback::ThemeTokenAlphaMul {
                            key: "accent",
                            mul: 0.12,
                        },
                    )),
                ),
            WidgetStateProperty::new(Some(token(
                "material3.button.text.label",
                ColorFallback::ThemeAccent,
            )))
            .when(
                WidgetStates::DISABLED,
                Some(token(
                    "material3.button.text.disabled.label",
                    ColorFallback::ThemeTextDisabled,
                )),
            ),
            WidgetStateProperty::new(None),
        ),
    }
}

#[derive(Clone, Default)]
pub struct Button {
    label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    a11y_label: Option<Arc<str>>,
    disabled: bool,
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    variant: ButtonVariant,
    style: ButtonStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Button {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: Some(label.into()),
            ..Default::default()
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn child(mut self, child: AnyElement) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(mut self, command: CommandId) -> Self {
        self.on_click = Some(command);
        self
    }

    pub fn on_activate(mut self, handler: OnActivate) -> Self {
        self.on_activate = Some(handler);
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

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let variant = self.variant;
        let (default_bg, default_fg, default_border) = default_style_for_variant(variant);
        let style_override = self.style;

        let label = self.label;
        let children = self.children;
        let a11y_label = self.a11y_label;

        let on_click = self.on_click;
        let on_activate = self.on_activate;
        let disabled = self.disabled
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));

        let user_chrome = self.chrome;
        let user_bg_override = user_chrome.background.is_some();
        let user_border_override = user_chrome.border_color.is_some();

        let layout = LayoutRefinement::default()
            .min_h(MetricRef::Px(Px(40.0)))
            .merge(self.layout);
        let pressable_layout = decl_style::layout_style(&theme, layout);

        let radius = MetricRef::radius(Radius::Full).resolve(&theme);
        let mut ring = decl_style::focus_ring(&theme, radius);
        ring.color = theme.color_required("ring");

        control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click);
            if let Some(on_activate) = on_activate.clone() {
                cx.pressable_on_activate(on_activate);
            }

            let states = WidgetStates::from_pressable(cx, st, !disabled);

            let bg_ref = style_override
                .background
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_bg.resolve(states).clone());
            let fg_ref = style_override
                .foreground
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_fg.resolve(states).clone());
            let border_ref = style_override
                .border_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_border.resolve(states).clone());

            let bg = bg_ref.map(|c| c.resolve(&theme));
            let fg = fg_ref
                .unwrap_or(ColorRef::Color(theme.color_required("foreground")))
                .resolve(&theme);
            let border = border_ref.map(|c| c.resolve(&theme));

            let padding = ChromeRefinement::default().px(Space::N4).py(Space::N2);
            let mut chrome = padding
                .merge(ChromeRefinement {
                    radius: Some(MetricRef::Px(radius)),
                    border_width: Some(MetricRef::Px(Px(if variant == ButtonVariant::Outlined {
                        1.0
                    } else {
                        0.0
                    }))),
                    ..Default::default()
                })
                .merge(user_chrome.clone());

            if !user_bg_override {
                chrome.background = bg.map(ColorRef::Color);
            }
            if !user_border_override {
                chrome.border_color = border.map(ColorRef::Color);
            }

            let chrome_props =
                decl_style::container_props(&theme, chrome, LayoutRefinement::default());

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: !disabled,
                focusable: !disabled,
                focus_ring: Some(ring),
                a11y: PressableA11y {
                    role: Some(fret_core::SemanticsRole::Button),
                    label: a11y_label.clone().or_else(|| label.clone()),
                    ..Default::default()
                },
                ..Default::default()
            };

            let content = move |cx: &mut ElementContext<'_, H>| {
                let mut out = Vec::new();
                out.extend(children);
                if let Some(label) = label {
                    out.push(
                        ui::label(cx, label)
                            .text_color(ColorRef::Color(fg))
                            .into_element(cx),
                    );
                }

                vec![hstack(
                    cx,
                    HStackProps::default()
                        .gap(Space::N2)
                        .justify_center()
                        .items_center(),
                    move |_cx| out,
                )]
            };

            (pressable_props, chrome_props, content)
        })
    }
}
