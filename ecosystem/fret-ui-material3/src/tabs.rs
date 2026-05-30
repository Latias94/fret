//! Material 3 tabs (primary and secondary navigation) (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.*-navigation-tab.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to the tab bounds.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Px, Rect, SemanticsOrientation, SemanticsRole, SvgFit,
    TextOverflow, TextWrap,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PositionStyle, PressableA11y, PressableProps, RovingFlexProps, ScrollAxis,
    ScrollProps, SemanticsDecoration, SemanticsProps, SvgIconProps, TextProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_with,
};

use crate::foundation::active_indicator::{ActiveIndicatorRect, material_active_indicator_layer};
use crate::foundation::arc_str::empty_arc_str;
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::layout_probe::LayoutProbeList;
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::foundation::test_id::part_test_id;
use crate::tokens::tabs as tabs_tokens;

#[derive(Debug, Default, Clone)]
struct TabListLayoutRuntime {
    tabs: LayoutProbeList,
    labels: LayoutProbeList,
    icons: LayoutProbeList,
}

#[derive(Debug, Clone)]
struct TabPartTestIds {
    chrome: Arc<str>,
    active_indicator: Arc<str>,
    divider: Arc<str>,
}

impl TabPartTestIds {
    fn from_base(base: &Arc<str>) -> Self {
        Self {
            chrome: part_test_id(base, "chrome"),
            active_indicator: part_test_id(base, "active-indicator"),
            divider: part_test_id(base, "divider"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabIconPlacement {
    Leading,
    Stacked,
}

#[derive(Debug, Clone)]
struct TabItemIcon {
    icon: IconId,
    placement: TabIconPlacement,
}

#[derive(Debug, Clone)]
pub struct TabItem {
    value: Arc<str>,
    label: Arc<str>,
    icon: Option<TabItemIcon>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl TabItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn icon(mut self, icon: IconId, placement: TabIconPlacement) -> Self {
        self.icon = Some(TabItemIcon { icon, placement });
        self
    }

    pub fn leading_icon(self, icon: IconId) -> Self {
        self.icon(icon, TabIconPlacement::Leading)
    }

    pub fn stacked_icon(self, icon: IconId) -> Self {
        self.icon(icon, TabIconPlacement::Stacked)
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn uses_stacked_icon(&self) -> bool {
        self.icon
            .as_ref()
            .is_some_and(|icon| icon.placement == TabIconPlacement::Stacked)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TabsStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
    pub active_indicator_color: OverrideSlot<ColorRef>,
}

impl TabsStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(color);
        self
    }

    pub fn state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.state_layer_color = Some(color);
        self
    }

    pub fn active_indicator_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.active_indicator_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.container_background.is_some() {
            self.container_background = other.container_background;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        if other.active_indicator_color.is_some() {
            self.active_indicator_color = other.active_indicator_color;
        }
        self
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TabsVariant {
    #[default]
    Primary,
    Secondary,
}

impl TabsVariant {
    fn token_kind(self) -> tabs_tokens::NavigationTabKind {
        match self {
            Self::Primary => tabs_tokens::NavigationTabKind::Primary,
            Self::Secondary => tabs_tokens::NavigationTabKind::Secondary,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tabs {
    model: Model<Arc<str>>,
    items: Vec<TabItem>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    disabled: bool,
    loop_navigation: bool,
    scrollable: bool,
    variant: TabsVariant,
    style: TabsStyle,
}

impl Tabs {
    pub fn new(model: Model<Arc<str>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            a11y_label: None,
            test_id: None,
            disabled: false,
            loop_navigation: true,
            scrollable: false,
            variant: TabsVariant::Primary,
            style: TabsStyle::default(),
        }
    }

    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<Arc<str>>>,
        default_value: impl Into<Arc<str>>,
    ) -> Self {
        let value =
            controllable_state::use_controllable_model(cx, value, || default_value.into()).model();
        Self::new(value)
    }

    pub fn uncontrolled<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        default_value: impl Into<Arc<str>>,
    ) -> Self {
        Self::new_controllable(cx, None, default_value)
    }

    pub fn value_model(&self) -> Model<Arc<str>> {
        self.model.clone()
    }

    pub fn items(mut self, items: Vec<TabItem>) -> Self {
        self.items = items;
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn secondary(mut self) -> Self {
        self.variant = TabsVariant::Secondary;
        self
    }

    pub fn style(mut self, style: TabsStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Tabs {
            model,
            items,
            a11y_label,
            test_id,
            disabled,
            loop_navigation,
            scrollable,
            variant,
            style,
        } = self;
        let token_kind = variant.token_kind();

        cx.scope(|cx| {
            let values: Arc<[Arc<str>]> =
                Arc::from(items.iter().map(|it| it.value.clone()).collect::<Vec<_>>());
            let disabled_items: Arc<[bool]> = Arc::from(
                items
                    .iter()
                    .map(|it| disabled || it.disabled)
                    .collect::<Vec<_>>(),
            );

            let selected_value = cx
                .get_model_cloned(&model, Invalidation::Paint)
                .unwrap_or_else(empty_arc_str);
            let selected_idx = items
                .iter()
                .position(|it| it.value.as_ref() == selected_value.as_ref());
            let has_stacked_tabs = items.iter().any(TabItem::uses_stacked_icon);

            let tab_stop = items
                .iter()
                .position(|it| {
                    !disabled && !it.disabled && it.value.as_ref() == selected_value.as_ref()
                })
                .or_else(|| items.iter().position(|it| !disabled && !it.disabled));

            let sem = SemanticsProps {
                role: SemanticsRole::TabList,
                label: a11y_label.clone(),
                test_id: test_id.clone(),
                disabled,
                orientation: Some(SemanticsOrientation::Horizontal),
                ..Default::default()
            };

            let part_test_ids = test_id.as_ref().map(TabPartTestIds::from_base);
            let chrome_test_id = part_test_ids.as_ref().map(|ids| ids.chrome.clone());
            let indicator_test_id = part_test_ids
                .as_ref()
                .map(|ids| ids.active_indicator.clone());
            let divider_test_id = part_test_ids.as_ref().map(|ids| ids.divider.clone());

            let container_states = if disabled {
                WidgetStates::DISABLED
            } else {
                Default::default()
            };
            let (container_height, container_bg) = {
                let theme = Theme::global(&*cx.app);
                let container_height = if has_stacked_tabs {
                    tabs_tokens::stacked_container_height_for(theme, token_kind)
                } else {
                    tabs_tokens::container_height_for(theme, token_kind)
                };
                let container_bg = resolve_override_slot_with(
                    style.container_background.as_ref(),
                    container_states,
                    |color| color.resolve(theme),
                    || tabs_tokens::container_background_for(theme, token_kind),
                );
                (container_height, container_bg)
            };

            let mut props = RovingFlexProps::default();
            props.flex.direction = Axis::Horizontal;
            props.flex.gap = Px(0.0).into();
            props.flex.justify = MainAlign::Start;
            props.flex.align = fret_ui::element::CrossAlign::Stretch;
            if scrollable {
                let edge_padding = {
                    let theme = Theme::global(&*cx.app);
                    tabs_tokens::scrollable_edge_padding_for(theme, token_kind)
                };
                props.flex.padding = Edges {
                    left: edge_padding,
                    right: edge_padding,
                    top: Px(0.0),
                    bottom: Px(0.0),
                }
                .into();
            }
            props.roving = fret_ui::element::RovingFocusProps {
                enabled: !disabled,
                wrap: loop_navigation,
                disabled: disabled_items.clone(),
            };

            cx.semantics(sem, move |cx| {
                vec![cx.container(
                    ContainerProps {
                        background: Some(container_bg),
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(container_height);
                            layout
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        let tab_count = items.len();
                        let container_id = cx.root_id();

                        cx.state_for(container_id, TabListLayoutRuntime::default, |rt| {
                            rt.tabs.ensure_len(tab_count);
                            rt.labels.ensure_len(tab_count);
                            rt.icons.ensure_len(tab_count);
                        });
                        let indicator = tab_list_indicator(
                            cx,
                            container_id,
                            tab_count,
                            selected_idx,
                            indicator_test_id.clone(),
                            scrollable,
                            disabled,
                            token_kind,
                            &style,
                        );
                        let divider = tab_row_divider(cx, token_kind, divider_test_id.clone());

                        let roving = cx.roving_flex(props, move |cx| {
                            let values_for_roving = values.clone();
                            let model_for_roving = model.clone();

                            cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                                use fret_ui::action::RovingNavigateResult;

                                let is_disabled = |idx: usize| -> bool {
                                    it.disabled.get(idx).copied().unwrap_or(false)
                                };

                                let forward = match (it.axis, it.key) {
                                    (Axis::Horizontal, KeyCode::ArrowRight) => Some(true),
                                    (Axis::Horizontal, KeyCode::ArrowLeft) => Some(false),
                                    _ => None,
                                };

                                if it.key == KeyCode::Home {
                                    let target = (0..it.len).find(|&i| !is_disabled(i));
                                    return RovingNavigateResult::Handled { target };
                                }
                                if it.key == KeyCode::End {
                                    let target = (0..it.len).rev().find(|&i| !is_disabled(i));
                                    return RovingNavigateResult::Handled { target };
                                }

                                let Some(forward) = forward else {
                                    return RovingNavigateResult::NotHandled;
                                };

                                let current = it
                                    .current
                                    .or_else(|| (0..it.len).find(|&i| !is_disabled(i)));
                                let Some(current) = current else {
                                    return RovingNavigateResult::Handled { target: None };
                                };

                                let len = it.len;
                                let mut target: Option<usize> = None;
                                if it.wrap {
                                    for step in 1..=len {
                                        let idx = if forward {
                                            (current + step) % len
                                        } else {
                                            (current + len - (step % len)) % len
                                        };
                                        if !is_disabled(idx) {
                                            target = Some(idx);
                                            break;
                                        }
                                    }
                                } else if forward {
                                    target = ((current + 1)..len).find(|&i| !is_disabled(i));
                                } else if current > 0 {
                                    target = (0..current).rev().find(|&i| !is_disabled(i));
                                }

                                RovingNavigateResult::Handled { target }
                            }));

                            cx.roving_on_active_change(Arc::new(move |host, action_cx, idx| {
                                let Some(value) = values_for_roving.get(idx).cloned() else {
                                    return;
                                };
                                let already_selected = host
                                    .models_mut()
                                    .read(&model_for_roving, |v| v.as_ref() == value.as_ref())
                                    .ok()
                                    .unwrap_or(false);
                                if already_selected {
                                    return;
                                }
                                let _ = host.update_model(&model_for_roving, |v| *v = value);
                                host.request_redraw(action_cx.window);
                            }));

                            items
                                .iter()
                                .enumerate()
                                .map(|(idx, it)| {
                                    let tab_stop = tab_stop.is_some_and(|t| t == idx);
                                    material_tab(
                                        cx,
                                        container_id,
                                        model.clone(),
                                        it,
                                        idx,
                                        items.len(),
                                        tab_stop,
                                        disabled,
                                        scrollable,
                                        token_kind,
                                        container_height,
                                        selected_idx.is_some_and(|t| t == idx),
                                        &style,
                                    )
                                })
                                .collect::<Vec<_>>()
                        });

                        let mut tabs = if scrollable {
                            let mut scroll_props = ScrollProps::default();
                            scroll_props.axis = ScrollAxis::X;
                            scroll_props.layout.size.width = Length::Fill;
                            scroll_props.layout.size.height = Length::Fill;
                            cx.scroll(scroll_props, move |_cx| vec![roving])
                        } else {
                            roving
                        };
                        if let Some(chrome_test_id) = chrome_test_id.clone() {
                            tabs = tabs.test_id(chrome_test_id);
                        }

                        vec![divider, indicator, tabs]
                    },
                )]
            })
        })
    }
}

fn material_tab<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    model: Model<Arc<str>>,
    item: &TabItem,
    idx: usize,
    set_size: usize,
    tab_stop: bool,
    disabled_group: bool,
    scrollable: bool,
    token_kind: tabs_tokens::NavigationTabKind,
    height: Px,
    selected: bool,
    style_override: &TabsStyle,
) -> AnyElement {
    let value = item.value.clone();
    let label = item.label.clone();
    let item_icon = item.icon.clone();
    let a11y_label = item.a11y_label.clone();
    let test_id = item.test_id.clone();

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !disabled_group && !item.disabled;

        cx.state_for(container_id, TabListLayoutRuntime::default, |rt| {
            rt.tabs.ensure_len(set_size);
            rt.labels.ensure_len(set_size);
            rt.icons.ensure_len(set_size);
            rt.tabs.set(idx, pressable_id);
            if item_icon.is_none() {
                rt.icons.set(idx, GlobalElementId(0));
            }
        });

        if enabled {
            let model_for_press = model.clone();
            let value_for_press = value.clone();
            let handler: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let already_selected = host
                    .models_mut()
                    .read(&model_for_press, |v| v.as_ref() == value_for_press.as_ref())
                    .ok()
                    .unwrap_or(false);
                if already_selected {
                    return;
                }
                let _ = host.update_model(&model_for_press, |v| *v = value_for_press.clone());
                host.request_redraw(action_cx.window);
            });
            cx.pressable_on_activate(handler);
        }

