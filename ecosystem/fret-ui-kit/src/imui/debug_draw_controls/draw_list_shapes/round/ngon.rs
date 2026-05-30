use fret_core::{Color, Point, Px};

use super::super::super::commands::DebugDrawCommand;
use super::super::super::{DebugDrawStrokeStyle, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_ngon(
        &mut self,
        center: Point,
        radius: Px,
        segments: usize,
        color: Color,
        thickness: Px,
    ) {
        self.add_ngon_with_style(center, radius, segments, color, thickness);
    }

    pub fn add_ngon_with_style(
        &mut self,
        center: Point,
        radius: Px,
        segments: usize,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Ngon {
            center,
            radius,
            segments,
            color,
            style: style.into(),
        });
    }

    pub fn add_ngon_filled(&mut self, center: Point, radius: Px, segments: usize, color: Color) {
        self.commands.push(DebugDrawCommand::NgonFilled {
            center,
            radius,
            segments,
            color,
        });
    }
}
