use crate::embla::limit::Limit;
use crate::embla::utils::array_last;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollContainOption {
    None,
    TrimSnaps,
    KeepSnaps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollContainLimit {
    pub min: usize,
    pub max: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScrollContainOutput {
    pub snaps_contained: Vec<f32>,
    pub scroll_contain_limit: ScrollContainLimit,
}

/// Ported from Embla `ScrollContain`.
///
/// Upstream: `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollContain.ts`
pub fn scroll_contain(
    view_size: f32,
    content_size: f32,
    snaps_aligned: &[f32],
    contain_scroll: ScrollContainOption,
    pixel_tolerance: f32,
) -> ScrollContainOutput {
    let view_size = view_size.max(0.0);
    let content_size = content_size.max(0.0);

    let scroll_bounds = Limit::new(-content_size + view_size, 0.0);
    let snaps_bounded = snaps_bounded(scroll_bounds, snaps_aligned, pixel_tolerance.max(0.0));
    let scroll_contain_limit = scroll_contain_limit(&snaps_bounded);
    let snaps_contained = snaps_contained(
        scroll_bounds,
        &snaps_bounded,
        scroll_contain_limit,
        view_size,
        content_size,
        contain_scroll,
        pixel_tolerance.max(0.0),
    );

    ScrollContainOutput {
        snaps_contained,
        scroll_contain_limit,
    }
}

fn snaps_bounded(scroll_bounds: Limit, snaps_aligned: &[f32], pixel_tolerance: f32) -> Vec<f32> {
    let len = snaps_aligned.len();
    if len == 0 {
        return Vec::new();
    }

    let use_pixel_tolerance = |bound: f32, snap: f32| -> bool {
        if pixel_tolerance <= 0.0 {
            return false;
        }
        // Upstream uses `deltaAbs(bound, snap) <= 1` when pixel tolerance is enabled.
        (bound - snap).abs() <= 1.0
    };

    let mut out = Vec::with_capacity(len);
    for (index, snap_aligned) in snaps_aligned.iter().copied().enumerate() {
        let snap = scroll_bounds.clamp(snap_aligned);
        let is_first = index == 0;
        let is_last = index + 1 == len;

        let bounded = if is_first {
            scroll_bounds.max
        } else if is_last {
            scroll_bounds.min
        } else if use_pixel_tolerance(scroll_bounds.min, snap) {
            scroll_bounds.min
        } else if use_pixel_tolerance(scroll_bounds.max, snap) {
            scroll_bounds.max
        } else {
            snap
        };

        // Embla uses `toFixed(3)` as a last step.
        out.push((bounded * 1000.0).round() / 1000.0);
    }
    out
}

fn scroll_contain_limit(snaps_bounded: &[f32]) -> ScrollContainLimit {
    if snaps_bounded.is_empty() {
        return ScrollContainLimit { min: 0, max: 0 };
    }

    let start_snap = snaps_bounded[0];
    let end_snap = array_last(snaps_bounded);

    let mut min = 0usize;
    for (idx, snap) in snaps_bounded.iter().copied().enumerate() {
        if snap == start_snap {
            min = idx;
        }
    }

    let mut max = snaps_bounded.len();
    for (idx, snap) in snaps_bounded.iter().copied().enumerate() {
        if snap == end_snap {
            max = idx + 1;
            break;
        }
    }

    ScrollContainLimit { min, max }
}

fn snaps_contained(
    scroll_bounds: Limit,
    snaps_bounded: &[f32],
    contain_limit: ScrollContainLimit,
    view_size: f32,
    content_size: f32,
    contain_scroll: ScrollContainOption,
    pixel_tolerance: f32,
) -> Vec<f32> {
    if content_size <= view_size + pixel_tolerance {
        return vec![scroll_bounds.max];
    }

    if contain_scroll == ScrollContainOption::KeepSnaps {
        return snaps_bounded.to_vec();
    }

    snaps_bounded[contain_limit.min..contain_limit.max].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_contain_short_circuits_when_content_fits_view_with_tolerance() {
        let out = scroll_contain(
            320.0,
            321.0,
            &[0.0, -100.0, -200.0],
            ScrollContainOption::TrimSnaps,
            2.0,
        );
        assert_eq!(out.snaps_contained, vec![0.0]);
    }

    #[test]
    fn scroll_contain_keep_snaps_preserves_bounded_list() {
        let out = scroll_contain(
            320.0,
            520.0,
            &[0.0, -10.0, -200.0],
            ScrollContainOption::KeepSnaps,
            2.0,
        );

        assert_eq!(out.snaps_contained.len(), 3);
        assert_eq!(out.snaps_contained[0], 0.0);
        assert_eq!(out.snaps_contained[2], -200.0);
    }

    #[test]
    fn scroll_contain_trim_snaps_slices_between_edge_duplicates() {
        // `snapsAligned` includes multiple snaps clamped to `max` (0). The contain limit should use
        // the last `0` as the start of the contained list.
        let out = scroll_contain(
            320.0,
            820.0,
            &[0.0, 0.1, -0.4, -400.0, -500.0],
            ScrollContainOption::TrimSnaps,
            2.0,
        );

        // First bounded snap is max=0, last bounded snap is min=-500.
        assert_eq!(out.snaps_contained[0], 0.0);
        assert_eq!(array_last(&out.snaps_contained), -500.0);
        assert!(out.scroll_contain_limit.min <= out.scroll_contain_limit.max);
    }
}
