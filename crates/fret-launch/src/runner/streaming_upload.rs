use std::collections::HashMap;

use fret_core::{AppWindowId, ImageId, RectPx};
use fret_runtime::Effect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StreamingUploadStats {
    pub update_effects_seen: u32,
    pub update_effects_kept: u32,
    pub update_effects_dropped_coalesced: u32,
    pub update_effects_dropped_budget: u32,
    pub upload_bytes_budgeted: u64,
    pub upload_bytes_kept: u64,
}

impl Default for StreamingUploadStats {
    fn default() -> Self {
        Self {
            update_effects_seen: 0,
            update_effects_kept: 0,
            update_effects_dropped_coalesced: 0,
            update_effects_dropped_budget: 0,
            upload_bytes_budgeted: 0,
            upload_bytes_kept: 0,
        }
    }
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

pub(crate) fn coalesce_and_budget_streaming_image_updates(
    effects: Vec<Effect>,
    per_window_budget_bytes: u64,
) -> (Vec<Effect>, StreamingUploadStats) {
    if effects.is_empty() {
        return (effects, StreamingUploadStats::default());
    }

    let mut stats = StreamingUploadStats {
        upload_bytes_budgeted: per_window_budget_bytes,
        ..Default::default()
    };

    let mut last_index: HashMap<StreamingUpdateKey, usize> = HashMap::new();
    for (idx, effect) in effects.iter().enumerate() {
        if let Effect::ImageUpdateRgba8 {
            window,
            image,
            stream_generation,
            ..
        } = effect
        {
            stats.update_effects_seen = stats.update_effects_seen.saturating_add(1);
            last_index.insert(update_key(*window, *image, *stream_generation), idx);
        }
    }

    if last_index.is_empty() {
        return (effects, stats);
    }

    let mut keep = vec![true; effects.len()];
    let mut used_bytes: HashMap<Option<AppWindowId>, u64> = HashMap::new();

    for idx in (0..effects.len()).rev() {
        let Effect::ImageUpdateRgba8 {
            window,
            image,
            stream_generation,
            width,
            height,
            update_rect_px,
            bytes_per_row,
            ..
        } = &effects[idx]
        else {
            continue;
        };

        let key = update_key(*window, *image, *stream_generation);
        if last_index.get(&key) != Some(&idx) {
            keep[idx] = false;
            stats.update_effects_dropped_coalesced =
                stats.update_effects_dropped_coalesced.saturating_add(1);
            continue;
        }

        let upload_bytes =
            estimate_rgba8_upload_bytes(*width, *height, *update_rect_px, *bytes_per_row);
        let used = used_bytes.get(window).copied().unwrap_or(0);

        let allow_oversize_if_first = used == 0;
        if used.saturating_add(upload_bytes) <= per_window_budget_bytes || allow_oversize_if_first {
            keep[idx] = true;
            used_bytes.insert(*window, used.saturating_add(upload_bytes));
            stats.update_effects_kept = stats.update_effects_kept.saturating_add(1);
            stats.upload_bytes_kept = stats.upload_bytes_kept.saturating_add(upload_bytes);
        } else {
            keep[idx] = false;
            stats.update_effects_dropped_budget =
                stats.update_effects_dropped_budget.saturating_add(1);
        }
    }

    let mut out = Vec::with_capacity(effects.len());
    for (idx, effect) in effects.into_iter().enumerate() {
        match effect {
            Effect::ImageUpdateRgba8 { .. } => {
                if keep[idx] {
                    out.push(effect);
                }
            }
            other => out.push(other),
        }
    }

    (out, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::ImageColorSpace;

    #[test]
    fn coalesces_latest_wins_per_key() {
        let image = ImageId::default();
        let w = AppWindowId::default();

        let e0 = Effect::ImageUpdateRgba8 {
            window: Some(w),
            image,
            stream_generation: 0,
            width: 2,
            height: 2,
            update_rect_px: None,
            bytes_per_row: 8,
            bytes: vec![0; 16],
            color_space: ImageColorSpace::Srgb,
        };
        let e1 = Effect::ImageUpdateRgba8 {
            window: Some(w),
            image,
            stream_generation: 0,
            width: 2,
            height: 2,
            update_rect_px: Some(RectPx::new(0, 0, 1, 1)),
            bytes_per_row: 4,
            bytes: vec![1; 4],
            color_space: ImageColorSpace::Srgb,
        };

        let (out, stats) = coalesce_and_budget_streaming_image_updates(vec![e0, e1], u64::MAX);
        assert_eq!(out.len(), 1);
        assert_eq!(stats.update_effects_seen, 2);
        assert_eq!(stats.update_effects_kept, 1);
        assert_eq!(stats.update_effects_dropped_coalesced, 1);
    }

    #[test]
    fn budgets_per_window_and_keeps_most_recent() {
        let image = ImageId::default();
        let w = AppWindowId::default();

        let mk = |generation: u64, fill: u8| Effect::ImageUpdateRgba8 {
            window: Some(w),
            image,
            stream_generation: generation,
            width: 24,
            height: 1,
            update_rect_px: Some(RectPx::new(0, 0, 24, 1)),
            bytes_per_row: 96,
            bytes: vec![fill; 96],
            color_space: ImageColorSpace::Srgb,
        };

        let e0 = mk(0, 0);
        let e1 = mk(1, 1);
        let e2 = mk(2, 2);

        // Each upload will be repacked to 256 bytes_per_row; budget allows only one.
        let (out, stats) = coalesce_and_budget_streaming_image_updates(vec![e0, e1, e2], 256);
        assert_eq!(stats.update_effects_seen, 3);
        assert_eq!(stats.update_effects_kept, 1);
        assert_eq!(stats.update_effects_dropped_budget, 2);
        assert_eq!(out.len(), 1);

        let Effect::ImageUpdateRgba8 {
            stream_generation, ..
        } = out[0]
        else {
            panic!("expected ImageUpdateRgba8");
        };
        assert_eq!(stream_generation, 2);
    }
}
