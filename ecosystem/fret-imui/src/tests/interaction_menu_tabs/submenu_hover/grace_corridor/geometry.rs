use super::*;

use fret_ui_kit::primitives::menu::pointer_grace_intent;

pub(super) fn find_grace_corridor_transition_points(
    reference: Rect,
    sibling: Rect,
    floating: Rect,
) -> Option<(Point, Point)> {
    let geometry = pointer_grace_intent::PointerGraceIntentGeometry {
        reference,
        floating,
    };
    let reference_right = reference.origin.x.0 + reference.size.width.0;
    let sibling_right = sibling.origin.x.0 + sibling.size.width.0;
    let sibling_bottom = sibling.origin.y.0 + sibling.size.height.0;

    for exit_y in (reference.origin.y.0.floor() as i32)..=(sibling_bottom.ceil() as i32) {
        for exit_x in
            (reference.origin.x.0.floor() as i32)..=((reference_right + 24.0).ceil() as i32)
        {
            let exit = Point::new(Px(exit_x as f32), Px(exit_y as f32));
            if reference.contains(exit) || sibling.contains(exit) || floating.contains(exit) {
                continue;
            }

            let Some(intent) =
                pointer_grace_intent::grace_intent_from_exit_point(exit, geometry, Px(5.0))
            else {
                continue;
            };

            for y in (sibling.origin.y.0.floor() as i32)..=(sibling_bottom.ceil() as i32) {
                for x in (sibling.origin.x.0.floor() as i32)..=(sibling_right.ceil() as i32) {
                    let candidate = Point::new(Px(x as f32), Px(y as f32));
                    if !sibling.contains(candidate) {
                        continue;
                    }

                    let moving_towards = match intent.side {
                        pointer_grace_intent::GraceSide::Right => candidate.x.0 > exit.x.0,
                        pointer_grace_intent::GraceSide::Left => candidate.x.0 < exit.x.0,
                    };
                    if moving_towards
                        && pointer_grace_intent::is_pointer_in_grace_area(candidate, intent)
                    {
                        return Some((exit, candidate));
                    }
                }
            }
        }
    }

    None
}

pub(super) fn find_safe_hover_corridor_points(
    search_bounds: Rect,
    reference: Rect,
    floating: Rect,
    buffer: Px,
) -> Option<(Point, Point)> {
    let geometry = pointer_grace_intent::PointerGraceIntentGeometry {
        reference,
        floating,
    };

    let mut safe_point: Option<Point> = None;
    for y in (0..=search_bounds.size.height.0 as i32).step_by(2) {
        for x in (0..=search_bounds.size.width.0 as i32).step_by(2) {
            let pos = Point::new(Px(x as f32), Px(y as f32));
            if pos.x.0 <= reference.origin.x.0 + reference.size.width.0 {
                continue;
            }
            if reference.contains(pos) || floating.contains(pos) {
                continue;
            }
            if !pointer_grace_intent::last_pointer_is_safe(pos, geometry, buffer) {
                continue;
            }
            safe_point = Some(pos);
            break;
        }
        if safe_point.is_some() {
            break;
        }
    }

    let safe_point = safe_point?;
    let mut unsafe_point: Option<Point> = None;
    for y in (0..=search_bounds.size.height.0 as i32).step_by(4) {
        for x in (0..=search_bounds.size.width.0 as i32).step_by(4) {
            let pos = Point::new(Px(x as f32), Px(y as f32));
            if pos.x.0 >= safe_point.x.0 {
                continue;
            }
            if reference.contains(pos) || floating.contains(pos) {
                continue;
            }
            if pointer_grace_intent::last_pointer_is_safe(pos, geometry, buffer) {
                continue;
            }
            unsafe_point = Some(pos);
            break;
        }
        if unsafe_point.is_some() {
            break;
        }
    }

    unsafe_point.map(|unsafe_point| (unsafe_point, safe_point))
}
