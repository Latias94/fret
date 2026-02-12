use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PaintedQuad {
    pub(crate) rect: Rect,
    pub(crate) background: fret_core::Paint,
}

pub(crate) fn find_best_background_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            continue;
        };

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad { rect, background });
        }
    }

    best
}

pub(crate) fn find_best_opaque_background_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            continue;
        };

        if paint_max_alpha(background) <= 0.001 {
            continue;
        }

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad { rect, background });
        }
    }

    best
}

pub(crate) fn find_scene_quad_with_rect_close(
    scene: &Scene,
    expected: WebRect,
    tol: f32,
) -> Option<Rect> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::Quad { rect, .. } => Some(*rect),
            _ => None,
        })
        .find(|rect| rect_close_px(*rect, expected, tol))
}

pub(crate) fn find_scene_quad_background_with_rect_close(
    scene: &Scene,
    expected: WebRect,
    tol: f32,
) -> Option<(Rect, fret_core::Paint)> {
    scene.ops().iter().find_map(|op| {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            return None;
        };
        if rect_close_px(rect, expected, tol) {
            Some((rect, background))
        } else {
            None
        }
    })
}

pub(crate) fn find_scene_quad_background_with_world_rect_close(
    scene: &Scene,
    expected: WebRect,
    tol: f32,
) -> Option<(Rect, fret_core::Paint)> {
    let mut transform_stack: Vec<Transform2D> = vec![Transform2D::IDENTITY];

    for op in scene.ops() {
        match *op {
            SceneOp::PushTransform { transform } => {
                let current = *transform_stack.last().expect("transform stack not empty");
                transform_stack.push(current * transform);
            }
            SceneOp::PopTransform => {
                transform_stack.pop();
                debug_assert!(!transform_stack.is_empty(), "unbalanced PopTransform");
                if transform_stack.is_empty() {
                    transform_stack.push(Transform2D::IDENTITY);
                }
            }
            SceneOp::Quad {
                rect, background, ..
            } => {
                let current = *transform_stack.last().expect("transform stack not empty");
                let world_rect = rect_aabb_after_transform(current, rect);
                if rect_close_px(world_rect, expected, tol) {
                    return Some((world_rect, background));
                }
            }
            _ => {}
        }
    }

    None
}

pub(crate) fn debug_dump_scene_quads_near_expected(
    scene: &Scene,
    expected: WebRect,
    expected_bg: Option<Rgba>,
) {
    let mut transform_stack: Vec<Transform2D> = vec![Transform2D::IDENTITY];
    let mut quads: Vec<(f32, Rect, fret_core::Paint, Transform2D)> = Vec::new();

    for op in scene.ops() {
        match *op {
            SceneOp::PushTransform { transform } => {
                let current = *transform_stack.last().expect("transform stack not empty");
                transform_stack.push(current * transform);
            }
            SceneOp::PopTransform => {
                transform_stack.pop();
                if transform_stack.is_empty() {
                    transform_stack.push(Transform2D::IDENTITY);
                }
            }
            SceneOp::Quad {
                rect, background, ..
            } => {
                let current = *transform_stack.last().expect("transform stack not empty");
                let world_rect = rect_aabb_after_transform(current, rect);
                let d = rect_diff_metric(world_rect, expected);
                quads.push((d, world_rect, background, current));
            }
            _ => {}
        }
    }

    quads.sort_by(|a, b| a.0.total_cmp(&b.0));

    eprintln!("--- debug_dump_scene_quads_near_expected ---");
    eprintln!(
        "expected rect: x={:.2} y={:.2} w={:.2} h={:.2}",
        expected.x, expected.y, expected.w, expected.h
    );
    if let Some(bg) = expected_bg {
        eprintln!(
            "expected bg (linear rgba): r={:.4} g={:.4} b={:.4} a={:.4}",
            bg.r, bg.g, bg.b, bg.a
        );
    }

    for (idx, (d, rect, bg, transform)) in quads.iter().take(12).enumerate() {
        let rgba = paint_to_rgba(*bg);
        eprintln!(
            "#{idx:02} rectΔ={d:.2} rect=({:.2},{:.2},{:.2},{:.2}) bg=({:.4},{:.4},{:.4},{:.4}) transform(tx={:.2},ty={:.2},a={:.3},b={:.3},c={:.3},d={:.3})",
            rect.origin.x.0,
            rect.origin.y.0,
            rect.size.width.0,
            rect.size.height.0,
            rgba.r,
            rgba.g,
            rgba.b,
            rgba.a,
            transform.tx,
            transform.ty,
            transform.a,
            transform.b,
            transform.c,
            transform.d
        );
    }

    if let Some(expected_bg) = expected_bg {
        let mut by_color: Vec<(f32, Rect, fret_core::Paint)> = quads
            .iter()
            .map(|(_d, rect, bg, _)| {
                (
                    rgba_diff_metric(paint_to_rgba(*bg), expected_bg),
                    *rect,
                    *bg,
                )
            })
            .collect();
        by_color.sort_by(|a, b| a.0.total_cmp(&b.0));
        eprintln!("top 8 by bg color diff:");
        for (idx, (d, rect, bg)) in by_color.iter().take(8).enumerate() {
            let rgba = paint_to_rgba(*bg);
            eprintln!(
                "#{idx:02} bgΔ={d:.4} rect=({:.2},{:.2},{:.2},{:.2}) bg=({:.4},{:.4},{:.4},{:.4})",
                rect.origin.x.0,
                rect.origin.y.0,
                rect.size.width.0,
                rect.size.height.0,
                rgba.r,
                rgba.g,
                rgba.b,
                rgba.a
            );
        }
    }
}
