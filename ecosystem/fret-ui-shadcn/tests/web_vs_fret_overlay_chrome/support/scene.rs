use super::*;

use crate::css_color;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PaintedQuad {
    #[allow(dead_code)]
    pub(crate) rect: Rect,
    pub(crate) background: Color,
    pub(crate) border: [f32; 4],
    pub(crate) border_color: Color,
    pub(crate) corners: [f32; 4],
}

pub(crate) fn paint_solid_color(paint: Paint) -> Color {
    match paint {
        Paint::Solid(c) => c,
        other => panic!("expected Paint::Solid in overlay-chrome test harness, got {other:?}"),
    }
}

pub(crate) fn has_border(border: &[f32; 4]) -> bool {
    border.iter().any(|v| *v > 0.01)
}

pub(crate) fn find_best_chrome_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
    let mut best_containing_border: Option<PaintedQuad> = None;
    let mut best_containing_border_area = f32::INFINITY;
    let mut best_containing_background: Option<PaintedQuad> = None;
    let mut best_containing_background_area = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_paint,
            ..
        } = *op
        else {
            continue;
        };

        let background = paint_solid_color(background);
        let border_color = paint_solid_color(border_paint);
        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let has_background = background.a > 0.01;
        if !has_background && !has_border(&border) {
            continue;
        }
        if !has_border(&border) && background.a < 0.5 {
            continue;
        }

        if rect_contains(rect, target) {
            let area = rect_area(rect);
            let quad = PaintedQuad {
                rect,
                background,
                border,
                border_color,
                corners: [
                    corner_radii.top_left.0,
                    corner_radii.top_right.0,
                    corner_radii.bottom_right.0,
                    corner_radii.bottom_left.0,
                ],
            };
            if has_border(&border) {
                if area < best_containing_border_area {
                    best_containing_border_area = area;
                    best_containing_border = Some(quad);
                }
            } else if area < best_containing_background_area {
                best_containing_background_area = area;
                best_containing_background = Some(quad);
            }
        }
    }

    if best_containing_border.is_some() || best_containing_background.is_some() {
        return best_containing_border.or(best_containing_background);
    }

    let mut best_border: Option<PaintedQuad> = None;
    let mut best_border_score = f32::INFINITY;
    let mut best_background: Option<PaintedQuad> = None;
    let mut best_background_score = f32::INFINITY;
    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_paint,
            ..
        } = *op
        else {
            continue;
        };

        let background = paint_solid_color(background);
        let border_color = paint_solid_color(border_paint);
        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let has_background = background.a > 0.01;
        if !has_background && !has_border(&border) {
            continue;
        }
        if !has_border(&border) && background.a < 0.5 {
            continue;
        }

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        let quad = PaintedQuad {
            rect,
            background,
            border,
            border_color,
            corners: [
                corner_radii.top_left.0,
                corner_radii.top_right.0,
                corner_radii.bottom_right.0,
                corner_radii.bottom_left.0,
            ],
        };

        if has_border(&border) {
            if score < best_border_score {
                best_border_score = score;
                best_border = Some(quad);
            }
        } else if score < best_background_score {
            best_background_score = score;
            best_background = Some(quad);
        }
    }

    best_border.or(best_background)
}

pub(crate) fn find_best_chrome_quad_indexed(
    scene: &Scene,
    target: Rect,
) -> Option<(usize, PaintedQuad)> {
    let mut best_containing_border: Option<(usize, PaintedQuad)> = None;
    let mut best_containing_border_area = f32::INFINITY;
    let mut best_containing_background: Option<(usize, PaintedQuad)> = None;
    let mut best_containing_background_area = f32::INFINITY;

    for (idx, op) in scene.ops().iter().enumerate() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_paint,
            ..
        } = *op
        else {
            continue;
        };

        let background = paint_solid_color(background);
        let border_color = paint_solid_color(border_paint);
        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let has_background = background.a > 0.01;
        if !has_background && !has_border(&border) {
            continue;
        }
        if !has_border(&border) && background.a < 0.5 {
            continue;
        }

        if rect_contains(rect, target) {
            let area = rect_area(rect);
            let quad = PaintedQuad {
                rect,
                background,
                border,
                border_color,
                corners: [
                    corner_radii.top_left.0,
                    corner_radii.top_right.0,
                    corner_radii.bottom_right.0,
                    corner_radii.bottom_left.0,
                ],
            };
            if has_border(&border) {
                if area < best_containing_border_area {
                    best_containing_border_area = area;
                    best_containing_border = Some((idx, quad));
                }
            } else if area < best_containing_background_area {
                best_containing_background_area = area;
                best_containing_background = Some((idx, quad));
            }
        }
    }

    if best_containing_border.is_some() || best_containing_background.is_some() {
        return best_containing_border.or(best_containing_background);
    }

    let mut best_border: Option<(usize, PaintedQuad)> = None;
    let mut best_border_score = f32::INFINITY;
    let mut best_background: Option<(usize, PaintedQuad)> = None;
    let mut best_background_score = f32::INFINITY;
    for (idx, op) in scene.ops().iter().enumerate() {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_paint,
            ..
        } = *op
        else {
            continue;
        };

        let background = paint_solid_color(background);
        let border_color = paint_solid_color(border_paint);
        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let has_background = background.a > 0.01;
        if !has_background && !has_border(&border) {
            continue;
        }
        if !has_border(&border) && background.a < 0.5 {
            continue;
        }

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        let quad = PaintedQuad {
            rect,
            background,
            border,
            border_color,
            corners: [
                corner_radii.top_left.0,
                corner_radii.top_right.0,
                corner_radii.bottom_right.0,
                corner_radii.bottom_left.0,
            ],
        };

        if has_border(&border) {
            if score < best_border_score {
                best_border_score = score;
                best_border = Some((idx, quad));
            }
        } else if score < best_background_score {
            best_background_score = score;
            best_background = Some((idx, quad));
        }
    }

    best_border.or(best_background)
}

