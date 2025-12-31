use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt;
use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::roving_focus;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, MainAlign, PressableA11y, PressableProps,
    RovingFlexProps, RovingFocusProps, SemanticsProps, TextProps,
};
use fret_ui::{ElementCx, Theme, UiHost};

fn tabs_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn tabs_list_bg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or(theme.colors.panel_background)
}

fn tabs_list_fg_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted.foreground")
        .or_else(|| theme.color_by_key("muted-foreground"))
        .unwrap_or(theme.colors.text_muted)
}

fn tabs_trigger_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.tabs.trigger.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.tabs.trigger.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::MEDIUM,
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn tabs_trigger_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.trigger.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(theme))
}

fn tabs_trigger_bg_active(theme: &Theme) -> Color {
    theme
        .color_by_key("background")
        .unwrap_or(theme.colors.surface_background)
}

fn tabs_trigger_border_active(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or(theme.colors.panel_border)
}

#[derive(Debug, Clone)]
pub struct TabsItem {
    value: Arc<str>,
    label: Arc<str>,
    content: Vec<AnyElement>,
    disabled: bool,
}

impl TabsItem {
    pub fn new(
        value: impl Into<Arc<str>>,
        label: impl Into<Arc<str>>,
        content: Vec<AnyElement>,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            content,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone)]
pub struct Tabs {
    model: Model<Option<Arc<str>>>,
    items: Vec<TabsItem>,
    disabled: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Tabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tabs")
            .field("model", &"<model>")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Tabs {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            disabled: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn item(mut self, item: TabsItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabsItem>) -> Self {
        self.items.extend(items);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let model = self.model;
        let items = self.items;
        let tabs_disabled = self.disabled;
        let chrome = self.chrome;
        let layout = self.layout;

        let theme = Theme::global(&*cx.app).clone();
        let gap = tabs_gap(&theme);
        let text_style = tabs_trigger_text_style(&theme);

        let selected: Option<Arc<str>> = cx.watch_model(&model).layout().cloned().flatten();

        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
        let disabled_flags: Vec<bool> = items.iter().map(|i| tabs_disabled || i.disabled).collect();
        let active_idx =
            roving_focus::active_index_from_str_keys(&values, selected.as_deref(), &disabled_flags);

        let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
        let roving = RovingFocusProps {
            enabled: !tabs_disabled,
            wrap: true,
            disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
            ..Default::default()
        };

        let list_props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .p(Space::N1)
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(tabs_list_bg(&theme))),
            LayoutRefinement::default(),
        );

        let active_label = active_idx
            .and_then(|active| items.get(active))
            .map(|item| item.label.clone())
            .unwrap_or_else(|| Arc::from(""));
        let active_children = active_idx
            .and_then(|active| items.get(active))
            .map(|item| item.content.clone())
            .unwrap_or_default();

        let root_props = decl_style::container_props(&theme, chrome, layout);

        cx.container(root_props, move |cx| {
            let mut children: Vec<AnyElement> = Vec::new();

            children.push(cx.container(list_props, |cx| {
                vec![cx.roving_flex(
                    RovingFlexProps {
                        flex: FlexProps {
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                            ..Default::default()
                        },
                        roving,
                    },
                    |cx| {
                        cx.roving_select_option_arc_str(&model, values_arc.clone());

                        let fg_muted = tabs_list_fg_muted(&theme);
                        let fg_disabled = theme.colors.text_disabled;
                        let fg_active = theme
                            .color_by_key("foreground")
                            .unwrap_or(theme.colors.text_primary);
                        let radius = tabs_trigger_radius(&theme);
                        let ring = decl_style::focus_ring(&theme, radius);
                        let bg_active = tabs_trigger_bg_active(&theme);
                        let border_active = tabs_trigger_border_active(&theme);

                        let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                        let pad_y = MetricRef::space(Space::N1).resolve(&theme);
                        let trigger_layout =
                            decl_style::layout_style(&theme, LayoutRefinement::default().flex_1());

                        let mut out: Vec<AnyElement> = Vec::with_capacity(disabled_flags.len());
                        for (idx, item) in items.iter().cloned().enumerate() {
                            let item_disabled = disabled_flags.get(idx).copied().unwrap_or(true);
                            let tab_stop = active_idx.is_some_and(|a| a == idx);
                            let active = tab_stop;

                            let fg = if item_disabled {
                                fg_disabled
                            } else if active {
                                fg_active
                            } else {
                                fg_muted
                            };
                            let bg = (active && !item_disabled).then_some(bg_active);
                            let border = (active && !item_disabled).then_some(border_active);
                            let shadow = (active && !item_disabled)
                                .then(|| decl_style::shadow_sm(&theme, radius));

                            let value = item.value.clone();
                            let label = item.label.clone();
                            let model = model.clone();
                            let text_style = text_style.clone();

                            out.push(cx.pressable(
                                PressableProps {
                                    layout: trigger_layout,
                                    enabled: !item_disabled,
                                    focusable: tab_stop,
                                    focus_ring: Some(ring),
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Tab),
                                        label: Some(label.clone()),
                                        selected: active,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |cx, _state| {
                                    cx.pressable_set_option_arc_str(&model, value.clone());

                                    vec![cx.container(
                                        ContainerProps {
                                            padding: Edges {
                                                top: pad_y,
                                                right: pad_x,
                                                bottom: pad_y,
                                                left: pad_x,
                                            },
                                            background: bg,
                                            shadow,
                                            border: border.map_or_else(
                                                || Edges::all(Px(0.0)),
                                                |_| Edges::all(Px(1.0)),
                                            ),
                                            border_color: border,
                                            corner_radii: Corners::all(radius),
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            vec![cx.text_props(TextProps {
                                                layout: Default::default(),
                                                text: label,
                                                style: Some(text_style.clone()),
                                                color: Some(fg),
                                                wrap: TextWrap::None,
                                                overflow: TextOverflow::Clip,
                                            })]
                                        },
                                    )]
                                },
                            ));
                        }
                        out
                    },
                )]
            }));

            children.push(cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: (!active_label.is_empty()).then_some(active_label),
                    ..Default::default()
                },
                move |_cx| active_children,
            ));

            vec![cx.flex(
                FlexProps {
                    direction: fret_core::Axis::Vertical,
                    gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                    ..Default::default()
                },
                move |_cx| children,
            )]
        })
    }
}

pub fn tabs<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<TabsItem>,
) -> AnyElement {
    Tabs::new(model).items(f(cx)).into_element(cx)
}
