// This file is part of the docking UI implementation.
//
// It owns floating-window hit tests used by drop target resolution.

use super::super::host_frame::{FloatingChrome, floating_chrome};
use super::super::prelude_core::*;

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
