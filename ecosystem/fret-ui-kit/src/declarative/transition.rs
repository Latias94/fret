use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::elements::ContinuousFrames;
use fret_ui::theme::CubicBezier;

use crate::headless::transition::{TransitionOutput, TransitionTimeline};

#[derive(Default)]
struct TransitionDriverState {
    last_app_tick: u64,
    last_frame_tick: u64,
    tick: u64,
    configured_open_ticks: u64,
    configured_close_ticks: u64,
    timeline: TransitionTimeline,
    lease: Option<ContinuousFrames>,
}

pub fn drive_transition<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    ticks: u64,
) -> TransitionOutput {
    drive_transition_with_durations_and_easing(
        cx,
        open,
        ticks,
        ticks,
        crate::headless::easing::smoothstep,
    )
}

pub fn drive_transition_with_durations<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
) -> TransitionOutput {
    drive_transition_with_durations_and_easing(
        cx,
        open,
        open_ticks,
        close_ticks,
        crate::headless::easing::smoothstep,
    )
}

pub fn drive_transition_with_durations_and_easing<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    ease: fn(f32) -> f32,
) -> TransitionOutput {
    let app_tick = cx.app.tick_id().0;
    let frame_tick = cx.frame_id.0;

    let (output, start_lease, stop_lease) = cx.with_state(TransitionDriverState::default, |st| {
        if st.configured_open_ticks != open_ticks || st.configured_close_ticks != close_ticks {
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
        cx.request_frame();
    }

    output
}

pub fn drive_transition_with_durations_and_cubic_bezier<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    bezier: CubicBezier,
) -> TransitionOutput {
    let app_tick = cx.app.tick_id().0;
    let frame_tick = cx.frame_id.0;

    let (output, start_lease, stop_lease) = cx.with_state(TransitionDriverState::default, |st| {
        if st.configured_open_ticks != open_ticks || st.configured_close_ticks != close_ticks {
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

        let output = st
            .timeline
            .update_with_cubic_bezier(open, st.tick, bezier.x1, bezier.y1, bezier.x2, bezier.y2);
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
        cx.request_frame();
    }

    output
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
}
