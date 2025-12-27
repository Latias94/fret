use fret_core::Px;

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

#[derive(Debug, Default, Clone)]
pub(crate) struct VirtualListMetrics {
    estimate: Px,
    gap: Px,
    scroll_margin: Px,
    heights: Vec<Px>,
    measured: Vec<bool>,
    fenwick: Fenwick,
}

impl VirtualListMetrics {
    pub fn ensure(&mut self, len: usize, estimate: Px, gap: Px, scroll_margin: Px) {
        let estimate = Px(estimate.0.max(0.0));
        let gap = Px(gap.0.max(0.0));
        let scroll_margin = Px(scroll_margin.0.max(0.0));
        if self.heights.len() == len
            && self.estimate == estimate
            && self.gap == gap
            && self.scroll_margin == scroll_margin
        {
            return;
        }

        let mut heights = Vec::with_capacity(len);
        let mut measured = Vec::with_capacity(len);
        for i in 0..len {
            if i < self.heights.len() && self.measured.get(i).copied().unwrap_or(false) {
                heights.push(self.heights[i]);
                measured.push(true);
            } else {
                heights.push(estimate);
                measured.push(false);
            }
        }

        self.estimate = estimate;
        self.gap = gap;
        self.scroll_margin = scroll_margin;
        self.heights = heights;
        self.measured = measured;
        self.fenwick = Fenwick::from_values(&self.step_values());
    }

    pub fn total_height(&self) -> Px {
        if self.heights.is_empty() {
            return self.scroll_margin;
        }
        let sum = self.fenwick.sum(self.heights.len());
        Px((self.scroll_margin.0 + (sum.0 - self.gap.0).max(0.0)).max(0.0))
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

    pub fn height_at(&self, index: usize) -> Px {
        self.heights.get(index).copied().unwrap_or(Px(0.0))
    }

    pub fn offset_for_index(&self, index: usize) -> Px {
        Px((self.scroll_margin.0 + self.fenwick.sum(index).0).max(0.0))
    }

    pub fn end_for_index(&self, index: usize) -> Px {
        let start = self.offset_for_index(index);
        let h = self.height_at(index);
        Px((start.0 + h.0).max(0.0))
    }

    pub fn index_for_offset(&self, offset: Px) -> usize {
        if self.heights.is_empty() {
            return 0;
        }
        let local = Px((offset.0 - self.scroll_margin.0).max(0.0));
        self.fenwick.lower_bound(local.0.max(0.0))
    }

    pub fn end_index_for_offset(&self, offset: Px) -> usize {
        if self.heights.is_empty() {
            return 0;
        }
        let idx = self.index_for_offset(offset);
        if idx >= self.heights.len() {
            return self.heights.len();
        }
        let start = self.offset_for_index(idx).0;
        if start < offset.0 {
            idx.saturating_add(1).min(self.heights.len())
        } else {
            idx
        }
    }

    pub fn set_measured_height(&mut self, index: usize, height: Px) -> bool {
        let Some(old) = self.heights.get(index).copied() else {
            return false;
        };

        let height = Px(height.0.max(0.0));
        let changed = (old.0 - height.0).abs() > 0.01;
        if !changed && self.measured.get(index).copied().unwrap_or(false) {
            return false;
        }

        let delta = Px(height.0 - old.0);
        self.heights[index] = height;
        if let Some(measured) = self.measured.get_mut(index) {
            *measured = true;
        }
        self.fenwick.add(index, delta);
        true
    }

    pub fn clamp_offset(&self, mut offset_y: Px, viewport_h: Px) -> Px {
        let viewport_h = Px(viewport_h.0.max(0.0));
        let content_h = self.total_height();
        let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
        offset_y = Px(offset_y.0.max(0.0));
        Px(offset_y.0.min(max_offset.0))
    }

    pub fn rebuild_from_heights(
        &mut self,
        heights: Vec<Px>,
        measured: Vec<bool>,
        estimate: Px,
        gap: Px,
        scroll_margin: Px,
    ) {
        let estimate = Px(estimate.0.max(0.0));
        let gap = Px(gap.0.max(0.0));
        let scroll_margin = Px(scroll_margin.0.max(0.0));

        self.estimate = estimate;
        self.gap = gap;
        self.scroll_margin = scroll_margin;
        self.heights = heights;
        self.measured = measured;
        self.fenwick = Fenwick::from_values(&self.step_values());
    }

    fn step_values(&self) -> Vec<Px> {
        let mut values = Vec::with_capacity(self.heights.len());
        for h in &self.heights {
            values.push(Px((h.0 + self.gap.0).max(0.0)));
        }
        values
    }
}

#[derive(Debug, Default, Clone)]
struct Fenwick {
    tree: Vec<f32>, // 1-based
}

impl Fenwick {
    fn from_values(values: &[Px]) -> Self {
        let mut this = Self {
            tree: vec![0.0; values.len() + 1],
        };
        for (i, v) in values.iter().enumerate() {
            this.add(i, *v);
        }
        this
    }

    fn add(&mut self, index: usize, delta: Px) {
        let mut i = index + 1;
        while i < self.tree.len() {
            self.tree[i] += delta.0;
            i += i & (!i + 1);
        }
    }

    fn sum(&self, index: usize) -> Px {
        let mut acc = 0.0;
        let mut i = index.min(self.tree.len().saturating_sub(1));
        while i > 0 {
            acc += self.tree[i];
            i &= i - 1;
        }
        Px(acc)
    }

    fn lower_bound(&self, target: f32) -> usize {
        if self.tree.len() <= 1 {
            return 0;
        }

        let target = target.max(0.0);
        let mut idx = 0usize;
        let mut bit = 1usize;
        while bit < self.tree.len() {
            bit <<= 1;
        }
        bit >>= 1;

        let mut remaining = target;
        while bit != 0 {
            let next = idx + bit;
            if next < self.tree.len() && self.tree[next] <= remaining {
                remaining -= self.tree[next];
                idx = next;
            }
            bit >>= 1;
        }

        idx
    }
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
}
