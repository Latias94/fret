use std::panic::Location;
use std::time::Duration;

use fret_core::{WindowFrameClockService, WindowMetricsService};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::declarative::scheduling::set_continuous_frames;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrivenMotionF32 {
    pub value: f32,
    pub velocity: f32,
    pub animating: bool,
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

const REFERENCE_FRAME_DELTA_60HZ: Duration = Duration::from_nanos(1_000_000_000 / 60);
const MAX_FRAME_DELTA: Duration = Duration::from_millis(50);

fn clamp_frame_delta(dt: Duration) -> Duration {
    if dt == Duration::ZERO {
        return REFERENCE_FRAME_DELTA_60HZ;
    }
    dt.min(MAX_FRAME_DELTA)
}

fn effective_frame_delta_for_cx<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
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

fn tween_value_at(
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

fn tween_velocity_at(
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
}
