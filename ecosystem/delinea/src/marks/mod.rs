use core::ops::Range;

use fret_core::{Point, Rect};

use crate::ids::SeriesId;
use crate::ids::{LayerId, MarkId, PaintId, Revision, StringId};
use crate::paint::StrokeStyleV2;
use crate::text::TextStyleId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MarkKind {
    Group,
    Polyline,
    Rect,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MarkOrderKey(pub u32);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MarkNode {
    pub id: MarkId,
    pub parent: Option<MarkId>,
    pub layer: LayerId,
    pub order: MarkOrderKey,
    pub kind: MarkKind,
    pub source_series: Option<SeriesId>,
    pub payload: MarkPayloadRef,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MarkPayloadRef {
    Group(MarkGroup),
    Polyline(MarkPolylineRef),
    Rect(MarkRectRef),
    Text(MarkText),
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MarkGroup {
    pub clip: Option<Rect>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MarkPolylineRef {
    pub points: Range<usize>,
    pub stroke: Option<(PaintId, StrokeStyleV2)>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MarkRectRef {
    pub rects: Range<usize>,
    pub fill: Option<PaintId>,
    pub stroke: Option<(PaintId, StrokeStyleV2)>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MarkText {
    pub rect: Rect,
    pub text: StringId,
    pub style: TextStyleId,
    pub fill: Option<PaintId>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MarkArena {
    pub points: Vec<Point>,
    pub data_indices: Vec<u32>,
    pub rects: Vec<Rect>,
    pub rect_data_indices: Vec<u32>,
}

impl MarkArena {
    pub fn clear(&mut self) {
        self.points.clear();
        self.data_indices.clear();
        self.rects.clear();
        self.rect_data_indices.clear();
    }

    pub fn extend_points(&mut self, points: impl IntoIterator<Item = Point>) -> Range<usize> {
        let start = self.points.len();
        self.points.extend(points);
        let end = self.points.len();
        start..end
    }

    pub fn extend_points_with_indices(
        &mut self,
        points: impl IntoIterator<Item = Point>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Range<usize> {
        let start = self.points.len();
        self.points.extend(points);
        self.data_indices.extend(indices);
        let end = self.points.len();
        debug_assert_eq!(self.data_indices.len(), self.points.len());
        start..end
    }

    pub fn extend_rects_with_indices(
        &mut self,
        rects: impl IntoIterator<Item = Rect>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Range<usize> {
        let start = self.rects.len();
        self.rects.extend(rects);
        self.rect_data_indices.extend(indices);
        let end = self.rects.len();
        debug_assert_eq!(self.rect_data_indices.len(), self.rects.len());
        start..end
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MarkTree {
    pub revision: Revision,
    pub arena: MarkArena,
    pub nodes: Vec<MarkNode>,
}

impl MarkTree {
    pub fn clear(&mut self) {
        self.arena.clear();
        self.nodes.clear();
        self.revision.bump();
    }
}
