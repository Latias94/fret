use super::*;

use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Corners, ImageId, PathCommand, Size, UvRect};
use fret_ui::SvgSource;
use fret_ui::element::ElementKind;

use super::geometry::triangle_is_degenerate;
use super::paint_helpers::{
    corner_radii_are_visible, normalized_opacity, rounded_rect_corner_radii, uv_rect_is_valid,
};
use super::paths::{
    bezier_cubic_path, bezier_quadratic_path, circle_path, concave_poly_fill_path,
    convex_poly_fill_path, ellipse_path, ngon_path, polyline_path, quad_path, rect_path,
    triangle_path,
};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    )
}

fn empty_commands() -> Arc<[DebugDrawCommand]> {
    Arc::from(Vec::<DebugDrawCommand>::new().into_boxed_slice())
}

fn assert_point_near(actual: Point, expected: Point) {
    assert!(
        (actual.x.0 - expected.x.0).abs() <= 0.000_1,
        "x mismatch: actual {:?}, expected {:?}",
        actual,
        expected
    );
    assert!(
        (actual.y.0 - expected.y.0).abs() <= 0.000_1,
        "y mismatch: actual {:?}, expected {:?}",
        actual,
        expected
    );
}

mod draw_list;
mod element;
mod paint_helpers;
mod path_builder;
mod paths;
mod style;
