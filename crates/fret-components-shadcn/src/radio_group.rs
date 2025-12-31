use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::roving_focus;
use fret_components_ui::{MetricRef, Space};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, SemanticsProps, SizeStyle,
    TextProps,
};
use fret_ui::{ElementCx, Theme, UiHost};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn row_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.radio_group.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N3).resolve(theme))
}

fn label_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.radio_group.label_gap")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.radio_group.icon_size_px")
        .unwrap_or(Px(16.0))
}

fn indicator_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.radio_group.indicator_size_px")
        .unwrap_or(Px(8.0))
}

fn radio_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.radio_group.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.radio_group.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);

    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn radio_border(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or(theme.colors.panel_border)
}

fn radio_ring(theme: &Theme) -> Color {
    theme
        .color_by_key("ring")
        .or_else(|| theme.color_by_key("primary"))
        .unwrap_or(theme.colors.selection_background)
}

fn radio_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary)
}

fn radio_indicator(theme: &Theme) -> Color {
    theme.color_by_key("primary").unwrap_or(theme.colors.accent)
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
    model: Model<Option<Arc<str>>>,
    items: Vec<RadioGroupItem>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
}

impl RadioGroup {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            disabled: false,
            a11y_label: None,
        }
    }

    pub fn item(mut self, item: RadioGroupItem) -> Self {
        self.items.push(item);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            cx.observe_model(&self.model, Invalidation::Paint);

            let theme = Theme::global(&*cx.app).clone();
            let gap_y = row_gap(&theme);
            let gap_x = label_gap(&theme);
            let icon = icon_size(&theme);
            let indicator = indicator_size(&theme);

            let text_style = radio_text_style(&theme);
            let fg = radio_fg(&theme);
            let fg_disabled = theme.colors.text_disabled;
            let border = radio_border(&theme);
            let ring = radio_ring(&theme);
            let dot = radio_indicator(&theme);

            let group_disabled = self.disabled;
            let group_label = self.a11y_label.clone();
            let items = self.items.clone();
            let model = self.model;

            cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::List,
                    label: group_label.clone(),
                    disabled: group_disabled,
                    ..Default::default()
                },
                move |cx| {
                    let selected: Option<Arc<str>> = cx.app.models().get_cloned(&model).flatten();

                    let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
                    let disabled: Vec<bool> =
                        items.iter().map(|i| group_disabled || i.disabled).collect();
                    let active =
                        roving_focus::active_index_from_str_keys(&values, selected.as_deref(), &disabled);

                    let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
                    let roving = RovingFocusProps {
                        enabled: !group_disabled,
                        wrap: true,
                        disabled: Arc::from(disabled.clone().into_boxed_slice()),
                        ..Default::default()
                    };

                    vec![cx.roving_flex(
                        RovingFlexProps {
                            flex: FlexProps {
                                direction: fret_core::Axis::Vertical,
                                gap: gap_y,
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                                ..Default::default()
                            },
                            roving,
                        },
                        move |cx| {
                            cx.roving_select_option_arc_str(&model, values_arc.clone());

                            let mut out = Vec::with_capacity(items.len());
                            for (idx, item) in items.iter().cloned().enumerate() {
                                let item_disabled = disabled.get(idx).copied().unwrap_or(true);
                                let item_enabled = !item_disabled;
                                let tab_stop = active.is_some_and(|a| a == idx);
                                let is_selected =
                                    selected.as_deref().is_some_and(|v| v == item.value.as_ref());

                                let radius = Px((icon.0 * 0.5).max(0.0));
                                let ring_style = decl_style::focus_ring(&theme, radius);
                                let pressable_layout = decl_style::layout_style(
                                    &theme,
                                    fret_components_ui::LayoutRefinement::default().w_full(),
                                );

                                let a11y_label = item.label.clone();
                                let pressable_item_value = item.value.clone();
                                let model = model.clone();
                                let text_style = text_style.clone();
                                out.push(cx.pressable(
                                    PressableProps {
                                        layout: pressable_layout,
                                        enabled: item_enabled,
                                        focusable: tab_stop,
                                        focus_ring: Some(ring_style),
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::ListItem),
                                            label: Some(a11y_label.clone()),
                                            selected: is_selected,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, st| {
                                        cx.pressable_set_option_arc_str(
                                            &model,
                                            pressable_item_value.clone(),
                                        );

                                        let theme = Theme::global(&*cx.app).clone();

                                        let mut border_color =
                                            if is_selected { dot } else { border };
                                        if item_enabled && (st.hovered || st.pressed) {
                                            border_color = alpha_mul(ring, 0.8);
                                        }

                                        let mut fg = if item_enabled { fg } else { fg_disabled };
                                        let mut dot = dot;
                                        if !item_enabled {
                                            border_color = alpha_mul(border_color, 0.5);
                                            fg = alpha_mul(fg, 0.8);
                                            dot = alpha_mul(dot, 0.8);
                                        }

                                        let icon_layout = decl_style::layout_style(
                                            &theme,
                                            fret_components_ui::LayoutRefinement::default()
                                                .w_px(MetricRef::Px(icon))
                                                .h_px(MetricRef::Px(icon)),
                                        );
                                        let icon_props = ContainerProps {
                                            layout: icon_layout,
                                            padding: Edges::all(Px(0.0)),
                                            background: None,
                                            shadow: None,
                                            border: Edges::all(Px(1.0)),
                                            border_color: Some(border_color),
                                            corner_radii: Corners::all(radius),
                                        };

                                        let row_layout = LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Auto,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        };

                                        let indicator_layout = decl_style::layout_style(
                                            &theme,
                                            fret_components_ui::LayoutRefinement::default()
                                                .w_px(MetricRef::Px(indicator))
                                                .h_px(MetricRef::Px(indicator)),
                                        );
                                        let indicator_props = ContainerProps {
                                            layout: indicator_layout,
                                            padding: Edges::all(Px(0.0)),
                                            background: Some(dot),
                                            shadow: None,
                                            border: Edges::all(Px(0.0)),
                                            border_color: None,
                                            corner_radii: Corners::all(Px(
                                                (indicator.0 * 0.5).max(0.0),
                                            )),
                                        };

                                        let label = item.label.clone();
                                        let label_props = TextProps {
                                            layout: LayoutStyle::default(),
                                            text: label,
                                            style: Some(text_style.clone()),
                                            color: Some(fg),
                                            wrap: TextWrap::Word,
                                            overflow: TextOverflow::Clip,
                                        };

                                        vec![cx.flex(
                                            FlexProps {
                                                layout: row_layout,
                                                direction: fret_core::Axis::Horizontal,
                                                gap: gap_x,
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                                wrap: false,
                                            },
                                            move |cx| {
                                                let mut out = Vec::new();
                                                out.push(cx.container(icon_props, move |cx| {
                                                    if !is_selected {
                                                        return Vec::new();
                                                    }

                                                    vec![cx.flex(
                                                        FlexProps {
                                                            layout: decl_style::layout_style(
                                                                &theme,
                                                                fret_components_ui::LayoutRefinement::default()
                                                                    .size_full(),
                                                            ),
                                                            direction: fret_core::Axis::Horizontal,
                                                            gap: Px(0.0),
                                                            padding: Edges::all(Px(0.0)),
                                                            justify: MainAlign::Center,
                                                            align: CrossAlign::Center,
                                                            wrap: false,
                                                        },
                                                        move |cx| {
                                                            vec![cx.container(
                                                                indicator_props,
                                                                |_cx| Vec::new(),
                                                            )]
                                                        },
                                                    )]
                                                }));
                                                out.push(cx.text_props(label_props));
                                                out
                                            },
                                        )]
                                    },
                                ));
                            }
                            out
                        },
                    )]
                },
            )
        })
    }
}

pub fn radio_group<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    model: Model<Option<Arc<str>>>,
    items: Vec<RadioGroupItem>,
) -> AnyElement {
    let mut group = RadioGroup::new(model);
    for item in items {
        group = group.item(item);
    }
    group.into_element(cx)
}
