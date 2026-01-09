use std::collections::HashSet;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;

use super::super::state::{MarqueeDrag, MarqueeMode, PendingMarqueeDrag, ViewSnapshot};
use super::NodeGraphCanvas;

fn nodes_in_marquee(
    graph: &crate::core::Graph,
    geom: &super::super::geometry::CanvasGeometry,
    a: Point,
    b: Point,
) -> Vec<GraphNodeId> {
    let rect = super::rect_from_points(a, b);
    geom.nodes
        .iter()
        .filter_map(|(id, ng)| {
            graph
                .nodes
                .contains_key(id)
                .then_some(*id)
                .filter(|_| super::rects_intersect(rect, ng.rect))
        })
        .collect()
}

fn union_nodes(a: &[GraphNodeId], b: &[GraphNodeId]) -> Vec<GraphNodeId> {
    let mut out: Vec<GraphNodeId> = Vec::with_capacity(a.len() + b.len());
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    out
}

fn toggle_nodes(base: &[GraphNodeId], delta: &[GraphNodeId]) -> Vec<GraphNodeId> {
    let mut set: HashSet<GraphNodeId> = base.iter().copied().collect();
    for id in delta.iter().copied() {
        if !set.insert(id) {
            set.remove(&id);
        }
    }
    set.into_iter().collect()
}

fn mode_from_modifiers(modifiers: Modifiers) -> MarqueeMode {
    if modifiers.shift {
        MarqueeMode::Add
    } else if modifiers.ctrl || modifiers.meta {
        MarqueeMode::Toggle
    } else {
        MarqueeMode::Replace
    }
}

pub(super) fn begin_background_marquee<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    pos: Point,
    modifiers: Modifiers,
) {
    let mode = mode_from_modifiers(modifiers);
    canvas.interaction.pending_marquee = Some(PendingMarqueeDrag {
        start_pos: pos,
        base_nodes: snapshot.selected_nodes.clone(),
        mode,
    });
    cx.capture_pointer(cx.node);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}

pub(super) fn handle_marquee_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    if let Some(mut marquee) = canvas.interaction.marquee.take() {
        marquee.pos = position;
        let (geom, _index) = canvas.canvas_derived(&*cx.app, snapshot);
        let selection = canvas
            .graph
            .read_ref(cx.app, |graph| {
                nodes_in_marquee(graph, geom.as_ref(), marquee.start_pos, marquee.pos)
            })
            .ok()
            .unwrap_or_default();

        let mut selected: Vec<GraphNodeId> = match marquee.mode {
            MarqueeMode::Replace => selection,
            MarqueeMode::Add => union_nodes(&marquee.base_nodes, &selection),
            MarqueeMode::Toggle => toggle_nodes(&marquee.base_nodes, &selection),
        };
        selected.sort();
        selected.dedup();

        canvas.interaction.marquee = Some(marquee);
        canvas.interaction.focused_edge = None;
        canvas.update_view_state(cx.app, |s| {
            s.selected_edges.clear();
            s.selected_nodes = selected;
        });

        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.node_drag.is_none() {
        if let Some(pending) = canvas.interaction.pending_marquee.clone() {
            let threshold_screen = snapshot.interaction.node_drag_threshold.max(0.0);
            let threshold_graph = threshold_screen / zoom;
            let dx = position.x.0 - pending.start_pos.x.0;
            let dy = position.y.0 - pending.start_pos.y.0;
            if threshold_graph <= 0.0 || dx * dx + dy * dy >= threshold_graph * threshold_graph {
                canvas.interaction.pending_marquee = None;
                let marquee = MarqueeDrag {
                    start_pos: pending.start_pos,
                    pos: position,
                    base_nodes: pending.base_nodes.clone(),
                    mode: pending.mode,
                };
                canvas.interaction.marquee = Some(marquee.clone());

                let (geom, _index) = canvas.canvas_derived(&*cx.app, snapshot);
                let selection = canvas
                    .graph
                    .read_ref(cx.app, |graph| {
                        nodes_in_marquee(graph, geom.as_ref(), marquee.start_pos, marquee.pos)
                    })
                    .ok()
                    .unwrap_or_default();

                let mut selected: Vec<GraphNodeId> = match marquee.mode {
                    MarqueeMode::Replace => selection,
                    MarqueeMode::Add => union_nodes(&marquee.base_nodes, &selection),
                    MarqueeMode::Toggle => toggle_nodes(&marquee.base_nodes, &selection),
                };
                selected.sort();
                selected.dedup();

                canvas.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes = selected;
                });

                cx.request_redraw();
                cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
                return true;
            }
        }
    }

    let _ = modifiers;
    false
}

pub(super) fn handle_left_up<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.marquee.take().is_some() {
        canvas.interaction.pending_marquee = None;
        canvas.interaction.snap_guides = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(pending) = canvas.interaction.pending_marquee.take() {
        if matches!(pending.mode, MarqueeMode::Replace) {
            canvas.update_view_state(cx.app, |s| {
                s.selected_nodes.clear();
                s.selected_edges.clear();
                s.selected_groups.clear();
            });
        }
        canvas.interaction.snap_guides = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    false
}
