use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{AppWindowId, PanelKey, Rect, TextConstraints, TextOverflow, TextStyle, TextWrap};
use fret_ui::UiHost;

use super::super::manager::DockManager;
use super::super::tab_bar_drop_target::tab_bar_insert_index_for_drop;
use super::super::types::{HoverTarget, PreparedTabTitle};
use super::DockSpaceElementFrame;
use super::interaction::DeclarativeDockInteractionService;

pub(super) fn prepare_declarative_tab_title(
    services: &mut dyn fret_core::UiServices,
    title: &str,
    scale_factor: f32,
) -> PreparedTabTitle {
    let pad_x = fret_core::Px(8.0);
    let reserve = fret_core::Px(
        super::super::consts::DOCK_TAB_CLOSE_SIZE.0 + super::super::consts::DOCK_TAB_CLOSE_GAP.0,
    );
    let inner_max_w = fret_core::Px(
        (super::super::consts::DOCK_TAB_MAX_W.0 - pad_x.0 * 2.0 - reserve.0).max(1.0),
    );
    let style = TextStyle {
        font: fret_core::FontId::default(),
        size: fret_core::Px(13.0),
        ..Default::default()
    };
    let constraints = TextConstraints {
        max_width: Some(inner_max_w),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor,
    };
    let (mut blob, mut metrics) = services.text().prepare_str(title, &style, constraints);
    if metrics.size.width.0 <= 0.0 && !title.is_empty() {
        services.text().release(blob);
        (blob, metrics) = services.text().prepare_str(
            title,
            &style,
            TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor,
            },
        );
    }
    PreparedTabTitle { blob, metrics }
}

fn prepare_declarative_tab_glyph(
    services: &mut dyn fret_core::UiServices,
    glyph: &str,
    scale_factor: f32,
) -> PreparedTabTitle {
    let style = TextStyle {
        font: fret_core::FontId::default(),
        size: fret_core::Px(13.0),
        ..Default::default()
    };
    let (blob, metrics) = services.text().prepare_str(
        glyph,
        &style,
        TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor,
        },
    );
    PreparedTabTitle { blob, metrics }
}

pub(super) fn declarative_tab_detail_titles<H: UiHost>(
    app: &H,
    frame: &DockSpaceElementFrame,
) -> HashMap<PanelKey, String> {
    let mut titles = HashMap::new();
    if let Some(dock) = app.global::<DockManager>() {
        for input in &frame.tab_detail_inputs {
            for panel in input.tabs.iter() {
                if titles.contains_key(panel) {
                    continue;
                }
                let title = dock
                    .panel(panel)
                    .map(|panel| panel.title.as_str())
                    .filter(|title| !title.is_empty())
                    .unwrap_or(panel.kind.0.as_str())
                    .to_string();
                titles.insert(panel.clone(), title);
            }
        }
    }
    titles
}

pub(super) fn prepare_declarative_tab_detail_paint(
    titles: HashMap<PanelKey, String>,
    services: &mut dyn fret_core::UiServices,
    scale_factor: f32,
) -> (
    HashMap<PanelKey, PreparedTabTitle>,
    PreparedTabTitle,
    PreparedTabTitle,
) {
    let mut tab_titles = HashMap::new();
    for (panel, title) in titles {
        tab_titles.insert(
            panel,
            prepare_declarative_tab_title(services, &title, scale_factor),
        );
    }
    let close_glyph = prepare_declarative_tab_glyph(services, "x", scale_factor);
    let overflow_glyph = prepare_declarative_tab_glyph(services, "...", scale_factor);
    (tab_titles, close_glyph, overflow_glyph)
}

pub(super) fn declarative_tab_widths_for_layout<H: UiHost>(
    app: &H,
    window: AppWindowId,
    theme: fret_ui::ThemeSnapshot,
    layout: &HashMap<fret_core::DockNodeId, Rect>,
) -> HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>> {
    use super::super::tab_bar_geometry::dock_tab_width_for_title;

    let mut widths = HashMap::new();
    let measured_widths = app
        .global::<DeclarativeDockInteractionService>()
        .map(|service| service.tab_widths_for(window))
        .unwrap_or_default();
    let Some(dock) = app.global::<DockManager>() else {
        return widths;
    };
    for &node in layout.keys() {
        let Some(fret_core::DockNode::Tabs { tabs, .. }) = dock.graph.node(node) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        if let Some(measured) = measured_widths.get(&node).filter(|w| w.len() == tabs.len()) {
            widths.insert(node, measured.clone());
            continue;
        }
        let tab_widths: Vec<_> = tabs
            .iter()
            .map(|panel| {
                let approx_title_chars = dock
                    .panel(panel)
                    .map(|panel| panel.title.as_str())
                    .filter(|title| !title.is_empty())
                    .unwrap_or(panel.kind.0.as_str())
                    .chars()
                    .count() as f32;
                let approx_title_width = fret_core::Px(approx_title_chars * 7.0);
                dock_tab_width_for_title(theme.clone(), approx_title_width, true)
            })
            .collect();
        widths.insert(node, Arc::from(tab_widths));
    }
    widths
}

