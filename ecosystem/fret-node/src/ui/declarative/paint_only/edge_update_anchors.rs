use std::collections::BTreeSet;
use std::sync::Arc;

use fret_canvas::view::PanZoom2D;
use fret_core::{Corners, Edges, MouseButton, Point, PointerId, Px, Rect, SemanticsRole, Size};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::action::{
    ActionCx, PressablePointerDownResult, PressablePointerUpResult, UiActionHost,
};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, Length, PositionStyle, PressableProps,
    SemanticsDecoration,
};
use fret_ui::{ElementContext, UiHost};

use crate::core::{EdgeId, EdgeReconnectable, EdgeReconnectableEndpoint, Graph, PortId};
use crate::io::{NodeGraphInteractionState, NodeGraphViewState};
use crate::ops::GraphTransaction;
use crate::rules::{ConnectDecision, ConnectPlan, EdgeEndpoint, plan_reconnect_edge_with_mode};
use crate::runtime::callbacks::{ConnectDragKind, ConnectEnd, ConnectEndOutcome, ConnectStart};
use crate::runtime::events::NodeGraphGestureEvent;
use crate::ui::NodeGraphSurfaceBinding;
use crate::ui::canvas::CanvasGeometry;
use crate::ui::internals::NodeGraphInternalsSnapshot;
use crate::ui::style::NodeGraphStyle;

