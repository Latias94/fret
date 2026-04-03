use super::*;
use crate::Size;

fn rect_inflate(rect: Rect, delta: Px) -> Rect {
    let d = delta.0.max(0.0);
    Rect::new(
        Point::new(Px(rect.origin.x.0 - d), Px(rect.origin.y.0 - d)),
        Size::new(
            Px(rect.size.width.0 + d * 2.0),
            Px(rect.size.height.0 + d * 2.0),
        ),
    )
}

fn rect_deflate(rect: Rect, delta: Px) -> Rect {
    let d = delta.0.max(0.0);
    let w = (rect.size.width.0 - d * 2.0).max(0.0);
    let h = (rect.size.height.0 - d * 2.0).max(0.0);
    Rect::new(
        Point::new(Px(rect.origin.x.0 + d), Px(rect.origin.y.0 + d)),
        Size::new(Px(w), Px(h)),
    )
}

fn rect_expand(rect: Rect, delta: Px) -> Rect {
    if delta.0 >= 0.0 {
        rect_inflate(rect, delta)
    } else {
        rect_deflate(rect, Px(-delta.0))
    }
}

fn corners_inflate(mut corners: Corners, delta: Px) -> Corners {
    let d = delta.0.max(0.0);
    corners.top_left = Px((corners.top_left.0 + d).max(0.0));
    corners.top_right = Px((corners.top_right.0 + d).max(0.0));
    corners.bottom_left = Px((corners.bottom_left.0 + d).max(0.0));
    corners.bottom_right = Px((corners.bottom_right.0 + d).max(0.0));
    corners
}

fn corners_deflate(mut corners: Corners, delta: Px) -> Corners {
    let d = delta.0.max(0.0);
    corners.top_left = Px((corners.top_left.0 - d).max(0.0));
    corners.top_right = Px((corners.top_right.0 - d).max(0.0));
    corners.bottom_left = Px((corners.bottom_left.0 - d).max(0.0));
    corners.bottom_right = Px((corners.bottom_right.0 - d).max(0.0));
    corners
}

fn corners_expand(corners: Corners, delta: Px) -> Corners {
    if delta.0 >= 0.0 {
        corners_inflate(corners, delta)
    } else {
        corners_deflate(corners, Px(-delta.0))
    }
}

fn color_with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha.clamp(0.0, 1.0);
    color
}

fn shadow_alpha_weight(step_index: usize) -> f32 {
    1.0 / (1.0 + step_index as f32)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowRRectFallbackSpec {
    pub order: DrawOrder,
    pub rect: Rect,
    pub corner_radii: Corners,
    pub offset: Point,
    pub spread: Px,
    pub blur_radius: Px,
    pub color: Color,
}

/// Return the deterministic quad-approximation fallback for a rounded-rect shadow primitive.
///
/// This helper exists for backends and tools that need an explicit degradation path for
/// `SceneOp::ShadowRRect` without routing back through `fret-ui` authoring APIs.
pub fn shadow_rrect_fallback_quads(spec: ShadowRRectFallbackSpec) -> Vec<SceneOp> {
    let ShadowRRectFallbackSpec {
        order,
        rect,
        corner_radii,
        offset,
        spread,
        blur_radius,
        color,
    } = spec;
    if rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0 {
        return Vec::new();
    }
    if !offset.x.0.is_finite()
        || !offset.y.0.is_finite()
        || !blur_radius.0.is_finite()
        || !spread.0.is_finite()
        || color.a <= 0.0
    {
        return Vec::new();
    }

    let blur = blur_radius.0.max(0.0);
    let max_steps = 32_f32;
    let steps = blur.ceil().clamp(0.0, max_steps) as usize;
    let denom = (steps as f32).max(1.0);
    let alpha_weight_sum = if steps == 0 {
        1.0
    } else {
        (0..=steps)
            .map(shadow_alpha_weight)
            .sum::<f32>()
            .max(f32::EPSILON)
    };

    let mut out = Vec::with_capacity(steps.saturating_add(1));
    for i in (0..=steps).rev() {
        let t = i as f32 / denom;
        let layer_spread = spread.0 + blur * t;
        let fallback_rect = {
            let mut fallback_rect = rect_expand(rect, Px(layer_spread));
            fallback_rect.origin.x = Px(fallback_rect.origin.x.0 + offset.x.0);
            fallback_rect.origin.y = Px(fallback_rect.origin.y.0 + offset.y.0);
            fallback_rect
        };
        if fallback_rect.size.width.0 <= 0.0 || fallback_rect.size.height.0 <= 0.0 {
            continue;
        }

        let alpha = if steps == 0 {
            color.a
        } else {
            color.a * (shadow_alpha_weight(i) / alpha_weight_sum)
        };
        out.push(SceneOp::Quad {
            order,
            rect: fallback_rect,
            background: Paint::Solid(color_with_alpha(color, alpha)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT).into(),
            corner_radii: corners_expand(corner_radii, Px(layer_spread)),
        });
    }

    out
}

impl SceneRecording {
    /// Replay the deterministic quad fallback for a rounded-rect shadow primitive into this scene.
    pub fn push_shadow_rrect_quad_fallback(&mut self, spec: ShadowRRectFallbackSpec) {
        for op in shadow_rrect_fallback_quads(spec) {
            self.push(op);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_rrect_fallback_quads_keep_expected_profile() {
        let ops = shadow_rrect_fallback_quads(ShadowRRectFallbackSpec {
            order: DrawOrder(3),
            rect: Rect::new(
                Point::new(Px(20.0), Px(10.0)),
                Size::new(Px(40.0), Px(24.0)),
            ),
            corner_radii: Corners::all(Px(6.0)),
            offset: Point::new(Px(0.0), Px(2.0)),
            spread: Px(1.0),
            blur_radius: Px(4.0),
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.18,
            },
        });

        assert_eq!(ops.len(), 5);
        let SceneOp::Quad {
            rect: outer_rect,
            background: outer_background,
            corner_radii: outer_radii,
            ..
        } = ops[0]
        else {
            panic!("expected outer fallback quad");
        };
        let SceneOp::Quad {
            rect: inner_rect,
            background: inner_background,
            corner_radii: inner_radii,
            ..
        } = ops[4]
        else {
            panic!("expected inner fallback quad");
        };

        let Paint::Solid(outer_color) = outer_background.paint else {
            panic!("expected outer fallback color");
        };
        let Paint::Solid(inner_color) = inner_background.paint else {
            panic!("expected inner fallback color");
        };

        assert!(outer_rect.size.width.0 > inner_rect.size.width.0);
        assert!(outer_rect.size.height.0 > inner_rect.size.height.0);
        assert!(outer_radii.top_left.0 > inner_radii.top_left.0);
        assert!(outer_color.a < inner_color.a);
    }

    #[test]
    fn push_shadow_rrect_quad_fallback_replays_ops_into_scene() {
        let mut scene = Scene::default();
        scene.push_shadow_rrect_quad_fallback(ShadowRRectFallbackSpec {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(12.0), Px(8.0))),
            corner_radii: Corners::all(Px(4.0)),
            offset: Point::new(Px(1.0), Px(2.0)),
            spread: Px(0.0),
            blur_radius: Px(2.0),
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.12,
            },
        });

        assert_eq!(scene.ops_len(), 3);
        assert!(
            scene
                .ops()
                .iter()
                .all(|op| matches!(op, SceneOp::Quad { .. }))
        );
    }
}
