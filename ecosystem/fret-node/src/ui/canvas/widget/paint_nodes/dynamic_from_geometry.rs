use crate::ui::canvas::geometry::node_size_default_px;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_nodes_dynamic_from_geometry<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        zoom: f32,
    ) {
        let insert_node_drag_preview = self.interaction.insert_node_drag_preview.as_ref();

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

        let mut node_text_style = self.style.context_menu_text_style.clone();
        node_text_style.size = Px(node_text_style.size.0 / zoom);
        if let Some(lh) = node_text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let corner = Px(self.style.node_corner_radius / zoom);
        let title_pad = self.style.node_padding / zoom;
        let title_h = self.style.node_header_height / zoom;

        if let Some(preview) = insert_node_drag_preview.as_ref() {
            let z = zoom.max(1.0e-6);
            let (w_px, h_px) = node_size_default_px(1, 1, &self.style);
            let w = w_px / z;
            let h = h_px / z;
            let rect = Rect::new(
                Point::new(Px(preview.pos.x.0 - 0.5 * w), Px(preview.pos.y.0 - 0.5 * h)),
                Size::new(Px(w), Px(h)),
            );

            let mut bg = self.style.node_background;
            bg.a *= 0.55;
            let mut border_color = self.style.node_border_selected;
            border_color.a *= 0.85;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: bg,
                border: Edges::all(Px(1.0 / z)),
                border_color,
                corner_radii: Corners::all(corner),
            });

            if !preview.label.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor * zoom,
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    cx.services,
                    preview.label.clone(),
                    &node_text_style,
                    constraints,
                );
                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
                let text_y = Px(inner_y + metrics.baseline.0);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }
        }

        for node in snapshot.selected_nodes.iter().copied() {
            let Some(node_geom) = geom.nodes.get(&node) else {
                continue;
            };
            let rect = node_geom.rect;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: self.style.node_background,
                border: Edges::all(Px(1.0 / zoom)),
                border_color: self.style.node_border_selected,
                corner_radii: Corners::all(corner),
            });

            let show_resize_handle = self
                .interaction
                .node_resize
                .as_ref()
                .is_some_and(|r| r.node == node)
                || self
                    .interaction
                    .last_pos
                    .is_some_and(|p| Self::rect_contains(rect, p));
            if show_resize_handle {
                let handles = self
                    .graph
                    .read_ref(cx.app, |g| {
                        self.presenter.node_resize_handles(g, node, &self.style)
                    })
                    .ok()
                    .unwrap_or_default();
                for handle in NodeResizeHandle::ALL {
                    if !handles.contains(handle) {
                        continue;
                    }
                    let rect = self.node_resize_handle_rect(rect, handle, zoom);
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect,
                        background: self.style.resize_handle_background,
                        border: Edges::all(Px(1.0 / zoom)),
                        border_color: self.style.resize_handle_border,
                        corner_radii: Corners::all(Px(2.0 / zoom)),
                    });
                }
            }
        }

        let resolve_port = |port: PortId| -> Option<(Rect, Color)> {
            let handle = geom.ports.get(&port)?;
            let bounds = handle.bounds;
            let color = self
                .graph
                .read_ref(cx.app, |g| self.presenter.port_color(g, port, &self.style))
                .ok()?;
            Some((bounds, color))
        };

        for port_id in marked_ports {
            let Some((rect, color)) = resolve_port(port_id) else {
                continue;
            };
            let pad = 5.0 / zoom;
            let marker_rect = Rect::new(
                Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                Size::new(
                    Px(rect.size.width.0 + 2.0 * pad),
                    Px(rect.size.height.0 + 2.0 * pad),
                ),
            );
            let r = Px(0.5 * marker_rect.size.width.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect: marker_rect,
                background: Color::TRANSPARENT,
                border: Edges::all(Px(1.0 / zoom)),
                border_color: Color {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: 0.55,
                },
                corner_radii: Corners::all(r),
            });
        }

        if let Some(port_id) = hovered_port
            && let Some((rect, color)) = resolve_port(port_id)
        {
            let border_color = if hovered_port_valid {
                color
            } else if hovered_port_convertible {
                Color {
                    r: 0.95,
                    g: 0.75,
                    b: 0.20,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.90,
                    g: 0.35,
                    b: 0.35,
                    a: 1.0,
                }
            };
            let pad = 2.0 / zoom;
            let hover_rect = Rect::new(
                Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                Size::new(
                    Px(rect.size.width.0 + 2.0 * pad),
                    Px(rect.size.height.0 + 2.0 * pad),
                ),
            );
            let r = Px(0.5 * hover_rect.size.width.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect: hover_rect,
                background: Color::TRANSPARENT,
                border: Edges::all(Px(2.0 / zoom)),
                border_color,
                corner_radii: Corners::all(r),
            });
        }

        if hovered_port != focused_port
            && let Some(port_id) = focused_port
            && let Some((rect, color)) = resolve_port(port_id)
        {
            let border_color = if self.interaction.wire_drag.is_some() {
                if focused_port_valid {
                    color
                } else if focused_port_convertible {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 1.0,
                    }
                } else {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 1.0,
                    }
                }
            } else {
                self.style.node_border_selected
            };

            let pad = 2.0 / zoom;
            let hover_rect = Rect::new(
                Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                Size::new(
                    Px(rect.size.width.0 + 2.0 * pad),
                    Px(rect.size.height.0 + 2.0 * pad),
                ),
            );
            let r = Px(0.5 * hover_rect.size.width.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect: hover_rect,
                background: Color::TRANSPARENT,
                border: Edges::all(Px(2.0 / zoom)),
                border_color,
                corner_radii: Corners::all(r),
            });
        }
    }
}
