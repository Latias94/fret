//! Headless carousel drag/snap state machine.
//!
//! This module intentionally does not depend on `fret-ui` so it can be reused across composition
//! layers without pulling in runtime/rendering contracts.

use fret_core::{Axis, LayoutDirection, Point, Px};

use crate::snap_points as headless_snap_points;

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
fn axis_direction_sign(axis: Axis, direction: LayoutDirection) -> f32 {
    match (axis, direction) {
        (Axis::Horizontal, LayoutDirection::Rtl) => -1.0,
        _ => 1.0,
    }
}

#[inline]
fn axis_delta_with_direction(
    axis: Axis,
    direction: LayoutDirection,
    from: Point,
    to: Point,
) -> f32 {
    axis_delta(axis, from, to) * axis_direction_sign(axis, direction)
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
    direction: LayoutDirection,
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
        let primary_abs = axis_delta_with_direction(axis, direction, state.start, position).abs();
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

    let delta = axis_delta_with_direction(axis, direction, state.start, position);
    // Embla sets `preventClick` when `diffScroll > dragThreshold` (strictly greater).
    // Mirror that boundary by starting a drag only once we exceed the threshold.
    if !state.dragging && state.armed && delta.abs() <= config.drag_threshold_px {
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
    direction: LayoutDirection,
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

    let delta = axis_delta_with_direction(axis, direction, state.start, position);
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

pub fn on_pointer_up_with_snaps(
    config: CarouselDragConfig,
    state: &mut CarouselDragState,
    axis: Axis,
    direction: LayoutDirection,
    position: Point,
    snaps: &[Px],
) -> Option<CarouselDragReleaseOutput> {
    if !state.dragging {
        state.armed = false;
        state.dragging = false;
        return None;
    }

    if snaps.is_empty() {
        *state = CarouselDragState::default();
        return Some(CarouselDragReleaseOutput {
            next_index: 0,
            target_offset: Px(0.0),
        });
    }

    let start_offset = state.start_offset;
    let (start_index, start_snap) = snaps
        .iter()
        .copied()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (a.0 - start_offset.0)
                .abs()
                .total_cmp(&(b.0 - start_offset.0).abs())
        })
        .expect("non-empty snaps");

    let delta = axis_delta_with_direction(axis, direction, state.start, position);
    let mut next_index = start_index;

    if snaps.len() > 1 {
        let neighbor = if delta > 0.0 {
            start_index.checked_sub(1)
        } else if delta < 0.0 {
            Some((start_index + 1).min(snaps.len().saturating_sub(1)))
        } else {
            None
        };

        if let Some(neighbor_index) = neighbor {
            let neighbor_snap = snaps[neighbor_index];
            let distance = (neighbor_snap.0 - start_snap.0).abs();
            let threshold = distance * config.snap_threshold_fraction;
            if distance > 0.0 && delta.abs() > threshold {
                next_index = neighbor_index;
            }
        }
    }

    let target_offset = snaps[next_index];

    *state = CarouselDragState::default();
    Some(CarouselDragReleaseOutput {
        next_index,
        target_offset,
    })
}

