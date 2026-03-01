use crate::ui::canvas::geometry::node_size_default_px;
use crate::ui::canvas::widget::*;
use crate::ui::{NodeChromeHint, NodeRingHint, PortChromeHint};

fn paint_node_ring(
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
        background: fret_core::Paint::TRANSPARENT,

        border: Edges::all(Px(w / z)),
        border_paint: fret_core::Paint::Solid(ring.color),

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
                background: fret_core::Paint::Solid(bg),

                border: Edges::all(Px(1.0 / z)),
                border_paint: fret_core::Paint::Solid(border_color),
                corner_radii: Corners::all(corner),
            });

            if !preview.label.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
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
                    paint: (self.style.context_menu_text).into(),
                    outline: None,
                    shadow: None,
                });
            }
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
        for node in snapshot.selected_nodes.iter().copied() {
            let Some(node_geom) = geom.nodes.get(&node) else {
                continue;
            };
            let rect = node_geom.rect;

            let hint: NodeChromeHint = if let Some(skin) = skin.as_ref() {
                self.graph
                    .read_ref(cx.app, |g| {
                        skin.node_chrome_hint_with_state(
                            g,
                            node,
                            &self.style,
                            true,
                            focused_node == Some(node),
                        )
                    })
                    .ok()
                    .unwrap_or_default()
            } else {
                NodeChromeHint::default()
            };

            if let Some(ring) = hint.ring_selected {
                paint_node_ring(cx.scene, rect, corner, ring, zoom);
            }
            if focused_node == Some(node)
                && let Some(ring) = hint.ring_focused
            {
                paint_node_ring(cx.scene, rect, corner, ring, zoom);
            }

            let background = hint.background.unwrap_or(self.style.node_background);
            let border_color = hint
                .border_selected
                .or(hint.border)
                .unwrap_or(self.style.node_border_selected);

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: fret_core::Paint::Solid(background),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(corner),
            });

            if let Some(color) = hint.header_background {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        rect.origin,
                        Size::new(rect.size.width, Px(title_h.min(rect.size.height.0))),
                    ),
                    background: fret_core::Paint::Solid(color),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

                    corner_radii: Corners {
                        top_left: corner,
                        top_right: corner,
                        bottom_right: Px(0.0),
                        bottom_left: Px(0.0),
                    },
                });
            }

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: fret_core::Paint::TRANSPARENT,

                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(border_color),

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
                        background: fret_core::Paint::Solid(self.style.resize_handle_background),

                        border: Edges::all(Px(1.0 / zoom)),
                        border_paint: fret_core::Paint::Solid(self.style.resize_handle_border),

                        corner_radii: Corners::all(Px(2.0 / zoom)),
                    });
                }
            }
        }

        if let Some(node) = focused_node
            && !snapshot.selected_nodes.iter().any(|n| *n == node)
            && let Some(node_geom) = geom.nodes.get(&node)
        {
            let rect = node_geom.rect;
            let hint: NodeChromeHint = if let Some(skin) = skin.as_ref() {
                self.graph
                    .read_ref(cx.app, |g| {
                        skin.node_chrome_hint_with_state(g, node, &self.style, false, true)
                    })
                    .ok()
                    .unwrap_or_default()
            } else {
                NodeChromeHint::default()
            };
            if let Some(ring) = hint.ring_focused {
                paint_node_ring(cx.scene, rect, corner, ring, zoom);
            }
        }

        let resolve_port = |port: PortId| -> Option<(Rect, Color)> {
            let handle = geom.ports.get(&port)?;
            let bounds = handle.bounds;
            let color = self
                .graph
                .read_ref(cx.app, |g| {
                    let base = self.presenter.port_color(g, port, &self.style);
                    if let Some(skin) = skin.as_ref() {
                        let hint: PortChromeHint =
                            skin.port_chrome_hint(g, port, &self.style, base);
                        hint.fill.unwrap_or(base)
                    } else {
                        base
                    }
                })
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
                background: fret_core::Paint::TRANSPARENT,

                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(Color {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: 0.55,
                }),
                corner_radii: Corners::all(r),
            });
        }

        if let Some(port_id) = hovered_port
            && let Some((rect, color)) = resolve_port(port_id)
        {
            let border_color = if hovered_port_valid {
                interaction_hint.hover.unwrap_or(color)
            } else if hovered_port_convertible {
                interaction_hint
                    .convertible
                    .unwrap_or_else(|| Color::from_srgb_hex_rgb(0xf2_bf_33))
            } else {
                interaction_hint
                    .invalid
                    .unwrap_or_else(|| Color::from_srgb_hex_rgb(0xe6_59_59))
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
                background: fret_core::Paint::TRANSPARENT,

                border: Edges::all(Px(2.0 / zoom)),
                border_paint: fret_core::Paint::Solid(border_color),
                corner_radii: Corners::all(r),
            });
        }

        if hovered_port != focused_port
            && let Some(port_id) = focused_port
            && let Some((rect, color)) = resolve_port(port_id)
        {
            let border_color = if self.interaction.wire_drag.is_some() {
                if focused_port_valid {
                    interaction_hint.hover.unwrap_or(color)
                } else if focused_port_convertible {
                    interaction_hint
                        .convertible
                        .unwrap_or_else(|| Color::from_srgb_hex_rgb(0xf2_bf_33))
                } else {
                    interaction_hint
                        .invalid
                        .unwrap_or_else(|| Color::from_srgb_hex_rgb(0xe6_59_59))
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
                background: fret_core::Paint::TRANSPARENT,

                border: Edges::all(Px(2.0 / zoom)),
                border_paint: fret_core::Paint::Solid(border_color),
                corner_radii: Corners::all(r),
            });
        }
    }
}
