//! Headless carousel drag/snap state machine.
//!
//! This module intentionally does not depend on `fret-ui` so it can be reused across composition
//! layers without pulling in runtime/rendering contracts.

use fret_core::{Axis, Point, Px};

pub const DEFAULT_DRAG_THRESHOLD_PX: f32 = 10.0;
pub const DEFAULT_SNAP_THRESHOLD_FRACTION: f32 = 0.25;
pub const DEFAULT_TOUCH_SCROLL_LOCK_THRESHOLD_PX: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarouselDragConfig {
    pub drag_threshold_px: f32,
    pub snap_threshold_fraction: f32,
    pub touch_prevent_scroll: bool,
    pub touch_scroll_lock_threshold_px: f32,
}

impl Default for CarouselDragConfig {
    fn default() -> Self {
        Self {
            drag_threshold_px: DEFAULT_DRAG_THRESHOLD_PX,
            snap_threshold_fraction: DEFAULT_SNAP_THRESHOLD_FRACTION,
            touch_prevent_scroll: true,
            touch_scroll_lock_threshold_px: DEFAULT_TOUCH_SCROLL_LOCK_THRESHOLD_PX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselDragInputKind {
    Mouse,
    Touch,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CarouselDragState {
    pub armed: bool,
    pub dragging: bool,
    pub start: Point,
    pub start_offset: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarouselDragMoveOutput {
    pub steal_capture: bool,
    pub consumed: bool,
    pub next_offset: Option<Px>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarouselDragReleaseOutput {
    pub next_index: usize,
    pub target_offset: Px,
}

#[inline]
fn axis_delta(axis: Axis, from: Point, to: Point) -> f32 {
    match axis {
        Axis::Horizontal => to.x.0 - from.x.0,
        Axis::Vertical => to.y.0 - from.y.0,
    }
}

#[inline]
fn cross_axis(axis: Axis) -> Axis {
    match axis {
        Axis::Horizontal => Axis::Vertical,
        Axis::Vertical => Axis::Horizontal,
    }
}

pub fn on_pointer_down(
    state: &mut CarouselDragState,
    button_left: bool,
    position: Point,
    start_offset: Px,
) {
    if !button_left {
        return;
    }

    state.armed = true;
    state.dragging = false;
    state.start = position;
    state.start_offset = start_offset;
}

pub fn on_pointer_move(
    config: CarouselDragConfig,
    state: &mut CarouselDragState,
    axis: Axis,
    position: Point,
    buttons_left: bool,
    input_kind: CarouselDragInputKind,
    max_offset: Px,
) -> CarouselDragMoveOutput {
    if !state.armed && !state.dragging {
        return CarouselDragMoveOutput {
            steal_capture: false,
            consumed: false,
            next_offset: None,
        };
    }

    if !buttons_left {
        *state = CarouselDragState::default();
        return CarouselDragMoveOutput {
            steal_capture: false,
            consumed: false,
            next_offset: None,
        };
    }

    if config.touch_prevent_scroll
        && input_kind == CarouselDragInputKind::Touch
        && state.armed
        && !state.dragging
    {
        let primary_abs = axis_delta(axis, state.start, position).abs();
        let cross_abs = axis_delta(cross_axis(axis), state.start, position).abs();
        if primary_abs.max(cross_abs) >= config.touch_scroll_lock_threshold_px
            && primary_abs <= cross_abs
        {
            *state = CarouselDragState::default();
            return CarouselDragMoveOutput {
                steal_capture: false,
                consumed: false,
                next_offset: None,
            };
        }
    }

    let delta = axis_delta(axis, state.start, position);
    if !state.dragging && state.armed && delta.abs() < config.drag_threshold_px {
        return CarouselDragMoveOutput {
            steal_capture: false,
            consumed: false,
            next_offset: None,
        };
    }

    let mut steal_capture = false;
    if !state.dragging && state.armed {
        steal_capture = true;
        state.armed = false;
        state.dragging = true;
    }

    let next = Px((state.start_offset.0 - delta).clamp(0.0, max_offset.0));
    CarouselDragMoveOutput {
        steal_capture,
        consumed: true,
        next_offset: Some(next),
    }
}

pub fn on_pointer_cancel(state: &mut CarouselDragState) -> bool {
    if !state.armed && !state.dragging {
        return false;
    }

    *state = CarouselDragState::default();
    true
}

pub fn on_pointer_up(
    config: CarouselDragConfig,
    state: &mut CarouselDragState,
    axis: Axis,
    position: Point,
    extent: Px,
    items_len: usize,
) -> Option<CarouselDragReleaseOutput> {
    if !state.dragging {
        state.armed = false;
        state.dragging = false;
        return None;
    }

    let max_index = items_len.saturating_sub(1);
    let start_index = if extent.0 > 0.0 {
        (state.start_offset.0 / extent.0)
            .round()
            .clamp(0.0, max_index as f32) as usize
    } else {
        0
    };

    let delta = axis_delta(axis, state.start, position);
    let mut next_index = start_index;
    if extent.0 > 0.0 {
        let threshold = extent.0 * config.snap_threshold_fraction;
        if delta.abs() > threshold {
            if delta > 0.0 {
                next_index = start_index.saturating_sub(1);
            } else {
                next_index = (start_index + 1).min(max_index);
            }
        }
    }

    let target_offset = if extent.0 > 0.0 {
        Px((next_index as f32) * extent.0)
    } else {
        Px(0.0)
    };

    *state = CarouselDragState::default();
    Some(CarouselDragReleaseOutput {
        next_index,
        target_offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_threshold_arms_then_starts_drag() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(0.0));
        assert!(state.armed);
        assert!(!state.dragging);

        let out = on_pointer_move(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            Point::new(Px(9.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(100.0),
        );
        assert_eq!(
            out,
            CarouselDragMoveOutput {
                steal_capture: false,
                consumed: false,
                next_offset: None
            }
        );

        let out = on_pointer_move(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            Point::new(Px(10.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(100.0),
        );
        assert!(out.steal_capture);
        assert!(out.consumed);
        assert!(out.next_offset.is_some());
        assert!(!state.armed);
        assert!(state.dragging);
    }

    #[test]
    fn move_clamps_offset_to_bounds() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(50.0));
        let out = on_pointer_move(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            Point::new(Px(-50.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(60.0),
        );
        assert_eq!(out.next_offset, Some(Px(60.0)));
    }

    #[test]
    fn release_snaps_by_fractional_threshold() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(100.0));
        let _ = on_pointer_move(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            Point::new(Px(-20.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(400.0),
        );

        let release = on_pointer_up(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            Point::new(Px(-30.0), Px(0.0)),
            Px(100.0),
            5,
        )
        .expect("release");
        assert_eq!(release.next_index, 2usize);
        assert_eq!(release.target_offset, Px(200.0));
    }

    #[test]
    fn touch_cross_axis_movement_cancels_armed_drag() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(0.0));

        let out = on_pointer_move(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            Point::new(Px(1.0), Px(5.0)),
            true,
            CarouselDragInputKind::Touch,
            Px(100.0),
        );
        assert_eq!(
            out,
            CarouselDragMoveOutput {
                steal_capture: false,
                consumed: false,
                next_offset: None
            }
        );
        assert_eq!(state, CarouselDragState::default());
    }
}
