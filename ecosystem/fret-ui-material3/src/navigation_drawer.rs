//! Material 3 navigation drawer (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing and colors via `md.comp.navigation-drawer.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to item pill bounds.
//! - Selected item uses `active-indicator.color` as the container background (Compose parity).

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
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, SvgSource, Theme, UiHost};

use crate::foundation::elevation::{
    apply_surface_tint_if_surface, shadow_for_elevation_with_color,
};
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    IndicationConfig, RippleClip, advance_indication_for_pressable, material_ink_layer,
};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::token_resolver::MaterialTokenResolver;

#[derive(Debug, Clone)]
pub struct NavigationDrawerItem {
    value: Arc<str>,
    label: Arc<str>,
    icon: IconId,
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
            let container_w = theme
                .metric_by_key("md.comp.navigation-drawer.container.width")
                .unwrap_or(Px(360.0));
            let active_indicator_w = theme
                .metric_by_key("md.comp.navigation-drawer.active-indicator.width")
                .unwrap_or(Px(336.0));
            let item_h_pad = Px(((container_w.0 - active_indicator_w.0) / 2.0).max(0.0));

            let (container_key, container_fallback) = match variant {
                NavigationDrawerVariant::Standard => (
                    "md.comp.navigation-drawer.standard.container.color",
                    "md.sys.color.surface",
                ),
                NavigationDrawerVariant::Modal => (
                    "md.comp.navigation-drawer.modal.container.color",
                    "md.sys.color.surface-container-low",
                ),
            };
            let mut container_bg = tokens.color_comp_or_sys(container_key, container_fallback);

            let elevation = match variant {
                NavigationDrawerVariant::Standard => theme
                    .metric_by_key("md.comp.navigation-drawer.standard.container.elevation")
                    .unwrap_or(Px(0.0)),
                NavigationDrawerVariant::Modal => theme
                    .metric_by_key("md.comp.navigation-drawer.modal.container.elevation")
                    .unwrap_or(Px(1.0)),
            };
            container_bg = apply_surface_tint_if_surface(&theme, container_bg, elevation);

            let container_shape = theme
                .corners_by_key("md.comp.navigation-drawer.container.shape")
                .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-large"))
                .unwrap_or_else(|| Corners::all(Px(0.0)));
            let shadow = shadow_for_elevation_with_color(&theme, elevation, None, container_shape);

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
                                        &theme,
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
    theme: &Theme,
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
            .metric_by_key("md.comp.navigation-drawer.active-indicator.height")
            .unwrap_or(Px(56.0));
        let radius = theme
            .metric_by_key("md.comp.navigation-drawer.active-indicator.shape")
            .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
            .unwrap_or(Px(9999.0));
        let corner_radii = Corners::all(radius);
        let focus_ring =
            material_focus_ring_for_component(theme, "md.comp.navigation-drawer", corner_radii);

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
                enforce_minimum_interactive_size(&mut l, theme);
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
                let label_color = drawer_label_color(theme, selected, interaction);
                let icon_color = drawer_icon_color(theme, selected, interaction);
                let state_layer_color = drawer_state_layer_color(theme, selected, interaction);
                let state_layer_target =
                    drawer_state_layer_opacity(theme, is_pressed, is_hovered, is_focused);

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
                    .number_by_key("md.comp.navigation-drawer.pressed.state-layer.opacity")
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
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    indication.state_layer_opacity,
                    indication.ripple_frame,
                    indication.want_frames,
                );

                let tokens = MaterialTokenResolver::new(theme);
                let selected_bg = tokens.color_comp_or_sys(
                    "md.comp.navigation-drawer.active-indicator.color",
                    "md.sys.color.secondary-container",
                );

                let icon_el = drawer_icon(cx, theme, &icon, icon_color);
                let label_el = drawer_label(cx, theme, &label, label_color, selected);

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
                    move |_cx| vec![icon_el, label_el],
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

