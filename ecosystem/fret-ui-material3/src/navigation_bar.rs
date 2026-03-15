//! Material 3 navigation bar (bottom navigation) (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing via `md.comp.navigation-bar.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to item bounds.
//! - Container-level active indicator that tracks the selected icon slot bounds.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Edges, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, SemanticsDecoration,
    SemanticsProps, SvgIconProps, TextProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_headless::motion::spring::SpringDescription;
use fret_ui_headless::motion::tolerance::Tolerance;
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::motion_value::{
    MotionToSpecF32, MotionValueF32Update, SpringSpecF32, drive_motion_value_f32,
};
use fret_ui_kit::typography::{self, TextIntent};

use crate::foundation::arc_str::empty_arc_str;
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::layout_probe::LayoutProbeList;
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::foundation::surface::material_surface_style;
use crate::tokens::navigation_bar as nav_tokens;
use crate::{Badge, BadgePlacement, BadgeValue};

#[derive(Debug, Default, Clone)]
struct NavigationBarLayoutRuntime {
    icon_slots: LayoutProbeList,
}

#[derive(Debug, Clone)]
pub struct NavigationBarItem {
    value: Arc<str>,
    label: Arc<str>,
    icon: IconId,
    badge: Option<BadgeValue>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl NavigationBarItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>, icon: IconId) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon,
            badge: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn badge(mut self, badge: BadgeValue) -> Self {
        self.badge = Some(badge);
        self
    }

    pub fn badge_dot(mut self) -> Self {
        self.badge = Some(BadgeValue::Dot);
        self
    }

    pub fn badge_text(mut self, value: impl Into<Arc<str>>) -> Self {
        self.badge = Some(BadgeValue::Text(value.into()));
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
}

#[derive(Debug, Clone)]
pub struct NavigationBar {
    model: Model<Arc<str>>,
    items: Vec<NavigationBarItem>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    disabled: bool,
    loop_navigation: bool,
}

impl NavigationBar {
    pub fn new(model: Model<Arc<str>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            a11y_label: None,
            test_id: None,
            disabled: false,
            loop_navigation: true,
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

    pub fn items(mut self, items: Vec<NavigationBarItem>) -> Self {
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let NavigationBar {
            model,
            items,
            a11y_label,
            test_id,
            disabled,
            loop_navigation,
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
            let indicator_test_id = test_id
                .as_ref()
                .map(|id| Arc::<str>::from(format!("{id}-active-indicator")));

            let (container_height, container_bg, shadow, corner_radii) = {
                let theme = Theme::global(&*cx.app);

                let container_height = nav_tokens::container_height(theme);
                let container_bg = nav_tokens::container_background(theme);
                let elevation = nav_tokens::container_elevation(theme);
                let shadow_color = nav_tokens::container_shadow_color(theme);
                let corner_radii = nav_tokens::container_shape(theme);
                let surface = material_surface_style(
                    theme,
                    container_bg,
                    elevation,
                    Some(shadow_color),
                    corner_radii,
                );
                (
                    container_height,
                    surface.background,
                    surface.shadow,
                    corner_radii,
                )
            };

            let mut props = RovingFlexProps::default();
            props.flex.direction = Axis::Horizontal;
            props.flex.gap = Px(0.0).into();
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
                        shadow,
                        corner_radii,
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(container_height);
                            layout
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        let container_id = cx.root_id();
                        let item_count = items.len();

                        cx.state_for(container_id, NavigationBarLayoutRuntime::default, |rt| {
                            rt.icon_slots.ensure_len(item_count);
                        });

                        let indicator = navigation_bar_active_indicator(
                            cx,
                            container_id,
                            item_count,
                            selected_idx,
                            indicator_test_id.clone(),
                        );

                        vec![
                            indicator,
                            cx.roving_flex(props, move |cx| {
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

                                cx.roving_on_active_change(Arc::new(
                                    move |host, action_cx, idx| {
                                        let Some(value) = values_for_roving.get(idx).cloned()
                                        else {
                                            return;
                                        };
                                        let already_selected = host
                                            .models_mut()
                                            .read(&model_for_roving, |v| {
                                                v.as_ref() == value.as_ref()
                                            })
                                            .ok()
                                            .unwrap_or(false);
                                        if already_selected {
                                            return;
                                        }
                                        let _ =
                                            host.update_model(&model_for_roving, |v| *v = value);
                                        host.request_redraw(action_cx.window);
                                    },
                                ));

                                items
                                    .iter()
                                    .enumerate()
                                    .map(|(idx, it)| {
                                        let tab_stop = tab_stop.is_some_and(|t| t == idx);
                                        navigation_bar_item(
                                            cx,
                                            container_id,
                                            model.clone(),
                                            it,
                                            idx,
                                            items.len(),
                                            tab_stop,
                                            disabled,
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        ]
                    },
                )]
            })
        })
    }
}

