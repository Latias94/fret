use fret_core::{Color, Corners, DrawOrder, Px, Rect};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, CanvasProps};
use fret_ui::elements::ElementContext;
use fret_ui::theme::CubicBezier;

use crate::foundation::context::{MaterialRippleConfiguration, inherited_ripple_configuration};
use crate::interaction::ripple::{RippleAnimator, RipplePaintFrame};
use crate::interaction::state_layer::StateLayerAnimator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RippleClip {
    /// Clip the ripple to the component's shape (bounded ripple).
    Bounded,
    /// Do not clip the ripple (unbounded ripple).
    Unbounded,
}

#[derive(Debug, Clone, Copy)]
pub struct IndicationConfig {
    pub state_duration_ms: u32,
    pub ripple_expand_ms: u32,
    pub ripple_fade_ms: u32,
    pub easing: CubicBezier,
}

impl Default for IndicationConfig {
    fn default() -> Self {
        Self {
            state_duration_ms: 100,
            ripple_expand_ms: 200,
            ripple_fade_ms: 100,
            easing: CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
        }
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
}

pub fn advance_indication_for_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    bounds: Rect,
    last_down: Option<fret_ui::action::PointerDownCx>,
    pressed: bool,
    state_layer_target: f32,
    ripple_base_opacity: f32,
    config: IndicationConfig,
) -> IndicationFrame {
    use crate::foundation::geometry::{down_origin_local, ripple_max_radius};

    let ripple_config = inherited_ripple_configuration(cx);
    let mut ripple_enabled = true;
    let mut ripple_base_opacity = ripple_base_opacity;
    match ripple_config {
        Some(MaterialRippleConfiguration::Disabled) => ripple_enabled = false,
        Some(MaterialRippleConfiguration::Custom {
            base_opacity: Some(base_opacity),
            ..
        }) => {
            ripple_base_opacity = base_opacity;
            ripple_enabled = base_opacity > 0.0;
        }
        Some(MaterialRippleConfiguration::UseDefault)
        | Some(MaterialRippleConfiguration::Custom {
            base_opacity: None, ..
        })
        | None => {}
    }

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
        }

        let pressed_rising = pressed && !rt.prev_pressed;
        rt.prev_pressed = pressed;
        if pressed_rising && ripple_enabled {
            let origin = down_origin_local(bounds, last_down);
            let max_radius = ripple_max_radius(bounds, origin);
            rt.ripple.start(
                now_frame,
                origin,
                max_radius,
                config.ripple_expand_ms,
                config.ripple_fade_ms,
                config.easing,
            );
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

pub fn advance_indication_for_pressable_with_ripple_bounds<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    bounds: Rect,
    ripple_bounds: Rect,
    last_down: Option<fret_ui::action::PointerDownCx>,
    pressed: bool,
    state_layer_target: f32,
    ripple_base_opacity: f32,
    config: IndicationConfig,
) -> IndicationFrame {
    use crate::foundation::geometry::{down_origin_local, ripple_max_radius};

    let ripple_config = inherited_ripple_configuration(cx);
    let mut ripple_enabled = true;
    let mut ripple_base_opacity = ripple_base_opacity;
    match ripple_config {
        Some(MaterialRippleConfiguration::Disabled) => ripple_enabled = false,
        Some(MaterialRippleConfiguration::Custom {
            base_opacity: Some(base_opacity),
            ..
        }) => {
            ripple_base_opacity = base_opacity;
            ripple_enabled = base_opacity > 0.0;
        }
        Some(MaterialRippleConfiguration::UseDefault)
        | Some(MaterialRippleConfiguration::Custom {
            base_opacity: None, ..
        })
        | None => {}
    }

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
        }

        let pressed_rising = pressed && !rt.prev_pressed;
        rt.prev_pressed = pressed;
        if pressed_rising && ripple_enabled {
            let origin = down_origin_local(bounds, last_down);
            let origin_in_ripple = fret_core::Point::new(
                Px(origin.x.0 - ripple_bounds.origin.x.0),
                Px(origin.y.0 - ripple_bounds.origin.y.0),
            );
            let max_radius = ripple_max_radius(
                Rect::new(fret_core::Point::new(Px(0.0), Px(0.0)), ripple_bounds.size),
                origin_in_ripple,
            );
            rt.ripple.start(
                now_frame,
                origin,
                max_radius,
                config.ripple_expand_ms,
                config.ripple_fade_ms,
                config.easing,
            );
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
            fret_ui::paint::paint_ripple(
                p.scene(),
                DrawOrder(1),
                bounds,
                r.origin,
                r.radius,
                color,
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
        if state_layer_opacity > 0.0 {
            fret_ui::paint::paint_state_layer(
                p.scene(),
                DrawOrder(0),
                paint_bounds,
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
            fret_ui::paint::paint_ripple(
                p.scene(),
                DrawOrder(1),
                paint_bounds,
                r.origin,
                r.radius,
                color,
                r.opacity,
                clip,
            );
        }

        if want_frames {
            p.request_animation_frame();
        }
    })
}