use super::commit_graph_transaction;
use super::surface_math::pointer_crossed_threshold;
use super::surface_support::read_authoritative_interaction_config_in_models;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct EdgeUpdateAnchorInfo {
    pub(super) edge: EdgeId,
    pub(super) endpoint: EdgeEndpoint,
    pub(super) anchor_port: PortId,
    pub(super) opposite_port: PortId,
    pub(super) center_window: Point,
    pub(super) radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReconnectDragPhase {
    Armed,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ReconnectDragState {
    pub(super) pointer_id: PointerId,
    pub(super) start_screen: Point,
    pub(super) current_screen: Point,
    pub(super) phase: ReconnectDragPhase,
    pub(super) edge: EdgeId,
    pub(super) endpoint: EdgeEndpoint,
    pub(super) anchor_port: PortId,
    pub(super) fixed_port: PortId,
}

#[derive(Debug, Clone)]
pub(super) struct ReconnectDropContext {
    pub(super) geom: Option<Arc<CanvasGeometry>>,
    pub(super) view: PanZoom2D,
    pub(super) bounds: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReconnectDropResult {
    target: Option<PortId>,
    outcome: ConnectEndOutcome,
}

impl ReconnectDragState {
    pub(super) fn is_armed(&self) -> bool {
        matches!(self.phase, ReconnectDragPhase::Armed)
    }

    pub(super) fn is_active(&self) -> bool {
        matches!(self.phase, ReconnectDragPhase::Active)
    }

    fn activate(&mut self, current_screen: Point) -> bool {
        if !self.is_armed() {
            return false;
        }
        self.phase = ReconnectDragPhase::Active;
        self.current_screen = current_screen;
        true
    }

    fn update_active_position(&mut self, current_screen: Point) -> bool {
        if !self.is_active() || self.current_screen == current_screen {
            return false;
        }
        self.current_screen = current_screen;
        true
    }
}

pub(super) fn collect_edge_update_anchor_infos(
    graph: &Graph,
    view_state: &NodeGraphViewState,
    internals: &NodeGraphInternalsSnapshot,
    interaction: &NodeGraphInteractionState,
) -> Vec<EdgeUpdateAnchorInfo> {
    let radius = normalized_reconnect_radius(interaction.reconnect_radius);
    if radius <= 0.0 {
        return Vec::new();
    }

    let mut candidates = BTreeSet::<EdgeId>::new();
    candidates.extend(view_state.selected_edges.iter().copied());
    if let Some(focused_edge) = internals.focused_edge {
        candidates.insert(focused_edge);
    }

    let mut out = Vec::new();
    for edge_id in candidates {
        let Some(edge) = graph.edges.get(&edge_id) else {
            continue;
        };

        for endpoint in [EdgeEndpoint::From, EdgeEndpoint::To] {
            if !edge_reconnect_endpoint_enabled(
                edge.reconnectable,
                interaction.edges_reconnectable,
                endpoint,
            ) {
                continue;
            }

            let (anchor_port, opposite_port) = match endpoint {
                EdgeEndpoint::From => (edge.from, edge.to),
                EdgeEndpoint::To => (edge.to, edge.from),
            };
            let Some(center_window) = internals.port_centers_window.get(&anchor_port).copied()
            else {
                continue;
            };

            out.push(EdgeUpdateAnchorInfo {
                edge: edge_id,
                endpoint,
                anchor_port,
                opposite_port,
                center_window,
                radius,
            });
        }
    }

    out
}

pub(super) fn edge_reconnect_endpoint_enabled(
    edge_reconnectable: Option<EdgeReconnectable>,
    global_edges_reconnectable: bool,
    endpoint: EdgeEndpoint,
) -> bool {
    match edge_reconnectable {
        None => global_edges_reconnectable,
        Some(EdgeReconnectable::Bool(enabled)) => enabled,
        Some(EdgeReconnectable::Endpoint(EdgeReconnectableEndpoint::Source)) => {
            endpoint == EdgeEndpoint::From
        }
        Some(EdgeReconnectable::Endpoint(EdgeReconnectableEndpoint::Target)) => {
            endpoint == EdgeEndpoint::To
        }
    }
}

pub(super) fn hit_test_edge_update_anchor_at_window_point(
    anchors: &[EdgeUpdateAnchorInfo],
    point: Point,
) -> Option<EdgeUpdateAnchorInfo> {
    anchors
        .iter()
        .rev()
        .find(|anchor| edge_update_anchor_rect(anchor).contains(point))
        .copied()
}

pub(super) fn push_edge_update_anchor_controls<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    interactive_overlay_children: &mut Vec<AnyElement>,
    anchors: &[EdgeUpdateAnchorInfo],
    reconnect_drag: &Model<Option<ReconnectDragState>>,
    drop_context: ReconnectDropContext,
    binding: &NodeGraphSurfaceBinding,
    bounds: Rect,
    style_tokens: &NodeGraphStyle,
) {
    if anchors.is_empty()
        || !bounds.size.width.0.is_finite()
        || !bounds.size.height.0.is_finite()
        || bounds.size.width.0 <= 0.0
        || bounds.size.height.0 <= 0.0
    {
        return;
    }

    for anchor in anchors.iter().copied() {
        let rect = edge_update_anchor_rect(&anchor);
        if !rect_intersects(bounds, rect) {
            continue;
        }

        let style_tokens = style_tokens.clone();
        let reconnect_drag = reconnect_drag.clone();
        let drop_context = drop_context.clone();
        let binding = binding.clone();
        interactive_overlay_children.push(cx.keyed(
            (
                "fret-node.edge-update-anchor.v1",
                anchor.edge,
                anchor.endpoint,
            ),
            move |cx| {
                edge_update_anchor_control(
                    cx,
                    bounds,
                    anchor,
                    rect,
                    reconnect_drag.clone(),
                    drop_context.clone(),
                    binding.clone(),
                    style_tokens,
                )
            },
        ));
    }
}

fn edge_update_anchor_control<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    bounds: Rect,
    anchor: EdgeUpdateAnchorInfo,
    rect: Rect,
    reconnect_drag: Model<Option<ReconnectDragState>>,
    drop_context: ReconnectDropContext,
    binding: NodeGraphSurfaceBinding,
    style_tokens: NodeGraphStyle,
) -> AnyElement {
    let endpoint = edge_endpoint_name(anchor.endpoint);
    let mut pressable = PressableProps::default();
    pressable.layout.position = PositionStyle::Absolute;
    pressable.layout.inset.left = InsetEdge::Px(Px(rect.origin.x.0 - bounds.origin.x.0));
    pressable.layout.inset.top = InsetEdge::Px(Px(rect.origin.y.0 - bounds.origin.y.0));
    pressable.layout.size.width = Length::Px(rect.size.width);
    pressable.layout.size.height = Length::Px(rect.size.height);

    let test_id = Arc::<str>::from(format!(
        "node_graph.edge_update_anchor.{}.{}",
        anchor.edge.0, endpoint
    ));
    let label = Arc::<str>::from(format!("Reconnect {endpoint} edge endpoint"));
    let value = Arc::<str>::from(format!(
        "edge_id={};endpoint={};anchor_port={};opposite_port={};radius={:.2}",
        anchor.edge.0, endpoint, anchor.anchor_port.0, anchor.opposite_port.0, anchor.radius,
    ));

    cx.pressable(pressable, move |cx, _state| {
        let reconnect_drag_for_down = reconnect_drag.clone();
        let binding_for_down = binding.clone();
        cx.pressable_on_pointer_down(Arc::new(move |host, action_cx, down| {
            begin_reconnect_drag_pointer_down_action_host(
                host,
                action_cx,
                &reconnect_drag_for_down,
                &binding_for_down,
                anchor,
                down,
            )
        }));

        let reconnect_drag_for_move = reconnect_drag.clone();
        let binding_for_move = binding.clone();
        cx.pressable_on_pointer_move(Arc::new(move |host, action_cx, mv| {
            handle_reconnect_drag_pointer_move_action_host(
                host,
                action_cx,
                &reconnect_drag_for_move,
                &binding_for_move,
                mv,
            )
        }));

        let reconnect_drag_for_up = reconnect_drag.clone();
        let drop_context_for_up = drop_context.clone();
        let binding_for_up = binding.clone();
        cx.pressable_on_pointer_up(Arc::new(move |host, action_cx, up| {
            if finish_reconnect_drag_pointer_up_action_host(
                host,
                action_cx,
                &reconnect_drag_for_up,
                &binding_for_up,
                &drop_context_for_up,
                up,
            ) {
                PressablePointerUpResult::SkipActivate
            } else {
                PressablePointerUpResult::Continue
            }
        }));
        vec![edge_update_anchor_visual(cx, anchor, style_tokens)]
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Button)
            .label(label)
            .test_id(test_id)
            .value(value),
    )
}

pub(super) fn begin_reconnect_drag_pointer_down_action_host(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    action_cx: ActionCx,
    reconnect_drag: &Model<Option<ReconnectDragState>>,
    binding: &NodeGraphSurfaceBinding,
    anchor: EdgeUpdateAnchorInfo,
    down: fret_ui::action::PointerDownCx,
) -> PressablePointerDownResult {
    if down.button != MouseButton::Left {
        return PressablePointerDownResult::Continue;
    }

    let next = ReconnectDragState {
        pointer_id: down.pointer_id,
        start_screen: down.position,
        current_screen: down.position,
        phase: ReconnectDragPhase::Armed,
        edge: anchor.edge,
        endpoint: anchor.endpoint,
        anchor_port: anchor.anchor_port,
        fixed_port: anchor.opposite_port,
    };
    let armed = host
        .models_mut()
        .update(reconnect_drag, |state| {
            *state = Some(next);
            true
        })
        .ok()
        .unwrap_or(false);
    if !armed {
        return PressablePointerDownResult::Continue;
    }
    emit_reconnect_start_action_host(host, binding, next);
    host.request_focus(action_cx.target);
    host.capture_pointer();
    host.invalidate(Invalidation::Layout);
    host.notify(action_cx);
    host.request_redraw(action_cx.window);
    PressablePointerDownResult::SkipDefaultAndStopPropagation
}

fn emit_reconnect_start_action_host(
    host: &mut dyn UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    drag: ReconnectDragState,
) {
    let store = binding.store_model();
    let _ = host.models_mut().update(&store, |store| {
        let ev = ConnectStart {
            kind: reconnect_drag_kind(drag),
            mode: store.resolved_interaction_state().connection_mode,
        };
        store.emit_gesture(NodeGraphGestureEvent::ConnectStart(ev));
    });
}

fn emit_reconnect_end_action_host(
    host: &mut dyn UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    drag: ReconnectDragState,
    target: Option<PortId>,
    outcome: ConnectEndOutcome,
) {
    let store = binding.store_model();
    let _ = host.models_mut().update(&store, |store| {
        let ev = ConnectEnd {
            kind: reconnect_drag_kind(drag),
            mode: store.resolved_interaction_state().connection_mode,
            target,
            outcome,
        };
        store.emit_gesture(NodeGraphGestureEvent::ConnectEnd(ev));
    });
}

fn reconnect_drag_kind(drag: ReconnectDragState) -> ConnectDragKind {
    ConnectDragKind::Reconnect {
        edge: drag.edge,
        endpoint: drag.endpoint,
        fixed: drag.fixed_port,
    }
}

pub(super) fn cancel_reconnect_drag_action_host(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    reconnect_drag: &Model<Option<ReconnectDragState>>,
    binding: &NodeGraphSurfaceBinding,
) -> bool {
    let current = host
        .models_mut()
        .read(reconnect_drag, |state| *state)
        .ok()
        .flatten();
    let Some(current) = current else {
        return false;
    };

    emit_reconnect_end_action_host(host, binding, current, None, ConnectEndOutcome::Canceled);
    let cleared = clear_reconnect_drag_action_host(host, reconnect_drag);
    if cleared {
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    }
    cleared
}

pub(super) fn handle_reconnect_drag_pointer_move_action_host(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    action_cx: ActionCx,
    reconnect_drag: &Model<Option<ReconnectDragState>>,
    binding: &NodeGraphSurfaceBinding,
    mv: fret_ui::action::PointerMoveCx,
) -> bool {
    let Some(current) = host
        .models_mut()
        .read(reconnect_drag, |state| *state)
        .ok()
        .flatten()
    else {
        return false;
    };

    if current.pointer_id != mv.pointer_id {
        return true;
    }

    if !mv.buttons.left {
        let cleared = cancel_reconnect_drag_action_host(host, action_cx, reconnect_drag, binding);
        if cleared {
            host.release_pointer_capture();
            host.invalidate(Invalidation::Layout);
        }
        return true;
    }

    let threshold =
        read_authoritative_interaction_config_in_models(host.models_mut(), binding, |config| {
            config.connection_drag_threshold
        })
        .unwrap_or(1.0);
    let should_activate = current.is_active()
        || pointer_crossed_threshold(current.start_screen, mv.position, threshold);
    if !should_activate {
        return true;
    }

    let mut changed = false;
    let _ = host.models_mut().update(reconnect_drag, |state| {
        if let Some(state) = state.as_mut() {
            if state.activate(mv.position) {
                changed = true;
            }
            if state.update_active_position(mv.position) {
                changed = true;
            }
        }
    });
    if changed {
        host.invalidate(Invalidation::Layout);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    }
    true
}

pub(super) fn finish_reconnect_drag_pointer_up_action_host(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    action_cx: ActionCx,
    reconnect_drag: &Model<Option<ReconnectDragState>>,
    binding: &NodeGraphSurfaceBinding,
    drop_context: &ReconnectDropContext,
    up: fret_ui::action::PointerUpCx,
) -> bool {
    if up.button != MouseButton::Left {
        return false;
    }

    let Some(current) = host
        .models_mut()
        .read(reconnect_drag, |state| *state)
        .ok()
        .flatten()
    else {
        return false;
    };
    if current.pointer_id != up.pointer_id {
        return true;
    }

    let drop_result = if current.is_active() {
        commit_reconnect_drop_action_host(host, binding, drop_context, current, up.position)
    } else {
        ReconnectDropResult {
            target: None,
            outcome: ConnectEndOutcome::NoOp,
        }
    };
    emit_reconnect_end_action_host(
        host,
        binding,
        current,
        drop_result.target,
        drop_result.outcome,
    );

    let cleared = clear_reconnect_drag_action_host(host, reconnect_drag);
    if cleared {
        host.release_pointer_capture();
        host.invalidate(Invalidation::Layout);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    }
    true
}

pub(super) fn cancel_reconnect_drag_pointer_action_host(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    action_cx: ActionCx,
    reconnect_drag: &Model<Option<ReconnectDragState>>,
    binding: &NodeGraphSurfaceBinding,
) -> bool {
    let cleared = cancel_reconnect_drag_action_host(host, action_cx, reconnect_drag, binding);
    if cleared {
        host.release_pointer_capture();
        host.invalidate(Invalidation::Layout);
    }
    cleared
}

pub(super) fn clear_reconnect_drag_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    reconnect_drag: &Model<Option<ReconnectDragState>>,
) -> bool {
    host.models_mut()
        .update(reconnect_drag, |state| {
            let was_active = state.is_some();
            *state = None;
            was_active
        })
        .ok()
        .unwrap_or(false)
}

