use std::panic::Location;

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
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::{Effect, FrameId, TickId};

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
}
