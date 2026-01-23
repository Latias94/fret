//! Material 3 switch (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.switch.*`.
//! - State layer (hover/pressed/focus) + bounded ripple using `fret_ui::paint`.

use std::sync::Arc;

use fret_core::{Color, Corners, DrawOrder, Edges, KeyCode, Px, Rect, SemanticsRole, Size};
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

#[derive(Clone)]
pub struct Switch {
    selected: Model<bool>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
}

impl std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("on_activate", &self.on_activate.is_some())
            .finish()
    }
}

impl Switch {
    pub fn new(selected: Model<bool>) -> Self {
        Self {
            selected,
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

    /// Called after the switch toggles its `Model<bool>`.
    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let size = switch_size_tokens(&theme);

            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;
                cx.key_add_on_key_down_for(pressable_id, consume_enter_key_handler());

                let selected_model_for_toggle = self.selected.clone();
                let enabled_for_toggle = enabled;
                let user_activate = self.on_activate.clone();
                cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                    if enabled_for_toggle {
                        let _ = host.update_model(&selected_model_for_toggle, |v| *v = !*v);
                        host.request_redraw(action_cx.window);
                    }
                    if let Some(h) = user_activate.as_ref() {
                        h(host, action_cx, reason);
                    }
                }));

                let corner_radii = Corners::all(Px(9999.0));
                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Switch),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        checked: Some(
                            cx.get_model_copied(&self.selected, Invalidation::Layout)
                                .unwrap_or(false),
                        ),
                        ..Default::default()
                    },
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = Overflow::Visible;
                        l
                    },
                    focus_ring: Some(material_focus_ring(&theme, size, corner_radii)),
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
                        let selected = cx
                            .get_model_copied(&self.selected, Invalidation::Paint)
                            .unwrap_or(false);

                        let interaction = interaction_state(is_pressed, is_hovered, is_focused);

                        let state_layer_target = switch_state_layer_target_opacity(
                            &theme,
                            selected,
                            enabled,
                            interaction,
                        );
                        let state_layer_color =
                            switch_state_layer_color(&theme, selected, interaction);

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

                        let switch_duration_ms = theme
                            .duration_ms_by_key("md.sys.motion.duration.short3")
                            .unwrap_or(150);

                        #[derive(Default)]
                        struct SwitchRuntime {
                            prev_pressed: bool,
                            state_target: f32,
                            state_layer: StateLayerAnimator,
                            ripple: RippleAnimator,
                            thumb_target: f32,
                            thumb: StateLayerAnimator,
                        }

                        let last_down = cx
                            .with_state(fret_ui::element::PointerRegionState::default, |st| {
                                st.last_down
                            });

                        let (state_layer_opacity, ripple_frame, thumb_t, want_frames) = cx
                            .with_state_for(pressable_id, SwitchRuntime::default, |rt| {
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

                                let desired_thumb = if selected { 1.0 } else { 0.0 };
                                if (desired_thumb - rt.thumb_target).abs() > 1e-6 {
                                    rt.thumb_target = desired_thumb;
                                    rt.thumb.set_target(
                                        now_frame,
                                        desired_thumb,
                                        switch_duration_ms,
                                        easing,
                                    );
                                }
                                rt.thumb.advance(now_frame);
                                let thumb_t = rt.thumb.value();

                                // Compute the current ink circle so the ripple origin clips to it.
                                let geom = switch_geometry(size, thumb_t, is_pressed);
                                let ink_bounds = geom.ink_bounds;

                                let pressed_rising = is_pressed && !rt.prev_pressed;
                                rt.prev_pressed = is_pressed;
                                if pressed_rising {
                                    let origin = down_origin_local(ink_bounds, last_down);
                                    let max_radius = ripple_max_radius(ink_bounds, origin);
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
                                    switch_ripple_base_opacity(&theme, selected);
                                let ripple_frame =
                                    rt.ripple.advance(now_frame, ripple_base_opacity);

                                let want_frames = rt.state_layer.is_active()
                                    || rt.ripple.is_active()
                                    || rt.thumb.is_active();

                                (rt.state_layer.value(), ripple_frame, thumb_t, want_frames)
                            });

                        let geom = switch_geometry(size, thumb_t, is_pressed);
                        let track =
                            switch_track(cx, &theme, size, selected, enabled, interaction, geom);

                        let overlay = material_ink_layer(
                            cx,
                            geom.ink_bounds,
                            state_layer_color,
                            state_layer_opacity,
                            ripple_frame,
                            want_frames,
                        );

                        let mut outer = ContainerProps::default();
                        outer.layout.size.width = Length::Px(size.track_width);
                        outer.layout.size.height = Length::Px(size.state_layer);
                        outer.layout.overflow = Overflow::Visible;
                        outer.padding = Edges {
                            top: size.track_y_offset,
                            right: Px(0.0),
                            bottom: size.track_y_offset,
                            left: Px(0.0),
                        };
                        outer.corner_radii = Corners::all(Px(0.0));

                        let chrome = cx.container(outer, move |_cx| vec![overlay, track]);
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
struct SwitchSizeTokens {
    state_layer: Px,
    track_width: Px,
    track_height: Px,
    track_outline_width: Px,
    selected_handle: Px,
    unselected_handle: Px,
    pressed_handle: Px,
    track_y_offset: Px,
    focus_indicator_thickness: Px,
    focus_indicator_offset: Px,
}

fn switch_size_tokens(theme: &Theme) -> SwitchSizeTokens {
    let state_layer = theme
        .metric_by_key("md.comp.switch.state-layer.size")
        .unwrap_or(Px(40.0));
    let track_width = theme
        .metric_by_key("md.comp.switch.track.width")
        .unwrap_or(Px(52.0));
    let track_height = theme
        .metric_by_key("md.comp.switch.track.height")
        .unwrap_or(Px(32.0));
    let track_outline_width = theme
        .metric_by_key("md.comp.switch.track.outline.width")
        .unwrap_or(Px(2.0));
    let selected_handle = theme
        .metric_by_key("md.comp.switch.selected.handle.size")
        .unwrap_or(Px(24.0));
    let unselected_handle = theme
        .metric_by_key("md.comp.switch.unselected.handle.size")
        .unwrap_or(Px(16.0));
    let pressed_handle = theme
        .metric_by_key("md.comp.switch.pressed.handle.size")
        .unwrap_or(Px(28.0));

    let track_y_offset = Px(((state_layer.0 - track_height.0) * 0.5).max(0.0));

    let focus_indicator_thickness = theme
        .metric_by_key("md.comp.switch.focus.indicator.thickness")
        .or_else(|| theme.metric_by_key("md.sys.state.focus-indicator.thickness"))
        .unwrap_or(Px(2.0));
    let focus_indicator_offset = theme
        .metric_by_key("md.comp.switch.focus.indicator.offset")
        .or_else(|| theme.metric_by_key("md.sys.state.focus-indicator.outer-offset"))
        .unwrap_or(Px(2.0));

    SwitchSizeTokens {
        state_layer,
        track_width,
        track_height,
        track_outline_width,
        selected_handle,
        unselected_handle,
        pressed_handle,
        track_y_offset,
        focus_indicator_thickness,
        focus_indicator_offset,
    }
}

fn switch_state_layer_target_opacity(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Interaction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let group = if selected { "selected" } else { "unselected" };
    match interaction {
        Interaction::Pressed => theme
            .number_by_key(&format!(
                "md.comp.switch.{group}.pressed.state-layer.opacity"
            ))
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        Interaction::Focused => theme
            .number_by_key(&format!("md.comp.switch.{group}.focus.state-layer.opacity"))
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        Interaction::Hovered => theme
            .number_by_key(&format!("md.comp.switch.{group}.hover.state-layer.opacity"))
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
        Interaction::None => 0.0,
    }
}

fn switch_ripple_base_opacity(theme: &Theme, selected: bool) -> f32 {
    let group = if selected { "selected" } else { "unselected" };
    theme
        .number_by_key(&format!(
            "md.comp.switch.{group}.pressed.state-layer.opacity"
        ))
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}

fn switch_state_layer_color(theme: &Theme, selected: bool, interaction: Interaction) -> Color {
    let group = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Interaction::Pressed => format!("md.comp.switch.{group}.pressed.state-layer.color"),
        Interaction::Focused => format!("md.comp.switch.{group}.focus.state-layer.color"),
        Interaction::Hovered => format!("md.comp.switch.{group}.hover.state-layer.color"),
        Interaction::None => format!("md.comp.switch.{group}.hover.state-layer.color"),
    };

    theme.color_by_key(&key).unwrap_or_else(|| {
        theme
            .color_by_key("md.sys.color.primary")
            .unwrap_or_else(|| theme.color_required("color.accent"))
    })
}

