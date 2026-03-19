use crate::parley_shaper::ShapedCluster;
use fret_core::CaretAffinity;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn wrap_grapheme_cut_end(
    text: &str,
    clusters: &[ShapedCluster],
    max_width_px: f32,
) -> usize {
    let mut cut_end = 0usize;
    for c in clusters {
        if c.text_range().start >= text.len() {
            continue;
        }
        if c.x1() > max_width_px + 0.5 {
            break;
        }
        cut_end = cut_end.max(c.text_range().end.min(text.len()));
    }
    cut_end
}

pub(crate) fn first_cluster_end(text: &str, clusters: &[ShapedCluster]) -> usize {
    for c in clusters {
        let end = c.text_range().end.min(text.len());
        if end > 0 {
            return end;
        }
    }
    0
}

pub(crate) fn first_grapheme_end(text: &str) -> usize {
    text.grapheme_indices(true)
        .next()
        .map(|(start, g)| start + g.len())
        .unwrap_or(0)
}

pub(crate) fn cut_end_for_available(
    text: &str,
    clusters: &[ShapedCluster],
    available: f32,
) -> usize {
    let mut cut_end = 0usize;
    for c in clusters {
        if c.x1() <= available + 0.5 {
            cut_end = cut_end.max(c.text_range().end.min(text.len()));
        }
    }
    cut_end
}

pub(crate) fn trim_trailing_whitespace(text: &str, mut end: usize) -> usize {
    while end > 0
        && text
            .as_bytes()
            .get(end.saturating_sub(1))
            .is_some_and(|b| b.is_ascii_whitespace())
    {
        end = end.saturating_sub(1);
    }
    end
}

pub(crate) fn clamp_to_char_boundary(text: &str, mut end: usize) -> usize {
    while end > 0 && !text.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    end
}

pub(crate) fn is_grapheme_boundary(text: &str, idx: usize) -> bool {
    let idx = idx.min(text.len());
    if idx == 0 || idx == text.len() {
        return true;
    }
    text.grapheme_indices(true).any(|(start, _)| start == idx)
}

pub(crate) fn clamp_to_grapheme_boundary_down(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    if is_grapheme_boundary(text, idx) {
        return idx;
    }

    let mut prev = 0usize;
    for (start, _) in text.grapheme_indices(true) {
        if start >= idx {
            break;
        }
        prev = start;
    }
    prev
}

pub(crate) fn clamp_to_grapheme_boundary_up(text: &str, idx: usize) -> usize {
    let idx = idx.min(text.len());
    if is_grapheme_boundary(text, idx) {
        return idx;
    }
    for (start, g) in text.grapheme_indices(true) {
        let end = start + g.len();
        if idx < end {
            return end;
        }
    }
    text.len()
}

pub(crate) fn empty_range_at(idx: usize) -> Range<usize> {
    idx..idx
}

#[allow(dead_code)]
pub(crate) fn hit_test_x(
    clusters: &[ShapedCluster],
    x: f32,
    text_len: usize,
) -> (usize, CaretAffinity) {
    if clusters.is_empty() {
        return (0, CaretAffinity::Downstream);
    }

    if x.is_nan() || x <= clusters[0].x0() {
        return (0, CaretAffinity::Downstream);
    }

    for c in clusters {
        if x > c.x1() {
            continue;
        }
        let text_range = c.text_range();
        if text_range.start == text_range.end {
            return (text_range.start, CaretAffinity::Downstream);
        }
        let mid = c.x0() + (c.x1() - c.x0()) * 0.5;
        let left_half = x <= mid;
        if c.is_rtl() {
            if left_half {
                return (text_range.end, CaretAffinity::Upstream);
            }
            return (text_range.start, CaretAffinity::Downstream);
        }
        if left_half {
            return (text_range.start, CaretAffinity::Downstream);
        }
        return (text_range.end, CaretAffinity::Upstream);
    }

    (text_len, CaretAffinity::Downstream)
}
