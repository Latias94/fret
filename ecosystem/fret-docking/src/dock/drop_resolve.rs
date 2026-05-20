// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::DockingPolicy;
use super::hit_test::tab_scroll_for_node;
use super::host_frame::{FloatingChrome, floating_chrome};
use super::layout::{
    compute_layout_map, dock_hint_pick_zone, dock_hint_rects_with_font, float_zone, split_tab_bar,
};
use super::prelude_core::*;
use super::prelude_runtime::*;
use super::tab_bar_drop_target::tab_bar_insert_index_for_drop;
use super::types::{DockDropTarget, HoverTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FloatingHitKind {
    Close,
    TitleBar,
    Body,
}

pub(super) fn hit_test_floating(
    graph: &DockGraph,
    window: fret_core::AppWindowId,
    position: Point,
) -> Option<(DockNodeId, FloatingChrome, FloatingHitKind)> {
    for floating in graph.floating_windows(window).iter().rev() {
        let chrome = floating_chrome(floating.rect);
        if !chrome.outer.contains(position) {
            continue;
        }
        if chrome.close_button.contains(position) {
            return Some((floating.floating, chrome, FloatingHitKind::Close));
        }
        if chrome.title_bar.contains(position) {
            return Some((floating.floating, chrome, FloatingHitKind::TitleBar));
        }
        return Some((floating.floating, chrome, FloatingHitKind::Body));
    }
    None
}

pub(super) fn layout_context_for_position(
    graph: &DockGraph,
    window: fret_core::AppWindowId,
    root: Option<DockNodeId>,
    dock_bounds: Rect,
    position: Point,
) -> (Option<DockNodeId>, Rect) {
    if let Some((floating, chrome, _)) = hit_test_floating(graph, window, position)
        && chrome.inner.contains(position)
    {
        return (Some(floating), chrome.inner);
    }
    (root, dock_bounds)
}

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
pub(super) fn resolve_dock_drop_target(
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
) -> (Option<DockDropTarget>, fret_runtime::DockDropResolveSource) {
    if invert_docking {
        return (
            Some(DockDropTarget::Float { window }),
            fret_runtime::DockDropResolveSource::InvertDocking,
        );
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
        return (None, source);
    }

    (target, source)
}

#[derive(Clone, Copy)]
pub(super) struct DockPanelDropDrag<'a> {
    pub(super) source_window: fret_core::AppWindowId,
    pub(super) panel: &'a PanelKey,
    pub(super) grab_offset: Point,
    pub(super) tear_off_requested: bool,
}

