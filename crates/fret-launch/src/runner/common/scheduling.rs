use fret_runtime::{FrameId, TickId};

/// Advance the runner's monotonic turn counter by exactly one turn.
pub(crate) fn begin_turn(tick_id: &mut TickId) -> TickId {
    tick_id.0 = tick_id.0.saturating_add(1);
    *tick_id
}

/// Commit exactly one successfully presented frame.
pub(crate) fn commit_presented_frame(frame_id: &mut FrameId) -> FrameId {
    frame_id.0 = frame_id.0.saturating_add(1);
    *frame_id
}

/// Commit a successfully presented frame and run follow-up bookkeeping with the committed id.
pub(crate) fn commit_presented_frame_and_then(
    frame_id: &mut FrameId,
    after_commit: impl FnOnce(FrameId),
) -> FrameId {
    let committed = commit_presented_frame(frame_id);
    after_commit(committed);
    committed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_turn_advances_tick_once_per_turn() {
        let mut tick_id = TickId::default();

        assert_eq!(begin_turn(&mut tick_id), TickId(1));
        assert_eq!(tick_id, TickId(1));

        assert_eq!(begin_turn(&mut tick_id), TickId(2));
        assert_eq!(tick_id, TickId(2));
    }

    #[test]
    fn commit_presented_frame_advances_frame_once_per_present() {
        let mut frame_id = FrameId::default();

        assert_eq!(commit_presented_frame(&mut frame_id), FrameId(1));
        assert_eq!(frame_id, FrameId(1));

        assert_eq!(commit_presented_frame(&mut frame_id), FrameId(2));
        assert_eq!(frame_id, FrameId(2));
    }

    #[test]
    fn commit_presented_frame_and_then_observes_committed_frame() {
        let mut frame_id = FrameId::default();
        let mut observed = Vec::new();

        assert_eq!(
            commit_presented_frame_and_then(&mut frame_id, |committed| observed.push(committed.0)),
            FrameId(1)
        );
        assert_eq!(
            commit_presented_frame_and_then(&mut frame_id, |committed| observed.push(committed.0)),
            FrameId(2)
        );

        assert_eq!(observed, vec![1, 2]);
        assert_eq!(frame_id, FrameId(2));
    }

    #[test]
    fn counter_advances_saturate_at_max() {
        let mut tick_id = TickId(u64::MAX);
        let mut frame_id = FrameId(u64::MAX);

        assert_eq!(begin_turn(&mut tick_id), TickId(u64::MAX));
        assert_eq!(commit_presented_frame(&mut frame_id), FrameId(u64::MAX));
    }
}
