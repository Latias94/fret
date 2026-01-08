use crate::view::RowRange;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RowSelection {
    #[default]
    All,
    Range(RowRange),
}

impl RowSelection {
    pub fn as_range(self, len: usize) -> core::ops::Range<usize> {
        match self {
            Self::All => 0..len,
            Self::Range(mut range) => {
                range.clamp_to_len(len);
                range.start..range.end
            }
        }
    }
}
