use std::{collections::HashMap, sync::Arc};

use fret_core::{
    Color, DockGraph, DockNode, DockNodeId, DropZone, EdgeDockDecision, Edges, Scene, SceneOp,
    geometry::{Point, Px, Rect, Size},
};

use super::super::hit_test::tab_scroll_for_node;
use super::super::layout::{drop_zone_rect, split_tab_bar};
use super::super::split_geometry;
use super::super::tab_bar_geometry::TabBarGeometry;
use super::super::tab_bar_geometry::dock_tab_width_for_title;
use super::super::tab_overflow::tab_strip_rect_with_overflow_button;
use super::super::types::{DockDropTarget, PreparedTabTitle};

#[derive(Debug, Clone)]
pub(in crate::dock) enum ComplexDropOverlayPaintInput {
    TabInsertMarker { marker: Rect, caps: [Rect; 2] },
    EdgeZone { overlay: Rect },
}

#[allow(clippy::too_many_arguments)]
pub(in crate::dock) fn complex_drop_overlay_paint_inputs(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    window: fret_core::AppWindowId,
    graph: &DockGraph,
    layout: &HashMap<DockNodeId, Rect>,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
) -> Vec<ComplexDropOverlayPaintInput> {
    let Some(DockDropTarget::Dock(target)) = target else {
        return Vec::new();
    };
    let Some(rect) = layout.get(&target.tabs).copied() else {
        return Vec::new();
    };

    if target.zone == DropZone::Center {
        let Some(i) = target.insert_index else {
            return Vec::new();
        };
        let (tab_bar, _content) = split_tab_bar(rect);
        let scroll = tab_scroll_for_node(tab_scroll, target.tabs);
        let tab_count = match graph.node(target.tabs) {
            Some(DockNode::Tabs { tabs, .. }) => tabs.len(),
            _ => 0,
        };
        let strip_candidate = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
        let geom_candidate = tab_widths
            .get(&target.tabs)
            .filter(|w| w.len() == tab_count)
            .map(|w| TabBarGeometry::variable(strip_candidate, w.clone()))
            .unwrap_or_else(|| TabBarGeometry::fixed(strip_candidate, tab_count));
        let overflow = geom_candidate.max_scroll().0 > 0.0;
        let geom = if overflow {
            geom_candidate
        } else {
            tab_widths
                .get(&target.tabs)
                .filter(|w| w.len() == tab_count)
                .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
                .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count))
        };
        let tab_strip = if overflow { strip_candidate } else { tab_bar };
        let x = geom.insert_x(i.min(tab_count), scroll).0;
        let marker = Rect::new(
            Point::new(Px(x - 3.0), Px(tab_strip.origin.y.0 + 3.0)),
            Size::new(Px(6.0), Px((tab_strip.size.height.0 - 6.0).max(0.0))),
        );
        let cap_w = Px(14.0);
        let cap_h = Px(3.0);
        let cap_x = Px(x - cap_w.0 * 0.5);
        let cap_top = Rect::new(Point::new(cap_x, marker.origin.y), Size::new(cap_w, cap_h));
        let cap_bottom = Rect::new(
            Point::new(
                cap_x,
                Px(marker.origin.y.0 + marker.size.height.0 - cap_h.0),
            ),
            Size::new(cap_w, cap_h),
        );
        return vec![ComplexDropOverlayPaintInput::TabInsertMarker {
            marker,
            caps: [cap_top, cap_bottom],
        }];
    }

    let overlay = match graph.edge_dock_decision(window, target.tabs, target.zone) {
        Some(EdgeDockDecision::InsertIntoSplit {
            split,
            anchor_index,
            insert_index,
        }) => {
            let preview = layout.get(&split).copied().and_then(|bounds| {
                let Some(DockNode::Split {
                    axis,
                    children,
                    fractions,
                }) = graph.node(split)
                else {
                    return None;
                };
                if children.is_empty() || children.len() != fractions.len() {
                    return None;
                }

                let mut next = fractions.clone();
                let old = *next.get(anchor_index)?;
                let keep = old * 0.5;
                let take = old * 0.5;

                next[anchor_index] = keep;
                next.insert(insert_index.min(next.len()), take);

                let computed = split_geometry::compute_layout(
                    *axis,
                    bounds,
                    children.len().saturating_add(1),
                    &next,
                    split_handle_gap,
                    split_handle_hit_thickness,
                    &[],
                );
                computed.panel_rects.get(insert_index).copied()
            });
            preview.unwrap_or_else(|| drop_zone_rect(rect, target.zone))
        }
        _ => drop_zone_rect(rect, target.zone),
    };
    vec![ComplexDropOverlayPaintInput::EdgeZone { overlay }]
}

