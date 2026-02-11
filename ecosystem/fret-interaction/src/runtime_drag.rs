//! Helpers for updating `fret-runtime` drag sessions from pointer gestures.
//!
//! This module is intentionally small and focused:
//! - It does not own pointer capture or focus; callers decide choreography.
//! - It only mutates an existing `DragSession` (position, phase, threshold crossing).
//! - It is reusable across multiple ecosystem surfaces that share `DragSession` semantics.

use fret_core::{AppWindowId, Point};
use fret_runtime::{DragPhase, DragSession};

use crate::drag::DragThreshold;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragMoveOutcome {
    Continue,
    StartedDragging,
    Canceled,
}

/// Updates a `DragSession` for a pointer move, honoring a drag threshold.
///
/// The caller is responsible for:
/// - ensuring this session is the intended one (kind/window checks),
/// - canceling the session in the host when `Canceled` is returned.
pub fn update_thresholded_move(
    drag: &mut DragSession,
    current_window: AppWindowId,
    position: Point,
    left_down: bool,
    threshold: DragThreshold,
) -> DragMoveOutcome {
    drag.current_window = current_window;
    drag.position = position;

    if !left_down {
        drag.phase = DragPhase::Canceled;
        return DragMoveOutcome::Canceled;
    }

    if drag.dragging {
        drag.phase = DragPhase::Dragging;
        return DragMoveOutcome::Continue;
    }

    if threshold.distance_sq_exceeded(drag.start_position, drag.position) {
        drag.dragging = true;
        drag.phase = DragPhase::Dragging;
        return DragMoveOutcome::StartedDragging;
    }

    DragMoveOutcome::Continue
}

/// Updates a `DragSession` for a pointer move, treating the gesture as immediately dragging.
pub fn update_immediate_move(
    drag: &mut DragSession,
    current_window: AppWindowId,
    position: Point,
    left_down: bool,
) -> DragMoveOutcome {
    drag.current_window = current_window;
    drag.position = position;

    if !left_down {
        drag.phase = DragPhase::Canceled;
        return DragMoveOutcome::Canceled;
    }

    let was_dragging = drag.dragging;
    drag.dragging = true;
    drag.phase = DragPhase::Dragging;
    if was_dragging {
        DragMoveOutcome::Continue
    } else {
        DragMoveOutcome::StartedDragging
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{PointerId, Px};
    use fret_runtime::{DragKindId, DragSessionId};

    fn make_session(start: Point) -> DragSession {
        DragSession::new(
            DragSessionId(1),
            PointerId(0),
            AppWindowId::default(),
            DragKindId(123),
            start,
            (),
        )
    }

    #[test]
    fn thresholded_move_starts_dragging_when_threshold_exceeded() {
        let mut s = make_session(Point::new(Px(0.0), Px(0.0)));
        let t = DragThreshold::new(Px(2.0));
        let out = update_thresholded_move(
            &mut s,
            AppWindowId::default(),
            Point::new(Px(2.0), Px(0.0)),
            true,
            t,
        );
        assert_eq!(out, DragMoveOutcome::StartedDragging);
        assert!(s.dragging);
        assert_eq!(s.phase, DragPhase::Dragging);
    }

    #[test]
    fn thresholded_move_cancels_when_left_released() {
        let mut s = make_session(Point::new(Px(0.0), Px(0.0)));
        let out = update_thresholded_move(
            &mut s,
            AppWindowId::default(),
            Point::new(Px(1.0), Px(0.0)),
            false,
            DragThreshold::default(),
        );
        assert_eq!(out, DragMoveOutcome::Canceled);
        assert_eq!(s.phase, DragPhase::Canceled);
    }

    #[test]
    fn immediate_move_sets_dragging_on_first_move() {
        let mut s = make_session(Point::new(Px(0.0), Px(0.0)));
        let out = update_immediate_move(
            &mut s,
            AppWindowId::default(),
            Point::new(Px(0.0), Px(0.0)),
            true,
        );
        assert_eq!(out, DragMoveOutcome::StartedDragging);
        assert!(s.dragging);
        assert_eq!(s.phase, DragPhase::Dragging);
    }
}
