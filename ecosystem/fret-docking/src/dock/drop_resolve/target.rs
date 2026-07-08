// This file is part of the docking UI implementation.
//
// It owns dock drop target resolution and leaves drop intent/effect projection in the parent.

use super::super::DockingPolicy;
use super::super::hit_test::tab_scroll_for_node;
use super::super::layout::{
    compute_layout_map, dock_hint_pick_zone, dock_hint_rects_with_font, float_zone, split_tab_bar,
};
use super::super::prelude_core::*;
use super::super::tab_bar_drop_target::tab_bar_insert_index_for_drop;
use super::super::types::{
    DockDropPolicyDecision, DockDropTarget, DockDropTargetResolution, HoverTarget,
};
use super::floating_hit::{FloatingHitKind, hit_test_floating, layout_context_for_position};

#[allow(clippy::too_many_arguments)]
fn dock_drop_target(
    graph: &DockGraph,
    root: DockNodeId,
    layout: &HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    theme: fret_ui::ThemeSnapshot,
    hint_font_size_inner: Px,
    hint_font_size_outer: Px,
    position: Point,
    dragged_tab_for_drop: Option<(DockNodeId, usize)>,
    mut candidates: Option<&mut Vec<fret_runtime::DockDropCandidateRectDiagnostics>>,
) -> Option<(HoverTarget, fret_runtime::DockDropResolveSource)> {
    fn leaf_tabs_node_at_pos(
        graph: &DockGraph,
        layout: &HashMap<DockNodeId, Rect>,
        position: Point,
    ) -> Option<(DockNodeId, Rect, usize)> {
        let mut best: Option<(DockNodeId, Rect, usize, f32)> = None;
        for (&node, &rect) in layout.iter() {
            let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
                continue;
            };
            if tabs.is_empty() || !rect.contains(position) {
                continue;
            }
            let area = rect.size.width.0 * rect.size.height.0;
            match best {
                None => best = Some((node, rect, tabs.len(), area)),
                Some((_best_node, _best_rect, _best_len, best_area)) => {
                    if area < best_area {
                        best = Some((node, rect, tabs.len(), area));
                    }
                }
            }
        }
        best.map(|(node, rect, len, _)| (node, rect, len))
    }

    let leaf = leaf_tabs_node_at_pos(graph, layout, position);
    if let Some((tabs_node, rect, tab_count)) = leaf {
        if let Some(candidates) = candidates.as_mut() {
            candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                kind: fret_runtime::DockDropCandidateRectKind::LeafTabsRect,
                zone: None,
                rect,
            });
        }
        let (tab_bar, _content) = split_tab_bar(rect);
        if tab_bar.contains(position) {
            if let Some(candidates) = candidates.as_mut() {
                candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                    kind: fret_runtime::DockDropCandidateRectKind::TabBarRect,
                    zone: None,
                    rect: tab_bar,
                });
                if let Some(&root_rect) = layout.get(&root) {
                    candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                        kind: fret_runtime::DockDropCandidateRectKind::RootRect,
                        zone: None,
                        rect: root_rect,
                    });
                    for (z, r) in dock_hint_rects_with_font(root_rect, hint_font_size_outer, true) {
                        candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                            kind: fret_runtime::DockDropCandidateRectKind::OuterHintRect,
                            zone: Some(z),
                            rect: r,
                        });
                    }
                }
                for (z, r) in dock_hint_rects_with_font(rect, hint_font_size_inner, false) {
                    candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                        kind: fret_runtime::DockDropCandidateRectKind::InnerHintRect,
                        zone: Some(z),
                        rect: r,
                    });
                }
            }
            let scroll = tab_scroll_for_node(tab_scroll, tabs_node);
            let dragged_tab_index = dragged_tab_for_drop
                .and_then(|(source_tabs, index)| (source_tabs == tabs_node).then_some(index));
            let insert_index = tab_bar_insert_index_for_drop(
                theme.clone(),
                tab_bar,
                tab_count,
                tab_widths.get(&tabs_node),
                scroll,
                position,
                dragged_tab_index,
            )?;
            return Some((
                HoverTarget {
                    tabs: tabs_node,
                    root,
                    leaf_tabs: tabs_node,
                    zone: DropZone::Center,
                    insert_index: Some(insert_index),
                    outer: false,
                    explicit: false,
                },
                fret_runtime::DockDropResolveSource::TabBar,
            ));
        }
    }

    if let Some(candidates) = candidates.as_mut() {
        if let Some(&root_rect) = layout.get(&root) {
            candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                kind: fret_runtime::DockDropCandidateRectKind::RootRect,
                zone: None,
                rect: root_rect,
            });
            for (z, r) in dock_hint_rects_with_font(root_rect, hint_font_size_outer, true) {
                candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                    kind: fret_runtime::DockDropCandidateRectKind::OuterHintRect,
                    zone: Some(z),
                    rect: r,
                });
            }
        }
        if let Some((_tabs_node, rect, _tab_count)) = leaf {
            for (z, r) in dock_hint_rects_with_font(rect, hint_font_size_inner, false) {
                candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                    kind: fret_runtime::DockDropCandidateRectKind::InnerHintRect,
                    zone: Some(z),
                    rect: r,
                });
            }
        }
    }

    if let Some(&root_rect) = layout.get(&root)
        && let Some((leaf_tabs, _leaf_rect, _leaf_tab_count)) = leaf
        && root != leaf_tabs
        && let Some(zone) = dock_hint_pick_zone(root_rect, hint_font_size_outer, true, position)
        && zone != DropZone::Center
    {
        return Some((
            HoverTarget {
                tabs: root,
                root,
                leaf_tabs,
                zone,
                insert_index: None,
                outer: true,
                explicit: true,
            },
            fret_runtime::DockDropResolveSource::OuterHintRect,
        ));
    }

    if let Some((tabs_node, rect, _tab_count)) = leaf
        && let Some(zone) = dock_hint_pick_zone(rect, hint_font_size_inner, false, position)
    {
        return Some((
            HoverTarget {
                tabs: tabs_node,
                root,
                leaf_tabs: tabs_node,
                zone,
                insert_index: None,
                outer: false,
                explicit: true,
            },
            fret_runtime::DockDropResolveSource::InnerHintRect,
        ));
    }

    None
}

