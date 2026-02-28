/// Slide translation helper for `loop=true`.
///
/// This is not a full port of Embla's `SlideLooper.ts` (which includes "slides that fit gap" logic),
/// but it preserves the key observable outcome needed by recipes:
/// when the scroll location wraps, a subset of slides are translated by `±content_size` so the
/// viewport remains visually continuous.
///
/// Upstream reference:
/// - `repo-ref/embla-carousel/packages/embla-carousel/src/components/SlideLooper.ts`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slide1D {
    pub start: f32,
    pub size: f32,
}

impl Slide1D {
    #[inline]
    fn center(&self) -> f32 {
        self.start + self.size * 0.5
    }
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

    let viewport_center = axis_offset + view_size * 0.5;
    slides
        .iter()
        .map(|slide| {
            let center = slide.center();
            let candidates = [0.0, content_size, -content_size];
            let mut best = 0.0;
            let mut best_dist = f32::INFINITY;
            for &candidate in &candidates {
                let dist = (center + candidate - viewport_center).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best = candidate;
                }
            }
            best
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn translates_last_slide_backward_near_start() {
        let slides = (0..5)
            .map(|i| Slide1D {
                start: i as f32 * 100.0,
                size: 100.0,
            })
            .collect::<Vec<_>>();

        // offset slightly before start (virtual), we expect the last slide to be shifted left.
        let out = compute_slide_translates(&slides, 20.0, 500.0, 100.0);
        assert_eq!(out.len(), 5);
        assert_eq!(out[4], -500.0);
        assert_eq!(out[0], 0.0);
    }
}
