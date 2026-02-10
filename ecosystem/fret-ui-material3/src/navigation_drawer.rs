//! Material 3 navigation drawer (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing and colors via `md.comp.navigation-drawer.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to item pill bounds.
//! - Selected item uses `active-indicator.color` as the container background (Compose parity).

use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, SemanticsProps,
    SpacerProps, SvgIconProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, Theme, UiHost};

use crate::foundation::arc_str::empty_arc_str;
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::surface::material_surface_style;
use crate::tokens::navigation_drawer as drawer_tokens;

#[derive(Debug, Clone)]
pub struct NavigationDrawerItem {
    value: Arc<str>,
    label: Arc<str>,
    icon: IconId,
    badge_label: Option<Arc<str>>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl NavigationDrawerItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>, icon: IconId) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon,
            badge_label: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn badge_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.badge_label = Some(label.into());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavigationDrawerVariant {
    #[default]
    Standard,
    Modal,
}

#[derive(Debug, Clone)]
pub struct NavigationDrawer {
    model: Model<Arc<str>>,
    items: Vec<NavigationDrawerItem>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    disabled: bool,
    loop_navigation: bool,
    variant: NavigationDrawerVariant,
}

impl NavigationDrawer {
    pub fn new(model: Model<Arc<str>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            a11y_label: None,
            test_id: None,
            disabled: false,
            loop_navigation: true,
            variant: NavigationDrawerVariant::default(),
        }
    }

    pub fn items(mut self, items: Vec<NavigationDrawerItem>) -> Self {
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

    pub fn variant(mut self, variant: NavigationDrawerVariant) -> Self {
        self.variant = variant;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let NavigationDrawer {
            model,
            items,
            a11y_label,
            test_id,
            disabled,
            loop_navigation,
            variant,
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

            let (container_w, item_h_pad, container_bg, shadow, container_shape) = {
                let theme = Theme::global(&*cx.app);

                let container_w = drawer_tokens::container_width(theme);
                let item_h_pad = drawer_tokens::item_horizontal_padding(theme);

                let container_bg = drawer_tokens::container_background(theme, variant);
                let elevation = drawer_tokens::container_elevation(theme, variant);
                let container_shape = drawer_tokens::container_shape(theme);
                let surface =
                    material_surface_style(theme, container_bg, elevation, None, container_shape);
                (
                    container_w,
                    item_h_pad,
                    surface.background,
                    surface.shadow,
                    container_shape,
                )
            };

            let mut props = RovingFlexProps::default();
            props.flex.direction = Axis::Vertical;
            props.flex.gap = Px(0.0);
            props.flex.justify = MainAlign::Start;
            props.flex.align = CrossAlign::Stretch;
            props.flex.padding = Edges {
                left: item_h_pad,
                right: item_h_pad,
                top: Px(0.0),
                bottom: Px(0.0),
            };
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
                        vec![cx.roving_flex(props, move |cx| {
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
                                    navigation_drawer_item(
                                        cx,
                                        model.clone(),
                                        it,
                                        idx,
                                        items.len(),
                                        tab_stop,
                                        disabled,
                                    )
                                })
                                .collect::<Vec<_>>()
                        })]
                    },
                )]
            })
        })
    }
}

