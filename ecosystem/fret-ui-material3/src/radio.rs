//! Material 3 radio button (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.radio-button.*`.
//! - State layer (hover/pressed/focus) + bounded ripple using `fret_ui::paint`.
//! - Inner dot grow animation on selection (best-effort).

use std::sync::Arc;

use fret_core::{Color, Corners, DrawOrder, Edges, KeyCode, Px, Rect, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, Length, Overflow, PointerRegionProps, PressableA11y,
    PressableProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, Theme, UiHost};

use crate::interaction::ripple::{RippleAnimator, RipplePaintFrame};
use crate::interaction::state_layer::StateLayerAnimator;

#[derive(Debug, Clone)]
enum RadioSelectionModel {
    Bool(Model<bool>),
    Group {
        value: Arc<str>,
        selected_value: Model<Option<Arc<str>>>,
    },
}

#[derive(Clone)]
pub struct Radio {
    selection: RadioSelectionModel,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
}

impl std::fmt::Debug for Radio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Radio")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("on_activate", &self.on_activate.is_some())
            .finish()
    }
}

impl Radio {
    /// A standalone radio bound to a `Model<bool>`.
    ///
    /// Note: activation sets the model to `true` (does not toggle off).
    pub fn new(selected: Model<bool>) -> Self {
        Self {
            selection: RadioSelectionModel::Bool(selected),
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_activate: None,
        }
    }

    /// A radio item bound to a shared group-value model.
    pub fn new_value(value: impl Into<Arc<str>>, group_value: Model<Option<Arc<str>>>) -> Self {
        Self {
            selection: RadioSelectionModel::Group {
                value: value.into(),
                selected_value: group_value,
            },
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_activate: None,
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

    /// Called after the radio updates its selection model.
    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let size = radio_size_tokens(&theme);

            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;
                cx.key_add_on_key_down_for(pressable_id, consume_enter_key_handler());

                let selection_for_activate = self.selection.clone();
                let enabled_for_toggle = enabled;
                let user_activate = self.on_activate.clone();
                cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                    if enabled_for_toggle {
                        match &selection_for_activate {
                            RadioSelectionModel::Bool(m) => {
                                let _ = host.update_model(m, |v| *v = true);
                            }
                            RadioSelectionModel::Group {
                                value,
                                selected_value,
                            } => {
                                let value = value.clone();
                                let _ = host.update_model(selected_value, |v| *v = Some(value));
                            }
                        }
                        host.request_redraw(action_cx.window);
                    }
                    if let Some(h) = user_activate.as_ref() {
                        h(host, action_cx, reason);
                    }
                }));

                let checked = match &self.selection {
                    RadioSelectionModel::Bool(m) => cx
                        .get_model_copied(m, Invalidation::Layout)
                        .unwrap_or(false),
                    RadioSelectionModel::Group {
                        value,
                        selected_value,
                    } => cx
                        .get_model_cloned(selected_value, Invalidation::Layout)
                        .flatten()
                        .is_some_and(|v| v.as_ref() == value.as_ref()),
                };

                let corner_radii = Corners::all(Px(9999.0));
                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::RadioButton),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        checked: Some(checked),
                        ..Default::default()
                    },
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = Overflow::Visible;
                        l
                    },
                    focus_ring: Some(material_focus_ring(&theme, corner_radii)),
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

                        let checked = match &self.selection {
                            RadioSelectionModel::Bool(m) => {
                                cx.get_model_copied(m, Invalidation::Paint).unwrap_or(false)
                            }
                            RadioSelectionModel::Group {
                                value,
                                selected_value,
                            } => cx
                                .get_model_cloned(selected_value, Invalidation::Paint)
                                .flatten()
                                .is_some_and(|v| v.as_ref() == value.as_ref()),
                        };

                        let state_layer_target =
                            radio_state_layer_target_opacity(&theme, checked, enabled, interaction);
                        let state_layer_color =
                            radio_state_layer_color(&theme, checked, interaction);

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

                        let dot_duration_ms = theme
                            .duration_ms_by_key("md.sys.motion.duration.medium2")
                            .unwrap_or(300);
                        let dot_easing = theme
                            .easing_by_key("md.sys.motion.easing.emphasized.decelerate")
                            .unwrap_or(easing);

                        #[derive(Default)]
                        struct RadioRuntime {
                            prev_pressed: bool,
                            state_target: f32,
                            state_layer: StateLayerAnimator,
                            ripple: RippleAnimator,
                            dot_target: f32,
                            dot: StateLayerAnimator,
                        }

                        let bounds = cx
                            .last_bounds_for_element(cx.root_id())
                            .unwrap_or(cx.bounds);
                        let last_down = cx
                            .with_state(fret_ui::element::PointerRegionState::default, |st| {
                                st.last_down
                            });

                        let (state_layer_opacity, ripple_frame, dot_scale, want_frames) = cx
                            .with_state_for(pressable_id, RadioRuntime::default, |rt| {
                                if (state_layer_target - rt.state_target).abs() > 1e-6 {
                                    rt.state_target = state_layer_target;
                                    rt.state_layer.set_target(
                                        now_frame,
                                        state_layer_target,
                                        state_duration_ms,
                                        easing,
                                    );
                                }
                                rt.state_layer.advance(now_frame);

                                let desired_dot = if checked { 1.0 } else { 0.0 };
                                if (desired_dot - rt.dot_target).abs() > 1e-6 {
                                    rt.dot_target = desired_dot;
                                    rt.dot.set_target(
                                        now_frame,
                                        desired_dot,
                                        dot_duration_ms,
                                        dot_easing,
                                    );
                                }
                                rt.dot.advance(now_frame);

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

                                let ripple_base_opacity =
                                    radio_ripple_base_opacity(&theme, checked);
                                let ripple_frame =
                                    rt.ripple.advance(now_frame, ripple_base_opacity);

                                let want_frames = rt.state_layer.is_active()
                                    || rt.ripple.is_active()
                                    || rt.dot.is_active();

                                (
                                    rt.state_layer.value(),
                                    ripple_frame,
                                    rt.dot.value(),
                                    want_frames,
                                )
                            });

                        let overlay = material_ink_layer(
                            cx,
                            Corners::all(Px(9999.0)),
                            state_layer_color,
                            state_layer_opacity,
                            ripple_frame,
                            want_frames,
                        );

                        let icon_color = radio_icon_color(&theme, checked, enabled, interaction);
                        let icon = radio_icon(cx, &theme, size, checked, icon_color, dot_scale);

                        let chrome = material_radio_chrome(cx, size, vec![overlay, icon]);
                        vec![chrome]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Interaction {
    None,
    Hovered,
    Focused,
    Pressed,
}

