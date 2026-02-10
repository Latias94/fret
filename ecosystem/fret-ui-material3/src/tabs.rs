//! Material 3 tabs (primary navigation) (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.primary-navigation-tab.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to the tab bounds.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, ScrollAxis, ScrollProps,
    SemanticsProps, TextProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_with,
};

use crate::foundation::arc_str::empty_arc_str;
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::layout_probe::LayoutProbeList;
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::motion::SpringAnimator;
use crate::tokens::tabs as tabs_tokens;

#[derive(Debug, Default, Clone)]
struct TabListLayoutRuntime {
    tabs: LayoutProbeList,
}

#[derive(Debug, Clone)]
pub struct TabItem {
    value: Arc<str>,
    label: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl TabItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            a11y_label: None,
            test_id: None,
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
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

#[derive(Debug, Clone)]
pub struct Tabs {
    model: Model<Arc<str>>,
    items: Vec<TabItem>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    disabled: bool,
    loop_navigation: bool,
    scrollable: bool,
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
            style: TabsStyle::default(),
        }
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

    pub fn style(mut self, style: TabsStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Tabs {
            model,
            items,
            a11y_label,
            test_id,
            disabled,
            loop_navigation,
            scrollable,
            style,
        } = self;

        cx.scope(|cx| {
            let values: Arc<[Arc<str>]> =
                Arc::from(items.iter().map(|it| it.value.clone()).collect::<Vec<_>>());
            let disabled_items: Arc<[bool]> = Arc::from(
                items
                    .iter()
                    .map(|it| disabled || it.disabled)
                    .collect::<Vec<_>>(),
            );

            let selected = cx
                .get_model_cloned(&model, Invalidation::Layout)
                .unwrap_or_else(empty_arc_str);
            let selected_idx = items
                .iter()
                .position(|it| it.value.as_ref() == selected.as_ref());

            let tab_stop = items
                .iter()
                .position(|it| !disabled && !it.disabled && it.value.as_ref() == selected.as_ref())
                .or_else(|| items.iter().position(|it| !disabled && !it.disabled));

            let sem = SemanticsProps {
                role: SemanticsRole::TabList,
                label: a11y_label.clone(),
                test_id: test_id.clone(),
                disabled,
                ..Default::default()
            };

            let container_states = disabled
                .then_some(WidgetStates::DISABLED)
                .unwrap_or_default();
            let (container_height, container_bg) = {
                let theme = Theme::global(&*cx.app);
                let container_height = tabs_tokens::container_height(theme);
                let container_bg = resolve_override_slot_with(
                    style.container_background.as_ref(),
                    container_states,
                    |color| color.resolve(theme),
                    || tabs_tokens::container_background(theme),
                );
                (container_height, container_bg)
            };

            let mut props = RovingFlexProps::default();
            props.flex.direction = Axis::Horizontal;
            props.flex.gap = Px(0.0);
            props.flex.justify = MainAlign::Start;
            props.flex.align = fret_ui::element::CrossAlign::Stretch;
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
                        let now_frame = cx.frame_id.0;
                        let tab_count = items.len();
                        let container_id = cx.root_id();

                        cx.with_state_for(container_id, TabListLayoutRuntime::default, |rt| {
                            rt.tabs.ensure_len(tab_count);
                        });
                        let indicator = primary_tab_list_indicator(
                            cx,
                            now_frame,
                            container_id,
                            tab_count,
                            selected_idx,
                            disabled,
                            &style,
                        );

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
                                    material_primary_tab(
                                        cx,
                                        container_id,
                                        model.clone(),
                                        it,
                                        idx,
                                        items.len(),
                                        tab_stop,
                                        disabled,
                                        scrollable,
                                        &style,
                                    )
                                })
                                .collect::<Vec<_>>()
                        });

                        let tabs = if scrollable {
                            let mut scroll_props = ScrollProps::default();
                            scroll_props.axis = ScrollAxis::X;
                            scroll_props.layout.size.width = Length::Fill;
                            scroll_props.layout.size.height = Length::Fill;
                            cx.scroll(scroll_props, move |_cx| vec![roving])
                        } else {
                            roving
                        };

                        vec![indicator, tabs]
                    },
                )]
            })
        })
    }
}

