use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::elements::ContinuousFrames;

use crate::headless::presence::{FadePresence, PresenceOutput};

#[derive(Default)]
struct FadePresenceDriverState {
    last_app_tick: u64,
    last_frame_tick: u64,
    tick: u64,
    configured_fade_ticks: u64,
    presence: FadePresence,
    lease: Option<ContinuousFrames>,
}

/// Drive [`FadePresence`] using the UI runtime's monotonic frame clock.
///
/// This helper keeps animation scheduling out of individual components:
/// - it uses `cx.frame_id` as a stable tick source,
/// - it holds a continuous-frames lease while animating (so the runtime continues to request RAF),
/// - and it requests redraws while animating.
pub fn fade_presence<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    fade_ticks: u64,
) -> PresenceOutput {
    let app_tick = cx.app.tick_id().0;
    let frame_tick = cx.frame_id.0;

    let (output, start_lease, stop_lease) = cx.with_state(FadePresenceDriverState::default, |st| {
        if st.configured_fade_ticks != fade_ticks {
            st.configured_fade_ticks = fade_ticks;
            st.presence.set_fade_ticks(fade_ticks);
        }

        // Prefer the runner-owned monotonic clocks when they advance.
        // In unit tests these values may be left at `0`, so we fall back to "call count".
        if st.last_frame_tick != frame_tick {
            st.last_frame_tick = frame_tick;
            st.tick = st.tick.saturating_add(1);
        } else if st.last_app_tick != app_tick {
            st.last_app_tick = app_tick;
            st.tick = st.tick.saturating_add(1);
        } else {
            st.tick = st.tick.saturating_add(1);
        }

        let output = st.presence.update(open, st.tick);
        let start_lease = output.animating && st.lease.is_none();
        let stop_lease = !output.animating && st.lease.is_some();
        (output, start_lease, stop_lease)
    });

    if start_lease {
        let lease = cx.begin_continuous_frames();
        cx.with_state(FadePresenceDriverState::default, |st| {
            st.lease = Some(lease);
        });
    } else if stop_lease {
        cx.with_state(FadePresenceDriverState::default, |st| {
            st.lease = None;
        });
    }

    if output.animating {
        cx.request_frame();
    }

    output
}
