use crate::runner::streaming_upload::{
    StreamingUploadAck, StreamingUploadAckKind, StreamingUploadStats,
};
use fret_app::Effect;
use fret_core::Event;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn process_streaming_upload_effects(
        &mut self,
        effects: Vec<Effect>,
    ) -> (Vec<Effect>, StreamingUploadStats, usize) {
        let (effects, stats, acks) = self.streaming_uploads.process_effects(
            self.frame_id,
            effects,
            self.config.streaming_upload_budget_bytes_per_frame,
            self.config.streaming_staging_budget_bytes,
            self.config.streaming_update_ack_enabled,
        );
        let ack_count = acks.len();
        if self.config.streaming_update_ack_enabled {
            self.deliver_streaming_upload_acks(acks);
        }
        (effects, stats, ack_count)
    }

    pub(super) fn publish_streaming_upload_diagnostics(&mut self, stats: &StreamingUploadStats) {
        if self.streaming_upload_perf_snapshot_enabled()
            && streaming_upload_stats_have_activity(stats)
        {
            self.publish_streaming_upload_perf_snapshot(stats);
        }

        if streaming_upload_debug_enabled()
            && (stats.update_effects_delayed_budget > 0
                || stats.update_effects_dropped_staging > 0
                || stats.update_effects_replaced > 0
                || stats.yuv_conversions_attempted > 0)
        {
            tracing::debug!(
                seen = stats.update_effects_seen,
                enqueued = stats.update_effects_enqueued,
                replaced = stats.update_effects_replaced,
                applied = stats.update_effects_applied,
                delayed_budget = stats.update_effects_delayed_budget,
                dropped_staging = stats.update_effects_dropped_staging,
                upload_bytes_budgeted = stats.upload_bytes_budgeted,
                upload_bytes_applied = stats.upload_bytes_applied,
                upload_budget_bytes_per_frame = stats.upload_budget_bytes_per_frame,
                staging_budget_bytes = stats.staging_budget_bytes,
                pending_updates = stats.pending_updates,
                pending_staging_bytes = stats.pending_staging_bytes,
                yuv_attempted = stats.yuv_conversions_attempted,
                yuv_applied = stats.yuv_conversions_applied,
                yuv_convert_us = stats.yuv_convert_us,
                yuv_output_bytes = stats.yuv_convert_output_bytes,
                "streaming image updates queued/budgeted"
            );
        }
    }

    pub(super) fn request_pending_streaming_upload_redraws(&mut self) {
        if self.streaming_uploads.has_pending() {
            self.request_streaming_pending_redraws();
        }
    }

    fn deliver_streaming_upload_acks(&mut self, acks: Vec<StreamingUploadAck>) {
        for ack in acks {
            let Some(window) = ack
                .window_hint
                .or(self.main_window)
                .or_else(|| self.windows.keys().next())
            else {
                continue;
            };
            match ack.kind {
                StreamingUploadAckKind::Dropped(reason) => {
                    self.deliver_window_event_now(
                        window,
                        &Event::ImageUpdateDropped {
                            token: ack.token,
                            image: ack.image,
                            reason,
                        },
                    );
                }
            }
        }
    }

    fn streaming_upload_perf_snapshot_enabled(&self) -> bool {
        self.config.streaming_perf_snapshot_enabled || streaming_upload_debug_enabled()
    }

    fn publish_streaming_upload_perf_snapshot(&mut self, stats: &StreamingUploadStats) {
        self.app.set_global(fret_core::StreamingUploadPerfSnapshot {
            frame_id: self.frame_id,
            upload_budget_bytes_per_frame: stats.upload_budget_bytes_per_frame,
            staging_budget_bytes: stats.staging_budget_bytes,
            update_effects_seen: u64::from(stats.update_effects_seen),
            update_effects_enqueued: u64::from(stats.update_effects_enqueued),
            update_effects_replaced: u64::from(stats.update_effects_replaced),
            update_effects_applied: u64::from(stats.update_effects_applied),
            update_effects_delayed_budget: u64::from(stats.update_effects_delayed_budget),
            update_effects_dropped_staging: u64::from(stats.update_effects_dropped_staging),
            upload_bytes_budgeted: stats.upload_bytes_budgeted,
            upload_bytes_applied: stats.upload_bytes_applied,
            pending_updates: u64::from(stats.pending_updates),
            pending_staging_bytes: stats.pending_staging_bytes,
            yuv_convert_us: stats.yuv_convert_us,
            yuv_convert_output_bytes: stats.yuv_convert_output_bytes,
            yuv_conversions_attempted: u64::from(stats.yuv_conversions_attempted),
            yuv_conversions_applied: u64::from(stats.yuv_conversions_applied),
        });
    }
}

fn streaming_upload_debug_enabled() -> bool {
    std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty())
}

fn streaming_upload_stats_have_activity(stats: &StreamingUploadStats) -> bool {
    stats.update_effects_seen > 0
        || stats.update_effects_enqueued > 0
        || stats.update_effects_replaced > 0
        || stats.update_effects_applied > 0
        || stats.update_effects_delayed_budget > 0
        || stats.update_effects_dropped_staging > 0
        || stats.upload_bytes_budgeted > 0
        || stats.upload_bytes_applied > 0
        || stats.pending_updates > 0
        || stats.pending_staging_bytes > 0
        || stats.yuv_conversions_attempted > 0
        || stats.yuv_convert_us > 0
}
