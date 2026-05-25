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
    pub(super) kind: DebugDrawCommandKind,
    pub(super) channel: Option<usize>,
    pub(super) clip_rect: Option<Rect>,
    pub(super) clip_depth: usize,
    pub(super) image: Option<ImageId>,
    pub(super) point_count: usize,
    pub(super) vertex_count: usize,
    pub(super) index_count: usize,
    pub(super) triangle_count: usize,
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

    pub(super) fn new(kind: DebugDrawCommandKind) -> Self {
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

    pub(super) fn with_channel(mut self, channel: Option<usize>) -> Self {
        self.channel = channel;
        self
    }
}

/// Aggregate source-level metadata for an IMUI debug draw list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct DebugDrawListSummary {
    command_count: usize,
    clip_push_count: usize,
    clip_pop_count: usize,
    max_clip_depth: usize,
    final_clip_depth: usize,
    image_command_count: usize,
    svg_command_count: usize,
    text_command_count: usize,
    point_count: usize,
    vertex_count: usize,
    index_count: usize,
    triangle_count: usize,
}

impl DebugDrawListSummary {
    pub(super) fn new() -> Self {
        Self {
            command_count: 0,
            clip_push_count: 0,
            clip_pop_count: 0,
            max_clip_depth: 0,
            final_clip_depth: 0,
            image_command_count: 0,
            svg_command_count: 0,
            text_command_count: 0,
            point_count: 0,
            vertex_count: 0,
            index_count: 0,
            triangle_count: 0,
        }
    }

    pub fn command_count(self) -> usize {
        self.command_count
    }

    pub fn clip_push_count(self) -> usize {
        self.clip_push_count
    }

    pub fn clip_pop_count(self) -> usize {
        self.clip_pop_count
    }

    pub fn max_clip_depth(self) -> usize {
        self.max_clip_depth
    }

    pub fn final_clip_depth(self) -> usize {
        self.final_clip_depth
    }

    pub fn image_command_count(self) -> usize {
        self.image_command_count
    }

    pub fn svg_command_count(self) -> usize {
        self.svg_command_count
    }

    pub fn text_command_count(self) -> usize {
        self.text_command_count
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

    pub(super) fn set_final_clip_depth(&mut self, final_clip_depth: usize) {
        self.final_clip_depth = final_clip_depth;
    }

    pub(super) fn include(&mut self, command: DebugDrawCommandSummary) {
        self.command_count += 1;
        self.point_count += command.point_count;
        self.vertex_count += command.vertex_count;
        self.index_count += command.index_count;
        self.triangle_count += command.triangle_count;
        self.max_clip_depth = self.max_clip_depth.max(command.clip_depth);

        match command.kind {
            DebugDrawCommandKind::PushClipRect => self.clip_push_count += 1,
            DebugDrawCommandKind::PopClipRect => self.clip_pop_count += 1,
            DebugDrawCommandKind::Image
            | DebugDrawCommandKind::ImageRegion
            | DebugDrawCommandKind::ImageQuad
            | DebugDrawCommandKind::ImageRounded
            | DebugDrawCommandKind::ImageRegionRounded
            | DebugDrawCommandKind::ImageTriangleMesh => self.image_command_count += 1,
            DebugDrawCommandKind::SvgImage | DebugDrawCommandKind::SvgMaskIcon => {
                self.svg_command_count += 1;
            }
            DebugDrawCommandKind::Text => self.text_command_count += 1,
            DebugDrawCommandKind::Line
            | DebugDrawCommandKind::Polyline
            | DebugDrawCommandKind::ConvexPolyFilled
            | DebugDrawCommandKind::ConcavePolyFilled
            | DebugDrawCommandKind::Rect
            | DebugDrawCommandKind::RectFilled
            | DebugDrawCommandKind::RectFilledMultiColor
            | DebugDrawCommandKind::Quad
            | DebugDrawCommandKind::QuadFilled
            | DebugDrawCommandKind::Triangle
            | DebugDrawCommandKind::TriangleFilled
            | DebugDrawCommandKind::TriangleMesh
            | DebugDrawCommandKind::Circle
            | DebugDrawCommandKind::CircleFilled
            | DebugDrawCommandKind::Ngon
            | DebugDrawCommandKind::NgonFilled
            | DebugDrawCommandKind::Ellipse
            | DebugDrawCommandKind::EllipseFilled
            | DebugDrawCommandKind::BezierQuadratic
            | DebugDrawCommandKind::BezierCubic => {}
        }
    }
}
