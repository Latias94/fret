use std::cell::RefCell;
use std::rc::Rc;

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
struct IndicationFrame {
    state_layer_opacity: f32,
    ripple_frame: Option<RipplePaintFrame>,
    want_frames: bool,
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

#[derive(Debug, Clone, Copy)]
struct ResolvedRipplePolicy {
    enabled: bool,
    base_opacity: f32,
    color_override: Option<Color>,
}

impl ResolvedRipplePolicy {
    fn color(self, fallback: Color) -> Color {
        self.color_override.unwrap_or(fallback)
    }
}

type SharedIndicationRuntime = Rc<RefCell<IndicationRuntime>>;

impl IndicationRuntime {
    #[allow(clippy::too_many_arguments)]
    fn update_pressable(
        &mut self,
        now_frame: u64,
        bounds: Rect,
        ripple_bounds: Rect,
        last_down: Option<fret_ui::action::PointerDownCx>,
        pressed: bool,
        state_layer_target: f32,
        ripple_fallback_color: Color,
        ripple: ResolvedRipplePolicy,
        config: IndicationConfig,
        is_keyboard: bool,
    ) {
        use crate::foundation::geometry::{rect_center, ripple_max_radius};
        use crate::motion::ms_to_frames;

        // Bring the retained paint-time state up to the render frame before retargeting. This
        // keeps interrupted hover/press fades continuous when the view-cache subtree is reused
        // between input edges.
        self.state_layer.advance(now_frame);
        if (state_layer_target - self.state_target).abs() > 1e-6 {
            self.state_target = state_layer_target;
            self.state_layer.set_target(
                now_frame,
                state_layer_target,
                config.state_duration_ms,
                config.easing,
            );
        }

        if !ripple.enabled {
            self.ripple = RippleAnimator::default();
            self.ripple_press_frame = None;
            self.ripple_release_due_frame = None;
        }

        let min_press_frames = ms_to_frames(config.ripple_min_press_ms).max(1);
        if let Some(release_due) = self.ripple_release_due_frame
            && now_frame >= release_due
        {
            self.ripple.release(now_frame);
            self.ripple_release_due_frame = None;
        }

        let pressed_rising = pressed && !self.prev_pressed;
        let pressed_falling = !pressed && self.prev_pressed;
        self.prev_pressed = pressed;

        if pressed_rising && ripple.enabled {
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
            self.ripple_press_frame = Some(now_frame);
            self.ripple_release_due_frame = None;
            self.ripple.start(
                now_frame,
                origin_for_paint,
                max_radius,
                ripple.color(ripple_fallback_color),
                config.ripple_expand_ms,
                config.ripple_fade_ms,
                config.easing,
            );
        }

        if pressed_falling && ripple.enabled {
            let min_release = self
                .ripple_press_frame
                .unwrap_or(now_frame)
                .saturating_add(min_press_frames);
            if now_frame < min_release {
                self.ripple_release_due_frame = Some(min_release);
            } else {
                self.ripple.release(now_frame);
                self.ripple_release_due_frame = None;
            }
        }
    }

