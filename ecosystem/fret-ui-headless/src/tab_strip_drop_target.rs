use fret_core::geometry::{Point, Rect};

use crate::tab_strip_surface::{TabStripSurface, classify_tab_strip_surface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabInsertionSide {
    Before,
    After,
}

/// A coarse drop target resolution for editor-grade tab strips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStripDropTarget {
    None,
    Tab {
        index: usize,
        side: TabInsertionSide,
    },
    PinnedBoundary,
    End,
}

/// Computes a tab "drop target" based on midpoint x comparison.
///
/// This matches the common editor-style semantics:
/// - dropping left of a tab midpoint inserts **before** that tab
/// - otherwise, inserts **after** the last tab
///
/// The returned index refers to the input slice index (not a sorted index).
pub fn compute_tab_drop_target_midpoint<T>(
    pointer: Point,
    tabs: &[T],
    mut tab_rect: impl FnMut(&T) -> Rect,
    mut tab_is_dragged: impl FnMut(&T) -> bool,
) -> Option<(usize, TabInsertionSide)> {
    let mut filtered: Vec<(usize, Rect)> = tabs
        .iter()
        .enumerate()
        .filter(|(_, t)| !tab_is_dragged(t))
        .map(|(ix, t)| (ix, tab_rect(t)))
        .collect();

    if filtered.is_empty() {
        return None;
    }

    filtered.sort_by(|a, b| a.1.origin.x.0.total_cmp(&b.1.origin.x.0));

    for (ix, rect) in &filtered {
        let mid_x = rect.origin.x.0 + (rect.size.width.0 * 0.5);
        if pointer.x.0 < mid_x {
            return Some((*ix, TabInsertionSide::Before));
        }
    }

    let (last_ix, _) = filtered.last().copied()?;
    Some((last_ix, TabInsertionSide::After))
}

/// Computes the coarse drop target for a tab strip by combining surface classification with a
/// midpoint-based "tab before/after" selection.
///
/// This is intended to be used by adapters that can provide per-tab bounds (workspace strips,
/// docking tab bars, etc.).
pub fn compute_tab_strip_drop_target_midpoint<T>(
    position: Point,
    tabs: &[T],
    tab_rect: impl FnMut(&T) -> Rect,
    tab_is_dragged: impl FnMut(&T) -> bool,
    pinned_boundary_rect: Option<Rect>,
    header_space_rect: Option<Rect>,
    scroll_viewport_rect: Option<Rect>,
    overflow_control_rect: Option<Rect>,
    scroll_left_control_rect: Option<Rect>,
    scroll_right_control_rect: Option<Rect>,
) -> TabStripDropTarget {
    let mut tab_rect = tab_rect;
    let mut tab_is_dragged = tab_is_dragged;

    let surface = classify_tab_strip_surface(
        position,
        tabs,
        |t| tab_rect(t),
        |t| tab_is_dragged(t),
        pinned_boundary_rect,
        header_space_rect,
        scroll_viewport_rect,
        overflow_control_rect,
        scroll_left_control_rect,
        scroll_right_control_rect,
    );

    match surface {
        TabStripSurface::Outside
        | TabStripSurface::OverflowControl
        | TabStripSurface::ScrollControls => TabStripDropTarget::None,
        TabStripSurface::PinnedBoundary => TabStripDropTarget::PinnedBoundary,
        TabStripSurface::HeaderSpace => TabStripDropTarget::End,
        TabStripSurface::TabsViewport => {
            compute_tab_drop_target_midpoint(position, tabs, |t| tab_rect(t), |t| tab_is_dragged(t))
                .map(|(index, side)| TabStripDropTarget::Tab { index, side })
                .unwrap_or(TabStripDropTarget::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::geometry::{Px, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn drop_before_first_when_left_of_midpoint() {
        let tabs = [rect(0.0, 0.0, 100.0, 20.0), rect(110.0, 0.0, 100.0, 20.0)];
        let pointer = Point::new(Px(10.0), Px(10.0));
        let hit = compute_tab_drop_target_midpoint(pointer, &tabs, |r| *r, |_| false);
        assert_eq!(hit, Some((0, TabInsertionSide::Before)));
    }

    #[test]
    fn drop_after_last_when_right_of_all_midpoints() {
        let tabs = [rect(0.0, 0.0, 100.0, 20.0), rect(110.0, 0.0, 100.0, 20.0)];
        let pointer = Point::new(Px(1000.0), Px(10.0));
        let hit = compute_tab_drop_target_midpoint(pointer, &tabs, |r| *r, |_| false);
        assert_eq!(hit, Some((1, TabInsertionSide::After)));
    }

    #[test]
    fn dragged_tab_is_excluded_from_candidates() {
        let tabs = [rect(0.0, 0.0, 100.0, 20.0), rect(110.0, 0.0, 100.0, 20.0)];
        let pointer = Point::new(Px(10.0), Px(10.0));
        let hit = compute_tab_drop_target_midpoint(pointer, &tabs, |r| *r, |t| *t == tabs[0]);
        assert_eq!(hit, Some((1, TabInsertionSide::Before)));
    }

    #[test]
    fn tab_strip_drop_target_end_wins_in_header_space() {
        let tabs = [rect(0.0, 0.0, 100.0, 20.0), rect(110.0, 0.0, 100.0, 20.0)];
        let viewport = rect(0.0, 0.0, 400.0, 20.0);
        let pos = Point::new(Px(380.0), Px(10.0));

        let target = compute_tab_strip_drop_target_midpoint(
            pos,
            &tabs,
            |r| *r,
            |_| false,
            None,
            None,
            Some(viewport),
            None,
            None,
            None,
        );
        assert_eq!(target, TabStripDropTarget::End);
    }

    #[test]
    fn tab_strip_drop_target_pinned_boundary_takes_precedence() {
        let tabs = [rect(0.0, 0.0, 100.0, 20.0)];
        let viewport = rect(0.0, 0.0, 200.0, 20.0);
        let pinned_boundary = rect(10.0, 0.0, 8.0, 20.0);
        let pos = Point::new(Px(12.0), Px(10.0));

        let target = compute_tab_strip_drop_target_midpoint(
            pos,
            &tabs,
            |r| *r,
            |_| false,
            Some(pinned_boundary),
            None,
            Some(viewport),
            None,
            None,
            None,
        );
        assert_eq!(target, TabStripDropTarget::PinnedBoundary);
    }
}
