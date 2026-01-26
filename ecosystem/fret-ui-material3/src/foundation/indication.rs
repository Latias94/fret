use fret_core::{Color, Corners, DrawOrder, Point, Px, Rect};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, CanvasProps};
use fret_ui::elements::ElementContext;
use fret_ui::theme::CubicBezier;

use crate::foundation::context::{MaterialRippleConfiguration, inherited_ripple_configuration};
use crate::interaction::ripple::{RippleAnimator, RippleOrigin, RipplePaintFrame};
use crate::interaction::state_layer::StateLayerAnimator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RippleClip {
    /// Clip the ripple to the component's shape (bounded ripple).
    Bounded,
    /// Do not clip the ripple (unbounded ripple).
    #[allow(dead_code)]
    Unbounded,
}

#[derive(Debug, Clone, Copy)]
pub struct IndicationConfig {
    pub state_duration_ms: u32,
    pub ripple_expand_ms: u32,
    pub ripple_fade_ms: u32,
    /// Minimum time the ripple should remain in the "pressed" state before it can be released.
    ///
    /// This matches Material Web v30 behavior, where a quick click / key press keeps the pressed
    /// ripple visible for a short minimum to avoid perceptual flicker.
    pub ripple_min_press_ms: u32,
    /// Optional override for the ripple's max radius (end size).
    ///
    /// This is used to align components like Checkbox/Radio/Switch with Compose Material 3, which
    /// specifies `radius = StateLayerSize / 2` for unbounded ripples.
    pub ripple_radius: Option<Px>,
    pub easing: CubicBezier,
}

impl Default for IndicationConfig {
    fn default() -> Self {
        Self {
            state_duration_ms: 100,
            ripple_expand_ms: 200,
            ripple_fade_ms: 100,
            ripple_min_press_ms: 225,
            ripple_radius: None,
            easing: CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
        }
    }
}

