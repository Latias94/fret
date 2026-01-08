#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RowRange {
    pub start: usize,
    pub end: usize,
}

impl RowRange {
    pub fn clamp_to_len(&mut self, len: usize) {
        self.start = self.start.min(len);
        self.end = self.end.min(len);
        if self.end < self.start {
            self.end = self.start;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    pub fn as_std_range(&self, len: usize) -> core::ops::Range<usize> {
        let mut r = *self;
        r.clamp_to_len(len);
        r.start..r.end
    }
}