fn clamp_point_inside_rect(rect: Rect, point: Point) -> Point {
    const EPS: f32 = 0.001;
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;

    let max_x = if x1 > x0 { (x1 - EPS).max(x0) } else { x0 };
    let max_y = if y1 > y0 { (y1 - EPS).max(y0) } else { y0 };

    Point::new(
        Px(point.x.0.clamp(x0, max_x)),
        Px(point.y.0.clamp(y0, max_y)),
    )
}

#[allow(clippy::too_many_arguments)]
fn compute_dock_drop_target(
    graph: &DockGraph,
    window: fret_core::AppWindowId,
    root: Option<DockNodeId>,
    dock_bounds: Rect,
    window_bounds: Rect,
    allow_floating_hit_test: bool,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    theme: fret_ui::ThemeSnapshot,
    hint_font_size_inner: Px,
    hint_font_size_outer: Px,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
    position: Point,
    dragged_tab_for_drop: Option<(DockNodeId, usize)>,
    mut candidates: Option<&mut Vec<fret_runtime::DockDropCandidateRectDiagnostics>>,
) -> (Option<DockDropTarget>, fret_runtime::DockDropResolveSource) {
    if let Some(candidates) = candidates.as_mut() {
        candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
            kind: fret_runtime::DockDropCandidateRectKind::WindowBounds,
            zone: None,
            rect: window_bounds,
        });
        candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
            kind: fret_runtime::DockDropCandidateRectKind::DockBounds,
            zone: None,
            rect: dock_bounds,
        });
        candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
            kind: fret_runtime::DockDropCandidateRectKind::FloatZone,
            zone: None,
            rect: float_zone(dock_bounds),
        });
    }
    if !window_bounds.contains(position) {
        return (
            Some(DockDropTarget::Float { window }),
            fret_runtime::DockDropResolveSource::OutsideWindow,
        );
    }
    if float_zone(dock_bounds).contains(position) {
        return (
            Some(DockDropTarget::Float { window }),
            fret_runtime::DockDropResolveSource::FloatZone,
        );
    }

    if allow_floating_hit_test
        && let Some((floating, chrome, FloatingHitKind::TitleBar)) =
            hit_test_floating(graph, window, position)
    {
        let layout_bounds = chrome.inner;
        let layout = compute_layout_map(
            graph,
            floating,
            layout_bounds,
            split_handle_gap,
            split_handle_hit_thickness,
        );
        let center = Point::new(
            Px(layout_bounds.origin.x.0 + layout_bounds.size.width.0 * 0.5),
            Px(layout_bounds.origin.y.0 + layout_bounds.size.height.0 * 0.5),
        );
        let mut best: Option<(DockNodeId, f32)> = None;
        for (&node_id, &rect) in layout.iter() {
            if !rect.contains(center) {
                continue;
            }
            let Some(DockNode::Tabs { tabs, .. }) = graph.node(node_id) else {
                continue;
            };
            if tabs.is_empty() {
                continue;
            }
            let area = rect.size.width.0 * rect.size.height.0;
            match best {
                None => best = Some((node_id, area)),
                Some((_best_node, best_area)) => {
                    if area < best_area {
                        best = Some((node_id, area));
                    }
                }
            }
        }

        if let Some((leaf_tabs, _area)) = best {
            return (
                Some(DockDropTarget::Dock(HoverTarget {
                    tabs: leaf_tabs,
                    root: floating,
                    leaf_tabs,
                    zone: DropZone::Center,
                    insert_index: None,
                    outer: false,
                    explicit: false,
                })),
                fret_runtime::DockDropResolveSource::FloatingTitleBar,
            );
        }
        return (None, fret_runtime::DockDropResolveSource::None);
    }

    let (layout_root, layout_bounds, effective_position) = if allow_floating_hit_test {
        match hit_test_floating(graph, window, position) {
            None | Some((_, _, FloatingHitKind::Close)) => {
                let (layout_root, layout_bounds) =
                    layout_context_for_position(graph, window, root, dock_bounds, position);
                (layout_root, layout_bounds, position)
            }
            Some((floating, chrome, FloatingHitKind::TitleBar)) => {
                let projected = Point::new(
                    Px(chrome.inner.origin.x.0 + chrome.inner.size.width.0 * 0.5),
                    Px(chrome.inner.origin.y.0 + chrome.inner.size.height.0 * 0.5),
                );
                (
                    Some(floating),
                    chrome.inner,
                    clamp_point_inside_rect(chrome.inner, projected),
                )
            }
            Some((floating, chrome, FloatingHitKind::Body)) => (
                Some(floating),
                chrome.inner,
                clamp_point_inside_rect(chrome.inner, position),
            ),
        }
    } else {
        (root, dock_bounds, position)
    };

    if !layout_bounds.contains(effective_position) {
        if let Some(candidates) = candidates.as_mut() {
            candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                kind: fret_runtime::DockDropCandidateRectKind::LayoutBounds,
                zone: None,
                rect: layout_bounds,
            });
        }
        return (None, fret_runtime::DockDropResolveSource::LayoutBoundsMiss);
    }

    let Some(layout_root) = layout_root else {
        if dock_bounds.contains(position) {
            return (
                Some(DockDropTarget::EmptyDockSpace { window }),
                fret_runtime::DockDropResolveSource::EmptyDockSpace,
            );
        }
        return (None, fret_runtime::DockDropResolveSource::LayoutBoundsMiss);
    };

    let layout = compute_layout_map(
        graph,
        layout_root,
        layout_bounds,
        split_handle_gap,
        split_handle_hit_thickness,
    );
    dock_drop_target(
        graph,
        layout_root,
        &layout,
        tab_scroll,
        tab_widths,
        theme,
        hint_font_size_inner,
        hint_font_size_outer,
        effective_position,
        dragged_tab_for_drop,
        candidates,
    )
    .map(|(target, source)| (Some(DockDropTarget::Dock(target)), source))
    .unwrap_or((None, fret_runtime::DockDropResolveSource::None))
}

