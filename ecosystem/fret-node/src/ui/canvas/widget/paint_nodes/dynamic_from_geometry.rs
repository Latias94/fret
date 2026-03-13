use crate::ui::NodeRingHint;
use crate::ui::canvas::widget::*;

pub(super) fn paint_node_ring(
    scene: &mut fret_core::Scene,
    rect: Rect,
    corner: Px,
    ring: NodeRingHint,
    zoom: f32,
) {
    let pad = ring.pad;
    let w = ring.width;
    if !pad.is_finite() || !w.is_finite() || w <= 0.0 || pad < 0.0 {
        return;
    }
    let z = zoom.max(1.0e-6);
    let pad = pad / z;
    let ring_rect = Rect::new(
        Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
        Size::new(
            Px(rect.size.width.0 + 2.0 * pad),
            Px(rect.size.height.0 + 2.0 * pad),
        ),
    );
    let ring_corner = Px((corner.0 + pad).max(0.0));
    scene.push(SceneOp::Quad {
        order: DrawOrder(3),
        rect: ring_rect,
        background: fret_core::Paint::TRANSPARENT.into(),

        border: Edges::all(Px(w / z)),
        border_paint: fret_core::Paint::Solid(ring.color).into(),

        corner_radii: Corners::all(ring_corner),
    });
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_nodes_dynamic_from_geometry<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        zoom: f32,
    ) {
        let insert_node_drag_preview = self.interaction.insert_node_drag_preview.clone();

        let hovered_port = self.interaction.hover_port;
        let hovered_port_valid = self.interaction.hover_port_valid;
        let hovered_port_convertible = self.interaction.hover_port_convertible;
        let focused_port = self.interaction.focused_port;
        let focused_port_valid = self.interaction.focused_port_valid;
        let focused_port_convertible = self.interaction.focused_port_convertible;

        let marked_ports: HashSet<PortId> =
            match self.interaction.wire_drag.as_ref().map(|w| &w.kind) {
                Some(WireDragKind::New { bundle, .. }) if bundle.len() > 1 => {
                    bundle.iter().copied().collect()
                }
                Some(WireDragKind::ReconnectMany { edges }) if edges.len() > 1 => edges
                    .iter()
                    .map(|(_edge, _endpoint, fixed)| *fixed)
                    .collect(),
                _ => HashSet::new(),
            };

        let mut node_text_style = self.style.geometry.context_menu_text_style.clone();
        node_text_style.size = Px(node_text_style.size.0 / zoom);
        if let Some(lh) = node_text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let corner = Px(self.style.paint.node_corner_radius / zoom);
        let title_pad = self.style.geometry.node_padding / zoom;
        let title_h = self.style.geometry.node_header_height / zoom;

        if let Some(preview) = insert_node_drag_preview.as_ref() {
            self.paint_insert_node_drag_preview(
                cx,
                preview,
                &node_text_style,
                zoom,
                corner,
                title_pad,
                title_h,
            );
        }

        let skin = self.skin.clone();
        let interaction_hint = if let Some(skin) = skin.as_ref() {
            self.graph
                .read_ref(cx.app, |g| skin.interaction_chrome_hint(g, &self.style))
                .ok()
                .unwrap_or_default()
        } else {
            crate::ui::InteractionChromeHint::default()
        };
        let focused_node = self.interaction.focused_node;
        self.paint_dynamic_selected_nodes(
            cx,
            snapshot,
            geom,
            skin.as_ref(),
            focused_node,
            corner,
            title_h,
            zoom,
        );
        self.paint_dynamic_focused_node_ring(
            cx,
            snapshot,
            geom,
            skin.as_ref(),
            focused_node,
            corner,
            zoom,
        );
        self.paint_dynamic_port_adorners(
            cx,
            geom,
            skin.as_ref(),
            &interaction_hint,
            &marked_ports,
            hovered_port,
            hovered_port_valid,
            hovered_port_convertible,
            focused_port,
            focused_port_valid,
            focused_port_convertible,
            zoom,
        );
    }
}
