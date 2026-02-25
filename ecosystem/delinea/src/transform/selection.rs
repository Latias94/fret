use std::sync::Arc;

use crate::transform::RowRange;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RowSelection {
    #[default]
    All,
    Range(RowRange),
    /// A non-contiguous selection of raw row indices.
    ///
    /// This aligns with the ECharts `DataStore` model (`_indices` + `getRawIndex`).
    Indices(Arc<[u32]>),
}

impl RowSelection {
    pub fn view_len(&self, total_len: usize) -> usize {
        match self {
            Self::All => total_len,
            Self::Range(range) => range.as_std_range(total_len).len(),
            Self::Indices(indices) => indices.len(),
        }
    }

    pub fn as_range(&self, len: usize) -> core::ops::Range<usize> {
        match self {
            Self::All => 0..len,
            Self::Range(range) => range.as_std_range(len),
            Self::Indices(indices) => {
                let mut min: Option<usize> = None;
                let mut max: Option<usize> = None;
                for &raw in indices.iter() {
                    let raw = raw as usize;
                    if raw >= len {
                        continue;
                    }
                    min = Some(min.map(|m| m.min(raw)).unwrap_or(raw));
                    max = Some(max.map(|m| m.max(raw)).unwrap_or(raw));
                }
                match (min, max) {
                    (Some(a), Some(b)) if b >= a => a..(b + 1),
                    _ => 0..0,
                }
            }
        }
    }

    pub fn get_raw_index(&self, total_len: usize, view_index: usize) -> Option<usize> {
        match self {
            Self::All => (view_index < total_len).then_some(view_index),
            Self::Range(range) => {
                let r = range.as_std_range(total_len);
                (view_index < r.len()).then_some(r.start + view_index)
            }
            Self::Indices(indices) => indices
                .get(view_index)
                .copied()
                .map(|i| i as usize)
                .filter(|&i| i < total_len),
        }
    }

    pub fn len(&self, total_len: usize) -> usize {
        match self {
            Self::All => total_len,
            Self::Range(range) => range.as_std_range(total_len).len(),
            Self::Indices(indices) => indices
                .iter()
                .filter(|&&i| (i as usize) < total_len)
                .count(),
        }
    }

    pub fn is_empty(&self, total_len: usize) -> bool {
        self.len(total_len) == 0
    }
}
