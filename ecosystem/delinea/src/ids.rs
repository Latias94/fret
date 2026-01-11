use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($name), "({})"), self.0)
            }
        }
    };
}

define_id!(ChartId);
define_id!(SeriesId);
define_id!(ComponentId);
define_id!(AxisId);
define_id!(GridId);
define_id!(DatasetId);
define_id!(FieldId);
define_id!(TransformId);
define_id!(StackId);
define_id!(LinkGroupId);
define_id!(DataZoomId);

define_id!(LayerId);
define_id!(MarkId);
define_id!(PaintId);
define_id!(FormatterId);

define_id!(StringId);

pub const MARK_VARIANT_BITS: u32 = 16;
pub const MARK_VARIANT_MASK: u64 = (1u64 << MARK_VARIANT_BITS) - 1;

pub fn series_mark_id(series: SeriesId, variant: u64) -> MarkId {
    MarkId((series.0 << MARK_VARIANT_BITS) | (variant & MARK_VARIANT_MASK))
}

pub fn mark_variant(id: MarkId) -> u64 {
    id.0 & MARK_VARIANT_MASK
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Revision(pub u64);

impl Revision {
    pub fn bump(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}