fn commit_reconnect_drop_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    drop_context: &ReconnectDropContext,
    drag: ReconnectDragState,
    drop_screen: Point,
) -> ReconnectDropResult {
    let Some(geom) = drop_context.geom.as_deref() else {
        return ReconnectDropResult {
            target: None,
            outcome: ConnectEndOutcome::NoOp,
        };
    };
    let decision = read_reconnect_drop_decision_action_host(
        host,
        binding,
        geom,
        drop_context,
        drag,
        drop_screen,
    );
    let ReconnectDropDecision::Accepted { target, plan } = decision else {
        return decision.into_result();
    };

    let tx = GraphTransaction {
        label: None,
        ops: plan.ops,
    }
    .with_label("Reconnect Edge");
    ReconnectDropResult {
        target: Some(target),
        outcome: if commit_graph_transaction(host, binding, &tx) {
            ConnectEndOutcome::Committed
        } else {
            ConnectEndOutcome::Rejected
        },
    }
}

enum ReconnectDropDecision {
    Empty,
    OpenInsertNodePicker,
    Rejected { target: PortId },
    NoOp { target: Option<PortId> },
    Accepted { target: PortId, plan: ConnectPlan },
}

impl ReconnectDropDecision {
    fn into_result(self) -> ReconnectDropResult {
        match self {
            Self::Empty => ReconnectDropResult {
                target: None,
                outcome: ConnectEndOutcome::NoOp,
            },
            Self::OpenInsertNodePicker => ReconnectDropResult {
                target: None,
                outcome: ConnectEndOutcome::OpenInsertNodePicker,
            },
            Self::Rejected { target } => ReconnectDropResult {
                target: Some(target),
                outcome: ConnectEndOutcome::Rejected,
            },
            Self::NoOp { target } => ReconnectDropResult {
                target,
                outcome: ConnectEndOutcome::NoOp,
            },
            Self::Accepted { target, .. } => ReconnectDropResult {
                target: Some(target),
                outcome: ConnectEndOutcome::Committed,
            },
        }
    }
}

