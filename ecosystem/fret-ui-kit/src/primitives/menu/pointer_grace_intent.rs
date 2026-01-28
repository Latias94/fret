//! Pointer grace intent (Radix Menu submenu safe-hover outcomes).
//!
//! Radix Menu uses a “pointer grace intent” to avoid closing submenus while the pointer moves
//! diagonally from a trigger into the submenu content. In the web implementation this is tracked
//! via pointer direction + a polygon area. In Fret we keep the behavior outcome:
//!
//! - treat “moving towards submenu” + “inside corridor” as safe
//! - cancel a pending close timer when safe
//! - arm a close-delay timer when unsafe
//!
//! This module intentionally does not decide *what* to close when the timer fires; that remains
//! component policy.

use std::time::Duration;

use fret_core::{Point, Px, Rect};
use fret_runtime::{Model, TimerToken};
use fret_ui::action::{ActionCx, PointerMoveCx, UiActionHost};

use crate::headless::safe_hover::safe_hover_contains;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerGraceIntentGeometry {
    pub reference: Rect,
    pub floating: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerGraceIntentConfig {
    pub buffer: Px,
    pub close_delay: Duration,
}

impl PointerGraceIntentConfig {
    pub fn new(buffer: Px, close_delay: Duration) -> Self {
        Self {
            buffer,
            close_delay,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraceSide {
    Left,
    Right,
}

pub type Polygon = [Point; 5];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraceIntent {
    pub area: Polygon,
    pub side: GraceSide,
}

pub fn grace_side(geometry: PointerGraceIntentGeometry) -> Option<GraceSide> {
    let reference_left = geometry.reference.origin.x.0;
    let reference_right = reference_left + geometry.reference.size.width.0;
    let reference_center = (reference_left + reference_right) / 2.0;

    let floating_left = geometry.floating.origin.x.0;
    let floating_right = floating_left + geometry.floating.size.width.0;

    if floating_left >= reference_right {
        return Some(GraceSide::Right);
    }
    if floating_right <= reference_left {
        return Some(GraceSide::Left);
    }

    // Allow a small horizontal overlap as long as the floating panel is predominantly on one side
    // of the reference. This matches typical menu submenu layouts that slightly overlap to avoid
    // visible seams between panels.
    if floating_left >= reference_center {
        return Some(GraceSide::Right);
    }
    if floating_right <= reference_center {
        return Some(GraceSide::Left);
    }
    None
}

pub fn pointer_dir(prev: Point, next: Point) -> Option<GraceSide> {
    if next.x.0 > prev.x.0 {
        Some(GraceSide::Right)
    } else if next.x.0 < prev.x.0 {
        Some(GraceSide::Left)
    } else {
        None
    }
}

fn is_point_in_polygon(point: Point, polygon: &Polygon) -> bool {
    let x = point.x.0;
    let y = point.y.0;
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let xi = polygon[i].x.0;
        let yi = polygon[i].y.0;
        let xj = polygon[j].x.0;
        let yj = polygon[j].y.0;

        let intersect = (yi > y) != (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi;
        if intersect {
            inside = !inside;
        }
        j = i;
    }

    inside
}

pub fn is_pointer_in_grace_area(point: Point, intent: GraceIntent) -> bool {
    is_point_in_polygon(point, &intent.area)
}

pub fn grace_intent_from_exit_point(
    exit: Point,
    geometry: PointerGraceIntentGeometry,
    bleed: Px,
) -> Option<GraceIntent> {
    let side = grace_side(geometry)?;

    let floating = geometry.floating;
    let floating_left = floating.origin.x.0;
    let floating_right = floating_left + floating.size.width.0;
    let floating_top = floating.origin.y.0;
    let floating_bottom = floating_top + floating.size.height.0;

    let (near_edge, far_edge, exit_x) = match side {
        GraceSide::Right => (floating_left, floating_right, exit.x.0 - bleed.0),
        GraceSide::Left => (floating_right, floating_left, exit.x.0 + bleed.0),
    };

    Some(GraceIntent {
        area: [
            Point::new(Px(exit_x), exit.y),
            Point::new(Px(near_edge), Px(floating_top)),
            Point::new(Px(far_edge), Px(floating_top)),
            Point::new(Px(far_edge), Px(floating_bottom)),
            Point::new(Px(near_edge), Px(floating_bottom)),
        ],
        side,
    })
}

fn is_pointer_moving_to_submenu(
    prev: Option<Point>,
    next: Point,
    geometry: PointerGraceIntentGeometry,
    buffer: Px,
) -> bool {
    let Some(side) = grace_side(geometry) else {
        return false;
    };

    let moving_towards = match prev {
        None => true,
        Some(prev) => pointer_dir(prev, next).is_none_or(|dir| dir == side),
    };

    moving_towards && safe_hover_contains(next, geometry.reference, geometry.floating, buffer)
}

/// Drive the “submenu close-delay” timer from pointer-move events.
///
/// Returns `true` when it changes the timer state.
pub fn drive_close_timer_on_pointer_move(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    mv: PointerMoveCx,
    geometry: Option<PointerGraceIntentGeometry>,
    config: PointerGraceIntentConfig,
    last_pointer: &Model<Option<Point>>,
    close_timer: &Model<Option<TimerToken>>,
) -> bool {
    let Some(geometry) = geometry else {
        let _ = host
            .models_mut()
            .update(last_pointer, |v| *v = Some(mv.position));
        return false;
    };

    let prev = host.models_mut().read(last_pointer, |v| *v).ok().flatten();
    let _ = host
        .models_mut()
        .update(last_pointer, |v| *v = Some(mv.position));

    let safe = is_pointer_moving_to_submenu(prev, mv.position, geometry, config.buffer);

    let pending = host.models_mut().read(close_timer, |v| *v).ok().flatten();
    if safe {
        let Some(token) = pending else {
            return false;
        };
        host.push_effect(fret_runtime::Effect::CancelTimer { token });
        let _ = host.models_mut().update(close_timer, |v| *v = None);
        host.request_redraw(acx.window);
        return true;
    }

    if pending.is_some() {
        return false;
    }

    let token = host.next_timer_token();
    host.push_effect(fret_runtime::Effect::SetTimer {
        window: Some(acx.window),
        token,
        after: config.close_delay,
        repeat: None,
    });
    let _ = host.models_mut().update(close_timer, |v| *v = Some(token));
    host.request_redraw(acx.window);
    true
}

pub fn last_pointer_is_safe(
    pointer: Point,
    geometry: PointerGraceIntentGeometry,
    buffer: Px,
) -> bool {
    safe_hover_contains(pointer, geometry.reference, geometry.floating, buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::Size;

    #[test]
    fn grace_side_classifies_overlapping_submenu_as_right() {
        let reference = Rect::new(Point::new(Px(10.0), Px(0.0)), Size::new(Px(20.0), Px(10.0)));
        // Overlaps by 2px but is mostly to the right.
        let floating = Rect::new(Point::new(Px(28.0), Px(0.0)), Size::new(Px(20.0), Px(10.0)));
        let geometry = PointerGraceIntentGeometry {
            reference,
            floating,
        };
        assert_eq!(grace_side(geometry), Some(GraceSide::Right));
    }

    #[test]
    fn last_pointer_is_safe_matches_geometry_corridor() {
        let reference = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(20.0), Px(2.0)), Size::new(Px(10.0), Px(10.0)));
        let geometry = PointerGraceIntentGeometry {
            reference,
            floating,
        };

        assert!(last_pointer_is_safe(
            Point::new(Px(12.0), Px(5.0)),
            geometry,
            Px(0.0)
        ));
        assert!(!last_pointer_is_safe(
            Point::new(Px(12.0), Px(30.0)),
            geometry,
            Px(0.0)
        ));
    }

    #[test]
    fn moving_away_is_not_considered_safe() {
        let reference = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(20.0), Px(2.0)), Size::new(Px(10.0), Px(10.0)));
        let geometry = PointerGraceIntentGeometry {
            reference,
            floating,
        };

        let prev = Some(Point::new(Px(13.0), Px(5.0)));
        let next = Point::new(Px(12.0), Px(5.0)); // moving left, away from right-side submenu
        assert!(!is_pointer_moving_to_submenu(prev, next, geometry, Px(0.0)));
    }
}
