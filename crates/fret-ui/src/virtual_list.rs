use fret_core::Px;
use std::sync::Arc;

use crate::element::VirtualListMeasureMode;
use crate::scroll::ScrollStrategy;

#[cfg(test)]
use std::cell::Cell;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualItem {
    pub key: crate::ItemKey,
    pub index: usize,
    pub start: Px,
    pub end: Px,
    pub size: Px,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualRange {
    pub start_index: usize,
    pub end_index: usize,
    pub overscan: usize,
    pub count: usize,
}

pub fn default_range_extractor(range: VirtualRange) -> Vec<usize> {
    if range.count == 0 {
        return Vec::new();
    }
    let start = range.start_index.saturating_sub(range.overscan);
    let end = (range.end_index + range.overscan).min(range.count.saturating_sub(1));

    (start..=end).collect()
}

#[derive(Debug, Clone)]
pub struct VirtualListMetrics {
    estimate: Px,
    gap: Px,
    scroll_margin: Px,
    mode: VirtualListMeasureMode,
    inner: virtualizer::Virtualizer<crate::ItemKey>,
    keys_signature: (u64, usize),
    measured_cross_extent_units: u32,
    fixed: FixedMetrics,
}

#[derive(Debug, Clone, Copy)]
struct FixedMetrics {
    count: usize,
    estimate_units: u32,
    gap_units: u32,
    padding_start_units: u32,
}

impl Default for VirtualListMetrics {
    fn default() -> Self {
        let options = virtualizer::VirtualizerOptions::new(0, |_| 0);
        Self {
            estimate: Px(0.0),
            gap: Px(0.0),
            scroll_margin: Px(0.0),
            mode: VirtualListMeasureMode::Measured,
            inner: virtualizer::Virtualizer::new(options),
            keys_signature: (0, 0),
            measured_cross_extent_units: 0,
            fixed: FixedMetrics {
                count: 0,
                estimate_units: 0,
                gap_units: 0,
                padding_start_units: 0,
            },
        }
    }
}

impl VirtualListMetrics {
    pub fn ensure_with_mode(
        &mut self,
        mode: VirtualListMeasureMode,
        len: usize,
        estimate: Px,
        gap: Px,
        scroll_margin: Px,
    ) {
        match mode {
            VirtualListMeasureMode::Measured => {
                self.ensure_measured(len, estimate, gap, scroll_margin)
            }
            VirtualListMeasureMode::Fixed => self.ensure_fixed(len, estimate, gap, scroll_margin),
        }
    }

    pub fn ensure(&mut self, len: usize, estimate: Px, gap: Px, scroll_margin: Px) {
        self.ensure_measured(len, estimate, gap, scroll_margin);
    }

    fn ensure_fixed(&mut self, len: usize, estimate: Px, gap: Px, scroll_margin: Px) {
        let estimate = Px(estimate.0.max(0.0));
        let gap = Px(gap.0.max(0.0));
        let scroll_margin = Px(scroll_margin.0.max(0.0));

        if self.mode == VirtualListMeasureMode::Fixed
            && self.fixed.count == len
            && self.estimate == estimate
            && self.gap == gap
            && self.scroll_margin == scroll_margin
        {
            return;
        }

        self.mode = VirtualListMeasureMode::Fixed;
        self.estimate = estimate;
        self.gap = gap;
        self.scroll_margin = scroll_margin;

        self.fixed = FixedMetrics {
            count: len,
            estimate_units: px_to_units_u32(estimate),
            gap_units: px_to_units_u32(gap),
            padding_start_units: px_to_units_u32(scroll_margin),
        };
    }

    fn ensure_measured(&mut self, len: usize, estimate: Px, gap: Px, scroll_margin: Px) {
        let estimate = Px(estimate.0.max(0.0));
        let gap = Px(gap.0.max(0.0));
        let scroll_margin = Px(scroll_margin.0.max(0.0));
        if self.mode == VirtualListMeasureMode::Measured
            && self.inner.options().count == len
            && self.estimate == estimate
            && self.gap == gap
            && self.scroll_margin == scroll_margin
        {
            return;
        }

        self.mode = VirtualListMeasureMode::Measured;
        self.estimate = estimate;
        self.gap = gap;
        self.scroll_margin = scroll_margin;

        let estimate_units = px_to_units_u32(estimate);
        let gap_units = px_to_units_u32(gap);
        let padding_start = px_to_units_u32(scroll_margin);

        let mut options = self.inner.options().clone();
        options.count = len;
        options.gap = gap_units;
        options.padding_start = padding_start;
        options.padding_end = 0;
        options.scroll_margin = 0;
        options.estimate_size = Arc::new(move |_| estimate_units);
        self.inner.set_options(options);
    }

    pub fn sync_keys(&mut self, keys: &[crate::ItemKey], items_revision: u64) {
        let signature = (items_revision, keys.len());
        if self.keys_signature == signature {
            return;
        }

        if self.mode == VirtualListMeasureMode::Fixed {
            self.keys_signature = signature;
            return;
        }

        let keys = Arc::new(keys.to_vec());
        let mut options = self.inner.options().clone();
        options.get_item_key = Arc::new({
            let keys = Arc::clone(&keys);
            move |i| keys.get(i).copied().unwrap_or(i as crate::ItemKey)
        });
        self.inner.set_options(options);
        self.keys_signature = signature;
    }

    pub fn total_height(&self) -> Px {
        match self.mode {
            VirtualListMeasureMode::Measured => units_u64_to_px(self.inner.total_size()),
            VirtualListMeasureMode::Fixed => {
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

    pub fn virtual_item(&self, index: usize, key: crate::ItemKey) -> VirtualItem {
        let start = self.offset_for_index(index);
        let size = self.height_at(index);
        let end = Px((start.0 + size.0).max(0.0));
        VirtualItem {
            key,
            index,
            start,
            end,
            size,
        }
    }

    pub fn estimate(&self) -> Px {
        self.estimate
    }

    pub fn gap(&self) -> Px {
        self.gap
    }

    pub fn scroll_margin(&self) -> Px {
        self.scroll_margin
    }

    pub fn is_measured(&self, index: usize) -> bool {
        match self.mode {
            VirtualListMeasureMode::Measured => {
                if index >= self.inner.options().count {
                    return false;
                }
                self.inner.is_measured(index)
            }
            VirtualListMeasureMode::Fixed => index < self.fixed.count,
        }
    }

    pub fn reset_measured_cache_if_cross_extent_changed(&mut self, cross_extent: Px) -> bool {
        if self.mode != VirtualListMeasureMode::Measured {
            return false;
        }

        let units = px_to_units_u32(Px(cross_extent.0.max(0.0)));
        if self.measured_cross_extent_units == units {
            return false;
        }

        self.measured_cross_extent_units = units;
        let options = self.inner.options().clone();
        self.inner = virtualizer::Virtualizer::new(options);
        true
    }

    pub fn height_at(&self, index: usize) -> Px {
        match self.mode {
            VirtualListMeasureMode::Measured => self
                .inner
                .item_size(index)
                .map(units_u32_to_px)
                .unwrap_or(Px(0.0)),
            VirtualListMeasureMode::Fixed => {
                if index >= self.fixed.count {
                    return Px(0.0);
                }
                units_u32_to_px(self.fixed.estimate_units)
            }
        }
    }

    pub fn offset_for_index(&self, index: usize) -> Px {
        match self.mode {
            VirtualListMeasureMode::Measured => {
                if index >= self.inner.options().count {
                    return self.total_height();
                }
                self.inner
                    .item_start(index)
                    .map(units_u64_to_px)
                    .unwrap_or(Px(0.0))
            }
            VirtualListMeasureMode::Fixed => {
                if index >= self.fixed.count {
                    return self.total_height();
                }

                let stride =
                    (self.fixed.estimate_units as u64).saturating_add(self.fixed.gap_units as u64);
                let start_units = (self.fixed.padding_start_units as u64)
                    .saturating_add((index as u64).saturating_mul(stride));
                units_u64_to_px(start_units)
            }
        }
    }

    pub fn end_for_index(&self, index: usize) -> Px {
        match self.mode {
            VirtualListMeasureMode::Measured => {
                if index >= self.inner.options().count {
                    return self.total_height();
                }
                self.inner
                    .item_end(index)
                    .map(units_u64_to_px)
                    .unwrap_or(Px(0.0))
            }
            VirtualListMeasureMode::Fixed => {
                if index >= self.fixed.count {
                    return self.total_height();
                }
                let start = px_to_units_u64(self.offset_for_index(index));
                let end = start.saturating_add(self.fixed.estimate_units as u64);
                units_u64_to_px(end)
            }
        }
    }

    pub fn index_for_offset(&self, offset: Px) -> usize {
        match self.mode {
            VirtualListMeasureMode::Measured => {
                if self.inner.options().count == 0 {
                    return 0;
                }
                if offset.0 >= self.total_height().0 {
                    return self.inner.options().count;
                }
                self.inner
                    .index_at_offset(px_to_units_u64(offset))
                    .unwrap_or(0)
            }
            VirtualListMeasureMode::Fixed => {
                let count = self.fixed.count;
                if count == 0 {
                    return 0;
                }
                if offset.0 >= self.total_height().0 {
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
                (idx as usize).min(count)
            }
        }
    }

    pub fn end_index_for_offset(&self, offset: Px) -> usize {
        match self.mode {
            VirtualListMeasureMode::Measured => {
                if self.inner.options().count == 0 {
                    return 0;
                }
                let idx = self.index_for_offset(offset);
                if idx >= self.inner.options().count {
                    return self.inner.options().count;
                }
                let start = self.offset_for_index(idx).0;
                if start < offset.0 {
                    idx.saturating_add(1).min(self.inner.options().count)
                } else {
                    idx
                }
            }
            VirtualListMeasureMode::Fixed => {
                let count = self.fixed.count;
                if count == 0 {
                    return 0;
                }
                let idx = self.index_for_offset(offset);
                if idx >= count {
                    return count;
                }
                let start = self.offset_for_index(idx).0;
                if start < offset.0 {
                    idx.saturating_add(1).min(count)
                } else {
                    idx
                }
            }
        }
    }

    pub fn set_measured_height(&mut self, index: usize, height: Px) -> bool {
        if self.mode == VirtualListMeasureMode::Fixed {
            return false;
        }

        let Some(old_units) = self.inner.item_size(index) else {
            return false;
        };

        let height = Px(height.0.max(0.0));
        let height_units = px_to_units_u32(height);
        let changed = old_units != height_units;
        if !changed && self.inner.is_measured(index) {
            return false;
        }

        self.inner.measure_unadjusted(index, height_units);
        true
    }

    pub fn clamp_offset(&self, mut offset_y: Px, viewport_h: Px) -> Px {
        let viewport_h = Px(viewport_h.0.max(0.0));
        let total = px_to_units_u64(self.total_height());
        let max_offset_units = total.saturating_sub(px_to_units_u64(viewport_h));
        let max_offset = units_u64_to_px(max_offset_units);
        offset_y = Px(offset_y.0.max(0.0));
        Px(offset_y.0.min(max_offset.0))
    }

    /// Computes the visible item range for a vertical viewport.
    ///
    /// `offset_y` is the current scroll offset, clamped by the caller as needed.
    ///
    /// Returns a [`VirtualRange`] with **inclusive** indices (`start_index..=end_index`).
    pub fn visible_range(
        &self,
        offset_y: Px,
        viewport_h: Px,
        overscan: usize,
    ) -> Option<VirtualRange> {
        let viewport_h = Px(viewport_h.0.max(0.0));
        let count = match self.mode {
            VirtualListMeasureMode::Measured => self.inner.options().count,
            VirtualListMeasureMode::Fixed => self.fixed.count,
        };
        if viewport_h.0 <= 0.0 || count == 0 {
            return None;
        }

        let start = self.index_for_offset(offset_y);
        if start >= count {
            return None;
        }
        let end_exclusive = self.end_index_for_offset(Px(offset_y.0 + viewport_h.0));
        let end = end_exclusive.saturating_sub(1).min(count.saturating_sub(1));

        Some(VirtualRange {
            start_index: start,
            end_index: end,
            overscan,
            count,
        })
    }

    pub fn scroll_offset_for_item(
        &self,
        index: usize,
        viewport_h: Px,
        current_offset_y: Px,
        strategy: ScrollStrategy,
    ) -> Px {
        let viewport_h = Px(viewport_h.0.max(0.0));
        if viewport_h.0 <= 0.0 {
            return current_offset_y;
        }

        let count = match self.mode {
            VirtualListMeasureMode::Measured => self.inner.options().count,
            VirtualListMeasureMode::Fixed => self.fixed.count,
        };
        if count == 0 {
            return current_offset_y;
        }
        let index = index.min(count.saturating_sub(1));

        let item_top = self.offset_for_index(index);
        let item_bottom = self.end_for_index(index);

        let view_top = current_offset_y;
        let view_bottom = Px(current_offset_y.0 + viewport_h.0);

        match strategy {
            ScrollStrategy::Start => item_top,
            ScrollStrategy::End => Px(item_bottom.0 - viewport_h.0),
            ScrollStrategy::Center => {
                let item_center = 0.5 * (item_top.0 + item_bottom.0);
                Px(item_center - 0.5 * viewport_h.0)
            }
            ScrollStrategy::Nearest => {
                if item_top.0 < view_top.0 {
                    item_top
                } else if item_bottom.0 > view_bottom.0 {
                    Px(item_bottom.0 - viewport_h.0)
                } else {
                    current_offset_y
                }
            }
        }
    }

    pub fn rebuild_from_heights(
        &mut self,
        heights: Vec<Px>,
        measured: Vec<bool>,
        estimate: Px,
        gap: Px,
        scroll_margin: Px,
    ) {
        let len = heights.len();
        self.ensure_measured(len, estimate, gap, scroll_margin);

        let mut entries = Vec::new();
        for (index, height) in heights.into_iter().enumerate() {
            let is_measured = measured.get(index).copied().unwrap_or(false);
            if !is_measured {
                continue;
            }
            entries.push((self.inner.key_for(index), px_to_units_u32(height)));
        }
        self.inner.import_measurement_cache(entries);
    }
}

#[cfg(test)]
thread_local! {
    static VIRTUAL_LIST_ITEM_MEASURE_CALLS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn debug_record_virtual_list_item_measure() {
    VIRTUAL_LIST_ITEM_MEASURE_CALLS.with(|calls| {
        calls.set(calls.get().saturating_add(1));
    });
}

#[cfg(test)]
pub(crate) fn debug_take_virtual_list_item_measures() -> usize {
    VIRTUAL_LIST_ITEM_MEASURE_CALLS.with(|calls| {
        let value = calls.get();
        calls.set(0);
        value
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fenwick_sums_match_uniform_heights() {
        let mut metrics = VirtualListMetrics::default();
        metrics.ensure(100, Px(10.0), Px(0.0), Px(0.0));

        assert!((metrics.total_height().0 - 1000.0).abs() < 0.01);
        assert!((metrics.offset_for_index(0).0 - 0.0).abs() < 0.01);
        assert!((metrics.offset_for_index(6).0 - 60.0).abs() < 0.01);
        assert!((metrics.offset_for_index(100).0 - 1000.0).abs() < 0.01);

        assert_eq!(metrics.index_for_offset(Px(0.0)), 0);
        assert_eq!(metrics.index_for_offset(Px(0.1)), 0);
        assert_eq!(metrics.index_for_offset(Px(9.9)), 0);
        assert_eq!(metrics.index_for_offset(Px(10.0)), 1);
        assert_eq!(metrics.index_for_offset(Px(59.9)), 5);
        assert_eq!(metrics.index_for_offset(Px(60.0)), 6);
        assert_eq!(metrics.end_index_for_offset(Px(50.0)), 5);
        assert_eq!(metrics.end_index_for_offset(Px(50.1)), 6);
    }

    #[test]
    fn visible_range_is_inclusive_and_clamped() {
        let mut metrics = VirtualListMetrics::default();
        metrics.ensure(10, Px(10.0), Px(0.0), Px(0.0));

        let r0 = metrics.visible_range(Px(0.0), Px(25.0), 0).expect("range");
        assert_eq!(r0.start_index, 0);
        assert_eq!(r0.end_index, 2);
        assert_eq!(r0.count, 10);

        let r1 = metrics.visible_range(Px(50.0), Px(20.0), 0).expect("range");
        assert_eq!(r1.start_index, 5);
        assert_eq!(r1.end_index, 6);

        assert!(metrics.visible_range(Px(0.0), Px(0.0), 0).is_none());

        let mut empty = VirtualListMetrics::default();
        empty.ensure(0, Px(10.0), Px(0.0), Px(0.0));
        assert!(empty.visible_range(Px(0.0), Px(10.0), 0).is_none());
    }

    #[test]
    fn scroll_offset_for_item_matches_nearest_semantics() {
        let mut metrics = VirtualListMetrics::default();
        metrics.ensure(10, Px(10.0), Px(0.0), Px(0.0));

        // Item fully visible -> keep current offset.
        assert_eq!(
            metrics.scroll_offset_for_item(2, Px(50.0), Px(0.0), ScrollStrategy::Nearest),
            Px(0.0)
        );

        // Item above -> align to start.
        assert_eq!(
            metrics.scroll_offset_for_item(0, Px(20.0), Px(50.0), ScrollStrategy::Nearest),
            Px(0.0)
        );

        // Item below -> align to end.
        assert_eq!(
            metrics.scroll_offset_for_item(9, Px(20.0), Px(0.0), ScrollStrategy::Nearest),
            Px(80.0)
        );
    }

    #[test]
    fn fixed_mode_range_math_matches_uniform_metrics() {
        let mut metrics = VirtualListMetrics::default();
        metrics.ensure_with_mode(
            VirtualListMeasureMode::Fixed,
            100,
            Px(10.0),
            Px(0.0),
            Px(0.0),
        );

        assert!((metrics.total_height().0 - 1000.0).abs() < 0.01);
        assert!((metrics.offset_for_index(0).0 - 0.0).abs() < 0.01);
        assert!((metrics.offset_for_index(6).0 - 60.0).abs() < 0.01);
        assert!((metrics.offset_for_index(100).0 - 1000.0).abs() < 0.01);

        assert_eq!(metrics.index_for_offset(Px(0.0)), 0);
        assert_eq!(metrics.index_for_offset(Px(0.1)), 0);
        assert_eq!(metrics.index_for_offset(Px(9.9)), 0);
        assert_eq!(metrics.index_for_offset(Px(10.0)), 1);
        assert_eq!(metrics.index_for_offset(Px(59.9)), 5);
        assert_eq!(metrics.index_for_offset(Px(60.0)), 6);
        assert_eq!(metrics.end_index_for_offset(Px(50.0)), 5);
        assert_eq!(metrics.end_index_for_offset(Px(50.1)), 6);

        let r0 = metrics.visible_range(Px(0.0), Px(25.0), 0).expect("range");
        assert_eq!(r0.start_index, 0);
        assert_eq!(r0.end_index, 2);
        assert_eq!(r0.count, 100);
    }
}