fn read_reconnect_drop_decision_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    geom: &CanvasGeometry,
    drop_context: &ReconnectDropContext,
    drag: ReconnectDragState,
    drop_screen: Point,
) -> ReconnectDropDecision {
    let store = binding.store_model();
    let Some((graph, interaction)) = host
        .models_mut()
        .read(&store, |store| {
            (store.graph().clone(), store.resolved_interaction_state())
        })
        .ok()
    else {
        return ReconnectDropDecision::Empty;
    };
    hit_test_reconnect_drop_decision(
        &graph,
        &interaction,
        geom,
        drop_context.view,
        drop_context.bounds,
        drag,
        drop_screen,
    )
}

fn hit_test_reconnect_drop_decision(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    geom: &CanvasGeometry,
    view: PanZoom2D,
    bounds: Rect,
    drag: ReconnectDragState,
    drop_screen: Point,
) -> ReconnectDropDecision {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let radius_canvas = interaction.connection_radius.max(0.0) / zoom;
    if radius_canvas <= 0.0 {
        return empty_reconnect_drop_decision(interaction);
    }
    let radius2 = radius_canvas * radius_canvas;
    let drop_canvas = view.screen_to_canvas(bounds, drop_screen);

    let mut best: Option<(f32, u32, PortId)> = None;
    for (port, handle) in &geom.ports {
        let dx = handle.center.x.0 - drop_canvas.x.0;
        let dy = handle.center.y.0 - drop_canvas.y.0;
        let dist2 = dx * dx + dy * dy;
        if !dist2.is_finite() || dist2 > radius2 {
            continue;
        }
        let rank = geom.node_rank.get(&handle.node).copied().unwrap_or(0);
        let replace = match &best {
            None => true,
            Some((best_dist2, best_rank, best_port)) => {
                dist2 < *best_dist2
                    || ((dist2 - *best_dist2).abs() <= f32::EPSILON
                        && (rank > *best_rank || (rank == *best_rank && *port > *best_port)))
            }
        };
        if replace {
            best = Some((dist2, rank, *port));
        }
    }

    let Some((_, _, target)) = best else {
        return empty_reconnect_drop_decision(interaction);
    };
    if !port_allows_reconnect_endpoint(graph, interaction, target, drag.endpoint) {
        return ReconnectDropDecision::Rejected { target };
    }
    let plan = plan_reconnect_edge_with_mode(
        graph,
        drag.edge,
        drag.endpoint,
        target,
        interaction.connection_mode,
    );
    if plan.decision != ConnectDecision::Accept {
        return ReconnectDropDecision::Rejected { target };
    }
    if target == drag.anchor_port || plan.ops.is_empty() {
        return ReconnectDropDecision::NoOp {
            target: Some(target),
        };
    }
    ReconnectDropDecision::Accepted { target, plan }
}

