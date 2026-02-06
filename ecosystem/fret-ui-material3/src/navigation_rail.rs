//! Material 3 navigation rail (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing and colors via `md.comp.navigation-rail.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to the indicator pill.
//! - Container-level active indicator that tracks the selected icon slot bounds.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Edges, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::{IconId, IconRegistry, ResolvedSvgOwned};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, SemanticsProps,
    SvgIconProps, TextProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::{Invalidation, SvgSource, Theme, UiHost};

use crate::foundation::indication::{
    material_ink_layer_for_pressable_with_ripple_bounds, material_pressable_indication_config,
    RippleClip,
};
use crate::foundation::interaction::{pressable_interaction, PressableInteraction};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::layout_probe::LayoutProbeList;
use crate::foundation::motion_scheme::{sys_spring_in_scope, MotionSchemeKey};
use crate::motion::SpringAnimator;
use crate::tokens::navigation_rail as rail_tokens;
use crate::{Badge, BadgePlacement, BadgeValue};

#[derive(Debug, Default, Clone)]
struct NavigationRailLayoutRuntime {
    icon_slots: LayoutProbeList,
}

#[derive(Debug, Clone)]
pub struct NavigationRailItem {
    value: Arc<str>,
    label: Arc<str>,
    icon: IconId,
    badge: Option<BadgeValue>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl NavigationRailItem {
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
pub struct NavigationRail {
    model: Model<Arc<str>>,
    items: Vec<NavigationRailItem>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    disabled: bool,
    loop_navigation: bool,
    always_show_label: bool,
}

impl NavigationRail {
    pub fn new(model: Model<Arc<str>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            a11y_label: None,
            test_id: None,
            disabled: false,
            loop_navigation: true,
            always_show_label: true,
        }
    }

    pub fn items(mut self, items: Vec<NavigationRailItem>) -> Self {
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

    pub fn always_show_label(mut self, always_show_label: bool) -> Self {
        self.always_show_label = always_show_label;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        navigation_rail_impl(cx, self)
    }
}

fn navigation_rail_impl<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rail: NavigationRail,
) -> AnyElement {
    let NavigationRail {
        model,
        items,
        a11y_label,
        test_id,
        disabled,
        loop_navigation,
        always_show_label,
    } = rail;

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

        let container_w = rail_tokens::container_width(&theme);
        let container_bg = rail_tokens::container_background(&theme);
        let container_shape = rail_tokens::container_shape(&theme);

        let mut props = RovingFlexProps::default();
        props.flex.direction = Axis::Vertical;
        props.flex.gap = Px(4.0);
        props.flex.justify = MainAlign::Start;
        props.flex.align = CrossAlign::Stretch;
        props.flex.padding = Edges::all(Px(4.0));
        props.roving = fret_ui::element::RovingFocusProps {
            enabled: !disabled,
            wrap: loop_navigation,
            disabled: disabled_items.clone(),
        };

        cx.semantics(sem, move |cx| {
            vec![cx.container(
                ContainerProps {
                    background: Some(container_bg),
                    corner_radii: container_shape,
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = Length::Px(container_w);
                        layout.size.height = Length::Fill;
                        layout.overflow = Overflow::Clip;
                        layout
                    },
                    ..Default::default()
                },
                move |cx| {
                    let now_frame = cx.frame_id.0;
                    let container_id = cx.root_id();
                    let item_count = items.len();

                    cx.with_state_for(container_id, NavigationRailLayoutRuntime::default, |rt| {
                        rt.icon_slots.ensure_len(item_count)
                    });

                    let indicator = navigation_rail_active_indicator(
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
                                    (Axis::Vertical, KeyCode::ArrowDown) => Some(true),
                                    (Axis::Vertical, KeyCode::ArrowUp) => Some(false),
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
                                    navigation_rail_item(
                                        cx,
                                        &theme,
                                        container_id,
                                        model.clone(),
                                        it,
                                        idx,
                                        items.len(),
                                        tab_stop,
                                        disabled,
                                        always_show_label,
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

fn navigation_rail_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    container_id: GlobalElementId,
    model: Model<Arc<str>>,
    item: &NavigationRailItem,
    idx: usize,
    set_size: usize,
    tab_stop: bool,
    disabled_group: bool,
    always_show_label: bool,
) -> AnyElement {
    let value = item.value.clone();
    let label = item.label.clone();
    let icon = item.icon.clone();
    let badge = item.badge.clone();
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

        let indicator_w = rail_tokens::active_indicator_width(theme);
        let indicator_h = rail_tokens::active_indicator_height(theme, always_show_label);

        let label_style = theme
            .text_style_by_key("md.sys.typescale.label-medium")
            .unwrap_or_else(TextStyle::default);
        let label_h = label_style
            .line_height
            .unwrap_or(Px(label_style.size.0 * 1.2));

        let show_label = always_show_label || selected;
        let item_h = if show_label {
            Px(indicator_w.0 + label_h.0 + 12.0)
        } else {
            indicator_w
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
                l.size.width = Length::Fill;
                l.size.height = Length::Px(item_h);
                l.overflow = Overflow::Visible;
                enforce_minimum_interactive_size(&mut l, theme);
                l
            },
            focus_ring: None,
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
                let label_color = rail_tokens::label_color(theme, selected, interaction);
                let icon_color = rail_tokens::icon_color(theme, selected, interaction);
                let state_layer_color =
                    rail_tokens::state_layer_color(theme, selected, interaction);
                let state_layer_target =
                    rail_tokens::state_layer_target_opacity(theme, enabled, interaction);

                let bounds = cx
                    .last_bounds_for_element(cx.root_id())
                    .unwrap_or(cx.bounds);

                let ripple_base_opacity = rail_tokens::pressed_state_layer_opacity(theme);
                let config = material_pressable_indication_config(theme, None);

                let indicator_bounds = fret_core::Rect::new(
                    fret_core::Point::new(Px((bounds.size.width.0 - indicator_w.0) * 0.5), Px(4.0)),
                    fret_core::Size::new(indicator_w, indicator_h),
                );

                let corner_radii = rail_tokens::active_indicator_shape(theme);
                let overlay = material_ink_layer_for_pressable_with_ripple_bounds(
                    cx,
                    pressable_id,
                    now_frame,
                    indicator_bounds,
                    indicator_bounds,
                    corner_radii,
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
                    cx.with_state_for(container_id, NavigationRailLayoutRuntime::default, |rt| {
                        rt.icon_slots.set(idx, icon_slot_id);
                    });

                    let icon_el = rail_icon(cx, theme, &icon, icon_color);
                    let icon_el = if let Some(badge) = badge.clone() {
                        let badge = match badge {
                            BadgeValue::Dot => Badge::dot(),
                            BadgeValue::Text(value) => Badge::text(value),
                        };
                        let badge_test_id = test_id
                            .as_ref()
                            .map(|id| Arc::<str>::from(format!("{id}-badge")));
                        let badge = badge
                            .placement(BadgePlacement::NavigationIcon)
                            .navigation_anchor_size(rail_tokens::icon_size(theme));
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
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| vec![icon_el],
                    )
                });

                let label_el =
                    show_label.then(|| rail_label(cx, theme, &label, label_color, selected));

                let mut col = FlexProps::default();
                col.layout.size.width = Length::Fill;
                col.layout.size.height = Length::Px(item_h);
                col.layout.overflow = Overflow::Visible;
                col.direction = Axis::Vertical;
                col.justify = MainAlign::Start;
                col.align = CrossAlign::Center;
                col.gap = Px(4.0);
                col.padding = Edges::all(Px(0.0));

                let mut children: Vec<AnyElement> = vec![overlay, icon_slot];
                if let Some(label_el) = label_el {
                    children.push(label_el);
                }
                vec![cx.flex(col, move |_cx| children)]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn navigation_rail_active_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    now_frame: u64,
    container_id: GlobalElementId,
    item_count: usize,
    selected_idx: Option<usize>,
) -> AnyElement {
    #[derive(Debug, Default, Clone)]
    struct IndicatorRuntime {
        x: SpringAnimator,
        y: SpringAnimator,
        width: SpringAnimator,
        height: SpringAnimator,
    }

    cx.named("navigation_rail_active_indicator", move |cx| {
        let id = cx.root_id();
        let container_bounds = cx.last_bounds_for_element(id).unwrap_or(cx.bounds);

        let indicator_color = rail_tokens::active_indicator_color(theme);
        let corner_radii = rail_tokens::active_indicator_shape(theme);

        let icon_slot_bounds = selected_idx
            .and_then(|idx| {
                cx.with_state_for(container_id, NavigationRailLayoutRuntime::default, |rt| {
                    rt.icon_slots.get(idx)
                })
            })
            .and_then(|icon_slot_id| cx.last_bounds_for_element(icon_slot_id));

        let (target_x, target_y, target_w, target_h, color) = if item_count > 0 {
            if let Some(b) = icon_slot_bounds {
                let x = b.origin.x.0 - container_bounds.origin.x.0;
                let y = b.origin.y.0 - container_bounds.origin.y.0;
                (x, y, b.size.width.0, b.size.height.0, indicator_color)
            } else {
                (0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT)
            }
        } else {
            (0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT)
        };

        let spring = sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial);
        let (x, y, w, h, want_frames) = cx.with_state_for(id, IndicatorRuntime::default, |rt| {
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

fn interaction_state(
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> rail_tokens::NavigationRailItemInteraction {
    match pressable_interaction(pressed, hovered, focused) {
        Some(PressableInteraction::Pressed) => rail_tokens::NavigationRailItemInteraction::Pressed,
        Some(PressableInteraction::Focused) => rail_tokens::NavigationRailItemInteraction::Focused,
        Some(PressableInteraction::Hovered) => rail_tokens::NavigationRailItemInteraction::Hovered,
        None => rail_tokens::NavigationRailItemInteraction::Default,
    }
}

fn rail_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: &Arc<str>,
    color: Color,
    active: bool,
) -> AnyElement {
    let mut style = theme
        .text_style_by_key("md.sys.typescale.label-medium")
        .unwrap_or_else(TextStyle::default);

    style.weight = rail_tokens::label_weight(theme, active);

    let mut props = TextProps::new(text.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}

fn rail_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    icon: &IconId,
    color: Color,
) -> AnyElement {
    let size = rail_tokens::icon_size(theme);
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
            icons.resolve_or_missing_owned(icon)
        });

    match resolved {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}
