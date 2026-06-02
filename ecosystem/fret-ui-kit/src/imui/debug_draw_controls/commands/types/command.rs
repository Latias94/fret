use std::sync::Arc;

use fret_core::{Color, Point, Px, Size};

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;

mod clip;
mod linear;
mod media;
mod mesh;

pub(in crate::imui::debug_draw_controls) use clip::DebugDrawClipCommand;
pub(in crate::imui::debug_draw_controls) use linear::DebugDrawLinearCommand;
pub(in crate::imui::debug_draw_controls) use media::DebugDrawMediaCommand;
pub(in crate::imui::debug_draw_controls) use mesh::DebugDrawMeshCommand;

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) enum DebugDrawCommand {
    Linear(DebugDrawLinearCommand),
    Mesh(DebugDrawMeshCommand),
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
    Clip(DebugDrawClipCommand),
    Media(DebugDrawMediaCommand),
    Text {
        origin: Point,
        text: Arc<str>,
        color: Color,
        size: Px,
    },
}
