use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;
use fret_core::TextStyle;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_port_labels<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        render: &RenderData,
        node_text_style: &TextStyle,
        zoom: f32,
        pin_r: f32,
        pin_gap: f32,
    ) {
        for (port_id, info) in &render.port_labels {
            let Some(center) = render.port_centers.get(port_id).copied() else {
                continue;
            };
            let port_constraints = TextConstraints {
                max_width: Some(info.max_width),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: effective_scale_factor(cx.scale_factor, zoom),
            };
            let (blob, metrics) = self.paint_cache.text_blob(
                cx.services,
                info.label.clone(),
                node_text_style,
                port_constraints,
            );

            let y = Px(center.y.0 - 0.5 * metrics.size.height.0 + metrics.baseline.0);
            let x = match info.dir {
                PortDirection::In => Px(center.x.0 + pin_r + pin_gap),
                PortDirection::Out => Px(center.x.0 - pin_r - pin_gap - metrics.size.width.0),
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(4),
                origin: Point::new(x, y),
                text: blob,
                paint: self.style.paint.context_menu_text.into(),
                outline: None,
                shadow: None,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_pins<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        render: &RenderData,
        marked_ports: &HashSet<PortId>,
        hovered_port: Option<PortId>,
        hovered_port_valid: bool,
        hovered_port_convertible: bool,
        focused_port: Option<PortId>,
        focused_port_valid: bool,
        focused_port_convertible: bool,
        zoom: f32,
    ) {
        for (port_id, rect, color, _hint) in &render.pins {
            let port_id = *port_id;
            let rect = *rect;
            let color = *color;

            if marked_ports.contains(&port_id) {
                self.paint_marked_pin(cx, rect, color, zoom);
            }

            if hovered_port == Some(port_id) {
                let border_color = if hovered_port_valid {
                    color
                } else if hovered_port_convertible {
                    Color::from_srgb_hex_rgb(0xf2_bf_33)
                } else {
                    Color::from_srgb_hex_rgb(0xe6_59_59)
                };
                self.paint_pin_ring(cx, rect, border_color, Px(2.0 / zoom), zoom);
            }

            if hovered_port != Some(port_id) && focused_port == Some(port_id) {
                let border_color = if self.interaction.wire_drag.is_some() {
                    if focused_port_valid {
                        color
                    } else if focused_port_convertible {
                        Color::from_srgb_hex_rgb(0xf2_bf_33)
                    } else {
                        Color::from_srgb_hex_rgb(0xe6_59_59)
                    }
                } else {
                    self.style.paint.node_border_selected
                };
                self.paint_pin_ring(cx, rect, border_color, Px(2.0 / zoom), zoom);
            }

            let r = Px(0.5 * rect.size.width.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect,
                background: fret_core::Paint::Solid(color).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: Corners::all(r),
            });
        }
    }

    pub(super) fn paint_marked_pin<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        rect: Rect,
        color: Color,
        zoom: f32,
    ) {
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
            background: fret_core::Paint::TRANSPARENT.into(),
            border: Edges::all(Px(1.0 / zoom)),
            border_paint: fret_core::Paint::Solid(Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: 0.55,
            })
            .into(),
            corner_radii: Corners::all(r),
        });
    }

    pub(super) fn paint_pin_ring<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        rect: Rect,
        border_color: Color,
        border_width: Px,
        zoom: f32,
    ) {
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
            background: fret_core::Paint::TRANSPARENT.into(),
            border: Edges::all(border_width),
            border_paint: fret_core::Paint::Solid(border_color).into(),
            corner_radii: Corners::all(r),
        });
    }
}