        let corner_radii = Corners::all(Px(0.0));
        let pressable_props = PressableProps {
            enabled,
            focusable: enabled && tab_stop,
            key_activation: Default::default(),
            a11y: PressableA11y {
                role: Some(SemanticsRole::Tab),
                label: a11y_label.clone().or_else(|| Some(label.clone())),
                test_id: test_id.clone(),
                selected,
                pos_in_set: Some((idx + 1) as u32),
                set_size: Some(set_size as u32),
                ..Default::default()
            },
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.height = Length::Px(height);
                if scrollable {
                    let min_width = {
                        let theme = Theme::global(&*cx.app);
                        tabs_tokens::scrollable_min_tab_width_for(theme, token_kind)
                    };
                    l.size.width = Length::Auto;
                    l.size.min_width = Some(Length::Px(min_width));
                    l.flex.grow = 0.0;
                    l.flex.shrink = 0.0;
                } else {
                    l.size.width = Length::Fill;
                    l.flex.grow = 1.0;
                }
                l.overflow = Overflow::Visible;
                {
                    let theme = Theme::global(&*cx.app);
                    enforce_minimum_interactive_size(&mut l, theme);
                }
                l
            },
            focus_ring: Some({
                let theme = Theme::global(&*cx.app);
                material_focus_ring_for_component(
                    theme,
                    tabs_tokens::component_prefix(token_kind),
                    corner_radii,
                )
            }),
            focus_ring_always_paint: false,
            focus_ring_bounds: None,
        };

        let chrome_test_id = test_id.as_ref().map(|id| part_test_id(id, "chrome"));

        let mut pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            props.layout.size.width = if scrollable {
                Length::Auto
            } else {
                Length::Fill
            };
            props.layout.size.height = Length::Fill;
            if scrollable {
                let theme = Theme::global(&*cx.app);
                props.layout.size.min_width = Some(Length::Px(
                    tabs_tokens::scrollable_min_tab_width_for(theme, token_kind),
                ));
            }
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let now_frame = cx.frame_id.0;
                let focus_visible =
                    fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
                let is_pressed = enabled && st.pressed;
                let is_hovered = enabled && st.hovered;
                let is_focused = enabled && st.focused && focus_visible;

                let mut states = WidgetStates::from_pressable(cx, st, enabled);
                if selected {
                    states |= WidgetStates::SELECTED;
                }

                let interaction = if is_pressed {
                    tabs_tokens::TabInteraction::Pressed
                } else if is_focused {
                    tabs_tokens::TabInteraction::Focused
                } else if is_hovered {
                    tabs_tokens::TabInteraction::Hovered
                } else {
                    tabs_tokens::TabInteraction::Default
                };

                let (
                    label_color,
                    icon_color,
                    icon_size,
                    state_layer_color,
                    state_layer_target,
                    ripple_base_opacity,
                    indication_config,
                ) = {
                    let theme = Theme::global(&*cx.app);
                    let label_color = resolve_override_slot_with(
                        style_override.label_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || tabs_tokens::label_color_for(theme, token_kind, selected, interaction),
                    );
                    let icon_color =
                        tabs_tokens::icon_color_for(theme, token_kind, selected, interaction);
                    let icon_size = tabs_tokens::icon_size_for(theme, token_kind);
                    let state_layer_color = resolve_override_slot_with(
                        style_override.state_layer_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || {
                            tabs_tokens::state_layer_color_for(
                                theme,
                                token_kind,
                                selected,
                                interaction,
                            )
                        },
                    );
                    let state_layer_target = tabs_tokens::state_layer_opacity_for(
                        theme,
                        token_kind,
                        selected,
                        interaction,
                    );
                    let ripple_base_opacity =
                        tabs_tokens::pressed_state_layer_opacity_for(theme, token_kind, selected);
                    let indication_config = material_pressable_indication_config(theme, None);
                    (
                        label_color,
                        icon_color,
                        icon_size,
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        indication_config,
                    )
                };
                let ink = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    is_pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    indication_config,
                    false,
                );
                let label_test_id = test_id.as_ref().map(|id| part_test_id(id, "label"));
                let label_el = tab_label(
                    cx,
                    container_id,
                    idx,
                    set_size,
                    &label,
                    label_color,
                    scrollable,
                    token_kind,
                    label_test_id,
                );
                let icon_test_id = test_id.as_ref().map(|id| part_test_id(id, "icon"));

                let content = tab_content(
                    cx,
                    item_icon.clone(),
                    icon_size,
                    icon_color,
                    label_el,
                    tabs_tokens::leading_icon_label_gap(),
                    container_id,
                    idx,
                    set_size,
                    icon_test_id,
                );

                let mut row = FlexProps::default();
                row.layout.size.width = if scrollable {
                    Length::Auto
                } else {
                    Length::Fill
                };
                row.layout.size.height = Length::Px(height);
                if scrollable {
                    let theme = Theme::global(&*cx.app);
                    row.layout.size.min_width = Some(Length::Px(
                        tabs_tokens::scrollable_min_tab_width_for(theme, token_kind),
                    ));
                }
                row.layout.overflow = Overflow::Clip;
                {
                    let theme = Theme::global(&*cx.app);
                    enforce_minimum_interactive_size(&mut row.layout, theme);
                }
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::Center;
                row.align = CrossAlign::Center;
                row.padding = if scrollable {
                    tabs_tokens::horizontal_text_padding().into()
                } else {
                    Edges::all(Px(0.0)).into()
                };

                let mut chrome = cx.flex(row, move |_cx| vec![ink, content]);
                if let Some(test_id) = chrome_test_id.clone() {
                    chrome = chrome.test_id(test_id);
                }
                vec![chrome]
            })
        });
        if let Some(chrome_test_id) = chrome_test_id {
            pointer_region = pointer_region
                .attach_semantics(SemanticsDecoration::default().test_id(chrome_test_id));
        }

        (pressable_props, vec![pointer_region])
    })
}