#[derive(Debug, Clone, Copy)]
struct SwitchGeometry {
    handle_x: Px,
    handle_y: Px,
    handle_size: Px,
    ink_bounds: Rect,
}

fn switch_geometry(size: SwitchSizeTokens, thumb_t: f32, pressed: bool) -> SwitchGeometry {
    let thumb_t = thumb_t.clamp(0.0, 1.0);
    let handle_size =
        if pressed {
            size.pressed_handle
        } else {
            Px(size.unselected_handle.0
                + (size.selected_handle.0 - size.unselected_handle.0) * thumb_t)
        };

    let padding = Px(((size.track_height.0 - handle_size.0) * 0.5).max(0.0));
    let on_x = Px(size.track_width.0 - handle_size.0 - padding.0);
    let off_x = padding;
    let handle_x = Px(off_x.0 + (on_x.0 - off_x.0) * thumb_t);
    let handle_y = padding;

    let thumb_cx = Px(handle_x.0 + handle_size.0 * 0.5);
    let thumb_cy = Px(size.track_y_offset.0 + handle_y.0 + handle_size.0 * 0.5);

    let ink_origin = fret_core::Point::new(
        Px(thumb_cx.0 - size.state_layer.0 * 0.5),
        Px(thumb_cy.0 - size.state_layer.0 * 0.5),
    );
    let ink_bounds = Rect::new(ink_origin, Size::new(size.state_layer, size.state_layer));

    SwitchGeometry {
        handle_x,
        handle_y,
        handle_size,
        ink_bounds,
    }
}

