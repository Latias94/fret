use std::collections::BTreeSet;
use std::sync::Arc;

use fret_canvas::view::{PanZoom2D, screen_rect_to_canvas_rect};
use fret_core::{Color, Point, Px, Rect};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, LayoutQueryRegionProps, Length, PositionStyle,
    SemanticsDecoration, SpacingEdges, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, ThemeSnapshot, UiHost};

use crate::core::{Graph, NodeId, PortDirection};
use crate::ui::declarative::view_reducer::apply_fit_view_to_canvas_rect;
use crate::ui::{NodeGraphSurfaceBinding, style::NodeGraphStyle};

use super::record_portal_measured_node_size_in_state;
use super::{
    NodeDragState, NodeRectDraw, PortalBoundsStore, PortalMeasuredGeometryState,
    canvas_viewport_rect, node_drag_delta_canvas, read_authoritative_graph_in_models, rect_union,
    rects_intersect, sync_portal_canvas_bounds_in_models, update_view_state_ui_host,
};

#[derive(Debug, Clone)]
pub(super) struct PortalLabelInfo {
    pub(super) id: NodeId,
    pub(super) left: Px,
    pub(super) top: Px,
    pub(super) width: Px,
    pub(super) height: Px,
    pub(super) label: Arc<str>,
    pub(super) ports_in: u32,
    pub(super) ports_out: u32,
    pub(super) selected: bool,
    pub(super) hovered: bool,
}

fn portal_node_rect_for_surface(
    draw: &NodeRectDraw,
    drag_active: bool,
    dragged_nodes: Option<&[NodeId]>,
    ddx: f32,
    ddy: f32,
) -> Rect {
    let mut rect = draw.rect;
    if drag_active && dragged_nodes.is_some_and(|nodes| nodes.binary_search(&draw.id).is_ok()) {
        rect.origin = Point::new(Px(rect.origin.x.0 + ddx), Px(rect.origin.y.0 + ddy));
    }
    rect
}

pub(super) fn collect_portal_label_infos_for_visible_subset(
    graph: &Graph,
    draws: Option<&[NodeRectDraw]>,
    bounds: Rect,
    view: PanZoom2D,
    cull_canvas: Option<Rect>,
    portal_max_nodes: usize,
    hovered_node: Option<NodeId>,
    selected_nodes: &[NodeId],
    node_drag: Option<&NodeDragState>,
) -> Vec<PortalLabelInfo> {
    let Some(draws) = draws else {
        return Vec::new();
    };
    if portal_max_nodes == 0 {
        return Vec::new();
    }

    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let drag_active = node_drag.is_some_and(NodeDragState::is_active);
    let (ddx, ddy) = node_drag
        .filter(|_| drag_active)
        .map(|drag| node_drag_delta_canvas(view, drag))
        .unwrap_or((0.0, 0.0));
    let dragged_nodes = node_drag
        .filter(|drag| drag.is_active())
        .map(|drag| drag.nodes_sorted.as_ref());

    let mut infos: Vec<PortalLabelInfo> = Vec::new();
    for draw in draws {
        if infos.len() >= portal_max_nodes {
            break;
        }

        let rect = portal_node_rect_for_surface(draw, drag_active, dragged_nodes, ddx, ddy);
        if let Some(cull) = cull_canvas
            && !rects_intersect(cull, rect)
        {
            continue;
        }

        let origin_screen = view.canvas_to_screen(bounds, rect.origin);
        let left = Px(origin_screen.x.0 - bounds.origin.x.0);
        let top = Px(origin_screen.y.0 - bounds.origin.y.0);
        let width = Px((rect.size.width.0 * zoom).max(0.0));

        if !left.0.is_finite() || !top.0.is_finite() || !width.0.is_finite() {
            continue;
        }

        let (label, ports_in, ports_out) = collect_hovered_node_label_and_ports(graph, draw.id)
            .unwrap_or_else(|| (Arc::<str>::from("node"), 0, 0));

        infos.push(PortalLabelInfo {
            id: draw.id,
            left,
            top,
            width,
            height: Px(38.0),
            label,
            ports_in,
            ports_out,
            selected: selected_nodes.contains(&draw.id),
            hovered: hovered_node.is_some_and(|id| id == draw.id),
        });
    }

    infos
}

