// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::prelude_core::*;
use fret_ui::ThemeSnapshot;

#[derive(Debug, Clone, Copy)]
pub(super) struct TabOverflowMenuState {
    pub(super) tabs: DockNodeId,
    pub(super) scroll: Px,
    pub(super) hovered: Option<usize>,
}

pub(super) fn tab_overflow_button_rect(theme: ThemeSnapshot, tab_bar: Rect) -> Rect {
    let pad = theme.metric_required("metric.padding.sm").0.max(0.0);
    let size = (tab_bar.size.height.0 * 0.80).clamp(18.0, 24.0);
    let x = tab_bar.origin.x.0 + tab_bar.size.width.0 - pad - size;
    let y = tab_bar.origin.y.0 + (tab_bar.size.height.0 - size) * 0.5;
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(size), Px(size)))
}

pub(super) fn tab_overflow_menu_rect(
    theme: ThemeSnapshot,
    tab_bar: Rect,
    tab_count: usize,
) -> Rect {
    let pad = theme.metric_required("metric.padding.sm").0.max(0.0);
    let width = (tab_bar.size.width.0 * 0.55).clamp(180.0, 320.0);
    let rows = overflow_menu_row_count(tab_count) as f32;
    let height = (rows * tab_bar.size.height.0).clamp(tab_bar.size.height.0 * 2.0, 320.0);
    let x = tab_bar.origin.x.0 + tab_bar.size.width.0 - pad - width;
    let y = tab_bar.origin.y.0 + tab_bar.size.height.0 + pad;
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(width), Px(height)))
}

/// A reduced tab strip rect that reserves room for the overflow button.
///
/// This should only be used when the tab bar actually overflows.
pub(super) fn tab_strip_rect_with_overflow_button(theme: ThemeSnapshot, tab_bar: Rect) -> Rect {
    let pad = theme.metric_required("metric.padding.sm").0.max(0.0);
    let button = tab_overflow_button_rect(theme, tab_bar);
    let end_x = (button.origin.x.0 - pad).max(tab_bar.origin.x.0);
    let w = (end_x - tab_bar.origin.x.0).max(0.0);
    Rect::new(tab_bar.origin, Size::new(Px(w), tab_bar.size.height))
}

pub(super) fn overflow_menu_row_height(tab_bar: Rect) -> Px {
    Px(tab_bar.size.height.0.max(0.0))
}

pub(super) fn overflow_menu_row_count(tab_count: usize) -> usize {
    tab_count.min(10).max(1)
}

pub(super) fn overflow_menu_max_scroll(tab_bar: Rect, tab_count: usize) -> Px {
    let row_h = overflow_menu_row_height(tab_bar).0;
    let visible = overflow_menu_row_count(tab_count) as f32;
    let visible_h = row_h * visible;
    let total_h = row_h * tab_count as f32;
    Px((total_h - visible_h).max(0.0))
}

pub(super) fn overflow_menu_row_at_pos(
    menu_rect: Rect,
    tab_bar: Rect,
    tab_count: usize,
    scroll: Px,
    pos: Point,
) -> Option<usize> {
    let row_h = overflow_menu_row_height(tab_bar).0;
    if row_h <= 0.0 {
        return None;
    }
    let y = (pos.y.0 - menu_rect.origin.y.0) + scroll.0;
    let idx = (y / row_h).floor() as isize;
    if idx < 0 {
        return None;
    }
    let idx = idx as usize;
    (idx < tab_count).then_some(idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::theme::{ThemeColors, ThemeMetrics};

    fn test_theme() -> ThemeSnapshot {
        ThemeSnapshot {
            colors: ThemeColors {
                surface_background: Color::TRANSPARENT,
                panel_background: Color::TRANSPARENT,
                panel_border: Color::TRANSPARENT,
                text_primary: Color::TRANSPARENT,
                text_muted: Color::TRANSPARENT,
                text_disabled: Color::TRANSPARENT,
                accent: Color::TRANSPARENT,
                selection_background: Color::TRANSPARENT,
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
            metrics: ThemeMetrics {
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
            revision: 0,
        }
    }

    #[test]
    fn tab_strip_rect_reserves_space_for_overflow_button() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(400.0), Px(24.0)));
        let strip = tab_strip_rect_with_overflow_button(theme, tab_bar);
        assert!(strip.size.width.0 < tab_bar.size.width.0);
        assert!(strip.size.width.0 >= 0.0);
    }

    #[test]
    fn overflow_menu_row_at_pos_accounts_for_scroll() {
        let theme = test_theme();
        let tab_bar = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(400.0), Px(20.0)));
        let menu = tab_overflow_menu_rect(theme, tab_bar, 20);

        let row_h = overflow_menu_row_height(tab_bar);
        let pos = Point::new(Px(menu.origin.x.0 + 10.0), Px(menu.origin.y.0 + 1.0));
        assert_eq!(
            overflow_menu_row_at_pos(menu, tab_bar, 20, Px(0.0), pos),
            Some(0)
        );
        assert_eq!(
            overflow_menu_row_at_pos(menu, tab_bar, 20, Px(row_h.0 * 3.0), pos),
            Some(3)
        );
    }
}