fn switch_track<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    size: SwitchSizeTokens,
    selected: bool,
    enabled: bool,
    interaction: Interaction,
    geom: SwitchGeometry,
) -> AnyElement {
    let colors = switch_chrome(theme, selected, enabled, interaction);

    let mut track = ContainerProps::default();
    track.layout.size.width = Length::Px(size.track_width);
    track.layout.size.height = Length::Px(size.track_height);
    track.layout.overflow = Overflow::Visible;
    track.background = Some(colors.track_color);
    track.corner_radii = Corners::all(Px(9999.0));
    if let Some(outline) = colors.outline_color {
        track.border = Edges::all(size.track_outline_width);
        track.border_color = Some(outline);
    }

    cx.container(track, move |cx| {
        let mut handle = ContainerProps::default();
        handle.layout.position = fret_ui::element::PositionStyle::Absolute;
        handle.layout.inset.left = Some(geom.handle_x);
        handle.layout.inset.top = Some(geom.handle_y);
        handle.layout.size.width = Length::Px(geom.handle_size);
        handle.layout.size.height = Length::Px(geom.handle_size);
        handle.corner_radii = Corners::all(Px(9999.0));
        handle.background = Some(colors.handle_color);

        vec![cx.container(handle, |_cx| Vec::new())]
    })
}

#[derive(Debug, Clone, Copy)]
struct SwitchChrome {
    track_color: Color,
    outline_color: Option<Color>,
    handle_color: Color,
}

