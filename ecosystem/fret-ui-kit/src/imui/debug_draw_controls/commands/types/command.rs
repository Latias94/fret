use std::sync::Arc;

use fret_core::{Color, Point, Px};

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;

mod clip;
mod linear;
mod media;
mod mesh;
mod round;

pub(in crate::imui::debug_draw_controls) use clip::DebugDrawClipCommand;
pub(in crate::imui::debug_draw_controls) use linear::DebugDrawLinearCommand;
pub(in crate::imui::debug_draw_controls) use media::DebugDrawMediaCommand;
pub(in crate::imui::debug_draw_controls) use mesh::DebugDrawMeshCommand;
pub(in crate::imui::debug_draw_controls) use round::DebugDrawRoundCommand;

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) enum DebugDrawCommand {
    Linear(DebugDrawLinearCommand),
    Mesh(DebugDrawMeshCommand),
    Round(DebugDrawRoundCommand),
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
