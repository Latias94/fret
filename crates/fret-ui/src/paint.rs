use crate::element::{RingPlacement, RingStyle, ShadowStyle};
use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect, Scene, SceneOp, Size};

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

fn color_with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha.clamp(0.0, 1.0);
    color
}

/// Paint a low-level drop shadow for an elevated surface.
///
/// The baseline implementation approximates softness by drawing multiple expanded quads with alpha
/// falloff (ADR 0060). Backends are free to implement a true blur later without changing this API.
pub fn paint_shadow(scene: &mut Scene, order: DrawOrder, bounds: Rect, shadow: ShadowStyle) {
    if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
        return;
    }
    if !shadow.offset_x.0.is_finite()
        || !shadow.offset_y.0.is_finite()
        || !shadow.spread.0.is_finite()
    {
        return;
    }

    let base_spread = Px(shadow.spread.0.max(0.0));
    let softness = shadow.softness.min(8) as usize;

    // Draw from the largest layer inward so tighter layers sit on top.
    for i in (0..=softness).rev() {
        let layer_spread = Px(base_spread.0 + i as f32);
        let mut rect = rect_inflate(bounds, layer_spread);
        rect.origin.x = Px(rect.origin.x.0 + shadow.offset_x.0);
        rect.origin.y = Px(rect.origin.y.0 + shadow.offset_y.0);

        let denom = 1.0 + i as f32;
        let alpha = shadow.color.a / denom;
        let background = color_with_alpha(shadow.color, alpha);

        scene.push(SceneOp::Quad {
            order,
            rect,
            background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: corners_inflate(shadow.corner_radii, layer_spread),
        });
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