fn tab_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    item_icon: Option<TabItemIcon>,
    icon_size: Px,
    icon_color: Color,
    label_el: AnyElement,
    gap: Px,
    container_id: GlobalElementId,
    idx: usize,
    set_size: usize,
    icon_test_id: Option<Arc<str>>,
) -> AnyElement {
    cx.named("tab_content", move |cx| {
        let mut children = Vec::new();
        if let Some(item_icon) = item_icon.clone() {
            let mut icon_el = tab_icon(
                cx,
                container_id,
                idx,
                set_size,
                &item_icon.icon,
                icon_size,
                icon_color,
            );
            if let Some(icon_test_id) = icon_test_id.clone() {
                icon_el = icon_el.test_id(icon_test_id);
            }
            children.push(icon_el);
        }
        children.push(label_el);

        let mut content = FlexProps::default();
        content.layout.size.width = Length::Auto;
        content.layout.size.min_width = Some(Length::Px(Px(0.0)));
        content.layout.size.max_width = Some(Length::Fill);
        content.layout.flex.shrink = 1.0;
        content.direction = match item_icon.as_ref().map(|icon| icon.placement) {
            Some(TabIconPlacement::Stacked) => Axis::Vertical,
            _ => Axis::Horizontal,
        };
        content.justify = MainAlign::Center;
        content.align = CrossAlign::Center;
        content.gap = match item_icon.as_ref().map(|icon| icon.placement) {
            Some(TabIconPlacement::Leading) => gap.into(),
            Some(TabIconPlacement::Stacked) => tabs_tokens::stacked_icon_label_gap().into(),
            None => Px(0.0).into(),
        };
        content.padding = Edges::all(Px(0.0)).into();

        cx.flex(content, move |_cx| children)
    })
}

