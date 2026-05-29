use std::collections::BTreeSet;
use std::sync::Arc;

use fret_core::{Corners, Edges, MouseButton, Point, Px, Rect, SemanticsRole, Size};
use fret_ui::action::PressablePointerDownResult;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, Length, PositionStyle, PressableProps,
    SemanticsDecoration,
};
use fret_ui::{ElementContext, UiHost};

use crate::core::{EdgeId, EdgeReconnectable, EdgeReconnectableEndpoint, Graph, PortId};
use crate::io::{NodeGraphInteractionState, NodeGraphViewState};
use crate::rules::EdgeEndpoint;
use crate::ui::internals::NodeGraphInternalsSnapshot;
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct EdgeUpdateAnchorInfo {
    pub(super) edge: EdgeId,
    pub(super) endpoint: EdgeEndpoint,
    pub(super) anchor_port: PortId,
    pub(super) opposite_port: PortId,
    pub(super) center_window: Point,
    pub(super) radius: f32,
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
        interactive_overlay_children.push(cx.keyed(
            (
                "fret-node.edge-update-anchor.v1",
                anchor.edge,
                anchor.endpoint,
            ),
            move |cx| edge_update_anchor_control(cx, bounds, anchor, rect, style_tokens),
        ));
    }
}

fn edge_update_anchor_control<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    bounds: Rect,
    anchor: EdgeUpdateAnchorInfo,
    rect: Rect,
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
        cx.pressable_on_pointer_down(Arc::new(move |_host, _cx, down| {
            if down.button == MouseButton::Left {
                return PressablePointerDownResult::SkipDefaultAndStopPropagation;
            }
            PressablePointerDownResult::Continue
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
