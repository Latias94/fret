use fret_core::Point;

use crate::ids::{LinkGroupId, SeriesId};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Action {
    HoverAt { point: Point },
    Pan { delta_px: Point },
    Zoom { center_px: Point, log2_scale: f32 },
    SetSeriesVisible { series: SeriesId, visible: bool },
    SetLinkGroup { group: Option<LinkGroupId> },
}