pub(crate) fn find_best_solid_quad_within_matching_bg(
    scene: &Scene,
    target: Rect,
    expected_bg: css_color::Rgba,
) -> Option<PaintedQuad> {
    let target_area = rect_area(target).max(1.0);
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;
    let mut best_area = f32::INFINITY;

    scene_walk(scene, |st, op| {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_paint,
            ..
        } = *op
        else {
            return;
        };

        let rect = transform_rect_bounds(st.transform, rect);
        let background = color_with_opacity(paint_solid_color(background), st.opacity);
        let border_color = paint_solid_color(border_paint);

        if rect_intersection_area(rect, target) <= 0.01 {
            return;
        }
        if background.a <= 0.01 {
            return;
        }
        if background.a < 0.5 {
            return;
        }
        let area = rect_area(rect);
        if area > target_area * 40.0 {
            return;
        }

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let quad = PaintedQuad {
            rect,
            background,
            border,
            border_color,
            corners: [
                corner_radii.top_left.0,
                corner_radii.top_right.0,
                corner_radii.bottom_right.0,
                corner_radii.bottom_left.0,
            ],
        };

        let bg = color_to_rgba(background);
        let score = (bg.r - expected_bg.r).abs()
            + (bg.g - expected_bg.g).abs()
            + (bg.b - expected_bg.b).abs()
            + (bg.a - expected_bg.a).abs();
        if score < best_score || (score <= best_score + 0.0001 && area < best_area) {
            best_score = score;
            best_area = area;
            best = Some(quad);
        }
    });

    best
}

pub(crate) fn find_best_quad_within_matching_bg(
    scene: &Scene,
    target: Rect,
    expected_bg: css_color::Rgba,
) -> Option<PaintedQuad> {
    let target_area = rect_area(target).max(1.0);
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;
    let mut best_area = f32::INFINITY;

    let max_area_ratio = 8.0;
    let min_alpha = (expected_bg.a * 0.25).max(0.01);

    scene_walk(scene, |st, op| {
        let SceneOp::Quad {
            rect,
            border,
            corner_radii,
            background,
            border_paint,
            ..
        } = *op
        else {
            return;
        };

        let rect = transform_rect_bounds(st.transform, rect);
        let background = color_with_opacity(paint_solid_color(background), st.opacity);
        let border_color = paint_solid_color(border_paint);

        if rect_intersection_area(rect, target) <= 0.01 {
            return;
        }
        if background.a < min_alpha {
            return;
        }
        let area = rect_area(rect);
        if area > target_area * max_area_ratio {
            return;
        }

        let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
        let quad = PaintedQuad {
            rect,
            background,
            border,
            border_color,
            corners: [
                corner_radii.top_left.0,
                corner_radii.top_right.0,
                corner_radii.bottom_right.0,
                corner_radii.bottom_left.0,
            ],
        };

        let bg = color_to_rgba(background);
        let score = (bg.r - expected_bg.r).abs()
            + (bg.g - expected_bg.g).abs()
            + (bg.b - expected_bg.b).abs()
            + (bg.a - expected_bg.a).abs();
        if score < best_score || (score <= best_score + 0.0001 && area < best_area) {
            best_score = score;
            best_area = area;
            best = Some(quad);
        }
    });

    best
}

