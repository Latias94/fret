use crate::FrameId;

/// Per-frame counters for streaming image uploads (ADR 0121).
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

    /// Estimated CPU->GPU upload bytes used for budget decisions this frame.
    ///
    /// This is computed before applying updates and may be conservative when a platform can
    /// apply an update through a more efficient path (e.g. GPU-assisted YUV conversion).
    pub upload_bytes_budgeted: u64,

    /// Actual CPU->GPU upload bytes performed by applied updates this frame.
    pub upload_bytes_applied: u64,

    pub pending_updates: u64,
    pub pending_staging_bytes: u64,

    /// Total CPU time spent preparing YUV updates during this frame (microseconds).
    ///
    /// This includes CPU-side work such as plane repacking and command encoding; it does not
    /// include GPU execution time.
    pub yuv_convert_us: u64,
    /// Total RGBA output bytes produced (or written) by the YUV conversion path during this frame.
    pub yuv_convert_output_bytes: u64,
    pub yuv_conversions_attempted: u64,
    pub yuv_conversions_applied: u64,
}