fn navigation_drawer_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Arc<str>>,
    item: &NavigationDrawerItem,
    idx: usize,
    set_size: usize,
    tab_stop: bool,
    disabled_group: bool,
) -> AnyElement {
    let value = item.value.clone();
    let label = item.label.clone();
    let icon = item.icon.clone();
    let badge_label = item.badge_label.clone();
    let a11y_label = item.a11y_label.clone();
    let test_id = item.test_id.clone();

    let (
        height,
        corner_radii,
        focus_ring,
        ripple_base_opacity,
        config,
        selected_bg,
        icon_size,
        label_style_base,
        label_weight_active,
        label_weight_inactive,
        badge_style,
        badge_color,
    ) = {
        let theme = Theme::global(&*cx.app);

        let height = drawer_tokens::active_indicator_height(theme);
        let corner_radii = Corners::all(drawer_tokens::active_indicator_radius(theme));
        let focus_ring =
            material_focus_ring_for_component(theme, "md.comp.navigation-drawer", corner_radii);

        let ripple_base_opacity = drawer_tokens::pressed_state_layer_opacity(theme);
        let config = material_pressable_indication_config(theme, None);

        let selected_bg = drawer_tokens::active_indicator_color(theme);
        let icon_size = drawer_tokens::icon_size(theme);

        let label_style_base = theme
            .text_style_by_key("md.sys.typescale.label-large")
            .unwrap_or_else(TextStyle::default);
        let label_weight_active = drawer_tokens::label_weight(theme, true);
        let label_weight_inactive = drawer_tokens::label_weight(theme, false);

        let mut badge_style = theme
            .text_style_by_key("md.sys.typescale.label-small")
            .unwrap_or_else(TextStyle::default);
        let weight = theme
            .number_by_key("md.comp.navigation-drawer.large-badge-label.weight")
            .unwrap_or(500.0);
        badge_style.weight = fret_core::FontWeight(weight.round().clamp(1.0, 1000.0) as u16);

        let badge_color = theme
            .color_by_key("md.comp.navigation-drawer.large-badge-label.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
            .unwrap_or_else(|| {
                crate::foundation::token_resolver::MaterialTokenResolver::new(theme)
                    .color_sys("md.sys.color.on-surface-variant")
            });

        (
            height,
            corner_radii,
            focus_ring,
            ripple_base_opacity,
            config,
            selected_bg,
            icon_size,
            label_style_base,
            label_weight_active,
            label_weight_inactive,
            badge_style,
            badge_color,
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
                l.overflow = Overflow::Visible;
                {
                    let theme = Theme::global(&*cx.app);
                    enforce_minimum_interactive_size(&mut l, theme);
                }
                l
            },
            focus_ring: Some(focus_ring),
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
                    let label_color = drawer_tokens::label_color(theme, selected, interaction);
                    let icon_color = drawer_tokens::icon_color(theme, selected, interaction);
                    let state_layer_color =
                        drawer_tokens::state_layer_color(theme, selected, interaction);
                    let state_layer_target =
                        drawer_tokens::state_layer_target_opacity(theme, enabled, interaction);
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
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    is_pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let icon_el = drawer_icon(cx, &icon, icon_size, icon_color);
                let label_el = {
                    let mut style = label_style_base.clone();
                    style.weight = if selected {
                        label_weight_active
                    } else {
                        label_weight_inactive
                    };
                    drawer_label(cx, &label, style, label_color)
                };
                let badge_el = badge_label.as_ref().map(|text| {
                    let mut props = TextProps::new(text.clone());
                    props.style = Some(badge_style.clone());
                    props.color = Some(badge_color);
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Clip;
                    cx.text_props(props)
                });
                let mut spacer = SpacerProps::default();
                spacer.layout.flex.grow = 1.0;
                let spacer = cx.spacer(spacer);

                let content_children = {
                    let mut children: Vec<AnyElement> = vec![icon_el, label_el, spacer];
                    if let Some(badge_el) = badge_el {
                        children.push(badge_el);
                    }
                    children
                };

                let content = cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        direction: Axis::Horizontal,
                        gap: Px(12.0),
                        padding: Edges {
                            left: Px(16.0),
                            right: Px(24.0),
                            top: Px(0.0),
                            bottom: Px(0.0),
                        },
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| content_children,
                );

                vec![cx.container(
                    ContainerProps {
                        background: selected.then_some(selected_bg),
                        corner_radii,
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        ..Default::default()
                    },
                    move |_cx| vec![ink, content],
                )]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn interaction_state(
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> drawer_tokens::NavigationDrawerItemInteraction {
    match pressable_interaction(pressed, hovered, focused) {
        Some(PressableInteraction::Pressed) => {
            drawer_tokens::NavigationDrawerItemInteraction::Pressed
        }
        Some(PressableInteraction::Focused) => {
            drawer_tokens::NavigationDrawerItemInteraction::Focused
        }
        Some(PressableInteraction::Hovered) => {
            drawer_tokens::NavigationDrawerItemInteraction::Hovered
        }
        None => drawer_tokens::NavigationDrawerItemInteraction::Default,
    }
}

fn drawer_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: &Arc<str>,
    style: TextStyle,
    color: Color,
) -> AnyElement {
    let mut props = TextProps::new(text.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}

fn drawer_icon<H: UiHost>(
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
