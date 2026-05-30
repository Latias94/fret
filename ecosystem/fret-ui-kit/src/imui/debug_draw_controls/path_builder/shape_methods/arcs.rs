use fret_core::{Point, Px, Size};

use super::super::ImUiDebugDrawPath;
use crate::imui::debug_draw_controls::paths::{
    append_arc_points, append_elliptical_arc_points, path_arc_segments,
    path_elliptical_arc_segments,
};

impl<'a> ImUiDebugDrawPath<'a> {
    pub fn arc_to(
        &mut self,
        center: Point,
        radius: Px,
        a_min: f32,
        a_max: f32,
        segments: usize,
    ) -> &mut Self {
        if !radius.0.is_finite() || !a_min.is_finite() || !a_max.is_finite() || radius.0 <= 0.0 {
            return self;
        }
        if radius.0 < 0.5 {
            self.points.push(center);
            return self;
        }
        append_arc_points(
            &mut self.points,
            center,
            radius,
            a_min,
            a_max,
            path_arc_segments(segments),
        );
        self
    }

    pub fn arc_to_fast(
        &mut self,
        center: Point,
        radius: Px,
        a_min_of_12: i32,
        a_max_of_12: i32,
    ) -> &mut Self {
        if !radius.0.is_finite() || radius.0 <= 0.0 {
            return self;
        }
        if radius.0 < 0.5 {
            self.points.push(center);
            return self;
        }
        let a_min = a_min_of_12 as f32 * std::f32::consts::TAU / 12.0;
        let a_max = a_max_of_12 as f32 * std::f32::consts::TAU / 12.0;
        append_arc_points(
            &mut self.points,
            center,
            radius,
            a_min,
            a_max,
            a_min_of_12.abs_diff(a_max_of_12) as usize,
        );
        self
    }

    pub fn elliptical_arc_to(
        &mut self,
        center: Point,
        radius: Size,
        rotation_radians: f32,
        a_min: f32,
        a_max: f32,
        segments: usize,
    ) -> &mut Self {
        if radius.width.0 <= 0.0
            || radius.height.0 <= 0.0
            || !radius.width.0.is_finite()
            || !radius.height.0.is_finite()
            || !rotation_radians.is_finite()
            || !a_min.is_finite()
            || !a_max.is_finite()
        {
            return self;
        }
        append_elliptical_arc_points(
            &mut self.points,
            center,
            radius,
            rotation_radians,
            a_min,
            a_max,
            path_elliptical_arc_segments(segments),
        );
        self
    }
}
