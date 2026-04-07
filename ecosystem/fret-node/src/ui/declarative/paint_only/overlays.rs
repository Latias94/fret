use std::sync::Arc;

use fret_canvas::view::PanZoom2D;
use fret_core::Rect;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, PositionStyle};
use fret_ui::{ElementContext, UiHost};

use crate::core::NodeId;
use crate::ui::style::NodeGraphStyle;

use super::HoverAnchorStore;
use super::MarqueeDragState;
use super::PortalBoundsStore;
use super::hover_anchor::resolve_hover_tooltip_anchor;
use super::marquee_rect_screen;
use super::overlay_elements::{
    build_hover_tooltip_overlay_spec, push_hover_tooltip_overlay, push_marquee_overlay,
};
use super::surface_support::{collect_node_label_and_ports, read_authoritative_graph_in_models};

pub(super) struct HoverTooltipOverlayParams<'a> {
    pub(super) binding: &'a crate::ui::NodeGraphSurfaceBinding,
    pub(super) portal_bounds_store: &'a Model<PortalBoundsStore>,
    pub(super) hover_anchor_store: &'a Model<HoverAnchorStore>,
    pub(super) style_tokens: &'a NodeGraphStyle,
    pub(super) diagnostics: super::NodeGraphDiagnosticsConfig,
    pub(super) panning: bool,
    pub(super) marquee_active: bool,
    pub(super) node_dragging: bool,
    pub(super) hovered_node: Option<NodeId>,
    pub(super) hovered_portal_hosted: bool,
    pub(super) portals_disabled: bool,
    pub(super) bounds: Rect,
    pub(super) view: PanZoom2D,
}

pub(super) fn push_hover_tooltip_overlay_if_needed<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    overlay_children: &mut Vec<AnyElement>,
    params: HoverTooltipOverlayParams<'_>,
) {
    if !params.diagnostics.hover_tooltip_enabled
        || params.panning
        || params.marquee_active
        || params.node_dragging
    {
        return;
    }

    let Some(hovered_id) = params.hovered_node else {
        return;
    };
    if !params.bounds.size.width.0.is_finite()
        || !params.bounds.size.height.0.is_finite()
        || params.bounds.size.width.0 <= 0.0
        || params.bounds.size.height.0 <= 0.0
    {
        return;
    }

    let portal_canvas_bounds = if params.portals_disabled {
        None
    } else {
        cx.app
            .models()
            .read(params.portal_bounds_store, |state| {
                state.nodes_canvas_bounds.get(&hovered_id).copied()
            })
            .ok()
            .flatten()
    };

    let anchor_canvas_bounds = cx
        .app
        .models()
        .read(params.hover_anchor_store, |state| {
            if state.hovered_id == Some(hovered_id) {
                state.hovered_canvas_bounds
            } else {
                None
            }
        })
        .ok()
        .flatten();

    let Some(tooltip_anchor) = resolve_hover_tooltip_anchor(
        params.bounds,
        params.view,
        params.portals_disabled,
        portal_canvas_bounds,
        anchor_canvas_bounds,
    ) else {
        return;
    };

    let (hovered_label, ports_in, ports_out) =
        read_authoritative_graph_in_models(cx.app.models_mut(), params.binding, |graph| {
            collect_node_label_and_ports(graph, hovered_id)
        })
        .flatten()
        .unwrap_or_else(|| (Arc::<str>::from("node"), 0, 0));

    if let Some(spec) = build_hover_tooltip_overlay_spec(
        params.bounds,
        hovered_id,
        tooltip_anchor,
        params.hovered_portal_hosted,
        hovered_label,
        ports_in,
        ports_out,
    ) {
        push_hover_tooltip_overlay(cx, overlay_children, spec, params.style_tokens.clone());
    }
}

pub(super) fn push_marquee_overlay_if_active<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    overlay_children: &mut Vec<AnyElement>,
    marquee: Option<&MarqueeDragState>,
    bounds: Rect,
    style_tokens: &NodeGraphStyle,
) {
    if let Some(marquee) = marquee
        && marquee.active
    {
        push_marquee_overlay(
            cx,
            overlay_children,
            bounds,
            marquee_rect_screen(marquee),
            style_tokens,
        );
    }
}

pub(super) fn push_overlay_layer_if_needed<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    overlay_children: Vec<AnyElement>,
) {
    if overlay_children.is_empty() {
        return;
    }

    let mut layer = ContainerProps::default();
    layer.layout = LayoutStyle::default();
    layer.layout.size.width = Length::Fill;
    layer.layout.size.height = Length::Fill;
    layer.layout.position = PositionStyle::Relative;

    out.push(cx.hit_test_gate(false, move |cx| {
        vec![cx.container(layer, move |_cx| overlay_children)]
    }));
}
