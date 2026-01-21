//! Headless 2D selection helpers for canvas-like surfaces.
//!
//! The goal is to standardize the boring parts:
//! - click vs box selection classification,
//! - a minimal hit-test trait surface,
//! - normalized selection rectangles and modifier-to-mode mapping helpers.
//!
//! Consumers are expected to:
//! - own their selection set state,
//! - decide "primary" vs "secondary" semantics,
//! - layer tool modes and snapping elsewhere.

use std::hash::Hash;

use fret_core::{Modifiers, Point, Px, Rect, Size};

use crate::drag::DragThreshold;
use crate::view::CanvasViewport2D;

/// Selection semantics describing how a gesture should be applied to an existing selection set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Replace the previous selection with the new hits.
    Replace,
    /// Add the new hits to the existing selection.
    Add,
    /// Toggle membership of each hit in the existing selection.
    Toggle,
}

/// A conservative default mapping for editor-grade selection behavior.
///
/// - `Ctrl`/`Meta` => `Toggle` (platform-agnostic "command" modifier).
/// - `Shift` => `Add`.
/// - Otherwise => `Replace`.
///
/// When both toggle and add modifiers are present, `Toggle` wins.
#[inline]
pub fn selection_mode_from_modifiers_default(modifiers: Modifiers) -> SelectionMode {
    if modifiers.ctrl || modifiers.meta {
        SelectionMode::Toggle
    } else if modifiers.shift {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    }
}

/// Minimal hit-testing hooks used by selection helpers.
///
/// This trait is intentionally small and does not prescribe a spatial index implementation.
pub trait HitTest2D<Id> {
    /// Returns the "best" hit for a click selection near `canvas_point`.
    fn hit_test_point(&self, canvas_point: Point, radius: Px) -> Option<Id>;

    /// Pushes all items intersecting `canvas_rect` into `out`.
    fn hit_test_rect(&self, canvas_rect: Rect, out: &mut Vec<Id>);
}

/// Settings for turning pointer drags into selection queries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionSettings2D {
    /// Minimum movement (in screen-space logical pixels) before a press becomes a box selection.
    pub drag_threshold: DragThreshold,
    /// Click selection hit slop expressed in screen-space logical pixels.
    pub click_radius_screen_px: f32,
}

impl Default for SelectionSettings2D {
    fn default() -> Self {
        Self {
            drag_threshold: DragThreshold { screen_px: 3.0 },
            click_radius_screen_px: 6.0,
        }
    }
}

/// A normalized canvas-space selection rectangle derived from two points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionRect2D {
    pub start: Point,
    pub end: Point,
}

impl SelectionRect2D {
    pub fn normalized_rect(self) -> Rect {
        let min_x = self.start.x.0.min(self.end.x.0);
        let min_y = self.start.y.0.min(self.end.y.0);
        let max_x = self.start.x.0.max(self.end.x.0);
        let max_y = self.start.y.0.max(self.end.y.0);
        Rect::new(
            Point::new(Px(min_x), Px(min_y)),
            Size::new(Px(max_x - min_x), Px(max_y - min_y)),
        )
    }
}

/// The resolved query for a completed selection gesture.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionQuery2D<Id> {
    /// Click selection with an optional hit.
    Click {
        mode: SelectionMode,
        hit: Option<Id>,
    },
    /// Box selection over a normalized canvas-space rectangle.
    Box {
        mode: SelectionMode,
        rect: Rect,
        hits: Vec<Id>,
    },
}

/// Returns `true` when the drag distance exceeds the threshold (screen-space, logical pixels).
#[inline]
pub fn exceeds_drag_threshold(
    start_screen: Point,
    end_screen: Point,
    threshold: DragThreshold,
) -> bool {
    let dx = end_screen.x.0 - start_screen.x.0;
    let dy = end_screen.y.0 - start_screen.y.0;
    if !dx.is_finite() || !dy.is_finite() {
        return false;
    }
    let dist2 = dx * dx + dy * dy;
    let t = threshold.screen_px;
    if !t.is_finite() || t <= 0.0 {
        return dist2 > 0.0;
    }
    dist2 >= t * t
}

