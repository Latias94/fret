use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole, Size};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, PressableState, ScrollProps, SizeStyle,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::{popper, popper_content, select as radix_select};
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, OverlayPresence,
    Radius, Space, WidgetState, WidgetStateProperty, WidgetStates, ui,
};

#[derive(Debug, Clone)]
pub struct SelectItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl SelectItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectStyle {
    pub trigger_background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub trigger_foreground: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub trigger_border_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub option_background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub option_foreground: Option<WidgetStateProperty<Option<ColorRef>>>,
}

impl SelectStyle {
    pub fn trigger_background(mut self, background: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_background = Some(background);
        self
    }

    pub fn trigger_foreground(mut self, foreground: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_foreground = Some(foreground);
        self
    }

    pub fn trigger_border_color(mut self, border: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_border_color = Some(border);
        self
    }

    pub fn option_background(mut self, background: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.option_background = Some(background);
        self
    }

    pub fn option_foreground(mut self, foreground: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.option_foreground = Some(foreground);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.trigger_background.is_some() {
            self.trigger_background = other.trigger_background;
        }
        if other.trigger_foreground.is_some() {
            self.trigger_foreground = other.trigger_foreground;
        }
        if other.trigger_border_color.is_some() {
            self.trigger_border_color = other.trigger_border_color;
        }
        if other.option_background.is_some() {
            self.option_background = other.option_background;
        }
        if other.option_foreground.is_some() {
            self.option_foreground = other.option_foreground;
        }
        self
    }
}

fn token(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

fn default_style() -> (
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
) {
    let trigger_bg = WidgetStateProperty::new(Some(token(
        "material3.select.trigger.container",
        ColorFallback::ThemeSurfaceBackground,
    )))
    .when(
        WidgetStates::DISABLED,
        Some(token(
            "material3.select.trigger.disabled.container",
            ColorFallback::ThemeTokenAlphaMul {
                key: "panel.background",
                mul: 0.38,
            },
        )),
    );

    let trigger_fg = WidgetStateProperty::new(Some(token(
        "material3.select.trigger.label",
        ColorFallback::ThemeTextPrimary,
    )))
    .when(
        WidgetStates::DISABLED,
        Some(token(
            "material3.select.trigger.disabled.label",
            ColorFallback::ThemeTextDisabled,
        )),
    );

    let trigger_border = WidgetStateProperty::new(Some(token(
        "material3.select.trigger.outline",
        ColorFallback::ThemePanelBorder,
    )))
    .when(
        WidgetStates::FOCUS_VISIBLE,
        Some(token(
            "material3.select.trigger.focus.outline",
            ColorFallback::ThemeAccent,
        )),
    )
    .when(
        WidgetStates::OPEN,
        Some(token(
            "material3.select.trigger.open.outline",
            ColorFallback::ThemeAccent,
        )),
    )
    .when(
        WidgetStates::DISABLED,
        Some(token(
            "material3.select.trigger.disabled.outline",
            ColorFallback::ThemeTokenAlphaMul {
                key: "border",
                mul: 0.38,
            },
        )),
    );

    let option_bg = WidgetStateProperty::new(None)
        .when(
            WidgetStates::HOVERED,
            Some(token(
                "material3.select.option.state_layer.hover",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "accent",
                    mul: 0.08,
                },
            )),
        )
        .when(
            WidgetStates::ACTIVE,
            Some(token(
                "material3.select.option.state_layer.pressed",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "accent",
                    mul: 0.12,
                },
            )),
        )
        .when(
            WidgetStates::SELECTED,
            Some(token(
                "material3.select.option.state_layer.selected",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "accent",
                    mul: 0.12,
                },
            )),
        );

    let option_fg = WidgetStateProperty::new(Some(token(
        "material3.select.option.label",
        ColorFallback::ThemeTextPrimary,
    )))
    .when(
        WidgetStates::DISABLED,
        Some(token(
            "material3.select.option.disabled.label",
            ColorFallback::ThemeTextDisabled,
        )),
    );

    (trigger_bg, trigger_fg, trigger_border, option_bg, option_fg)
}

#[derive(Clone)]
pub struct Select {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    placeholder: Arc<str>,
    disabled: bool,
    items: Vec<SelectItem>,
    a11y_label: Option<Arc<str>>,
    on_dismiss_request: Option<OnDismissRequest>,
    style: SelectStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    side: Side,
    align: Align,
}

