mod draw;
mod marker;
mod target;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;
use fret_core::scene::DashPatternV1;

#[derive(Debug, Clone, Copy)]
pub(super) struct PreviewTargetState {
    hovered_port: Option<PortId>,
    hovered_port_valid: bool,
    hovered_port_convertible: bool,
    focused_port: Option<PortId>,
    focused_port_valid: bool,
    focused_port_convertible: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PreviewWireStyle {
    pub color: Color,
    pub dash: Option<DashPatternV1>,
}

impl PreviewTargetState {
    pub(super) fn from_widget<M: NodeGraphCanvasMiddleware>(
        widget: &NodeGraphCanvasWith<M>,
    ) -> Self {
        Self {
            hovered_port: widget.interaction.hover_port,
            hovered_port_valid: widget.interaction.hover_port_valid,
            hovered_port_convertible: widget.interaction.hover_port_convertible,
            focused_port: widget.interaction.focused_port,
            focused_port_valid: widget.interaction.focused_port_valid,
            focused_port_convertible: widget.interaction.focused_port_convertible,
        }
    }

    pub(super) fn resolve_target(self, geom: &CanvasGeometry, fallback: Point) -> Point {
        let focused_target = self
            .focused_port
            .filter(|_| self.focused_port_valid || self.focused_port_convertible);
        self.hovered_port
            .filter(|_| self.hovered_port_valid || self.hovered_port_convertible)
            .or(focused_target)
            .and_then(|port| geom.port_center(port))
            .unwrap_or(fallback)
    }

    pub(super) fn resolve_style(
        self,
        interaction_hint: crate::ui::InteractionChromeHint,
        preview_color: Color,
    ) -> PreviewWireStyle {
        let invalid = interaction_hint.invalid.unwrap_or_else(|| Color {
            a: 0.95,
            ..Color::from_srgb_hex_rgb(0xe6_59_59)
        });
        let convertible = interaction_hint.convertible.unwrap_or_else(|| Color {
            a: 0.95,
            ..Color::from_srgb_hex_rgb(0xf2_bf_33)
        });
        let preview = interaction_hint.preview_wire.unwrap_or(preview_color);
        let dash_invalid = interaction_hint.dash_invalid;
        let dash_preview = interaction_hint.dash_preview;

        let (color, dash) = if self.hovered_port.is_some()
            && !self.hovered_port_valid
            && !self.hovered_port_convertible
        {
            (invalid, dash_invalid)
        } else if self.hovered_port.is_some()
            && self.hovered_port_convertible
            && !self.hovered_port_valid
        {
            (convertible, dash_preview)
        } else if self.focused_port.is_some()
            && !self.focused_port_valid
            && !self.focused_port_convertible
            && self.hovered_port.is_none()
        {
            (invalid, dash_invalid)
        } else if self.focused_port.is_some()
            && self.focused_port_convertible
            && !self.focused_port_valid
            && self.hovered_port.is_none()
        {
            (convertible, dash_preview)
        } else {
            (preview, dash_preview)
        };

        PreviewWireStyle { color, dash }
    }
}

pub(super) fn push_drop_marker(scene: &mut fret_core::Scene, pos: Point, color: Color, zoom: f32) {
    marker::push_drop_marker(scene, pos, color, zoom);
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_wire_drag_preview<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        render: &RenderData,
        geom: &CanvasGeometry,
        zoom: f32,
        interaction_hint: crate::ui::InteractionChromeHint,
        outline_budget: &mut WorkBudget,
        outline_budget_skipped: &mut u32,
    ) {
        let Some(wire_drag) = self.interaction.wire_drag.clone() else {
            return;
        };

        let preview =
            target::resolve_preview_target_and_style(self, geom, wire_drag.pos, interaction_hint);

        let mut draw_preview = |from: Point| {
            draw::push_preview_wire_path(
                self,
                cx,
                from,
                preview.to,
                zoom,
                interaction_hint,
                preview.style,
                outline_budget,
                outline_budget_skipped,
            );
        };

        match &wire_drag.kind {
            WireDragKind::New { from, bundle } => {
                let ports = if bundle.is_empty() {
                    std::slice::from_ref(from)
                } else {
                    bundle.as_slice()
                };
                for port in ports {
                    if let Some(from) = render.port_centers.get(port).copied() {
                        draw_preview(from);
                    }
                }
            }
            WireDragKind::Reconnect { fixed, .. } => {
                if let Some(from) = render.port_centers.get(fixed).copied() {
                    draw_preview(from);
                }
            }
            WireDragKind::ReconnectMany { edges } => {
                for (_edge, _endpoint, fixed) in edges {
                    if let Some(from) = render.port_centers.get(fixed).copied() {
                        draw_preview(from);
                    }
                }
            }
        }
    }
}
