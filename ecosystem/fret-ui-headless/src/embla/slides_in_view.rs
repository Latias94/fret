use crate::embla::slide_looper::Slide1D;

#[derive(Debug, Clone, PartialEq)]
pub struct SlidesInViewUpdate {
    pub slides_in_view: Vec<usize>,
    pub slides_enter_view: Vec<usize>,
    pub slides_left_view: Vec<usize>,
    pub changed: bool,
}

/// Deterministic, headless approximation of Embla's `SlidesInView` observer.
///
/// Upstream reference:
/// - `repo-ref/embla-carousel/packages/embla-carousel/src/components/SlidesInView.ts`
///
/// Differences:
/// - Embla uses `IntersectionObserver` and reports `isIntersecting` changes.
/// - In headless mode we compute intersection ratios against a 1D viewport interval.
/// - `threshold` is treated as the minimum visible fraction required to count as "in view".
/// - `margin_px` expands the viewport interval on both ends.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SlidesInViewTracker {
    in_view: Vec<bool>,
    generation: u64,
}

impl SlidesInViewTracker {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn update(
        &mut self,
        slides: &[Slide1D],
        axis_offset: f32,
        view_size: f32,
        threshold: f32,
        margin_px: f32,
    ) -> SlidesInViewUpdate {
        let view_size = view_size.max(0.0);
        let margin_px = margin_px.max(0.0);
        let threshold = threshold.clamp(0.0, 1.0);

        if self.in_view.len() != slides.len() {
            self.in_view = vec![false; slides.len()];
        }

        let view_start = axis_offset - margin_px;
        let view_end = axis_offset + view_size + margin_px;

        let mut next_in_view = vec![false; slides.len()];
        for (idx, slide) in slides.iter().enumerate() {
            if slide.size <= 0.0 {
                continue;
            }
            let slide_start = slide.start;
            let slide_end = slide.start + slide.size;
            let intersection = (slide_end.min(view_end) - slide_start.max(view_start)).max(0.0);
            if intersection <= 0.0 {
                continue;
            }
            let ratio = (intersection / slide.size).clamp(0.0, 1.0);
            if ratio >= threshold {
                next_in_view[idx] = true;
            }
        }

        let mut slides_in_view = Vec::new();
        let mut slides_enter_view = Vec::new();
        let mut slides_left_view = Vec::new();
        let mut changed = false;

        for (idx, &now) in next_in_view.iter().enumerate() {
            let was = self.in_view[idx];
            if now {
                slides_in_view.push(idx);
            }
            if now != was {
                changed = true;
                if now {
                    slides_enter_view.push(idx);
                } else {
                    slides_left_view.push(idx);
                }
            }
        }

        if changed {
            self.generation = self.generation.saturating_add(1);
            self.in_view = next_in_view;
        }

        SlidesInViewUpdate {
            slides_in_view,
            slides_enter_view,
            slides_left_view,
            changed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slides_uniform(n: usize, size: f32) -> Vec<Slide1D> {
        (0..n)
            .map(|i| Slide1D {
                start: i as f32 * size,
                size,
            })
            .collect()
    }

    #[test]
    fn threshold_zero_counts_any_intersection() {
        let slides = slides_uniform(3, 100.0);
        let mut tracker = SlidesInViewTracker::default();
        let out = tracker.update(&slides, 50.0, 100.0, 0.0, 0.0);
        assert_eq!(out.slides_in_view, vec![0, 1]);
        assert!(out.changed);
    }

    #[test]
    fn higher_threshold_requires_visible_fraction() {
        let slides = slides_uniform(3, 100.0);
        let mut tracker = SlidesInViewTracker::default();
        let out = tracker.update(&slides, 50.0, 100.0, 0.6, 0.0);
        assert_eq!(out.slides_in_view, Vec::<usize>::new());
        assert!(!out.changed);
    }

    #[test]
    fn margin_expands_viewport() {
        let slides = slides_uniform(3, 100.0);
        let mut tracker = SlidesInViewTracker::default();
        let out = tracker.update(&slides, 100.0, 100.0, 0.0, 20.0);
        assert_eq!(out.slides_in_view, vec![0, 1, 2]);
    }

    #[test]
    fn emits_enter_and_leave_sets() {
        let slides = slides_uniform(3, 100.0);
        let mut tracker = SlidesInViewTracker::default();
        let _ = tracker.update(&slides, 0.0, 100.0, 0.0, 0.0);
        let out = tracker.update(&slides, 100.0, 100.0, 0.0, 0.0);
        assert_eq!(out.slides_in_view, vec![1]);
        assert_eq!(out.slides_enter_view, vec![1]);
        assert_eq!(out.slides_left_view, vec![0]);
        assert!(out.changed);
    }
}