pub fn material_pressable_indication_config(
    theme: &fret_ui::Theme,
    ripple_radius: Option<Px>,
) -> IndicationConfig {
    let defaults = IndicationConfig::default();
    let state_duration_ms = theme
        .duration_ms_by_key("md.sys.motion.duration.short2")
        .unwrap_or(defaults.state_duration_ms);
    let ripple_expand_ms = theme
        .duration_ms_by_key("md.sys.motion.duration.short4")
        .unwrap_or(defaults.ripple_expand_ms);
    let ripple_fade_ms = theme
        .duration_ms_by_key("md.sys.motion.duration.short2")
        .unwrap_or(defaults.ripple_fade_ms);
    let easing = theme
        .easing_by_key("md.sys.motion.easing.standard")
        .unwrap_or(defaults.easing);

    IndicationConfig {
        state_duration_ms,
        ripple_expand_ms,
        ripple_fade_ms,
        ripple_min_press_ms: defaults.ripple_min_press_ms,
        ripple_radius,
        easing,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndicationFrame {
    pub state_layer_opacity: f32,
    pub ripple_frame: Option<RipplePaintFrame>,
    pub want_frames: bool,
}

#[derive(Default)]
struct IndicationRuntime {
    prev_pressed: bool,
    state_target: f32,
    state_layer: StateLayerAnimator,
    ripple: RippleAnimator,
    ripple_press_frame: Option<u64>,
    ripple_release_due_frame: Option<u64>,
}

pub fn advance_indication_for_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    bounds: Rect,
    last_down: Option<fret_ui::action::PointerDownCx>,
    pressed: bool,
    state_layer_target: f32,
    ripple_fallback_color: Color,
    ripple_base_opacity: f32,
    config: IndicationConfig,
) -> IndicationFrame {
    use crate::foundation::geometry::{rect_center, ripple_max_radius};
    use crate::motion::ms_to_frames;

    let ripple_config = inherited_ripple_configuration(cx);
    let mut ripple_enabled = true;
    let mut ripple_base_opacity = ripple_base_opacity;
    let mut ripple_color_override: Option<Color> = None;
    match ripple_config {
        Some(MaterialRippleConfiguration::Disabled) => ripple_enabled = false,
        Some(MaterialRippleConfiguration::Custom {
            base_opacity: Some(base_opacity),
            ..
        }) => {
            ripple_base_opacity = base_opacity;
            ripple_enabled = base_opacity > 0.0;
        }
        Some(MaterialRippleConfiguration::Custom {
            color: Some(color), ..
        }) => {
            ripple_color_override = Some(color);
        }
        Some(MaterialRippleConfiguration::UseDefault)
        | Some(MaterialRippleConfiguration::Custom {
            base_opacity: None, ..
        })
        | None => {}
    }

    let now_tick = cx.app.tick_id();
    let is_keyboard = fret_ui::input_modality::is_keyboard(&mut *cx.app, Some(cx.window));
    let last_down = (!is_keyboard)
        .then_some(last_down)
        .flatten()
        .filter(|down| now_tick.0.saturating_sub(down.tick_id.0) <= 2);

    cx.with_state_for(pressable_id, IndicationRuntime::default, |rt| {
        if (state_layer_target - rt.state_target).abs() > 1e-6 {
            rt.state_target = state_layer_target;
            rt.state_layer.set_target(
                now_frame,
                state_layer_target,
                config.state_duration_ms,
                config.easing,
            );
        }
        rt.state_layer.advance(now_frame);

        if !ripple_enabled {
            rt.ripple = RippleAnimator::default();
            rt.ripple_press_frame = None;
            rt.ripple_release_due_frame = None;
        }

        let min_press_frames = ms_to_frames(config.ripple_min_press_ms).max(1);
        if let Some(release_due) = rt.ripple_release_due_frame
            && now_frame >= release_due
        {
            rt.ripple.release(now_frame);
            rt.ripple_release_due_frame = None;
        }

        let pressed_rising = pressed && !rt.prev_pressed;
        let pressed_falling = !pressed && rt.prev_pressed;
        rt.prev_pressed = pressed;
        if pressed_rising && ripple_enabled {
            let abs_fallback_center = rect_center(bounds);
            let abs_origin_for_radius = last_down
                .map(|down| down.position)
                .unwrap_or(abs_fallback_center);
            let origin_for_paint = if is_keyboard && last_down.is_none() {
                RippleOrigin::Local(Point::new(
                    Px(bounds.size.width.0 * 0.5),
                    Px(bounds.size.height.0 * 0.5),
                ))
            } else {
                RippleOrigin::Absolute(abs_origin_for_radius)
            };
            let max_radius = config
                .ripple_radius
                .filter(|r| r.0.is_finite() && r.0 > 0.0)
                .unwrap_or_else(|| match origin_for_paint {
                    RippleOrigin::Absolute(origin) => ripple_max_radius(bounds, origin),
                    RippleOrigin::Local(origin) => {
                        let local_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), bounds.size);
                        ripple_max_radius(local_bounds, origin)
                    }
                });
            let ripple_color = ripple_color_override.unwrap_or(ripple_fallback_color);
            rt.ripple_press_frame = Some(now_frame);
            rt.ripple_release_due_frame = None;
            rt.ripple.start(
                now_frame,
                origin_for_paint,
                max_radius,
                ripple_color,
                config.ripple_expand_ms,
                config.ripple_fade_ms,
                config.easing,
            );
        }
        if pressed_falling && ripple_enabled {
            let min_release = rt
                .ripple_press_frame
                .unwrap_or(now_frame)
                .saturating_add(min_press_frames);
            if now_frame < min_release {
                rt.ripple_release_due_frame = Some(min_release);
            } else {
                rt.ripple.release(now_frame);
                rt.ripple_release_due_frame = None;
            }
        }

        let ripple_frame = ripple_enabled
            .then(|| rt.ripple.advance(now_frame, ripple_base_opacity))
            .flatten();
        let want_frames = rt.state_layer.is_active() || rt.ripple.is_active();

        IndicationFrame {
            state_layer_opacity: rt.state_layer.value(),
            ripple_frame,
            want_frames,
        }
    })
}

pub fn material_ink_layer_for_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    corner_radii: Corners,
    ripple_clip: RippleClip,
    state_layer_color: Color,
    pressed: bool,
    state_layer_target: f32,
    ripple_base_opacity: f32,
    config: IndicationConfig,
    extra_want_frames: bool,
) -> AnyElement {
    let bounds = cx
        .last_bounds_for_element(cx.root_id())
        .unwrap_or(cx.bounds);
    let last_down = cx.with_state(fret_ui::element::PointerRegionState::default, |st| {
        st.last_down
    });

    let indication = advance_indication_for_pressable(
        cx,
        pressable_id,
        now_frame,
        bounds,
        last_down,
        pressed,
        state_layer_target,
        state_layer_color,
        ripple_base_opacity,
        config,
    );

    material_ink_layer(
        cx,
        corner_radii,
        ripple_clip,
        state_layer_color,
        indication.state_layer_opacity,
        indication.ripple_frame,
        indication.want_frames || extra_want_frames,
    )
}

