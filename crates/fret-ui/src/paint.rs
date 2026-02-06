use crate::element::{RingPlacement, RingStyle, ShadowLayerStyle, ShadowStyle};
use fret_core::{Color, Corners, DrawOrder, Edges, Point, Px, Rect, Scene, SceneOp, Size};

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

fn rect_inflate(rect: Rect, delta: Px) -> Rect {
    let d = delta.0.max(0.0);
    Rect::new(
        fret_core::Point::new(Px(rect.origin.x.0 - d), Px(rect.origin.y.0 - d)),
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
        fret_core::Point::new(Px(rect.origin.x.0 + d), Px(rect.origin.y.0 + d)),
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

fn paint_shadow_layer(
    scene: &mut Scene,
    order: DrawOrder,
    bounds: Rect,
    layer: ShadowLayerStyle,
    corner_radii: Corners,
) {
    if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
        return;
    }
    if !layer.offset_x.0.is_finite()
        || !layer.offset_y.0.is_finite()
        || !layer.blur.0.is_finite()
        || !layer.spread.0.is_finite()
    {
        return;
    }

    let blur = layer.blur.0.max(0.0);
    let spread = layer.spread.0;

    // Approximate blur by drawing multiple expanded quads with alpha falloff. Keep the number of
    // steps bounded, but preserve the correct outer footprint (`spread + blur`) by using fractional
    // expansion steps when `blur` exceeds the cap.
    let max_steps = 32_f32;
    let steps = blur.ceil().clamp(0.0, max_steps) as usize;
    let denom = (steps as f32).max(1.0);

    for i in (0..=steps).rev() {
        let t = i as f32 / denom;
        let layer_spread = spread + blur * t;
        let rect = {
            let mut rect = rect_expand(bounds, Px(layer_spread));
            rect.origin.x = Px(rect.origin.x.0 + layer.offset_x.0);
            rect.origin.y = Px(rect.origin.y.0 + layer.offset_y.0);
            rect
        };
        if rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0 {
            continue;
        }

        let alpha = if steps == 0 {
            layer.color.a
        } else {
            layer.color.a / (1.0 + i as f32)
        };
        let background = color_with_alpha(layer.color, alpha);

        scene.push(SceneOp::Quad {
            order,
            rect,
            background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: corners_expand(corner_radii, Px(layer_spread)),
        });
    }
}

/// Paint a low-level drop shadow for an elevated surface.
///
/// The baseline implementation approximates softness by drawing multiple expanded quads with alpha
/// falloff (ADR 0060). Backends are free to implement a true blur later without changing this API.
pub fn paint_shadow(scene: &mut Scene, order: DrawOrder, bounds: Rect, shadow: ShadowStyle) {
    paint_shadow_layer(scene, order, bounds, shadow.primary, shadow.corner_radii);
    if let Some(layer) = shadow.secondary {
        paint_shadow_layer(scene, order, bounds, layer, shadow.corner_radii);
    }
}

/// Paint a simple focus ring decoration (e.g. shadcn-style `focus-visible:ring-*`).
///
/// This intentionally stays renderer-friendly: it maps to one or two `SceneOp::Quad` operations.
pub fn paint_focus_ring(scene: &mut Scene, order: DrawOrder, bounds: Rect, ring: RingStyle) {
    let width = Px(ring.width.0.max(0.0));
    if width.0 <= 0.0 || !width.0.is_finite() {
        return;
    }
    let offset = Px(ring.offset.0.max(0.0));
    let color = ring.color;

    match ring.placement {
        RingPlacement::Inset => {
            let rect = rect_deflate(bounds, offset);
            if rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0 {
                return;
            }
            scene.push(SceneOp::Quad {
                order,
                rect,
                background: Color::TRANSPARENT,
                border: Edges::all(width),
                border_color: color,
                corner_radii: corners_deflate(ring.corner_radii, offset),
            });
        }
        RingPlacement::Outset => {
            if let Some(offset_color) = ring.offset_color
                && offset.0 > 0.0
                && offset.0.is_finite()
            {
                let rect = rect_inflate(bounds, offset);
                scene.push(SceneOp::Quad {
                    order,
                    rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(offset),
                    border_color: offset_color,
                    corner_radii: corners_inflate(ring.corner_radii, offset),
                });
            }

            let outer = Px(offset.0 + width.0);
            if outer.0 > 0.0 && outer.0.is_finite() {
                let rect = rect_inflate(bounds, outer);
                scene.push(SceneOp::Quad {
                    order,
                    rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(width),
                    border_color: color,
                    corner_radii: corners_inflate(ring.corner_radii, outer),
                });
            }
        }
    }
}

/// Paint a paint-only "state layer" overlay (Material-like interaction feedback).
///
/// This is a mechanism-level primitive: it does not define *when* a state layer should be shown,
/// only how to render one efficiently in the scene.
pub fn paint_state_layer(
    scene: &mut Scene,
    order: DrawOrder,
    bounds: Rect,
    color: Color,
    opacity: f32,
    corner_radii: Corners,
) {
    if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
        return;
    }

    let a = (color.a * opacity).clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }

    scene.push(SceneOp::Quad {
        order,
        rect: bounds,
        background: Color { a, ..color },
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii,
    });
}

