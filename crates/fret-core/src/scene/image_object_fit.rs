use crate::ViewportFit;
use crate::geometry::{Point, Px, Rect, Size};
use crate::scene::UvRect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageObjectFitMapped {
    pub draw_rect: Rect,
    pub uv: UvRect,
}

/// Maps a destination rect + source image size into a draw rect and a normalized UV rect,
/// following the `SceneOp::Image` v1 `object-fit` contract (ADR 0231).
///
/// Returns `None` for degenerate inputs (zero/negative destination size or zero source size).
pub fn map_image_object_fit(
    destination_rect: Rect,
    source_px_size: (u32, u32),
    fit: ViewportFit,
) -> Option<ImageObjectFitMapped> {
    let (sw, sh) = source_px_size;
    if sw == 0 || sh == 0 {
        return None;
    }

    let dw = destination_rect.size.width.0.max(0.0);
    let dh = destination_rect.size.height.0.max(0.0);
    if dw <= 0.0 || dh <= 0.0 || !dw.is_finite() || !dh.is_finite() {
        return None;
    }

    let sw = sw as f32;
    let sh = sh as f32;

    match fit {
        ViewportFit::Stretch => Some(ImageObjectFitMapped {
            draw_rect: destination_rect,
            uv: UvRect::FULL,
        }),
        ViewportFit::Contain => {
            let s = (dw / sw).min(dh / sh);
            if !s.is_finite() || s <= 0.0 {
                return None;
            }

            let draw_w = sw * s;
            let draw_h = sh * s;
            let x = destination_rect.origin.x.0 + (dw - draw_w) * 0.5;
            let y = destination_rect.origin.y.0 + (dh - draw_h) * 0.5;

            Some(ImageObjectFitMapped {
                draw_rect: Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(draw_w), Px(draw_h))),
                uv: UvRect::FULL,
            })
        }
        ViewportFit::Cover => {
            let s = (dw / sw).max(dh / sh);
            if !s.is_finite() || s <= 0.0 {
                return None;
            }

            let cover_w = sw * s;
            let cover_h = sh * s;
            if cover_w <= 0.0 || cover_h <= 0.0 || !cover_w.is_finite() || !cover_h.is_finite() {
                return None;
            }

            let mut u0 = ((cover_w - dw) * 0.5) / cover_w;
            let mut v0 = ((cover_h - dh) * 0.5) / cover_h;
            let mut u1 = 1.0 - u0;
            let mut v1 = 1.0 - v0;

            if !(u0.is_finite() && v0.is_finite() && u1.is_finite() && v1.is_finite()) {
                return None;
            }

            u0 = u0.clamp(0.0, 1.0);
            v0 = v0.clamp(0.0, 1.0);
            u1 = u1.clamp(0.0, 1.0);
            v1 = v1.clamp(0.0, 1.0);

            if u1 < u0 {
                u1 = u0;
            }
            if v1 < v0 {
                v1 = v0;
            }

            Some(ImageObjectFitMapped {
                draw_rect: destination_rect,
                uv: UvRect { u0, v0, u1, v1 },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn stretch_maps_to_full_uv_and_dest_rect() {
        let mapped = map_image_object_fit(
            rect(10.0, 20.0, 100.0, 80.0),
            (200, 100),
            ViewportFit::Stretch,
        )
        .unwrap();
        assert_eq!(mapped.draw_rect, rect(10.0, 20.0, 100.0, 80.0));
        assert_eq!(mapped.uv, UvRect::FULL);
    }

    #[test]
    fn contain_letterboxes_by_shrinking_draw_rect() {
        let mapped = map_image_object_fit(
            rect(0.0, 0.0, 100.0, 100.0),
            (200, 100),
            ViewportFit::Contain,
        )
        .unwrap();
        assert_eq!(mapped.draw_rect, rect(0.0, 25.0, 100.0, 50.0));
        assert_eq!(mapped.uv, UvRect::FULL);
    }

    #[test]
    fn contain_pillarboxes_when_source_is_tall() {
        let mapped = map_image_object_fit(
            rect(0.0, 0.0, 200.0, 100.0),
            (100, 200),
            ViewportFit::Contain,
        )
        .unwrap();
        assert_eq!(mapped.draw_rect, rect(75.0, 0.0, 50.0, 100.0));
        assert_eq!(mapped.uv, UvRect::FULL);
    }

    #[test]
    fn cover_center_crops_by_adjusting_uv() {
        let mapped =
            map_image_object_fit(rect(0.0, 0.0, 100.0, 100.0), (200, 100), ViewportFit::Cover)
                .unwrap();
        assert_eq!(mapped.draw_rect, rect(0.0, 0.0, 100.0, 100.0));
        assert!((mapped.uv.u0 - 0.25).abs() <= 1.0e-6);
        assert!((mapped.uv.u1 - 0.75).abs() <= 1.0e-6);
        assert!((mapped.uv.v0 - 0.0).abs() <= 1.0e-6);
        assert!((mapped.uv.v1 - 1.0).abs() <= 1.0e-6);
    }

    #[test]
    fn cover_handles_tall_images() {
        let mapped =
            map_image_object_fit(rect(0.0, 0.0, 200.0, 100.0), (100, 200), ViewportFit::Cover)
                .unwrap();
        assert_eq!(mapped.draw_rect, rect(0.0, 0.0, 200.0, 100.0));
        assert!((mapped.uv.u0 - 0.0).abs() <= 1.0e-6);
        assert!((mapped.uv.u1 - 1.0).abs() <= 1.0e-6);
        assert!((mapped.uv.v0 - 0.375).abs() <= 1.0e-6);
        assert!((mapped.uv.v1 - 0.625).abs() <= 1.0e-6);
    }

    #[test]
    fn cover_uv_is_clamped_and_monotonic() {
        let mapped =
            map_image_object_fit(rect(0.0, 0.0, 100.0, 100.0), (1, 1), ViewportFit::Cover).unwrap();

        assert!((0.0..=1.0).contains(&mapped.uv.u0));
        assert!((0.0..=1.0).contains(&mapped.uv.v0));
        assert!((0.0..=1.0).contains(&mapped.uv.u1));
        assert!((0.0..=1.0).contains(&mapped.uv.v1));
        assert!(mapped.uv.u0 <= mapped.uv.u1);
        assert!(mapped.uv.v0 <= mapped.uv.v1);
    }

    #[test]
    fn degenerate_inputs_return_none() {
        assert!(
            map_image_object_fit(rect(0.0, 0.0, 0.0, 10.0), (10, 10), ViewportFit::Cover).is_none()
        );
        assert!(
            map_image_object_fit(rect(0.0, 0.0, 10.0, 10.0), (0, 10), ViewportFit::Cover).is_none()
        );
    }
}
