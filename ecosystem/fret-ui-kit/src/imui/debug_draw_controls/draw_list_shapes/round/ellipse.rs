use fret_core::{Color, Point, Px, Size};

use super::super::super::commands::{DebugDrawCommand, DebugDrawRoundCommand};
use super::super::super::{DebugDrawStrokeStyle, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_ellipse(
        &mut self,
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
        thickness: Px,
    ) {
        self.add_ellipse_with_style(center, radius, rotation_radians, segments, color, thickness);
    }

    pub fn add_ellipse_with_style(
        &mut self,
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands
            .push(DebugDrawCommand::Round(DebugDrawRoundCommand::Ellipse {
                center,
                radius,
                rotation_radians,
                segments,
                color,
                style: style.into(),
            }));
    }

    pub fn add_ellipse_filled(
        &mut self,
        center: Point,
        radius: Size,
        rotation_radians: f32,
        segments: usize,
        color: Color,
    ) {
        self.commands.push(DebugDrawCommand::Round(
            DebugDrawRoundCommand::EllipseFilled {
                center,
                radius,
                rotation_radians,
                segments,
                color,
            },
        ));
    }
}
