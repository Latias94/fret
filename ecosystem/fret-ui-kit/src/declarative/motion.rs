use std::hash::Hash;
use std::panic::Location;
use std::time::Duration;

use fret_core::{Color, WindowFrameClockService, WindowMetricsService};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_headless::motion::inertia::{InertiaBounds, InertiaSimulation};
use fret_ui_headless::motion::simulation::Simulation1D;
use fret_ui_headless::motion::spring::{SpringDescription, SpringSimulation};
use fret_ui_headless::motion::tolerance::Tolerance;

use crate::declarative::scheduling::set_continuous_frames;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrivenMotionF32 {
    pub value: f32,
    pub velocity: f32,
    pub animating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrivenMotionColor {
    pub value: Color,
    pub animating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringKick {
    pub id: u64,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InertiaKick {
    pub id: u64,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy)]
struct TweenF32State {
    initialized: bool,
    last_frame_id: u64,
    start: f32,
    target: f32,
    value: f32,
    velocity: f32,
    elapsed: Duration,
    duration: Duration,
    ease: fn(f32) -> f32,
    animating: bool,
}

impl Default for TweenF32State {
    fn default() -> Self {
        Self {
            initialized: false,
            last_frame_id: 0,
            start: 0.0,
            target: 0.0,
            value: 0.0,
            velocity: 0.0,
            elapsed: Duration::ZERO,
            duration: Duration::from_millis(200),
            ease: crate::headless::easing::smoothstep,
            animating: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TweenColorState {
    initialized: bool,
    last_frame_id: u64,
    start: Color,
    target: Color,
    value: Color,
    elapsed: Duration,
    duration: Duration,
    ease: fn(f32) -> f32,
    animating: bool,
}

impl Default for TweenColorState {
    fn default() -> Self {
        Self {
            initialized: false,
            last_frame_id: 0,
            start: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            target: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            value: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            elapsed: Duration::ZERO,
            duration: Duration::from_millis(200),
            ease: crate::headless::easing::smoothstep,
            animating: false,
        }
    }
}

const REFERENCE_FRAME_DELTA_60HZ: Duration = Duration::from_nanos(1_000_000_000 / 60);
const MAX_FRAME_DELTA: Duration = Duration::from_millis(50);

fn clamp_frame_delta(dt: Duration) -> Duration {
    if dt == Duration::ZERO {
        return REFERENCE_FRAME_DELTA_60HZ;
    }
    dt.min(MAX_FRAME_DELTA)
}

pub(crate) fn effective_frame_delta_for_cx<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
    let Some(svc) = cx.app.global::<WindowFrameClockService>() else {
        return REFERENCE_FRAME_DELTA_60HZ;
    };

    if let Some(fixed) = svc.effective_fixed_delta(cx.window) {
        return clamp_frame_delta(fixed);
    }

    let has_window_metrics = cx.app.global::<WindowMetricsService>().is_some();
    if !has_window_metrics {
        // Headless tests often drive "frames" without present-time. In that regime, snapshot deltas
        // can reflect CPU time (near-zero), which would effectively stall a Duration-driven
        // animation. Use a stable reference delta unless a fixed delta is explicitly configured.
        return REFERENCE_FRAME_DELTA_60HZ;
    }

    let Some(snapshot) = svc.snapshot(cx.window) else {
        return REFERENCE_FRAME_DELTA_60HZ;
    };

    clamp_frame_delta(snapshot.delta)
}

pub(crate) fn tween_value_at(
    start: f32,
    end: f32,
    duration: Duration,
    ease: fn(f32) -> f32,
    elapsed: Duration,
) -> f32 {
    if duration == Duration::ZERO {
        return end;
    }
    let t = (elapsed.as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0) as f32;
    let eased = ease(t).clamp(0.0, 1.0);
    start + (end - start) * eased
}

fn tween_color_at(
    start: Color,
    end: Color,
    duration: Duration,
    ease: fn(f32) -> f32,
    elapsed: Duration,
) -> Color {
    if duration == Duration::ZERO {
        return end;
    }
    let t = (elapsed.as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0) as f32;
    let eased = ease(t).clamp(0.0, 1.0);
    Color {
        r: start.r + (end.r - start.r) * eased,
        g: start.g + (end.g - start.g) * eased,
        b: start.b + (end.b - start.b) * eased,
        a: start.a + (end.a - start.a) * eased,
    }
}

pub(crate) fn tween_value_at_unclamped(
    start: f32,
    end: f32,
    duration: Duration,
    ease: fn(f32) -> f32,
    elapsed: Duration,
) -> f32 {
    if duration == Duration::ZERO {
        return end;
    }
    let t = (elapsed.as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0) as f32;
    let eased = ease(t);
    start + (end - start) * eased
}

pub(crate) fn tween_velocity_at(
    start: f32,
    end: f32,
    duration: Duration,
    ease: fn(f32) -> f32,
    elapsed: Duration,
) -> f32 {
    // Finite-difference approximation. This is primarily used for retargeting continuity.
    let dt = Duration::from_millis(1);
    let t0 = elapsed.saturating_sub(dt);
    let t1 = (elapsed + dt).min(duration);
    if t1 <= t0 {
        return 0.0;
    }
    let v0 = tween_value_at(start, end, duration, ease, t0);
    let v1 = tween_value_at(start, end, duration, ease, t1);
    (v1 - v0) / (t1 - t0).as_secs_f32()
}

pub(crate) fn tween_velocity_at_unclamped(
    start: f32,
    end: f32,
    duration: Duration,
    ease: fn(f32) -> f32,
    elapsed: Duration,
) -> f32 {
    // Finite-difference approximation. This is primarily used for retargeting continuity.
    let dt = Duration::from_millis(1);
    let t0 = elapsed.saturating_sub(dt);
    let t1 = (elapsed + dt).min(duration);
    if t1 <= t0 {
        return 0.0;
    }
    let v0 = tween_value_at_unclamped(start, end, duration, ease, t0);
    let v1 = tween_value_at_unclamped(start, end, duration, ease, t1);
    (v1 - v0) / (t1 - t0).as_secs_f32()
}

#[track_caller]
pub fn drive_tween_f32<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    target: f32,
    duration: Duration,
    ease: fn(f32) -> f32,
) -> DrivenMotionF32 {
    let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
    if reduced_motion {
        set_continuous_frames(cx, false);
        return DrivenMotionF32 {
            value: target,
            velocity: 0.0,
            animating: false,
        };
    }

    let loc = Location::caller();
    cx.keyed(
        (loc.file(), loc.line(), loc.column(), "drive_tween_f32"),
        |cx| {
            let frame_id = cx.frame_id.0;
            let dt = effective_frame_delta_for_cx(cx);

            let out = cx.with_state(TweenF32State::default, |st| {
                if !st.initialized {
                    st.initialized = true;
                    st.last_frame_id = frame_id;
                    st.start = target;
                    st.target = target;
                    st.value = target;
                    st.velocity = 0.0;
                    st.elapsed = Duration::ZERO;
                    st.duration = duration;
                    st.ease = ease;
                    st.animating = false;
                }

                // Retarget.
                if target != st.target
                    || st.duration != duration
                    || st.ease as usize != ease as usize
                {
                    st.start = st.value;
                    st.target = target;
                    st.duration = duration;
                    st.ease = ease;
                    st.elapsed = Duration::ZERO;
                    st.animating = true;
                    // Keep current value as start to avoid a jump.
                }

                // Advance at most once per frame.
                if st.animating && st.last_frame_id != frame_id {
                    st.last_frame_id = frame_id;
                    st.elapsed = st.elapsed.saturating_add(dt);

                    let value =
                        tween_value_at(st.start, st.target, st.duration, st.ease, st.elapsed);
                    let velocity =
                        tween_velocity_at(st.start, st.target, st.duration, st.ease, st.elapsed);
                    st.value = value;
                    st.velocity = velocity;

                    if st.elapsed >= st.duration {
                        st.value = st.target;
                        st.velocity = 0.0;
                        st.animating = false;
                    }
                } else if st.last_frame_id == 0 {
                    st.last_frame_id = frame_id;
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

#[track_caller]
pub fn drive_tween_color<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    target: Color,
    duration: Duration,
    ease: fn(f32) -> f32,
) -> DrivenMotionColor {
    let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
    if reduced_motion {
        set_continuous_frames(cx, false);
        return DrivenMotionColor {
            value: target,
            animating: false,
        };
    }

    let loc = Location::caller();
    cx.keyed(
        (loc.file(), loc.line(), loc.column(), "drive_tween_color"),
        |cx| {
            let frame_id = cx.frame_id.0;
            let dt = effective_frame_delta_for_cx(cx);

            let out = cx.with_state(TweenColorState::default, |st| {
                if !st.initialized {
                    st.initialized = true;
                    st.last_frame_id = frame_id;
                    st.start = target;
                    st.target = target;
                    st.value = target;
                    st.elapsed = Duration::ZERO;
                    st.duration = duration;
                    st.ease = ease;
                    st.animating = false;
                }

                // Retarget.
                if target != st.target
                    || st.duration != duration
                    || st.ease as usize != ease as usize
                {
                    st.start = st.value;
                    st.target = target;
                    st.duration = duration;
                    st.ease = ease;
                    st.elapsed = Duration::ZERO;
                    st.animating = true;
                }

                // Advance at most once per frame.
                if st.animating && st.last_frame_id != frame_id {
                    st.last_frame_id = frame_id;
                    st.elapsed = st.elapsed.saturating_add(dt);

                    st.value =
                        tween_color_at(st.start, st.target, st.duration, st.ease, st.elapsed);
                    if st.elapsed >= st.duration {
                        st.value = st.target;
                        st.animating = false;
                    }
                } else if st.last_frame_id == 0 {
                    st.last_frame_id = frame_id;
                }

                DrivenMotionColor {
                    value: st.value,
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

#[track_caller]
pub fn drive_tween_f32_unclamped<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    target: f32,
    duration: Duration,
    ease: fn(f32) -> f32,
) -> DrivenMotionF32 {
    let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
    if reduced_motion {
        set_continuous_frames(cx, false);
        return DrivenMotionF32 {
            value: target,
            velocity: 0.0,
            animating: false,
        };
    }

    let loc = Location::caller();
    cx.keyed(
        (
            loc.file(),
            loc.line(),
            loc.column(),
            "drive_tween_f32_unclamped",
        ),
        |cx| {
            let frame_id = cx.frame_id.0;
            let dt = effective_frame_delta_for_cx(cx);

            let out = cx.with_state(TweenF32State::default, |st| {
                if !st.initialized {
                    st.initialized = true;
                    st.last_frame_id = frame_id;
                    st.start = target;
                    st.target = target;
                    st.value = target;
                    st.velocity = 0.0;
                    st.elapsed = Duration::ZERO;
                    st.duration = duration;
                    st.ease = ease;
                    st.animating = false;
                }

                // Retarget.
                if target != st.target
                    || st.duration != duration
                    || st.ease as usize != ease as usize
                {
                    st.start = st.value;
                    st.target = target;
                    st.duration = duration;
                    st.ease = ease;
                    st.elapsed = Duration::ZERO;
                    st.animating = true;
                }

                // Advance at most once per frame.
                if st.animating && st.last_frame_id != frame_id {
                    st.last_frame_id = frame_id;
                    st.elapsed = st.elapsed.saturating_add(dt);

                    let value = tween_value_at_unclamped(
                        st.start,
                        st.target,
                        st.duration,
                        st.ease,
                        st.elapsed,
                    );
                    let velocity = tween_velocity_at_unclamped(
                        st.start,
                        st.target,
                        st.duration,
                        st.ease,
                        st.elapsed,
                    );
                    st.value = value;
                    st.velocity = velocity;

                    if st.elapsed >= st.duration {
                        st.value = st.target;
                        st.velocity = 0.0;
                        st.animating = false;
                    }
                } else if st.last_frame_id == 0 {
                    st.last_frame_id = frame_id;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrivenLoopProgress {
    /// Normalized progress within the loop cycle. (`[0.0, 1.0)` while animating.)
    pub progress: f32,
    pub animating: bool,
}

#[derive(Debug, Clone, Copy)]
struct LoopProgressState {
    initialized: bool,
    last_frame_id: u64,
    elapsed: Duration,
    period: Duration,
}

impl Default for LoopProgressState {
    fn default() -> Self {
        Self {
            initialized: false,
            last_frame_id: 0,
            elapsed: Duration::ZERO,
            period: Duration::from_secs(1),
        }
    }
}

fn duration_mod(elapsed: Duration, period: Duration) -> Duration {
    if period == Duration::ZERO {
        return Duration::ZERO;
    }
    let period_ns = period.as_nanos();
    if period_ns == 0 {
        return Duration::ZERO;
    }
    let rem_ns = elapsed.as_nanos() % period_ns;
    Duration::from_nanos(rem_ns.min(u64::MAX as u128) as u64)
}

#[track_caller]
pub fn drive_loop_progress<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    period: Duration,
) -> DrivenLoopProgress {
    let loc = Location::caller();
    drive_loop_progress_keyed(
        cx,
        (loc.file(), loc.line(), loc.column(), "drive_loop_progress"),
        enabled,
        period,
    )
}

pub fn drive_loop_progress_keyed<H: UiHost, K: Hash>(
    cx: &mut ElementContext<'_, H>,
    key: K,
    enabled: bool,
    period: Duration,
) -> DrivenLoopProgress {
    let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
    cx.keyed(key, |cx| {
        let frame_id = cx.frame_id.0;
        let dt = effective_frame_delta_for_cx(cx);

        let out = cx.with_state(LoopProgressState::default, |st| {
            if !st.initialized {
                st.initialized = true;
                st.last_frame_id = frame_id;
                st.elapsed = Duration::ZERO;
                st.period = period;
            }

            if reduced_motion || !enabled || period == Duration::ZERO {
                *st = LoopProgressState::default();
                return DrivenLoopProgress {
                    progress: 0.0,
                    animating: false,
                };
            }

            if st.period != period {
                let frac = if st.period == Duration::ZERO {
                    0.0
                } else {
                    (st.elapsed.as_secs_f64() / st.period.as_secs_f64()).clamp(0.0, 1.0)
                };
                st.period = period;
                st.elapsed = Duration::from_secs_f64(frac * period.as_secs_f64());
            }

            if st.last_frame_id != frame_id {
                st.last_frame_id = frame_id;
                st.elapsed = duration_mod(st.elapsed.saturating_add(dt), st.period);
            } else if st.last_frame_id == 0 {
                st.last_frame_id = frame_id;
            }

            let progress = if st.period == Duration::ZERO {
                0.0
            } else {
                (st.elapsed.as_secs_f64() / st.period.as_secs_f64()).clamp(0.0, 1.0) as f32
            };

            DrivenLoopProgress {
                progress,
                animating: true,
            }
        });

        set_continuous_frames(cx, out.animating);
        if out.animating {
            cx.notify_for_animation_frame();
        }
        out
    })
}

#[derive(Debug, Clone, Copy)]
struct InertiaF32State {
    initialized: bool,
    last_frame_id: u64,
    start: f32,
    start_velocity: f32,
    value: f32,
    velocity: f32,
    elapsed: Duration,
    drag: f64,
    bounds: Option<(f32, f32)>,
    bounce_spring: SpringDescription,
    tolerance: Tolerance,
    last_kick_id: u64,
    animating: bool,
}

impl Default for InertiaF32State {
    fn default() -> Self {
        Self {
            initialized: false,
            last_frame_id: 0,
            start: 0.0,
            start_velocity: 0.0,
            value: 0.0,
            velocity: 0.0,
            elapsed: Duration::ZERO,
            drag: 0.135,
            bounds: None,
            bounce_spring: SpringDescription::with_duration_and_bounce(
                Duration::from_millis(240),
                0.25,
            ),
            tolerance: Tolerance::default(),
            last_kick_id: 0,
            animating: false,
        }
    }
}

#[track_caller]
pub fn drive_inertia_f32<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    kick: Option<InertiaKick>,
    drag: f64,
    bounds: Option<(f32, f32)>,
    bounce_spring: SpringDescription,
    tolerance: Tolerance,
) -> DrivenMotionF32 {
    let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
    if reduced_motion {
        set_continuous_frames(cx, false);
        return DrivenMotionF32 {
            value: cx.with_state(InertiaF32State::default, |st| st.value),
            velocity: 0.0,
            animating: false,
        };
    }

    let loc = Location::caller();
    cx.keyed(
        (loc.file(), loc.line(), loc.column(), "drive_inertia_f32"),
        |cx| {
            let frame_id = cx.frame_id.0;
            let dt = effective_frame_delta_for_cx(cx);

            let out = cx.with_state(InertiaF32State::default, |st| {
                if !st.initialized {
                    st.initialized = true;
                    st.last_frame_id = frame_id;
                    st.start = 0.0;
                    st.start_velocity = 0.0;
                    st.value = 0.0;
                    st.velocity = 0.0;
                    st.elapsed = Duration::ZERO;
                    st.drag = drag;
                    st.bounds = bounds;
                    st.bounce_spring = bounce_spring;
                    st.tolerance = tolerance;
                    st.last_kick_id = kick.map(|k| k.id).unwrap_or(0);
                    st.animating = false;
                }

                let kick_retarget =
                    kick.is_some() && kick.map(|k| k.id).unwrap_or(0) != st.last_kick_id;
                if kick_retarget
                    || st.drag != drag
                    || st.bounds != bounds
                    || st.bounce_spring != bounce_spring
                    || st.tolerance != tolerance
                {
                    if let Some(kick) = kick {
                        st.last_kick_id = kick.id;
                        st.start = st.value;
                        st.start_velocity = kick.velocity;
                        st.velocity = kick.velocity;
                        st.animating = true;
                        st.elapsed = Duration::ZERO;
                    } else if st.animating {
                        // Parameter change while animating: rebase from current state.
                        st.start = st.value;
                        st.start_velocity = st.velocity;
                        st.elapsed = Duration::ZERO;
                    }
                    st.drag = drag;
                    st.bounds = bounds;
                    st.bounce_spring = bounce_spring;
                    st.tolerance = tolerance;
                }

                if st.animating && st.last_frame_id != frame_id {
                    st.last_frame_id = frame_id;
                    st.elapsed = st.elapsed.saturating_add(dt);

                    let inertia_bounds = st.bounds.map(|(min, max)| InertiaBounds {
                        min: min as f64,
                        max: max as f64,
                    });

                    let sim = InertiaSimulation::new(
                        st.start as f64,
                        st.start_velocity as f64,
                        st.drag,
                        inertia_bounds,
                        st.bounce_spring,
                        st.tolerance,
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

#[derive(Debug, Clone, Copy)]
struct SpringF32State {
    initialized: bool,
    last_frame_id: u64,
    start: f32,
    target: f32,
    value: f32,
    velocity: f32,
    elapsed: Duration,
    spring: SpringDescription,
    tolerance: Tolerance,
    snap_to_target: bool,
    last_kick_id: u64,
    animating: bool,
}

impl Default for SpringF32State {
    fn default() -> Self {
        Self {
            initialized: false,
            last_frame_id: 0,
            start: 0.0,
            target: 0.0,
            value: 0.0,
            velocity: 0.0,
            elapsed: Duration::ZERO,
            spring: SpringDescription::with_duration_and_bounce(Duration::from_millis(240), 0.0),
            tolerance: Tolerance::default(),
            snap_to_target: true,
            last_kick_id: 0,
            animating: false,
        }
    }
}

#[track_caller]
pub fn drive_spring_f32<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    target: f32,
    kick: Option<SpringKick>,
    spring: SpringDescription,
    tolerance: Tolerance,
    snap_to_target: bool,
) -> DrivenMotionF32 {
    let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
    if reduced_motion {
        set_continuous_frames(cx, false);
        return DrivenMotionF32 {
            value: target,
            velocity: 0.0,
            animating: false,
        };
    }

    let loc = Location::caller();
    cx.keyed(
        (loc.file(), loc.line(), loc.column(), "drive_spring_f32"),
        |cx| {
            let frame_id = cx.frame_id.0;
            let dt = effective_frame_delta_for_cx(cx);

            let out = cx.with_state(SpringF32State::default, |st| {
                if !st.initialized {
                    st.initialized = true;
                    st.last_frame_id = frame_id;
                    st.start = target;
                    st.target = target;
                    st.value = target;
                    st.velocity = 0.0;
                    st.elapsed = Duration::ZERO;
                    st.spring = spring;
                    st.tolerance = tolerance;
                    st.snap_to_target = snap_to_target;
                    st.last_kick_id = kick.map(|k| k.id).unwrap_or(0);
                    st.animating = false;
                }

                let kick_retarget =
                    kick.is_some() && kick.map(|k| k.id).unwrap_or(0) != st.last_kick_id;

                if target != st.target
                    || st.spring != spring
                    || st.tolerance != tolerance
                    || st.snap_to_target != snap_to_target
                    || kick_retarget
                {
                    st.start = st.value;
                    st.target = target;
                    st.elapsed = Duration::ZERO;
                    st.spring = spring;
                    st.tolerance = tolerance;
                    st.snap_to_target = snap_to_target;
                    st.animating = true;

                    if let Some(kick) = kick
                        && kick.id != st.last_kick_id
                    {
                        st.velocity = kick.velocity;
                        st.last_kick_id = kick.id;
                    }
                }

                if st.animating && st.last_frame_id != frame_id {
                    st.last_frame_id = frame_id;
                    st.elapsed = st.elapsed.saturating_add(dt);

                    let sim = SpringSimulation::new(
                        st.spring,
                        st.start as f64,
                        st.target as f64,
                        st.velocity as f64,
                        st.snap_to_target,
                        st.tolerance,
                    );

                    st.value = sim.x(st.elapsed) as f32;
                    st.velocity = sim.dx(st.elapsed) as f32;

                    if sim.is_done(st.elapsed) {
                        st.value = st.target;
                        st.velocity = 0.0;
                        st.animating = false;
                    }
                } else if st.last_frame_id == 0 {
                    st.last_frame_id = frame_id;
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
    use fret_core::{AppWindowId, WindowFrameClockService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::elements::with_element_cx;

    fn bounds() -> fret_core::Rect {
        fret_core::Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            fret_core::Size::new(fret_core::Px(800.0), fret_core::Px(600.0)),
        )
    }

    #[test]
    fn loop_progress_advances_across_frames() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        let p0 = with_element_cx(&mut app, window, bounds(), "loop", |cx| {
            drive_loop_progress_keyed(cx, "loop_progress", true, Duration::from_secs(2)).progress
        });

        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        let p1 = with_element_cx(&mut app, window, bounds(), "loop", |cx| {
            drive_loop_progress_keyed(cx, "loop_progress", true, Duration::from_secs(2)).progress
        });

        assert!(p1 > p0, "expected loop progress to advance (p0={p0} p1={p1})");
        assert!(p1 < 1.0, "expected loop progress to remain normalized (p1={p1})");
    }

    #[test]
    fn tween_scales_with_fixed_delta_and_settles_in_expected_frames() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(8)));
        });

        for fid in [FrameId(1), FrameId(2)] {
            app.set_frame_id(fid);
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
        }

        fn drive<H: UiHost>(cx: &mut ElementContext<'_, H>, target: f32) -> DrivenMotionF32 {
            drive_tween_f32(
                cx,
                target,
                Duration::from_millis(200),
                crate::headless::easing::linear,
            )
        }

        // Initialize at 0.0 so we can retarget to 1.0 and observe motion over time.
        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(2));
        let _ = with_element_cx(&mut app, window, bounds(), "tween", |cx| drive(cx, 0.0));

        let mut frames = 0u64;
        let mut frame_id = 2u64;
        loop {
            frames += 1;
            frame_id += 1;
            app.set_tick_id(TickId(frames));
            app.set_frame_id(FrameId(frame_id));
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });

            let out = with_element_cx(&mut app, window, bounds(), "tween", |cx| drive(cx, 1.0));
            if !out.animating {
                break;
            }
            assert!(
                frames < 200,
                "tween did not settle in a reasonable number of frames"
            );
        }

        // 200ms / 8ms ~= 25 frames.
        assert_eq!(frames, 25);
    }

    #[test]
    fn spring_settles_with_fixed_delta_and_kick_velocity() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(8)));
        });

        for fid in [FrameId(1), FrameId(2)] {
            app.set_frame_id(fid);
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
        }

        fn drive<H: UiHost>(
            cx: &mut ElementContext<'_, H>,
            target: f32,
            kick: Option<SpringKick>,
        ) -> DrivenMotionF32 {
            drive_spring_f32(
                cx,
                target,
                kick,
                SpringDescription::with_duration_and_bounce(Duration::from_millis(240), 0.0),
                Tolerance::default(),
                true,
            )
        }

        // Initialize at rest.
        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(2));
        let _ = with_element_cx(&mut app, window, bounds(), "spring", |cx| {
            drive(cx, 0.0, None)
        });

        let kick = SpringKick {
            id: 1,
            velocity: 1200.0,
        };
        let mut frame_id = 2u64;
        let mut frames = 0u64;
        loop {
            frames += 1;
            frame_id += 1;
            app.set_tick_id(TickId(frames));
            app.set_frame_id(FrameId(frame_id));
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });

            let out = with_element_cx(&mut app, window, bounds(), "spring", |cx| {
                drive(cx, 1.0, Some(kick))
            });

            if !out.animating {
                assert!((out.value - 1.0).abs() < 1e-4);
                assert!(out.velocity.abs() < 1e-3);
                break;
            }

            assert!(
                frames < 200,
                "spring did not settle in a reasonable number of frames"
            );
        }
    }

    #[test]
    fn inertia_decays_and_respects_bounds_under_fixed_delta() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(8)));
        });

        for fid in [FrameId(1), FrameId(2)] {
            app.set_frame_id(fid);
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
        }

        fn drive<H: UiHost>(
            cx: &mut ElementContext<'_, H>,
            kick: Option<InertiaKick>,
        ) -> DrivenMotionF32 {
            drive_inertia_f32(
                cx,
                kick,
                0.135,
                Some((0.0, 1.0)),
                SpringDescription::with_duration_and_bounce(Duration::from_millis(240), 0.25),
                Tolerance::default(),
            )
        }

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(2));
        let _ = with_element_cx(&mut app, window, bounds(), "inertia", |cx| drive(cx, None));

        let kick = InertiaKick {
            id: 1,
            velocity: 5000.0,
        };

        let mut frames = 0u64;
        let mut frame_id = 2u64;
        let mut saw_motion = false;
        loop {
            frames += 1;
            frame_id += 1;
            app.set_tick_id(TickId(frames));
            app.set_frame_id(FrameId(frame_id));
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });

            let out = with_element_cx(&mut app, window, bounds(), "inertia", |cx| {
                drive(cx, Some(kick))
            });
            if out.animating {
                saw_motion = true;
            }
            assert!(
                (0.0..=1.0).contains(&out.value) || out.value.is_finite(),
                "inertia output must be finite; got value={:?}",
                out.value
            );
            if !out.animating {
                assert!(saw_motion, "expected inertia to animate at least one frame");
                assert!((out.value - 1.0).abs() < 1e-3);
                break;
            }

            assert!(frames < 800, "inertia did not settle in time");
        }
    }
}