    fn paint_frame(&mut self, now_frame: u64, ripple: ResolvedRipplePolicy) -> IndicationFrame {
        self.state_layer.advance(now_frame);

        if !ripple.enabled {
            self.ripple = RippleAnimator::default();
            self.ripple_press_frame = None;
            self.ripple_release_due_frame = None;
        }

        if let Some(release_due) = self.ripple_release_due_frame
            && now_frame >= release_due
        {
            self.ripple.release(now_frame);
            self.ripple_release_due_frame = None;
        }

        let ripple_frame = ripple
            .enabled
            .then(|| self.ripple.advance(now_frame, ripple.base_opacity))
            .flatten();
        let want_frames = self.state_layer.is_active()
            || self.ripple.is_active()
            || self.ripple_release_due_frame.is_some();

        IndicationFrame {
            state_layer_opacity: self.state_layer.value(),
            ripple_frame,
            want_frames,
        }
    }
}

fn resolve_ripple_policy<H: UiHost>(
    cx: &ElementContext<'_, H>,
    default_base_opacity: f32,
) -> ResolvedRipplePolicy {
    let mut policy = ResolvedRipplePolicy {
        enabled: default_base_opacity > 0.0,
        base_opacity: default_base_opacity,
        color_override: None,
    };

    match inherited_ripple_configuration(cx) {
        Some(MaterialRippleConfiguration::Disabled) => {
            policy.enabled = false;
        }
        Some(MaterialRippleConfiguration::Custom {
            color,
            base_opacity,
        }) => {
            if let Some(base_opacity) = base_opacity {
                policy.base_opacity = base_opacity;
                policy.enabled = base_opacity > 0.0;
            }
            policy.color_override = color;
        }
        Some(MaterialRippleConfiguration::UseDefault) | None => {}
    }

    policy
}

fn local_bounds_for(bounds: Rect) -> Rect {
    Rect::new(Point::new(Px(0.0), Px(0.0)), bounds.size)
}

fn indication_runtime_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
) -> SharedIndicationRuntime {
    cx.state_for(
        pressable_id,
        || Rc::new(RefCell::new(IndicationRuntime::default())),
        |runtime| runtime.clone(),
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_indication_for_pressable_with_ripple_bounds<H: UiHost>(
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
) -> (SharedIndicationRuntime, ResolvedRipplePolicy) {
    let ripple = resolve_ripple_policy(cx, ripple_base_opacity);
    let now_tick = cx.app.tick_id();
    let is_keyboard = fret_ui::input_modality::is_keyboard(&mut *cx.app, Some(cx.window));
    let last_down = (!is_keyboard)
        .then_some(last_down)
        .flatten()
        .filter(|down| now_tick.0.saturating_sub(down.tick_id.0) <= 2);
    let runtime = indication_runtime_for(cx, pressable_id);
    runtime.borrow_mut().update_pressable(
        now_frame,
        bounds,
        ripple_bounds,
        last_down,
        pressed,
        state_layer_target,
        ripple_fallback_color,
        ripple,
        config,
        is_keyboard,
    );
    (runtime, ripple)
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
    material_ink_layer_for_pressable_with_last_down(
        cx,
        pressable_id,
        now_frame,
        None,
        corner_radii,
        ripple_clip,
        state_layer_color,
        pressed,
        state_layer_target,
        ripple_base_opacity,
        config,
        extra_want_frames,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn material_ink_layer_for_pressable_with_last_down<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    last_down_override: Option<fret_ui::action::PointerDownCx>,
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
    let last_down = last_down_override.or_else(|| {
        cx.root_state(fret_ui::element::PointerRegionState::default, |st| {
            st.last_down
        })
    });

    let (runtime, ripple) = prepare_indication_for_pressable_with_ripple_bounds(
        cx,
        pressable_id,
        now_frame,
        bounds,
        local_bounds_for(bounds),
        last_down,
        pressed,
        state_layer_target,
        state_layer_color,
        ripple_base_opacity,
        config,
    );

    material_ink_layer_driven(
        cx,
        corner_radii,
        ripple_clip,
        state_layer_color,
        runtime,
        ripple,
        extra_want_frames,
    )
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
    let last_down = cx.root_state(fret_ui::element::PointerRegionState::default, |st| {
        st.last_down
    });

    let (runtime, ripple) = prepare_indication_for_pressable_with_ripple_bounds(
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

    material_ink_layer_with_bounds_driven(
        cx,
        paint_bounds,
        corner_radii,
        ripple_clip,
        state_layer_color,
        runtime,
        ripple,
        extra_want_frames,
    )
}

fn material_ink_layer_driven<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    corner_radii: Corners,
    ripple_clip: RippleClip,
    color: Color,
    runtime: SharedIndicationRuntime,
    ripple: ResolvedRipplePolicy,
    extra_want_frames: bool,
) -> AnyElement {
    let mut props = CanvasProps::default();
    props.layout.position = fret_ui::element::PositionStyle::Absolute;
    props.layout.inset.top = Some(Px(0.0)).into();
    props.layout.inset.right = Some(Px(0.0)).into();
    props.layout.inset.bottom = Some(Px(0.0)).into();
    props.layout.inset.left = Some(Px(0.0)).into();

    cx.canvas(props, move |p| {
        let bounds = p.bounds();
        let frame = runtime.borrow_mut().paint_frame(p.frame_id(), ripple);
        paint_ink_frame(
            p,
            bounds,
            corner_radii,
            ripple_clip,
            color,
            frame.state_layer_opacity,
            frame.ripple_frame,
        );

        if extra_want_frames {
            p.request_animation_frame();
        } else if frame.want_frames {
            p.request_animation_frame_paint_only();
        }
    })
}

fn material_ink_layer_with_bounds_driven<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    paint_bounds: Rect,
    corner_radii: Corners,
    ripple_clip: RippleClip,
    color: Color,
    runtime: SharedIndicationRuntime,
    ripple: ResolvedRipplePolicy,
    extra_want_frames: bool,
) -> AnyElement {
    let mut props = CanvasProps::default();
    props.layout.position = fret_ui::element::PositionStyle::Absolute;
    props.layout.inset.top = Some(Px(0.0)).into();
    props.layout.inset.right = Some(Px(0.0)).into();
    props.layout.inset.bottom = Some(Px(0.0)).into();
    props.layout.inset.left = Some(Px(0.0)).into();

    cx.canvas(props, move |p| {
        let bounds = p.bounds();
        let abs_paint_bounds = Rect::new(
            fret_core::Point::new(
                Px(bounds.origin.x.0 + paint_bounds.origin.x.0),
                Px(bounds.origin.y.0 + paint_bounds.origin.y.0),
            ),
            paint_bounds.size,
        );
        let frame = runtime.borrow_mut().paint_frame(p.frame_id(), ripple);
        paint_ink_frame(
            p,
            abs_paint_bounds,
            corner_radii,
            ripple_clip,
            color,
            frame.state_layer_opacity,
            frame.ripple_frame,
        );

        if extra_want_frames {
            p.request_animation_frame();
        } else if frame.want_frames {
            p.request_animation_frame_paint_only();
        }
    })
}

fn paint_ink_frame(
    p: &mut fret_ui::canvas::CanvasPainter<'_>,
    bounds: Rect,
    corner_radii: Corners,
    ripple_clip: RippleClip,
    color: Color,
    state_layer_opacity: f32,
    ripple_frame: Option<RipplePaintFrame>,
) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Size;

    fn test_color() -> Color {
        Color {
            r: 0.2,
            g: 0.3,
            b: 0.4,
            a: 1.0,
        }
    }

    fn test_policy() -> ResolvedRipplePolicy {
        ResolvedRipplePolicy {
            enabled: true,
            base_opacity: 0.12,
            color_override: None,
        }
    }

    fn test_bounds() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)))
    }

    #[test]
    fn indication_runtime_advances_ripple_from_paint_frames_without_render_update() {
        let mut runtime = IndicationRuntime::default();
        let bounds = test_bounds();
        let policy = test_policy();
        let config = IndicationConfig::default();

        runtime.update_pressable(
            0,
            bounds,
            local_bounds_for(bounds),
            None,
            true,
            0.0,
            test_color(),
            policy,
            config,
            false,
        );

        let first = runtime
            .paint_frame(0, policy)
            .ripple_frame
            .expect("expected active ripple on first paint");
        let later = runtime
            .paint_frame(4, policy)
            .ripple_frame
            .expect("expected active ripple on later paint");

        assert!(
            later.radius.0 > first.radius.0,
            "paint-only frames should advance the retained ripple radius"
        );
    }

    #[test]
    fn indication_runtime_releases_delayed_ripple_from_paint_frames() {
        let mut runtime = IndicationRuntime::default();
        let bounds = test_bounds();
        let policy = test_policy();
        let config = IndicationConfig {
            ripple_min_press_ms: 100,
            ripple_fade_ms: 100,
            ..Default::default()
        };

        runtime.update_pressable(
            0,
            bounds,
            local_bounds_for(bounds),
            None,
            true,
            0.0,
            test_color(),
            policy,
            config,
            false,
        );
        let _ = runtime.paint_frame(0, policy);

        runtime.update_pressable(
            1,
            bounds,
            local_bounds_for(bounds),
            None,
            false,
            0.0,
            test_color(),
            policy,
            config,
            false,
        );

        let release_due = crate::motion::ms_to_frames(config.ripple_min_press_ms).max(1);
        assert_eq!(runtime.ripple_release_due_frame, Some(release_due));

        let at_release = runtime
            .paint_frame(release_due, policy)
            .ripple_frame
            .expect("expected ripple fade to start on the delayed release frame");
        let after_release = runtime
            .paint_frame(release_due + 1, policy)
            .ripple_frame
            .expect("expected ripple fade to continue after delayed release");

        assert!(
            after_release.opacity < at_release.opacity,
            "paint-only frames should continue the delayed release fade"
        );
    }
}
