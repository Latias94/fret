use fret_core::{Color, Point, Px, Size};

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;

// This file owns round geometry debug-draw command payload variants.

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) enum DebugDrawRoundCommand {
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
}