pub(crate) fn find_best_solid_quad_near_point(scene: &Scene, point: Point) -> Option<Rect> {
    let mut best_rect: Option<Rect> = None;
    let mut best_area = f32::INFINITY;
    let mut best_center_dist = f32::INFINITY;

    scene_walk(scene, |st, op| {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            return;
        };

        let rect = transform_rect_bounds(st.transform, rect);
        let background = color_with_opacity(paint_solid_color(background), st.opacity);

        if background.a < 0.5 {
            return;
        }
        if !rect_contains_point_with_margin(rect, point, 6.0) {
            return;
        }

        let area = rect_area(rect);
        let center = bounds_center(rect);
        let center_dist = (center.x.0 - point.x.0).abs() + (center.y.0 - point.y.0).abs();

        if area < best_area || (area == best_area && center_dist < best_center_dist) {
            best_area = area;
            best_center_dist = center_dist;
            best_rect = Some(rect);
        }
    });

    best_rect
}

#[derive(Clone, Copy)]
pub(crate) struct SceneWalkState {
    pub(crate) transform: Transform2D,
    pub(crate) opacity: f32,
}

pub(crate) fn scene_walk(scene: &Scene, mut f: impl FnMut(SceneWalkState, &SceneOp)) {
    let mut transform_stack: Vec<Transform2D> = Vec::new();
    let mut opacity_stack: Vec<f32> = Vec::new();
    let mut st = SceneWalkState {
        transform: Transform2D::IDENTITY,
        opacity: 1.0,
    };

    for op in scene.ops() {
        match *op {
            SceneOp::PushTransform { transform } => {
                transform_stack.push(st.transform);
                st.transform = st.transform.compose(transform);
            }
            SceneOp::PopTransform => {
                st.transform = transform_stack.pop().unwrap_or(Transform2D::IDENTITY);
            }
            SceneOp::PushOpacity { opacity } => {
                opacity_stack.push(st.opacity);
                st.opacity *= opacity;
            }
            SceneOp::PopOpacity => {
                st.opacity = opacity_stack.pop().unwrap_or(1.0);
            }
            _ => f(st, op),
        }
    }
}

pub(crate) fn transform_rect_bounds(transform: Transform2D, rect: Rect) -> Rect {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;

    let p0 = transform.apply_point(Point::new(Px(x0), Px(y0)));
    let p1 = transform.apply_point(Point::new(Px(x1), Px(y0)));
    let p2 = transform.apply_point(Point::new(Px(x0), Px(y1)));
    let p3 = transform.apply_point(Point::new(Px(x1), Px(y1)));

    let min_x = p0.x.0.min(p1.x.0).min(p2.x.0).min(p3.x.0);
    let max_x = p0.x.0.max(p1.x.0).max(p2.x.0).max(p3.x.0);
    let min_y = p0.y.0.min(p1.y.0).min(p2.y.0).min(p3.y.0);
    let max_y = p0.y.0.max(p1.y.0).max(p2.y.0).max(p3.y.0);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

pub(crate) fn color_with_opacity(color: Color, opacity: f32) -> Color {
    Color {
        a: (color.a * opacity).clamp(0.0, 1.0),
        ..color
    }
}

pub(crate) fn find_best_text_color_near(
    scene: &Scene,
    search_within: Rect,
    near: Point,
) -> Option<css_color::Rgba> {
    let mut best_raw: Option<css_color::Rgba> = None;
    let mut best_raw_score = f32::INFINITY;
    let mut best_tx: Option<css_color::Rgba> = None;
    let mut best_tx_score = f32::INFINITY;
    let mut best_any: Option<css_color::Rgba> = None;
    let mut best_any_score = f32::INFINITY;

    scene_walk(scene, |st, op| {
        let SceneOp::Text { origin, color, .. } = *op else {
            return;
        };
        let raw_origin = origin;
        let tx_origin = st.transform.apply_point(origin);
        let rgba = color_to_rgba(color_with_opacity(color, st.opacity));
        if rgba.a <= 0.01 {
            return;
        }

        if rect_contains_point_with_margin(search_within, tx_origin, 10.0) {
            let dist_score = (tx_origin.x.0 - near.x.0).abs() + (tx_origin.y.0 - near.y.0).abs();
            if dist_score < best_tx_score {
                best_tx_score = dist_score;
                best_tx = Some(rgba);
            }
        }
        if rect_contains_point_with_margin(search_within, raw_origin, 10.0) {
            let dist_score = (raw_origin.x.0 - near.x.0).abs() + (raw_origin.y.0 - near.y.0).abs();
            if dist_score < best_raw_score {
                best_raw_score = dist_score;
                best_raw = Some(rgba);
            }
        }

        let dist_score = (tx_origin.x.0 - near.x.0).abs() + (tx_origin.y.0 - near.y.0).abs();
        if dist_score < best_any_score {
            best_any_score = dist_score;
            best_any = Some(rgba);
        }
    });

    best_tx.or(best_raw).or(best_any)
}
