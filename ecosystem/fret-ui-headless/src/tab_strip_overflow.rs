use fret_core::geometry::{Px, Rect};

/// Computes which tabs are overflowed (clipped) by a scroll viewport.
///
/// Returns indices into `tabs` (stable / caller-owned identity).
pub fn compute_overflowed_tab_indices<T>(
    tabs: &[T],
    mut rect_for_tab: impl FnMut(&T) -> Option<Rect>,
    viewport: Rect,
    margin: Px,
) -> Vec<usize> {
    let view_left = viewport.origin.x.0 + margin.0;
    let view_right = viewport.origin.x.0 + viewport.size.width.0 - margin.0;
    let epsilon = 0.5f32;

    tabs.iter()
        .enumerate()
        .filter_map(|(ix, tab)| {
            let rect = rect_for_tab(tab)?;
            let tab_left = rect.origin.x.0;
            let tab_right = rect.origin.x.0 + rect.size.width.0;
            let overflowed = tab_left < view_left - epsilon || tab_right > view_right + epsilon;
            overflowed.then_some(ix)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::geometry::{Point, Size};

    fn rect(x: f32, w: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(0.0)), Size::new(Px(w), Px(10.0)))
    }

    #[test]
    fn overflow_membership_tracks_viewport_and_margin() {
        let tabs = ["a", "b", "c", "d"];
        let rects = [
            rect(0.0, 25.0),
            rect(25.0, 25.0),
            rect(50.0, 25.0),
            rect(75.0, 30.0),
        ];
        let viewport = rect(0.0, 100.0);

        let overflowed = compute_overflowed_tab_indices(
            &tabs,
            |t| tabs.iter().position(|x| x == t).map(|ix| rects[ix]),
            viewport,
            Px(0.0),
        );
        assert_eq!(overflowed, vec![3]);

        let overflowed = compute_overflowed_tab_indices(
            &tabs,
            |t| tabs.iter().position(|x| x == t).map(|ix| rects[ix]),
            viewport,
            Px(10.0),
        );
        assert_eq!(overflowed, vec![0, 3]);
    }

    #[test]
    fn overflow_membership_changes_with_scroll_like_translation() {
        let tabs = ["a", "b", "c", "d"];
        let rects = [
            rect(-40.0, 25.0),
            rect(-15.0, 25.0),
            rect(10.0, 25.0),
            rect(35.0, 30.0),
        ];
        let viewport = rect(0.0, 60.0);

        let overflowed = compute_overflowed_tab_indices(
            &tabs,
            |t| tabs.iter().position(|x| x == t).map(|ix| rects[ix]),
            viewport,
            Px(0.0),
        );
        assert_eq!(overflowed, vec![0, 1, 3]);
    }
}