pub fn advance_indication_for_pressable_with_ripple_bounds<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    bounds: Rect,
    ripple_bounds: Rect,
    last_down: Option<fret_ui::action::PointerDownCx>,
    pressed: bool,
    state_layer_target: f32,
    ripple_fallback_color: Color,
    ripple_base_opacity: f32,
    config: IndicationConfig,
) -> IndicationFrame {
    use crate::foundation::geometry::{rect_center, ripple_max_radius};
    use crate::motion::ms_to_frames;

    let ripple_config = inherited_ripple_configuration(cx);
    let mut ripple_enabled = true;
    let mut ripple_base_opacity = ripple_base_opacity;
    let mut ripple_color_override: Option<Color> = None;
    match ripple_config {
        Some(MaterialRippleConfiguration::Disabled) => ripple_enabled = false,
        Some(MaterialRippleConfiguration::Custom {
            base_opacity: Some(base_opacity),
            ..
        }) => {
            ripple_base_opacity = base_opacity;
            ripple_enabled = base_opacity > 0.0;
        }
        Some(MaterialRippleConfiguration::Custom {
            color: Some(color), ..
        }) => {
            ripple_color_override = Some(color);
        }
        Some(MaterialRippleConfiguration::UseDefault)
        | Some(MaterialRippleConfiguration::Custom {
            base_opacity: None, ..
        })
        | None => {}
    }

    let now_tick = cx.app.tick_id();
    let is_keyboard = fret_ui::input_modality::is_keyboard(&mut *cx.app, Some(cx.window));
    let last_down = (!is_keyboard)
        .then_some(last_down)
        .flatten()
        .filter(|down| now_tick.0.saturating_sub(down.tick_id.0) <= 2);

    cx.with_state_for(pressable_id, IndicationRuntime::default, |rt| {
        if (state_layer_target - rt.state_target).abs() > 1e-6 {
            rt.state_target = state_layer_target;
            rt.state_layer.set_target(
                now_frame,
                state_layer_target,
                config.state_duration_ms,
                config.easing,
            );
        }
        rt.state_layer.advance(now_frame);

        if !ripple_enabled {
            rt.ripple = RippleAnimator::default();
            rt.ripple_press_frame = None;
            rt.ripple_release_due_frame = None;
        }

        let min_press_frames = ms_to_frames(config.ripple_min_press_ms).max(1);
        if let Some(release_due) = rt.ripple_release_due_frame
            && now_frame >= release_due
        {
            rt.ripple.release(now_frame);
            rt.ripple_release_due_frame = None;
        }

        let pressed_rising = pressed && !rt.prev_pressed;
        let pressed_falling = !pressed && rt.prev_pressed;
        rt.prev_pressed = pressed;
        if pressed_rising && ripple_enabled {
            let abs_ripple_bounds = Rect::new(
                fret_core::Point::new(
                    Px(bounds.origin.x.0 + ripple_bounds.origin.x.0),
                    Px(bounds.origin.y.0 + ripple_bounds.origin.y.0),
                ),
                ripple_bounds.size,
            );
            let abs_fallback_center = rect_center(abs_ripple_bounds);
            let abs_origin_for_radius = last_down
                .map(|down| down.position)
                .unwrap_or(abs_fallback_center);
            let origin_for_paint = if is_keyboard && last_down.is_none() {
                RippleOrigin::Local(Point::new(
                    Px(ripple_bounds.size.width.0 * 0.5),
                    Px(ripple_bounds.size.height.0 * 0.5),
                ))
            } else {
                RippleOrigin::Absolute(abs_origin_for_radius)
            };
            let max_radius = config
                .ripple_radius
                .filter(|r| r.0.is_finite() && r.0 > 0.0)
                .unwrap_or_else(|| match origin_for_paint {
                    RippleOrigin::Absolute(origin) => ripple_max_radius(abs_ripple_bounds, origin),
                    RippleOrigin::Local(origin) => {
                        let local_bounds =
                            Rect::new(Point::new(Px(0.0), Px(0.0)), ripple_bounds.size);
                        ripple_max_radius(local_bounds, origin)
                    }
                });
            let ripple_color = ripple_color_override.unwrap_or(ripple_fallback_color);
            rt.ripple_press_frame = Some(now_frame);
            rt.ripple_release_due_frame = None;
            rt.ripple.start(
                now_frame,
                origin_for_paint,
                max_radius,
                ripple_color,
                config.ripple_expand_ms,
                config.ripple_fade_ms,
                config.easing,
            );
        }
        if pressed_falling && ripple_enabled {
            let min_release = rt
                .ripple_press_frame
                .unwrap_or(now_frame)
                .saturating_add(min_press_frames);
            if now_frame < min_release {
                rt.ripple_release_due_frame = Some(min_release);
            } else {
                rt.ripple.release(now_frame);
                rt.ripple_release_due_frame = None;
            }
        }

        let ripple_frame = ripple_enabled
            .then(|| rt.ripple.advance(now_frame, ripple_base_opacity))
            .flatten();
        let want_frames = rt.state_layer.is_active() || rt.ripple.is_active();

        IndicationFrame {
            state_layer_opacity: rt.state_layer.value(),
            ripple_frame,
            want_frames,
        }
    })
}

