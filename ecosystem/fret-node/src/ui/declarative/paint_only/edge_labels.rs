use std::sync::Arc;

use fret_core::{Corners, Point, Px, Rect, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, Length, PositionStyle, SemanticsDecoration,
    SpacingEdges, SpacingLength, TextProps,
};
use fret_ui::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use fret_ui::{ElementContext, UiHost};

use crate::core::{EdgeId, Graph};
use crate::ui::NodeGraphSurfaceBinding;
use crate::ui::style::NodeGraphStyle;

use super::cache::EdgePathDraw;
use super::surface_support::read_authoritative_graph_in_models;
use super::{
    NodeGraphDeclarativeEdgeLabelRenderer, NodeGraphEdgeLabelHitTestMode, NodeGraphEdgeLabelLayout,
};

#[derive(Debug, Clone)]
struct EdgeLabelInfo {
    edge: EdgeId,
    center: Point,
    label: Option<Arc<str>>,
}

pub(super) fn push_edge_label_overlays<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    overlay_children: &mut Vec<AnyElement>,
    interactive_overlay_children: &mut Vec<AnyElement>,
    binding: &NodeGraphSurfaceBinding,
    edge_draws: Option<&[EdgePathDraw]>,
    bounds: Rect,
    zoom: f32,
    style_tokens: &NodeGraphStyle,
    mut edge_label_renderer: Option<&mut dyn NodeGraphDeclarativeEdgeLabelRenderer<H>>,
) {
    let Some(edge_draws) = edge_draws else {
        return;
    };
    if !bounds.size.width.0.is_finite()
        || !bounds.size.height.0.is_finite()
        || bounds.size.width.0 <= 0.0
        || bounds.size.height.0 <= 0.0
    {
        return;
    }

    let internals = binding.internals_store().snapshot();
    let custom_renderer_enabled = edge_label_renderer.is_some();
    let graph_snapshot = if custom_renderer_enabled {
        read_authoritative_graph_in_models(cx.app.models_mut(), binding, Clone::clone)
            .unwrap_or_default()
    } else {
        Graph::default()
    };
    for (ordinal, info) in edge_draws
        .iter()
        .filter_map(|draw| {
            edge_label_info(
                draw,
                &internals.edge_centers_window,
                bounds,
                custom_renderer_enabled,
            )
        })
        .enumerate()
    {
        let style_tokens = style_tokens.clone();
        let graph_snapshot = graph_snapshot.clone();
        let custom_hit_test_mode = if let Some(renderer) = edge_label_renderer.as_deref_mut() {
            let layout = NodeGraphEdgeLabelLayout {
                edge: info.edge,
                edge_center_window: info.center,
                label: info.label.clone(),
                zoom,
            };
            renderer.edge_label_hit_test_mode(&graph_snapshot, &layout)
        } else {
            NodeGraphEdgeLabelHitTestMode::Transparent
        };
        let custom_hit_testable =
            custom_hit_test_mode == NodeGraphEdgeLabelHitTestMode::ChildBounds;
        let target_children = if custom_hit_testable {
            &mut *interactive_overlay_children
        } else {
            &mut *overlay_children
        };
        target_children.push(cx.keyed(("fret-node.edge-label.v1", info.edge), |cx| {
            let custom_children = if let Some(renderer) = edge_label_renderer.as_deref_mut() {
                let layout = NodeGraphEdgeLabelLayout {
                    edge: info.edge,
                    edge_center_window: info.center,
                    label: info.label.clone(),
                    zoom,
                };
                renderer
                    .render_edge_label(cx, &graph_snapshot, layout)
                    .into_vec()
            } else {
                Vec::new()
            };
            let custom_hit_testable = custom_hit_testable && !custom_children.is_empty();
            edge_label_host_element(
                cx,
                bounds,
                info.clone(),
                ordinal,
                style_tokens.clone(),
                custom_children,
                custom_hit_testable,
            )
        }));
    }
}

