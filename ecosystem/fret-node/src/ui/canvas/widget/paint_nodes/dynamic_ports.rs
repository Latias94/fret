use crate::ui::canvas::widget::*;
use crate::ui::{NodeGraphSkinRef, PortChromeHint};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_dynamic_port_adorners<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        geom: &CanvasGeometry,
        skin: Option<&NodeGraphSkinRef>,
        interaction_hint: &crate::ui::InteractionChromeHint,
        marked_ports: &HashSet<PortId>,
        hovered_port: Option<PortId>,
        hovered_port_valid: bool,
        hovered_port_convertible: bool,
        focused_port: Option<PortId>,
        focused_port_valid: bool,
        focused_port_convertible: bool,
        zoom: f32,
    ) {
        for port_id in marked_ports.iter().copied() {
            let Some((rect, color)) = self.resolve_dynamic_port(cx.app, geom, skin, port_id) else {
                continue;
            };
            self.paint_marked_pin(cx, rect, color, zoom);
        }

        if let Some(port_id) = hovered_port
            && let Some((rect, color)) = self.resolve_dynamic_port(cx.app, geom, skin, port_id)
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
            self.paint_pin_ring(cx, rect, border_color, Px(2.0 / zoom), zoom);
        }

        if hovered_port != focused_port
            && let Some(port_id) = focused_port
            && let Some((rect, color)) = self.resolve_dynamic_port(cx.app, geom, skin, port_id)
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
                self.style.paint.node_border_selected
            };
            self.paint_pin_ring(cx, rect, border_color, Px(2.0 / zoom), zoom);
        }
    }

    fn resolve_dynamic_port<H: UiHost>(
        &self,
        app: &H,
        geom: &CanvasGeometry,
        skin: Option<&NodeGraphSkinRef>,
        port: PortId,
    ) -> Option<(Rect, Color)> {
        let handle = geom.ports.get(&port)?;
        let bounds = handle.bounds;
        let color = self
            .graph
            .read_ref(app, |graph| {
                let base = self.presenter.port_color(graph, port, &self.style);
                if let Some(skin) = skin {
                    let hint: PortChromeHint =
                        skin.port_chrome_hint(graph, port, &self.style, base);
                    hint.fill.unwrap_or(base)
                } else {
                    base
                }
            })
            .ok()?;
        Some((bounds, color))
    }
}
