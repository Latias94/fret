// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::layout::split_tab_bar;
use super::prelude_core::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ViewportHit {
    pub(super) panel: PanelKey,
    pub(super) viewport: ViewportPanel,
    pub(super) content: Rect,
    pub(super) draw_rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ViewportCaptureState {
    pub(super) hit: ViewportHit,
    pub(super) button: fret_core::MouseButton,
    pub(super) start: Point,
    pub(super) moved: bool,
}

pub(super) fn viewport_input_from_hit(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    position: Point,
    kind: ViewportInputKind,
) -> Option<ViewportInputEvent> {
    let mapping = ViewportMapping {
        content_rect: hit.content,
        target_px_size: hit.viewport.target_px_size,
        fit: hit.viewport.fit,
    };
    let uv = mapping.window_point_to_uv(position)?;
    let target_px = mapping.window_point_to_target_px(position)?;
    Some(ViewportInputEvent {
        window,
        target: hit.viewport.target,
        uv,
        target_px,
        kind,
    })
}

pub(super) fn viewport_input_from_hit_clamped(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    position: Point,
    kind: ViewportInputKind,
) -> ViewportInputEvent {
    let mapping = ViewportMapping {
        content_rect: hit.content,
        target_px_size: hit.viewport.target_px_size,
        fit: hit.viewport.fit,
    };
    let uv = mapping.window_point_to_uv_clamped(position);
    let target_px = mapping.window_point_to_target_px_clamped(position);
    ViewportInputEvent {
        window,
        target: hit.viewport.target,
        uv,
        target_px,
        kind,
    }
}

pub(super) fn hit_test_active_viewport_panel(
    graph: &DockGraph,
    panels: &HashMap<PanelKey, DockPanel>,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<ViewportHit> {
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let Some(panel_key) = tabs.get(*active).cloned() else {
            continue;
        };
        let Some(panel) = panels.get(&panel_key) else {
            continue;
        };
        let Some(viewport) = panel.viewport else {
            continue;
        };

        let (_tab_bar, content) = split_tab_bar(rect);
        let mapping = ViewportMapping {
            content_rect: content,
            target_px_size: viewport.target_px_size,
            fit: viewport.fit,
        };
        let draw_rect = mapping.map().draw_rect;
        if draw_rect.contains(position) {
            return Some(ViewportHit {
                panel: panel_key,
                viewport,
                content,
                draw_rect,
            });
        }
    }
    None
}
