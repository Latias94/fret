use std::sync::Arc;

use fret_core::{Color, ImageId, Point, Px, Rect, Size, UvPoint, UvRect};
use fret_ui::SvgSource;

use super::super::{
    DebugDrawImageMeshOptions, DebugDrawImageOptions, DebugDrawImageQuadOptions,
    DebugDrawRoundCorners, DebugDrawStrokeStyle, DebugDrawSvgOptions, DebugDrawVertex,
};

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) enum DebugDrawCommand {
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