pub fn on_pointer_up_with_snaps_options(
    config: CarouselDragConfig,
    state: &mut CarouselDragState,
    axis: Axis,
    direction: LayoutDirection,
    position: Point,
    snaps: &[Px],
    max_offset: Px,
    loop_enabled: bool,
    skip_snaps: bool,
    drag_free: bool,
) -> Option<CarouselDragReleaseOutput> {
    if !state.dragging {
        state.armed = false;
        state.dragging = false;
        return None;
    }

    if snaps.is_empty() {
        *state = CarouselDragState::default();
        return Some(CarouselDragReleaseOutput {
            next_index: 0,
            target_offset: Px(0.0),
        });
    }

    let start_offset = state.start_offset;
    let start_index = headless_snap_points::closest_index_px(snaps, start_offset).unwrap_or(0);
    let start_index = start_index.min(snaps.len().saturating_sub(1));
    let start_snap = snaps[start_index];

    let delta = axis_delta_with_direction(axis, direction, state.start, position);
    let projected_offset = Px(start_offset.0 - delta);
    let projected_offset = clamp_px(projected_offset, Px(0.0), max_offset);
    let projected_offset = round_3(projected_offset);

    let (next_index, target_offset) = if drag_free {
        let ix =
            headless_snap_points::closest_index_px(snaps, projected_offset).unwrap_or(start_index);
        (ix.min(snaps.len().saturating_sub(1)), projected_offset)
    } else if skip_snaps {
        let ix =
            headless_snap_points::closest_index_px(snaps, projected_offset).unwrap_or(start_index);
        let ix = ix.min(snaps.len().saturating_sub(1));
        (ix, snaps[ix])
    } else {
        let mut next_index = start_index;
        if snaps.len() > 1 {
            let neighbor = if delta > 0.0 {
                if loop_enabled {
                    headless_snap_points::step_index_wrapped(snaps.len(), start_index, -1)
                } else {
                    start_index.checked_sub(1)
                }
            } else if delta < 0.0 {
                if loop_enabled {
                    headless_snap_points::step_index_wrapped(snaps.len(), start_index, 1)
                } else {
                    Some((start_index + 1).min(snaps.len().saturating_sub(1)))
                }
            } else {
                None
            };

            if let Some(neighbor_index) = neighbor {
                let neighbor_snap = snaps[neighbor_index];
                let distance = (neighbor_snap.0 - start_snap.0).abs();
                let threshold = distance * config.snap_threshold_fraction;
                if distance > 0.0 && delta.abs() > threshold {
                    next_index = neighbor_index;
                }
            }
        }

        (next_index, snaps[next_index])
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselSlidesToScrollOption {
    Auto,
    Fixed(usize),
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselContainScrollOption {
    None,
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

#[derive(Debug, Clone, PartialEq)]
pub struct CarouselSnapModel1D {
    pub snaps_px: Vec<Px>,
    pub slides_by_snap: Vec<Vec<usize>>,
    pub snap_by_slide: Vec<usize>,
    pub max_offset_px: Px,
}

fn group_by_number<T: Clone>(items: &[T], group_size: usize) -> Vec<Vec<T>> {
    let group_size = group_size.max(1);
    let mut groups = Vec::new();
    let mut i = 0usize;
    while i < items.len() {
        groups.push(items[i..items.len().min(i + group_size)].to_vec());
        i += group_size;
    }
    groups
}

fn group_slide_indexes_auto(
    view_size: Px,
    slides: &[CarouselSlide1D],
    start_gap: Px,
    end_gap: Px,
    loop_enabled: bool,
    pixel_tolerance_px: f32,
) -> Vec<Vec<usize>> {
    if slides.is_empty() {
        return Vec::new();
    }

    let mut boundaries: Vec<usize> = Vec::new();
    for (index, rect_b) in (0..slides.len()).enumerate() {
        let rect_a = boundaries.last().copied().unwrap_or(0);
        let is_first = rect_a == 0;
        let is_last = rect_b + 1 == slides.len();

        let a = slides[rect_a];
        let b = slides[rect_b];
        let gap_a = if !loop_enabled && is_first {
            start_gap
        } else {
            Px(0.0)
        };
        let gap_b = if !loop_enabled && is_last {
            end_gap
        } else {
            Px(0.0)
        };

        let chunk_size = Px((b.end().0 + gap_b.0) - (a.start.0 + gap_a.0)).0.abs();

        if index != 0 && chunk_size > view_size.0 + pixel_tolerance_px {
            boundaries.push(rect_b);
        }
        if is_last {
            boundaries.push(slides.len());
        }
    }

    let mut groups = Vec::new();
    for (index, &end) in boundaries.iter().enumerate() {
        let start = boundaries.get(index.wrapping_sub(1)).copied().unwrap_or(0);
        let start = start.min(end);
        groups.push((start..end).collect::<Vec<_>>());
    }
    groups
}

fn group_slide_indexes(
    view_size: Px,
    slides: &[CarouselSlide1D],
    slides_to_scroll: CarouselSlidesToScrollOption,
    start_gap: Px,
    end_gap: Px,
    loop_enabled: bool,
    pixel_tolerance_px: f32,
) -> Vec<Vec<usize>> {
    match slides_to_scroll {
        CarouselSlidesToScrollOption::Fixed(n) => {
            group_by_number(&(0..slides.len()).collect::<Vec<_>>(), n)
        }
        CarouselSlidesToScrollOption::Auto => group_slide_indexes_auto(
            view_size,
            slides,
            start_gap,
            end_gap,
            loop_enabled,
            pixel_tolerance_px,
        ),
    }
}

fn array_from_range(end: usize, start: usize) -> Vec<usize> {
    (start..=end).collect()
}

/// Headless, deterministic snap model inspired by Embla's `ScrollSnaps`, `SlidesToScroll`,
/// and `ScrollContain`, expressed using Fret's preferred positive-offset convention.
///
/// Coordinate / sign conventions:
///
/// - All `Px` values are in the carousel's **main axis** (X for horizontal, Y for vertical).
/// - `slides[i].start` is measured from the viewport's start edge at rest.
/// - `snaps_px[k]` is the **positive** offset you apply when rendering, e.g.:
///   `transform: translate(-snaps_px[k])`.
/// - `0` means "unshifted track" (content starts at the viewport start edge).
///
/// Inputs:
///
/// - `start_gap` / `end_gap` represent extra blank space before/after slides (e.g. margins).
/// - `slides_to_scroll` controls snap grouping:
///   - `Fixed(n)`: group slides in fixed-size chunks.
///   - `Auto`: group as many slides as fit into `view_size` (with tolerance).
/// - `align` computes the within-viewport alignment for each group (start/center/end/custom).
/// - `contain_scroll`:
///   - `None`: do not clamp/contain snaps (may return snap values outside `[0, max_offset_px]`).
///   - `KeepSnaps`: clamp snaps, but preserve the original snap count (duplicates allowed).
///   - `TrimSnaps`: clamp snaps, then trim duplicates at the ends and expand edge slide groups so
///     every slide maps to a valid snap.
/// - `pixel_tolerance_px`:
///   - short-circuits to a single snap when `content_size <= view_size + tolerance`.
///   - participates in auto-grouping boundaries and containment "near edge" snapping.
///
/// Outputs / invariants:
///
/// - `snaps_px` is never empty. (Empty inputs return `[0]`.)
/// - `max_offset_px` is `max(content_size - view_size, 0)`.
/// - When containment is enabled (`contain_scroll != None` and `!loop_enabled`):
///   - `snaps_px[0] == 0`, and the last snap is `max_offset_px`.
///   - Middle snaps are clamped and rounded to 3 decimals (Embla-style determinism).
/// - `slides_by_snap.len() == snaps_px.len()` (after containment trimming/adjustments).
/// - For every slide index `i`, `snap_by_slide[i]` is a valid index into `snaps_px`.
pub fn snap_model_1d(
    view_size: Px,
    slides: &[CarouselSlide1D],
    start_gap: Px,
    end_gap: Px,
    slides_to_scroll: CarouselSlidesToScrollOption,
    loop_enabled: bool,
    align: CarouselSnapAlign,
    contain_scroll: CarouselContainScrollOption,
    pixel_tolerance_px: f32,
) -> CarouselSnapModel1D {
    let slide_count = slides.len();
    if slide_count == 0 {
        return CarouselSnapModel1D {
            snaps_px: vec![Px(0.0)],
            slides_by_snap: vec![Vec::new()],
            snap_by_slide: Vec::new(),
            max_offset_px: Px(0.0),
        };
    }

    let content_size = slides
        .iter()
        .map(|s| s.end())
        .fold(Px(0.0), |a, b| Px(a.0.max(b.0)));
    let content_size = Px(content_size.0 + end_gap.0.max(0.0));
    let max_offset_px = Px((content_size.0 - view_size.0).max(0.0));

    if content_size.0 <= view_size.0 + pixel_tolerance_px {
        let all = (0..slide_count).collect::<Vec<_>>();
        return CarouselSnapModel1D {
            snaps_px: vec![Px(0.0)],
            slides_by_snap: vec![all.clone()],
            snap_by_slide: vec![0; slide_count],
            max_offset_px,
        };
    }

    let contain_snaps =
        !loop_enabled && !matches!(contain_scroll, CarouselContainScrollOption::None);

    let slide_groups = group_slide_indexes(
        view_size,
        slides,
        slides_to_scroll,
        start_gap,
        end_gap,
        loop_enabled,
        pixel_tolerance_px,
    );

    let mut group_sizes = Vec::with_capacity(slide_groups.len());
    for group in &slide_groups {
        if group.is_empty() {
            group_sizes.push(Px(0.0));
            continue;
        }
        let first = slides[group[0]];
        let last = slides[*group.last().expect("group last")];
        group_sizes.push(Px((last.end().0 - first.start.0).abs()));
    }

    let mut alignments = Vec::with_capacity(group_sizes.len());
    for (i, size) in group_sizes.iter().copied().enumerate() {
        alignments.push(align.measure(view_size, size, i));
    }

    let snaps_unaligned = slides.iter().map(|s| s.start).collect::<Vec<_>>();
    let mut snaps_aligned = Vec::with_capacity(slide_groups.len());
    for (group_index, group) in slide_groups.iter().enumerate() {
        let first_slide_ix = group.first().copied().unwrap_or(0);
        let snap = snaps_unaligned[first_slide_ix];
        snaps_aligned.push(Px(snap.0 - alignments[group_index].0));
    }

    let (snaps_px, contain_limit_min, contain_limit_max) = if contain_snaps {
        let mut snaps_bounded = Vec::with_capacity(snaps_aligned.len());
        for (i, snap) in snaps_aligned.iter().copied().enumerate() {
            let is_first = i == 0;
            let is_last = i + 1 == snaps_aligned.len();
            if is_first {
                snaps_bounded.push(Px(0.0));
                continue;
            }
            if is_last {
                snaps_bounded.push(max_offset_px);
                continue;
            }

            let mut clamped = clamp_px(snap, Px(0.0), max_offset_px);
            if pixel_tolerance_px > 0.0 {
                if (clamped.0 - 0.0).abs() <= 1.0 {
                    clamped = Px(0.0);
                } else if (clamped.0 - max_offset_px.0).abs() <= 1.0 {
                    clamped = max_offset_px;
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

        let snaps_contained = match contain_scroll {
            CarouselContainScrollOption::KeepSnaps => snaps_bounded.clone(),
            CarouselContainScrollOption::TrimSnaps => snaps_bounded[min_ix..max_ix].to_vec(),
            CarouselContainScrollOption::None => snaps_bounded.clone(),
        };
        (snaps_contained, min_ix, max_ix)
    } else {
        (snaps_aligned, 0usize, slide_groups.len())
    };

    let mut slides_by_snap = if snaps_px.len() == 1 {
        vec![(0..slide_count).collect::<Vec<_>>()]
    } else if !contain_snaps || matches!(contain_scroll, CarouselContainScrollOption::KeepSnaps) {
        slide_groups.clone()
    } else {
        let groups = slide_groups[contain_limit_min..contain_limit_max].to_vec();
        groups
            .iter()
            .enumerate()
            .map(|(index, group)| {
                let is_first = index == 0;
                let is_last = index + 1 == groups.len();
                if is_first {
                    let range_end = *group.last().expect("first group last");
                    return array_from_range(range_end, 0);
                }
                if is_last {
                    let range_end = slide_count - 1;
                    return array_from_range(range_end, group[0]);
                }
                group.clone()
            })
            .collect::<Vec<_>>()
    };

    if slides_by_snap.is_empty() {
        slides_by_snap.push((0..slide_count).collect::<Vec<_>>());
    }

    let mut snap_by_slide = vec![0usize; slide_count];
    for (snap_index, group) in slides_by_snap.iter().enumerate() {
        for &slide_index in group {
            if slide_index < snap_by_slide.len() {
                snap_by_slide[slide_index] = snap_index;
            }
        }
    }

    CarouselSnapModel1D {
        snaps_px,
        slides_by_snap,
        snap_by_slide,
        max_offset_px,
    }
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
    let model = snap_model_1d(
        view_size,
        slides,
        Px(0.0),
        end_gap,
        CarouselSlidesToScrollOption::Fixed(1),
        false,
        align,
        match config.contain_scroll {
            CarouselContainScroll::KeepSnaps => CarouselContainScrollOption::KeepSnaps,
            CarouselContainScroll::TrimSnaps => CarouselContainScrollOption::TrimSnaps,
        },
        config.pixel_tolerance_px,
    );
    model.snaps_px
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
            LayoutDirection::Ltr,
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
            LayoutDirection::Ltr,
            Point::new(Px(10.0), Px(0.0)),
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
            LayoutDirection::Ltr,
            Point::new(Px(11.0), Px(0.0)),
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
            LayoutDirection::Ltr,
            Point::new(Px(-50.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(60.0),
        );
        assert_eq!(out.next_offset, Some(Px(60.0)));
    }

    #[test]
    fn move_mirrors_horizontal_delta_in_rtl() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(50.0));

        // In RTL, dragging right should increase the offset (mirror Embla's `direction` sign).
        let out = on_pointer_move(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Rtl,
            Point::new(Px(20.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(100.0),
        );
        assert_eq!(out.next_offset, Some(Px(70.0)));
    }

    #[test]
    fn release_snaps_by_fractional_threshold() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(100.0));
        let _ = on_pointer_move(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-20.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(400.0),
        );

        let release = on_pointer_up(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-30.0), Px(0.0)),
            Px(100.0),
            5,
        )
        .expect("release");
        assert_eq!(release.next_index, 2usize);
        assert_eq!(release.target_offset, Px(200.0));
    }

    #[test]
    fn release_snaps_by_fractional_threshold_with_snaps() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(100.0));

        let _ = on_pointer_move(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-40.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(200.0),
        );

        let snaps = [Px(0.0), Px(100.0), Px(180.0)];
        let release = on_pointer_up_with_snaps(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                snap_threshold_fraction: 0.3,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-40.0), Px(0.0)),
            &snaps,
        )
        .expect("release");

        assert_eq!(release.next_index, 2usize);
        assert_eq!(release.target_offset, Px(180.0));
    }

    #[test]
    fn release_with_skip_snaps_allows_skipping_multiple_snaps() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(100.0));

        let _ = on_pointer_move(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-12.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(300.0),
        );

        let snaps = [Px(0.0), Px(100.0), Px(200.0), Px(300.0)];

        let release_neighbor_only = on_pointer_up_with_snaps_options(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-240.0), Px(0.0)),
            &snaps,
            Px(300.0),
            false,
            false,
            false,
        )
        .expect("release");

        assert_eq!(release_neighbor_only.next_index, 2);
        assert_eq!(release_neighbor_only.target_offset, Px(200.0));

        // Re-arm the drag state for a second release with skipSnaps enabled.
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(100.0));
        let _ = on_pointer_move(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-12.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(300.0),
        );

        let release_skipping = on_pointer_up_with_snaps_options(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-240.0), Px(0.0)),
            &snaps,
            Px(300.0),
            false,
            true,
            false,
        )
        .expect("release");

        assert_eq!(release_skipping.next_index, 3);
        assert_eq!(release_skipping.target_offset, Px(300.0));
    }

    #[test]
    fn release_with_drag_free_settles_to_projected_offset() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(100.0));

        let _ = on_pointer_move(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-12.0), Px(0.0)),
            true,
            CarouselDragInputKind::Mouse,
            Px(300.0),
        );

        let snaps = [Px(0.0), Px(100.0), Px(200.0), Px(300.0)];

        let release = on_pointer_up_with_snaps_options(
            CarouselDragConfig {
                drag_threshold_px: 0.0,
                ..Default::default()
            },
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
            Point::new(Px(-160.0), Px(0.0)),
            &snaps,
            Px(300.0),
            false,
            false,
            true,
        )
        .expect("release");

        assert_eq!(release.next_index, 3);
        assert_eq!(release.target_offset, Px(260.0));
    }

    #[test]
    fn touch_cross_axis_movement_cancels_armed_drag() {
        let mut state = CarouselDragState::default();
        on_pointer_down(&mut state, true, Point::new(Px(0.0), Px(0.0)), Px(0.0));

        let out = on_pointer_move(
            CarouselDragConfig::default(),
            &mut state,
            Axis::Horizontal,
            LayoutDirection::Ltr,
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
    fn snap_model_short_circuits_when_content_fits_view_with_tolerance() {
        let view = Px(100.0);
        let slides = vec![
            CarouselSlide1D {
                start: Px(0.0),
                size: Px(50.0),
            },
            CarouselSlide1D {
                start: Px(50.0),
                size: Px(50.0),
            },
        ];

        // Exact fit.
        let model = snap_model_1d(
            view,
            &slides,
            Px(0.0),
            Px(0.0),
            CarouselSlidesToScrollOption::Fixed(1),
            false,
            CarouselSnapAlign::Start,
            CarouselContainScrollOption::TrimSnaps,
            0.0,
        );
        assert_eq!(model.snaps_px, vec![Px(0.0)]);
        assert_eq!(model.slides_by_snap, vec![vec![0, 1]]);
        assert_eq!(model.snap_by_slide, vec![0, 0]);
        assert_eq!(model.max_offset_px, Px(0.0));

        // Slightly exceeds view, but within tolerance.
        let model = snap_model_1d(
            view,
            &slides,
            Px(0.0),
            Px(0.5),
            CarouselSlidesToScrollOption::Fixed(1),
            false,
            CarouselSnapAlign::Start,
            CarouselContainScrollOption::TrimSnaps,
            1.0,
        );
        assert_eq!(model.snaps_px, vec![Px(0.0)]);
        assert_eq!(model.slides_by_snap, vec![vec![0, 1]]);
        assert_eq!(model.snap_by_slide, vec![0, 0]);
        assert_eq!(model.max_offset_px, Px(0.5));
    }

    #[test]
    fn snap_model_fixed_slides_to_scroll_groups_slides_by_n() {
        let view = Px(150.0);
        let slides = (0..5)
            .map(|i| CarouselSlide1D {
                start: Px((i as f32) * 100.0),
                size: Px(100.0),
            })
            .collect::<Vec<_>>();

        let model = snap_model_1d(
            view,
            &slides,
            Px(0.0),
            Px(0.0),
            CarouselSlidesToScrollOption::Fixed(2),
            false,
            CarouselSnapAlign::Start,
            CarouselContainScrollOption::None,
            0.0,
        );

        assert_eq!(model.snaps_px, vec![Px(0.0), Px(200.0), Px(400.0)]);
        assert_eq!(model.slides_by_snap, vec![vec![0, 1], vec![2, 3], vec![4]]);
        assert_eq!(model.snap_by_slide, vec![0, 0, 1, 1, 2]);
        assert_eq!(model.max_offset_px, Px(350.0));
    }

    #[test]
    fn snap_model_auto_slides_to_scroll_groups_by_view_size() {
        // Three slides of 40px each => first two fit in a 100px view, third becomes its own group.
        let view = Px(100.0);
        let slides = (0..3)
            .map(|i| CarouselSlide1D {
                start: Px((i as f32) * 40.0),
                size: Px(40.0),
            })
            .collect::<Vec<_>>();

        let model = snap_model_1d(
            view,
            &slides,
            Px(0.0),
            Px(0.0),
            CarouselSlidesToScrollOption::Auto,
            false,
            CarouselSnapAlign::Start,
            CarouselContainScrollOption::None,
            0.0,
        );

        assert_eq!(model.snaps_px, vec![Px(0.0), Px(80.0)]);
        assert_eq!(model.slides_by_snap, vec![vec![0, 1], vec![2]]);
        assert_eq!(model.snap_by_slide, vec![0, 0, 1]);
        assert_eq!(model.max_offset_px, Px(20.0));
    }

    fn fixture_keep_trim_none_1() -> (Px, Vec<CarouselSlide1D>) {
        let view = Px(250.0);
        let slides = vec![
            CarouselSlide1D {
                start: Px(0.0),
                size: Px(100.0),
            },
            CarouselSlide1D {
                start: Px(100.0),
                size: Px(100.0),
            },
            CarouselSlide1D {
                start: Px(200.0),
                size: Px(100.0),
            },
            CarouselSlide1D {
                start: Px(300.0),
                size: Px(100.0),
            },
        ];
        (view, slides)
    }

    #[test]
    fn snap_model_contain_scroll_keep_snaps_preserves_count_and_duplicates() {
        let (view, slides) = fixture_keep_trim_none_1();
        let model = snap_model_1d(
            view,
            &slides,
            Px(0.0),
            Px(0.0),
            CarouselSlidesToScrollOption::Fixed(1),
            false,
            CarouselSnapAlign::Start,
            CarouselContainScrollOption::KeepSnaps,
            0.0,
        );

        // max_offset = 400 - 250 = 150. Slides at 200/300 clamp to 150, keeping duplicates.
        assert_eq!(
            model.snaps_px,
            vec![Px(0.0), Px(100.0), Px(150.0), Px(150.0)]
        );
        assert_eq!(
            model.slides_by_snap,
            vec![vec![0], vec![1], vec![2], vec![3]]
        );
        assert_eq!(model.snap_by_slide, vec![0, 1, 2, 3]);
        assert_eq!(model.max_offset_px, Px(150.0));
    }

    #[test]
    fn snap_model_contain_scroll_trim_snaps_trims_and_expands_edge_groups() {
        let (view, slides) = fixture_keep_trim_none_1();
        let model = snap_model_1d(
            view,
            &slides,
            Px(0.0),
            Px(0.0),
            CarouselSlidesToScrollOption::Fixed(1),
            false,
            CarouselSnapAlign::Start,
            CarouselContainScrollOption::TrimSnaps,
            0.0,
        );

        assert_eq!(model.snaps_px, vec![Px(0.0), Px(100.0), Px(150.0)]);
        assert_eq!(model.slides_by_snap, vec![vec![0], vec![1], vec![2, 3]]);
        assert_eq!(model.snap_by_slide, vec![0, 1, 2, 2]);
        assert_eq!(model.max_offset_px, Px(150.0));
    }

    #[test]
    fn snap_model_contain_scroll_none_does_not_clamp_snaps() {
        let (view, slides) = fixture_keep_trim_none_1();
        let model = snap_model_1d(
            view,
            &slides,
            Px(0.0),
            Px(0.0),
            CarouselSlidesToScrollOption::Fixed(1),
            false,
            CarouselSnapAlign::Start,
            CarouselContainScrollOption::None,
            0.0,
        );

        assert_eq!(
            model.snaps_px,
            vec![Px(0.0), Px(100.0), Px(200.0), Px(300.0)]
        );
        assert_eq!(
            model.slides_by_snap,
            vec![vec![0], vec![1], vec![2], vec![3]]
        );
        assert_eq!(model.snap_by_slide, vec![0, 1, 2, 3]);
        assert_eq!(model.max_offset_px, Px(150.0));
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