/// Paint a low-level ripple ink circle (Material-like interaction feedback).
///
/// This is a mechanism-level primitive: it only emits scene ops for a ripple "frame". Policy and
/// state management (timing, easing, bounded/unbounded choice, fade rules) live in ecosystem crates.
///
/// - `bounds` is used for bounded clipping and as the coordinate space for `origin`.
/// - If `clip_corner_radii` is `Some`, the ripple is clipped to a rounded-rect matching `bounds`.
#[allow(clippy::too_many_arguments)]
pub fn paint_ripple(
    scene: &mut Scene,
    order: DrawOrder,
    bounds: Rect,
    origin: Point,
    radius: Px,
    color: Color,
    opacity: f32,
    clip_corner_radii: Option<Corners>,
) {
    if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
        return;
    }

    let r = radius.0.max(0.0);
    if r <= 0.0 || !r.is_finite() {
        return;
    }

    let a = (color.a * opacity).clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }

    if let Some(corner_radii) = clip_corner_radii {
        scene.push(SceneOp::PushClipRRect {
            rect: bounds,
            corner_radii,
        });
    }

    let circle = Rect::new(
        Point::new(Px(origin.x.0 - r), Px(origin.y.0 - r)),
        Size::new(Px(r * 2.0), Px(r * 2.0)),
    );

    scene.push(SceneOp::Quad {
        order,
        rect: circle,
        background: Color { a, ..color },
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(r)),
    });

    if clip_corner_radii.is_some() {
        scene.push(SceneOp::PopClip);
    }
}

#[cfg(test)]
mod tests {
    use super::{paint_ripple, paint_state_layer};
    use fret_core::{Color, Corners, DrawOrder, Px, Rect, Scene, Size};

    #[test]
    fn paint_state_layer_emits_single_quad_with_expected_alpha() {
        let mut scene = Scene::default();
        paint_state_layer(
            &mut scene,
            DrawOrder(0),
            Rect::new(
                fret_core::Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.5,
            },
            0.2,
            Corners::all(Px(4.0)),
        );

        assert_eq!(scene.ops().len(), 1);
        match scene.ops()[0] {
            fret_core::SceneOp::Quad { background, .. } => {
                assert!((background.a - 0.1).abs() < 1e-6);
            }
            _ => panic!("expected quad"),
        }
    }

    #[test]
    fn paint_ripple_bounded_emits_clip_quad_and_pop() {
        let mut scene = Scene::default();
        paint_ripple(
            &mut scene,
            DrawOrder(0),
            Rect::new(
                fret_core::Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
            fret_core::Point::new(Px(5.0), Px(5.0)),
            Px(4.0),
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.5,
            },
            0.5,
            Some(Corners::all(Px(2.0))),
        );

        assert_eq!(scene.ops().len(), 3);
        match scene.ops()[0] {
            fret_core::SceneOp::PushClipRRect { rect, .. } => {
                assert!((rect.size.width.0 - 10.0).abs() < 1e-6);
            }
            _ => panic!("expected clip push"),
        }
        match scene.ops()[1] {
            fret_core::SceneOp::Quad {
                rect,
                background,
                corner_radii,
                ..
            } => {
                assert!((rect.origin.x.0 - 1.0).abs() < 1e-6);
                assert!((rect.origin.y.0 - 1.0).abs() < 1e-6);
                assert!((rect.size.width.0 - 8.0).abs() < 1e-6);
                assert!((corner_radii.top_left.0 - 4.0).abs() < 1e-6);
                assert!((background.a - 0.25).abs() < 1e-6);
            }
            _ => panic!("expected quad"),
        }
        match scene.ops()[2] {
            fret_core::SceneOp::PopClip => {}
            _ => panic!("expected clip pop"),
        }
    }

    #[test]
    fn paint_ripple_unbounded_emits_single_quad() {
        let mut scene = Scene::default();
        paint_ripple(
            &mut scene,
            DrawOrder(0),
            Rect::new(
                fret_core::Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
            fret_core::Point::new(Px(5.0), Px(5.0)),
            Px(4.0),
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            0.1,
            None,
        );

        assert_eq!(scene.ops().len(), 1);
        match scene.ops()[0] {
            fret_core::SceneOp::Quad { background, .. } => {
                assert!((background.a - 0.1).abs() < 1e-6);
            }
            _ => panic!("expected quad"),
        }
    }
}
