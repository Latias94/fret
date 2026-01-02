use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt;
use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::roving_focus;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space};
use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, MainAlign, PressableA11y, PressableProps, RovingFlexProps,
    RovingFocusProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::toggle::{ToggleSize, ToggleVariant};

fn toggle_bg_hover(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or(theme.colors.hover_background)
}

fn toggle_bg_on(theme: &Theme) -> Color {
    theme.color_by_key("accent").unwrap_or(theme.colors.accent)
}

fn toggle_border(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or(theme.colors.panel_border)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleGroupKind {
    Single,
    Multiple,
}

#[derive(Clone)]
enum ToggleGroupModel {
    Single(Model<Option<Arc<str>>>),
    Multiple(Model<Vec<Arc<str>>>),
}

#[derive(Clone)]
pub struct ToggleGroupItem {
    value: Arc<str>,
    children: Vec<AnyElement>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
}

impl std::fmt::Debug for ToggleGroupItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToggleGroupItem")
            .field("value", &self.value.as_ref())
            .field("children_len", &self.children.len())
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .finish()
    }
}

impl ToggleGroupItem {
    pub fn new(value: impl Into<Arc<str>>, children: Vec<AnyElement>) -> Self {
        Self {
            value: value.into(),
            children,
            disabled: false,
            a11y_label: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }
}

#[derive(Clone)]
pub struct ToggleGroup {
    model: ToggleGroupModel,
    items: Vec<ToggleGroupItem>,
    disabled: bool,
    variant: ToggleVariant,
    size: ToggleSize,
    spacing: Space,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ToggleGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.model {
            ToggleGroupModel::Single(_) => ToggleGroupKind::Single,
            ToggleGroupModel::Multiple(_) => ToggleGroupKind::Multiple,
        };
        f.debug_struct("ToggleGroup")
            .field("kind", &kind)
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("spacing", &self.spacing)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl ToggleGroup {
    pub fn single(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: ToggleGroupModel::Single(model),
            items: Vec::new(),
            disabled: false,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
        Self {
            model: ToggleGroupModel::Multiple(model),
            items: Vec::new(),
            disabled: false,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn spacing(mut self, spacing: Space) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn item(mut self, item: ToggleGroupItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToggleGroupItem>) -> Self {
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model = self.model;
        let items = self.items;
        let group_disabled = self.disabled;
        let variant = self.variant;
        let size = self.size.component_size();
        let spacing = self.spacing;
        let chrome = self.chrome;
        let layout = self.layout;

        let theme = Theme::global(&*cx.app).clone();

        let (selected_single, selected_multi) = match &model {
            ToggleGroupModel::Single(m) => (cx.watch_model(m).layout().cloned().flatten(), None),
            ToggleGroupModel::Multiple(m) => (None, cx.watch_model(m).layout().cloned()),
        };

        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
        let disabled_flags: Vec<bool> =
            items.iter().map(|i| group_disabled || i.disabled).collect();

        let tab_stop = match (selected_single.as_deref(), selected_multi.as_ref()) {
            (Some(selected), _) => {
                roving_focus::active_index_from_str_keys(&values, Some(selected), &disabled_flags)
            }
            (_, Some(selected)) => {
                let first_selected_enabled = values.iter().enumerate().find_map(|(idx, v)| {
                    let enabled = !disabled_flags.get(idx).copied().unwrap_or(true);
                    let on = selected.iter().any(|s| s.as_ref() == v.as_ref());
                    (enabled && on).then_some(idx)
                });
                first_selected_enabled.or_else(|| roving_focus::first_enabled(&disabled_flags))
            }
            _ => roving_focus::first_enabled(&disabled_flags),
        };

        let gap = MetricRef::space(spacing).resolve(&theme);
        let radius = size.control_radius(&theme);
        let ring = decl_style::focus_ring(&theme, radius);
        let pad_x = size.button_px(&theme);
        let pad_y = size.button_py(&theme);

        let bg_hover = toggle_bg_hover(&theme);
        let bg_on = toggle_bg_on(&theme);
        let border = toggle_border(&theme);

        let group_props = decl_style::container_props(&theme, chrome, layout);

        let base_chrome = match variant {
            ToggleVariant::Default => ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                ..Default::default()
            },
            ToggleVariant::Outline => ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(border)),
                ..Default::default()
            },
        };

        let (model_single, model_multi) = match &model {
            ToggleGroupModel::Single(m) => (Some(m.clone()), None),
            ToggleGroupModel::Multiple(m) => (None, Some(m.clone())),
        };

        let roving = RovingFocusProps {
            enabled: !group_disabled,
            wrap: true,
            disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
            ..Default::default()
        };

        cx.container(group_props, move |cx| {
            vec![cx.roving_flex(
                RovingFlexProps {
                    flex: FlexProps {
                        direction: fret_core::Axis::Horizontal,
                        gap,
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                        ..Default::default()
                    },
                    roving,
                },
                move |cx| {
                    cx.roving_nav_apg();
                    let n = items.len();
                    let mut out = Vec::with_capacity(n);

                    for (idx, item) in items.into_iter().enumerate() {
                        let item_disabled = disabled_flags.get(idx).copied().unwrap_or(true);
                        let enabled = !item_disabled;
                        let focusable = tab_stop.is_some_and(|i| i == idx);
                        let on = selected_single
                            .as_deref()
                            .is_some_and(|v| v == item.value.as_ref())
                            || selected_multi.as_ref().is_some_and(|selected| {
                                selected.iter().any(|v| v.as_ref() == item.value.as_ref())
                            });

                        let corners = if gap.0 <= 0.0 {
                            let left = idx == 0;
                            let right = idx + 1 == n;
                            Corners {
                                top_left: if left { radius } else { Px(0.0) },
                                bottom_left: if left { radius } else { Px(0.0) },
                                top_right: if right { radius } else { Px(0.0) },
                                bottom_right: if right { radius } else { Px(0.0) },
                            }
                        } else {
                            Corners::all(radius)
                        };

                        let mut base_props = decl_style::container_props(
                            &theme,
                            base_chrome.clone(),
                            LayoutRefinement::default(),
                        );
                        base_props.padding = Edges {
                            top: pad_y,
                            right: pad_x,
                            bottom: pad_y,
                            left: pad_x,
                        };
                        base_props.corner_radii = corners;

                        if gap.0 <= 0.0
                            && variant == ToggleVariant::Outline
                            && idx > 0
                            && base_props.border.left.0 > 0.0
                        {
                            base_props.border.left = Px(0.0);
                        }

                        let value = item.value.clone();
                        let a11y_label = item.a11y_label.clone().unwrap_or_else(|| value.clone());
                        let children = item.children;
                        let model_single = model_single.clone();
                        let model_multi = model_multi.clone();

                        out.push(
                            cx.pressable(
                                PressableProps {
                                    layout: decl_style::layout_style(
                                        &theme,
                                        LayoutRefinement::default()
                                            .min_h(MetricRef::Px(size.button_h(&theme)))
                                            .min_w_0()
                                            .flex_none(),
                                    ),
                                    enabled,
                                    focusable,
                                    focus_ring: Some(ring),
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(a11y_label),
                                        selected: on,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |cx, state| {
                                    if let Some(m) = model_single.as_ref() {
                                        cx.pressable_set_option_arc_str(m, value.clone());
                                    }
                                    if let Some(m) = model_multi.as_ref() {
                                        cx.pressable_toggle_vec_arc_str(m, value.clone());
                                    }

                                    let hovered = state.hovered && !state.pressed;
                                    let pressed = state.pressed;

                                    let bg = if on && !item_disabled {
                                        Some(bg_on)
                                    } else if (hovered || pressed) && !item_disabled {
                                        Some(bg_hover)
                                    } else {
                                        None
                                    };

                                    let mut props = base_props;
                                    if bg.is_some() {
                                        props.background = bg;
                                    }

                                    vec![cx.container(props, move |_cx| children)]
                                },
                            ),
                        );
                    }

                    out
                },
            )]
        })
    }
}

pub fn toggle_group_single<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ToggleGroupItem>,
) -> AnyElement {
    ToggleGroup::single(model).items(f(cx)).into_element(cx)
}

pub fn toggle_group_multiple<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Vec<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ToggleGroupItem>,
) -> AnyElement {
    ToggleGroup::multiple(model).items(f(cx)).into_element(cx)
}