fn navigation_bar_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    model: Model<Arc<str>>,
    item: &NavigationBarItem,
    idx: usize,
    set_size: usize,
    tab_stop: bool,
    disabled_group: bool,
) -> AnyElement {
    let value = item.value.clone();
    let label = item.label.clone();
    let icon = item.icon.clone();
    let badge = item.badge.clone();
    let a11y_label = item.a11y_label.clone();
    let test_id = item.test_id.clone();

    let (
        height,
        indicator_w,
        indicator_h,
        icon_size,
        state_layer_shape,
        focus_ring,
        ripple_base_opacity,
        config,
        label_style_base,
        label_weight_active,
        label_weight_inactive,
    ) = {
        let theme = Theme::global(&*cx.app);

        let height = nav_tokens::container_height(theme);
        let indicator_w = nav_tokens::active_indicator_width(theme);
        let indicator_h = nav_tokens::active_indicator_height(theme);
        let icon_size = nav_tokens::icon_size(theme);

        let state_layer_shape = nav_tokens::active_indicator_shape(theme);
        let focus_ring =
            material_focus_ring_for_component(theme, "md.comp.navigation-bar", state_layer_shape);

        let ripple_base_opacity = nav_tokens::pressed_state_layer_opacity(theme);
        let config = material_pressable_indication_config(theme, None);

        let label_style_base = theme
            .text_style_by_key("md.sys.typescale.label-medium")
            .unwrap_or_default();
        let label_style_base = typography::with_intent(label_style_base, TextIntent::Control);
        let label_weight_active = nav_tokens::label_weight(theme, true);
        let label_weight_inactive = nav_tokens::label_weight(theme, false);

        (
            height,
            indicator_w,
            indicator_h,
            icon_size,
            state_layer_shape,
            focus_ring,
            ripple_base_opacity,
            config,
            label_style_base,
            label_weight_active,
            label_weight_inactive,
        )
    };

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !disabled_group && !item.disabled;
        let selected = cx
            .get_model_cloned(&model, Invalidation::Layout)
            .map(|v| v.as_ref() == value.as_ref())
            .unwrap_or(false);

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
                l.size.width = Length::Fill;
                l.flex.grow = 1.0;
                l.overflow = Overflow::Visible;
                {
                    let theme = Theme::global(&*cx.app);
                    enforce_minimum_interactive_size(&mut l, theme);
                }
                l
            },
            focus_ring: Some(focus_ring),
            focus_ring_always_paint: false,
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

                let interaction = interaction_state(is_pressed, is_hovered, is_focused);
                let (label_color, icon_color, state_layer_color, state_layer_target) = {
                    let theme = Theme::global(&*cx.app);
                    let label_color = nav_tokens::label_color(theme, selected, interaction);
                    let icon_color = nav_tokens::icon_color(theme, selected, interaction);
                    let state_layer_color =
                        nav_tokens::state_layer_color(theme, selected, interaction);
                    let state_layer_target =
                        nav_tokens::state_layer_target_opacity(theme, enabled, interaction);
                    (
                        label_color,
                        icon_color,
                        state_layer_color,
                        state_layer_target,
                    )
                };
                let ink = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    state_layer_shape,
                    RippleClip::Bounded,
                    state_layer_color,
                    is_pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let icon_slot = cx.named("icon_slot", |cx| {
                    let icon_slot_id = cx.root_id();
                    cx.state_for(container_id, NavigationBarLayoutRuntime::default, |rt| {
                        rt.icon_slots.set(idx, icon_slot_id);
                    });

                    let icon_el = nav_icon(cx, &icon, icon_size, icon_color);
                    let icon_el = if let Some(badge) = badge.clone() {
                        #[derive(Default)]
                        struct DerivedBadgeTestId {
                            base: Option<Arc<str>>,
                            badge: Option<Arc<str>>,
                        }

                        let badge = match badge {
                            BadgeValue::Dot => Badge::dot(),
                            BadgeValue::Text(value) => Badge::text(value),
                        };

                        let badge_test_id = cx.slot_state(DerivedBadgeTestId::default, |st| {
                            if st.base.as_deref() != test_id.as_deref() {
                                st.base = test_id.clone();
                                st.badge = st
                                    .base
                                    .as_ref()
                                    .map(|id| Arc::<str>::from(format!("{id}-badge")));
                            }
                            st.badge.clone()
                        });
                        let badge = badge
                            .placement(BadgePlacement::NavigationIcon)
                            .navigation_anchor_size(icon_size);
                        let badge = if let Some(badge_test_id) = badge_test_id {
                            badge.test_id(badge_test_id)
                        } else {
                            badge
                        };
                        badge.into_element(cx, move |_cx| vec![icon_el])
                    } else {
                        icon_el
                    };
                    cx.flex(
                        FlexProps {
                            layout: {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = Length::Px(indicator_w);
                                layout.size.height = Length::Px(indicator_h);
                                layout
                            },
                            direction: Axis::Horizontal,
                            gap: Px(0.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| vec![icon_el],
                    )
                });

                let label_el = {
                    let mut style = label_style_base.clone();
                    style.weight = if selected {
                        label_weight_active
                    } else {
                        label_weight_inactive
                    };
                    nav_label(cx, &label, style, label_color)
                };

                let mut col = FlexProps::default();
                col.layout.size.width = Length::Fill;
                col.layout.size.height = Length::Px(height);
                col.layout.overflow = Overflow::Clip;
                col.direction = Axis::Vertical;
                col.justify = MainAlign::Center;
                col.align = CrossAlign::Center;
                col.gap = Px(4.0).into();
                col.padding = Edges::all(Px(0.0)).into();

                let chrome_test_id = test_id
                    .as_ref()
                    .map(|id| Arc::<str>::from(format!("{id}.chrome")));
                let mut chrome = cx.flex(col, move |_cx| vec![ink, icon_slot, label_el]);
                if let Some(test_id) = chrome_test_id {
                    chrome = chrome.test_id(test_id);
                }
                vec![chrome]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn navigation_bar_active_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    item_count: usize,
    selected_idx: Option<usize>,
    indicator_test_id: Option<Arc<str>>,
) -> AnyElement {
    cx.named("navigation_bar_active_indicator", move |cx| {
        let id = cx.root_id();
        let container_bounds = cx.last_bounds_for_element(id).unwrap_or(cx.bounds);

        let (indicator_w, indicator_h, indicator_color, corner_radii, label_h, spring) = {
            let theme = Theme::global(&*cx.app);
            let indicator_w = nav_tokens::active_indicator_width(theme);
            let indicator_h = nav_tokens::active_indicator_height(theme);
            let indicator_color = nav_tokens::active_indicator_color(theme);
            let corner_radii = nav_tokens::active_indicator_shape(theme);
            let label_style = theme
                .text_style_by_key("md.sys.typescale.label-medium")
                .unwrap_or_default();
            let label_style = typography::with_intent(label_style, TextIntent::Control);
            let label_h = label_style
                .line_height
                .unwrap_or(Px(label_style.size.0 * 1.2))
                .0;
            let spring = sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial);
            (
                indicator_w,
                indicator_h,
                indicator_color,
                corner_radii,
                label_h,
                spring,
            )
        };

        let icon_slot_bounds = selected_idx
            .and_then(|idx| {
                cx.state_for(container_id, NavigationBarLayoutRuntime::default, |rt| {
                    rt.icon_slots.get(idx)
                })
            })
            .and_then(|icon_slot_id| cx.last_bounds_for_element(icon_slot_id));

        let (target_x, target_y, target_w, target_h, color) = if item_count > 0 {
            if let Some(b) = icon_slot_bounds {
                let x = b.origin.x.0 - container_bounds.origin.x.0;
                let y = b.origin.y.0 - container_bounds.origin.y.0;
                (x, y, b.size.width.0, b.size.height.0, indicator_color)
            } else if let Some(idx) = selected_idx {
                let content_h = indicator_h.0 + 4.0 + label_h;
                let y = ((container_bounds.size.height.0 - content_h) / 2.0).max(0.0);

                let item_w = container_bounds.size.width.0 / (item_count as f32);
                let x = item_w * (idx as f32) + (item_w - indicator_w.0) / 2.0;
                (x, y, indicator_w.0, indicator_h.0, indicator_color)
            } else {
                (0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT)
            }
        } else {
            (0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT)
        };

        let spring = SpringDescription::with_damping_ratio(
            1.0,
            spring.stiffness as f64,
            spring.damping as f64,
        );
        let spec = MotionToSpecF32::Spring(SpringSpecF32 {
            spring,
            tolerance: Tolerance::default(),
            snap_to_target: true,
        });

        let x = drive_motion_value_f32(
            cx,
            target_x,
            MotionValueF32Update::To {
                target: target_x,
                spec,
                kick: None,
            },
        );
        let y = drive_motion_value_f32(
            cx,
            target_y,
            MotionValueF32Update::To {
                target: target_y,
                spec,
                kick: None,
            },
        );
        let w = drive_motion_value_f32(
            cx,
            target_w,
            MotionValueF32Update::To {
                target: target_w,
                spec,
                kick: None,
            },
        );
        let h = drive_motion_value_f32(
            cx,
            target_h,
            MotionValueF32Update::To {
                target: target_h,
                spec,
                kick: None,
            },
        );

        let mut props = fret_ui::element::CanvasProps::default();
        props.layout.position = fret_ui::element::PositionStyle::Absolute;
        props.layout.inset.top = Some(Px(0.0)).into();
        props.layout.inset.right = Some(Px(0.0)).into();
        props.layout.inset.bottom = Some(Px(0.0)).into();
        props.layout.inset.left = Some(Px(0.0)).into();

        let mut indicator = cx.canvas(props, move |p| {
            if w.value > 0.0 && h.value > 0.0 && color.a > 0.0 {
                let bounds = p.bounds();
                let x_px = x.value.clamp(0.0, bounds.size.width.0);
                let y_px = y.value.clamp(0.0, bounds.size.height.0);
                let max_w = (bounds.size.width.0 - x_px).max(0.0);
                let max_h = (bounds.size.height.0 - y_px).max(0.0);
                let w_px = w.value.clamp(0.0, max_w);
                let h_px = h.value.clamp(0.0, max_h);

                let rect = fret_core::Rect::new(
                    fret_core::Point::new(
                        Px(bounds.origin.x.0 + x_px),
                        Px(bounds.origin.y.0 + y_px),
                    ),
                    fret_core::Size::new(Px(w_px), Px(h_px)),
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
        });

        if let Some(test_id) = indicator_test_id.as_ref() {
            indicator =
                indicator.attach_semantics(SemanticsDecoration::default().test_id(test_id.clone()));
        }

        indicator
    })
}

fn interaction_state(
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> nav_tokens::NavigationBarItemInteraction {
    match pressable_interaction(pressed, hovered, focused) {
        Some(PressableInteraction::Pressed) => nav_tokens::NavigationBarItemInteraction::Pressed,
        Some(PressableInteraction::Focused) => nav_tokens::NavigationBarItemInteraction::Focused,
        Some(PressableInteraction::Hovered) => nav_tokens::NavigationBarItemInteraction::Hovered,
        None => nav_tokens::NavigationBarItemInteraction::Default,
    }
}

fn nav_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: &Arc<str>,
    style: TextStyle,
    color: Color,
) -> AnyElement {
    let mut props = TextProps::new(text.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.layout.size.width = Length::Fill;
    props.layout.size.min_width = Some(Length::Px(Px(0.0)));
    props.layout.flex.grow = 1.0;
    props.layout.flex.basis = Length::Px(Px(0.0));
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{Point, Rect, Size};
    use fret_icons::ids;
    use fret_ui::element::{ElementKind, TextProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
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
    fn navigation_bar_labels_can_shrink_within_item_slots() {
        let window = fret_core::AppWindowId::default();
        let mut app = App::new();
        let selected = Arc::<str>::from("inbox");
        let label = Arc::<str>::from(
            "A very long navigation bar label that should shrink within the item slot",
        );
        let model = app.models_mut().insert(selected.clone());

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-navigation-bar",
            |cx| {
                NavigationBar::new(model.clone())
                    .items(vec![
                        NavigationBarItem::new(selected.clone(), label.clone(), ids::ui::SEARCH)
                            .badge_text("99+"),
                        NavigationBarItem::new("settings", "Settings", ids::ui::SEARCH),
                    ])
                    .into_element(cx)
            },
        );

        let label = find_text_by_content(&el, label.as_ref()).expect("navigation bar label text");
        assert_eq!(label.wrap, TextWrap::None);
        assert_eq!(label.overflow, TextOverflow::Clip);
        assert_eq!(label.layout.size.width, Length::Fill);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(label.layout.flex.grow, 1.0);
        assert_eq!(label.layout.flex.basis, Length::Px(Px(0.0)));
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
            Size::new(Px(320.0), Px(160.0)),
        )
    }

    #[test]
    fn navigation_bar_new_controllable_uses_controlled_value_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controlled = app.models_mut().insert(Arc::<str>::from("settings"));

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-navigation-bar-controlled",
            |cx| {
                let bar = NavigationBar::new_controllable(cx, Some(controlled.clone()), "search");
                assert_eq!(bar.value_model(), controlled);
            },
        );
    }

    #[test]
    fn navigation_bar_new_controllable_applies_default_value() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-navigation-bar-default",
            |cx| {
                let bar = NavigationBar::new_controllable(cx, None, "search");
                let value = cx
                    .watch_model(&bar.value_model())
                    .layout()
                    .cloned()
                    .unwrap_or_else(Arc::<str>::default);
                assert_eq!(value.as_ref(), "search");
            },
        );
    }

    #[test]
    fn navigation_bar_uncontrolled_multiple_instances_do_not_share_models() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-navigation-bar-uncontrolled",
            |cx| {
                let a = NavigationBar::uncontrolled(cx, "search");
                let b = NavigationBar::uncontrolled(cx, "settings");
                assert_ne!(a.value_model(), b.value_model());
            },
        );
    }
}

fn nav_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
    size: Px,
    color: Color,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = fret_core::SvgFit::Contain;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.color = color;
    cx.svg_icon_props(props)
}
