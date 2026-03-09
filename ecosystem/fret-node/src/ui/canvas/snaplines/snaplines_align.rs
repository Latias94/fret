use fret_core::Rect;

use super::{SnapGuides, SnapResult};

fn anchors_x(rect: Rect) -> [f32; 3] {
    let x0 = rect.origin.x.0;
    let x1 = rect.origin.x.0 + rect.size.width.0;
    let xc = 0.5 * (x0 + x1);
    [x0, xc, x1]
}

fn anchors_y(rect: Rect) -> [f32; 3] {
    let y0 = rect.origin.y.0;
    let y1 = rect.origin.y.0 + rect.size.height.0;
    let yc = 0.5 * (y0 + y1);
    [y0, yc, y1]
}

pub(crate) fn snap_delta_for_rects(
    moving: Rect,
    candidates: &[Rect],
    threshold: f32,
) -> SnapResult {
    let moving_x = anchors_x(moving);
    let moving_y = anchors_y(moving);
    let threshold = threshold.max(0.0);

    let mut best_x: Option<(f32, u8, f32, f32)> = None;
    let mut best_y: Option<(f32, u8, f32, f32)> = None;

    for rect in candidates {
        let candidate_x = anchors_x(*rect);
        let candidate_y = anchors_y(*rect);

        for i in 0..3 {
            let priority = if i == 1 { 0 } else { 1 };

            let delta_x = candidate_x[i] - moving_x[i];
            let abs_x = delta_x.abs();
            if abs_x <= threshold {
                match best_x {
                    Some((best_abs_x, best_priority, _, _))
                        if (best_abs_x, best_priority) <= (abs_x, priority) => {}
                    _ => best_x = Some((abs_x, priority, delta_x, candidate_x[i])),
                }
            }

            let delta_y = candidate_y[i] - moving_y[i];
            let abs_y = delta_y.abs();
            if abs_y <= threshold {
                match best_y {
                    Some((best_abs_y, best_priority, _, _))
                        if (best_abs_y, best_priority) <= (abs_y, priority) => {}
                    _ => best_y = Some((abs_y, priority, delta_y, candidate_y[i])),
                }
            }
        }
    }

    SnapResult {
        delta_x: best_x.map(|(_, _, delta, _)| delta).unwrap_or(0.0),
        delta_y: best_y.map(|(_, _, delta, _)| delta).unwrap_or(0.0),
        guides: SnapGuides {
            x: best_x.map(|(_, _, _, position)| position),
            y: best_y.map(|(_, _, _, position)| position),
        },
    }
}
