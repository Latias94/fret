use std::sync::Arc;

use fret_core::{Color, ImageId, Point, Px, Rect, Size};

use crate::imui::debug_draw_controls::{
    DebugDrawImageMeshOptions, DebugDrawStrokeStyle, DebugDrawVertex,
};

mod media;

pub(in crate::imui::debug_draw_controls) use media::DebugDrawMediaCommand;

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
    Media(DebugDrawMediaCommand),
    Text {
        origin: Point,
        text: Arc<str>,
        color: Color,
        size: Px,
    },
}
