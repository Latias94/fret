//! Shared Material field motion policy.

use fret_core::{Color, Edges, Px};
use fret_ui::UiHost;
use fret_ui::elements::ElementContext;

use crate::motion::{SpringAnimator, SpringSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FieldInputPhase {
    Focused,
    #[default]
    UnfocusedEmpty,
    UnfocusedNotEmpty,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldMotionTargets {
    pub disabled: bool,
    pub should_float: bool,
    pub input_phase: FieldInputPhase,
    pub placeholder_target_opacity: f32,
    pub border: Edges,
    pub border_color: Color,
    pub spatial: SpringSpec,
    pub fast_effects: SpringSpec,
    pub slow_effects: SpringSpec,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldMotionFrame {
    pub want_frames: bool,
    pub float_progress: f32,
    pub border: Edges,
    pub border_color: Color,
    pub placeholder_opacity: f32,
}

#[derive(Debug, Default)]
struct FieldMotionRuntime {
    float_target: bool,
    float: SpringAnimator,
    last_phase: FieldInputPhase,
    placeholder_opacity: SpringAnimator,
    border_top: SpringAnimator,
    border_right: SpringAnimator,
    border_bottom: SpringAnimator,
    border_left: SpringAnimator,
    border_color: AnimatedColor,
}

#[derive(Debug, Default)]
struct AnimatedColor {
    r: SpringAnimator,
    g: SpringAnimator,
    b: SpringAnimator,
    a: SpringAnimator,
}

impl AnimatedColor {
    fn reset(&mut self, now_frame: u64, color: Color) {
        self.r.reset(now_frame, color.r);
        self.g.reset(now_frame, color.g);
        self.b.reset(now_frame, color.b);
        self.a.reset(now_frame, color.a);
    }

    fn set_target(&mut self, now_frame: u64, color: Color, spec: SpringSpec) {
        self.r.set_target(now_frame, color.r, spec);
        self.g.set_target(now_frame, color.g, spec);
        self.b.set_target(now_frame, color.b, spec);
        self.a.set_target(now_frame, color.a, spec);
    }

    fn advance(&mut self, now_frame: u64) {
        self.r.advance(now_frame);
        self.g.advance(now_frame);
        self.b.advance(now_frame);
        self.a.advance(now_frame);
    }

    fn is_active(&self) -> bool {
        self.r.is_active() || self.g.is_active() || self.b.is_active() || self.a.is_active()
    }

    fn value(&self) -> Color {
        Color {
            r: self.r.value().clamp(0.0, 1.0),
            g: self.g.value().clamp(0.0, 1.0),
            b: self.b.value().clamp(0.0, 1.0),
            a: self.a.value().clamp(0.0, 1.0),
        }
    }
}

pub(crate) fn field_motion_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    targets: FieldMotionTargets,
) -> FieldMotionFrame {
    let now_frame = cx.frame_id.0;

    let frame = cx.root_state(FieldMotionRuntime::default, |rt| {
        if targets.disabled {
            rt.float_target = targets.should_float;
            rt.float
                .reset(now_frame, if targets.should_float { 1.0 } else { 0.0 });
            rt.last_phase = targets.input_phase;
            rt.placeholder_opacity
                .reset(now_frame, targets.placeholder_target_opacity);
            rt.border_top.reset(now_frame, targets.border.top.0);
            rt.border_right.reset(now_frame, targets.border.right.0);
            rt.border_bottom.reset(now_frame, targets.border.bottom.0);
            rt.border_left.reset(now_frame, targets.border.left.0);
            rt.border_color.reset(now_frame, targets.border_color);

            return FieldMotionFrame {
                want_frames: false,
                float_progress: rt.float.value(),
                border: targets.border,
                border_color: targets.border_color,
                placeholder_opacity: rt.placeholder_opacity.value(),
            };
        }

        if !rt.float.is_initialized() {
            rt.float_target = targets.should_float;
            rt.float
                .reset(now_frame, if targets.should_float { 1.0 } else { 0.0 });
        }

        if rt.float_target != targets.should_float {
            rt.float_target = targets.should_float;
            rt.float.set_target(
                now_frame,
                if targets.should_float { 1.0 } else { 0.0 },
                targets.spatial,
            );
        }

        let placeholder_effects = match (rt.last_phase, targets.input_phase) {
            (FieldInputPhase::Focused, FieldInputPhase::UnfocusedEmpty) => targets.fast_effects,
            (FieldInputPhase::UnfocusedEmpty, FieldInputPhase::Focused)
            | (FieldInputPhase::UnfocusedNotEmpty, FieldInputPhase::UnfocusedEmpty) => {
                targets.slow_effects
            }
            _ => targets.fast_effects,
        };
        rt.last_phase = targets.input_phase;

        rt.placeholder_opacity.set_target(
            now_frame,
            targets.placeholder_target_opacity,
            placeholder_effects,
        );

        rt.border_top
            .set_target(now_frame, targets.border.top.0, targets.spatial);
        rt.border_right
            .set_target(now_frame, targets.border.right.0, targets.spatial);
        rt.border_bottom
            .set_target(now_frame, targets.border.bottom.0, targets.spatial);
        rt.border_left
            .set_target(now_frame, targets.border.left.0, targets.spatial);

        rt.border_color
            .set_target(now_frame, targets.border_color, targets.fast_effects);

        rt.float.advance(now_frame);
        rt.placeholder_opacity.advance(now_frame);
        rt.border_top.advance(now_frame);
        rt.border_right.advance(now_frame);
        rt.border_bottom.advance(now_frame);
        rt.border_left.advance(now_frame);
        rt.border_color.advance(now_frame);

        let want_frames = rt.float.is_active()
            || rt.placeholder_opacity.is_active()
            || rt.border_top.is_active()
            || rt.border_right.is_active()
            || rt.border_bottom.is_active()
            || rt.border_left.is_active()
            || rt.border_color.is_active();

        FieldMotionFrame {
            want_frames,
            float_progress: rt.float.value(),
            border: Edges {
                top: Px(rt.border_top.value().max(0.0)),
                right: Px(rt.border_right.value().max(0.0)),
                bottom: Px(rt.border_bottom.value().max(0.0)),
                left: Px(rt.border_left.value().max(0.0)),
            },
            border_color: rt.border_color.value(),
            placeholder_opacity: rt.placeholder_opacity.value(),
        }
    });

    if frame.want_frames {
        cx.request_animation_frame();
    }

    frame
}
