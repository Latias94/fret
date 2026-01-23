//! Material 3 tabs (primary navigation) (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.primary-navigation-tab.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to the tab bounds.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, DrawOrder, Edges, KeyCode, Px, Rect, SemanticsRole, TextOverflow,
    TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, SemanticsProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, Theme, UiHost};

use crate::interaction::ripple::{RippleAnimator, RipplePaintFrame};
use crate::interaction::state_layer::StateLayerAnimator;

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

#[derive(Debug, Clone)]
pub struct Tabs {
    model: Model<Arc<str>>,
    items: Vec<TabItem>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    disabled: bool,
    loop_navigation: bool,
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Tabs {
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

            let container_height = theme
                .metric_by_key("md.comp.primary-navigation-tab.container.height")
                .unwrap_or(Px(48.0));
            let container_bg = theme
                .color_by_key("md.comp.primary-navigation-tab.container.color")
                .or_else(|| theme.color_by_key("md.sys.color.surface"))
                .unwrap_or_else(|| theme.color_required("card"));

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
                        vec![cx.roving_flex(props, move |cx| {
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

fn material_primary_tab<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    model: Model<Arc<str>>,
    item: &TabItem,
    idx: usize,
    set_size: usize,
    tab_stop: bool,
    disabled_group: bool,
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

        if enabled {
            let model_for_press = model.clone();
            let value_for_press = value.clone();
            let handler: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host.update_model(&model_for_press, |v| *v = value_for_press.clone());
                host.request_redraw(action_cx.window);
            });
            cx.pressable_on_activate(handler);
        }

        let corner_radii = Corners::all(Px(0.0));
        let height = theme
            .metric_by_key("md.comp.primary-navigation-tab.container.height")
            .unwrap_or(Px(48.0));

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
            focus_ring: Some(primary_tab_focus_ring(theme, corner_radii)),
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
                let label_color = primary_tab_label_color(theme, selected, interaction);
                let state_layer_color = primary_tab_state_layer_color(theme, selected, interaction);
                let state_layer_target =
                    primary_tab_state_layer_opacity(theme, selected, is_pressed, is_hovered, is_focused);

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

                #[derive(Default)]
                struct TabRuntime {
                    prev_pressed: bool,
                    state_target: f32,
                    state_layer: StateLayerAnimator,
                    ripple: RippleAnimator,
                }

                let bounds = cx
                    .last_bounds_for_element(cx.root_id())
                    .unwrap_or(cx.bounds);
                let last_down = cx
                    .with_state(fret_ui::element::PointerRegionState::default, |st| st.last_down);

                let (state_layer_opacity, ripple_frame, want_frames) = cx.with_state_for(
                    pressable_id,
                    TabRuntime::default,
                    |rt| {
                        if (state_layer_target - rt.state_target).abs() > 1e-6 {
                            rt.state_target = state_layer_target;
                            rt.state_layer
                                .set_target(now_frame, state_layer_target, state_duration_ms, easing);
                        }
                        rt.state_layer.advance(now_frame);

                        let pressed_rising = is_pressed && !rt.prev_pressed;
                        rt.prev_pressed = is_pressed;
                        if pressed_rising {
                            let origin = down_origin_local(bounds, last_down);
                            let max_radius = ripple_max_radius(bounds, origin);
                            rt.ripple.start(
                                now_frame,
                                origin,
                                max_radius,
                                ripple_expand_ms,
                                ripple_fade_ms,
                                easing,
                            );
                        }

                        let ripple_base_opacity = theme
                            .number_by_key(if selected {
                                "md.comp.primary-navigation-tab.active.pressed.state-layer.opacity"
                            } else {
                                "md.comp.primary-navigation-tab.inactive.pressed.state-layer.opacity"
                            })
                            .unwrap_or(0.1);
                        let ripple_frame = rt.ripple.advance(now_frame, ripple_base_opacity);
                        let want_frames = rt.state_layer.is_active() || rt.ripple.is_active();
                        (rt.state_layer.value(), ripple_frame, want_frames)
                    },
                );

                let ink = primary_tab_ink_layer(
                    cx,
                    corner_radii,
                    state_layer_color,
                    state_layer_opacity,
                    ripple_frame,
                    want_frames,
                );
                let label_el = primary_tab_label(cx, theme, &label, label_color);
                let indicator = selected.then(|| primary_tab_indicator(cx, theme));

                let mut row = FlexProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Px(height);
                row.layout.overflow = Overflow::Clip;
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::Center;
                row.align = CrossAlign::Center;
                row.padding = Edges::all(Px(0.0));

                vec![cx.flex(row, move |_cx| {
                    let mut children: Vec<AnyElement> = vec![ink, label_el];
                    if let Some(indicator) = indicator.clone() {
                        children.push(indicator);
                    }
                    children
                })]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn primary_tab_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    label: &Arc<str>,
    color: Color,
) -> AnyElement {
    let style = theme
        .text_style_by_key("md.sys.typescale.title-small")
        .unwrap_or_else(|| TextStyle::default());

    let mut props = TextProps::new(label.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}

fn primary_tab_indicator<H: UiHost>(cx: &mut ElementContext<'_, H>, theme: &Theme) -> AnyElement {
    let height = theme
        .metric_by_key("md.comp.primary-navigation-tab.active-indicator.height")
        .unwrap_or(Px(3.0));
    let color = theme
        .color_by_key("md.comp.primary-navigation-tab.active-indicator.color")
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("foreground"));
    let radius = theme
        .metric_by_key("md.comp.primary-navigation-tab.active-indicator.shape")
        .unwrap_or(Px(3.0));

    let mut props = fret_ui::element::ContainerProps::default();
    props.layout.position = fret_ui::element::PositionStyle::Absolute;
    props.layout.inset.left = Some(Px(0.0));
    props.layout.inset.right = Some(Px(0.0));
    props.layout.inset.bottom = Some(Px(0.0));
    props.layout.size.height = Length::Px(height);
    props.background = Some(color);
    props.corner_radii = Corners {
        top_left: radius,
        top_right: radius,
        bottom_right: Px(0.0),
        bottom_left: Px(0.0),
    };
    cx.container(props, |_cx| vec![])
}

fn primary_tab_ink_layer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    corner_radii: Corners,
    color: Color,
    state_layer_opacity: f32,
    ripple_frame: Option<RipplePaintFrame>,
    want_frames: bool,
) -> AnyElement {
    let mut props = CanvasProps::default();
    props.layout.position = fret_ui::element::PositionStyle::Absolute;
    props.layout.inset.top = Some(Px(0.0));
    props.layout.inset.right = Some(Px(0.0));
    props.layout.inset.bottom = Some(Px(0.0));
    props.layout.inset.left = Some(Px(0.0));

    cx.canvas(props, move |p| {
        let bounds = p.bounds();

        if state_layer_opacity > 0.0 {
            fret_ui::paint::paint_state_layer(
                p.scene(),
                DrawOrder(0),
                bounds,
                color,
                state_layer_opacity,
                corner_radii,
            );
        }

        if let Some(r) = ripple_frame {
            fret_ui::paint::paint_ripple(
                p.scene(),
                DrawOrder(1),
                bounds,
                r.origin,
                r.radius,
                color,
                r.opacity,
                Some(corner_radii),
            );
        }

        if want_frames {
            p.request_animation_frame();
        }
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

fn primary_tab_label_color(theme: &Theme, active: bool, interaction: InteractionState) -> Color {
    let key = if active {
        match interaction {
            InteractionState::Focused => {
                "md.comp.primary-navigation-tab.with-label-text.active.focus.label-text.color"
            }
            InteractionState::Hovered => {
                "md.comp.primary-navigation-tab.with-label-text.active.hover.label-text.color"
            }
            InteractionState::Pressed => {
                "md.comp.primary-navigation-tab.with-label-text.active.pressed.label-text.color"
            }
            InteractionState::Default => {
                "md.comp.primary-navigation-tab.with-label-text.active.label-text.color"
            }
        }
    } else {
        match interaction {
            InteractionState::Focused => {
                "md.comp.primary-navigation-tab.with-label-text.inactive.focus.label-text.color"
            }
            InteractionState::Hovered => {
                "md.comp.primary-navigation-tab.with-label-text.inactive.hover.label-text.color"
            }
            InteractionState::Pressed => {
                "md.comp.primary-navigation-tab.with-label-text.inactive.pressed.label-text.color"
            }
            InteractionState::Default => {
                "md.comp.primary-navigation-tab.with-label-text.inactive.label-text.color"
            }
        }
    };

    theme
        .color_by_key(key)
        .or_else(|| {
            if active {
                theme.color_by_key("md.sys.color.primary")
            } else {
                theme.color_by_key("md.sys.color.on-surface-variant")
            }
        })
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn primary_tab_state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: InteractionState,
) -> Color {
    let key = if active {
        match interaction {
            InteractionState::Focused => {
                "md.comp.primary-navigation-tab.active.focus.state-layer.color"
            }
            InteractionState::Hovered => {
                "md.comp.primary-navigation-tab.active.hover.state-layer.color"
            }
            InteractionState::Pressed => {
                "md.comp.primary-navigation-tab.active.pressed.state-layer.color"
            }
            InteractionState::Default => {
                "md.comp.primary-navigation-tab.active.hover.state-layer.color"
            }
        }
    } else {
        match interaction {
            InteractionState::Focused => {
                "md.comp.primary-navigation-tab.inactive.focus.state-layer.color"
            }
            InteractionState::Hovered => {
                "md.comp.primary-navigation-tab.inactive.hover.state-layer.color"
            }
            InteractionState::Pressed => {
                "md.comp.primary-navigation-tab.inactive.pressed.state-layer.color"
            }
            InteractionState::Default => {
                "md.comp.primary-navigation-tab.inactive.hover.state-layer.color"
            }
        }
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn primary_tab_state_layer_opacity(
    theme: &Theme,
    active: bool,
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> f32 {
    if pressed {
        return theme
            .number_by_key(if active {
                "md.comp.primary-navigation-tab.active.pressed.state-layer.opacity"
            } else {
                "md.comp.primary-navigation-tab.inactive.pressed.state-layer.opacity"
            })
            .unwrap_or(0.1);
    }
    if focused {
        return theme
            .number_by_key(if active {
                "md.comp.primary-navigation-tab.active.focus.state-layer.opacity"
            } else {
                "md.comp.primary-navigation-tab.inactive.focus.state-layer.opacity"
            })
            .unwrap_or(0.1);
    }
    if hovered {
        return theme
            .number_by_key(if active {
                "md.comp.primary-navigation-tab.active.hover.state-layer.opacity"
            } else {
                "md.comp.primary-navigation-tab.inactive.hover.state-layer.opacity"
            })
            .unwrap_or(0.08);
    }
    0.0
}

fn primary_tab_focus_ring(theme: &Theme, corner_radii: Corners) -> fret_ui::element::RingStyle {
    let mut c = theme
        .color_by_key("md.comp.primary-navigation-tab.focus.indicator.color")
        .or_else(|| theme.color_by_key("md.sys.color.secondary"))
        .unwrap_or_else(|| theme.color_required("color.accent"));

    let opacity = theme
        .number_by_key("md.sys.state.focus-indicator.opacity")
        .unwrap_or(1.0);
    c.a *= opacity;

    let width = theme
        .metric_by_key("md.comp.primary-navigation-tab.focus.indicator.thickness")
        .or_else(|| theme.metric_by_key("md.sys.state.focus-indicator.thickness"))
        .unwrap_or(Px(3.0));

    let offset = theme
        .metric_by_key("md.comp.primary-navigation-tab.focus.indicator.outline.offset")
        .or_else(|| theme.metric_by_key("md.sys.state.focus-indicator.inner-offset"))
        .unwrap_or(Px(0.0));

    fret_ui::element::RingStyle {
        placement: fret_ui::element::RingPlacement::Inset,
        width,
        offset,
        color: c,
        offset_color: None,
        corner_radii,
    }
}

fn down_origin_local(
    bounds: Rect,
    down: Option<fret_ui::action::PointerDownCx>,
) -> fret_core::Point {
    let pos = down.map(|d| d.position).unwrap_or_else(|| {
        fret_core::Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
            Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
        )
    });
    fret_core::Point::new(
        Px(pos.x.0 - bounds.origin.x.0),
        Px(pos.y.0 - bounds.origin.y.0),
    )
}

fn ripple_max_radius(bounds: Rect, origin_local: fret_core::Point) -> Px {
    let w = bounds.size.width.0.max(0.0);
    let h = bounds.size.height.0.max(0.0);
    let ox = origin_local.x.0.clamp(0.0, w);
    let oy = origin_local.y.0.clamp(0.0, h);
    let corners = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)];
    let mut max: f32 = 0.0;
    for (cx, cy) in corners {
        let dx = cx - ox;
        let dy = cy - oy;
        max = max.max((dx * dx + dy * dy).sqrt());
    }
    Px(max)
}
