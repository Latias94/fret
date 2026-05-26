use fret_core::{Color, Point, Px};

use super::DebugDrawStrokeStyle;
use super::ImUiDebugDrawList;
use super::paths::path_stroke_required_points;

mod shape_methods;

#[derive(Debug)]
pub struct ImUiDebugDrawPath<'a> {
    pub(super) draw_list: &'a mut ImUiDebugDrawList,
    pub(super) points: Vec<Point>,
}

impl<'a> ImUiDebugDrawPath<'a> {
    pub(super) fn new(draw_list: &'a mut ImUiDebugDrawList) -> Self {
        Self {
            draw_list,
            points: Vec::new(),
        }
    }

    pub fn clear(&mut self) -> &mut Self {
        self.points.clear();
        self
    }

    pub fn line_to(&mut self, point: Point) -> &mut Self {
        self.points.push(point);
        self
    }

    pub fn line_to_merge_duplicate(&mut self, point: Point) -> &mut Self {
        if self.points.last().copied() != Some(point) {
            self.points.push(point);
        }
        self
    }

    pub fn stroke(&mut self, color: Color, thickness: Px, closed: bool) {
        self.stroke_with_style(color, thickness, closed);
    }

    pub fn stroke_with_style(
        &mut self,
        color: Color,
        style: impl Into<DebugDrawStrokeStyle>,
        closed: bool,
    ) {
        let points = std::mem::take(&mut self.points);
        if points.len() < path_stroke_required_points(closed) {
            return;
        }
        self.draw_list
            .add_polyline_with_style(points, color, style, closed);
    }

    pub fn fill_convex(&mut self, color: Color) {
        let points = std::mem::take(&mut self.points);
        if points.len() < 3 {
            return;
        }
        self.draw_list.add_convex_poly_filled(points, color);
    }

    pub fn fill_concave(&mut self, color: Color) {
        let points = std::mem::take(&mut self.points);
        if points.len() < 3 {
            return;
        }
        self.draw_list.add_concave_poly_filled(points, color);
    }

    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}