fn drawer_state_layer_opacity(theme: &Theme, pressed: bool, hovered: bool, focused: bool) -> f32 {
    let key = if pressed {
        "md.comp.navigation-drawer.pressed.state-layer.opacity"
    } else if focused {
        "md.comp.navigation-drawer.focus.state-layer.opacity"
    } else if hovered {
        "md.comp.navigation-drawer.hover.state-layer.opacity"
    } else {
        return 0.0;
    };
    theme.number_by_key(key).unwrap_or(0.0)
}

fn drawer_state_layer_color(theme: &Theme, active: bool, interaction: InteractionState) -> Color {
    let key = if active {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-drawer.active.focus.state-layer.color",
            InteractionState::Hovered => "md.comp.navigation-drawer.active.hover.state-layer.color",
            InteractionState::Pressed => {
                "md.comp.navigation-drawer.active.pressed.state-layer.color"
            }
            InteractionState::Default => return Color::TRANSPARENT,
        }
    } else {
        match interaction {
            InteractionState::Focused => {
                "md.comp.navigation-drawer.inactive.focus.state-layer.color"
            }
            InteractionState::Hovered => {
                "md.comp.navigation-drawer.inactive.hover.state-layer.color"
            }
            InteractionState::Pressed => {
                "md.comp.navigation-drawer.inactive.pressed.state-layer.color"
            }
            InteractionState::Default => return Color::TRANSPARENT,
        }
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
}

fn drawer_label_color(theme: &Theme, active: bool, interaction: InteractionState) -> Color {
    let key = if active {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-drawer.active.focus.label-text.color",
            InteractionState::Hovered => "md.comp.navigation-drawer.active.hover.label-text.color",
            InteractionState::Pressed => {
                "md.comp.navigation-drawer.active.pressed.label-text.color"
            }
            InteractionState::Default => "md.comp.navigation-drawer.active.label-text.color",
        }
    } else {
        match interaction {
            InteractionState::Focused => {
                "md.comp.navigation-drawer.inactive.focus.label-text.color"
            }
            InteractionState::Hovered => {
                "md.comp.navigation-drawer.inactive.hover.label-text.color"
            }
            InteractionState::Pressed => {
                "md.comp.navigation-drawer.inactive.pressed.label-text.color"
            }
            InteractionState::Default => "md.comp.navigation-drawer.inactive.label-text.color",
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

fn drawer_icon_color(theme: &Theme, active: bool, interaction: InteractionState) -> Color {
    let key = if active {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-drawer.active.focus.icon.color",
            InteractionState::Hovered => "md.comp.navigation-drawer.active.hover.icon.color",
            InteractionState::Pressed => "md.comp.navigation-drawer.active.pressed.icon.color",
            InteractionState::Default => "md.comp.navigation-drawer.active.icon.color",
        }
    } else {
        match interaction {
            InteractionState::Focused => "md.comp.navigation-drawer.inactive.focus.icon.color",
            InteractionState::Hovered => "md.comp.navigation-drawer.inactive.hover.icon.color",
            InteractionState::Pressed => "md.comp.navigation-drawer.inactive.pressed.icon.color",
            InteractionState::Default => "md.comp.navigation-drawer.inactive.icon.color",
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

fn drawer_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: &Arc<str>,
    color: Color,
    active: bool,
) -> AnyElement {
    let mut style = theme
        .text_style_by_key("md.sys.typescale.label-large")
        .unwrap_or_else(TextStyle::default);

    let weight = if active {
        theme
            .number_by_key("md.comp.navigation-drawer.active.label-text.weight")
            .unwrap_or(700.0)
    } else {
        theme
            .number_by_key("md.comp.navigation-drawer.label-text.weight")
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

fn drawer_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    icon: &IconId,
    color: Color,
) -> AnyElement {
    let size = theme
        .metric_by_key("md.comp.navigation-drawer.icon.size")
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
