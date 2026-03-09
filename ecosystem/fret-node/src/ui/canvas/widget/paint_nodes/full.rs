use crate::ui::NodeChromeHint;
use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(dead_code)]
    pub(in super::super) fn paint_nodes<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        render: &RenderData,
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

        let node_hints: HashMap<GraphNodeId, NodeChromeHint> =
            if let Some(skin) = self.skin.as_ref() {
                self.graph
                    .read_ref(cx.app, |g| {
                        render
                            .nodes
                            .iter()
                            .map(
                                |(
                                    node,
                                    _rect,
                                    is_selected,
                                    _title,
                                    _body,
                                    _pin_rows,
                                    _handles,
                                    _hint,
                                )| {
                                    (
                                        *node,
                                        skin.node_chrome_hint(g, *node, &self.style, *is_selected),
                                    )
                                },
                            )
                            .collect()
                    })
                    .ok()
                    .unwrap_or_default()
            } else {
                HashMap::new()
            };

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

        for (node, rect, is_selected, title, body, pin_rows, resize_handles, _hint) in &render.nodes
        {
            self.paint_full_node(
                cx,
                *node,
                *rect,
                *is_selected,
                title,
                body.as_ref(),
                *pin_rows,
                resize_handles,
                node_hints.get(node).copied().unwrap_or_default(),
                &node_text_style,
                zoom,
                corner,
                title_pad,
                title_h,
            );
        }

        let pin_r = self.style.geometry.pin_radius / zoom;
        let pin_gap = 8.0 / zoom;

        self.paint_port_labels(cx, render, &node_text_style, zoom, pin_r, pin_gap);
        self.paint_pins(
            cx,
            render,
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
