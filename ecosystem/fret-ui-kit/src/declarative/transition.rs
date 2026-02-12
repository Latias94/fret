use std::panic::Location;
use std::time::Duration;

use fret_core::{WindowFrameClockService, WindowMetricsService};
use fret_ui::ElementContext;
use fret_ui::Invalidation;
use fret_ui::UiHost;
use fret_ui::elements::ContinuousFrames;
use fret_ui::theme::CubicBezier;

use crate::headless::transition::{TransitionOutput, TransitionTimeline};

#[derive(Default)]
struct TransitionDriverState {
    initialized: bool,
    last_app_tick: u64,
    last_frame_tick: u64,
    tick: u64,
    configured_open_ticks: u64,
    configured_close_ticks: u64,
    timeline: TransitionTimeline,
    lease: Option<ContinuousFrames>,
}

const REFERENCE_FRAME_DELTA_NS_60HZ: u64 = 1_000_000_000 / 60;

fn scale_60fps_ticks_to_frame_ticks_rounded(ticks: u64, frame_delta: Duration) -> u64 {
    if ticks == 0 {
        return 0;
    }
    if frame_delta == Duration::default() {
        return ticks;
    }
    if frame_delta < Duration::from_millis(1) {
        // When producing frames in a tight loop (common in unit tests), a best-effort frame clock
        // snapshot may report very small deltas that do not correspond to present-time. Avoid
        // scaling in that regime to keep tick-driven tests deterministic.
        return ticks;
    }

    let ref_secs = (REFERENCE_FRAME_DELTA_NS_60HZ as f64) / 1_000_000_000.0;
    let delta_secs = frame_delta.as_secs_f64();
    if delta_secs <= 0.0 {
        return ticks;
    }

    // Desired wall time: ticks * (1/60)s. Actual frame time: delta_secs.
    // Compute how many *frames* we need at this delta to match the intended wall time.
    let scaled = (ticks as f64 * ref_secs / delta_secs).round();
    scaled.clamp(1.0, 10_000.0) as u64
}

pub(crate) fn effective_transition_durations_for_cx<H: UiHost>(
    cx: &ElementContext<'_, H>,
    open_ticks_60fps: u64,
    close_ticks_60fps: u64,
) -> (u64, u64) {
    let Some(svc) = cx.app.global::<WindowFrameClockService>() else {
        return (open_ticks_60fps, close_ticks_60fps);
    };

    let has_window_metrics = cx.app.global::<WindowMetricsService>().is_some();
    let has_fixed_delta = svc.effective_fixed_delta(cx.window).is_some();
    if !has_window_metrics && !has_fixed_delta {
        // Many headless tests drive "frames" without a real window runner. In that setup,
        // `record_frame` deltas reflect CPU time (not present-time), so duration scaling can
        // explode and make interaction tests flaky. Only enable scaling when the host provides
        // real window metrics (runner environment), or when a fixed frame delta is explicitly
        // configured for determinism.
        return (open_ticks_60fps, close_ticks_60fps);
    }

    let Some(frame_delta) = svc.snapshot(cx.window).map(|s| s.delta) else {
        return (open_ticks_60fps, close_ticks_60fps);
    };

    (
        scale_60fps_ticks_to_frame_ticks_rounded(open_ticks_60fps, frame_delta),
        scale_60fps_ticks_to_frame_ticks_rounded(close_ticks_60fps, frame_delta),
    )
}

fn settled_transition_output(open: bool) -> TransitionOutput {
    TransitionOutput {
        present: open,
        linear: if open { 1.0 } else { 0.0 },
        progress: if open { 1.0 } else { 0.0 },
        animating: false,
    }
}

#[track_caller]
pub fn drive_transition<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    ticks: u64,
) -> TransitionOutput {
    let loc = Location::caller();
    cx.keyed((loc.file(), loc.line(), loc.column(), "smoothstep"), |cx| {
        drive_transition_with_durations_and_easing_impl(
            cx,
            open,
            ticks,
            ticks,
            crate::headless::easing::smoothstep,
            true,
        )
    })
}

#[track_caller]
pub fn drive_transition_with_durations<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
) -> TransitionOutput {
    let loc = Location::caller();
    cx.keyed((loc.file(), loc.line(), loc.column(), "smoothstep"), |cx| {
        drive_transition_with_durations_and_easing_impl(
            cx,
            open,
            open_ticks,
            close_ticks,
            crate::headless::easing::smoothstep,
            true,
        )
    })
}

