use fret_core::{Point, Px, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDropZone {
    Left,
    Right,
    Up,
    Down,
}

fn rect_contains_point(rect: Rect, point: Point) -> bool {
    let left = rect.origin.x.0;
    let top = rect.origin.y.0;
    let right = rect.origin.x.0 + rect.size.width.0;
    let bottom = rect.origin.y.0 + rect.size.height.0;
    point.x.0 >= left && point.x.0 <= right && point.y.0 >= top && point.y.0 <= bottom
}

fn edge_distance(rect: Rect, point: Point, zone: EdgeDropZone) -> Px {
    let left = rect.origin.x.0;
    let top = rect.origin.y.0;
    let right = rect.origin.x.0 + rect.size.width.0;
    let bottom = rect.origin.y.0 + rect.size.height.0;

    match zone {
        EdgeDropZone::Left => Px(point.x.0 - left),
        EdgeDropZone::Right => Px(right - point.x.0),
        EdgeDropZone::Up => Px(point.y.0 - top),
        EdgeDropZone::Down => Px(bottom - point.y.0),
    }
}

fn nearest_edge_within_margin(rect: Rect, point: Point, margin: Px) -> Option<EdgeDropZone> {
    if margin.0 <= 0.0 || !margin.0.is_finite() {
        return None;
    }
    if !rect_contains_point(rect, point) {
        return None;
    }

    let candidates = [
        EdgeDropZone::Up,
        EdgeDropZone::Right,
        EdgeDropZone::Down,
        EdgeDropZone::Left,
    ];

    let mut best: Option<(EdgeDropZone, f32)> = None;
    for edge in candidates {
        let dist = edge_distance(rect, point, edge).0;
        if dist < 0.0 || !dist.is_finite() || dist > margin.0 {
            continue;
        }
        best = Some(match best {
            Some((_prev_edge, prev_dist)) if prev_dist <= dist => (best.unwrap().0, prev_dist),
            _ => (edge, dist),
        });
    }

    best.map(|(edge, _)| edge)
}

/// Compute a nearest-edge drop zone for "drag to split" style interactions.
///
/// - Returns `None` when `point` is not inside `rect` or not inside the edge margin.
/// - When multiple edges are inside the margin (corners), picks the nearest edge.
/// - `previous` + `hysteresis` provide stability to avoid flicker between adjacent edges.
pub fn compute_edge_drop_zone(
    rect: Rect,
    point: Point,
    margin: Px,
    previous: Option<EdgeDropZone>,
    hysteresis: Px,
) -> Option<EdgeDropZone> {
    let candidate = nearest_edge_within_margin(rect, point, margin);
    let Some(previous) = previous else {
        return candidate;
    };

    let h = if hysteresis.0.is_finite() {
        hysteresis.0.max(0.0)
    } else {
        0.0
    };
    if h <= 0.0 {
        return candidate;
    }

    let prev_dist = edge_distance(rect, point, previous).0;
    if !prev_dist.is_finite() {
        return candidate;
    }

    match candidate {
        None => {
            // Sticky release: keep the previous zone a bit longer while moving away from the edge.
            (prev_dist <= margin.0 + h).then_some(previous)
        }
        Some(edge) if edge == previous => Some(edge),
        Some(edge) => {
            let cand_dist = edge_distance(rect, point, edge).0;
            if !cand_dist.is_finite() {
                return Some(previous);
            }

            // Sticky switch: only switch when the new edge is clearly closer.
            if cand_dist + h < prev_dist {
                Some(edge)
            } else if prev_dist <= margin.0 + h {
                Some(previous)
            } else {
                Some(edge)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn returns_none_when_not_inside_margin() {
        let r = rect(0.0, 0.0, 100.0, 80.0);
        let p = Point::new(Px(50.0), Px(40.0));
        assert_eq!(nearest_edge_within_margin(r, p, Px(10.0)), None);
    }

    #[test]
    fn picks_nearest_edge_inside_margin() {
        let r = rect(0.0, 0.0, 100.0, 80.0);
        let margin = Px(20.0);

        let left = Point::new(Px(5.0), Px(40.0));
        assert_eq!(
            nearest_edge_within_margin(r, left, margin),
            Some(EdgeDropZone::Left)
        );

        let up = Point::new(Px(50.0), Px(5.0));
        assert_eq!(
            nearest_edge_within_margin(r, up, margin),
            Some(EdgeDropZone::Up)
        );

        let right = Point::new(Px(95.0), Px(10.0));
        assert_eq!(
            nearest_edge_within_margin(r, right, margin),
            Some(EdgeDropZone::Right)
        );

        let down = Point::new(Px(50.0), Px(78.0));
        assert_eq!(
            nearest_edge_within_margin(r, down, margin),
            Some(EdgeDropZone::Down)
        );
    }

    #[test]
    fn hysteresis_prefers_previous_when_distances_are_close() {
        let r = rect(0.0, 0.0, 100.0, 80.0);
        let margin = Px(20.0);
        let hysteresis = Px(8.0);

        // In the top-left corner, the nearest edge may flip between `Up` and `Left`.
        // With hysteresis, we expect it to keep the previous edge unless the new edge is clearly closer.
        let p = Point::new(Px(6.0), Px(6.0));
        assert_eq!(
            compute_edge_drop_zone(r, p, margin, Some(EdgeDropZone::Up), hysteresis),
            Some(EdgeDropZone::Up)
        );
        assert_eq!(
            compute_edge_drop_zone(r, p, margin, Some(EdgeDropZone::Left), hysteresis),
            Some(EdgeDropZone::Left)
        );
    }
}
