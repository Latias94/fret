use fret_core::Point;
use fret_ui::elements::ContinuousFrames;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use fret_ui_headless::scroll_area::ScrollAreaType;
use fret_ui_headless::scroll_area_visibility::{
    ScrollAreaVisibility, ScrollAreaVisibilityConfig, ScrollAreaVisibilityInput,
    ScrollAreaVisibilityOutput,
};

/// Default `scrollHideDelay` (600ms) expressed as frame-ish ticks (assuming ~60fps).
pub const DEFAULT_SCROLL_HIDE_DELAY_TICKS: u64 = 36;

/// Radix uses a 100ms debounce to detect "scroll end".
pub const DEFAULT_SCROLL_END_DEBOUNCE_TICKS: u64 = 6;

#[derive(Default)]
struct ScrollAreaVisibilityDriverState {
    last_app_tick: u64,
    last_frame_tick: u64,
    tick: u64,
    last_ty: Option<ScrollAreaType>,
    last_offset: Point,
    visibility: ScrollAreaVisibility,
    lease: Option<ContinuousFrames>,
}

pub fn scrollbar_visibility<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    ty: ScrollAreaType,
    hovered: bool,
    handle: ScrollHandle,
    scroll_hide_delay_ticks: u64,
) -> ScrollAreaVisibilityOutput {
    let app_tick = cx.app.tick_id().0;
    let frame_tick = cx.frame_id.0;

    let (out, start_lease, stop_lease) =
        cx.with_state(ScrollAreaVisibilityDriverState::default, |st| {
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

            let offset = handle.offset();
            let max_offset = handle.max_offset();
            let has_overflow = max_offset.x.0 > 0.01 || max_offset.y.0 > 0.01;

            let scrolled = if st.last_ty != Some(ty) {
                st.last_ty = Some(ty);
                st.last_offset = offset;
                false
            } else {
                let scrolled = (offset.x.0 - st.last_offset.x.0).abs() > 0.01
                    || (offset.y.0 - st.last_offset.y.0).abs() > 0.01;
                st.last_offset = offset;
                scrolled
            };

            let out = st.visibility.update(
                ScrollAreaVisibilityInput {
                    ty,
                    hovered,
                    has_overflow,
                    scrolled,
                    tick: st.tick,
                },
                ScrollAreaVisibilityConfig {
                    scroll_hide_delay_ticks,
                    scroll_end_debounce_ticks: DEFAULT_SCROLL_END_DEBOUNCE_TICKS,
                },
            );

            let start_lease = out.animating && st.lease.is_none();
            let stop_lease = !out.animating && st.lease.is_some();
            (out, start_lease, stop_lease)
        });

    if start_lease {
        let lease = cx.begin_continuous_frames();
        cx.with_state(ScrollAreaVisibilityDriverState::default, |st| {
            st.lease = Some(lease);
        });
    } else if stop_lease {
        cx.with_state(ScrollAreaVisibilityDriverState::default, |st| {
            st.lease = None;
        });
    }

    if out.animating {
        cx.request_frame();
    }

    out
}
