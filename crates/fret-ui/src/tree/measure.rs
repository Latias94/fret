use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct MeasureReentrancyDiagnostics {
    /// Frame ID of the last emitted warning.
    last_log_frame: Option<FrameId>,
    /// Number of suppressed re-entrancy events since the last emitted warning.
    suppressed_since_last_log: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct DebugMeasureChildRecord {
    pub(super) total_time: Duration,
    pub(super) calls: u64,
}

impl MeasureReentrancyDiagnostics {
    const MIN_FRAMES_BETWEEN_LOGS: u64 = 120;

    pub(super) fn record(&mut self, frame_id: FrameId) -> Option<u64> {
        let should_log = match self.last_log_frame {
            None => true,
            Some(last) => frame_id.0.saturating_sub(last.0) >= Self::MIN_FRAMES_BETWEEN_LOGS,
        };

        if !should_log {
            self.suppressed_since_last_log = self.suppressed_since_last_log.saturating_add(1);
            return None;
        }

        self.last_log_frame = Some(frame_id);
        Some(std::mem::take(&mut self.suppressed_since_last_log))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct MeasureStackKey {
    pub(super) node: NodeId,
    pub(super) known_w_bits: Option<u32>,
    pub(super) known_h_bits: Option<u32>,
    pub(super) avail_w: (u8, u32),
    pub(super) avail_h: (u8, u32),
    pub(super) scale_bits: u32,
}