fn interaction_state(pressed: bool, hovered: bool, focused: bool) -> Interaction {
    if pressed {
        Interaction::Pressed
    } else if focused {
        Interaction::Focused
    } else if hovered {
        Interaction::Hovered
    } else {
        Interaction::None
    }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy)]
struct RadioSizeTokens {
    icon: Px,
    state_layer: Px,
}

fn radio_size_tokens(theme: &Theme) -> RadioSizeTokens {
    let icon = theme
        .metric_by_key("md.comp.radio-button.icon.size")
        .unwrap_or(Px(20.0));
    let state_layer = theme
        .metric_by_key("md.comp.radio-button.state-layer.size")
        .unwrap_or(Px(40.0));
    RadioSizeTokens { icon, state_layer }
}

fn radio_state_layer_target_opacity(
    theme: &Theme,
    checked: bool,
    enabled: bool,
    interaction: Interaction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let group = if checked { "selected" } else { "unselected" };
    match interaction {
        Interaction::Pressed => theme
            .number_by_key(&format!(
                "md.comp.radio-button.{group}.pressed.state-layer.opacity"
            ))
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        Interaction::Focused => theme
            .number_by_key(&format!(
                "md.comp.radio-button.{group}.focus.state-layer.opacity"
            ))
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        Interaction::Hovered => theme
            .number_by_key(&format!(
                "md.comp.radio-button.{group}.hover.state-layer.opacity"
            ))
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
        Interaction::None => 0.0,
    }
}

fn radio_ripple_base_opacity(theme: &Theme, checked: bool) -> f32 {
    let group = if checked { "selected" } else { "unselected" };
    theme
        .number_by_key(&format!(
            "md.comp.radio-button.{group}.pressed.state-layer.opacity"
        ))
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}

fn radio_state_layer_color(theme: &Theme, checked: bool, interaction: Interaction) -> Color {
    let group = if checked { "selected" } else { "unselected" };
    let key = match interaction {
        Interaction::Pressed => format!("md.comp.radio-button.{group}.pressed.state-layer.color"),
        Interaction::Focused => format!("md.comp.radio-button.{group}.focus.state-layer.color"),
        Interaction::Hovered => format!("md.comp.radio-button.{group}.hover.state-layer.color"),
        Interaction::None => format!("md.comp.radio-button.{group}.hover.state-layer.color"),
    };
    theme.color_by_key(&key).unwrap_or_else(|| {
        theme
            .color_by_key("md.sys.color.primary")
            .unwrap_or_else(|| theme.color_required("color.accent"))
    })
}

