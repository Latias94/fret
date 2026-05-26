use std::sync::Arc;

use fret_core::{Color, Point, Px, Rect};

use super::super::commands::DebugDrawCommand;
use super::super::{DebugDrawStrokeStyle, ImUiDebugDrawList};

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
        self.commands.push(DebugDrawCommand::Line {
            from,
            to,
            color,
            style: style.into(),
        });
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
        self.commands.push(DebugDrawCommand::Polyline {
            points,
            color,
            style: style.into(),
            closed,
        });
    }

    pub fn add_convex_poly_filled<I>(&mut self, points: I, color: Color)
    where
        I: IntoIterator<Item = Point>,
    {
        let points: Arc<[Point]> = Arc::from(points.into_iter().collect::<Vec<_>>());
        self.commands
            .push(DebugDrawCommand::ConvexPolyFilled { points, color });
    }

    pub fn add_concave_poly_filled<I>(&mut self, points: I, color: Color)
    where
        I: IntoIterator<Item = Point>,
    {
        let points: Arc<[Point]> = Arc::from(points.into_iter().collect::<Vec<_>>());
        self.commands
            .push(DebugDrawCommand::ConcavePolyFilled { points, color });
    }

    pub fn add_rect(&mut self, rect: Rect, color: Color, thickness: Px) {
        self.add_rect_with_style(rect, color, thickness);
    }

    pub fn add_rect_with_style(
        &mut self,
        rect: Rect,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Rect {
            rect,
            color,
            style: style.into(),
        });
    }

    pub fn add_rect_filled(&mut self, rect: Rect, color: Color) {
        self.commands
            .push(DebugDrawCommand::RectFilled { rect, color });
    }

    pub fn add_rect_filled_multi_color(
        &mut self,
        rect: Rect,
        upper_left: Color,
        upper_right: Color,
        bottom_right: Color,
        bottom_left: Color,
    ) {
        self.commands.push(DebugDrawCommand::RectFilledMultiColor {
            rect,
            upper_left,
            upper_right,
            bottom_right,
            bottom_left,
        });
    }

    pub fn add_quad(
        &mut self,
        p1: Point,
        p2: Point,
        p3: Point,
        p4: Point,
        color: Color,
        thickness: Px,
    ) {
        self.add_quad_with_style(p1, p2, p3, p4, color, thickness);
    }

    pub fn add_quad_with_style(
        &mut self,
        p1: Point,
        p2: Point,
        p3: Point,
        p4: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Quad {
            p1,
            p2,
            p3,
            p4,
            color,
            style: style.into(),
        });
    }

    pub fn add_quad_filled(&mut self, p1: Point, p2: Point, p3: Point, p4: Point, color: Color) {
        self.commands.push(DebugDrawCommand::QuadFilled {
            p1,
            p2,
            p3,
            p4,
            color,
        });
    }

    pub fn add_triangle(&mut self, p1: Point, p2: Point, p3: Point, color: Color, thickness: Px) {
        self.add_triangle_with_style(p1, p2, p3, color, thickness);
    }

    pub fn add_triangle_with_style(
        &mut self,
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
    ) {
        self.commands.push(DebugDrawCommand::Triangle {
            p1,
            p2,
            p3,
            color,
            style: style.into(),
        });
    }

    pub fn add_triangle_filled(&mut self, p1: Point, p2: Point, p3: Point, color: Color) {
        self.commands
            .push(DebugDrawCommand::TriangleFilled { p1, p2, p3, color });
    }
}