#[derive(Clone, Copy)]
pub(super) struct DockTabsDropDrag<'a> {
    pub(super) source_window: fret_core::AppWindowId,
    pub(super) source_tabs: DockNodeId,
    pub(super) tabs: &'a [PanelKey],
    pub(super) active: usize,
    pub(super) grab_offset: Point,
    pub(super) tear_off_requested: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_dock_drop_intent_panel<F>(
    target: Option<DockDropTarget>,
    drag: DockPanelDropDrag<'_>,
    target_window: fret_core::AppWindowId,
    window_bounds: Rect,
    position: Point,
    allow_tear_off: bool,
    mark_drag_tear_off_requested: bool,
    default_floating_rect_for_panel: F,
) -> DockDropIntent
where
    F: FnOnce(&PanelKey, Point, Point, Rect) -> Rect,
{
    match target {
        Some(DockDropTarget::Dock(target)) => DockDropIntent::MovePanel {
            source_window: drag.source_window,
            panel: drag.panel.clone(),
            target_window,
            target_tabs: target.tabs,
            zone: target.zone,
            insert_index: target.insert_index,
        },
        Some(DockDropTarget::EmptyDockSpace { .. }) => DockDropIntent::MovePanelToEmptyDockSpace {
            source_window: drag.source_window,
            panel: drag.panel.clone(),
            target_window,
        },
        Some(DockDropTarget::Float { .. }) => {
            let wants_tear_off = allow_tear_off && !window_bounds.contains(position);
            if wants_tear_off {
                if drag.tear_off_requested || mark_drag_tear_off_requested {
                    DockDropIntent::None
                } else {
                    DockDropIntent::RequestFloatPanelToNewWindow {
                        source_window: drag.source_window,
                        panel: drag.panel.clone(),
                        anchor: Some(fret_core::WindowAnchor {
                            window: target_window,
                            position: drag.grab_offset,
                        }),
                    }
                }
            } else {
                let rect = default_floating_rect_for_panel(
                    drag.panel,
                    position,
                    drag.grab_offset,
                    window_bounds,
                );
                DockDropIntent::FloatPanelInWindow {
                    source_window: drag.source_window,
                    panel: drag.panel.clone(),
                    target_window,
                    rect,
                }
            }
        }
        None => {
            let wants_tear_off = allow_tear_off && !window_bounds.contains(position);
            if wants_tear_off {
                if drag.tear_off_requested || mark_drag_tear_off_requested {
                    DockDropIntent::None
                } else {
                    DockDropIntent::RequestFloatPanelToNewWindow {
                        source_window: drag.source_window,
                        panel: drag.panel.clone(),
                        anchor: Some(fret_core::WindowAnchor {
                            window: target_window,
                            position: drag.grab_offset,
                        }),
                    }
                }
            } else {
                DockDropIntent::None
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_dock_drop_intent_tabs<F>(
    target: Option<DockDropTarget>,
    drag: DockTabsDropDrag<'_>,
    target_window: fret_core::AppWindowId,
    window_bounds: Rect,
    position: Point,
    allow_tear_off: bool,
    mark_drag_tear_off_requested: bool,
    default_floating_rect_for_panel: F,
) -> DockDropIntent
where
    F: FnOnce(&PanelKey, Point, Point, Rect) -> Rect,
{
    match target {
        Some(DockDropTarget::Dock(target)) => DockDropIntent::MoveTabs {
            source_window: drag.source_window,
            source_tabs: drag.source_tabs,
            target_window,
            target_tabs: target.tabs,
            zone: target.zone,
            insert_index: target.insert_index,
        },
        Some(DockDropTarget::EmptyDockSpace { .. }) => DockDropIntent::MoveTabsToEmptyDockSpace {
            source_window: drag.source_window,
            source_tabs: drag.source_tabs,
            target_window,
        },
        Some(DockDropTarget::Float { .. }) => {
            let wants_tear_off = allow_tear_off && !window_bounds.contains(position);
            let panel = drag
                .tabs
                .get(drag.active)
                .or_else(|| drag.tabs.first())
                .cloned();
            let Some(panel) = panel else {
                return DockDropIntent::None;
            };
            if wants_tear_off {
                if drag.tear_off_requested || mark_drag_tear_off_requested {
                    DockDropIntent::None
                } else {
                    DockDropIntent::RequestFloatTabsToNewWindow {
                        source_window: drag.source_window,
                        source_tabs: drag.source_tabs,
                        panel,
                        anchor: Some(fret_core::WindowAnchor {
                            window: target_window,
                            position: drag.grab_offset,
                        }),
                    }
                }
            } else {
                let rect = default_floating_rect_for_panel(
                    &panel,
                    position,
                    drag.grab_offset,
                    window_bounds,
                );
                DockDropIntent::FloatTabsInWindow {
                    source_window: drag.source_window,
                    source_tabs: drag.source_tabs,
                    target_window,
                    rect,
                }
            }
        }
        None => DockDropIntent::None,
    }
}

pub(super) fn apply_dock_drop_intent(
    intent: DockDropIntent,
    pending_effects: &mut Vec<Effect>,
    invalidate_layout: &mut bool,
) {
    match intent {
        DockDropIntent::None => {}
        DockDropIntent::MovePanel {
            source_window,
            panel,
            target_window,
            target_tabs,
            zone,
            insert_index,
        } => {
            pending_effects.push(Effect::Dock(DockOp::MovePanel {
                source_window,
                panel,
                target_window,
                target_tabs,
                zone,
                insert_index,
            }));
            *invalidate_layout = true;
        }
        DockDropIntent::MovePanelToEmptyDockSpace {
            source_window,
            panel,
            target_window,
        } => {
            pending_effects.push(Effect::Dock(DockOp::MovePanelToEmptyDockSpace {
                source_window,
                panel,
                target_window,
            }));
            *invalidate_layout = true;
        }
        DockDropIntent::MoveTabs {
            source_window,
            source_tabs,
            target_window,
            target_tabs,
            zone,
            insert_index,
        } => {
            pending_effects.push(Effect::Dock(DockOp::MoveTabs {
                source_window,
                source_tabs,
                target_window,
                target_tabs,
                zone,
                insert_index,
            }));
            *invalidate_layout = true;
        }
        DockDropIntent::MoveTabsToEmptyDockSpace {
            source_window,
            source_tabs,
            target_window,
        } => {
            pending_effects.push(Effect::Dock(DockOp::MoveTabsToEmptyDockSpace {
                source_window,
                source_tabs,
                target_window,
            }));
            *invalidate_layout = true;
        }
        DockDropIntent::FloatPanelInWindow {
            source_window,
            panel,
            target_window,
            rect,
        } => {
            pending_effects.push(Effect::Dock(DockOp::FloatPanelInWindow {
                source_window,
                panel,
                target_window,
                rect,
            }));
            *invalidate_layout = true;
        }
        DockDropIntent::FloatTabsInWindow {
            source_window,
            source_tabs,
            target_window,
            rect,
        } => {
            pending_effects.push(Effect::Dock(DockOp::FloatTabsInWindow {
                source_window,
                source_tabs,
                target_window,
                rect,
            }));
            *invalidate_layout = true;
        }
        DockDropIntent::RequestFloatPanelToNewWindow {
            source_window,
            panel,
            anchor,
        } => {
            pending_effects.push(Effect::Dock(DockOp::RequestFloatPanelToNewWindow {
                source_window,
                panel,
                anchor,
            }));
            *invalidate_layout = true;
        }
        DockDropIntent::RequestFloatTabsToNewWindow {
            source_window,
            source_tabs,
            panel,
            anchor,
        } => {
            pending_effects.push(Effect::Dock(DockOp::RequestFloatTabsToNewWindow {
                source_window,
                source_tabs,
                panel,
                anchor,
            }));
            *invalidate_layout = true;
        }
    }
}

pub(super) fn dock_drop_intent_debug_kind(intent: &DockDropIntent) -> &'static str {
    match intent {
        DockDropIntent::None => "none",
        DockDropIntent::MovePanel { .. } => "move_panel",
        DockDropIntent::MovePanelToEmptyDockSpace { .. } => "move_panel_to_empty_dock_space",
        DockDropIntent::MoveTabs { .. } => "move_tabs",
        DockDropIntent::MoveTabsToEmptyDockSpace { .. } => "move_tabs_to_empty_dock_space",
        DockDropIntent::FloatPanelInWindow { .. } => "float_panel_in_window",
        DockDropIntent::FloatTabsInWindow { .. } => "float_tabs_in_window",
        DockDropIntent::RequestFloatPanelToNewWindow { .. } => "request_float_panel_to_new_window",
        DockDropIntent::RequestFloatTabsToNewWindow { .. } => "request_float_tabs_to_new_window",
    }
}

pub(super) fn dock_drop_target_diagnostics(
    target: Option<&DockDropTarget>,
) -> Option<fret_runtime::DockDropTargetDiagnostics> {
    match target {
        Some(DockDropTarget::Dock(t)) => Some(fret_runtime::DockDropTargetDiagnostics {
            layout_root: t.root,
            tabs: t.tabs,
            zone: t.zone,
            insert_index: t.insert_index,
            outer: t.outer,
        }),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compute_dock_drop_resolve_diagnostics(
    pointer_id: fret_core::PointerId,
    position: Point,
    window_bounds: Rect,
    dock_bounds: Rect,
    source: fret_runtime::DockDropResolveSource,
    graph: &DockGraph,
    window: fret_core::AppWindowId,
    target: Option<&DockDropTarget>,
    candidates: Vec<fret_runtime::DockDropCandidateRectDiagnostics>,
) -> fret_runtime::DockDropResolveDiagnostics {
    let preview = match target {
        Some(DockDropTarget::Dock(t)) if t.zone != DropZone::Center => {
            let kind = match graph.edge_dock_decision(window, t.tabs, t.zone) {
                Some(fret_core::EdgeDockDecision::InsertIntoSplit {
                    split,
                    insert_index,
                    ..
                }) => {
                    let axis = match graph.node(split) {
                        Some(DockNode::Split { axis, .. }) => *axis,
                        _ => match t.zone {
                            DropZone::Left | DropZone::Right => fret_core::Axis::Horizontal,
                            DropZone::Top | DropZone::Bottom => fret_core::Axis::Vertical,
                            DropZone::Center => fret_core::Axis::Horizontal,
                        },
                    };
                    fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit {
                        axis,
                        split,
                        insert_index,
                    }
                }
                _ => fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary,
            };
            Some(fret_runtime::DockDropPreviewDiagnostics { kind })
        }
        _ => None,
    };

    fret_runtime::DockDropResolveDiagnostics {
        pointer_id,
        position,
        window_bounds,
        dock_bounds,
        source,
        resolved: dock_drop_target_diagnostics(target),
        preview,
        candidates,
    }
}