pub(super) fn declarative_tab_widths_from_prepared_titles<H: UiHost>(
    app: &H,
    theme: fret_ui::ThemeSnapshot,
    frame: &DockSpaceElementFrame,
    tab_titles: &HashMap<PanelKey, PreparedTabTitle>,
    close_glyph_present: bool,
) -> HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>> {
    use super::super::tab_bar_geometry::dock_tab_width_for_title;

    let mut widths = HashMap::new();
    let Some(dock) = app.global::<DockManager>() else {
        return widths;
    };
    for &node in frame.layout_all.keys() {
        let Some(fret_core::DockNode::Tabs { tabs, .. }) = dock.graph.node(node) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let tab_widths: Vec<_> = tabs
            .iter()
            .map(|panel| {
                let title_width = tab_titles
                    .get(panel)
                    .map(|title| title.metrics.size.width)
                    .unwrap_or(fret_core::Px(0.0));
                dock_tab_width_for_title(theme.clone(), title_width, close_glyph_present)
            })
            .collect();
        widths.insert(node, Arc::from(tab_widths));
    }
    widths
}

pub(super) fn declarative_tab_bar_geometry(
    theme: fret_ui::ThemeSnapshot,
    tab_widths: &HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
    tabs: fret_core::DockNodeId,
    tab_bar: Rect,
    tab_count: usize,
) -> (super::super::tab_bar_geometry::TabBarGeometry, bool) {
    let strip_candidate =
        super::super::tab_overflow::tab_strip_rect_with_overflow_button(theme, tab_bar);
    let geom_candidate = tab_widths
        .get(&tabs)
        .filter(|w| w.len() == tab_count)
        .map(|w| {
            super::super::tab_bar_geometry::TabBarGeometry::variable(strip_candidate, w.clone())
        })
        .unwrap_or_else(|| {
            super::super::tab_bar_geometry::TabBarGeometry::fixed(strip_candidate, tab_count)
        });
    if geom_candidate.max_scroll().0 > 0.0 {
        return (geom_candidate, true);
    }

    let geom = tab_widths
        .get(&tabs)
        .filter(|w| w.len() == tab_count)
        .map(|w| super::super::tab_bar_geometry::TabBarGeometry::variable(tab_bar, w.clone()))
        .unwrap_or_else(|| {
            super::super::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, tab_count)
        });
    (geom, false)
}

pub(super) fn declarative_clamp_and_ensure_active_visible(
    tab_scroll: &mut HashMap<fret_core::DockNodeId, fret_core::Px>,
    tab_widths: &HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
    theme: fret_ui::ThemeSnapshot,
    tabs_node: fret_core::DockNodeId,
    tab_bar: Rect,
    tab_count: usize,
    active: usize,
) {
    if tab_count == 0 {
        tab_scroll.remove(&tabs_node);
        return;
    }

    let (geom, _overflow) =
        declarative_tab_bar_geometry(theme, tab_widths, tabs_node, tab_bar, tab_count);
    let max_scroll = geom.max_scroll();
    if max_scroll.0 <= 0.0 {
        tab_scroll.remove(&tabs_node);
        return;
    }

    let scroll = tab_scroll
        .get(&tabs_node)
        .copied()
        .unwrap_or(fret_core::Px(0.0));
    let next = geom.ensure_tab_visible(scroll, active.min(tab_count.saturating_sub(1)));
    if next.0 <= 0.0 {
        tab_scroll.remove(&tabs_node);
    } else {
        tab_scroll.insert(tabs_node, next);
    }
}

