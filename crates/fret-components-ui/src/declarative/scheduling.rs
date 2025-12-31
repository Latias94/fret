use fret_ui::elements::ContinuousFrames;
use fret_ui::{ElementCx, UiHost};

#[derive(Default)]
struct ContinuousFramesLeaseState {
    lease: Option<ContinuousFrames>,
}

/// Toggle the runtime "continuous frames" lease for the current element.
///
/// Prefer this over emitting `Effect::RequestAnimationFrame` directly from leaf components:
/// - it reduces duplicated scheduling policy in components,
/// - it lets the runtime drive per-window RAF requests while any lease is held,
/// - and it keeps the lease lifetime tied to element state.
pub fn set_continuous_frames<H: UiHost>(cx: &mut ElementCx<'_, H>, enabled: bool) {
    let (start, stop) = cx.with_state(ContinuousFramesLeaseState::default, |st| {
        let start = enabled && st.lease.is_none();
        let stop = !enabled && st.lease.is_some();
        (start, stop)
    });

    if start {
        let lease = cx.begin_continuous_frames();
        cx.with_state(ContinuousFramesLeaseState::default, |st| {
            st.lease = Some(lease);
        });
    } else if stop {
        cx.with_state(ContinuousFramesLeaseState::default, |st| {
            st.lease = None;
        });
    }

    if enabled {
        cx.request_frame();
    }
}