fn tab_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    idx: usize,
    set_size: usize,
    icon: &IconId,
    size: Px,
    color: Color,
) -> AnyElement {
    let icon = icon.clone();

    cx.named("tab_icon", move |cx| {
        let svg = svg_source_for_icon(cx, &icon);

        let mut props = SvgIconProps::new(svg);
        props.fit = SvgFit::Contain;
        props.layout.size.width = Length::Px(size);
        props.layout.size.height = Length::Px(size);
        props.layout.flex.shrink = 0.0;
        props.color = color;
        let icon_el = cx.svg_icon_props(props);
        let icon_id = icon_el.id;
        cx.state_for(container_id, TabListLayoutRuntime::default, |rt| {
            rt.icons.ensure_len(set_size);
            rt.icons.set(idx, icon_id);
        });
        icon_el
    })
}

fn tab_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    idx: usize,
    set_size: usize,
    label: &Arc<str>,
    color: Color,
    scrollable: bool,
    token_kind: tabs_tokens::NavigationTabKind,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let label = label.clone();

    cx.named("tab_label", move |cx| {
        let style = {
            let theme = Theme::global(&*cx.app);
            tabs_tokens::label_text_style_for(theme, token_kind)
        };

        let mut props = TextProps::new(label.clone());
        props.style = Some(style);
        props.color = Some(color);
        props.layout.size.width = Length::Auto;
        props.layout.size.min_width = Some(Length::Px(Px(0.0)));
        props.layout.size.max_width = Some(Length::Fill);
        props.layout.flex.grow = 0.0;
        props.layout.flex.shrink = 1.0;
        props.layout.flex.basis = Length::Auto;
        if scrollable {
            props.layout.flex.shrink = 0.0;
        }
        props.wrap = TextWrap::None;
        props.overflow = TextOverflow::Clip;

        let mut label_el = cx.text_props(props);
        let label_id = label_el.id;
        cx.state_for(container_id, TabListLayoutRuntime::default, |rt| {
            rt.labels.ensure_len(set_size);
            rt.labels.set(idx, label_id);
        });
        if let Some(test_id) = test_id.clone() {
            label_el = label_el.test_id(test_id);
        }
        label_el
    })
}

