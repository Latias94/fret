/// Slide translation helper for `loop=true`.
///
/// This is a partial port of Embla's `SlideLooper.ts` and `SlideSizes.ts` logic, adapted to a
/// headless model (no DOM, no CSS margins). The intent is to match Embla's *observable outcomes*:
///
/// - Only a subset of slides are eligible for looping at each edge ("slides that fit gap").
/// - Eligibility is derived from per-slide sizes + gaps (start-to-start deltas).
/// - `canLoop` can be used by integrators to downgrade `loop=true` when the content cannot
///   physically loop.
///
/// Upstream reference:
/// - `repo-ref/embla-carousel/packages/embla-carousel/src/components/SlideLooper.ts`
/// - `repo-ref/embla-carousel/packages/embla-carousel/src/components/SlideSizes.ts`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slide1D {
    pub start: f32,
    pub size: f32,
}

impl Slide1D {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SlideBound1D {
    start: f32,
    end: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LoopPoint1D {
    index: usize,
    loop_point: f32,
    initial: f32,
    altered: f32,
}

fn slide_sizes(slides: &[Slide1D]) -> Vec<f32> {
    slides.iter().map(|s| s.size.abs()).collect()
}

fn slide_sizes_with_gaps(slides: &[Slide1D]) -> Vec<f32> {
    if slides.is_empty() {
        return Vec::new();
    }

    slides
        .iter()
        .enumerate()
        .map(|(index, slide)| {
            let is_first = index == 0;
            let is_last = index + 1 == slides.len();
            if is_first {
                slide.size.abs() + slide.start.abs()
            } else if is_last {
                slide.size.abs()
            } else {
                (slides[index + 1].start - slide.start).abs()
            }
        })
        .collect()
}

fn snaps_from_slides(slides: &[Slide1D]) -> Vec<f32> {
    slides.iter().map(|s| -s.start).collect()
}

fn get_remaining_gap_after_slides(
    slide_sizes_with_gaps: &[f32],
    indexes: &[usize],
    from: f32,
) -> f32 {
    indexes.iter().copied().fold(from, |remaining, index| {
        remaining - slide_sizes_with_gaps[index]
    })
}

fn get_slides_that_fit_gap(
    slide_sizes_with_gaps: &[f32],
    indexes: &[usize],
    gap: f32,
) -> Vec<usize> {
    let mut slides_that_fit = Vec::new();
    for &index in indexes {
        let remaining_gap =
            get_remaining_gap_after_slides(slide_sizes_with_gaps, &slides_that_fit, gap);
        if remaining_gap > 0.0 {
            slides_that_fit.push(index);
        }
    }
    slides_that_fit
}

fn get_slide_bounds(
    snaps: &[f32],
    slide_sizes: &[f32],
    view_size: f32,
    offset: f32,
) -> Vec<SlideBound1D> {
    let rounding_safety = 0.5;
    snaps
        .iter()
        .enumerate()
        .map(|(index, &snap)| SlideBound1D {
            start: snap - slide_sizes[index] + rounding_safety + offset,
            end: snap + view_size - rounding_safety + offset,
        })
        .collect()
}

fn get_loop_points(
    snaps: &[f32],
    slide_sizes: &[f32],
    view_size: f32,
    content_size: f32,
    indexes: &[usize],
    offset: f32,
    is_end_edge: bool,
) -> Vec<LoopPoint1D> {
    let slide_bounds = get_slide_bounds(snaps, slide_sizes, view_size, offset);

    indexes
        .iter()
        .copied()
        .map(|index| {
            let initial = if is_end_edge { 0.0 } else { -content_size };
            let altered = if is_end_edge { content_size } else { 0.0 };
            let loop_point = if is_end_edge {
                slide_bounds[index].end
            } else {
                slide_bounds[index].start
            };
            LoopPoint1D {
                index,
                loop_point,
                initial,
                altered,
            }
        })
        .collect()
}

fn loop_points(slides: &[Slide1D], view_size: f32, content_size: f32) -> Vec<LoopPoint1D> {
    let slide_sizes = slide_sizes(slides);
    let slide_sizes_with_gaps = slide_sizes_with_gaps(slides);
    let snaps = snaps_from_slides(slides);
    let scroll_snaps0 = snaps.first().copied().unwrap_or(0.0);

    let asc_items = (0..slides.len()).collect::<Vec<_>>();
    let mut desc_items = asc_items.clone();
    desc_items.reverse();

    let start_gap = scroll_snaps0;
    let start_indexes = get_slides_that_fit_gap(&slide_sizes_with_gaps, &desc_items, start_gap);
    let mut points = get_loop_points(
        &snaps,
        &slide_sizes,
        view_size,
        content_size,
        &start_indexes,
        content_size,
        false,
    );

    let end_gap = view_size - scroll_snaps0 - 1.0;
    let end_indexes = get_slides_that_fit_gap(&slide_sizes_with_gaps, &asc_items, end_gap);
    points.extend(get_loop_points(
        &snaps,
        &slide_sizes,
        view_size,
        content_size,
        &end_indexes,
        -content_size,
        true,
    ));

    points
}

/// Returns `true` if the slide set can loop in the current `view_size`.
///
/// This mirrors Embla's `slideLooper.canLoop()` downgrade logic (used by `EmblaCarousel.ts` to
/// disable loop when content cannot physically loop).
pub fn can_loop(slides: &[Slide1D], view_size: f32) -> bool {
    if slides.len() <= 1 || view_size <= 0.0 {
        return false;
    }

    let slide_sizes_with_gaps = slide_sizes_with_gaps(slides);
    let scroll_snaps0 = snaps_from_slides(slides).first().copied().unwrap_or(0.0);

    let asc_items = (0..slides.len()).collect::<Vec<_>>();
    let mut desc_items = asc_items.clone();
    desc_items.reverse();

    let mut loop_point_indexes = Vec::new();
    loop_point_indexes.extend(get_slides_that_fit_gap(
        &slide_sizes_with_gaps,
        &desc_items,
        scroll_snaps0,
    ));
    loop_point_indexes.extend(get_slides_that_fit_gap(
        &slide_sizes_with_gaps,
        &asc_items,
        view_size - scroll_snaps0 - 1.0,
    ));

    loop_point_indexes.iter().copied().all(|index| {
        let remaining_gap = (0..slides.len())
            .filter(|i| *i != index)
            .fold(view_size, |remaining, i| {
                remaining - slide_sizes_with_gaps[i]
            });
        remaining_gap <= 0.1
    })
}

/// Computes per-slide loop translations in px.
///
/// - `axis_offset` is the current positive scroll offset (i.e. `-location` in Embla space).
/// - `content_size` is the full content length along the main axis (not the max offset).
/// - `view_size` is the viewport length along the main axis.
///
/// Returns a list aligned with `slides` where each entry is an additional translation to apply
/// to that slide (typically `0`, `+content_size`, or `-content_size`).
pub fn compute_slide_translates(
    slides: &[Slide1D],
    axis_offset: f32,
    content_size: f32,
    view_size: f32,
) -> Vec<f32> {
    if slides.is_empty() {
        return Vec::new();
    }
    if content_size <= 0.0 || view_size <= 0.0 {
        return vec![0.0; slides.len()];
    }

    let location = -axis_offset;
    let points = loop_points(slides, view_size, content_size);

    let mut out = vec![0.0; slides.len()];
    for point in points {
        let shift_location = if location > point.loop_point {
            point.initial
        } else {
            point.altered
        };
        out[point.index] = shift_location;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_none_away_from_edges() {
        let slides = (0..5)
            .map(|i| Slide1D {
                start: i as f32 * 100.0,
                size: 100.0,
            })
            .collect::<Vec<_>>();

        let out = compute_slide_translates(&slides, 200.0, 500.0, 100.0);
        assert_eq!(out, vec![0.0; 5]);
    }

    #[test]
    fn translates_first_slide_forward_near_end() {
        let slides = (0..5)
            .map(|i| Slide1D {
                start: i as f32 * 100.0,
                size: 100.0,
            })
            .collect::<Vec<_>>();

        // view=100, content=500, offset near end (showing last slide).
        let out = compute_slide_translates(&slides, 480.0, 500.0, 100.0);
        assert_eq!(out.len(), 5);
        assert_eq!(out[0], 500.0);
        assert_eq!(out[4], 0.0);
    }

    #[test]
    fn translates_multiple_slides_forward_near_end_with_large_view() {
        let slides = (0..5)
            .map(|i| Slide1D {
                start: i as f32 * 100.0,
                size: 100.0,
            })
            .collect::<Vec<_>>();

        // With a large view, multiple slides can be eligible for end-edge recycling.
        let out = compute_slide_translates(&slides, 480.0, 500.0, 320.0);
        assert_eq!(out.len(), 5);
        assert_eq!(out[0], 500.0);
        assert_eq!(out[1], 500.0);
        assert_eq!(out[2], 500.0);
        assert_eq!(out[3], 0.0);
        assert_eq!(out[4], 0.0);
    }

    #[test]
    fn can_loop_true_when_many_slides_exceed_view() {
        let slides = (0..5)
            .map(|i| Slide1D {
                start: i as f32 * 100.0,
                size: 100.0,
            })
            .collect::<Vec<_>>();

        assert!(can_loop(&slides, 320.0));
    }

    #[test]
    fn can_loop_false_when_two_slides_do_not_fit_view() {
        let slides = vec![
            Slide1D {
                start: 0.0,
                size: 200.0,
            },
            Slide1D {
                start: 200.0,
                size: 200.0,
            },
        ];

        assert!(!can_loop(&slides, 320.0));
    }
}
