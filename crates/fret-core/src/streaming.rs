use crate::FrameId;

/// Per-frame counters for streaming image uploads (ADR 0123).
///
/// This is intended for debugging/telemetry surfaces (e.g. an on-screen overlay). The runner
/// updates it when enabled by configuration.
#[derive(Debug, Default, Clone, Copy)]
pub struct StreamingUploadPerfSnapshot {
    pub frame_id: FrameId,

    pub upload_budget_bytes_per_frame: u64,
    pub staging_budget_bytes: u64,

    pub update_effects_seen: u64,
    pub update_effects_enqueued: u64,
    pub update_effects_replaced: u64,
    pub update_effects_applied: u64,
    pub update_effects_delayed_budget: u64,
    pub update_effects_dropped_staging: u64,

    pub upload_bytes_applied: u64,

    pub pending_updates: u64,
    pub pending_staging_bytes: u64,
}