fn edge_label_info(
    draw: &EdgePathDraw,
    centers: &std::collections::BTreeMap<EdgeId, Point>,
    bounds: Rect,
    custom_renderer_enabled: bool,
) -> Option<EdgeLabelInfo> {
    let label = draw.label.clone();
    if label.is_none() && !custom_renderer_enabled {
        return None;
    }
    let center = centers.get(&draw.edge).copied()?;
    if !center.x.0.is_finite()
        || !center.y.0.is_finite()
        || center.x.0 < bounds.origin.x.0
        || center.y.0 < bounds.origin.y.0
        || center.x.0 > bounds.origin.x.0 + bounds.size.width.0
        || center.y.0 > bounds.origin.y.0 + bounds.size.height.0
    {
        return None;
    }

    Some(EdgeLabelInfo {
        edge: draw.edge,
        center,
        label,
    })
}

fn edge_label_host_element<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    bounds: Rect,
    info: EdgeLabelInfo,
    ordinal: usize,
    style_tokens: NodeGraphStyle,
    custom_children: Vec<AnyElement>,
    custom_hit_testable: bool,
) -> AnyElement {
    let center = info.center;
    let mut surface = fret_ui::element::ManagedSurfaceProps::default();
    surface.layout.position = PositionStyle::Absolute;
    surface.layout.inset.left = InsetEdge::Px(Px(0.0));
    surface.layout.inset.top = InsetEdge::Px(Px(0.0));
    surface.layout.size.width = Length::Px(bounds.size.width);
    surface.layout.size.height = Length::Px(bounds.size.height);

    cx.managed_surface(
        surface,
        move |cx| {
            let Some(child) = cx.children().first().copied() else {
                return;
            };
            let size = cx.measure_child(
                child,
                LayoutConstraints::new(
                    LayoutSize::new(None, None),
                    LayoutSize::new(
                        AvailableSpace::Definite(bounds.size.width),
                        AvailableSpace::Definite(bounds.size.height),
                    ),
                ),
            );
            let rect = Rect::new(
                Point::new(
                    Px(center.x.0 - size.width.0 / 2.0),
                    Px(center.y.0 - size.height.0 / 2.0),
                ),
                size,
            );
            cx.layout_child(child, rect);
            if custom_hit_testable {
                cx.set_hit_test_rects([rect]);
            } else {
                cx.set_hit_test_rects([]);
            }
        },
        move |cx| {
            for child in cx.children().to_vec() {
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint_child(child, bounds);
                }
            }
        },
        move |cx| {
            if !custom_children.is_empty() {
                return edge_label_custom_children(cx, custom_children);
            }

            match info.label.clone() {
                Some(label) => vec![edge_label_child(
                    cx,
                    info.edge,
                    label,
                    ordinal,
                    &style_tokens,
                )],
                None => Vec::new(),
            }
        },
    )
}

fn edge_label_custom_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
) -> Vec<AnyElement> {
    if children.len() == 1 {
        return children;
    }

    vec![cx.container(ContainerProps::default(), move |_cx| children)]
}

fn edge_label_child<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    edge: EdgeId,
    label: Arc<str>,
    ordinal: usize,
    style_tokens: &NodeGraphStyle,
) -> AnyElement {
    let padding = style_tokens.paint.edge_label_padding.max(0.0);
    let mut container = ContainerProps::default();
    container.layout.size.max_width = Some(Length::Px(Px(style_tokens
        .paint
        .edge_label_max_width
        .max(1.0))));
    container.padding = SpacingEdges::all(SpacingLength::Px(Px(padding)));
    container.snap_to_device_pixels = true;
    container.background = Some(style_tokens.paint.edge_label_background);
    container.corner_radii = Corners::all(Px(style_tokens.paint.edge_label_corner_radius.max(0.0)));
    container.border =
        fret_core::Edges::all(Px(style_tokens.paint.edge_label_border_width.max(0.0)));
    container.border_color = Some(style_tokens.paint.edge_label_border);

    let mut text = TextProps::new(label.clone());
    text.style = Some(style_tokens.paint.edge_label_text_style.clone());
    text.color = Some(style_tokens.paint.edge_label_text);
    text.layout.size.max_width = Some(Length::Px(Px(style_tokens
        .paint
        .edge_label_max_width
        .max(1.0))));

    cx.container(container, move |cx| vec![cx.text_props(text)])
        .attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Generic)
                .label(label)
                .test_id(Arc::<str>::from(format!("node_graph.edge_label.{ordinal}")))
                .value(Arc::<str>::from(format!("edge_id={}", edge.0))),
        )
}
