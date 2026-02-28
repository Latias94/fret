use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Px, Rect};

use super::WorkspaceTab;
use crate::tab_drag::WorkspaceTabHitRect;

pub(crate) fn compute_overflowed_tab_ids(
    tabs: &[WorkspaceTab],
    tab_rects: &[WorkspaceTabHitRect],
    viewport: Rect,
    margin: Px,
) -> Vec<Arc<str>> {
    let by_id: HashMap<&str, Rect> = tab_rects.iter().map(|r| (r.id.as_ref(), r.rect)).collect();

    let view_left = viewport.origin.x.0 + margin.0;
    let view_right = viewport.origin.x.0 + viewport.size.width.0 - margin.0;
    let epsilon = 0.5f32;

    tabs.iter()
        .filter_map(|tab| {
            let rect = by_id.get(tab.id.as_ref())?;
            let tab_left = rect.origin.x.0;
            let tab_right = rect.origin.x.0 + rect.size.width.0;
            let overflowed = tab_left < view_left - epsilon || tab_right > view_right + epsilon;
            overflowed.then(|| tab.id.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Size};

    fn rect(x: f32, w: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(0.0)), Size::new(Px(w), Px(10.0)))
    }

    #[test]
    fn overflow_membership_tracks_viewport_and_margin() {
        let tabs = vec![
            WorkspaceTab::new("a", "A", "cmd.a"),
            WorkspaceTab::new("b", "B", "cmd.b"),
            WorkspaceTab::new("c", "C", "cmd.c"),
            WorkspaceTab::new("d", "D", "cmd.d"),
        ];
        let rects = vec![
            WorkspaceTabHitRect {
                id: Arc::from("a"),
                rect: rect(0.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("b"),
                rect: rect(25.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("c"),
                rect: rect(50.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("d"),
                rect: rect(75.0, 30.0),
            },
        ];
        let viewport = rect(0.0, 100.0);

        let overflowed = compute_overflowed_tab_ids(&tabs, &rects, viewport, Px(0.0));
        assert_eq!(
            overflowed,
            vec![Arc::from("d")],
            "expected right-clipped tab to be considered overflowed"
        );

        let overflowed = compute_overflowed_tab_ids(&tabs, &rects, viewport, Px(10.0));
        assert_eq!(
            overflowed,
            vec![Arc::from("a"), Arc::from("d")],
            "expected margin to treat edge-adjacent tabs as overflowed"
        );
    }

    #[test]
    fn overflow_membership_changes_with_scroll_like_translation() {
        let tabs = vec![
            WorkspaceTab::new("a", "A", "cmd.a"),
            WorkspaceTab::new("b", "B", "cmd.b"),
            WorkspaceTab::new("c", "C", "cmd.c"),
            WorkspaceTab::new("d", "D", "cmd.d"),
        ];

        // Simulate a scroll offset shifting all tabs left by 40px.
        let rects = vec![
            WorkspaceTabHitRect {
                id: Arc::from("a"),
                rect: rect(-40.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("b"),
                rect: rect(-15.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("c"),
                rect: rect(10.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("d"),
                rect: rect(35.0, 30.0),
            },
        ];
        let viewport = rect(0.0, 60.0);

        let overflowed = compute_overflowed_tab_ids(&tabs, &rects, viewport, Px(0.0));
        assert_eq!(
            overflowed,
            vec![Arc::from("a"), Arc::from("b"), Arc::from("d")],
            "expected left-clipped and right-clipped tabs to be overflowed"
        );
    }
}
