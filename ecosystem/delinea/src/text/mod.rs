use fret_core::Px;

use crate::ids::StringId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FontId(pub u32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TextStyleId(pub u32);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TextMetrics {
    pub width: Px,
    pub height: Px,
}

pub trait TextMeasurer {
    fn measure(&mut self, text: StringId, style: TextStyleId) -> TextMetrics;
}
