use fret_core::Point;

use crate::ids::{LinkGroupId, SeriesId};
use crate::selection::BrushSelection2D;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LinkEvent {
    CursorMoved { point: Point },
    DomainWindowChanged { series: Option<SeriesId> },
    BrushSelectionChanged { selection: Option<BrushSelection2D> },
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LinkConfig {
    pub group: Option<LinkGroupId>,
}