fn switch_chrome(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Interaction,
) -> SwitchChrome {
    if !enabled {
        let track_base = if selected {
            theme.color_by_key("md.comp.switch.disabled.selected.track.color")
        } else {
            theme.color_by_key("md.comp.switch.disabled.unselected.track.color")
        }
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("color.accent"));

        let track_opacity = theme
            .number_by_key("md.comp.switch.disabled.track.opacity")
            .unwrap_or(0.12);
        let track_color = alpha_mul(track_base, track_opacity);

        let handle_base = if selected {
            theme
                .color_by_key("md.comp.switch.disabled.selected.handle.color")
                .or_else(|| theme.color_by_key("md.sys.color.surface"))
        } else {
            theme
                .color_by_key("md.comp.switch.disabled.unselected.handle.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        }
        .unwrap_or_else(|| theme.color_required("foreground"));

        let handle_opacity = if selected {
            theme.number_by_key("md.comp.switch.disabled.selected.handle.opacity")
        } else {
            theme.number_by_key("md.comp.switch.disabled.unselected.handle.opacity")
        }
        .unwrap_or(0.38);
        let handle_color = alpha_mul(handle_base, handle_opacity);

        let outline_color = if selected {
            None
        } else {
            theme
                .color_by_key("md.comp.switch.disabled.unselected.track.outline.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                .map(|c| alpha_mul(c, handle_opacity))
        };

        return SwitchChrome {
            track_color,
            outline_color,
            handle_color,
        };
    }

    let group = if selected { "selected" } else { "unselected" };
    let state = match interaction {
        Interaction::Pressed => "pressed",
        Interaction::Focused => "focus",
        Interaction::Hovered => "hover",
        Interaction::None => "",
    };

    let track_key = if state.is_empty() {
        format!("md.comp.switch.{group}.track.color")
    } else {
        format!("md.comp.switch.{group}.{state}.track.color")
    };
    let handle_key = if state.is_empty() {
        format!("md.comp.switch.{group}.handle.color")
    } else {
        format!("md.comp.switch.{group}.{state}.handle.color")
    };

    let track_color = theme.color_by_key(&track_key).unwrap_or_else(|| {
        if selected {
            theme
                .color_by_key("md.sys.color.primary")
                .unwrap_or_else(|| theme.color_required("color.accent"))
        } else {
            theme
                .color_by_key("md.sys.color.surface-container-highest")
                .unwrap_or_else(|| theme.color_required("background"))
        }
    });

    let handle_color = theme.color_by_key(&handle_key).unwrap_or_else(|| {
        if selected {
            theme
                .color_by_key("md.sys.color.on-primary")
                .unwrap_or_else(|| theme.color_required("foreground"))
        } else {
            theme
                .color_by_key("md.sys.color.outline")
                .unwrap_or_else(|| theme.color_required("foreground"))
        }
    });

    let outline_color = if selected {
        None
    } else {
        let outline_key = if state.is_empty() {
            "md.comp.switch.unselected.track.outline.color".to_string()
        } else {
            format!("md.comp.switch.unselected.{state}.track.outline.color")
        };
        Some(
            theme
                .color_by_key(&outline_key)
                .or_else(|| theme.color_by_key("md.sys.color.outline"))
                .unwrap_or_else(|| theme.color_required("border")),
        )
    };

    SwitchChrome {
        track_color,
        outline_color,
        handle_color,
    }
}

fn material_ink_layer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    ink_bounds: Rect,
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

    let corner_radii = Corners::all(Px(9999.0));
    cx.canvas(props, move |p| {
        if state_layer_opacity > 0.0 {
            fret_ui::paint::paint_state_layer(
                p.scene(),
                DrawOrder(0),
                ink_bounds,
                color,
                state_layer_opacity,
                corner_radii,
            );
        }

        if let Some(r) = ripple_frame {
            fret_ui::paint::paint_ripple(
                p.scene(),
                DrawOrder(1),
                ink_bounds,
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

fn material_focus_ring(
    theme: &Theme,
    size: SwitchSizeTokens,
    corner_radii: Corners,
) -> fret_ui::element::RingStyle {
    let mut c = theme
        .color_by_key("md.comp.switch.focus.indicator.color")
        .or_else(|| theme.color_by_key("md.sys.color.secondary"))
        .unwrap_or_else(|| theme.color_required("color.accent"));
    c.a = 1.0;

    fret_ui::element::RingStyle {
        placement: fret_ui::element::RingPlacement::Outset,
        width: size.focus_indicator_thickness,
        offset: size.focus_indicator_offset,
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