pub(super) fn host_visible_portal_labels<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    overlay_children: &mut Vec<AnyElement>,
    binding: &NodeGraphSurfaceBinding,
    node_draws: Option<&Arc<Vec<NodeRectDraw>>>,
    bounds: Rect,
    view: PanZoom2D,
    cull_margin_screen_px: f32,
    portal_max_nodes: usize,
    hovered_node: Option<NodeId>,
    selected_nodes: &[NodeId],
    node_drag: Option<&NodeDragState>,
    portal_bounds_store: &Model<PortalBoundsStore>,
    portal_measured_geometry_state: &Model<PortalMeasuredGeometryState>,
    measured_geometry_enabled: bool,
    style_tokens: &NodeGraphStyle,
    theme: &ThemeSnapshot,
) -> bool {
    if portal_max_nodes == 0
        || !bounds.size.width.0.is_finite()
        || !bounds.size.height.0.is_finite()
        || bounds.size.width.0 <= 0.0
        || bounds.size.height.0 <= 0.0
    {
        return false;
    }

    let cull_canvas = canvas_viewport_rect(bounds, view, cull_margin_screen_px);
    let portal_infos: Vec<PortalLabelInfo> =
        read_authoritative_graph_in_models(cx.app.models_mut(), binding, |graph_value| {
            collect_portal_label_infos_for_visible_subset(
                graph_value,
                node_draws.map(|draws| draws.as_slice()),
                bounds,
                view,
                cull_canvas,
                portal_max_nodes,
                hovered_node,
                selected_nodes,
                node_drag,
            )
        })
        .unwrap_or_default();
    if portal_infos.is_empty() {
        return false;
    }

    let hovered_portal_hosted =
        hovered_node.is_some_and(|id| portal_infos.iter().any(|portal| portal.id == id));

    let visible: BTreeSet<NodeId> = portal_infos.iter().map(|portal| portal.id).collect();
    let should_prune = cx
        .app
        .models()
        .read(portal_bounds_store, |state| {
            state
                .nodes_canvas_bounds
                .keys()
                .any(|id| !visible.contains(id))
        })
        .unwrap_or(false);
    if should_prune {
        let _ = cx.app.models_mut().update(portal_bounds_store, |state| {
            state
                .nodes_canvas_bounds
                .retain(|id, _| visible.contains(id));
        });
        cx.request_frame();
    }

    for (ordinal, info) in portal_infos.into_iter().enumerate() {
        let style_tokens = style_tokens.clone();
        let theme = theme.clone();
        let portal_bounds_store = portal_bounds_store.clone();
        let portal_measured_geometry_state = portal_measured_geometry_state.clone();
        overlay_children.push(cx.keyed(("fret-node.portal-label.v1", info.id), move |cx| {
            let mut query = LayoutQueryRegionProps {
                name: Some("fret-node.portal.node_label.v1".into()),
                ..Default::default()
            };
            query.layout.position = PositionStyle::Absolute;
            query.layout.inset.left = Some(info.left).into();
            query.layout.inset.top = Some(info.top).into();
            query.layout.size.width = Length::Px(info.width);
            query.layout.size.height = Length::Px(info.height);

            cx.layout_query_region_with_id(query, move |cx, element| {
                let visual_bounds = cx
                    .last_visual_bounds_for_element(element)
                    .or_else(|| cx.last_bounds_for_element(element));

                if let Some(visual_bounds) = visual_bounds {
                    let canvas_bounds = screen_rect_to_canvas_rect(bounds, view, visual_bounds);

                    let mut request_frame = sync_portal_canvas_bounds_in_models(
                        cx.app.models_mut(),
                        &portal_bounds_store,
                        info.id,
                        canvas_bounds,
                    );
                    if measured_geometry_enabled {
                        request_frame |= record_portal_measured_node_size_in_state(
                            cx.app.models_mut(),
                            &portal_measured_geometry_state,
                            info.id,
                            (visual_bounds.size.width.0, visual_bounds.size.height.0),
                        );
                    }
                    if request_frame {
                        cx.request_frame();
                    }
                }

                let mut props = ContainerProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;
                props.padding = SpacingEdges::all(SpacingLength::Px(Px(4.0)));
                props.snap_to_device_pixels = true;
                props.background = Some(Color {
                    a: if info.selected {
                        0.98
                    } else if info.hovered {
                        0.95
                    } else {
                        0.92
                    },
                    ..style_tokens.paint.node_background
                });
                props.border = fret_core::Edges::all(Px(1.0));
                props.border_color = Some(Color {
                    a: 0.35,
                    ..style_tokens.paint.node_border
                });

                let header_color = theme.color_token("card-foreground");
                let ports_color = theme.color_token("muted-foreground");

                let header = {
                    let mut props = TextProps::new(info.label.clone());
                    props.color = Some(header_color);
                    cx.text_props(props)
                        .attach_semantics(SemanticsDecoration::default().test_id(Arc::<str>::from(
                            format!("node_graph.portal.node.{ordinal}.header"),
                        )))
                };
                let ports = {
                    let mut props = TextProps::new(Arc::<str>::from(format!(
                        "in:{} out:{}",
                        info.ports_in, info.ports_out
                    )));
                    props.color = Some(ports_color);
                    cx.text_props(props)
                        .attach_semantics(SemanticsDecoration::default().test_id(Arc::<str>::from(
                            format!("node_graph.portal.node.{ordinal}.ports"),
                        )))
                };

                vec![
                    cx.container(props, move |cx| {
                        let mut col = ColumnProps::default();
                        col.layout.size.width = Length::Fill;
                        col.layout.size.height = Length::Fill;
                        col.gap = SpacingLength::Px(Px(2.0));
                        vec![cx.column(col, move |_cx| vec![header, ports])]
                    })
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id(Arc::<str>::from(format!(
                                "node_graph.portal.node.{ordinal}"
                            )))
                            .value(Arc::<str>::from(format!(
                                "node_id={}; ports_in={} ports_out={}",
                                info.id.0, info.ports_in, info.ports_out
                            ))),
                    ),
                ]
            })
        }));
    }

    hovered_portal_hosted
}

