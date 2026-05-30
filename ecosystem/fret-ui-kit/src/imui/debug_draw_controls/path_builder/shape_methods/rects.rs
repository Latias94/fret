use fret_core::{Px, Rect};

use super::super::ImUiDebugDrawPath;
use crate::imui::debug_draw_controls::DebugDrawRoundCorners;
use crate::imui::debug_draw_controls::geometry::{rect_is_empty, rect_is_finite};
use crate::imui::debug_draw_controls::paths::append_path_rect_points;

impl<'a> ImUiDebugDrawPath<'a> {
    pub fn rect(&mut self, rect: Rect) -> &mut Self {
        self.rect_with_rounding(rect, Px(0.0), DebugDrawRoundCorners::ALL)
    }

    pub fn rect_with_rounding(
        &mut self,
        rect: Rect,
        rounding: Px,
        corners: DebugDrawRoundCorners,
    ) -> &mut Self {
        if rect_is_empty(rect) || !rect_is_finite(rect) || !rounding.0.is_finite() {
            return self;
        }
        append_path_rect_points(&mut self.points, rect, rounding, corners);
        self
    }
}
