use fret_core::{Color, Point, Px};

use super::super::commands::DebugDrawCommand;
use super::super::{DebugDrawStrokeStyle, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_bezier_quadratic(
        &mut self,
        from: Point,
        ctrl: Point,
        to: Point,
        color: Color,
        thickness: Px,
    ) {
        self.add_bezier_quadratic_with_style(from, ctrl, to, color, thickness);
    }

    pub fn add_bezier_quadratic_with_style(
        &mut self,
        from: Point,
        ctrl: Point,
        to: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::BezierQuadratic {
            from,
            ctrl,
            to,
            color,
            style: style.into(),
        });
    }

    pub fn add_bezier_cubic(
        &mut self,
        from: Point,
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
        color: Color,
        thickness: Px,
    ) {
        self.add_bezier_cubic_with_style(from, ctrl1, ctrl2, to, color, thickness);
    }

    pub fn add_bezier_cubic_with_style(
        &mut self,
        from: Point,
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::BezierCubic {
            from,
            ctrl1,
            ctrl2,
            to,
            color,
            style: style.into(),
        });
    }
}