pub(super) fn apply_pending_fit_to_portals<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    binding: &NodeGraphSurfaceBinding,
    portal_bounds_store: &Model<PortalBoundsStore>,
    portals_enabled: bool,
    portals_disabled: bool,
    bounds: Rect,
    min_zoom: f32,
    max_zoom: f32,
) {
    let pending_fit = cx
        .app
        .models()
        .read(portal_bounds_store, |state| state.pending_fit_to_portals)
        .unwrap_or(false);
    if !pending_fit || !portals_enabled || portals_disabled {
        return;
    }

    let bounds_valid = bounds.size.width.0.is_finite()
        && bounds.size.height.0.is_finite()
        && bounds.size.width.0 > 0.0
        && bounds.size.height.0 > 0.0;
    let target = cx
        .app
        .models()
        .read(portal_bounds_store, |state| {
            let mut out: Option<Rect> = None;
            for rect in state.nodes_canvas_bounds.values().copied() {
                out = Some(match out {
                    Some(prev) => rect_union(prev, rect),
                    None => rect,
                });
            }
            out
        })
        .ok()
        .flatten();

    if bounds_valid && let Some(target) = target {
        let applied = update_view_state_ui_host(cx.app, binding, |state| {
            let _ = apply_fit_view_to_canvas_rect(state, bounds, target, 24.0, min_zoom, max_zoom);
        });

        if applied {
            let _ = cx.app.models_mut().update(portal_bounds_store, |state| {
                state.fit_to_portals_count = state.fit_to_portals_count.saturating_add(1);
                state.pending_fit_to_portals = false;
            });
            cx.request_frame();
        }
    }

    let still_pending = cx
        .app
        .models()
        .read(portal_bounds_store, |state| state.pending_fit_to_portals)
        .unwrap_or(false);
    if still_pending {
        cx.request_frame();
    }
}

pub(super) fn collect_hovered_node_label_and_ports(
    graph: &Graph,
    node_id: NodeId,
) -> Option<(Arc<str>, u32, u32)> {
    graph.nodes.get(&node_id).map(|node| {
        let mut ports_in = 0u32;
        let mut ports_out = 0u32;
        for port_id in node.ports.iter() {
            let Some(port) = graph.ports.get(port_id) else {
                continue;
            };
            match port.dir {
                PortDirection::In => ports_in += 1,
                PortDirection::Out => ports_out += 1,
            }
        }
        (Arc::<str>::from(node.kind.0.as_str()), ports_in, ports_out)
    })
}