fn drive_transition_with_durations_and_easing_impl<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    ease: fn(f32) -> f32,
    animate_on_mount: bool,
) -> TransitionOutput {
    let (open_ticks, close_ticks) =
        effective_transition_durations_for_cx(cx, open_ticks, close_ticks);

    let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
    if reduced_motion || (open_ticks == 0 && close_ticks == 0) {
        let app_tick = cx.app.tick_id().0;
        let frame_tick = cx.frame_id.0;
        cx.with_state(TransitionDriverState::default, |st| {
            st.initialized = true;
            st.last_app_tick = app_tick;
            st.last_frame_tick = frame_tick;
            st.tick = 0;
            st.configured_open_ticks = open_ticks;
            st.configured_close_ticks = close_ticks;
            st.timeline.set_durations(open_ticks, close_ticks);
            st.lease = None;
        });
        return settled_transition_output(open);
    }

    let app_tick = cx.app.tick_id().0;
    let frame_tick = cx.frame_id.0;

    let (output, start_lease, stop_lease) = cx.with_state(TransitionDriverState::default, |st| {
        if st.configured_open_ticks != open_ticks || st.configured_close_ticks != close_ticks {
            st.configured_open_ticks = open_ticks;
            st.configured_close_ticks = close_ticks;
            st.timeline.set_durations(open_ticks, close_ticks);
        }

        if !st.initialized {
            st.initialized = true;
            st.last_app_tick = app_tick;
            st.last_frame_tick = frame_tick;

            if !animate_on_mount {
                if open {
                    for _ in 0..=open_ticks.max(1) {
                        st.tick = st.tick.saturating_add(1);
                        let seeded = st.timeline.update_with_easing(true, st.tick, ease);
                        if !seeded.animating {
                            break;
                        }
                    }
                } else {
                    let _ = st.timeline.update_with_easing(false, st.tick, ease);
                }

                let settled = TransitionOutput {
                    present: open,
                    linear: if open { 1.0 } else { 0.0 },
                    progress: if open { 1.0 } else { 0.0 },
                    animating: false,
                };
                return (settled, false, false);
            }
        }

        if st.last_frame_tick != frame_tick {
            st.last_frame_tick = frame_tick;
            st.tick = st.tick.saturating_add(1);
        } else if st.last_app_tick != app_tick {
            st.last_app_tick = app_tick;
            st.tick = st.tick.saturating_add(1);
        } else {
            st.tick = st.tick.saturating_add(1);
        }

        let output = st.timeline.update_with_easing(open, st.tick, ease);
        let start_lease = output.animating && st.lease.is_none();
        let stop_lease = !output.animating && st.lease.is_some();
        (output, start_lease, stop_lease)
    });

    if start_lease {
        let lease = cx.begin_continuous_frames();
        cx.with_state(TransitionDriverState::default, |st| {
            st.lease = Some(lease);
        });
    } else if stop_lease {
        cx.with_state(TransitionDriverState::default, |st| {
            st.lease = None;
        });
    }

    if output.animating {
        // Force paint-cache roots to rerun paint while animating (opacity/transform changes).
        cx.notify_for_animation_frame();
        cx.request_frame();
    }

    output
}

#[track_caller]
pub fn drive_transition_with_durations_and_easing<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    ease: fn(f32) -> f32,
) -> TransitionOutput {
    let loc = Location::caller();
    cx.keyed((loc.file(), loc.line(), loc.column()), |cx| {
        drive_transition_with_durations_and_easing_impl(
            cx,
            open,
            open_ticks,
            close_ticks,
            ease,
            true,
        )
    })
}

#[track_caller]
pub fn drive_transition_with_durations_and_easing_with_mount_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    ease: fn(f32) -> f32,
    animate_on_mount: bool,
) -> TransitionOutput {
    let loc = Location::caller();
    cx.keyed(
        (loc.file(), loc.line(), loc.column(), "mount_behavior"),
        |cx| {
            drive_transition_with_durations_and_easing_impl(
                cx,
                open,
                open_ticks,
                close_ticks,
                ease,
                animate_on_mount,
            )
        },
    )
}

