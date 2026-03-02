// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::prelude_core::*;
use super::tab_bar_geometry::TabBarGeometry;
use fret_ui::ThemeSnapshot;

#[derive(Debug, Clone)]
pub(super) struct TabBarOverflowCandidateGeometry {
    pub(super) strip_rect: Rect,
    pub(super) geom: TabBarGeometry,
    pub(super) overflow_button_rect: Rect,
    pub(super) reserved_header_space_rect: Rect,
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
    let reserved_header_space_rect = reserved_tab_bar_header_space_rect(tab_bar, strip_rect);
    TabBarOverflowCandidateGeometry {
        strip_rect,
        geom,
        overflow_button_rect,
        reserved_header_space_rect,
        overflows,
    }
}

pub(super) fn reserved_tab_bar_header_space_rect(tab_bar: Rect, strip_candidate: Rect) -> Rect {
    let x0 = strip_candidate.origin.x.0 + strip_candidate.size.width.0;
    let x1 = tab_bar.origin.x.0 + tab_bar.size.width.0;
    let w = (x1 - x0).max(0.0);
    Rect::new(
        Point::new(Px(x0), tab_bar.origin.y),
        Size::new(Px(w), tab_bar.size.height),
    )
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