pub fn material_ink_layer_for_pressable_with_ripple_bounds<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    paint_bounds: Rect,
    ripple_bounds: Rect,
    corner_radii: Corners,
    ripple_clip: RippleClip,
    state_layer_color: Color,
    pressed: bool,
    state_layer_target: f32,
    ripple_base_opacity: f32,
    config: IndicationConfig,
    extra_want_frames: bool,
) -> AnyElement {
    let bounds = cx
        .last_bounds_for_element(cx.root_id())
        .unwrap_or(cx.bounds);
    let last_down = cx.with_state(fret_ui::element::PointerRegionState::default, |st| {
        st.last_down
    });

    let indication = advance_indication_for_pressable_with_ripple_bounds(
        cx,
        pressable_id,
        now_frame,
        bounds,
        ripple_bounds,
        last_down,
        pressed,
        state_layer_target,
        state_layer_color,
        ripple_base_opacity,
        config,
    );

    material_ink_layer_with_bounds(
        cx,
        paint_bounds,
        corner_radii,
        ripple_clip,
        state_layer_color,
        indication.state_layer_opacity,
        indication.ripple_frame,
        indication.want_frames || extra_want_frames,
    )
}

pub fn material_ink_layer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    corner_radii: Corners,
    ripple_clip: RippleClip,
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
            let clip = match ripple_clip {
                RippleClip::Bounded => Some(corner_radii),
                RippleClip::Unbounded => None,
            };
            let origin = match r.origin {
                RippleOrigin::Absolute(origin) => origin,
                RippleOrigin::Local(origin) => Point::new(
                    Px(bounds.origin.x.0 + origin.x.0),
                    Px(bounds.origin.y.0 + origin.y.0),
                ),
            };
            fret_ui::paint::paint_ripple(
                p.scene(),
                DrawOrder(1),
                bounds,
                origin,
                r.radius,
                r.color,
                r.opacity,
                clip,
            );
        }

        if want_frames {
            p.request_animation_frame();
        }
    })
}

pub fn material_ink_layer_with_bounds<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    paint_bounds: Rect,
    corner_radii: Corners,
    ripple_clip: RippleClip,
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
        let abs_paint_bounds = Rect::new(
            fret_core::Point::new(
                Px(bounds.origin.x.0 + paint_bounds.origin.x.0),
                Px(bounds.origin.y.0 + paint_bounds.origin.y.0),
            ),
            paint_bounds.size,
        );

        if state_layer_opacity > 0.0 {
            fret_ui::paint::paint_state_layer(
                p.scene(),
                DrawOrder(0),
                abs_paint_bounds,
                color,
                state_layer_opacity,
                corner_radii,
            );
        }

        if let Some(r) = ripple_frame {
            let clip = match ripple_clip {
                RippleClip::Bounded => Some(corner_radii),
                RippleClip::Unbounded => None,
            };
            let origin = match r.origin {
                RippleOrigin::Absolute(origin) => origin,
                RippleOrigin::Local(origin) => Point::new(
                    Px(abs_paint_bounds.origin.x.0 + origin.x.0),
                    Px(abs_paint_bounds.origin.y.0 + origin.y.0),
                ),
            };
            fret_ui::paint::paint_ripple(
                p.scene(),
                DrawOrder(1),
                abs_paint_bounds,
                origin,
                r.radius,
                r.color,
                r.opacity,
                clip,
            );
        }

        if want_frames {
            p.request_animation_frame();
        }
    })
}
