//! Material 3 navigation bar (bottom navigation) (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing via `md.comp.navigation-bar.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to item bounds.
//! - Container-level active indicator that tracks the selected icon slot bounds.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, FontWeight, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap,
};
use fret_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, SemanticsProps,
    SvgIconProps, TextProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::{Invalidation, SvgSource, Theme, UiHost};

use crate::foundation::elevation::{apply_surface_tint, shadow_for_elevation_with_color};
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    IndicationConfig, RippleClip, advance_indication_for_pressable, material_ink_layer,
};
use crate::foundation::layout_probe::LayoutProbeList;
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion::SpringAnimator;

#[derive(Debug, Default, Clone)]
struct NavigationBarLayoutRuntime {
    icon_slots: LayoutProbeList,
}

#[derive(Debug, Clone)]
pub struct NavigationBarItem {
    value: Arc<str>,
    label: Arc<str>,
    icon: IconId,
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
            let theme = Theme::global(&*cx.app).clone();

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
                .unwrap_or_else(|| Arc::<str>::from(""));
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

            let tokens = MaterialTokenResolver::new(&theme);
            let container_height = theme
                .metric_by_key("md.comp.navigation-bar.container.height")
                .unwrap_or(Px(80.0));
            let container_bg = tokens.color_comp_or_sys(
                "md.comp.navigation-bar.container.color",
                "md.sys.color.surface-container",
            );
            let elevation = theme
                .metric_by_key("md.comp.navigation-bar.container.elevation")
                .unwrap_or(Px(0.0));
            let surface_tint = tokens.color_comp_or_sys(
                "md.comp.navigation-bar.container.surface-tint-layer.color",
                "md.sys.color.surface-tint",
            );
            let container_bg = apply_surface_tint(container_bg, surface_tint, elevation);
            let shadow_color = tokens.color_comp_or_sys(
                "md.comp.navigation-bar.container.shadow-color",
                "md.sys.color.shadow",
            );
            let radius = theme
                .metric_by_key("md.comp.navigation-bar.container.shape")
                .unwrap_or(Px(0.0));
            let corner_radii = Corners::all(radius);
            let shadow = shadow_for_elevation_with_color(
                &theme,
                elevation,
                Some(shadow_color),
                corner_radii,
            );

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
                        let now_frame = cx.frame_id.0;
                        let container_id = cx.root_id();
                        let item_count = items.len();

                        cx.with_state_for(
                            container_id,
                            NavigationBarLayoutRuntime::default,
                            |rt| {
                                rt.icon_slots.ensure_len(item_count);
                            },
                        );

                        let indicator = navigation_bar_active_indicator(
                            cx,
                            &theme,
                            now_frame,
                            container_id,
                            item_count,
                            selected_idx,
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
                                            &theme,
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
    theme: &Theme,
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
    let a11y_label = item.a11y_label.clone();
    let test_id = item.test_id.clone();

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

        let height = theme
            .metric_by_key("md.comp.navigation-bar.container.height")
            .unwrap_or(Px(80.0));
        let focus_ring = material_focus_ring_for_component(
            theme,
            "md.comp.navigation-bar",
            Corners::all(Px(0.0)),
        );

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
                l.size.width = Length::Fill;
                l.flex.grow = 1.0;
                l.overflow = Overflow::Visible;
                l
            },
            focus_ring: Some(focus_ring),
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let now_frame = cx.frame_id.0;
                let focus_visible =
                    fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
                let is_pressed = enabled && st.pressed;
                let is_hovered = enabled && st.hovered;
                let is_focused = enabled && st.focused && focus_visible;

                let interaction = interaction_state(is_pressed, is_hovered, is_focused);
                let label_color = nav_label_color(theme, selected, interaction);
                let icon_color = nav_icon_color(theme, selected, interaction);
                let state_layer_color = nav_state_layer_color(theme, selected, interaction);
                let state_layer_target =
                    nav_state_layer_opacity(theme, is_pressed, is_hovered, is_focused);