fn material_primary_tab<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    model: Model<Arc<str>>,
    item: &TabItem,
    idx: usize,
    set_size: usize,
    tab_stop: bool,
    disabled_group: bool,
    scrollable: bool,
    style_override: &TabsStyle,
) -> AnyElement {
    let value = item.value.clone();
    let label = item.label.clone();
    let a11y_label = item.a11y_label.clone();
    let test_id = item.test_id.clone();

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !disabled_group && !item.disabled;
        let selected = cx
            .get_model_cloned(&model, Invalidation::Layout)
            .map(|v| v.as_ref() == value.as_ref())
            .unwrap_or(false);

        cx.with_state_for(container_id, TabListLayoutRuntime::default, |rt| {
            rt.tabs.ensure_len(set_size);
            rt.tabs.set(idx, pressable_id);
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
        let height = {
            let theme = Theme::global(&*cx.app);
            theme
                .metric_by_key("md.comp.primary-navigation-tab.container.height")
                .unwrap_or(Px(48.0))
        };

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled && tab_stop,
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
                    l.size.width = Length::Auto;
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
                    "md.comp.primary-navigation-tab",
                    corner_radii,
                )
            }),
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
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
                        || tabs_tokens::label_color(theme, selected, interaction),
                    );
                    let state_layer_color = resolve_override_slot_with(
                        style_override.state_layer_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || tabs_tokens::state_layer_color(theme, selected, interaction),
                    );
                    let state_layer_target =
                        tabs_tokens::state_layer_opacity(theme, selected, interaction);
                    let ripple_base_opacity =
                        tabs_tokens::pressed_state_layer_opacity(theme, selected);
                    let indication_config = material_pressable_indication_config(theme, None);
                    (
                        label_color,
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
                let label_el = primary_tab_label(cx, &label, label_color);

                let mut row = FlexProps::default();
                row.layout.size.width = if scrollable {
                    Length::Auto
                } else {
                    Length::Fill
                };
                row.layout.size.height = Length::Px(height);
                row.layout.overflow = Overflow::Clip;
                {
                    let theme = Theme::global(&*cx.app);
                    enforce_minimum_interactive_size(&mut row.layout, theme);
                }
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::Center;
                row.align = CrossAlign::Center;
                row.padding = if scrollable {
                    Edges {
                        left: Px(16.0),
                        right: Px(16.0),
                        top: Px(0.0),
                        bottom: Px(0.0),
                    }
                } else {
                    Edges::all(Px(0.0))
                };

                vec![cx.flex(row, move |_cx| vec![ink, label_el])]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn primary_tab_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &Arc<str>,
    color: Color,
) -> AnyElement {
    let style = {
        let theme = Theme::global(&*cx.app);
        theme
            .text_style_by_key("md.sys.typescale.title-small")
            .unwrap_or_else(TextStyle::default)
    };

    let mut props = TextProps::new(label.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}

fn primary_tab_list_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    now_frame: u64,
    container_id: GlobalElementId,
    tab_count: usize,
    selected_idx: Option<usize>,
    disabled: bool,
    style_override: &TabsStyle,
) -> AnyElement {
    #[derive(Debug, Default, Clone)]
    struct TabListIndicatorRuntime {
        x: SpringAnimator,
        width: SpringAnimator,
        height: SpringAnimator,
    }

    cx.named("primary_tab_indicator", move |cx| {
        let id = cx.root_id();
        let container_bounds = cx.last_bounds_for_element(id).unwrap_or(cx.bounds);
        let tab_bounds = selected_idx
            .and_then(|idx| {
                cx.with_state_for(container_id, TabListLayoutRuntime::default, |rt| {
                    rt.tabs.get(idx)
                })
            })
            .and_then(|tab_id| cx.last_bounds_for_element(tab_id));

        let mut states = WidgetStates::empty();
        if disabled {
            states |= WidgetStates::DISABLED;
        }
        if selected_idx.is_some() {
            states |= WidgetStates::SELECTED;
        }

        let (target_x, target_width, target_height, color, corner_radii, spring) = {
            let theme = Theme::global(&*cx.app);

            let (target_x, target_width, target_height, color) = if tab_count > 0 {
                if let Some(tab_bounds) = tab_bounds {
                    let height = tabs_tokens::active_indicator_height(theme);
                    let color = resolve_override_slot_with(
                        style_override.active_indicator_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || tabs_tokens::active_indicator_color(theme),
                    );
                    let x = tab_bounds.origin.x.0 - container_bounds.origin.x.0;
                    (x, tab_bounds.size.width.0, height.0, color)
                } else if let Some(idx) = selected_idx {
                    let tab_width_px = container_bounds.size.width.0 / (tab_count as f32);
                    let height = tabs_tokens::active_indicator_height(theme);
                    let color = resolve_override_slot_with(
                        style_override.active_indicator_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || tabs_tokens::active_indicator_color(theme),
                    );
                    (tab_width_px * (idx as f32), tab_width_px, height.0, color)
                } else {
                    (0.0, 0.0, 0.0, Color::TRANSPARENT)
                }
            } else {
                (0.0, 0.0, 0.0, Color::TRANSPARENT)
            };

            let corner_radii = tabs_tokens::active_indicator_shape(theme);
            let spring = sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial);

            (
                target_x,
                target_width,
                target_height,
                color,
                corner_radii,
                spring,
            )
        };
        let (x, width, height, want_frames) =
            cx.with_state_for(id, TabListIndicatorRuntime::default, |rt| {
                if !rt.x.is_initialized() {
                    rt.x.reset(now_frame, target_x);
                }
                if !rt.width.is_initialized() {
                    rt.width.reset(now_frame, target_width);
                }
                if !rt.height.is_initialized() {
                    rt.height.reset(now_frame, target_height);
                }

                rt.x.set_target(now_frame, target_x, spring);
                rt.width.set_target(now_frame, target_width, spring);
                rt.height.set_target(now_frame, target_height, spring);

                rt.x.advance(now_frame);
                rt.width.advance(now_frame);
                rt.height.advance(now_frame);

                (
                    Px(rt.x.value()),
                    Px(rt.width.value()),
                    Px(rt.height.value()),
                    rt.x.is_active() || rt.width.is_active() || rt.height.is_active(),
                )
            });

        let mut props = fret_ui::element::CanvasProps::default();
        props.layout.position = fret_ui::element::PositionStyle::Absolute;
        props.layout.inset.top = Some(Px(0.0));
        props.layout.inset.right = Some(Px(0.0));
        props.layout.inset.bottom = Some(Px(0.0));
        props.layout.inset.left = Some(Px(0.0));

        cx.canvas(props, move |p| {
            if height.0 > 0.0 && width.0 > 0.0 && color.a > 0.0 {
                let bounds = p.bounds();

                let x_px = x.0.clamp(0.0, bounds.size.width.0);
                let max_width = (bounds.size.width.0 - x_px).max(0.0);
                let width_px = width.0.clamp(0.0, max_width);

                let top = Px(bounds.origin.y.0 + bounds.size.height.0 - height.0);
                let rect = fret_core::Rect::new(
                    fret_core::Point::new(Px(bounds.origin.x.0 + x_px), top),
                    fret_core::Size::new(Px(width_px), height),
                );

                fret_ui::paint::paint_state_layer(
                    p.scene(),
                    fret_core::DrawOrder(0),
                    rect,
                    color,
                    1.0,
                    corner_radii,
                );
            }

            if want_frames {
                p.request_animation_frame();
            }
        })
    })
}
