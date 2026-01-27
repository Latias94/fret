// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::layout::split_tab_bar;
use super::prelude_core::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ViewportHit {
    pub(super) panel: PanelKey,
    pub(super) viewport: ViewportPanel,
    pub(super) mapping: ViewportMapping,
    pub(super) draw_rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ViewportCaptureState {
    pub(super) pointer_id: fret_core::PointerId,
    pub(super) hit: ViewportHit,
    pub(super) button: fret_core::MouseButton,
    pub(super) start: Point,
    pub(super) last: Point,
    pub(super) moved: bool,
}

pub(super) fn viewport_input_from_hit(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    pixels_per_point: f32,
    pointer_id: fret_core::PointerId,
    pointer_type: fret_core::PointerType,
    position: Point,
    kind: ViewportInputKind,
) -> Option<ViewportInputEvent> {
    ViewportInputEvent::from_mapping_window_point_maybe_clamped(
        window,
        hit.viewport.target,
        &hit.mapping,
        pixels_per_point,
        pointer_id,
        pointer_type,
        position,
        kind,
        false,
    )
}

pub(super) fn viewport_input_from_hit_clamped(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    pixels_per_point: f32,
    pointer_id: fret_core::PointerId,
    pointer_type: fret_core::PointerType,
    position: Point,
    kind: ViewportInputKind,
) -> ViewportInputEvent {
    ViewportInputEvent::from_mapping_window_point_maybe_clamped(
        window,
        hit.viewport.target,
        &hit.mapping,
        pixels_per_point,
        pointer_id,
        pointer_type,
        position,
        kind,
        true,
    )
    .expect("clamped viewport mapping must always yield an input event")
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
                mapping,
                draw_rect,
            });
        }
    }
    None
}
