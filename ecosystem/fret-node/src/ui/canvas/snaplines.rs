//! Alignment guides ("snaplines") for node dragging.
//!
//! This module is UI-light: it computes guide positions and the delta adjustment required to
//! snap a moving rectangle to candidate rectangles.

use fret_core::Rect;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct SnapGuides {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct SnapResult {
    pub delta_x: f32,
    pub delta_y: f32,
    pub guides: SnapGuides,
}

fn anchors_x(r: Rect) -> [f32; 3] {
    let x0 = r.origin.x.0;
    let x1 = r.origin.x.0 + r.size.width.0;
    let xc = 0.5 * (x0 + x1);
    [x0, xc, x1]
}

fn anchors_y(r: Rect) -> [f32; 3] {
    let y0 = r.origin.y.0;
    let y1 = r.origin.y.0 + r.size.height.0;
    let yc = 0.5 * (y0 + y1);
    [y0, yc, y1]
}

pub(crate) fn snap_delta_for_rects(
    moving: Rect,
    candidates: &[Rect],
    threshold: f32,
) -> SnapResult {
    let mx = anchors_x(moving);
    let my = anchors_y(moving);
    let threshold = threshold.max(0.0);

    let mut best_x: Option<(f32, u8, f32, f32)> = None; // (abs, pri, delta, pos)
    let mut best_y: Option<(f32, u8, f32, f32)> = None;

    for r in candidates {
        let cx = anchors_x(*r);
        let cy = anchors_y(*r);

        for i in 0..3 {
            let pri = if i == 1 { 0 } else { 1 };

            let dx = cx[i] - mx[i];
            let ax = dx.abs();
            if ax <= threshold {
                match best_x {
                    Some((best_ax, best_pri, _, _)) if (best_ax, best_pri) <= (ax, pri) => {}
                    _ => best_x = Some((ax, pri, dx, cx[i])),
                }
            }

            let dy = cy[i] - my[i];
            let ay = dy.abs();
            if ay <= threshold {
                match best_y {
                    Some((best_ay, best_pri, _, _)) if (best_ay, best_pri) <= (ay, pri) => {}
                    _ => best_y = Some((ay, pri, dy, cy[i])),
                }
            }
        }
    }

    SnapResult {
        delta_x: best_x.map(|(_, _, d, _)| d).unwrap_or(0.0),
        delta_y: best_y.map(|(_, _, d, _)| d).unwrap_or(0.0),
        guides: SnapGuides {
            x: best_x.map(|(_, _, _, p)| p),
            y: best_y.map(|(_, _, _, p)| p),
        },
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px, Rect, Size};

    use super::snap_delta_for_rects;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn snap_delta_for_rects_snaps_left_edge() {
        let moving = rect(10.0, 10.0, 100.0, 50.0);
        let candidates = [rect(50.0, 0.0, 80.0, 40.0)];
        let r = snap_delta_for_rects(moving, &candidates, 0.5);
        assert_eq!(r.delta_x, 0.0);

        let r = snap_delta_for_rects(moving, &candidates, 40.0);
        assert_eq!(r.delta_x, 20.0);
        assert_eq!(r.guides.x, Some(130.0));
    }

    #[test]
    fn snap_delta_for_rects_snaps_center_y() {
        let moving = rect(0.0, 0.0, 10.0, 10.0); // center y = 5
        let candidates = [rect(100.0, 25.0, 10.0, 10.0)]; // center y = 30
        let r = snap_delta_for_rects(moving, &candidates, 30.0);
        assert_eq!(r.delta_y, 25.0);
        assert_eq!(r.guides.y, Some(30.0));
    }
}
