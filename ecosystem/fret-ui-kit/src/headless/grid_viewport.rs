use std::hash::Hash;
use std::sync::Arc;

use fret_core::Px;

const VIRTUALIZER_PX_SCALE: f32 = 64.0;

fn px_to_units_u32(px: Px) -> u32 {
    let scaled = (px.0.max(0.0) * VIRTUALIZER_PX_SCALE).round();
    scaled.clamp(0.0, u32::MAX as f32) as u32
}

fn px_to_units_u64(px: Px) -> u64 {
    let scaled = (px.0.max(0.0) * VIRTUALIZER_PX_SCALE).round();
    scaled.clamp(0.0, u64::MAX as f32) as u64
}

fn units_u32_to_px(units: u32) -> Px {
    Px(units as f32 / VIRTUALIZER_PX_SCALE)
}

fn units_u64_to_px(units: u64) -> Px {
    Px(units as f32 / VIRTUALIZER_PX_SCALE)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridAxisMeasureMode {
    Fixed,
    Measured,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridAxisRange {
    pub start_index: usize,
    pub end_index: usize,
    pub overscan: usize,
    pub count: usize,
}

pub fn default_range_extractor(range: GridAxisRange) -> Vec<usize> {
    if range.count == 0 {
        return Vec::new();
    }
    let start = range.start_index.saturating_sub(range.overscan);
    let end = (range.end_index + range.overscan).min(range.count.saturating_sub(1));
    (start..=end).collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridAxisItem<K> {
    pub key: K,
    pub index: usize,
    pub start: Px,
    pub end: Px,
    pub size: Px,
}

#[derive(Debug, Clone)]
struct FixedAxisMetrics {
    count: usize,
    estimate_units: u32,
    gap_units: u32,
    padding_start_units: u32,
}

/// Variable-size axis metrics for 2D grid virtualization.
///
/// This is a headless helper that tracks an axis (rows or columns) and computes:
/// - total content size
/// - visible range for a scroll offset and viewport length
/// - per-index start/size/end offsets
///
/// It supports both fixed and measured strategies:
/// - `Fixed`: constant item size (fast path)
/// - `Measured`: size estimates plus measurement write-back per key/index
///
/// Notes:
/// - Size caching is keyed by `K` to preserve measured sizes across reordering.
/// - We scale `Px` into integer units to reduce float precision drift in hot offset math.
#[derive(Debug, Clone)]
pub struct GridAxisMetrics<K> {
    mode: GridAxisMeasureMode,
    estimate: Px,
    gap: Px,
    padding_start: Px,
    keys_signature: (u64, usize),
    keys: Arc<Vec<K>>,
    inner: virtualizer::Virtualizer<K>,
    fixed: FixedAxisMetrics,
}

impl<K> Default for GridAxisMetrics<K>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        let options: virtualizer::VirtualizerOptions<K> =
            virtualizer::VirtualizerOptions::new_with_key(
                0,
                |_| 0,
                |_| {
                    panic!(
                        "GridAxisMetrics default key resolver should not be called for empty axes"
                    )
                },
            );
        Self {
            mode: GridAxisMeasureMode::Measured,
            estimate: Px(0.0),
            gap: Px(0.0),
            padding_start: Px(0.0),
            keys_signature: (0, 0),
            keys: Arc::new(Vec::new()),
            inner: virtualizer::Virtualizer::new(options),
            fixed: FixedAxisMetrics {
                count: 0,
                estimate_units: 0,
                gap_units: 0,
                padding_start_units: 0,
            },
        }
    }
}

impl<K> GridAxisMetrics<K>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
{
    pub fn ensure_with_mode(
        &mut self,
        mode: GridAxisMeasureMode,
        keys: Arc<Vec<K>>,
        items_revision: u64,
        estimate: Px,
        gap: Px,
        padding_start: Px,
    ) {
        match mode {
            GridAxisMeasureMode::Fixed => {
                self.ensure_fixed(keys, items_revision, estimate, gap, padding_start);
            }
            GridAxisMeasureMode::Measured => {
                self.ensure_measured(keys, items_revision, estimate, gap, padding_start);
            }
        }
    }

    pub fn ensure_measured(
        &mut self,
        keys: Arc<Vec<K>>,
        items_revision: u64,
        estimate: Px,
        gap: Px,
        padding_start: Px,
    ) {
        let estimate = Px(estimate.0.max(0.0));
        let gap = Px(gap.0.max(0.0));
        let padding_start = Px(padding_start.0.max(0.0));

        let signature = (items_revision, keys.len());
        if self.mode == GridAxisMeasureMode::Measured
            && self.keys_signature == signature
            && self.estimate == estimate
            && self.gap == gap
            && self.padding_start == padding_start
        {
            return;
        }

        self.mode = GridAxisMeasureMode::Measured;
        self.estimate = estimate;
        self.gap = gap;
        self.padding_start = padding_start;
        self.keys_signature = signature;
        self.keys = Arc::clone(&keys);

        let estimate_units = px_to_units_u32(estimate);
        let gap_units = px_to_units_u32(gap);
        let padding_start_units = px_to_units_u32(padding_start);

        let mut options = self.inner.options().clone();
        options.count = keys.len();
        options.gap = gap_units;
        options.padding_start = padding_start_units;
        options.padding_end = 0;
        options.scroll_margin = 0;
        options.estimate_size = Arc::new(move |_| estimate_units);
        options.get_item_key = Arc::new(move |i| {
            keys.get(i)
                .cloned()
                .unwrap_or_else(|| keys.last().expect("non-empty keys").clone())
        });
        self.inner.set_options(options);
    }

    pub fn ensure_fixed(
        &mut self,
        keys: Arc<Vec<K>>,
        items_revision: u64,
        estimate: Px,
        gap: Px,
        padding_start: Px,
    ) {
        let estimate = Px(estimate.0.max(0.0));
        let gap = Px(gap.0.max(0.0));
        let padding_start = Px(padding_start.0.max(0.0));

        let signature = (items_revision, keys.len());
        if self.mode == GridAxisMeasureMode::Fixed
            && self.keys_signature == signature
            && self.estimate == estimate
            && self.gap == gap
            && self.padding_start == padding_start
        {
            return;
        }

        self.mode = GridAxisMeasureMode::Fixed;
        self.estimate = estimate;
        self.gap = gap;
        self.padding_start = padding_start;
        self.keys_signature = signature;
        self.keys = keys;

        self.fixed = FixedAxisMetrics {
            count: self.keys.len(),
            estimate_units: px_to_units_u32(estimate),
            gap_units: px_to_units_u32(gap),
            padding_start_units: px_to_units_u32(padding_start),
        };
    }

    pub fn total_size(&self) -> Px {
        match self.mode {
            GridAxisMeasureMode::Measured => units_u64_to_px(self.inner.total_size()),
            GridAxisMeasureMode::Fixed => {
                let count = self.fixed.count as u64;
                if count == 0 {
                    return Px(0.0);
                }
                let estimate = self.fixed.estimate_units as u64;
                let gap = self.fixed.gap_units as u64;
                let padding_start = self.fixed.padding_start_units as u64;
                let gaps = count.saturating_sub(1);
                let total_units = padding_start
                    .saturating_add(count.saturating_mul(estimate))
                    .saturating_add(gaps.saturating_mul(gap));
                units_u64_to_px(total_units)
            }
        }
    }

    pub fn clamp_scroll_offset(&self, offset: Px, viewport: Px) -> Px {
        let viewport = Px(viewport.0.max(0.0));
        let total_units = px_to_units_u64(self.total_size());
        let max_offset_units = total_units.saturating_sub(px_to_units_u64(viewport));
        let max_offset = units_u64_to_px(max_offset_units);
        let offset = Px(offset.0.max(0.0));
        Px(offset.0.min(max_offset.0))
    }

    pub fn axis_item(&self, index: usize) -> Option<GridAxisItem<K>> {
        let key = self.keys.get(index)?.clone();
        let start = self.offset_for_index(index);
        let size = self.size_at(index);
        let end = Px((start.0 + size.0).max(0.0));
        Some(GridAxisItem {
            key,
            index,
            start,
            end,
            size,
        })
    }

    pub fn size_at(&self, index: usize) -> Px {
        match self.mode {
            GridAxisMeasureMode::Measured => self
                .inner
                .item_size(index)
                .map(units_u32_to_px)
                .unwrap_or(Px(0.0)),
            GridAxisMeasureMode::Fixed => {
                if index >= self.fixed.count {
                    return Px(0.0);
                }
                units_u32_to_px(self.fixed.estimate_units)
            }
        }
    }

    pub fn offset_for_index(&self, index: usize) -> Px {
        match self.mode {
            GridAxisMeasureMode::Measured => {
                if index >= self.inner.options().count {
                    return self.total_size();
                }
                self.inner
                    .item_start(index)
                    .map(units_u64_to_px)
                    .unwrap_or(Px(0.0))
            }
            GridAxisMeasureMode::Fixed => {
                if index >= self.fixed.count {
                    return self.total_size();
                }
                let stride =
                    (self.fixed.estimate_units as u64).saturating_add(self.fixed.gap_units as u64);
                let start_units = (self.fixed.padding_start_units as u64)
                    .saturating_add((index as u64).saturating_mul(stride));
                units_u64_to_px(start_units)
            }
        }
    }

    pub fn index_for_offset(&self, offset: Px) -> usize {
        match self.mode {
            GridAxisMeasureMode::Measured => {
                if self.inner.options().count == 0 {
                    return 0;
                }
                if offset.0 >= self.total_size().0 {
                    return self.inner.options().count;
                }
                self.inner
                    .index_at_offset(px_to_units_u64(offset))
                    .unwrap_or(0)
            }
            GridAxisMeasureMode::Fixed => {
                let count = self.fixed.count;
                if count == 0 {
                    return 0;
                }
                if offset.0 >= self.total_size().0 {
                    return count;
                }

                let offset_units = px_to_units_u64(offset);
                let padding_start = self.fixed.padding_start_units as u64;
                if offset_units <= padding_start {
                    return 0;
                }
                let stride =
                    (self.fixed.estimate_units as u64).saturating_add(self.fixed.gap_units as u64);
                if stride == 0 {
                    return 0;
                }

                let adjusted = offset_units.saturating_sub(padding_start);
                let idx = adjusted / stride;
                (idx as usize).min(count.saturating_sub(1))
            }
        }
    }

    /// Returns a [`GridAxisRange`] with **inclusive** indices (`start_index..=end_index`).
    pub fn visible_range(
        &self,
        offset: Px,
        viewport: Px,
        overscan: usize,
    ) -> Option<GridAxisRange> {
        let viewport = Px(viewport.0.max(0.0));
        let count = match self.mode {
            GridAxisMeasureMode::Measured => self.inner.options().count,
            GridAxisMeasureMode::Fixed => self.fixed.count,
        };
        if viewport.0 <= 0.0 || count == 0 {
            return None;
        }

        let (start, end) = match self.mode {
            GridAxisMeasureMode::Measured => {
                let range = self
                    .inner
                    .visible_range_for(px_to_units_u64(offset), px_to_units_u32(viewport));
                if range.is_empty() {
                    return None;
                }
                let start = range.start_index;
                let end = range
                    .end_index
                    .saturating_sub(1)
                    .min(count.saturating_sub(1));
                (start, end)
            }
            GridAxisMeasureMode::Fixed => {
                let start = self.index_for_offset(offset);
                if start >= count {
                    return None;
                }

                let end_exclusive = {
                    let offset = Px(offset.0 + viewport.0);
                    let total = self.total_size();
                    if offset.0 >= total.0 {
                        count
                    } else {
                        let offset_units = px_to_units_u64(offset);
                        let padding_start = self.fixed.padding_start_units as u64;
                        if offset_units <= padding_start {
                            1
                        } else {
                            let stride = (self.fixed.estimate_units as u64)
                                .saturating_add(self.fixed.gap_units as u64);
                            if stride == 0 {
                                1
                            } else {
                                let adjusted = offset_units.saturating_sub(padding_start);
                                let idx = adjusted / stride;
                                (idx as usize).saturating_add(1).min(count)
                            }
                        }
                    }
                };

                let end = end_exclusive.saturating_sub(1).min(count.saturating_sub(1));
                (start, end)
            }
        };

        Some(GridAxisRange {
            start_index: start,
            end_index: end,
            overscan,
            count,
        })
    }

    pub fn measure(&mut self, index: usize, size: Px) {
        if self.mode != GridAxisMeasureMode::Measured {
            return;
        }
        let size_units = px_to_units_u32(size);
        let Some(old_units) = self.inner.item_size(index) else {
            return;
        };
        if old_units == size_units {
            return;
        }
        self.inner.measure_unadjusted(index, size_units);
    }

    pub fn reset_measurements(&mut self) {
        if self.mode != GridAxisMeasureMode::Measured {
            return;
        }
        self.inner.reset_measurements();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridViewport2D {
    pub row_range: GridAxisRange,
    pub col_range: GridAxisRange,
    pub scroll_x: Px,
    pub scroll_y: Px,
    pub total_width: Px,
    pub total_height: Px,
}

pub fn compute_grid_viewport_2d<KR, KC>(
    rows: &GridAxisMetrics<KR>,
    cols: &GridAxisMetrics<KC>,
    scroll_x: Px,
    scroll_y: Px,
    viewport_w: Px,
    viewport_h: Px,
    overscan_rows: usize,
    overscan_cols: usize,
) -> Option<GridViewport2D>
where
    KR: Hash + Eq + Clone + Send + Sync + 'static,
    KC: Hash + Eq + Clone + Send + Sync + 'static,
{
    let scroll_x = cols.clamp_scroll_offset(scroll_x, viewport_w);
    let scroll_y = rows.clamp_scroll_offset(scroll_y, viewport_h);

    let row_range = rows.visible_range(scroll_y, viewport_h, overscan_rows)?;
    let col_range = cols.visible_range(scroll_x, viewport_w, overscan_cols)?;

    Some(GridViewport2D {
        row_range,
        col_range,
        scroll_x,
        scroll_y,
        total_width: cols.total_size(),
        total_height: rows.total_size(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_axis_total_size_includes_padding_and_gaps() {
        let mut axis: GridAxisMetrics<u64> = GridAxisMetrics::default();
        axis.ensure_with_mode(
            GridAxisMeasureMode::Fixed,
            Arc::new(vec![0, 1, 2]),
            1,
            Px(10.0),
            Px(2.0),
            Px(5.0),
        );

        // padding 5 + (3 * 10) + (2 gaps * 2) = 39
        assert_eq!(axis.total_size(), Px(39.0));
        assert_eq!(axis.offset_for_index(0), Px(5.0));
        assert_eq!(axis.offset_for_index(1), Px(17.0));
        assert_eq!(axis.offset_for_index(2), Px(29.0));
    }

    #[test]
    fn measured_axis_preserves_sizes_across_reorder_by_key() {
        let mut axis: GridAxisMetrics<u64> = GridAxisMetrics::default();
        axis.ensure_measured(Arc::new(vec![10, 20, 30]), 1, Px(10.0), Px(0.0), Px(0.0));

        // Override the middle item size (key 20) to 50.
        axis.measure(1, Px(50.0));
        assert_eq!(axis.size_at(1), Px(50.0));

        // Reorder keys: move key 20 to index 0.
        axis.ensure_measured(Arc::new(vec![20, 10, 30]), 2, Px(10.0), Px(0.0), Px(0.0));
        assert_eq!(axis.size_at(0), Px(50.0));
    }

    #[test]
    fn grid_viewport_2d_returns_ranges() {
        let mut rows: GridAxisMetrics<u64> = GridAxisMetrics::default();
        rows.ensure_with_mode(
            GridAxisMeasureMode::Fixed,
            Arc::new((0..100).collect()),
            1,
            Px(10.0),
            Px(0.0),
            Px(0.0),
        );

        let mut cols: GridAxisMetrics<u64> = GridAxisMetrics::default();
        cols.ensure_with_mode(
            GridAxisMeasureMode::Fixed,
            Arc::new((0..50).collect()),
            1,
            Px(20.0),
            Px(0.0),
            Px(0.0),
        );

        let vp =
            compute_grid_viewport_2d(&rows, &cols, Px(30.0), Px(25.0), Px(100.0), Px(50.0), 2, 1)
                .expect("viewport");

        // Rows: offset 25, viewport 50 => start 2, end 7 (10px rows)
        assert_eq!(vp.row_range.start_index, 2);
        assert_eq!(vp.row_range.end_index, 7);

        // Cols: offset 30, viewport 100 => start 1, end 6 (20px cols)
        assert_eq!(vp.col_range.start_index, 1);
        assert_eq!(vp.col_range.end_index, 6);
    }
}
