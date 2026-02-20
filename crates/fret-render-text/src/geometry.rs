use crate::parley_shaper::ShapedCluster;
use fret_core::geometry::Px;

fn utf8_grapheme_boundaries(text: &str) -> Vec<usize> {
    use unicode_segmentation::UnicodeSegmentation as _;

    let mut out: Vec<usize> = Vec::with_capacity(text.chars().count().saturating_add(2));
    out.push(0);
    for (i, _) in text.grapheme_indices(true) {
        out.push(i);
    }
    out.push(text.len());
    out.sort_unstable();
    out.dedup();
    out
}

pub fn caret_stops_for_slice(
    slice: &str,
    base_offset: usize,
    clusters: &[ShapedCluster],
    line_width_px: f32,
    scale: f32,
    kept_end: usize,
) -> Vec<(usize, Px)> {
    let mut out: Vec<(usize, Px)> = Vec::new();
    let boundaries = utf8_grapheme_boundaries(slice);

    if boundaries.is_empty() {
        return vec![(base_offset, Px(0.0))];
    }

    if clusters.is_empty() {
        for &b in &boundaries {
            let idx = base_offset + b;
            if idx > kept_end {
                continue;
            }
            let x = if b >= slice.len() {
                (line_width_px / scale).max(0.0)
            } else {
                0.0
            };
            out.push((idx, Px(x)));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
        out.dedup_by(|a, b| a.0 == b.0);
        return out;
    }

    let last_cluster_end = clusters
        .iter()
        .map(|c| c.text_range.end)
        .max()
        .unwrap_or(0)
        .min(slice.len());
    let effective_line_width_px = clusters
        .iter()
        .flat_map(|c| [c.x0, c.x1])
        .fold(line_width_px, |acc, x| acc.max(x.max(0.0)));

    let mut cluster_i = 0usize;
    for &b in &boundaries {
        let idx = base_offset + b;
        if idx > kept_end {
            continue;
        }

        while cluster_i + 1 < clusters.len() && clusters[cluster_i].text_range.end < b {
            cluster_i = cluster_i.saturating_add(1);
        }

        let x = if b <= clusters[0].text_range.start {
            let first = &clusters[0];
            if first.is_rtl {
                first.x1.max(0.0)
            } else {
                first.x0.max(0.0)
            }
        } else if b > last_cluster_end {
            let last = clusters.last().unwrap_or(&clusters[0]);
            if last.is_rtl {
                0.0
            } else {
                effective_line_width_px
            }
        } else if cluster_i >= clusters.len() {
            let last = clusters.last().unwrap_or(&clusters[0]);
            if last.is_rtl {
                0.0
            } else {
                line_width_px.max(0.0)
            }
        } else {
            let c = &clusters[cluster_i];
            let start = c.text_range.start.min(slice.len());
            let end = c.text_range.end.min(slice.len());

            if start == end {
                c.x0.max(0.0)
            } else if b <= start {
                if c.is_rtl {
                    c.x1.max(0.0)
                } else {
                    c.x0.max(0.0)
                }
            } else if b >= end {
                if c.is_rtl {
                    c.x0.max(0.0)
                } else {
                    c.x1.max(0.0)
                }
            } else {
                let denom = (end - start) as f32;
                let mut t = ((b - start) as f32 / denom).clamp(0.0, 1.0);
                if c.is_rtl {
                    t = 1.0 - t;
                }
                let w = (c.x1 - c.x0).max(0.0);
                (c.x0 + w * t).max(0.0)
            }
        };

        out.push((idx, Px((x / scale).max(0.0))));
    }

    out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
    out.dedup_by(|a, b| a.0 == b.0);
    out
}
