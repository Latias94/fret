use std::collections::HashMap;

use fret_core::{AppWindowId, Rect};
use fret_ui::UiHost;

use super::super::host_frame::DockSpaceLayoutSnapshot;
use super::super::manager::DockManager;
use super::super::paint::FloatingChromePaintInput;
use super::super::services::DockingPolicyService;
use super::super::types::{DockDropTarget, HoverTarget};
use super::frame::DockSpaceElementFrame;
use super::geometry::declarative_layout_snapshot_for_bounds;
use super::interaction::{DeclarativeDockInteractionService, DeclarativeFloatingHover};

pub(super) fn declarative_floating_hover_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> DeclarativeFloatingHover {
    app.global::<DeclarativeDockInteractionService>()
        .map(|service| service.floating_hover(window))
        .unwrap_or_default()
}

pub(super) fn apply_declarative_floating_hover_paint_state(
    frame: &DockSpaceElementFrame,
    hover: DeclarativeFloatingHover,
    inputs: &mut [FloatingChromePaintInput],
) {
    for (node, input) in frame.floating_chrome_nodes.iter().zip(inputs.iter_mut()) {
        input.title_bar_hovered = hover.title_bar == Some(*node);
        input.close_hovered = hover.close == Some(*node);
    }
}

pub(super) fn floating_chrome_paint_inputs(
    snapshot: &DockSpaceLayoutSnapshot,
    pressed_floating_close: Option<fret_core::DockNodeId>,
    floating_hover: DeclarativeFloatingHover,
) -> Vec<FloatingChromePaintInput> {
    snapshot
        .floating_layouts
        .iter()
        .map(|floating| FloatingChromePaintInput {
            outer: floating.chrome.outer,
            title_bar: floating.chrome.title_bar,
            close_button: floating.chrome.close_button,
            title_bar_hovered: floating_hover.title_bar == Some(floating.floating.floating),
            close_hovered: floating_hover.close == Some(floating.floating.floating),
            close_pressed: pressed_floating_close == Some(floating.floating.floating),
        })
        .collect()
}

pub(super) fn declarative_pressed_floating_close_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Option<fret_core::DockNodeId> {
    app.global::<DeclarativeDockInteractionService>()
        .and_then(|service| service.pressed_floating_close(window))
}

pub(super) fn declarative_hit_test_floating_close(
    snapshot: &DockSpaceLayoutSnapshot,
    position: fret_core::Point,
) -> Option<fret_core::DockNodeId> {
    for floating in snapshot.floating_layouts.iter().rev() {
        if !floating.chrome.outer.contains(position) {
            continue;
        }
        return floating
            .chrome
            .close_button
            .contains(position)
            .then_some(floating.floating.floating);
    }
    None
}

pub(super) fn declarative_hit_test_floating_title_bar(
    snapshot: &DockSpaceLayoutSnapshot,
    position: fret_core::Point,
) -> Option<(fret_core::DockNodeId, fret_core::Point, Rect)> {
    for floating in snapshot.floating_layouts.iter().rev() {
        if !floating.chrome.outer.contains(position) {
            continue;
        }
        if floating.chrome.close_button.contains(position) {
            return None;
        }
        if floating.chrome.title_bar.contains(position) {
            let rect = floating.floating.rect;
            let grab_offset = fret_core::Point::new(
                fret_core::Px(position.x.0 - rect.origin.x.0),
                fret_core::Px(position.y.0 - rect.origin.y.0),
            );
            return Some((floating.floating.floating, grab_offset, rect));
        }
        return None;
    }
    None
}

fn declarative_leaf_tabs_node_at_pos(
    graph: &fret_core::DockGraph,
    layout: &HashMap<fret_core::DockNodeId, Rect>,
    position: fret_core::Point,
) -> Option<(fret_core::DockNodeId, Rect)> {
    let mut best: Option<(fret_core::DockNodeId, Rect, f32)> = None;
    for (&node, &rect) in layout {
        let Some(fret_core::DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if tabs.is_empty() || !rect.contains(position) {
            continue;
        }
        let area = rect.size.width.0 * rect.size.height.0;
        match best {
            None => best = Some((node, rect, area)),
            Some((_node, _rect, best_area)) if area < best_area => {
                best = Some((node, rect, area));
            }
            _ => {}
        }
    }
    best.map(|(node, rect, _area)| (node, rect))
}

pub(super) fn declarative_resolve_floating_title_bar_drag_target<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    dock_previews_enabled: bool,
    position: fret_core::Point,
) -> Option<DockDropTarget> {
    if !dock_previews_enabled {
        return Some(DockDropTarget::Float { window });
    }
    let dock = app.global::<DockManager>()?;
    let snapshot = declarative_layout_snapshot_for_bounds(app, window, bounds)?;
    let root = snapshot.root?;
    if super::super::layout::float_zone(snapshot.dock_bounds).contains(position) {
        return Some(DockDropTarget::Float { window });
    }
    if !snapshot.dock_bounds.contains(position) || !bounds.contains(position) {
        return Some(DockDropTarget::Float { window });
    }
    let (tabs, rect) =
        declarative_leaf_tabs_node_at_pos(&dock.workspace.graph, &snapshot.root_layout, position)?;

    let font_size = theme.metric_token("font.size");
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let hint_font_size_inner =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_inner.max(0.0)).max(0.0));
    let hint_font_size_outer =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_outer.max(0.0)).max(0.0));

    let target = if let Some(root_rect) = snapshot.root_layout.get(&root).copied()
        && root != tabs
        && let Some(zone) = super::super::layout::dock_hint_pick_zone(
            root_rect,
            hint_font_size_outer,
            true,
            position,
        )
        && zone != fret_core::DropZone::Center
    {
        HoverTarget {
            tabs: root,
            root,
            leaf_tabs: tabs,
            zone,
            insert_index: None,
            outer: true,
            explicit: true,
        }
    } else if let Some(zone) =
        super::super::layout::dock_hint_pick_zone(rect, hint_font_size_inner, false, position)
    {
        HoverTarget {
            tabs,
            root,
            leaf_tabs: tabs,
            zone,
            insert_index: None,
            outer: false,
            explicit: true,
        }
    } else {
        return None;
    };

    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    if policy.as_deref().is_some_and(|policy| {
        !policy.allow_dock_drop_target(window, target.root, target.tabs, target.zone, target.outer)
    }) {
        return None;
    }
    Some(DockDropTarget::Dock(target))
}
