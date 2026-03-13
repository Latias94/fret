use std::sync::Arc;

use fret_core::{Color, Point, Px, Rect};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, Length, PositionStyle, SemanticsDecoration,
    SpacingEdges, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::core::NodeId;
use crate::ui::style::NodeGraphStyle;

use super::hover_anchor::{HoverTooltipAnchor, HoverTooltipAnchorSource};

#[derive(Debug, Clone)]
pub(super) struct HoverTooltipOverlaySpec {
    pub(super) hovered_id: NodeId,
    pub(super) left: Px,
    pub(super) top: Px,
    pub(super) width: Px,
    pub(super) source: HoverTooltipAnchorSource,
    pub(super) hovered_label: Arc<str>,
    pub(super) ports_in: u32,
    pub(super) ports_out: u32,
    pub(super) hide_label_summary: bool,
}

pub(super) fn build_hover_tooltip_overlay_spec(
    bounds: Rect,
    hovered_id: NodeId,
    tooltip_anchor: HoverTooltipAnchor,
    hovered_portal_hosted: bool,
    hovered_label: Arc<str>,
    ports_in: u32,
    ports_out: u32,
) -> Option<HoverTooltipOverlaySpec> {
    let origin_screen = tooltip_anchor.origin_screen;
    let width = tooltip_anchor.width_screen;
    let left = Px(origin_screen.x.0 - bounds.origin.x.0);
    let mut top = Px(origin_screen.y.0 - bounds.origin.y.0 - 30.0);
    if top.0 < 0.0 {
        top = Px(origin_screen.y.0 - bounds.origin.y.0 + 6.0);
    }

    if !left.0.is_finite() || !top.0.is_finite() || !width.0.is_finite() || width.0 <= 0.0 {
        return None;
    }

    Some(HoverTooltipOverlaySpec {
        hovered_id,
        left,
        top,
        width,
        source: tooltip_anchor.source,
        hovered_label,
        ports_in,
        ports_out,
        hide_label_summary: hovered_portal_hosted,
    })
}

pub(super) fn push_hover_tooltip_overlay<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    overlay_children: &mut Vec<AnyElement>,
    spec: HoverTooltipOverlaySpec,
    style_tokens: NodeGraphStyle,
) {
    let source = Arc::<str>::from(match spec.source {
        HoverTooltipAnchorSource::PortalBoundsStore => "portal_bounds_store",
        HoverTooltipAnchorSource::HoverAnchorStore => "hover_anchor_store",
    });
    let source_for_text = source.clone();
    let hovered_id = spec.hovered_id;
    let hovered_label = spec.hovered_label;
    let ports_in = spec.ports_in;
    let ports_out = spec.ports_out;
    let hide_label_summary = spec.hide_label_summary;

    overlay_children.push(
        cx.keyed(("fret-node.portal.tooltip.v1", hovered_id), move |cx| {
            let mut p = ContainerProps::default();
            p.layout.position = PositionStyle::Absolute;
            p.layout.inset.left = Some(spec.left).into();
            p.layout.inset.top = Some(spec.top).into();
            p.layout.size.width = Length::Px(spec.width);
            p.layout.size.height = Length::Px(Px(30.0));
            p.padding = SpacingEdges::all(SpacingLength::Px(Px(4.0)));
            p.snap_to_device_pixels = true;
            p.background = Some(Color {
                a: 0.26,
                ..style_tokens.paint.node_background
            });
            p.border = fret_core::Edges::all(Px(1.0));
            p.border_color = Some(Color {
                a: 0.35,
                ..style_tokens.paint.node_border
            });

            cx.container(p, move |cx| {
                let mut col = ColumnProps::default();
                col.layout.size.width = Length::Fill;
                col.layout.size.height = Length::Fill;
                col.gap = SpacingLength::Px(Px(2.0));
                vec![cx.column(col, move |cx| {
                    let mut lines: Vec<AnyElement> = vec![
                        cx.text(Arc::<str>::from(format!("id:{}", hovered_id.0))),
                        cx.text(Arc::<str>::from(format!("source:{source_for_text}"))),
                    ];

                    if !hide_label_summary {
                        lines.push(cx.text(hovered_label.clone()));
                        lines.push(cx.text(Arc::<str>::from(format!(
                            "in:{} out:{}",
                            ports_in, ports_out
                        ))));
                    }
                    lines
                })]
            })
            .attach_semantics(
                SemanticsDecoration::default()
                    .test_id("node_graph.portal.tooltip")
                    .value(Arc::<str>::from(format!(
                        "source={source}; node_id={}; ports_in={} ports_out={}",
                        hovered_id.0, ports_in, ports_out
                    ))),
            )
        }),
    );
}

pub(super) fn clamp_marquee_overlay_rect_to_bounds(bounds: Rect, rect: Rect) -> Option<Rect> {
    if !bounds.size.width.0.is_finite()
        || !bounds.size.height.0.is_finite()
        || bounds.size.width.0 <= 0.0
        || bounds.size.height.0 <= 0.0
        || rect.size.width.0 <= 0.0
        || rect.size.height.0 <= 0.0
    {
        return None;
    }

    let x0 = rect.origin.x.0.max(bounds.origin.x.0);
    let y0 = rect.origin.y.0.max(bounds.origin.y.0);
    let x1 = (rect.origin.x.0 + rect.size.width.0).min(bounds.origin.x.0 + bounds.size.width.0);
    let y1 = (rect.origin.y.0 + rect.size.height.0).min(bounds.origin.y.0 + bounds.size.height.0);
    let clamped = Rect::new(
        Point::new(Px(x0), Px(y0)),
        fret_core::Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
    );

    (clamped.size.width.0 > 0.0 && clamped.size.height.0 > 0.0).then_some(clamped)
}

pub(super) fn push_marquee_overlay<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    overlay_children: &mut Vec<AnyElement>,
    bounds: Rect,
    rect: Rect,
    style_tokens: &NodeGraphStyle,
) {
    let Some(rect) = clamp_marquee_overlay_rect_to_bounds(bounds, rect) else {
        return;
    };

    let left = Px(rect.origin.x.0 - bounds.origin.x.0);
    let top = Px(rect.origin.y.0 - bounds.origin.y.0);

    let mut p = ContainerProps::default();
    p.layout.position = PositionStyle::Absolute;
    p.layout.inset.left = Some(left).into();
    p.layout.inset.top = Some(top).into();
    p.layout.size.width = Length::Px(rect.size.width);
    p.layout.size.height = Length::Px(rect.size.height);
    p.background = Some(style_tokens.paint.marquee_fill);
    p.border = fret_core::Edges::all(Px(style_tokens.paint.marquee_border_width.max(0.0)));
    p.border_color = Some(style_tokens.paint.marquee_border);
    p.snap_to_device_pixels = true;

    overlay_children.push(cx.keyed("fret-node.marquee.overlay.v1", move |cx| {
        cx.container(p, |_cx| std::iter::empty())
    }));
}
