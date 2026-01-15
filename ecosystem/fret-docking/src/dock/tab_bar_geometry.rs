// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::prelude_core::*;

#[derive(Debug, Clone)]
enum TabWidthModel {
    Fixed { tab_count: usize, tab_width: Px },
}

#[derive(Debug, Clone)]
pub(super) struct TabBarGeometry {
    tab_bar: Rect,
    model: TabWidthModel,
}

impl TabBarGeometry {
    pub(super) fn fixed(tab_bar: Rect, tab_count: usize) -> Self {
        Self {
            tab_bar,
            model: TabWidthModel::Fixed {
                tab_count,
                tab_width: DOCK_TAB_W,
            },
        }
    }

    pub(super) fn tab_count(&self) -> usize {
        match self.model {
            TabWidthModel::Fixed { tab_count, .. } => tab_count,
        }
    }

    pub(super) fn total_width(&self) -> Px {
        match self.model {
            TabWidthModel::Fixed {
                tab_count,
                tab_width,
            } => Px(tab_width.0 * tab_count as f32),
        }
    }

    pub(super) fn tab_width(&self, _index: usize) -> Px {
        match self.model {
            TabWidthModel::Fixed { tab_width, .. } => tab_width,
        }
    }

    pub(super) fn tab_start_x_unscrolled(&self, index: usize) -> Px {
        match self.model {
            TabWidthModel::Fixed { tab_width, .. } => Px(tab_width.0 * index as f32),
        }
    }

    pub(super) fn tab_end_x_unscrolled(&self, index: usize) -> Px {
        let start = self.tab_start_x_unscrolled(index);
        Px(start.0 + self.tab_width(index).0)
    }

    pub(super) fn tab_rect(&self, index: usize, scroll: Px) -> Rect {
        Rect {
            origin: Point::new(
                Px(self.tab_bar.origin.x.0 + self.tab_start_x_unscrolled(index).0 - scroll.0),
                self.tab_bar.origin.y,
            ),
            size: Size::new(self.tab_width(index), self.tab_bar.size.height),
        }
    }

    /// Returns the screen-space x coordinate of the insertion marker.
    pub(super) fn insert_x(&self, insert_index: usize, scroll: Px) -> Px {
        let x_unscrolled = match self.model {
            TabWidthModel::Fixed { tab_width, .. } => Px(tab_width.0 * insert_index as f32),
        };
        Px(self.tab_bar.origin.x.0 + x_unscrolled.0 - scroll.0)
    }

    pub(super) fn max_scroll(&self) -> Px {
        Px((self.total_width().0 - self.tab_bar.size.width.0).max(0.0))
    }

    pub(super) fn clamp_scroll(&self, scroll: Px) -> Px {
        let max_scroll = self.max_scroll();
        Px(scroll.0.clamp(0.0, max_scroll.0))
    }

    pub(super) fn ensure_tab_visible(&self, scroll: Px, tab_index: usize) -> Px {
        if self.tab_count() == 0 {
            return Px(0.0);
        }
        let scroll = self.clamp_scroll(scroll);
        let tab_start = self.tab_start_x_unscrolled(tab_index).0;
        let tab_end = self.tab_end_x_unscrolled(tab_index).0;
        let view_start = scroll.0;
        let view_end = scroll.0 + self.tab_bar.size.width.0;
        let next = if tab_start < view_start {
            Px(tab_start)
        } else if tab_end > view_end {
            Px((tab_end - self.tab_bar.size.width.0).max(0.0))
        } else {
            scroll
        };
        self.clamp_scroll(next)
    }

    pub(super) fn hit_test_tab_index(&self, position: Point, scroll: Px) -> Option<usize> {
        let tab_count = self.tab_count();
        if tab_count == 0 {
            return None;
        }
        let rel_x = position.x.0 - self.tab_bar.origin.x.0 + scroll.0;
        if rel_x < 0.0 || rel_x >= self.total_width().0 {
            return None;
        }
        match self.model {
            TabWidthModel::Fixed { tab_width, .. } => {
                let idx = (rel_x / tab_width.0).floor() as usize;
                (idx < tab_count).then_some(idx)
            }
        }
    }

    pub(super) fn compute_insert_index(&self, position: Point, scroll: Px) -> usize {
        let tab_count = self.tab_count();
        if tab_count == 0 {
            return 0;
        }

        let rel_x = position.x.0 - self.tab_bar.origin.x.0 + scroll.0;
        if rel_x <= 0.0 {
            return 0;
        }
        if rel_x >= self.total_width().0 {
            return tab_count;
        }

        let over_index = self
            .hit_test_tab_index(position, scroll)
            .unwrap_or(tab_count.saturating_sub(1));
        let over_rect = self.tab_rect(over_index, scroll);
        let side = fret_dnd::insertion_side_for_pointer(position, over_rect, fret_dnd::Axis::X);
        over_index.saturating_add(match side {
            fret_dnd::InsertionSide::Before => 0,
            fret_dnd::InsertionSide::After => 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_test_tab_index_ignores_trailing_empty_space() {
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(800.0), Px(24.0)));
        let geom = TabBarGeometry::fixed(tab_bar, 1);
        let scroll = Px(0.0);

        let position = Point::new(Px(600.0), Px(12.0));
        assert_eq!(geom.hit_test_tab_index(position, scroll), None);
    }

    #[test]
    fn insert_index_clamps_to_ends() {
        let tab_bar = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(800.0), Px(24.0)),
        );
        let geom = TabBarGeometry::fixed(tab_bar, 3);
        let scroll = Px(0.0);

        let left = Point::new(Px(tab_bar.origin.x.0 - 1.0), Px(32.0));
        assert_eq!(geom.compute_insert_index(left, scroll), 0);

        let right = Point::new(
            Px(tab_bar.origin.x.0 + tab_bar.size.width.0 + 1.0),
            Px(32.0),
        );
        assert_eq!(geom.compute_insert_index(right, scroll), 3);
    }
}