fn empty_reconnect_drop_decision(interaction: &NodeGraphInteractionState) -> ReconnectDropDecision {
    if interaction.reconnect_on_drop_empty {
        ReconnectDropDecision::OpenInsertNodePicker
    } else {
        ReconnectDropDecision::Empty
    }
}

fn port_allows_reconnect_endpoint(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    port: PortId,
    endpoint: EdgeEndpoint,
) -> bool {
    let Some(port_value) = graph.ports.get(&port) else {
        return false;
    };
    let Some(node) = graph.nodes.get(&port_value.node) else {
        return false;
    };
    let connectable = port_value
        .connectable
        .or(node.connectable)
        .unwrap_or(interaction.nodes_connectable);
    let endpoint_connectable = match endpoint {
        EdgeEndpoint::From => port_value.connectable_start,
        EdgeEndpoint::To => port_value.connectable_end,
    }
    .unwrap_or(true);
    connectable && endpoint_connectable
}

fn edge_update_anchor_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    anchor: EdgeUpdateAnchorInfo,
    style_tokens: NodeGraphStyle,
) -> AnyElement {
    let mut fill = style_tokens.paint.node_border_selected;
    fill.a = fill.a.min(0.28);

    let mut container = ContainerProps::default();
    container.layout.size.width = Length::Fill;
    container.layout.size.height = Length::Fill;
    container.snap_to_device_pixels = true;
    container.background = Some(fill);
    container.corner_radii = Corners::all(Px(anchor.radius.max(0.0)));
    container.border = Edges::all(Px(1.0));
    container.border_color = Some(style_tokens.paint.node_border_selected);

    cx.hit_test_gate(false, move |cx| {
        vec![cx.container(container, |_cx| Vec::new())]
    })
}

