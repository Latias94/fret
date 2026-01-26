use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, PressableProps, RovingFlexProps, RovingFocusProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack::{HStackProps, hstack};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::radio_group::{self as radio, RadioGroupOrientation, RadioGroupRoot};
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Radius, Space,
    WidgetState, WidgetStateProperty, WidgetStates, ui,
};

fn token(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

#[derive(Debug, Clone, Default)]
pub struct RadioGroupStyle {
    pub item_background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub icon_outline_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub label_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub indicator_color: Option<WidgetStateProperty<Option<ColorRef>>>,
}

impl RadioGroupStyle {
    pub fn item_background(
        mut self,
        item_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.item_background = Some(item_background);
        self
    }

    pub fn icon_outline_color(
        mut self,
        icon_outline_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.icon_outline_color = Some(icon_outline_color);
        self
    }

    pub fn label_color(mut self, label_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(label_color);
        self
    }

    pub fn indicator_color(
        mut self,
        indicator_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.indicator_color = Some(indicator_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.item_background.is_some() {
            self.item_background = other.item_background;
        }
        if other.icon_outline_color.is_some() {
            self.icon_outline_color = other.icon_outline_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.indicator_color.is_some() {
            self.indicator_color = other.indicator_color;
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct RadioGroupItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl RadioGroupItem {
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

#[derive(Clone)]
pub struct RadioGroup {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<RadioGroupItem>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    orientation: radio::RadioGroupOrientation,
    loop_navigation: bool,
    style: RadioGroupStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self {
            model: None,
            default_value: None,
            items: Vec::new(),
            disabled: false,
            a11y_label: None,
            orientation: radio::RadioGroupOrientation::default(),
            loop_navigation: true,
            style: RadioGroupStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }
}

impl RadioGroup {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: Some(model),
            ..Default::default()
        }
    }

    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            default_value: default_value.map(Into::into),
            ..Default::default()
        }
    }

    pub fn item(mut self, item: RadioGroupItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = RadioGroupItem>) -> Self {
        self.items.extend(items);
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

    pub fn orientation(mut self, orientation: radio::RadioGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
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

    pub fn style(mut self, style: RadioGroupStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let orientation = self.orientation;
        let loop_navigation = self.loop_navigation;
        let chrome = self.chrome;
        let layout = self.layout;

        let icon = theme
            .metric_by_key("material3.radio.icon")
            .unwrap_or(Px(20.0));
        let icon_r = Px((icon.0 * 0.5).max(0.0));
        let pad_x = MetricRef::space(Space::N2).resolve(&theme);
        let pad_y = MetricRef::space(Space::N1).resolve(&theme);
        let gap_x = MetricRef::space(Space::N2).resolve(&theme);
        let gap_y = MetricRef::space(Space::N1).resolve(&theme);

        let ring_border = theme.color_required("ring");
        let mut ring_style = decl_style::focus_ring(&theme, icon_r);
        ring_style.color = ring_border;

        let focus_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(icon, icon));

        let default_item_bg = WidgetStateProperty::new(None)
            .when(
                WidgetStates::HOVERED,
                Some(token(
                    "material3.radio.state_layer.hover",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "accent",
                        mul: 0.08,
                    },
                )),
            )
            .when(
                WidgetStates::ACTIVE,
                Some(token(
                    "material3.radio.state_layer.pressed",
                    ColorFallback::ThemeTokenAlphaMul {
                        key: "accent",
                        mul: 0.12,
                    },
                )),
            )
            .when(WidgetStates::DISABLED, None);

        let default_icon_outline = WidgetStateProperty::new(Some(token(
            "material3.radio.outline",
            ColorFallback::ThemePanelBorder,
        )))
        .when(
            WidgetStates::SELECTED,
            Some(token(
                "material3.radio.selected.outline",
                ColorFallback::ThemeAccent,
            )),
        )
        .when(
            WidgetStates::FOCUS_VISIBLE,
            Some(token(
                "material3.radio.focus.outline",
                ColorFallback::ThemeFocusRing,
            )),
        )
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.radio.disabled.outline",
                ColorFallback::ThemeTokenAlphaMul {
                    key: "border",
                    mul: 0.38,
                },
            )),
        );

        let default_label_color = WidgetStateProperty::new(Some(token(
            "material3.radio.label",
            ColorFallback::ThemeTextPrimary,
        )))
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.radio.disabled.label",
                ColorFallback::ThemeTextDisabled,
            )),
        );

        let default_indicator = WidgetStateProperty::new(Some(token(
            "material3.radio.selected.indicator",
            ColorFallback::ThemeAccent,
        )))
        .when(
            WidgetStates::DISABLED,
            Some(token(
                "material3.radio.disabled.indicator",
                ColorFallback::ThemeTextDisabled,
            )),
        );

        let group_disabled = self.disabled;
        let style_override = self.style;
        let items = self.items;

        let model = radio::radio_group_use_model(cx, self.model, || self.default_value).model();
        let selected: Option<Arc<str>> = cx.watch_model(&model).cloned().flatten();

        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
        let disabled_flags: Vec<bool> =
            items.iter().map(|i| group_disabled || i.disabled).collect();

        let active = roving_focus_group::active_index_from_str_keys(
            &values,
            selected.as_deref(),
            &disabled_flags,
        );

        let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
        let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.clone().into_boxed_slice());
        let set_size = u32::try_from(items.len())
            .ok()
            .and_then(|n| (n > 0).then_some(n));

        let mut root = RadioGroupRoot::new(model)
            .disabled(group_disabled)
            .orientation(orientation)
            .loop_navigation(loop_navigation);
        if let Some(label) = self.a11y_label {
            root = root.a11y_label(label);
        }

        let root_for_items = root.clone();
        let list = root.list(values_arc, disabled_arc);

        let list_theme = theme.clone();
        let list_element = list.into_element(
            cx,
            RovingFlexProps {
                flex: FlexProps {
                    gap: match orientation {
                        RadioGroupOrientation::Vertical => gap_y,
                        RadioGroupOrientation::Horizontal => gap_x,
                    },
                    padding: Edges::all(Px(0.0)),
                    ..Default::default()
                },
                roving: RovingFocusProps::default(),
            },
            move |cx| {
                let mut out = Vec::with_capacity(items.len());
                for (idx, item) in items.iter().cloned().enumerate() {
                    let item_disabled = disabled_flags.get(idx).copied().unwrap_or(true);
                    let item_enabled = !item_disabled;
                    let tab_stop = active.is_some_and(|a| a == idx);

                    let pressable_layout =
                        decl_style::layout_style(&list_theme, LayoutRefinement::default().w_full());

                    let root_for_item = root_for_items.clone();
                    let style_override = style_override.clone();
                    let default_item_bg = default_item_bg.clone();
                    let default_icon_outline = default_icon_outline.clone();
                    let default_label_color = default_label_color.clone();
                    let default_indicator = default_indicator.clone();

                    let ring_style = ring_style;
                    let focus_bounds = focus_bounds;
                    out.push(cx.keyed(item.value.clone(), move |cx| {
                        radio::RadioGroupItem::new(item.value.clone())
                            .label(item.label.clone())
                            .disabled(!item_enabled)
                            .index(idx)
                            .tab_stop(tab_stop)
                            .set_size(set_size)
                            .into_element(
                                cx,
                                &root_for_item,
                                PressableProps {
                                    layout: pressable_layout,
                                    enabled: item_enabled,
                                    focusable: tab_stop,
                                    focus_ring: Some(ring_style),
                                    focus_ring_bounds: Some(focus_bounds),
                                    ..Default::default()
                                },
                                move |cx, st, checked| {
                                    let theme = Theme::global(&*cx.app).clone();

                                    let mut states =
                                        WidgetStates::from_pressable(cx, st, item_enabled);
                                    states.set(WidgetState::Selected, checked);

                                    let item_bg = style_override
                                        .item_background
                                        .as_ref()
                                        .and_then(|p| p.resolve(states).clone())
                                        .or_else(|| default_item_bg.resolve(states).clone())
                                        .map(|c| c.resolve(&theme));
                                    let outline = style_override
                                        .icon_outline_color
                                        .as_ref()
                                        .and_then(|p| p.resolve(states).clone())
                                        .or_else(|| default_icon_outline.resolve(states).clone())
                                        .map(|c| c.resolve(&theme))
                                        .unwrap_or(Color::TRANSPARENT);
                                    let label_color = style_override
                                        .label_color
                                        .as_ref()
                                        .and_then(|p| p.resolve(states).clone())
                                        .or_else(|| default_label_color.resolve(states).clone())
                                        .map(|c| c.resolve(&theme))
                                        .unwrap_or(theme.color_required("foreground"));
                                    let indicator = style_override
                                        .indicator_color
                                        .as_ref()
                                        .and_then(|p| p.resolve(states).clone())
                                        .or_else(|| default_indicator.resolve(states).clone())
                                        .map(|c| c.resolve(&theme));

                                    let row_props = ContainerProps {
                                        padding: Edges {
                                            top: pad_y,
                                            right: pad_x,
                                            bottom: pad_y,
                                            left: pad_x,
                                        },
                                        background: item_bg,
                                        corner_radii: Corners::all(
                                            MetricRef::radius(Radius::Md).resolve(&theme),
                                        ),
                                        ..Default::default()
                                    };

                                    let icon_layout = decl_style::layout_style(
                                        &theme,
                                        LayoutRefinement::default()
                                            .w_px(MetricRef::Px(icon))
                                            .h_px(MetricRef::Px(icon)),
                                    );
                                    let icon_props = ContainerProps {
                                        layout: icon_layout,
                                        padding: Edges::all(Px(0.0)),
                                        background: None,
                                        shadow: None,
                                        border: Edges::all(Px(2.0)),
                                        border_color: Some(outline),
                                        corner_radii: Corners::all(icon_r),
                                        ..Default::default()
                                    };

                                    let dot_el = if checked {
                                        let dot_size = Px((icon.0 * 0.45).max(0.0));
                                        let dot_layout = decl_style::layout_style(
                                            &theme,
                                            LayoutRefinement::default()
                                                .w_px(MetricRef::Px(dot_size))
                                                .h_px(MetricRef::Px(dot_size)),
                                        );
                                        let dot_color =
                                            indicator.unwrap_or(theme.color_required("primary"));
                                        cx.container(
                                            ContainerProps {
                                                layout: dot_layout,
                                                background: Some(dot_color),
                                                corner_radii: Corners::all(Px(dot_size.0 * 0.5)),
                                                ..Default::default()
                                            },
                                            |_| Vec::new(),
                                        )
                                    } else {
                                        cx.container(ContainerProps::default(), |_| Vec::new())
                                    };

                                    let icon_el = cx.container(icon_props, move |cx| {
                                        vec![hstack(
                                            cx,
                                            HStackProps::default().justify_center().items_center(),
                                            move |_cx| vec![dot_el],
                                        )]
                                    });

                                    let label_el = ui::label(cx, item.label.clone())
                                        .text_color(ColorRef::Color(label_color))
                                        .into_element(cx);

                                    let row = hstack(
                                        cx,
                                        HStackProps::default().gap(Space::N2).items_center(),
                                        move |_cx| vec![icon_el, label_el],
                                    );

                                    vec![cx.container(row_props, move |_cx| vec![row])]
                                },
                            )
                    }));
                }
                out
            },
        );

        let container_props = decl_style::container_props(&theme, chrome, layout);
        cx.container(container_props, move |_cx| vec![list_element])
    }
}
