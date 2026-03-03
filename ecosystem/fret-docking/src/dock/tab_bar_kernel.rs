// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::prelude_core::*;
use super::tab_bar_geometry::TabBarGeometry;
use fret_ui::ThemeSnapshot;
use fret_ui_headless::tab_strip_drop_target::{
    TabInsertionSide, TabStripDropTarget, compute_tab_strip_drop_target_midpoint,
};
use fret_ui_headless::tab_strip_surface::{TabStripSurface, classify_tab_strip_surface_no_tabs};

#[derive(Debug, Clone)]
pub(super) struct TabBarOverflowCandidateGeometry {
    pub(super) strip_rect: Rect,
    pub(super) geom: TabBarGeometry,
    pub(super) overflow_button_rect: Rect,
    pub(super) overflows: bool,
}

pub(super) fn compute_tab_bar_overflow_candidate_geometry(
    theme: ThemeSnapshot,
    tab_bar: Rect,
    tab_count: usize,
    tab_widths: Option<&Arc<[Px]>>,
) -> TabBarOverflowCandidateGeometry {
    let strip_rect = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
    let geom = tab_widths
        .filter(|w| w.len() == tab_count)
        .map(|w| TabBarGeometry::variable(strip_rect, (*w).clone()))
        .unwrap_or_else(|| TabBarGeometry::fixed(strip_rect, tab_count));
    let overflows = geom.max_scroll().0 > 0.0;
    let overflow_button_rect = tab_overflow_button_rect(theme, tab_bar);
    TabBarOverflowCandidateGeometry {
        strip_rect,
        geom,
        overflow_button_rect,
        overflows,
    }
}

pub(super) fn tab_overflow_button_rect(theme: ThemeSnapshot, tab_bar: Rect) -> Rect {
    let pad = theme.metric_token("metric.padding.sm").0.max(0.0);
    let size = (tab_bar.size.height.0 * 0.80).clamp(18.0, 24.0);
    let x = tab_bar.origin.x.0 + tab_bar.size.width.0 - pad - size;
    let y = tab_bar.origin.y.0 + (tab_bar.size.height.0 - size) * 0.5;
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(size), Px(size)))
}

