use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, Rect, Size};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack::{HStackProps, hstack};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::switch::{
    switch_a11y, switch_checked_from_optional_bool, toggle_optional_bool,
};
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Space, WidgetState,
    WidgetStateProperty, WidgetStates, ui,
};

fn token(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

#[derive(Debug, Clone, Default)]
pub struct SwitchStyle {
    pub track_background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub thumb_background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub outline_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub label_color: Option<WidgetStateProperty<Option<ColorRef>>>,
}

impl SwitchStyle {
    pub fn track_background(
        mut self,
        track_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.track_background = Some(track_background);
        self
    }

    pub fn thumb_background(
        mut self,
        thumb_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.thumb_background = Some(thumb_background);
        self
    }

    pub fn outline_color(mut self, outline_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.outline_color = Some(outline_color);
        self
    }

    pub fn label_color(mut self, label_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(label_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.track_background.is_some() {
            self.track_background = other.track_background;
        }
        if other.thumb_background.is_some() {
            self.thumb_background = other.thumb_background;
        }
        if other.outline_color.is_some() {
            self.outline_color = other.outline_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        self
    }
}

#[derive(Debug, Clone)]
enum SwitchModel {
    Bool(Model<bool>),
    OptionalBool(Model<Option<bool>>),
}

#[derive(Clone)]
pub struct Switch {
    model: SwitchModel,
    label: Option<Arc<str>>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_click: Option<CommandId>,
    style: SwitchStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Switch {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            model: SwitchModel::Bool(model),
            label: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
            style: SwitchStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn new_optional(model: Model<Option<bool>>) -> Self {
        Self {
            model: SwitchModel::OptionalBool(model),
            label: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
            style: SwitchStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn on_click(mut self, command: CommandId) -> Self {
        self.on_click = Some(command);
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

    pub fn style(mut self, style: SwitchStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let track_h = theme
            .metric_by_key("material3.switch.track_h")
            .unwrap_or(Px(32.0));
        let track_w = theme
            .metric_by_key("material3.switch.track_w")
            .unwrap_or(Px(52.0));
        let thumb = theme
            .metric_by_key("material3.switch.thumb")
            .unwrap_or(Px(24.0));
        let pad = theme
            .metric_by_key("material3.switch.padding")
            .unwrap_or(Px(4.0));

        let radius = Px((track_h.0 * 0.5).max(0.0));
        let focus_bounds = Rect::new(
            fret_core::Point::new(
                MetricRef::space(Space::N2).resolve(&theme),
                MetricRef::space(Space::N1).resolve(&theme),
            ),
            Size::new(track_w, track_h),
        );

        let ring_border = theme.color_required("ring");
        let mut ring = decl_style::focus_ring(&theme, radius);
        ring.color = ring_border;

        let default_track_bg = WidgetStateProperty::new(Some(token(
            "material3.switch.track.off",
            ColorFallback::ThemePanelBackground,
        )))
        .when(
            WidgetStates::SELECTED,
            Some(token(
                "material3.switch.track.on",
                ColorFallback::ThemeAccent,
            )),
        )
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.switch.track.disabled",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "card",
                    mul: 0.38,
                },
            )),
        );

        let default_thumb_bg = WidgetStateProperty::new(Some(token(
            "material3.switch.thumb.off",
            ColorFallback::ThemeSurfaceBackground,
        )))
        .when(
            WidgetStates::SELECTED,
            Some(token(
                "material3.switch.thumb.on",
                ColorFallback::ThemePanelBackground,
            )),
        )
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.switch.thumb.disabled",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "background",
                    mul: 0.38,
                },
            )),
        );

        let default_outline = WidgetStateProperty::new(Some(token(
            "material3.switch.outline",
            ColorFallback::ThemePanelBorder,
        )))
        .when(
            WidgetStates::FOCUS_VISIBLE,
            Some(token(
                "material3.switch.focus.outline",
                ColorFallback::ThemeFocusRing,
            )),
        )
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.switch.disabled.outline",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "border",
                    mul: 0.38,
                },
            )),
        );

        let default_label_color = WidgetStateProperty::new(Some(token(
            "material3.switch.label",
            ColorFallback::ThemeTextPrimary,
        )))
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.switch.disabled.label",
                ColorFallback::ThemeTextDisabled,
            )),
        );

        let chrome = self.chrome;
        let style_override = self.style;
        let label = self.label;
        let a11y_label = self.a11y_label;
        let test_id = self.test_id;
        let on_click = self.on_click;
        let disabled_explicit = self.disabled;
        let model = self.model;

        let disabled = disabled_explicit
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));

        let layout = LayoutRefinement::default().merge(self.layout);
        let pressable_layout = decl_style::layout_style(&theme, layout);

        control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click);
            match &model {
                SwitchModel::Bool(model) => cx.pressable_toggle_bool(model),
                SwitchModel::OptionalBool(model) => {
                    cx.pressable_update_model(model, |v| *v = toggle_optional_bool(*v));
                }
            }

            let theme = Theme::global(&*cx.app).clone();
            let checked = match &model {
                SwitchModel::Bool(model) => cx.watch_model(model).copied().unwrap_or(false),
                SwitchModel::OptionalBool(model) => {
                    switch_checked_from_optional_bool(cx.watch_model(model).copied().flatten())
                }
            };

            let mut states = WidgetStates::from_pressable(cx, st, !disabled);
            states.set(WidgetState::Selected, checked);

            let track_bg = style_override
                .track_background
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_track_bg.resolve(states).clone())
                .map(|c| c.resolve(&theme))
                .unwrap_or(Color::TRANSPARENT);
            let thumb_bg = style_override
                .thumb_background
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_thumb_bg.resolve(states).clone())
                .map(|c| c.resolve(&theme))
                .unwrap_or(Color::TRANSPARENT);
            let outline = style_override
                .outline_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_outline.resolve(states).clone())
                .map(|c| c.resolve(&theme))
                .unwrap_or(Color::TRANSPARENT);
            let label_color = style_override
                .label_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_label_color.resolve(states).clone())
                .map(|c| c.resolve(&theme))
                .unwrap_or(theme.color_required("foreground"));

            let mut chrome_props =
                decl_style::container_props(&theme, chrome.clone(), LayoutRefinement::default());
            chrome_props.padding = Edges {
                top: MetricRef::space(Space::N1).resolve(&theme),
                right: MetricRef::space(Space::N2).resolve(&theme),
                bottom: MetricRef::space(Space::N1).resolve(&theme),
                left: MetricRef::space(Space::N2).resolve(&theme),
            };
            chrome_props.layout.size = pressable_layout.size;

            let mut a11y = switch_a11y(a11y_label.clone().or_else(|| label.clone()), checked);
            a11y.test_id = test_id.clone();

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: !disabled,
                focusable: !disabled,
                focus_ring: Some(ring),
                focus_ring_bounds: Some(focus_bounds),
                a11y,
                ..Default::default()
            };

            let children = move |cx: &mut ElementContext<'_, H>| {
                let track_layout = LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(track_w),
                        height: Length::Px(track_h),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let track_props = ContainerProps {
                    layout: track_layout,
                    padding: Edges::all(Px(0.0)),
                    background: Some(track_bg),
                    shadow: None,
                    border: Edges::all(Px(1.0)),
                    border_color: Some(outline),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                };

                let pad_y = Px(((track_h.0 - thumb.0) * 0.5).max(0.0));
                let x = if checked {
                    Px((track_w.0 - pad.0 - thumb.0).max(0.0))
                } else {
                    pad
                };

                let thumb_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: Some(pad_y),
                        left: Some(x),
                        ..Default::default()
                    },
                    size: SizeStyle {
                        width: Length::Px(thumb),
                        height: Length::Px(thumb),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let thumb_props = ContainerProps {
                    layout: thumb_layout,
                    padding: Edges::all(Px(0.0)),
                    background: Some(thumb_bg),
                    shadow: Some(decl_style::shadow_xs(&theme, Px(thumb.0 * 0.5))),
                    border: Edges::all(Px(0.0)),
                    border_color: None,
                    corner_radii: Corners::all(Px(thumb.0 * 0.5)),
                    ..Default::default()
                };

                let track_el = cx.container(track_props, move |cx| {
                    vec![cx.container(thumb_props, |_| Vec::new())]
                });

                let mut row_children = Vec::new();
                row_children.push(track_el);
                if let Some(label) = label.clone() {
                    row_children.push(
                        ui::label(cx, label)
                            .text_color(ColorRef::Color(label_color))
                            .into_element(cx),
                    );
                }

                vec![hstack(
                    cx,
                    HStackProps::default().gap_x(Space::N2).items_center(),
                    move |_cx| row_children,
                )]
            };

            (pressable_props, chrome_props, children)
        })
    }
}