impl Select {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            placeholder: Arc::from("Select..."),
            disabled: false,
            items: Vec::new(),
            a11y_label: None,
            on_dismiss_request: None,
            style: SelectStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            side: Side::Bottom,
            align: Align::Start,
        }
    }

    pub fn new_controllable<H: UiHost, T: Into<Arc<str>>>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<Option<Arc<str>>>>,
        default_value: Option<T>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let default_value: Option<Arc<str>> = default_value.map(Into::into);
        let model =
            radix_select::select_use_value_model(cx, value, || default_value.clone()).model();

        let open = radix_select::SelectRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);

        Self::new(model, open)
    }

    pub fn item(mut self, item: SelectItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
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

    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    pub fn style(mut self, style: SelectStyle) -> Self {
        self.style = self.style.merged(style);
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

    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let (
            default_trigger_bg,
            default_trigger_fg,
            default_trigger_border,
            default_option_bg,
            default_option_fg,
        ) = default_style();
        let style_override = self.style;

        let disabled = self.disabled;
        let model = self.model;
        let open = self.open;
        let placeholder = self.placeholder;
        let items = self.items;
        let a11y_label = self.a11y_label;
        let on_dismiss_request = self.on_dismiss_request;
        let user_chrome = self.chrome;
        let side = self.side;
        let align = self.align;

        let layout = LayoutRefinement::default()
            .min_h(MetricRef::Px(Px(40.0)))
            .merge(self.layout);
        let pressable_layout = decl_style::layout_style(&theme, layout);

        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        let mut ring = decl_style::focus_ring(&theme, radius);
        ring.color = theme.color_required("ring");

        control_chrome_pressable_with_id_props(cx, move |cx, st: PressableState, trigger_id| {
            let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);

            cx.pressable_on_activate({
                let open = open.clone();
                Arc::new(move |host, action_cx: ActionCx, _reason| {
                    let _ = host.models_mut().update(&open, |v| *v = !*v);
                    host.request_redraw(action_cx.window);
                })
            });

            let mut states = WidgetStates::from_pressable(cx, st, !disabled);
            states.set(WidgetState::Open, is_open);

            let trigger_bg_ref = style_override
                .trigger_background
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_trigger_bg.resolve(states).clone());
            let trigger_fg_ref = style_override
                .trigger_foreground
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_trigger_fg.resolve(states).clone());
            let trigger_border_ref = style_override
                .trigger_border_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_trigger_border.resolve(states).clone());

            let trigger_bg = trigger_bg_ref.map(|c| c.resolve(&theme));
            let trigger_fg = trigger_fg_ref
                .unwrap_or(ColorRef::Color(theme.color_required("foreground")))
                .resolve(&theme);
            let trigger_border = trigger_border_ref.map(|c| c.resolve(&theme));

            let padding = ChromeRefinement::default().px(Space::N3).py(Space::N2);
            let mut chrome = padding
                .merge(ChromeRefinement {
                    radius: Some(MetricRef::Px(radius)),
                    border_width: Some(MetricRef::Px(Px(1.0))),
                    ..Default::default()
                })
                .merge(user_chrome.clone());
            if chrome.background.is_none() {
                chrome.background = trigger_bg.map(ColorRef::Color);
            }
            if chrome.border_color.is_none() {
                chrome.border_color = trigger_border.map(ColorRef::Color);
            }
            let chrome_props =
                decl_style::container_props(&theme, chrome, LayoutRefinement::default());

            let overlay_root_name = radix_select::select_root_name(trigger_id);
            let listbox_id_for_trigger =
                radix_select::select_listbox_semantics_id(cx, overlay_root_name.as_str());

            if is_open && !disabled {
                let overlay_presence = OverlayPresence::instant(true);

                let open_for_overlay_children = open.clone();
                let model_for_overlay_children = model.clone();
                let items_for_overlay_children = items.clone();
                let style_for_overlay_children = style_override.clone();
                let on_dismiss_request_for_overlay_children = on_dismiss_request.clone();
                let theme_for_overlay = theme.clone();

                let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                    let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id) else {
                        return vec![radix_select::select_modal_barrier_with_dismiss_handler(
                            cx,
                            open_for_overlay_children.clone(),
                            true,
                            on_dismiss_request_for_overlay_children.clone(),
                            Vec::new(),
                        )];
                    };

                    let window_margin = Px(8.0);
                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                    let item_h = Px(40.0);
                    let max_h = Px(240.0);
                    let item_count = items_for_overlay_children.len().max(1) as f32;
                    let desired_h = Px((item_h.0 * item_count).min(max_h.0).max(item_h.0));

                    let desired = Size::new(anchor.size.width, desired_h);
                    let placement = popper::PopperContentPlacement::new(
                        popper::LayoutDirection::Ltr,
                        side,
                        align,
                        Px(4.0),
                    )
                    .with_collision_padding(Edges::all(window_margin));
                    let layout =
                        popper::popper_content_layout_sized(outer, anchor, desired, placement);
                    let placed = layout.rect;
                    let open_for_layer_children = open_for_overlay_children.clone();
                    let open_for_items_children = open_for_overlay_children.clone();

                    let content = popper_content::popper_wrapper_panel_at(
                        cx,
                        placed,
                        Edges::all(Px(0.0)),
                        Overflow::Visible,
                        move |cx| {
                            let shadow = decl_style::shadow_md(&theme_for_overlay, radius);
                            let panel_bg = theme_for_overlay.colors.panel_background;
                            let panel_border = theme_for_overlay.colors.panel_border;

                            let panel = cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Fill;
                                        layout.overflow = Overflow::Clip;
                                        layout
                                    },
                                    padding: Edges::all(Px(4.0)),
                                    background: Some(panel_bg),
                                    shadow: Some(shadow),
                                    border: Edges::all(Px(1.0)),
                                    border_color: Some(panel_border),
                                    corner_radii: Corners::all(radius),
                                    ..Default::default()
                                },
                                move |cx| {
                                    let list_layout = {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Fill;
                                        layout
                                    };
                                    let scroll = cx.scroll(
                                        ScrollProps {
                                            layout: list_layout,
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![cx.flex(
                                                FlexProps {
                                                    layout: {
                                                        let mut layout = LayoutStyle::default();
                                                        layout.size.width = Length::Fill;
                                                        layout
                                                    },
                                                    direction: fret_core::Axis::Vertical,
                                                    gap: Px(0.0),
                                                    padding: Edges::all(Px(0.0)),
                                                    justify: MainAlign::Start,
                                                    align: CrossAlign::Stretch,
                                                    wrap: false,
                                                },
                                                |cx| {
                                                    let selected = cx
                                                        .watch_model(&model_for_overlay_children)
                                                        .layout()
                                                        .cloned()
                                                        .flatten();
                                                let open_for_items =
                                                    open_for_items_children.clone();
                                                    let model_for_items =
                                                        model_for_overlay_children.clone();
                                                    let style_for_items =
                                                        style_for_overlay_children.clone();
                                                    let default_option_bg_prop =
                                                        default_option_bg.clone();
                                                    let default_option_fg_prop =
                                                        default_option_fg.clone();
                                                    let theme_for_items = theme_for_overlay.clone();

                                                    items_for_overlay_children
                                                        .iter()
                                                        .cloned()
                                                        .map(|item| {
                                                            let item_enabled =
                                                                !item.disabled && !disabled;
                                                            let item_value = item.value.clone();
                                                            let item_label = item.label.clone();
                                                            let selected = selected.clone();
                                                            let style_for_item =
                                                                style_for_items.clone();
                                                            let default_option_bg =
                                                                default_option_bg_prop.clone();
                                                            let default_option_fg =
                                                                default_option_fg_prop.clone();
                                                            let theme_for_item =
                                                                theme_for_items.clone();
                                                            let model_for_item =
                                                                model_for_items.clone();
                                                            let open_for_item =
                                                                open_for_items.clone();

                                                            cx.pressable(
                                                                PressableProps {
                                                                    layout: {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.size = SizeStyle {
                                                                            width: Length::Fill,
                                                                            min_height: Some(item_h),
                                                                            ..Default::default()
                                                                        };
                                                                        layout
                                                                    },
                                                                    enabled: item_enabled,
                                                                    focusable: item_enabled,
                                                                    a11y: PressableA11y {
                                                                        role: Some(SemanticsRole::ListBoxOption),
                                                                        label: Some(item_label.clone()),
                                                                        ..Default::default()
                                                                    },
                                                                    ..Default::default()
                                                                },
                                                                move |cx, st| {
                                                                    let mut states =
                                                                        WidgetStates::from_pressable(
                                                                            cx,
                                                                            st,
                                                                            item_enabled,
                                                                        );
                                                                    states.set(
                                                                        WidgetState::Selected,
                                                                        selected.as_ref().is_some_and(
                                                                            |v: &Arc<str>| {
                                                                                v.as_ref()
                                                                                    == item_value.as_ref()
                                                                            },
                                                                        ),
                                                                    );

                                                                    let bg_ref = style_for_item
                                                                        .option_background
                                                                        .as_ref()
                                                                        .and_then(|p| p.resolve(states).clone())
                                                                        .or_else(|| {
                                                                            default_option_bg
                                                                                .resolve(states)
                                                                                .clone()
                                                                        });
                                                                    let fg_ref = style_for_item
                                                                        .option_foreground
                                                                        .as_ref()
                                                                        .and_then(|p| p.resolve(states).clone())
                                                                        .or_else(|| {
                                                                            default_option_fg
                                                                                .resolve(states)
                                                                                .clone()
                                                                        });

                                                                    let bg =
                                                                        bg_ref.map(|c| c.resolve(&theme_for_item));
                                                                    let fg = fg_ref
                                                                        .unwrap_or(ColorRef::Color(
                                                                            theme_for_item.color_required("foreground"),
                                                                        ))
                                                                        .resolve(&theme_for_item);

                                                                    cx.pressable_on_activate({
                                                                        let model = model_for_item.clone();
                                                                        let open = open_for_item.clone();
                                                                        let item_value = item_value.clone();
                                                                        Arc::new(move |host, action_cx, _reason| {
                                                                            let _ = host
                                                                                .models_mut()
                                                                                .update(&model, |v| {
                                                                                    *v = Some(item_value.clone());
                                                                                });
                                                                            let _ = host
                                                                                .models_mut()
                                                                                .update(&open, |v| *v = false);
                                                                            host.request_redraw(action_cx.window);
                                                                        })
                                                                    });

                                                                    let row = cx.container(
                                                                        ContainerProps {
                                                                            layout: {
                                                                                let mut layout =
                                                                                    LayoutStyle::default();
                                                                                layout.size.width =
                                                                                    Length::Fill;
                                                                                layout.size.height =
                                                                                    Length::Fill;
                                                                                layout
                                                                            },
                                                                            padding: Edges {
                                                                                top: Px(0.0),
                                                                                right: Px(12.0),
                                                                                bottom: Px(0.0),
                                                                                left: Px(12.0),
                                                                            },
                                                                            background: bg,
                                                                            ..Default::default()
                                                                        },
                                                                        move |cx| {
                                                                            vec![ui::label(cx, item_label.clone())
                                                                                .text_color(ColorRef::Color(fg))
                                                                                .into_element(cx)]
                                                                        },
                                                                    );
                                                                    vec![row]
                                                                },
                                                            )
                                                        })
                                                        .collect::<Vec<_>>()
                                                },
                                            )]
                                        },
                                    );
                                    vec![scroll]
                                },
                            );

                            vec![radix_select::select_listbox_pressable_with_id_props(
                                cx,
                                move |_cx, _st, _id| {
                                    (
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Fill;
                                                layout.size.height = Length::Fill;
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::ListBox),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        vec![panel],
                                    )
                                },
                            )]
                        },
                    );

                    radix_select::select_modal_layer_children_with_dismiss_handler(
                        cx,
                        open_for_layer_children.clone(),
                        true,
                        on_dismiss_request_for_overlay_children.clone(),
                        Vec::new(),
                        content,
                    )
                });

                let mut request = radix_select::modal_select_request_with_dismiss_handler(
                    trigger_id,
                    trigger_id,
                    open.clone(),
                    overlay_presence,
                    on_dismiss_request.clone(),
                    overlay_children,
                );
                request.initial_focus = Some(listbox_id_for_trigger);
                radix_select::request_select(cx, request);
            }

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: !disabled,
                focusable: !disabled,
                focus_ring: Some(ring),
                a11y: radix_select::select_trigger_a11y(
                    a11y_label.clone(),
                    is_open,
                    Some(listbox_id_for_trigger),
                ),
                ..Default::default()
            };

            let content = move |cx: &mut ElementContext<'_, H>| {
                let selected = cx.watch_model(&model).layout().cloned().flatten();
                let label = selected
                    .as_ref()
                    .and_then(|v: &Arc<str>| items.iter().find(|i| i.value.as_ref() == v.as_ref()))
                    .map(|i| i.label.clone())
                    .unwrap_or_else(|| placeholder.clone());

                vec![
                    ui::label(cx, label)
                        .text_color(ColorRef::Color(trigger_fg))
                        .into_element(cx),
                ]
            };

            (pressable_props, chrome_props, content)
        })
    }
}