fn tab_row_divider<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    token_kind: tabs_tokens::NavigationTabKind,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    cx.named("tab_row_divider", move |cx| {
        let (height, color) = {
            let theme = Theme::global(&*cx.app);
            (
                tabs_tokens::divider_height_for(theme, token_kind),
                tabs_tokens::divider_color_for(theme, token_kind),
            )
        };

        let mut props = ContainerProps::default();
        props.background = Some(color);
        props.layout.position = PositionStyle::Absolute;
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Px(height);
        props.layout.inset.left = Some(Px(0.0)).into();
        props.layout.inset.right = Some(Px(0.0)).into();
        props.layout.inset.bottom = Some(Px(0.0)).into();

        let divider = cx.container(props, |_cx| Vec::new());
        match test_id {
            Some(test_id) => divider.test_id(test_id),
            None => divider,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{Point, Rect, Size};
    use fret_ui::element::{ElementKind, TextProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        )
    }

    fn find_text_by_content<'a>(el: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => Some(props),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, text)),
        }
    }

    #[test]
    fn primary_tab_labels_can_shrink_within_equal_width_slots() {
        let window = fret_core::AppWindowId::default();
        let mut app = App::new();
        let selected = Arc::<str>::from("overview");
        let label = Arc::<str>::from(
            "A very long primary tab label that should shrink inside equal-width tab slots",
        );
        let model = app.models_mut().insert(selected.clone());

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "m3-tabs", |cx| {
            Tabs::new(model.clone())
                .items(vec![
                    TabItem::new(selected.clone(), label.clone()),
                    TabItem::new("details", "Details"),
                ])
                .into_element(cx)
        });

        let label = find_text_by_content(&el, label.as_ref()).expect("primary tab label text");
        assert_eq!(label.wrap, TextWrap::None);
        assert_eq!(label.overflow, TextOverflow::Clip);
        assert_eq!(label.layout.size.width, Length::Auto);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(label.layout.size.max_width, Some(Length::Fill));
        assert_eq!(label.layout.flex.grow, 0.0);
        assert_eq!(label.layout.flex.shrink, 1.0);
        assert_eq!(label.layout.flex.basis, Length::Auto);
    }

    #[test]
    fn scrollable_primary_tab_labels_keep_intrinsic_width() {
        let window = fret_core::AppWindowId::default();
        let mut app = App::new();
        let selected = Arc::<str>::from("overview");
        let label = Arc::<str>::from("Overview");
        let model = app.models_mut().insert(selected.clone());

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "m3-tabs", |cx| {
            Tabs::new(model.clone())
                .items(vec![
                    TabItem::new(selected.clone(), label.clone()),
                    TabItem::new("details", "Details"),
                ])
                .scrollable(true)
                .into_element(cx)
        });

        let label = find_text_by_content(&el, label.as_ref()).expect("primary tab label text");
        assert_eq!(label.wrap, TextWrap::None);
        assert_eq!(label.overflow, TextOverflow::Clip);
        assert_eq!(label.layout.size.width, Length::Auto);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(label.layout.flex.grow, 0.0);
        assert_eq!(label.layout.flex.shrink, 0.0);
        assert_eq!(label.layout.flex.basis, Length::Auto);
    }
}