#[allow(clippy::too_many_arguments)]
pub(in crate::dock) fn resolve_dock_drop_target(
    prev_hover: Option<DockDropTarget>,
    invert_docking: bool,
    allow_floating_hit_test: bool,
    window: fret_core::AppWindowId,
    docking_policy: Option<&dyn DockingPolicy>,
    graph: &DockGraph,
    root: Option<DockNodeId>,
    dock_bounds: Rect,
    window_bounds: Rect,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    theme: fret_ui::ThemeSnapshot,
    hint_font_size_inner: Px,
    hint_font_size_outer: Px,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
    position: Point,
    dragged_tab_for_drop: Option<(DockNodeId, usize)>,
    candidates: Option<&mut Vec<fret_runtime::DockDropCandidateRectDiagnostics>>,
) -> DockDropTargetResolution {
    if invert_docking {
        return DockDropTargetResolution {
            target: Some(DockDropTarget::Float { window }),
            source: fret_runtime::DockDropResolveSource::InvertDocking,
            policy: DockDropPolicyDecision::NotApplicable,
        };
    }
    let (target, source) = if let Some(prev_hover) = prev_hover {
        (
            Some(prev_hover),
            fret_runtime::DockDropResolveSource::LatchedPreviousHover,
        )
    } else {
        compute_dock_drop_target(
            graph,
            window,
            root,
            dock_bounds,
            window_bounds,
            allow_floating_hit_test,
            tab_scroll,
            tab_widths,
            theme,
            hint_font_size_inner,
            hint_font_size_outer,
            split_handle_gap,
            split_handle_hit_thickness,
            position,
            dragged_tab_for_drop,
            candidates,
        )
    };

    if let (Some(DockDropTarget::Dock(t)), Some(policy)) = (target.as_ref(), docking_policy)
        && !policy.allow_dock_drop_target(window, t.root, t.tabs, t.zone, t.outer)
    {
        return DockDropTargetResolution {
            target: None,
            source,
            policy: DockDropPolicyDecision::Denied { target: *t },
        };
    }

    let policy = match target.as_ref() {
        Some(DockDropTarget::Dock(_)) => DockDropPolicyDecision::Allowed,
        _ => DockDropPolicyDecision::NotApplicable,
    };

    DockDropTargetResolution {
        target,
        source,
        policy,
    }
}