#[track_caller]
pub fn drive_transition_with_durations_and_cubic_bezier<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    bezier: CubicBezier,
) -> TransitionOutput {
    let loc = Location::caller();
    cx.keyed(
        (loc.file(), loc.line(), loc.column(), "cubic_bezier"),
        |cx| {
            let (open_ticks, close_ticks) =
                effective_transition_durations_for_cx(cx, open_ticks, close_ticks);

            let reduced_motion = super::prefers_reduced_motion(cx, Invalidation::Paint, false);
            if reduced_motion || (open_ticks == 0 && close_ticks == 0) {
                let app_tick = cx.app.tick_id().0;
                let frame_tick = cx.frame_id.0;
                cx.with_state(TransitionDriverState::default, |st| {
                    st.initialized = true;
                    st.last_app_tick = app_tick;
                    st.last_frame_tick = frame_tick;
                    st.tick = 0;
                    st.configured_open_ticks = open_ticks;
                    st.configured_close_ticks = close_ticks;
                    st.timeline.set_durations(open_ticks, close_ticks);
                    st.lease = None;
                });
                return settled_transition_output(open);
            }

            let app_tick = cx.app.tick_id().0;
            let frame_tick = cx.frame_id.0;

            let (output, start_lease, stop_lease) =
                cx.with_state(TransitionDriverState::default, |st| {
                    if st.configured_open_ticks != open_ticks
                        || st.configured_close_ticks != close_ticks
                    {
                        st.configured_open_ticks = open_ticks;
                        st.configured_close_ticks = close_ticks;
                        st.timeline.set_durations(open_ticks, close_ticks);
                    }

                    if st.last_frame_tick != frame_tick {
                        st.last_frame_tick = frame_tick;
                        st.tick = st.tick.saturating_add(1);
                    } else if st.last_app_tick != app_tick {
                        st.last_app_tick = app_tick;
                        st.tick = st.tick.saturating_add(1);
                    } else {
                        st.tick = st.tick.saturating_add(1);
                    }

                    let output = st.timeline.update_with_cubic_bezier(
                        open, st.tick, bezier.x1, bezier.y1, bezier.x2, bezier.y2,
                    );
                    let start_lease = output.animating && st.lease.is_none();
                    let stop_lease = !output.animating && st.lease.is_some();
                    (output, start_lease, stop_lease)
                });

            if start_lease {
                let lease = cx.begin_continuous_frames();
                cx.with_state(TransitionDriverState::default, |st| {
                    st.lease = Some(lease);
                });
            } else if stop_lease {
                cx.with_state(TransitionDriverState::default, |st| {
                    st.lease = None;
                });
            }

            if output.animating {
                // Force paint-cache roots to rerun paint while animating (opacity/transform changes).
                cx.notify_for_animation_frame();
                cx.request_frame();
            }

            output
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size, WindowFrameClockService};
    use fret_runtime::{Effect, FrameId, TickId};
    use std::time::Duration;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn transition_requests_redraw_while_animating() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));

        let out0 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "t0", |cx| {
            drive_transition_with_durations(cx, true, 3, 3)
        });
        let effects0 = app.flush_effects();
        assert!(out0.present);
        assert!(out0.animating);
        assert!(
            effects0
                .iter()
                .any(|e| *e == Effect::RequestAnimationFrame(window))
        );
        assert!(effects0.iter().any(|e| *e == Effect::Redraw(window)));
    }

    #[test]
    fn multiple_transitions_in_one_element_do_not_share_state() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));

        let (a, b) = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "t1", |cx| {
            let a = drive_transition_with_durations(cx, true, 6, 6);
            let b = drive_transition_with_durations(cx, false, 6, 6);
            (a, b)
        });

        assert!(a.present);
        assert!(a.animating);
        assert!(a.progress > 0.0 && a.progress < 1.0);

        assert!(!b.present);
        assert!(!b.animating);
        assert_eq!(b.progress, 0.0);
    }

    #[test]
    fn transition_can_snap_to_target_on_mount_then_animate_on_toggle() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        let open = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "t2", |cx| {
            drive_transition_with_durations_and_easing_with_mount_behavior(
                cx,
                true,
                6,
                6,
                crate::headless::easing::smoothstep,
                false,
            )
        });
        assert!(open.present);
        assert!(!open.animating);
        assert_eq!(open.progress, 1.0);

        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        let close = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "t2", |cx| {
            drive_transition_with_durations_and_easing_with_mount_behavior(
                cx,
                false,
                6,
                6,
                crate::headless::easing::smoothstep,
                false,
            )
        });
        assert!(!close.present);
        assert!(!close.animating);
        assert_eq!(close.progress, 0.0);
    }

    #[test]
    fn transition_respects_reduced_motion_and_does_not_request_frames() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.with_global_mut(fret_ui::elements::ElementRuntime::new, |rt, _app| {
            rt.set_window_prefers_reduced_motion(window, Some(true));
        });

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));

        let out0 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "t3", |cx| {
            drive_transition_with_durations(cx, true, 6, 6)
        });
        let effects0 = app.flush_effects();

        assert!(out0.present);
        assert!(!out0.animating);
        assert_eq!(out0.progress, 1.0);
        assert!(effects0.is_empty());

        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));

        let out1 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "t3", |cx| {
            drive_transition_with_durations(cx, false, 6, 6)
        });
        let effects1 = app.flush_effects();

        assert!(!out1.present);
        assert!(!out1.animating);
        assert_eq!(out1.progress, 0.0);
        assert!(effects1.is_empty());
    }

    #[test]
    fn transition_scales_60fps_ticks_using_fixed_frame_delta() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(8)));
        });

        // Prime the frame clock so the snapshot delta is non-zero before the first transition
        // evaluation (otherwise the first call sees a 0 delta and may configure unscaled durations).
        for fid in [FrameId(1), FrameId(2)] {
            app.set_frame_id(fid);
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
        }

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

            let out =
                fret_ui::elements::with_element_cx(&mut app, window, bounds(), "t_scale", |cx| {
                    drive_transition_with_durations(cx, true, 12, 12)
                });
            if !out.animating {
                break;
            }
            assert!(
                frames < 200,
                "transition did not settle in a reasonable number of frames"
            );
        }

        // With a fixed delta of 8ms (~125Hz), a 12-tick (60Hz) transition targets ~200ms, which
        // requires ~25 frames.
        assert_eq!(frames, 25);
    }
}
