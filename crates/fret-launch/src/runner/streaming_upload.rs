use std::collections::{HashMap, VecDeque};

use fret_core::{AppWindowId, FrameId, ImageId, RectPx};
use fret_runtime::Effect;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StreamingUploadStats {
    pub update_effects_seen: u32,
    pub update_effects_enqueued: u32,
    pub update_effects_replaced: u32,
    pub update_effects_applied: u32,
    pub update_effects_delayed_budget: u32,
    pub update_effects_dropped_staging: u32,
    pub upload_budget_bytes_per_frame: u64,
    pub upload_bytes_applied: u64,
    pub staging_budget_bytes: u64,
    pub pending_updates: u32,
    pub pending_staging_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StreamingUpdateKey {
    window: Option<AppWindowId>,
    image: ImageId,
    stream_generation: u64,
}

fn update_key(
    window: Option<AppWindowId>,
    image: ImageId,
    stream_generation: u64,
) -> StreamingUpdateKey {
    StreamingUpdateKey {
        window,
        image,
        stream_generation,
    }
}

#[derive(Debug)]
struct PendingUpdate {
    seq: u64,
    effect: Effect,
    staging_bytes: u64,
}

fn staging_bytes_for_effect(effect: &Effect) -> u64 {
    match effect {
        Effect::ImageUpdateRgba8 { bytes, .. } => bytes.len() as u64,
        _ => 0,
    }
}

fn update_rect_or_full(width: u32, height: u32, update_rect_px: Option<RectPx>) -> RectPx {
    update_rect_px.unwrap_or_else(|| RectPx::full(width, height))
}

fn round_up(value: u32, align: u32) -> u32 {
    if align == 0 {
        return value;
    }
    value.div_ceil(align).saturating_mul(align)
}

fn estimate_rgba8_upload_bytes(
    width: u32,
    height: u32,
    update_rect_px: Option<RectPx>,
    bytes_per_row: u32,
) -> u64 {
    let rect = update_rect_or_full(width, height, update_rect_px);
    let row_bytes = rect.w.saturating_mul(4);
    let aligned_row_bytes = round_up(row_bytes, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let effective_bpr = if bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0 {
        bytes_per_row
    } else {
        aligned_row_bytes.max(row_bytes)
    };
    (effective_bpr as u64).saturating_mul(rect.h as u64)
}

fn estimate_upload_bytes(effect: &Effect) -> u64 {
    match effect {
        Effect::ImageUpdateRgba8 {
            width,
            height,
            update_rect_px,
            bytes_per_row,
            ..
        } => estimate_rgba8_upload_bytes(*width, *height, *update_rect_px, *bytes_per_row),
        _ => 0,
    }
}

pub(crate) struct StreamingUploadQueue {
    next_seq: u64,
    frame_id: FrameId,
    used_upload_bytes_this_frame: HashMap<Option<AppWindowId>, u64>,

    pending: HashMap<StreamingUpdateKey, PendingUpdate>,
    pending_order: HashMap<Option<AppWindowId>, VecDeque<(StreamingUpdateKey, u64)>>,
    pending_staging_bytes: HashMap<Option<AppWindowId>, u64>,
}

impl Default for StreamingUploadQueue {
    fn default() -> Self {
        Self {
            next_seq: 1,
            frame_id: FrameId::default(),
            used_upload_bytes_this_frame: HashMap::new(),
            pending: HashMap::new(),
            pending_order: HashMap::new(),
            pending_staging_bytes: HashMap::new(),
        }
    }
}

impl StreamingUploadQueue {
    pub(crate) fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub(crate) fn pending_redraw_hint(&self) -> Option<Vec<AppWindowId>> {
        if self.pending.is_empty() {
            return None;
        }

        let mut windows: Vec<AppWindowId> = Vec::new();
        for key in self.pending.keys() {
            if let Some(w) = key.window {
                windows.push(w);
            } else {
                return Some(Vec::new());
            }
        }
        Some(windows)
    }

    fn begin_frame_if_needed(&mut self, frame_id: FrameId) {
        if self.frame_id == frame_id {
            return;
        }
        self.frame_id = frame_id;
        self.used_upload_bytes_this_frame.clear();
    }

    fn enqueue_update(&mut self, effect: Effect, stats: &mut StreamingUploadStats) {
        let Effect::ImageUpdateRgba8 {
            window,
            image,
            stream_generation,
            ..
        } = &effect
        else {
            return;
        };

        let key = update_key(*window, *image, *stream_generation);
        let staging_bytes = staging_bytes_for_effect(&effect);
        let seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);

        let prev = self.pending.insert(
            key,
            PendingUpdate {
                seq,
                effect,
                staging_bytes,
            },
        );

        let window_bucket = key.window;
        let order = self
            .pending_order
            .entry(window_bucket)
            .or_insert_with(VecDeque::new);
        order.push_back((key, seq));

        let bytes = self.pending_staging_bytes.entry(window_bucket).or_insert(0);
        if let Some(prev) = prev {
            stats.update_effects_replaced = stats.update_effects_replaced.saturating_add(1);
            *bytes = bytes.saturating_sub(prev.staging_bytes);
        }
        *bytes = bytes.saturating_add(staging_bytes);

        stats.update_effects_enqueued = stats.update_effects_enqueued.saturating_add(1);
    }

    fn enforce_staging_budget(
        &mut self,
        per_window_staging_budget_bytes: u64,
        stats: &mut StreamingUploadStats,
    ) {
        if per_window_staging_budget_bytes == 0 {
            return;
        }

        let windows: Vec<Option<AppWindowId>> =
            self.pending_staging_bytes.keys().copied().collect();
        for window in windows {
            while self
                .pending_staging_bytes
                .get(&window)
                .copied()
                .unwrap_or(0)
                > per_window_staging_budget_bytes
            {
                let Some(order) = self.pending_order.get_mut(&window) else {
                    break;
                };
                let Some((key, seq)) = order.pop_front() else {
                    break;
                };
                let Some(entry) = self.pending.get(&key) else {
                    continue;
                };
                if entry.seq != seq {
                    continue;
                }
                let removed = self.pending.remove(&key);
                if let Some(removed) = removed {
                    if let Some(bytes) = self.pending_staging_bytes.get_mut(&window) {
                        *bytes = bytes.saturating_sub(removed.staging_bytes);
                    }
                    stats.update_effects_dropped_staging =
                        stats.update_effects_dropped_staging.saturating_add(1);
                }
            }
        }
    }

    fn drain_updates_for_frame(
        &mut self,
        per_window_upload_budget_bytes: u64,
        stats: &mut StreamingUploadStats,
    ) -> Vec<Effect> {
        let mut out: Vec<Effect> = Vec::new();
        if self.pending.is_empty() {
            return out;
        }

        let windows: Vec<Option<AppWindowId>> = self.pending_order.keys().copied().collect();
        for window in windows {
            let Some(order) = self.pending_order.get_mut(&window) else {
                continue;
            };

            let used = self
                .used_upload_bytes_this_frame
                .get(&window)
                .copied()
                .unwrap_or(0);
            let mut used = used;
            let mut applied_any = false;

            let mut skipped: Vec<(StreamingUpdateKey, u64)> = Vec::new();
            while let Some((key, seq)) = order.pop_back() {
                let Some(entry) = self.pending.get(&key) else {
                    continue;
                };
                if entry.seq != seq {
                    continue;
                }

                let upload_bytes = estimate_upload_bytes(&entry.effect);
                let allow_oversize_if_first =
                    per_window_upload_budget_bytes > 0 && !applied_any && used == 0;
                if used.saturating_add(upload_bytes) > per_window_upload_budget_bytes
                    && !allow_oversize_if_first
                {
                    skipped.push((key, seq));
                    continue;
                }

                let removed = self.pending.remove(&key);
                if let Some(removed) = removed {
                    if let Some(bytes) = self.pending_staging_bytes.get_mut(&window) {
                        *bytes = bytes.saturating_sub(removed.staging_bytes);
                    }
                    used = used.saturating_add(upload_bytes);
                    applied_any = true;
                    stats.update_effects_applied = stats.update_effects_applied.saturating_add(1);
                    stats.upload_bytes_applied =
                        stats.upload_bytes_applied.saturating_add(upload_bytes);
                    out.push(removed.effect);
                }
            }

            for item in skipped.into_iter().rev() {
                order.push_back(item);
            }

            self.used_upload_bytes_this_frame.insert(window, used);
        }

        out
    }

    pub(crate) fn process_effects(
        &mut self,
        frame_id: FrameId,
        effects: Vec<Effect>,
        per_window_upload_budget_bytes: u64,
        per_window_staging_budget_bytes: u64,
    ) -> (Vec<Effect>, StreamingUploadStats) {
        self.begin_frame_if_needed(frame_id);

        let mut stats = StreamingUploadStats {
            upload_budget_bytes_per_frame: per_window_upload_budget_bytes,
            staging_budget_bytes: per_window_staging_budget_bytes,
            ..Default::default()
        };

        let mut out: Vec<Effect> = Vec::with_capacity(effects.len());
        for effect in effects {
            match effect {
                Effect::ImageUpdateRgba8 { .. } => {
                    stats.update_effects_seen = stats.update_effects_seen.saturating_add(1);
                    self.enqueue_update(effect, &mut stats);
                }
                other => out.push(other),
            }
        }

        self.enforce_staging_budget(per_window_staging_budget_bytes, &mut stats);

        let pending_before = self.pending.len() as u32;
        let mut applied = self.drain_updates_for_frame(per_window_upload_budget_bytes, &mut stats);
        out.append(&mut applied);

        let pending_after = self.pending.len() as u32;
        stats.pending_updates = pending_after;
        stats.pending_staging_bytes = self
            .pending_staging_bytes
            .values()
            .copied()
            .fold(0u64, |acc, v| acc.saturating_add(v));

        let not_applied_this_turn = pending_before.saturating_sub(stats.update_effects_applied);
        if not_applied_this_turn > 0 {
            stats.update_effects_delayed_budget = not_applied_this_turn;
        }

        (out, stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(window: AppWindowId, image: ImageId, stream_generation: u64, fill: u8) -> Effect {
        Effect::ImageUpdateRgba8 {
            window: Some(window),
            image,
            stream_generation,
            width: 2,
            height: 2,
            update_rect_px: None,
            bytes_per_row: 8,
            bytes: vec![fill; 16],
            color_space: fret_runtime::ImageColorSpace::Srgb,
        }
    }

    #[test]
    fn coalesces_latest_wins_per_key_and_applies_once() {
        let image = ImageId::default();
        let w = AppWindowId::default();

        let e0 = update(w, image, 0, 1);
        let e1 = update(w, image, 0, 2);

        let mut q = StreamingUploadQueue::default();
        let (out, stats) = q.process_effects(FrameId(1), vec![e0, e1], u64::MAX, u64::MAX);
        assert_eq!(stats.update_effects_seen, 2);
        assert_eq!(stats.update_effects_replaced, 1);
        assert_eq!(stats.update_effects_applied, 1);
        assert_eq!(out.len(), 1);

        let Effect::ImageUpdateRgba8 { bytes, .. } = &out[0] else {
            panic!("expected ImageUpdateRgba8");
        };
        assert_eq!(bytes[0], 2);
    }

    #[test]
    fn delays_under_budget_and_applies_over_multiple_frames() {
        let image = ImageId::default();
        let w = AppWindowId::default();

        let e0 = update(w, image, 0, 0);
        let e1 = update(w, image, 1, 1);
        let e2 = update(w, image, 2, 2);

        let mut q = StreamingUploadQueue::default();

        // Budget only allows one upload per frame.
        let (out, stats) = q.process_effects(FrameId(1), vec![e0, e1, e2], 32, u64::MAX);
        assert_eq!(stats.update_effects_seen, 3);
        assert_eq!(stats.update_effects_applied, 1);
        assert!(q.has_pending());
        assert_eq!(out.len(), 1);

        let (out2, stats2) = q.process_effects(FrameId(2), Vec::new(), 32, u64::MAX);
        assert_eq!(stats2.update_effects_applied, 1);
        assert_eq!(out2.len(), 1);

        let (out3, stats3) = q.process_effects(FrameId(3), Vec::new(), 32, u64::MAX);
        assert_eq!(stats3.update_effects_applied, 1);
        assert_eq!(out3.len(), 1);

        assert!(!q.has_pending());
    }

    #[test]
    fn drops_oldest_when_staging_budget_exceeded() {
        let image = ImageId::default();
        let w = AppWindowId::default();

        let e0 = update(w, image, 0, 0);
        let e1 = update(w, image, 1, 1);

        let mut q = StreamingUploadQueue::default();
        // Staging budget can only retain one update (each is 16 bytes).
        let (_out, stats) = q.process_effects(FrameId(1), vec![e0, e1], 0, 16);
        assert_eq!(stats.update_effects_seen, 2);
        assert_eq!(stats.update_effects_dropped_staging, 1);
        assert_eq!(stats.pending_updates, 1);
    }
}
