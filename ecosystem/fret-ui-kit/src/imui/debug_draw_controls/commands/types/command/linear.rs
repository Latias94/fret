use std::sync::Arc;

use fret_core::{Color, Point, Rect};

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;

// This file owns linear geometry debug-draw command payload variants.

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) enum DebugDrawLinearCommand {
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
}
