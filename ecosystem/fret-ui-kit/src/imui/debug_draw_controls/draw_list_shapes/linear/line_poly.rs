use std::sync::Arc;

use fret_core::{Color, Point, Px};

use super::super::super::commands::{DebugDrawCommand, DebugDrawLinearCommand};
use super::super::super::{DebugDrawStrokeStyle, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn add_line(&mut self, from: Point, to: Point, color: Color, thickness: Px) {
        self.add_line_with_style(from, to, color, thickness);
    }

    pub fn add_line_with_style(
        &mut self,
        from: Point,
        to: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands
            .push(DebugDrawCommand::Linear(DebugDrawLinearCommand::Line {
                from,
                to,
                color,
                style: style.into(),
            }));
    }

    pub fn add_polyline<I>(&mut self, points: I, color: Color, thickness: Px, closed: bool)
    where
        I: IntoIterator<Item = Point>,
    {
        self.add_polyline_with_style(points, color, thickness, closed);
    }

    pub fn add_polyline_with_style<I>(
        &mut self,
        points: I,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
        closed: bool,
    ) where
        I: IntoIterator<Item = Point>,
    {
        let points: Arc<[Point]> = Arc::from(points.into_iter().collect::<Vec<_>>());
        self.commands
            .push(DebugDrawCommand::Linear(DebugDrawLinearCommand::Polyline {
                points,
                color,
                style: style.into(),
                closed,
            }));
    }

    pub fn add_convex_poly_filled<I>(&mut self, points: I, color: Color)
    where
        I: IntoIterator<Item = Point>,
    {
        let points: Arc<[Point]> = Arc::from(points.into_iter().collect::<Vec<_>>());
        self.commands.push(DebugDrawCommand::Linear(
            DebugDrawLinearCommand::ConvexPolyFilled { points, color },
        ));
    }

    pub fn add_concave_poly_filled<I>(&mut self, points: I, color: Color)
    where
        I: IntoIterator<Item = Point>,
    {
        let points: Arc<[Point]> = Arc::from(points.into_iter().collect::<Vec<_>>());
        self.commands.push(DebugDrawCommand::Linear(
            DebugDrawLinearCommand::ConcavePolyFilled { points, color },
        ));
    }
}