#[cfg(test)]
mod controllable_state_tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::elements::with_element_cx;
    use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        )
    }

    #[test]
    fn tabs_new_controllable_uses_controlled_value_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controlled = app.models_mut().insert(Arc::<str>::from("settings"));

        with_element_cx(&mut app, window, bounds(), "m3-tabs-controlled", |cx| {
            let tabs = Tabs::new_controllable(cx, Some(controlled.clone()), "overview");
            assert_eq!(tabs.value_model(), controlled);
        });
    }

    #[test]
    fn tabs_new_controllable_applies_default_value() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(&mut app, window, bounds(), "m3-tabs-default-value", |cx| {
            let tabs = Tabs::new_controllable(cx, None, "overview");
            let value = cx
                .watch_model(&tabs.value_model())
                .layout()
                .cloned()
                .unwrap_or_else(Arc::<str>::default);
            assert_eq!(value.as_ref(), "overview");
        });
    }

    #[test]
    fn tabs_uncontrolled_multiple_instances_do_not_share_models() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(&mut app, window, bounds(), "m3-tabs-uncontrolled", |cx| {
            let a = Tabs::uncontrolled(cx, "overview");
            let b = Tabs::uncontrolled(cx, "settings");
            assert_ne!(a.value_model(), b.value_model());
        });
    }
}

