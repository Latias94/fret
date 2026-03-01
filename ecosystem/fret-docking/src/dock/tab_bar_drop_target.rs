// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::prelude_core::*;
use super::tab_bar_geometry::TabBarGeometry;
use super::tab_overflow::{tab_overflow_button_rect, tab_strip_rect_with_overflow_button};
use fret_ui::ThemeSnapshot;
use fret_ui_headless::tab_strip_surface::{TabStripSurface, classify_tab_strip_surface_no_tabs};

pub(super) fn tab_bar_insert_index_for_drop(
    theme: ThemeSnapshot,
    tab_bar: Rect,
    tab_count: usize,
    tab_widths: Option<&Arc<[Px]>>,
    scroll: Px,
    position: Point,
) -> Option<usize> {
    if tab_count == 0 {
        return None;
    }
    if !tab_bar.contains(position) {
        return None;
    }

    let strip_candidate = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
    let geom_candidate = tab_widths
        .filter(|w| w.len() == tab_count)
        .map(|w| TabBarGeometry::variable(strip_candidate, (*w).clone()))
        .unwrap_or_else(|| TabBarGeometry::fixed(strip_candidate, tab_count));
    let overflow = geom_candidate.max_scroll().0 > 0.0;

    if overflow {
        let overflow_button = tab_overflow_button_rect(theme.clone(), tab_bar);
        let reserved_header_space_rect = {
            let x0 = strip_candidate.origin.x.0 + strip_candidate.size.width.0;
            let x1 = tab_bar.origin.x.0 + tab_bar.size.width.0;
            let w = (x1 - x0).max(0.0);
            Rect::new(
                Point::new(Px(x0), tab_bar.origin.y),
                Size::new(Px(w), tab_bar.size.height),
            )
        };

        return match classify_tab_strip_surface_no_tabs(
            position,
            None,
            Some(reserved_header_space_rect),
            Some(strip_candidate),
            Some(overflow_button),
            None,
            None,
        ) {
            TabStripSurface::OverflowControl => None,
            TabStripSurface::HeaderSpace => Some(tab_count),
            TabStripSurface::TabsViewport => {
                Some(geom_candidate.compute_insert_index(position, scroll))
            }
            TabStripSurface::Outside
            | TabStripSurface::ScrollControls
            | TabStripSurface::PinnedBoundary => None,
        };
    }

    let geom_full = tab_widths
        .filter(|w| w.len() == tab_count)
        .map(|w| TabBarGeometry::variable(tab_bar, (*w).clone()))
        .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count));
    Some(geom_full.compute_insert_index(position, scroll))
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
    fn overflow_button_is_not_a_drop_surface() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(24.0)));
        let widths: Arc<[Px]> = Arc::from([Px(80.0), Px(80.0), Px(80.0)].as_slice());
        let button = tab_overflow_button_rect(theme.clone(), tab_bar);
        let pos = Point::new(
            Px(button.origin.x.0 + button.size.width.0 * 0.5),
            Px(button.origin.y.0 + button.size.height.0 * 0.5),
        );
        assert_eq!(
            tab_bar_insert_index_for_drop(theme, tab_bar, 3, Some(&widths), Px(0.0), pos),
            None
        );
    }

    #[test]
    fn overflow_header_space_is_explicit_end_drop_surface() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(24.0)));
        let widths: Arc<[Px]> = Arc::from([Px(80.0), Px(80.0), Px(80.0)].as_slice());

        let strip = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
        let button = tab_overflow_button_rect(theme.clone(), tab_bar);

        // Pick a point between strip end and overflow button.
        let x = Px(((strip.origin.x.0 + strip.size.width.0) + button.origin.x.0) * 0.5);
        let y = Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5);
        let pos = Point::new(x, y);

        assert_eq!(
            tab_bar_insert_index_for_drop(theme, tab_bar, 3, Some(&widths), Px(0.0), pos),
            Some(3)
        );
    }
}