/// Resolves a pointer gesture into a selection query using the provided hit-test hooks.
///
/// The gesture is classified as:
/// - `Click` when it does not exceed `settings.drag_threshold`.
/// - `Box` when it exceeds the threshold (marquee selection).
pub fn resolve_selection_query_2d<Id: Clone>(
    viewport: CanvasViewport2D,
    start_screen: Point,
    end_screen: Point,
    modifiers: Modifiers,
    settings: SelectionSettings2D,
    hit_test: &impl HitTest2D<Id>,
) -> SelectionQuery2D<Id> {
    let mode = selection_mode_from_modifiers_default(modifiers);
    let is_box = exceeds_drag_threshold(start_screen, end_screen, settings.drag_threshold);

    if !is_box {
        let canvas = viewport.screen_to_canvas(end_screen);
        let radius = viewport.canvas_units_from_screen_px(settings.click_radius_screen_px);
        let radius = if radius.is_finite() && radius > 0.0 {
            Px(radius)
        } else {
            Px(0.0)
        };
        let hit = hit_test.hit_test_point(canvas, radius);
        return SelectionQuery2D::Click { mode, hit };
    }

    let start_canvas = viewport.screen_to_canvas(start_screen);
    let end_canvas = viewport.screen_to_canvas(end_screen);
    let rect = SelectionRect2D {
        start: start_canvas,
        end: end_canvas,
    }
    .normalized_rect();

    let mut hits = Vec::new();
    hit_test.hit_test_rect(rect, &mut hits);
    SelectionQuery2D::Box { mode, rect, hits }
}

/// Convenience helper for applying a `SelectionQuery2D` to a `HashSet`.
///
/// This is optional sugar for common ecosystem usage. Consumers with ordering or "primary" needs
/// should implement their own application logic.
pub fn apply_query_to_hash_set<Id: Eq + Hash + Clone>(
    selection: &mut std::collections::HashSet<Id>,
    query: SelectionQuery2D<Id>,
) {
    match query {
        SelectionQuery2D::Click { mode, hit } => {
            let hits = hit.into_iter().collect::<Vec<_>>();
            apply_hits_to_hash_set(selection, mode, &hits);
        }
        SelectionQuery2D::Box { mode, hits, .. } => {
            apply_hits_to_hash_set(selection, mode, &hits);
        }
    }
}