fn tab_list_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    tab_count: usize,
    selected_idx: Option<usize>,
    indicator_test_id: Option<Arc<str>>,
    scrollable: bool,
    disabled: bool,
    token_kind: tabs_tokens::NavigationTabKind,
    style_override: &TabsStyle,
) -> AnyElement {
    cx.named("tab_indicator", move |cx| {
        let container_bounds = cx.last_bounds_for_element(container_id);
        let tab_bounds = selected_idx
            .and_then(|idx| {
                cx.state_for(container_id, TabListLayoutRuntime::default, |rt| {
                    rt.tabs.get(idx)
                })
            })
            .and_then(|tab_id| cx.last_bounds_for_element(tab_id));
        let label_bounds = selected_idx
            .and_then(|idx| {
                cx.state_for(container_id, TabListLayoutRuntime::default, |rt| {
                    rt.labels.get(idx)
                })
            })
            .and_then(|label_id| cx.last_bounds_for_element(label_id));
        let icon_bounds = selected_idx
            .and_then(|idx| {
                cx.state_for(container_id, TabListLayoutRuntime::default, |rt| {
                    rt.icons.get(idx)
                })
            })
            .and_then(|icon_id| cx.last_bounds_for_element(icon_id));

        let mut states = WidgetStates::empty();
        if disabled {
            states |= WidgetStates::DISABLED;
        }
        if selected_idx.is_some() {
            states |= WidgetStates::SELECTED;
        }

        let (target_x, target_y, target_width, target_height, color, corner_radii, spring) = {
            let theme = Theme::global(&*cx.app);

            let (target_x, target_y, target_width, target_height, color) = if tab_count > 0 {
                if let Some(tab_bounds) = tab_bounds {
                    let height = tabs_tokens::active_indicator_height(theme);
                    let min_width = tabs_tokens::active_indicator_min_width(theme).0;
                    let edge_padding = if scrollable {
                        tabs_tokens::scrollable_edge_padding_for(theme, token_kind).0
                    } else {
                        0.0
                    };
                    let color = resolve_override_slot_with(
                        style_override.active_indicator_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || tabs_tokens::active_indicator_color(theme),
                    );
                    let idx = selected_idx.unwrap_or(0);
                    let tab_x = container_bounds
                        .map(|bounds| tab_bounds.origin.x.0 - bounds.origin.x.0)
                        .unwrap_or_else(|| edge_padding + tab_bounds.size.width.0 * (idx as f32));
                    let tab_y = container_bounds
                        .map(|bounds| tab_bounds.origin.y.0 - bounds.origin.y.0)
                        .unwrap_or(0.0);
                    let (x, content_width) = if tabs_tokens::indicator_matches_content(token_kind) {
                        let content_span = tab_content_span(label_bounds, icon_bounds);
                        if let Some((content_left, content_width)) = content_span {
                            let target_width = content_width.max(min_width);
                            let content_x = container_bounds
                                .map(|bounds| content_left - bounds.origin.x.0)
                                .unwrap_or_else(|| {
                                    tab_x + (tab_bounds.size.width.0 - content_width) * 0.5
                                });
                            (
                                content_x + (content_width - target_width) * 0.5,
                                target_width,
                            )
                        } else {
                            let target_width = min_width.min(tab_bounds.size.width.0);
                            (
                                tab_x + (tab_bounds.size.width.0 - target_width) * 0.5,
                                target_width,
                            )
                        }
                    } else {
                        (tab_x, tab_bounds.size.width.0)
                    };
                    let y = tab_y + (tab_bounds.size.height.0 - height.0).max(0.0);
                    (x, y, content_width, height.0, color)
                } else if let Some(idx) = selected_idx {
                    let min_width = tabs_tokens::active_indicator_min_width(theme).0;
                    let edge_padding = if scrollable {
                        tabs_tokens::scrollable_edge_padding_for(theme, token_kind).0
                    } else {
                        0.0
                    };
                    let tab_width_px = if scrollable {
                        tabs_tokens::scrollable_min_tab_width_for(theme, token_kind).0
                    } else {
                        container_bounds
                            .map(|bounds| bounds.size.width.0 / (tab_count as f32))
                            .unwrap_or(min_width.max(48.0))
                    };
                    let height = tabs_tokens::active_indicator_height(theme);
                    let target_width = if tabs_tokens::indicator_matches_content(token_kind) {
                        min_width.min(tab_width_px)
                    } else {
                        tab_width_px
                    };
                    let target_y = container_bounds
                        .map(|bounds| (bounds.size.height.0 - height.0).max(0.0))
                        .unwrap_or_else(|| {
                            (tabs_tokens::container_height_for(theme, token_kind).0 - height.0)
                                .max(0.0)
                        });
                    let color = resolve_override_slot_with(
                        style_override.active_indicator_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || tabs_tokens::active_indicator_color(theme),
                    );
                    let tab_x = edge_padding + tab_width_px * (idx as f32);
                    (
                        tab_x + (tab_width_px - target_width) * 0.5,
                        target_y,
                        target_width,
                        height.0,
                        color,
                    )
                } else {
                    (0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT)
                }
            } else {
                (0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT)
            };

            let corner_radii = tabs_tokens::active_indicator_shape_for(theme, token_kind);
            let spring = sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial);

            (
                target_x,
                target_y,
                target_width,
                target_height,
                color,
                corner_radii,
                spring,
            )
        };
        let target = ActiveIndicatorRect::new(target_x, target_y, target_width, target_height);

        material_active_indicator_layer(
            cx,
            target,
            color,
            corner_radii,
            spring,
            indicator_test_id.clone(),
        )
    })
}

fn tab_content_span(label_bounds: Option<Rect>, icon_bounds: Option<Rect>) -> Option<(f32, f32)> {
    match (label_bounds, icon_bounds) {
        (Some(label), Some(icon)) => {
            let left = label.origin.x.0.min(icon.origin.x.0);
            let right =
                (label.origin.x.0 + label.size.width.0).max(icon.origin.x.0 + icon.size.width.0);
            Some((left, (right - left).max(0.0)))
        }
        (Some(label), None) => Some((label.origin.x.0, label.size.width.0)),
        (None, Some(icon)) => Some((icon.origin.x.0, icon.size.width.0)),
        (None, None) => None,
    }
}
