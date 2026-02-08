use std::collections::BTreeSet;

use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{MarqueeDrag, PendingMarqueeDrag, ViewSnapshot};

fn nodes_in_marquee(
    graph: &crate::core::Graph,
    geom: &crate::ui::canvas::geometry::CanvasGeometry,
    a: Point,
    b: Point,
    mode: crate::io::NodeGraphSelectionMode,
) -> Vec<GraphNodeId> {
    let rect = super::rect_from_points(a, b);
    geom.nodes
        .iter()
        .filter_map(|(id, ng)| {
            let node = graph.nodes.get(id)?;
            if !node.selectable.unwrap_or(true) {
                return None;
            }
            match mode {
                crate::io::NodeGraphSelectionMode::Full => {
                    let fully_contained = ng.rect.origin.x.0 >= rect.origin.x.0
                        && ng.rect.origin.y.0 >= rect.origin.y.0
                        && (ng.rect.origin.x.0 + ng.rect.size.width.0)
                            <= (rect.origin.x.0 + rect.size.width.0)
                        && (ng.rect.origin.y.0 + ng.rect.size.height.0)
                            <= (rect.origin.y.0 + rect.size.height.0);
                    fully_contained.then_some(*id)
                }
                crate::io::NodeGraphSelectionMode::Partial => {
                    super::rects_intersect(rect, ng.rect).then_some(*id)
                }
            }
        })
        .collect()
}

pub(super) fn begin_background_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    pos: Point,
    modifiers: Modifiers,
    clear_selection_on_up: bool,
) {
    let _ = snapshot;
    let _ = modifiers;
    canvas.interaction.pending_marquee = Some(PendingMarqueeDrag {
        start_pos: pos,
        clear_selection_on_up,
    });
    cx.capture_pointer(cx.node);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}

pub(super) fn handle_marquee_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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
                nodes_in_marquee(
                    graph,
                    geom.as_ref(),
                    marquee.start_pos,
                    marquee.pos,
                    snapshot.interaction.selection_mode,
                )
            })
            .ok()
            .unwrap_or_default();

        let mut selected: Vec<GraphNodeId> = selection;
        selected.sort();
        selected.dedup();

        let selected_edges =
            if snapshot.interaction.elements_selectable && snapshot.interaction.edges_selectable {
                let nodes: BTreeSet<GraphNodeId> = selected.iter().copied().collect();
                canvas.box_select_edges_for_nodes(cx.app, &snapshot.interaction, &nodes)
            } else {
                Vec::new()
            };

        canvas.interaction.marquee = Some(marquee);
        canvas.interaction.focused_edge = None;
        canvas.update_view_state(cx.app, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes = selected;
            s.selected_edges = selected_edges;
        });

        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.node_drag.is_none() {
        if let Some(pending) = canvas.interaction.pending_marquee.clone() {
            let selection_key_pressed = snapshot.interaction.selection_key.is_pressed(modifiers);
            let threshold_screen = if selection_key_pressed {
                0.0
            } else {
                snapshot.interaction.pane_click_distance.max(0.0)
            };
            let threshold_graph = canvas_units_from_screen_px(threshold_screen, zoom);
            let dx = position.x.0 - pending.start_pos.x.0;
            let dy = position.y.0 - pending.start_pos.y.0;
            if threshold_graph <= 0.0 || dx * dx + dy * dy >= threshold_graph * threshold_graph {
                let selection_box_active =
                    snapshot.interaction.selection_on_drag || selection_key_pressed;

                if selection_box_active {
                    canvas.interaction.pending_marquee = None;
                    let marquee = MarqueeDrag {
                        start_pos: pending.start_pos,
                        pos: position,
                    };
                    canvas.interaction.marquee = Some(marquee.clone());

                    let (geom, _index) = canvas.canvas_derived(&*cx.app, snapshot);
                    let selection = canvas
                        .graph
                        .read_ref(cx.app, |graph| {
                            nodes_in_marquee(
                                graph,
                                geom.as_ref(),
                                marquee.start_pos,
                                marquee.pos,
                                snapshot.interaction.selection_mode,
                            )
                        })
                        .ok()
                        .unwrap_or_default();

                    let mut selected: Vec<GraphNodeId> = selection;
                    selected.sort();
                    selected.dedup();

                    let selected_edges = if snapshot.interaction.elements_selectable
                        && snapshot.interaction.edges_selectable
                    {
                        let nodes: BTreeSet<GraphNodeId> = selected.iter().copied().collect();
                        canvas.box_select_edges_for_nodes(cx.app, &snapshot.interaction, &nodes)
                    } else {
                        Vec::new()
                    };

                    canvas.update_view_state(cx.app, |s| {
                        s.selected_edges.clear();
                        s.selected_groups.clear();
                        s.selected_nodes = selected;
                        s.selected_edges = selected_edges;
                    });

                    cx.request_redraw();
                    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
                    return true;
                }

                if snapshot.interaction.pan_on_drag.left {
                    canvas.interaction.pending_marquee = None;
                    let _ = super::pan_zoom::begin_panning(
                        canvas,
                        cx,
                        snapshot,
                        pending.start_pos,
                        MouseButton::Left,
                    );
                    return true;
                }
            }
        }
    }

    let _ = modifiers;
    false
}

pub(super) fn handle_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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
        if pending.clear_selection_on_up {
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