                let state_duration_ms = theme
                    .duration_ms_by_key("md.sys.motion.duration.short2")
                    .unwrap_or(100);
                let easing = theme
                    .easing_by_key("md.sys.motion.easing.standard")
                    .unwrap_or(fret_ui::theme::CubicBezier {
                        x1: 0.0,
                        y1: 0.0,
                        x2: 1.0,
                        y2: 1.0,
                    });

                let ripple_expand_ms = theme
                    .duration_ms_by_key("md.sys.motion.duration.short4")
                    .unwrap_or(200);
                let ripple_fade_ms = theme
                    .duration_ms_by_key("md.sys.motion.duration.short2")
                    .unwrap_or(100);

                let bounds = cx
                    .last_bounds_for_element(cx.root_id())
                    .unwrap_or(cx.bounds);
                let last_down = cx
                    .with_state(fret_ui::element::PointerRegionState::default, |st| {
                        st.last_down
                    });

                let ripple_base_opacity = theme
                    .number_by_key("md.comp.navigation-bar.pressed.state-layer.opacity")
                    .unwrap_or(0.1);
                let config = IndicationConfig {
                    state_duration_ms,
                    ripple_expand_ms,
                    ripple_fade_ms,
                    ripple_radius: None,
                    easing,
                };
                let indication = advance_indication_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    bounds,
                    last_down,
                    is_pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                );

                let ink = material_ink_layer(
                    cx,
                    Corners::all(Px(0.0)),
                    RippleClip::Bounded,
                    state_layer_color,
                    indication.state_layer_opacity,
                    indication.ripple_frame,
                    indication.want_frames,
                );

                let indicator_w = theme
                    .metric_by_key("md.comp.navigation-bar.active-indicator.width")
                    .unwrap_or(Px(64.0));
                let indicator_h = theme
                    .metric_by_key("md.comp.navigation-bar.active-indicator.height")
                    .unwrap_or(Px(32.0));

                let icon_slot = cx.named("icon_slot", |cx| {
                    let icon_slot_id = cx.root_id();
                    cx.with_state_for(container_id, NavigationBarLayoutRuntime::default, |rt| {
                        rt.icon_slots.set(idx, icon_slot_id);
                    });

                    let icon_el = nav_icon(cx, theme, &icon, icon_color);
                    cx.flex(
                        FlexProps {
                            layout: {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = Length::Px(indicator_w);
                                layout.size.height = Length::Px(indicator_h);
                                layout
                            },
                            direction: Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| vec![icon_el],
                    )
                });

                let label_el = nav_label(cx, theme, &label, label_color, selected);

                let mut col = FlexProps::default();
                col.layout.size.width = Length::Fill;
                col.layout.size.height = Length::Px(height);
                col.layout.overflow = Overflow::Clip;
                col.direction = Axis::Vertical;
                col.justify = MainAlign::Center;
                col.align = CrossAlign::Center;
                col.gap = Px(4.0);
                col.padding = Edges::all(Px(0.0));

                vec![cx.flex(col, move |_cx| vec![ink, icon_slot, label_el])]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn navigation_bar_active_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    now_frame: u64,
    container_id: GlobalElementId,
    item_count: usize,
    selected_idx: Option<usize>,
) -> AnyElement {
    #[derive(Debug, Default, Clone)]
    struct NavIndicatorRuntime {
        x: SpringAnimator,
        y: SpringAnimator,
        width: SpringAnimator,
        height: SpringAnimator,
    }

    cx.named("navigation_bar_active_indicator", move |cx| {
        let id = cx.root_id();
        let container_bounds = cx.last_bounds_for_element(id).unwrap_or(cx.bounds);

        let indicator_w = theme
            .metric_by_key("md.comp.navigation-bar.active-indicator.width")
            .unwrap_or(Px(64.0));
        let indicator_h = theme
            .metric_by_key("md.comp.navigation-bar.active-indicator.height")
            .unwrap_or(Px(32.0));

        let tokens = MaterialTokenResolver::new(theme);
        let indicator_color = tokens.color_comp_or_sys(
            "md.comp.navigation-bar.active-indicator.color",
            "md.sys.color.secondary-container",
        );

        let radius = theme
            .metric_by_key("md.comp.navigation-bar.active-indicator.shape")
            .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
            .unwrap_or(Px(9999.0));
        let corner_radii = Corners::all(radius);

        let icon_slot_bounds = selected_idx
            .and_then(|idx| {
                cx.with_state_for(container_id, NavigationBarLayoutRuntime::default, |rt| {
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
                let label_style = theme
                    .text_style_by_key("md.sys.typescale.label-medium")
                    .unwrap_or_else(TextStyle::default);
                let label_h = label_style
                    .line_height
                    .unwrap_or(Px(label_style.size.0 * 1.2))
                    .0;
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

        let spring = sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial);
        let (x, y, w, h, want_frames) = cx.with_state_for(id, NavIndicatorRuntime::default, |rt| {
            if !rt.x.is_initialized() {
                rt.x.reset(now_frame, target_x);
            }
            if !rt.y.is_initialized() {
                rt.y.reset(now_frame, target_y);
            }
            if !rt.width.is_initialized() {
                rt.width.reset(now_frame, target_w);
            }
            if !rt.height.is_initialized() {
                rt.height.reset(now_frame, target_h);
            }

            rt.x.set_target(now_frame, target_x, spring);
            rt.y.set_target(now_frame, target_y, spring);
            rt.width.set_target(now_frame, target_w, spring);
            rt.height.set_target(now_frame, target_h, spring);

            rt.x.advance(now_frame);
            rt.y.advance(now_frame);
            rt.width.advance(now_frame);
            rt.height.advance(now_frame);

            (
                Px(rt.x.value()),
                Px(rt.y.value()),
                Px(rt.width.value()),
                Px(rt.height.value()),
                rt.x.is_active()
                    || rt.y.is_active()
                    || rt.width.is_active()
                    || rt.height.is_active(),
            )
        });

        let mut props = fret_ui::element::CanvasProps::default();
        props.layout.position = fret_ui::element::PositionStyle::Absolute;
        props.layout.inset.top = Some(Px(0.0));
        props.layout.inset.right = Some(Px(0.0));
        props.layout.inset.bottom = Some(Px(0.0));
        props.layout.inset.left = Some(Px(0.0));

        cx.canvas(props, move |p| {
            if w.0 > 0.0 && h.0 > 0.0 && color.a > 0.0 {
                let bounds = p.bounds();
                let x_px = x.0.clamp(0.0, bounds.size.width.0);
                let y_px = y.0.clamp(0.0, bounds.size.height.0);
                let max_w = (bounds.size.width.0 - x_px).max(0.0);
                let max_h = (bounds.size.height.0 - y_px).max(0.0);
                let w_px = w.0.clamp(0.0, max_w);
                let h_px = h.0.clamp(0.0, max_h);

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

            if want_frames {
                p.request_animation_frame();
            }
        })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractionState {
    Default,
    Hovered,
    Focused,
    Pressed,
}

fn interaction_state(pressed: bool, hovered: bool, focused: bool) -> InteractionState {
    if pressed {
        InteractionState::Pressed
    } else if focused {
        InteractionState::Focused
    } else if hovered {
        InteractionState::Hovered
    } else {
        InteractionState::Default
    }
}

fn nav_state_layer_opacity(theme: &Theme, pressed: bool, hovered: bool, focused: bool) -> f32 {
    let key = if pressed {
        "md.comp.navigation-bar.pressed.state-layer.opacity"
    } else if focused {
        "md.comp.navigation-bar.focus.state-layer.opacity"
    } else if hovered {
        "md.comp.navigation-bar.hover.state-layer.opacity"
    } else {
        return 0.0;
    };
    theme.number_by_key(key).unwrap_or(0.0)
}

fn nav_state_layer_color(theme: &Theme, active: bool, interaction: InteractionState) -> Color {
    let key = if active {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-bar.active.focus.state-layer.color",
            InteractionState::Hovered => "md.comp.navigation-bar.active.hover.state-layer.color",
            InteractionState::Pressed => "md.comp.navigation-bar.active.pressed.state-layer.color",
            InteractionState::Default => return Color::TRANSPARENT,
        }
    } else {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-bar.inactive.focus.state-layer.color",
            InteractionState::Hovered => "md.comp.navigation-bar.inactive.hover.state-layer.color",
            InteractionState::Pressed => {
                "md.comp.navigation-bar.inactive.pressed.state-layer.color"
            }
            InteractionState::Default => return Color::TRANSPARENT,
        }
    };
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
}

fn nav_icon_color(theme: &Theme, active: bool, interaction: InteractionState) -> Color {
    let key = if active {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-bar.active.focus.icon.color",
            InteractionState::Hovered => "md.comp.navigation-bar.active.hover.icon.color",
            InteractionState::Pressed => "md.comp.navigation-bar.active.pressed.icon.color",
            InteractionState::Default => "md.comp.navigation-bar.active.icon.color",
        }
    } else {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-bar.inactive.focus.icon.color",
            InteractionState::Hovered => "md.comp.navigation-bar.inactive.hover.icon.color",
            InteractionState::Pressed => "md.comp.navigation-bar.inactive.pressed.icon.color",
            InteractionState::Default => "md.comp.navigation-bar.inactive.icon.color",
        }
    };
    theme
        .color_by_key(key)
        .or_else(|| {
            if active {
                theme.color_by_key("md.sys.color.on-secondary-container")
            } else {
                theme.color_by_key("md.sys.color.on-surface-variant")
            }
        })
        .unwrap_or_else(|| {
            if active {
                MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-secondary-container")
            } else {
                MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface-variant")
            }
        })
}

fn nav_label_color(theme: &Theme, active: bool, interaction: InteractionState) -> Color {
    let key = if active {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-bar.active.focus.label-text.color",
            InteractionState::Hovered => "md.comp.navigation-bar.active.hover.label-text.color",
            InteractionState::Pressed => "md.comp.navigation-bar.active.pressed.label-text.color",
            InteractionState::Default => "md.comp.navigation-bar.active.label-text.color",
        }
    } else {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-bar.inactive.focus.label-text.color",
            InteractionState::Hovered => "md.comp.navigation-bar.inactive.hover.label-text.color",
            InteractionState::Pressed => "md.comp.navigation-bar.inactive.pressed.label-text.color",
            InteractionState::Default => "md.comp.navigation-bar.inactive.label-text.color",
        }
    };
    theme
        .color_by_key(key)
        .or_else(|| {
            if active {
                theme.color_by_key("md.sys.color.on-surface")
            } else {
                theme.color_by_key("md.sys.color.on-surface-variant")
            }
        })
        .unwrap_or_else(|| {
            if active {
                MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface")
            } else {
                MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface-variant")
            }
        })
}

fn nav_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: &Arc<str>,
    color: Color,
    active: bool,
) -> AnyElement {
    let mut style = theme
        .text_style_by_key("md.sys.typescale.label-medium")
        .unwrap_or_else(TextStyle::default);

    let weight = if active {
        theme
            .number_by_key("md.comp.navigation-bar.active.label-text.weight")
            .unwrap_or(700.0)
    } else {
        theme
            .number_by_key("md.comp.navigation-bar.label-text.weight")
            .unwrap_or(500.0)
    };
    style.weight = FontWeight(weight.round().clamp(1.0, 1000.0) as u16);

    let mut props = TextProps::new(text.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}

fn nav_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    icon: &IconId,
    color: Color,
) -> AnyElement {
    let size = theme
        .metric_by_key("md.comp.navigation-bar.icon.size")
        .unwrap_or(Px(24.0));
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = fret_core::SvgFit::Contain;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.color = color;
    cx.svg_icon_props(props)
}

fn svg_source_for_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: &IconId) -> SvgSource {
    let resolved = cx
        .app
        .with_global_mut(IconRegistry::default, |icons, _app| {
            icons
                .resolve_svg_owned(icon)
                .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
        });

    match resolved {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}