pub(super) fn declarative_tab_scroll_for_frame<H: UiHost>(
    app: &H,
    window: AppWindowId,
    theme: fret_ui::ThemeSnapshot,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    tab_widths: &HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
    ensure_active: bool,
) -> HashMap<fret_core::DockNodeId, fret_core::Px> {
    let mut tab_scroll = app
        .global::<DeclarativeDockInteractionService>()
        .map(|service| service.tab_scroll_for(window))
        .unwrap_or_default();
    let mut visible_tabs = HashSet::new();

    if let Some(dock) = app.global::<DockManager>() {
        for (&node_id, &rect) in layout_all {
            let Some(fret_core::DockNode::Tabs { tabs, active }) = dock.graph.node(node_id) else {
                continue;
            };
            visible_tabs.insert(node_id);
            let (tab_bar, _content) = super::super::layout::split_tab_bar(rect);
            if ensure_active {
                declarative_clamp_and_ensure_active_visible(
                    &mut tab_scroll,
                    tab_widths,
                    theme.clone(),
                    node_id,
                    tab_bar,
                    tabs.len(),
                    *active,
                );
            } else {
                let (geom, _overflow) = declarative_tab_bar_geometry(
                    theme.clone(),
                    tab_widths,
                    node_id,
                    tab_bar,
                    tabs.len(),
                );
                let max_scroll = geom.max_scroll();
                let scroll = tab_scroll
                    .get(&node_id)
                    .copied()
                    .unwrap_or(fret_core::Px(0.0));
                let scroll = geom.clamp_scroll(scroll);
                if max_scroll.0 <= 0.0 || scroll.0 <= 0.0 {
                    tab_scroll.remove(&node_id);
                } else {
                    tab_scroll.insert(node_id, scroll);
                }
            }
        }
    }

    tab_scroll.retain(|tabs, _| visible_tabs.contains(tabs));
    tab_scroll
}

pub(super) fn declarative_apply_tab_bar_drag_auto_scroll(
    theme: fret_ui::ThemeSnapshot,
    hover: &mut HoverTarget,
    tab_bar: Rect,
    tab_count: usize,
    font_size: fret_core::Px,
    position: fret_core::Point,
    tab_widths: &HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
    tab_scroll: &mut HashMap<fret_core::DockNodeId, fret_core::Px>,
    dragged_tab_for_drop: Option<(fret_core::DockNodeId, usize)>,
) -> bool {
    if tab_count == 0
        || hover.zone != fret_core::DropZone::Center
        || hover.insert_index.is_none()
        || hover.outer
    {
        return false;
    }
    if !tab_bar.contains(position) {
        return false;
    }

    let tabs = hover.tabs;
    let (geom, _overflow) =
        declarative_tab_bar_geometry(theme.clone(), tab_widths, tabs, tab_bar, tab_count);
    let max_scroll = geom.max_scroll();
    if max_scroll.0 <= 0.0 {
        return false;
    }

    let edge =
        fret_core::Px(((tab_bar.size.height.0 * 0.6).max(font_size.0 * 1.25)).clamp(12.0, 28.0));
    let base = (font_size.0 * 0.9).clamp(8.0, 22.0);
    let cfg = fret_dnd::AutoScrollConfig {
        margin_px: edge.0,
        min_speed_px_per_tick: base * 0.20,
        max_speed_px_per_tick: base,
    };
    let prev_scroll = tab_scroll.get(&tabs).copied().unwrap_or(fret_core::Px(0.0));
    let dx =
        fret_dnd::compute_autoscroll_x_clamped(cfg, tab_bar, position, prev_scroll, max_scroll);
    if dx.0.abs() < 0.01 {
        return false;
    }

    let next_scroll = fret_core::Px((prev_scroll.0 + dx.0).clamp(0.0, max_scroll.0));
    if (next_scroll.0 - prev_scroll.0).abs() < 0.01 {
        return false;
    }

    if next_scroll.0 <= 0.0 {
        tab_scroll.remove(&tabs);
    } else {
        tab_scroll.insert(tabs, next_scroll);
    }

    let dragged_tab_index = dragged_tab_for_drop
        .and_then(|(source_tabs, index)| (source_tabs == tabs).then_some(index));
    hover.insert_index = tab_bar_insert_index_for_drop(
        theme,
        tab_bar,
        tab_count,
        tab_widths.get(&tabs),
        next_scroll,
        position,
        dragged_tab_index,
    );
    true
}

pub(super) fn declarative_sync_tab_scroll_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    tab_scroll: &HashMap<fret_core::DockNodeId, fret_core::Px>,
    visible_tabs: impl IntoIterator<Item = fret_core::DockNodeId>,
) {
    let visible_tabs: HashSet<_> = visible_tabs.into_iter().collect();
    app.with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| {
            service.retain_tab_scroll_for_window(window, &visible_tabs);
            for (&tabs, &scroll) in tab_scroll {
                service.set_tab_scroll_for(window, tabs, scroll);
            }
        },
    );
}
