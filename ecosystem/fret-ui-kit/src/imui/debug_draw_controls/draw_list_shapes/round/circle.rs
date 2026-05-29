use fret_core::{Color, Point, Px};

use super::super::super::commands::DebugDrawCommand;
use super::super::super::{DebugDrawStrokeStyle, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_circle(&mut self, center: Point, radius: Px, color: Color, thickness: Px) {
        self.add_circle_with_style(center, radius, color, thickness);
    }

    pub fn add_circle_with_style(
        &mut self,
        center: Point,
        radius: Px,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Circle {
            center,
            radius,
            color,
            style: style.into(),
        });
    }

    pub fn add_circle_filled(&mut self, center: Point, radius: Px, color: Color) {
        self.commands.push(DebugDrawCommand::CircleFilled {
            center,
            radius,
            color,
        });
    }
}