fn radio_icon_color(
    theme: &Theme,
    checked: bool,
    enabled: bool,
    interaction: Interaction,
) -> Color {
    if !enabled {
        let (color_key, opacity_key) = if checked {
            (
                "md.comp.radio-button.disabled.selected.icon.color",
                "md.comp.radio-button.disabled.selected.icon.opacity",
            )
        } else {
            (
                "md.comp.radio-button.disabled.unselected.icon.color",
                "md.comp.radio-button.disabled.unselected.icon.opacity",
            )
        };

        let base = theme
            .color_by_key(color_key)
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("foreground"));
        let opacity = theme.number_by_key(opacity_key).unwrap_or(0.38);
        return alpha_mul(base, opacity);
    }

    let group = if checked { "selected" } else { "unselected" };
    let key = match interaction {
        Interaction::Pressed => format!("md.comp.radio-button.{group}.pressed.icon.color"),
        Interaction::Focused => format!("md.comp.radio-button.{group}.focus.icon.color"),
        Interaction::Hovered => format!("md.comp.radio-button.{group}.hover.icon.color"),
        Interaction::None => format!("md.comp.radio-button.{group}.icon.color"),
    };

    theme.color_by_key(&key).unwrap_or_else(|| {
        theme
            .color_by_key("md.sys.color.primary")
            .unwrap_or_else(|| theme.color_required("color.accent"))
    })
}

fn material_radio_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: RadioSizeTokens,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.corner_radii = Corners::all(Px(9999.0));
    props.layout.size.width = Length::Px(size.state_layer);
    props.layout.size.height = Length::Px(size.state_layer);
    cx.container(props, move |_cx| children)
}

fn radio_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _theme: &Theme,
    size: RadioSizeTokens,
    checked: bool,
    color: Color,
    dot_scale: f32,
) -> AnyElement {
    let outline_width = Px(2.0);
    let dot_max = Px(10.0);
    let dot_size = Px(dot_max.0 * dot_scale.clamp(0.0, 1.0));
    let dot_inset = Px(((size.icon.0 - dot_size.0) * 0.5).max(0.0));

    let mut outer = ContainerProps::default();
    outer.layout.size.width = Length::Px(size.state_layer);
    outer.layout.size.height = Length::Px(size.state_layer);
    outer.layout.overflow = Overflow::Visible;
    outer.corner_radii = Corners::all(Px(9999.0));
    cx.container(outer, move |cx| {
        let mut icon = ContainerProps::default();
        icon.layout.position = fret_ui::element::PositionStyle::Absolute;
        icon.layout.inset.left = Some(Px((size.state_layer.0 - size.icon.0) * 0.5));
        icon.layout.inset.top = Some(Px((size.state_layer.0 - size.icon.0) * 0.5));
        icon.layout.size.width = Length::Px(size.icon);
        icon.layout.size.height = Length::Px(size.icon);
        icon.corner_radii = Corners::all(Px(9999.0));
        icon.background = Some(Color::TRANSPARENT);
        icon.border = Edges::all(outline_width);
        icon.border_color = Some(color);

        let dot = if checked || dot_size.0 > 0.1 {
            let mut dot_props = ContainerProps::default();
            dot_props.layout.position = fret_ui::element::PositionStyle::Absolute;
            dot_props.layout.inset.left = Some(dot_inset);
            dot_props.layout.inset.top = Some(dot_inset);
            dot_props.layout.size.width = Length::Px(dot_size);
            dot_props.layout.size.height = Length::Px(dot_size);
            dot_props.corner_radii = Corners::all(Px(9999.0));
            dot_props.background = Some(color);
            vec![cx.container(dot_props, |_cx| Vec::new())]
        } else {
            Vec::new()
        };

        vec![cx.container(icon, move |_cx| dot)]
    })
}

fn material_ink_layer<H: UiHost>(
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

fn material_focus_ring(theme: &Theme, corner_radii: Corners) -> fret_ui::element::RingStyle {
    let mut c = theme
        .color_by_key("md.sys.color.primary")
        .or_else(|| theme.color_by_key("color.accent"))
        .unwrap_or_else(|| theme.color_required("color.accent"));
    c.a = 1.0;

    fret_ui::element::RingStyle {
        placement: fret_ui::element::RingPlacement::Outset,
        width: Px(2.0),
        offset: Px(2.0),
        color: c,
        offset_color: None,
        corner_radii,
    }
}

fn consume_enter_key_handler() -> fret_ui::action::OnKeyDown {
    Arc::new(|_host, _cx, down| matches!(down.key, KeyCode::Enter | KeyCode::NumpadEnter))
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
