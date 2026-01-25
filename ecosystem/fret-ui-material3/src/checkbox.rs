use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, Rect, Size};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, ContainerProps, PressableProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack::{HStackProps, hstack};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::checkbox::{
    CheckedState, checkbox_a11y, checked_state_from_optional_bool, toggle_optional_bool,
};
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Space, WidgetState,
    WidgetStateProperty, WidgetStates, ui,
};

fn token(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

#[derive(Debug, Clone, Default)]
pub struct CheckboxStyle {
    pub container_background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub outline_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub indicator_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub label_color: Option<WidgetStateProperty<Option<ColorRef>>>,
}

impl CheckboxStyle {
    pub fn container_background(
        mut self,
        container_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(container_background);
        self
    }

    pub fn outline_color(mut self, outline_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.outline_color = Some(outline_color);
        self
    }

    pub fn indicator_color(
        mut self,
        indicator_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.indicator_color = Some(indicator_color);
        self
    }

    pub fn label_color(mut self, label_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(label_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.container_background.is_some() {
            self.container_background = other.container_background;
        }
        if other.outline_color.is_some() {
            self.outline_color = other.outline_color;
        }
        if other.indicator_color.is_some() {
            self.indicator_color = other.indicator_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        self
    }
}

#[derive(Debug, Clone)]
enum CheckboxCheckedModel {
    Bool(Model<bool>),
    OptionalBool(Model<Option<bool>>),
    TriState(Model<CheckedState>),
}

#[derive(Clone)]
pub struct Checkbox {
    checked: CheckboxCheckedModel,
    label: Option<Arc<str>>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_click: Option<CommandId>,
    style: CheckboxStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Checkbox {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            checked: CheckboxCheckedModel::Bool(model),
            label: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
            style: CheckboxStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn new_optional(model: Model<Option<bool>>) -> Self {
        Self {
            checked: CheckboxCheckedModel::OptionalBool(model),
            label: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
            style: CheckboxStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn new_tristate(model: Model<CheckedState>) -> Self {
        Self {
            checked: CheckboxCheckedModel::TriState(model),
            label: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
            style: CheckboxStyle::default(),
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

    pub fn style(mut self, style: CheckboxStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let size = theme
            .metric_by_key("material3.checkbox.size")
            .unwrap_or(Px(18.0));
        let radius = theme
            .metric_by_key("material3.checkbox.radius")
            .unwrap_or(Px(4.0));

        let pad_x = MetricRef::space(Space::N2).resolve(&theme);
        let pad_y = MetricRef::space(Space::N1).resolve(&theme);

        let focus_bounds = Rect::new(fret_core::Point::new(pad_x, pad_y), Size::new(size, size));

        let ring_border = theme.color_required("ring");
        let mut ring = decl_style::focus_ring(&theme, radius);
        ring.color = ring_border;

        let default_container_bg = WidgetStateProperty::new(None)
            .when(
                WidgetStates::HOVERED,
                Some(token(
                    "material3.checkbox.state_layer.hover",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "accent",
                        mul: 0.08,
                    },
                )),
            )
            .when(
                WidgetStates::ACTIVE,
                Some(token(
                    "material3.checkbox.state_layer.pressed",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "accent",
                        mul: 0.12,
                    },
                )),
            )
            .when(
                WidgetStates::SELECTED,
                Some(token(
                    "material3.checkbox.selected.container",
                    ColorFallback::ThemeAccent,
                )),
            )
            .when(WidgetStates::DISABLED, None);

        let default_outline = WidgetStateProperty::new(Some(token(
            "material3.checkbox.outline",
            ColorFallback::ThemePanelBorder,
        )))
        .when(
            WidgetStates::SELECTED,
            Some(token(
                "material3.checkbox.selected.outline",
                ColorFallback::ThemeAccent,
            )),
        )
        .when(
            WidgetStates::FOCUS_VISIBLE,
            Some(token(
                "material3.checkbox.focus.outline",
                ColorFallback::ThemeFocusRing,
            )),
        )
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.checkbox.disabled.outline",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "border",
                    mul: 0.38,
                },
            )),
        );

        let default_indicator = WidgetStateProperty::new(None)
            .when(
                WidgetStates::SELECTED,
                Some(token(
                    "material3.checkbox.selected.indicator",
                    ColorFallback::ThemeTextPrimary,
                )),
            )
            .when(WidgetStates::DISABLED, None);

        let default_label_color = WidgetStateProperty::new(Some(token(
            "material3.checkbox.label",
            ColorFallback::ThemeTextPrimary,
        )))
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.checkbox.disabled.label",
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
        let checked = self.checked;

        let disabled = disabled_explicit
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));

        let layout = LayoutRefinement::default().merge(self.layout);
        let pressable_layout = decl_style::layout_style(&theme, layout);

        control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click);

            match &checked {
                CheckboxCheckedModel::Bool(model) => cx.pressable_toggle_bool(model),
                CheckboxCheckedModel::OptionalBool(model) => {
                    cx.pressable_update_model(model, |v| *v = toggle_optional_bool(*v));
                }
                CheckboxCheckedModel::TriState(model) => {
                    cx.pressable_update_model(model, |v| *v = v.toggle());
                }
            }

            let theme = Theme::global(&*cx.app).clone();
            let state = match &checked {
                CheckboxCheckedModel::Bool(model) => {
                    CheckedState::from(cx.watch_model(model).copied().unwrap_or(false))
                }
                CheckboxCheckedModel::OptionalBool(model) => {
                    checked_state_from_optional_bool(cx.watch_model(model).copied().flatten())
                }
                CheckboxCheckedModel::TriState(model) => {
                    cx.watch_model(model).copied().unwrap_or_default()
                }
            };

            let mut states = WidgetStates::from_pressable(cx, st, !disabled);
            states.set(WidgetState::Selected, state.is_on());

            let bg = style_override
                .container_background
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_container_bg.resolve(states).clone())
                .map(|c| c.resolve(&theme));
            let outline = style_override
                .outline_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_outline.resolve(states).clone())
                .map(|c| c.resolve(&theme))
                .unwrap_or(Color::TRANSPARENT);
            let indicator = style_override
                .indicator_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_indicator.resolve(states).clone())
                .map(|c| c.resolve(&theme));
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
                top: pad_y,
                right: pad_x,
                bottom: pad_y,
                left: pad_x,
            };
            chrome_props.layout.size = pressable_layout.size;

            let mut a11y = checkbox_a11y(a11y_label.clone().or_else(|| label.clone()), state);
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
                let box_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(size))
                        .h_px(MetricRef::Px(size)),
                );
                let box_props = ContainerProps {
                    layout: box_layout,
                    padding: Edges::all(Px(0.0)),
                    background: bg,
                    shadow: None,
                    border: Edges::all(Px(1.0)),
                    border_color: Some(outline),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                };

                let mut children = Vec::new();
                if let Some(indicator) = indicator {
                    children.push(
                        ui::label(cx, Arc::from("✓"))
                            .text_color(ColorRef::Color(indicator))
                            .into_element(cx),
                    );
                }

                let box_el = cx.container(box_props, move |cx| {
                    vec![hstack(
                        cx,
                        HStackProps::default().justify_center().items_center(),
                        move |_cx| children,
                    )]
                });

                let mut row_children = Vec::new();
                row_children.push(box_el);
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
