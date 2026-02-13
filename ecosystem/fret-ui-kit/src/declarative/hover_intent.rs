use fret_ui::{ElementContext, UiHost};

use crate::declarative::scheduling;
use crate::declarative::transition;
use crate::headless::hover_intent::{HoverIntentConfig, HoverIntentState, HoverIntentUpdate};

#[derive(Debug, Default, Clone, Copy)]
struct HoverIntentDriverState {
    last_frame_tick: Option<u64>,
    tick: u64,
    intent: HoverIntentState,
}

/// Drive [`HoverIntentState`] using the UI runtime's frame clock, and request continuous frames
/// while the intent is in a delayed-transition state.
pub fn hover_intent<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hovered: bool,
    cfg: HoverIntentConfig,
) -> HoverIntentUpdate {
    let (open_delay_ticks, close_delay_ticks) = transition::effective_transition_durations_for_cx(
        &*cx,
        cfg.open_delay_ticks,
        cfg.close_delay_ticks,
    );
    let cfg = HoverIntentConfig::new(open_delay_ticks, close_delay_ticks);

    let frame_tick = cx.app.frame_id().0;
    let update = cx.with_state(HoverIntentDriverState::default, |st| {
        match st.last_frame_tick {
            None => {
                st.last_frame_tick = Some(frame_tick);
                st.tick = frame_tick;
            }
            Some(prev) if prev != frame_tick => {
                st.last_frame_tick = Some(frame_tick);
                st.tick = frame_tick;
            }
            Some(_) => {
                // In some unit tests the runner-owned frame clock may not advance; fall back to a
                // per-call monotonic tick so delays can still elapse deterministically.
                st.tick = st.tick.saturating_add(1);
            }
        }
        st.intent.update(hovered, st.tick, cfg)
    });

    scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);
    update
}