pub(in crate::dock) fn paint_complex_drop_overlay_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[ComplexDropOverlayPaintInput],
    scene: &mut Scene,
) {
    let primary = theme.color_token("primary");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let primary_alpha = |alpha: f32| Color {
        a: alpha,
        ..primary
    };
    let overlay_zone_bg = theme
        .color_by_key("component.docking.drop_overlay.zone.bg")
        .unwrap_or_else(|| primary_alpha(0.16));
    let overlay_zone_border = theme
        .color_by_key("component.docking.drop_overlay.zone.border")
        .unwrap_or_else(|| primary_alpha(0.85));

    let tab_insert_marker_bg = theme
        .color_by_key("component.docking.tab_insert.marker.bg")
        .unwrap_or_else(|| primary_alpha(0.85));
    let tab_insert_marker_border = theme
        .color_by_key("component.docking.tab_insert.marker.border")
        .unwrap_or_else(|| primary_alpha(1.0));
    let tab_insert_marker_cap_bg = theme
        .color_by_key("component.docking.tab_insert.marker.cap.bg")
        .unwrap_or_else(|| primary_alpha(0.92));

    for input in inputs {
        match input {
            ComplexDropOverlayPaintInput::TabInsertMarker { marker, caps } => {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(10_000),
                    rect: *marker,
                    background: fret_core::Paint::Solid(tab_insert_marker_bg).into(),
                    border: Edges::all(Px(1.0)),
                    border_paint: fret_core::Paint::Solid(tab_insert_marker_border).into(),
                    corner_radii: fret_core::Corners::all(Px(3.0)),
                });

                for &cap in caps {
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(10_001),
                        rect: cap,
                        background: fret_core::Paint::Solid(tab_insert_marker_cap_bg).into(),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),
                        corner_radii: fret_core::Corners::all(Px(2.0)),
                    });
                }
            }
            ComplexDropOverlayPaintInput::EdgeZone { overlay } => {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(10_000),
                    rect: *overlay,
                    background: fret_core::Paint::Solid(overlay_zone_bg).into(),
                    border: Edges::all(Px(2.0)),
                    border_paint: fret_core::Paint::Solid(overlay_zone_border).into(),
                    corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
                });
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::dock) fn paint_tab_insert_preview_title(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    layout: &HashMap<DockNodeId, Rect>,
    tab_count: usize,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    drag_source_tabs: Option<DockNodeId>,
    drag_tab_title: Option<&PreparedTabTitle>,
    close_glyph_present: bool,
    scene: &mut Scene,
) {
    let Some(DockDropTarget::Dock(target)) = target else {
        return;
    };
    if target.zone != DropZone::Center {
        return;
    }
    let Some(title) = drag_tab_title else {
        return;
    };
    if drag_source_tabs.is_some_and(|src| src == target.tabs) {
        return;
    }
    let Some(rect) = layout.get(&target.tabs).copied() else {
        return;
    };

    let primary = theme.color_token("primary");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let primary_alpha = |alpha: f32| Color {
        a: alpha,
        ..primary
    };
    let tab_insert_preview_bg = theme
        .color_by_key("component.docking.tab_insert.preview.bg")
        .unwrap_or_else(|| primary_alpha(0.22));
    let tab_insert_preview_border = theme
        .color_by_key("component.docking.tab_insert.preview.border")
        .unwrap_or_else(|| primary_alpha(0.85));

    let (tab_bar, _content) = split_tab_bar(rect);
    let scroll = tab_scroll_for_node(tab_scroll, target.tabs);
    let strip_candidate = tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
    let geom_candidate = tab_widths
        .get(&target.tabs)
        .filter(|w| w.len() == tab_count)
        .map(|w| TabBarGeometry::variable(strip_candidate, w.clone()))
        .unwrap_or_else(|| TabBarGeometry::fixed(strip_candidate, tab_count));
    let overflow = geom_candidate.max_scroll().0 > 0.0;
    let geom = if overflow {
        geom_candidate
    } else {
        tab_widths
            .get(&target.tabs)
            .filter(|w| w.len() == tab_count)
            .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
            .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count))
    };
    let tab_strip = if overflow { strip_candidate } else { tab_bar };

    let insert_index = target.insert_index.unwrap_or(tab_count);
    let mut x = geom.insert_x(insert_index.min(tab_count), scroll).0;
    let mut w =
        dock_tab_width_for_title(theme.clone(), title.metrics.size.width, close_glyph_present).0;

    let min_x = tab_strip.origin.x.0;
    let max_x = tab_strip.origin.x.0 + tab_strip.size.width.0;
    if x < min_x {
        x = min_x;
    }
    if x > max_x {
        x = max_x;
    }
    w = w.max(0.0).min((max_x - x).max(0.0));
    if w <= 6.0 {
        return;
    }

    let preview = Rect::new(
        Point::new(Px(x), tab_strip.origin.y),
        Size::new(Px(w), tab_strip.size.height),
    );
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(9_995),
        rect: preview,
        background: fret_core::Paint::Solid(tab_insert_preview_bg).into(),
        border: Edges::all(Px(2.0)),
        border_paint: fret_core::Paint::Solid(tab_insert_preview_border).into(),
        corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
    });

    let pad_x = theme.metric_token("metric.padding.md");
    let text_x = Px(preview.origin.x.0 + pad_x.0.max(0.0));
    let inner_y =
        preview.origin.y.0 + ((preview.size.height.0 - title.metrics.size.height.0) * 0.5);
    let text_y = Px(inner_y + title.metrics.baseline.0);
    let fg = theme.color_token("foreground");
    scene.push(SceneOp::PushClipRect { rect: preview });
    scene.push(SceneOp::Text {
        order: fret_core::DrawOrder(9_996),
        origin: Point::new(text_x, text_y),
        text: title.blob,
        paint: (Color { a: 0.92, ..fg }).into(),
        outline: None,
        shadow: None,
    });
    scene.push(SceneOp::PopClip);
}
