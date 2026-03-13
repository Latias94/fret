use std::panic::Location;
use std::time::Duration;

use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_headless::motion::inertia::{InertiaBounds, InertiaSimulation};
use fret_ui_headless::motion::simulation::Simulation1D;
use fret_ui_headless::motion::spring::{SpringDescription, SpringSimulation};
use fret_ui_headless::motion::tolerance::Tolerance;

use crate::declarative::motion::{
    DrivenMotionF32, effective_frame_delta_for_cx, tween_value_at, tween_velocity_at,
};
use crate::declarative::scheduling::set_continuous_frames;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionKickF32 {
    pub id: u64,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct TweenSpecF32 {
    pub duration: Duration,
    pub ease: fn(f32) -> f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringSpecF32 {
    pub spring: SpringDescription,
    pub tolerance: Tolerance,
    pub snap_to_target: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InertiaSpecF32 {
    pub drag: f64,
    pub bounds: Option<(f32, f32)>,
    pub bounce_spring: SpringDescription,
    pub tolerance: Tolerance,
}

#[derive(Debug, Clone, Copy)]
pub enum MotionToSpecF32 {
    Tween(TweenSpecF32),
    Spring(SpringSpecF32),
}

#[derive(Debug, Clone, Copy)]
pub enum MotionValueF32Update {
    Snap(f32),
    To {
        target: f32,
        spec: MotionToSpecF32,
        kick: Option<MotionKickF32>,
    },
    Inertia {
        spec: InertiaSpecF32,
        kick: MotionKickF32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MotionValueF32Kind {
    Snap,
    Tween,
    Spring,
    Inertia,
}

#[derive(Debug, Clone, Copy)]
struct MotionValueF32State {
    initialized: bool,
    last_frame_id: u64,
    kind: MotionValueF32Kind,

    value: f32,
    velocity: f32,
    animating: bool,
    elapsed: Duration,

    tween_start: f32,
    tween_target: f32,
    tween_duration: Duration,
    tween_ease: fn(f32) -> f32,

    spring_start: f32,
    spring_target: f32,
    spring: SpringDescription,
    spring_tolerance: Tolerance,
    spring_snap_to_target: bool,
    spring_last_kick_id: u64,

    inertia_start: f32,
    inertia_start_velocity: f32,
    inertia_drag: f64,
    inertia_bounds: Option<(f32, f32)>,
    inertia_bounce_spring: SpringDescription,
    inertia_tolerance: Tolerance,
    inertia_last_kick_id: u64,
}

impl Default for MotionValueF32State {
    fn default() -> Self {
        Self {
            initialized: false,
            last_frame_id: 0,
            kind: MotionValueF32Kind::Snap,
            value: 0.0,
            velocity: 0.0,
            animating: false,
            elapsed: Duration::ZERO,

            tween_start: 0.0,
            tween_target: 0.0,
            tween_duration: Duration::from_millis(200),
            tween_ease: crate::headless::easing::smoothstep,

            spring_start: 0.0,
            spring_target: 0.0,
            spring: SpringDescription::with_duration_and_bounce(Duration::from_millis(240), 0.0),
            spring_tolerance: Tolerance::default(),
            spring_snap_to_target: true,
            spring_last_kick_id: 0,

            inertia_start: 0.0,
            inertia_start_velocity: 0.0,
            inertia_drag: 0.135,
            inertia_bounds: None,
            inertia_bounce_spring: SpringDescription::with_duration_and_bounce(
                Duration::from_millis(240),
                0.25,
            ),
            inertia_tolerance: Tolerance::default(),
            inertia_last_kick_id: 0,
        }
    }
}

#[track_caller]
pub fn drive_motion_value_f32<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    initial: f32,
    update: MotionValueF32Update,
) -> DrivenMotionF32 {
    let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
    let loc = Location::caller();
    cx.keyed(
        (
            loc.file(),
            loc.line(),
            loc.column(),
            "drive_motion_value_f32",
        ),
        |cx| {
            let frame_id = cx.frame_id.0;
            let dt = effective_frame_delta_for_cx(cx);

            let out = cx.slot_state(MotionValueF32State::default, |st| {
                if !st.initialized {
                    st.initialized = true;
                    st.last_frame_id = frame_id;
                    st.kind = MotionValueF32Kind::Snap;
                    st.value = initial;
                    st.velocity = 0.0;
                    st.animating = false;
                    st.elapsed = Duration::ZERO;
                }

                if reduced_motion {
                    match update {
                        MotionValueF32Update::Snap(v) => {
                            st.kind = MotionValueF32Kind::Snap;
                            st.value = v;
                            st.velocity = 0.0;
                            st.animating = false;
                            st.elapsed = Duration::ZERO;
                        }
                        MotionValueF32Update::To { target, .. } => {
                            st.kind = MotionValueF32Kind::Snap;
                            st.value = target;
                            st.velocity = 0.0;
                            st.animating = false;
                            st.elapsed = Duration::ZERO;
                        }
                        MotionValueF32Update::Inertia { .. } => {
                            st.kind = MotionValueF32Kind::Snap;
                            st.velocity = 0.0;
                            st.animating = false;
                            st.elapsed = Duration::ZERO;
                        }
                    }

                    return DrivenMotionF32 {
                        value: st.value,
                        velocity: st.velocity,
                        animating: st.animating,
                    };
                }

                match update {
                    MotionValueF32Update::Snap(v) => {
                        st.kind = MotionValueF32Kind::Snap;
                        st.value = v;
                        st.velocity = 0.0;
                        st.animating = false;
                        st.elapsed = Duration::ZERO;
                    }
                    MotionValueF32Update::To { target, spec, kick } => match spec {
                        MotionToSpecF32::Tween(spec) => {
                            let needs_retarget = st.kind != MotionValueF32Kind::Tween
                                || st.tween_target != target
                                || st.tween_duration != spec.duration
                                || st.tween_ease as usize != spec.ease as usize;

                            if needs_retarget {
                                st.kind = MotionValueF32Kind::Tween;
                                st.tween_start = st.value;
                                st.tween_target = target;
                                st.tween_duration = spec.duration;
                                st.tween_ease = spec.ease;
                                st.elapsed = Duration::ZERO;
                                st.animating = true;
                            }

                            if st.animating && st.last_frame_id != frame_id {
                                st.last_frame_id = frame_id;
                                st.elapsed = st.elapsed.saturating_add(dt);

                                st.value = tween_value_at(
                                    st.tween_start,
                                    st.tween_target,
                                    st.tween_duration,
                                    st.tween_ease,
                                    st.elapsed,
                                );
                                st.velocity = tween_velocity_at(
                                    st.tween_start,
                                    st.tween_target,
                                    st.tween_duration,
                                    st.tween_ease,
                                    st.elapsed,
                                );

                                if st.elapsed >= st.tween_duration {
                                    st.value = st.tween_target;
                                    st.velocity = 0.0;
                                    st.animating = false;
                                }
                            } else if st.last_frame_id == 0 {
                                st.last_frame_id = frame_id;
                            }
                        }
                        MotionToSpecF32::Spring(spec) => {
                            let kick_retarget = kick.is_some()
                                && kick.map(|k| k.id).unwrap_or(0) != st.spring_last_kick_id;
                            let needs_retarget = st.kind != MotionValueF32Kind::Spring
                                || st.spring_target != target
                                || st.spring != spec.spring
                                || st.spring_tolerance != spec.tolerance
                                || st.spring_snap_to_target != spec.snap_to_target
                                || kick_retarget;

                            if needs_retarget {
                                st.kind = MotionValueF32Kind::Spring;
                                st.spring_start = st.value;
                                st.spring_target = target;
                                st.spring = spec.spring;
                                st.spring_tolerance = spec.tolerance;
                                st.spring_snap_to_target = spec.snap_to_target;
                                st.elapsed = Duration::ZERO;
                                st.animating = true;

                                if let Some(kick) = kick
                                    && kick.id != st.spring_last_kick_id
                                {
                                    st.velocity = kick.velocity;
                                    st.spring_last_kick_id = kick.id;
                                }
                            }

                            if st.animating && st.last_frame_id != frame_id {
                                st.last_frame_id = frame_id;
                                st.elapsed = st.elapsed.saturating_add(dt);

                                let sim = SpringSimulation::new(
                                    st.spring,
                                    st.spring_start as f64,
                                    st.spring_target as f64,
                                    st.velocity as f64,
                                    st.spring_snap_to_target,
                                    st.spring_tolerance,
                                );

                                st.value = sim.x(st.elapsed) as f32;
                                st.velocity = sim.dx(st.elapsed) as f32;

                                if sim.is_done(st.elapsed) {
                                    st.value = st.spring_target;
                                    st.velocity = 0.0;
                                    st.animating = false;
                                }
                            } else if st.last_frame_id == 0 {
                                st.last_frame_id = frame_id;
                            }
                        }
                    },
                    MotionValueF32Update::Inertia { spec, kick } => {
                        let kick_retarget = kick.id != st.inertia_last_kick_id;
                        let needs_retarget = st.kind != MotionValueF32Kind::Inertia
                            || kick_retarget
                            || st.inertia_drag != spec.drag
                            || st.inertia_bounds != spec.bounds
                            || st.inertia_bounce_spring != spec.bounce_spring
                            || st.inertia_tolerance != spec.tolerance;

                        if needs_retarget {
                            st.kind = MotionValueF32Kind::Inertia;
                            st.inertia_drag = spec.drag;
                            st.inertia_bounds = spec.bounds;
                            st.inertia_bounce_spring = spec.bounce_spring;
                            st.inertia_tolerance = spec.tolerance;

                            if kick.id != st.inertia_last_kick_id {
                                st.inertia_last_kick_id = kick.id;
                                st.inertia_start = st.value;
                                st.inertia_start_velocity = kick.velocity;
                                st.velocity = kick.velocity;
                                st.elapsed = Duration::ZERO;
                                st.animating = true;
                            } else if st.animating {
                                st.inertia_start = st.value;
                                st.inertia_start_velocity = st.velocity;
                                st.elapsed = Duration::ZERO;
                            }
                        }

                        if st.animating && st.last_frame_id != frame_id {
                            st.last_frame_id = frame_id;
                            st.elapsed = st.elapsed.saturating_add(dt);

                            let inertia_bounds =
                                st.inertia_bounds.map(|(min, max)| InertiaBounds {
                                    min: min as f64,
                                    max: max as f64,
                                });
                            let sim = InertiaSimulation::new(
                                st.inertia_start as f64,
                                st.inertia_start_velocity as f64,
                                st.inertia_drag,
                                inertia_bounds,
                                st.inertia_bounce_spring,
                                st.inertia_tolerance,
                            );

                            st.value = sim.x(st.elapsed) as f32;
                            st.velocity = sim.dx(st.elapsed) as f32;

                            if sim.is_done(st.elapsed) {
                                st.value = sim.final_x() as f32;
                                st.velocity = 0.0;
                                st.animating = false;
                            }
                        } else if st.last_frame_id == 0 {
                            st.last_frame_id = frame_id;
                        }
                    }
                }

                DrivenMotionF32 {
                    value: st.value,
                    velocity: st.velocity,
                    animating: st.animating,
                }
            });

            set_continuous_frames(cx, out.animating);
            if out.animating {
                cx.notify_for_animation_frame();
            }
            out
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Px, WindowFrameClockService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::elements::with_element_cx;

    fn bounds() -> fret_core::Rect {
        fret_core::Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        )
    }

    fn drive<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        update: MotionValueF32Update,
    ) -> DrivenMotionF32 {
        drive_motion_value_f32(cx, 0.0, update)
    }

    #[test]
    fn motion_value_snap_then_spring_to_does_not_jump_on_first_frame() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(16)));
        });

        for fid in [FrameId(1), FrameId(2)] {
            app.set_frame_id(fid);
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
        }

        app.set_tick_id(TickId(1));
        let mut out = with_element_cx(&mut app, window, bounds(), "motion_value", |cx| {
            drive(cx, MotionValueF32Update::Snap(10.0))
        });
        assert_eq!(out.value, 10.0);
        assert!(!out.animating);

        app.set_tick_id(TickId(2));
        out = with_element_cx(&mut app, window, bounds(), "motion_value", |cx| {
            drive(
                cx,
                MotionValueF32Update::To {
                    target: 20.0,
                    spec: MotionToSpecF32::Spring(SpringSpecF32 {
                        spring: SpringDescription::with_duration_and_bounce(
                            Duration::from_millis(240),
                            0.0,
                        ),
                        tolerance: Tolerance::default(),
                        snap_to_target: true,
                    }),
                    kick: Some(MotionKickF32 {
                        id: 1,
                        velocity: 0.0,
                    }),
                },
            )
        });

        // First frame after retarget should start from the snapped value.
        assert_eq!(out.value, 10.0);
        assert!(out.animating);
    }
}
