use std::sync::Arc;

use fret_core::{Color, ImageId, Point, Px, Rect, Size, UvPoint, UvRect};
use fret_ui::SvgSource;

use super::{
    DebugDrawImageMeshOptions, DebugDrawImageOptions, DebugDrawImageQuadOptions,
    DebugDrawRoundCorners, DebugDrawStrokeStyle, DebugDrawSvgOptions, DebugDrawVertex,
};

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
    kind: DebugDrawCommandKind,
    channel: Option<usize>,
    clip_rect: Option<Rect>,
    clip_depth: usize,
    image: Option<ImageId>,
    point_count: usize,
    vertex_count: usize,
    index_count: usize,
    triangle_count: usize,
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

    fn new(kind: DebugDrawCommandKind) -> Self {
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

    fn with_channel(mut self, channel: Option<usize>) -> Self {
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

#[derive(Debug, Clone)]
pub(super) enum DebugDrawCommand {
    Line {
        from: Point,
        to: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    Polyline {
        points: Arc<[Point]>,
        color: Color,
        style: DebugDrawStrokeStyle,
        closed: bool,
    },
    ConvexPolyFilled {
        points: Arc<[Point]>,
        color: Color,
    },
    ConcavePolyFilled {
        points: Arc<[Point]>,
        color: Color,
    },
    Rect {
        rect: Rect,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    RectFilled {
        rect: Rect,
        color: Color,
    },
    RectFilledMultiColor {
        rect: Rect,
        upper_left: Color,
        upper_right: Color,
        bottom_right: Color,
        bottom_left: Color,
    },
    Quad {
        p1: Point,
        p2: Point,
        p3: Point,
        p4: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    QuadFilled {
        p1: Point,
        p2: Point,
        p3: Point,
        p4: Point,
        color: Color,
    },
    Triangle {
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    TriangleFilled {
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
    },
    TriangleMesh {
        vertices: Arc<[DebugDrawVertex]>,
        indices: Arc<[u32]>,
    },
    ImageTriangleMesh {
        image: ImageId,
        vertices: Arc<[DebugDrawVertex]>,
        indices: Arc<[u32]>,
        options: DebugDrawImageMeshOptions,
    },
    Circle {
        center: Point,
        radius: Px,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    CircleFilled {
        center: Point,
        radius: Px,
        color: Color,
    },
    Ngon {
        center: Point,
        radius: Px,
        segments: usize,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    NgonFilled {
        center: Point,
        radius: Px,
        segments: usize,
        color: Color,
    },
    Ellipse {
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    EllipseFilled {
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
    },
    BezierQuadratic {
        from: Point,
        ctrl: Point,
        to: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    BezierCubic {
        from: Point,
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
        color: Color,
        style: DebugDrawStrokeStyle,
    },
    PushClipRect {
        rect: Rect,
    },
    PopClipRect,
    Image {
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
    },
    ImageRegion {
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
    },
    ImageQuad {
        image: ImageId,
        points: [Point; 4],
        uvs: [UvPoint; 4],
        options: DebugDrawImageQuadOptions,
    },
    ImageRounded {
        rect: Rect,
        image: ImageId,
        options: DebugDrawImageOptions,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    },
    ImageRegionRounded {
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        options: DebugDrawImageOptions,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    },
    SvgImage {
        rect: Rect,
        svg: SvgSource,
        options: DebugDrawSvgOptions,
    },
    SvgMaskIcon {
        rect: Rect,
        svg: SvgSource,
        color: Color,
        options: DebugDrawSvgOptions,
    },
    Text {
        origin: Point,
        text: Arc<str>,
        color: Color,
        size: Px,
    },
}

impl DebugDrawCommand {
    pub(super) fn summary_with_clip_state(
        &self,
        channel: Option<usize>,
        clip_stack: &mut Vec<Rect>,
    ) -> DebugDrawCommandSummary {
        let mut summary = self.summary().with_channel(channel);
        match self {
            DebugDrawCommand::PushClipRect { rect } => {
                clip_stack.push(*rect);
                summary.clip_rect = Some(*rect);
            }
            DebugDrawCommand::PopClipRect => {
                clip_stack.pop();
                summary.clip_rect = clip_stack.last().copied();
            }
            _ => {
                summary.clip_rect = clip_stack.last().copied();
            }
        }
        summary.clip_depth = clip_stack.len();
        summary
    }

    fn summary(&self) -> DebugDrawCommandSummary {
        match self {
            DebugDrawCommand::Line { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Line);
                summary.point_count = 2;
                summary
            }
            DebugDrawCommand::Polyline { points, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Polyline);
                summary.point_count = points.len();
                summary
            }
            DebugDrawCommand::ConvexPolyFilled { points, .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::ConvexPolyFilled);
                summary.point_count = points.len();
                summary
            }
            DebugDrawCommand::ConcavePolyFilled { points, .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::ConcavePolyFilled);
                summary.point_count = points.len();
                summary
            }
            DebugDrawCommand::Rect { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Rect);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::RectFilled { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilled);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::RectFilledMultiColor { .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilledMultiColor);
                summary.point_count = 4;
                summary.vertex_count = 4;
                summary.index_count = 6;
                summary.triangle_count = 2;
                summary
            }
            DebugDrawCommand::Quad { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Quad);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::QuadFilled { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::QuadFilled);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::Triangle { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Triangle);
                summary.point_count = 3;
                summary
            }
            DebugDrawCommand::TriangleFilled { .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleFilled);
                summary.point_count = 3;
                summary.triangle_count = 1;
                summary
            }
            DebugDrawCommand::TriangleMesh { vertices, indices } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleMesh);
                summary.vertex_count = vertices.len();
                summary.index_count = indices.len();
                summary.triangle_count = indices.len() / 3;
                summary
            }
            DebugDrawCommand::ImageTriangleMesh {
                image,
                vertices,
                indices,
                ..
            } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::ImageTriangleMesh);
                summary.image = Some(*image);
                summary.vertex_count = vertices.len();
                summary.index_count = indices.len();
                summary.triangle_count = indices.len() / 3;
                summary
            }
            DebugDrawCommand::Circle { .. } => {
                DebugDrawCommandSummary::new(DebugDrawCommandKind::Circle)
            }
            DebugDrawCommand::CircleFilled { .. } => {
                DebugDrawCommandSummary::new(DebugDrawCommandKind::CircleFilled)
            }
            DebugDrawCommand::Ngon { segments, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Ngon);
                summary.point_count = *segments;
                summary
            }
            DebugDrawCommand::NgonFilled { segments, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::NgonFilled);
                summary.point_count = *segments;
                summary
            }
            DebugDrawCommand::Ellipse { segments, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Ellipse);
                summary.point_count = *segments;
                summary
            }
            DebugDrawCommand::EllipseFilled { segments, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::EllipseFilled);
                summary.point_count = *segments;
                summary
            }
            DebugDrawCommand::BezierQuadratic { .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::BezierQuadratic);
                summary.point_count = 3;
                summary
            }
            DebugDrawCommand::BezierCubic { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::BezierCubic);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::PushClipRect { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::PushClipRect);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::PopClipRect => {
                DebugDrawCommandSummary::new(DebugDrawCommandKind::PopClipRect)
            }
            DebugDrawCommand::Image { image, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Image);
                summary.image = Some(*image);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::ImageRegion { image, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ImageRegion);
                summary.image = Some(*image);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::ImageQuad { image, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ImageQuad);
                summary.image = Some(*image);
                summary.point_count = 4;
                summary.vertex_count = 4;
                summary.index_count = 6;
                summary.triangle_count = 2;
                summary
            }
            DebugDrawCommand::ImageRounded { image, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ImageRounded);
                summary.image = Some(*image);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::ImageRegionRounded { image, .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::ImageRegionRounded);
                summary.image = Some(*image);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::SvgImage { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::SvgImage);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::SvgMaskIcon { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::SvgMaskIcon);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::Text { .. } => {
                DebugDrawCommandSummary::new(DebugDrawCommandKind::Text)
            }
        }
    }
}
