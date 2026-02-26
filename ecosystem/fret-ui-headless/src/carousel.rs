//! Headless carousel drag/snap state machine.
//!
//! This module intentionally does not depend on `fret-ui` so it can be reused across composition
//! layers without pulling in runtime/rendering contracts.

use fret_core::{Axis, Point, Px};

pub const DEFAULT_DRAG_THRESHOLD_PX: f32 = 10.0;
pub const DEFAULT_SNAP_THRESHOLD_FRACTION: f32 = 0.25;
pub const DEFAULT_TOUCH_SCROLL_LOCK_THRESHOLD_PX: f32 = 2.0;
pub const DEFAULT_SCROLL_CONTAIN_PIXEL_TOLERANCE_PX: f32 = 2.0;

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

// -------------------------------------------------------------------------------------------------
// Snap / contain-scroll helpers (Embla-aligned, headless, deterministic)
// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum CarouselSnapAlign {
    Start,
    Center,
    End,
    Custom(fn(view_size: Px, snap_size: Px, index: usize) -> Px),
}

impl CarouselSnapAlign {
    fn measure(self, view_size: Px, snap_size: Px, index: usize) -> Px {
        match self {
            CarouselSnapAlign::Start => Px(0.0),
            CarouselSnapAlign::Center => Px((view_size.0 - snap_size.0) / 2.0),
            CarouselSnapAlign::End => Px(view_size.0 - snap_size.0),
            CarouselSnapAlign::Custom(f) => f(view_size, snap_size, index),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselContainScroll {
    KeepSnaps,
    TrimSnaps,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarouselContainScrollConfig {
    pub contain_scroll: CarouselContainScroll,
    pub pixel_tolerance_px: f32,
}

impl Default for CarouselContainScrollConfig {
    fn default() -> Self {
        Self {
            contain_scroll: CarouselContainScroll::TrimSnaps,
            pixel_tolerance_px: DEFAULT_SCROLL_CONTAIN_PIXEL_TOLERANCE_PX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarouselSlide1D {
    pub start: Px,
    pub size: Px,
}

impl CarouselSlide1D {
    #[inline]
    pub fn end(self) -> Px {
        Px(self.start.0 + self.size.0)
    }
}

#[inline]
fn round_3(px: Px) -> Px {
    Px((px.0 * 1000.0).round() / 1000.0)
}

#[inline]
fn clamp_px(v: Px, min: Px, max: Px) -> Px {
    Px(v.0.clamp(min.0, max.0))
}

/// Compute contained scroll snaps (1D) using Embla's `ScrollSnaps` + `ScrollContain` semantics,
/// expressed in Fret's preferred positive-offset convention:
///
/// - `0` means the track is unshifted (first slide starts at the viewport start edge).
/// - increasing values shift content left (e.g. `transform: translate(-offset)`).
pub fn contained_scroll_snaps_1d(
    view_size: Px,
    slides: &[CarouselSlide1D],
    align: CarouselSnapAlign,
    config: CarouselContainScrollConfig,
) -> Vec<Px> {
    contained_scroll_snaps_1d_with_end_gap(view_size, slides, Px(0.0), align, config)
}

/// `contained_scroll_snaps_1d` with an explicit `end_gap` (e.g. trailing margin).
pub fn contained_scroll_snaps_1d_with_end_gap(
    view_size: Px,
    slides: &[CarouselSlide1D],
    end_gap: Px,
    align: CarouselSnapAlign,
    config: CarouselContainScrollConfig,
) -> Vec<Px> {
    if slides.is_empty() {
        return vec![Px(0.0)];
    }

    let content_size = slides
        .iter()
        .map(|s| s.end())
        .fold(Px(0.0), |a, b| Px(a.0.max(b.0)));
    let content_size = Px(content_size.0 + end_gap.0.max(0.0));

    // Embla: if contentSize <= viewSize + pixelTolerance => [0]
    if content_size.0 <= view_size.0 + config.pixel_tolerance_px {
        return vec![Px(0.0)];
    }

    let max_offset = Px((content_size.0 - view_size.0).max(0.0));

    let mut snaps_aligned = Vec::with_capacity(slides.len());
    for (i, slide) in slides.iter().copied().enumerate() {
        let alignment = align.measure(view_size, slide.size, i);
        snaps_aligned.push(Px(slide.start.0 - alignment.0));
    }

    let mut snaps_bounded = Vec::with_capacity(snaps_aligned.len());
    for (i, snap) in snaps_aligned.iter().copied().enumerate() {
        let is_first = i == 0;
        let is_last = i + 1 == snaps_aligned.len();
        if is_first {
            snaps_bounded.push(Px(0.0));
            continue;
        }
        if is_last {
            snaps_bounded.push(max_offset);
            continue;
        }

        let mut clamped = clamp_px(snap, Px(0.0), max_offset);
        if config.pixel_tolerance_px > 0.0 {
            if (clamped.0 - 0.0).abs() <= 1.0 {
                clamped = Px(0.0);
            } else if (clamped.0 - max_offset.0).abs() <= 1.0 {
                clamped = max_offset;
            }
        }
        snaps_bounded.push(round_3(clamped));
    }

    let start_snap = snaps_bounded[0];
    let end_snap = *snaps_bounded.last().expect("snaps_bounded non-empty");

    let mut min_ix = 0usize;
    for (i, snap) in snaps_bounded.iter().copied().enumerate() {
        if snap == start_snap {
            min_ix = i;
        }
    }
    let mut max_ix = snaps_bounded.len();
    for (i, snap) in snaps_bounded.iter().copied().enumerate() {
        if snap == end_snap {
            max_ix = i + 1;
            break;
        }
    }

    match config.contain_scroll {
        CarouselContainScroll::KeepSnaps => snaps_bounded,
        CarouselContainScroll::TrimSnaps => snaps_bounded[min_ix..max_ix].to_vec(),
    }
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

    fn fixture_contain_scroll_ltr_1() -> (Px, Vec<CarouselSlide1D>) {
        let view = Px(1000.0);
        let slides = vec![
            CarouselSlide1D {
                start: Px(0.0),
                size: Px(100.0),
            },
            CarouselSlide1D {
                start: Px(100.0),
                size: Px(200.0),
            },
            CarouselSlide1D {
                start: Px(300.0),
                size: Px(150.0),
            },
            CarouselSlide1D {
                start: Px(450.0),
                size: Px(250.0),
            },
            CarouselSlide1D {
                start: Px(700.0),
                size: Px(130.0),
            },
            CarouselSlide1D {
                start: Px(830.0),
                size: Px(100.0),
            },
            CarouselSlide1D {
                start: Px(930.0),
                size: Px(200.0),
            },
            CarouselSlide1D {
                start: Px(1130.0),
                size: Px(150.0),
            },
            CarouselSlide1D {
                start: Px(1280.0),
                size: Px(250.0),
            },
            CarouselSlide1D {
                start: Px(1530.0),
                size: Px(130.0),
            },
        ];
        (view, slides)
    }

    fn align_10pct(view_size: Px, _snap_size: Px, _index: usize) -> Px {
        Px(view_size.0 * 0.1)
    }

    #[test]
    fn embla_fixture_ltr_1_trim_snaps_align_start_matches() {
        let (view, slides) = fixture_contain_scroll_ltr_1();
        let snaps = contained_scroll_snaps_1d(
            view,
            &slides,
            CarouselSnapAlign::Start,
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(
            snaps,
            vec![Px(0.0), Px(100.0), Px(300.0), Px(450.0), Px(660.0)]
        );
    }

    #[test]
    fn embla_fixture_ltr_1_trim_snaps_align_center_matches() {
        let (view, slides) = fixture_contain_scroll_ltr_1();
        let snaps = contained_scroll_snaps_1d(
            view,
            &slides,
            CarouselSnapAlign::Center,
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(
            snaps,
            vec![
                Px(0.0),
                Px(75.0),
                Px(265.0),
                Px(380.0),
                Px(530.0),
                Px(660.0)
            ]
        );
    }

    #[test]
    fn embla_fixture_ltr_1_trim_snaps_align_end_matches() {
        let (view, slides) = fixture_contain_scroll_ltr_1();
        let snaps = contained_scroll_snaps_1d(
            view,
            &slides,
            CarouselSnapAlign::End,
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(
            snaps,
            vec![Px(0.0), Px(130.0), Px(280.0), Px(530.0), Px(660.0)]
        );
    }

    #[test]
    fn embla_fixture_ltr_1_trim_snaps_align_custom_matches() {
        let (view, slides) = fixture_contain_scroll_ltr_1();
        let snaps = contained_scroll_snaps_1d(
            view,
            &slides,
            CarouselSnapAlign::Custom(align_10pct),
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(
            snaps,
            vec![Px(0.0), Px(200.0), Px(350.0), Px(600.0), Px(660.0)]
        );
    }

    #[test]
    fn embla_fixture_content_size_within_pixel_tolerance_collapses_to_zero() {
        let view = Px(1000.0);
        let slides = vec![
            CarouselSlide1D {
                start: Px(0.0),
                size: Px(501.0),
            },
            CarouselSlide1D {
                start: Px(501.0),
                size: Px(501.0),
            },
        ];

        let snaps = contained_scroll_snaps_1d(
            view,
            &slides,
            CarouselSnapAlign::Center,
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(snaps, vec![Px(0.0)]);
    }

    #[test]
    fn embla_fixture_content_size_just_outside_pixel_tolerance_keeps_small_edge_snap() {
        let view = Px(1000.0);
        let slides = vec![
            CarouselSlide1D {
                start: Px(0.0),
                size: Px(502.0),
            },
            CarouselSlide1D {
                start: Px(502.0),
                size: Px(501.0),
            },
        ];

        let snaps = contained_scroll_snaps_1d(
            view,
            &slides,
            CarouselSnapAlign::Center,
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(snaps, vec![Px(0.0), Px(3.0)]);
    }

    fn fixture_contain_scroll_ltr_2() -> (Px, Px, Vec<CarouselSlide1D>) {
        let view = Px(1000.0);
        let end_gap = Px(10.0);
        let slides = vec![
            CarouselSlide1D {
                start: Px(10.0),
                size: Px(100.0),
            },
            CarouselSlide1D {
                start: Px(130.0),
                size: Px(200.0),
            },
            CarouselSlide1D {
                start: Px(350.0),
                size: Px(150.0),
            },
            CarouselSlide1D {
                start: Px(520.0),
                size: Px(250.0),
            },
            CarouselSlide1D {
                start: Px(790.0),
                size: Px(130.0),
            },
            CarouselSlide1D {
                start: Px(940.0),
                size: Px(100.0),
            },
            CarouselSlide1D {
                start: Px(1060.0),
                size: Px(200.0),
            },
            CarouselSlide1D {
                start: Px(1280.0),
                size: Px(150.0),
            },
            CarouselSlide1D {
                start: Px(1450.0),
                size: Px(250.0),
            },
            CarouselSlide1D {
                start: Px(1720.0),
                size: Px(130.0),
            },
        ];
        (view, end_gap, slides)
    }

    #[test]
    fn embla_fixture_ltr_2_trim_snaps_align_start_matches() {
        let (view, end_gap, slides) = fixture_contain_scroll_ltr_2();
        let snaps = contained_scroll_snaps_1d_with_end_gap(
            view,
            &slides,
            end_gap,
            CarouselSnapAlign::Start,
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(
            snaps,
            vec![
                Px(0.0),
                Px(130.0),
                Px(350.0),
                Px(520.0),
                Px(790.0),
                Px(860.0)
            ]
        );
    }

    #[test]
    fn embla_fixture_ltr_2_trim_snaps_align_center_matches() {
        let (view, end_gap, slides) = fixture_contain_scroll_ltr_2();
        let snaps = contained_scroll_snaps_1d_with_end_gap(
            view,
            &slides,
            end_gap,
            CarouselSnapAlign::Center,
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(
            snaps,
            vec![
                Px(0.0),
                Px(145.0),
                Px(355.0),
                Px(490.0),
                Px(660.0),
                Px(855.0),
                Px(860.0)
            ]
        );
    }

    #[test]
    fn embla_fixture_ltr_2_trim_snaps_align_end_matches() {
        let (view, end_gap, slides) = fixture_contain_scroll_ltr_2();
        let snaps = contained_scroll_snaps_1d_with_end_gap(
            view,
            &slides,
            end_gap,
            CarouselSnapAlign::End,
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(
            snaps,
            vec![
                Px(0.0),
                Px(40.0),
                Px(260.0),
                Px(430.0),
                Px(700.0),
                Px(860.0)
            ]
        );
    }

    #[test]
    fn embla_fixture_ltr_2_trim_snaps_align_custom_matches() {
        let (view, end_gap, slides) = fixture_contain_scroll_ltr_2();
        let snaps = contained_scroll_snaps_1d_with_end_gap(
            view,
            &slides,
            end_gap,
            CarouselSnapAlign::Custom(align_10pct),
            CarouselContainScrollConfig::default(),
        );
        assert_eq!(
            snaps,
            vec![
                Px(0.0),
                Px(30.0),
                Px(250.0),
                Px(420.0),
                Px(690.0),
                Px(840.0),
                Px(860.0)
            ]
        );
    }
}