/// A reduced tab strip rect that reserves room for the overflow button.
///
/// This should only be used when the tab bar actually overflows.
pub(super) fn tab_strip_rect_with_overflow_button(theme: ThemeSnapshot, tab_bar: Rect) -> Rect {
    let pad = theme.metric_token("metric.padding.sm").0.max(0.0);
    let button = tab_overflow_button_rect(theme, tab_bar);
    let end_x = (button.origin.x.0 - pad).max(tab_bar.origin.x.0);
    let w = (end_x - tab_bar.origin.x.0).max(0.0);
    Rect::new(tab_bar.origin, Size::new(Px(w), tab_bar.size.height))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TabBarDropResolution {
    pub(super) surface: TabStripSurface,
    pub(super) insert_index: Option<usize>,
}

pub(super) fn resolve_tab_bar_drop(
    theme: ThemeSnapshot,
    tab_bar: Rect,
    tab_count: usize,
    tab_widths: Option<&Arc<[Px]>>,
    scroll: Px,
    position: Point,
    dragged_tab_index: Option<usize>,
) -> TabBarDropResolution {
    if tab_count == 0 {
        return TabBarDropResolution {
            surface: TabStripSurface::Outside,
            insert_index: None,
        };
    }
    if !tab_bar.contains(position) {
        return TabBarDropResolution {
            surface: TabStripSurface::Outside,
            insert_index: None,
        };
    }

    let candidate =
        compute_tab_bar_overflow_candidate_geometry(theme.clone(), tab_bar, tab_count, tab_widths);
    let dragged_tab_index = dragged_tab_index.filter(|ix| *ix < tab_count);
    if candidate.overflows {
        // Docking tab bars reserve space for an overflow button. We still want right-edge
        // auto-scroll to work, so only the *gap* between the strip and the overflow button is an
        // explicit "end drop" surface; the trailing padding to the right of the button remains a
        // viewport drop surface.
        let end_drop_rect = {
            let strip_end = candidate.strip_rect.origin.x.0 + candidate.strip_rect.size.width.0;
            let button_start = candidate.overflow_button_rect.origin.x.0;
            let w = (button_start - strip_end).max(0.0);
            (w > 0.0).then_some(Rect::new(
                Point::new(Px(strip_end), tab_bar.origin.y),
                Size::new(Px(w), tab_bar.size.height),
            ))
        };

        let surface = classify_tab_strip_surface_no_tabs(
            position,
            None,
            end_drop_rect,
            Some(tab_bar),
            Some(candidate.overflow_button_rect),
            None,
            None,
        );

        let indices: Vec<usize> = (0..tab_count).collect();
        let drop = compute_tab_strip_drop_target_midpoint(
            position,
            &indices,
            |ix| candidate.geom.tab_rect(*ix, scroll),
            |ix| dragged_tab_index.is_some_and(|dragged| *ix == dragged),
            None,
            end_drop_rect,
            Some(tab_bar),
            Some(candidate.overflow_button_rect),
            None,
            None,
        );
        let insert_index = match drop {
            TabStripDropTarget::None | TabStripDropTarget::PinnedBoundary => None,
            TabStripDropTarget::End => Some(tab_count),
            TabStripDropTarget::Tab { index, side } => Some(
                index
                    + match side {
                        TabInsertionSide::Before => 0,
                        TabInsertionSide::After => 1,
                    },
            ),
        };
        return TabBarDropResolution {
            surface,
            insert_index,
        };
    }

    let geom_full = tab_widths
        .filter(|w| w.len() == tab_count)
        .map(|w| TabBarGeometry::variable(tab_bar, (*w).clone()))
        .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count));

    let indices: Vec<usize> = (0..tab_count).collect();
    let drop = compute_tab_strip_drop_target_midpoint(
        position,
        &indices,
        |ix| geom_full.tab_rect(*ix, scroll),
        |ix| dragged_tab_index.is_some_and(|dragged| *ix == dragged),
        None,
        None,
        Some(tab_bar),
        None,
        None,
        None,
    );
    let insert_index = match drop {
        TabStripDropTarget::None | TabStripDropTarget::PinnedBoundary => None,
        TabStripDropTarget::End => Some(tab_count),
        TabStripDropTarget::Tab { index, side } => Some(
            index
                + match side {
                    TabInsertionSide::Before => 0,
                    TabInsertionSide::After => 1,
                },
        ),
    };

    TabBarDropResolution {
        surface: TabStripSurface::TabsViewport,
        insert_index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::theme::{ThemeColors, ThemeMetrics};

    fn test_theme() -> ThemeSnapshot {
        ThemeSnapshot::from_baseline(
            ThemeColors {
                surface_background: Color::TRANSPARENT,
                panel_background: Color::TRANSPARENT,
                panel_border: Color::TRANSPARENT,
                text_primary: Color::TRANSPARENT,
                text_muted: Color::TRANSPARENT,
                text_disabled: Color::TRANSPARENT,
                accent: Color::TRANSPARENT,
                selection_background: Color::TRANSPARENT,
                selection_inactive_background: Color::TRANSPARENT,
                selection_window_inactive_background: Color::TRANSPARENT,
                hover_background: Color::TRANSPARENT,
                focus_ring: Color::TRANSPARENT,
                menu_background: Color::TRANSPARENT,
                menu_border: Color::TRANSPARENT,
                menu_item_hover: Color::TRANSPARENT,
                menu_item_selected: Color::TRANSPARENT,
                list_background: Color::TRANSPARENT,
                list_border: Color::TRANSPARENT,
                list_row_hover: Color::TRANSPARENT,
                list_row_selected: Color::TRANSPARENT,
                scrollbar_track: Color::TRANSPARENT,
                scrollbar_thumb: Color::TRANSPARENT,
                scrollbar_thumb_hover: Color::TRANSPARENT,
                viewport_selection_fill: Color::TRANSPARENT,
                viewport_selection_stroke: Color::TRANSPARENT,
                viewport_marker: Color::TRANSPARENT,
                viewport_drag_line_pan: Color::TRANSPARENT,
                viewport_drag_line_orbit: Color::TRANSPARENT,
                viewport_gizmo_x: Color::TRANSPARENT,
                viewport_gizmo_y: Color::TRANSPARENT,
                viewport_gizmo_handle_background: Color::TRANSPARENT,
                viewport_gizmo_handle_border: Color::TRANSPARENT,
                viewport_rotate_gizmo: Color::TRANSPARENT,
            },
            ThemeMetrics {
                radius_sm: Px(6.0),
                radius_md: Px(8.0),
                radius_lg: Px(10.0),
                padding_sm: Px(8.0),
                padding_md: Px(10.0),
                scrollbar_width: Px(10.0),
                font_size: Px(13.0),
                mono_font_size: Px(13.0),
                font_line_height: Px(16.0),
                mono_font_line_height: Px(16.0),
            },
            0,
        )
    }

    #[test]
    fn resolve_tab_bar_drop_returns_outside_when_pointer_is_outside_tab_bar() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(24.0)));
        let pos = Point::new(Px(240.0), Px(12.0));
        let resolved = resolve_tab_bar_drop(theme, tab_bar, 3, None, Px(0.0), pos, None);
        assert_eq!(resolved.surface, TabStripSurface::Outside);
        assert_eq!(resolved.insert_index, None);
    }

    #[test]
    fn resolve_tab_bar_drop_no_overflow_drop_end_resolves_to_tab_count() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(800.0), Px(24.0)));
        let y = Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5);
        let pos = Point::new(Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - 1.0), y);

        let resolved_1 = resolve_tab_bar_drop(theme.clone(), tab_bar, 1, None, Px(0.0), pos, None);
        assert_eq!(resolved_1.surface, TabStripSurface::TabsViewport);
        assert_eq!(resolved_1.insert_index, Some(1));

        let resolved_2 = resolve_tab_bar_drop(theme, tab_bar, 2, None, Px(0.0), pos, None);
        assert_eq!(resolved_2.surface, TabStripSurface::TabsViewport);
        assert_eq!(resolved_2.insert_index, Some(2));
    }

    #[test]
    fn resolve_tab_bar_drop_excludes_dragged_tab_from_candidates() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(400.0), Px(24.0)));
        let y = Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5);
        // Drag the last tab (index 2). A position that would normally resolve "before tab 2"
        // should instead resolve as an end-drop when the dragged tab is excluded.
        let pos = Point::new(Px(260.0), y);

        let resolved = resolve_tab_bar_drop(theme, tab_bar, 3, None, Px(0.0), pos, Some(2));
        assert_eq!(resolved.insert_index, Some(3));
    }

    #[test]
    fn resolve_tab_bar_drop_treats_reserved_overflow_header_space_as_end_drop() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(24.0)));
        let widths: Arc<[Px]> = Arc::from([Px(80.0), Px(80.0), Px(80.0)].as_slice());

        let candidate =
            compute_tab_bar_overflow_candidate_geometry(theme.clone(), tab_bar, 3, Some(&widths));
        assert!(candidate.overflows);

        // Pick a point between strip end and overflow button. (The reserved header rect may overlap
        // the overflow button rect; `OverflowControl` must win when overlapping.)
        let strip_end_x = candidate.strip_rect.origin.x.0 + candidate.strip_rect.size.width.0;
        let button_start_x = candidate.overflow_button_rect.origin.x.0;
        let x = Px(((strip_end_x + button_start_x) * 0.5).clamp(strip_end_x, button_start_x));
        let y = Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5);
        let pos = Point::new(x, y);

        let resolved = resolve_tab_bar_drop(theme, tab_bar, 3, Some(&widths), Px(0.0), pos, None);
        assert_eq!(resolved.surface, TabStripSurface::HeaderSpace);
        assert_eq!(resolved.insert_index, Some(3));
    }

    #[test]
    fn resolve_tab_bar_drop_treats_overflow_button_as_non_drop_surface() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(24.0)));
        let widths: Arc<[Px]> = Arc::from([Px(80.0), Px(80.0), Px(80.0)].as_slice());

        let candidate =
            compute_tab_bar_overflow_candidate_geometry(theme.clone(), tab_bar, 3, Some(&widths));
        assert!(candidate.overflows);

        let x = Px(candidate.overflow_button_rect.origin.x.0
            + candidate.overflow_button_rect.size.width.0 * 0.5);
        let y = Px(candidate.overflow_button_rect.origin.y.0
            + candidate.overflow_button_rect.size.height.0 * 0.5);
        let pos = Point::new(x, y);

        let resolved = resolve_tab_bar_drop(theme, tab_bar, 3, Some(&widths), Px(0.0), pos, None);
        assert_eq!(resolved.surface, TabStripSurface::OverflowControl);
        assert_eq!(resolved.insert_index, None);
    }
}