fn apply_hits_to_hash_set<Id: Eq + Hash + Clone>(
    selection: &mut std::collections::HashSet<Id>,
    mode: SelectionMode,
    hits: &[Id],
) {
    match mode {
        SelectionMode::Replace => {
            selection.clear();
            selection.extend(hits.iter().cloned());
        }
        SelectionMode::Add => {
            selection.extend(hits.iter().cloned());
        }
        SelectionMode::Toggle => {
            for id in hits {
                if !selection.remove(id) {
                    selection.insert(id.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::PanZoom2D;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct Id(u32);

    struct RectHitTest {
        items: Vec<(Id, Rect)>,
    }

    impl HitTest2D<Id> for RectHitTest {
        fn hit_test_point(&self, canvas_point: Point, radius: Px) -> Option<Id> {
            let r = radius.0.max(0.0);
            for (id, rect) in &self.items {
                let min_x = rect.origin.x.0 - r;
                let min_y = rect.origin.y.0 - r;
                let max_x = rect.origin.x.0 + rect.size.width.0 + r;
                let max_y = rect.origin.y.0 + rect.size.height.0 + r;
                if canvas_point.x.0 >= min_x
                    && canvas_point.x.0 <= max_x
                    && canvas_point.y.0 >= min_y
                    && canvas_point.y.0 <= max_y
                {
                    return Some(*id);
                }
            }
            None
        }

        fn hit_test_rect(&self, canvas_rect: Rect, out: &mut Vec<Id>) {
            let a0 = canvas_rect.origin;
            let a1 = Point::new(
                Px(canvas_rect.origin.x.0 + canvas_rect.size.width.0),
                Px(canvas_rect.origin.y.0 + canvas_rect.size.height.0),
            );
            for (id, rect) in &self.items {
                let b0 = rect.origin;
                let b1 = Point::new(
                    Px(rect.origin.x.0 + rect.size.width.0),
                    Px(rect.origin.y.0 + rect.size.height.0),
                );
                let intersects =
                    a0.x.0 <= b1.x.0 && a1.x.0 >= b0.x.0 && a0.y.0 <= b1.y.0 && a1.y.0 >= b0.y.0;
                if intersects {
                    out.push(*id);
                }
            }
        }
    }

    fn viewport_identity() -> CanvasViewport2D {
        CanvasViewport2D::new(
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(100.0), Px(100.0)),
            ),
            PanZoom2D::default(),
        )
    }

    #[test]
    fn selection_mode_default_mapping() {
        assert_eq!(
            selection_mode_from_modifiers_default(Modifiers {
                ctrl: false,
                meta: false,
                shift: false,
                alt: false,
                alt_gr: false,
            }),
            SelectionMode::Replace
        );
        assert_eq!(
            selection_mode_from_modifiers_default(Modifiers {
                ctrl: false,
                meta: false,
                shift: true,
                alt: false,
                alt_gr: false,
            }),
            SelectionMode::Add
        );
        assert_eq!(
            selection_mode_from_modifiers_default(Modifiers {
                ctrl: true,
                meta: false,
                shift: true,
                alt: false,
                alt_gr: false,
            }),
            SelectionMode::Toggle
        );
    }

    #[test]
    fn resolve_click_vs_box_by_threshold() {
        let hit_test = RectHitTest {
            items: vec![(
                Id(1),
                Rect::new(
                    Point::new(Px(10.0), Px(10.0)),
                    Size::new(Px(10.0), Px(10.0)),
                ),
            )],
        };
        let viewport = viewport_identity();
        let settings = SelectionSettings2D {
            drag_threshold: DragThreshold { screen_px: 4.0 },
            click_radius_screen_px: 0.0,
        };

        let q = resolve_selection_query_2d(
            viewport,
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(3.0), Px(0.0)),
            Modifiers::default(),
            settings,
            &hit_test,
        );
        assert!(matches!(q, SelectionQuery2D::Click { .. }));

        let q = resolve_selection_query_2d(
            viewport,
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(4.0), Px(0.0)),
            Modifiers::default(),
            settings,
            &hit_test,
        );
        assert!(matches!(q, SelectionQuery2D::Box { .. }));
    }

    #[test]
    fn click_hit_uses_end_position() {
        let hit_test = RectHitTest {
            items: vec![(
                Id(1),
                Rect::new(
                    Point::new(Px(10.0), Px(10.0)),
                    Size::new(Px(10.0), Px(10.0)),
                ),
            )],
        };
        let viewport = viewport_identity();
        let settings = SelectionSettings2D {
            drag_threshold: DragThreshold { screen_px: 100.0 },
            click_radius_screen_px: 0.0,
        };

        let q = resolve_selection_query_2d(
            viewport,
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(12.0), Px(12.0)),
            Modifiers::default(),
            settings,
            &hit_test,
        );
        assert_eq!(
            q,
            SelectionQuery2D::Click {
                mode: SelectionMode::Replace,
                hit: Some(Id(1)),
            }
        );
    }

    #[test]
    fn box_selection_collects_hits() {
        let hit_test = RectHitTest {
            items: vec![
                (
                    Id(1),
                    Rect::new(
                        Point::new(Px(10.0), Px(10.0)),
                        Size::new(Px(10.0), Px(10.0)),
                    ),
                ),
                (
                    Id(2),
                    Rect::new(Point::new(Px(40.0), Px(40.0)), Size::new(Px(5.0), Px(5.0))),
                ),
            ],
        };
        let viewport = viewport_identity();
        let settings = SelectionSettings2D {
            drag_threshold: DragThreshold { screen_px: 1.0 },
            click_radius_screen_px: 0.0,
        };

        let q = resolve_selection_query_2d(
            viewport,
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(30.0), Px(30.0)),
            Modifiers::default(),
            settings,
            &hit_test,
        );
        match q {
            SelectionQuery2D::Box { hits, .. } => {
                assert_eq!(hits, vec![Id(1)]);
            }
            _ => panic!("expected box selection"),
        }
    }

    #[test]
    fn apply_query_to_hash_set_toggle_and_replace() {
        let mut set = std::collections::HashSet::new();
        set.insert(Id(1));
        set.insert(Id(2));

        apply_query_to_hash_set(
            &mut set,
            SelectionQuery2D::Box {
                mode: SelectionMode::Toggle,
                rect: viewport_identity().visible_canvas_rect(),
                hits: vec![Id(2), Id(3)],
            },
        );
        assert!(set.contains(&Id(1)));
        assert!(!set.contains(&Id(2)));
        assert!(set.contains(&Id(3)));

        apply_query_to_hash_set(
            &mut set,
            SelectionQuery2D::Click {
                mode: SelectionMode::Replace,
                hit: Some(Id(9)),
            },
        );
        assert_eq!(set.len(), 1);
        assert!(set.contains(&Id(9)));
    }
}