fn edge_update_anchor_rect(anchor: &EdgeUpdateAnchorInfo) -> Rect {
    let radius = anchor.radius.max(0.0);
    Rect::new(
        Point::new(
            Px(anchor.center_window.x.0 - radius),
            Px(anchor.center_window.y.0 - radius),
        ),
        Size::new(Px(radius * 2.0), Px(radius * 2.0)),
    )
}

fn edge_endpoint_name(endpoint: EdgeEndpoint) -> &'static str {
    match endpoint {
        EdgeEndpoint::From => "source",
        EdgeEndpoint::To => "target",
    }
}

fn rect_intersects(a: Rect, b: Rect) -> bool {
    let ax1 = a.origin.x.0;
    let ay1 = a.origin.y.0;
    let ax2 = ax1 + a.size.width.0;
    let ay2 = ay1 + a.size.height.0;
    let bx1 = b.origin.x.0;
    let by1 = b.origin.y.0;
    let bx2 = bx1 + b.size.width.0;
    let by2 = by1 + b.size.height.0;

    ax1 <= bx2 && ax2 >= bx1 && ay1 <= by2 && ay2 >= by1
}

fn normalized_reconnect_radius(radius: f32) -> f32 {
    if radius.is_finite() && radius > 0.0 {
        radius
    } else {
        0.0
    }
}
