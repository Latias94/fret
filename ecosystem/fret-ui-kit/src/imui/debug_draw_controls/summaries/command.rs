use fret_core::{ImageId, Rect};

/// Public command classes exposed by the IMUI debug draw list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DebugDrawCommandKind {
    Line,
    Polyline,
    ConvexPolyFilled,
    ConcavePolyFilled,
    Rect,
    RectFilled,
    RectFilledMultiColor,
    Quad,
    QuadFilled,
    Triangle,
    TriangleFilled,
    TriangleMesh,
    ImageTriangleMesh,
    Circle,
    CircleFilled,
    Ngon,
    NgonFilled,
    Ellipse,
    EllipseFilled,
    BezierQuadratic,
    BezierCubic,
    PushClipRect,
    PopClipRect,
    Image,
    ImageRegion,
    ImageQuad,
    ImageRounded,
    ImageRegionRounded,
    SvgImage,
    SvgMaskIcon,
    Text,
}

/// Stable metadata for one recorded debug draw command.
///
/// Counts describe source-level debug draw payloads, not guaranteed backend draw calls.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct DebugDrawCommandSummary {
    pub(in crate::imui::debug_draw_controls) kind: DebugDrawCommandKind,
    pub(in crate::imui::debug_draw_controls) channel: Option<usize>,
    pub(in crate::imui::debug_draw_controls) clip_rect: Option<Rect>,
    pub(in crate::imui::debug_draw_controls) clip_depth: usize,
    pub(in crate::imui::debug_draw_controls) image: Option<ImageId>,
    pub(in crate::imui::debug_draw_controls) point_count: usize,
    pub(in crate::imui::debug_draw_controls) vertex_count: usize,
    pub(in crate::imui::debug_draw_controls) index_count: usize,
    pub(in crate::imui::debug_draw_controls) triangle_count: usize,
}

impl DebugDrawCommandSummary {
    pub fn kind(self) -> DebugDrawCommandKind {
        self.kind
    }

    pub fn channel(self) -> Option<usize> {
        self.channel
    }

    pub fn clip_rect(self) -> Option<Rect> {
        self.clip_rect
    }

    pub fn clip_depth(self) -> usize {
        self.clip_depth
    }

    pub fn image(self) -> Option<ImageId> {
        self.image
    }

    pub fn point_count(self) -> usize {
        self.point_count
    }

    pub fn vertex_count(self) -> usize {
        self.vertex_count
    }

    pub fn index_count(self) -> usize {
        self.index_count
    }

    pub fn triangle_count(self) -> usize {
        self.triangle_count
    }

    pub(in crate::imui::debug_draw_controls) fn new(kind: DebugDrawCommandKind) -> Self {
        Self {
            kind,
            channel: None,
            clip_rect: None,
            clip_depth: 0,
            image: None,
            point_count: 0,
            vertex_count: 0,
            index_count: 0,
            triangle_count: 0,
        }
    }

    pub(in crate::imui::debug_draw_controls) fn with_channel(
        mut self,
        channel: Option<usize>,
    ) -> Self {
        self.channel = channel;
        self
    }
}
