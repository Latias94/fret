use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    Size, TextBlobId, TextConstraints, TextOverflow, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::{UiHost, retained_bridge::*};

use fret_node::io::{NodeGraphConnectionMode, NodeGraphViewState};
use fret_node::ui::style::NodeGraphStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TuningHit {
    ToggleMode,
    ConnRadiusDec,
    ConnRadiusInc,
    EdgeWidthDec,
    EdgeWidthInc,
    ConnThresholdDec,
    ConnThresholdInc,
    NodeDragThresholdDec,
    NodeDragThresholdInc,
    NodeClickDistanceDec,
    NodeClickDistanceInc,
    AutoPanMarginDec,
    AutoPanMarginInc,
    AutoPanSpeedDec,
    AutoPanSpeedInc,
}

#[derive(Debug, Clone)]
struct TuningLayout {
    panel: Rect,
    hits: Vec<(TuningHit, Rect)>,
}

pub struct NodeGraphTuningOverlay {
    canvas_node: fret_core::NodeId,
    view_state: Model<NodeGraphViewState>,
    style: NodeGraphStyle,

    hovered: Option<TuningHit>,
    pressed: Option<TuningHit>,
    text_blobs: Vec<TextBlobId>,
}

impl NodeGraphTuningOverlay {
    pub fn new(
        canvas_node: fret_core::NodeId,
        view_state: Model<NodeGraphViewState>,
        style: NodeGraphStyle,
    ) -> Self {
        Self {
            canvas_node,
            view_state,
            style,
            hovered: None,
            pressed: None,
            text_blobs: Vec::new(),
        }
    }

    fn compute_layout(&self, bounds: Rect) -> TuningLayout {
        let margin = self.style.minimap_margin.max(10.0);
        let pad = self.style.context_menu_padding.max(8.0);
        let gap = 6.0;
        let row_h = self.style.context_menu_item_height.max(22.0);
        let btn = self.style.controls_button_size.max(22.0);

        let panel_w = 280.0;
        let rows = 8.0;
        let panel_h = 2.0 * pad + row_h * rows + gap * (rows - 1.0);

        let x = bounds.origin.x.0 + margin;
        let y = bounds.origin.y.0 + (bounds.size.height.0 - margin - panel_h).max(margin);
        let panel = Rect::new(
            Point::new(Px(x), Px(y)),
            Size::new(Px(panel_w), Px(panel_h)),
        );

        let mut hits: Vec<(TuningHit, Rect)> = Vec::new();

        let left = panel.origin.x.0 + pad;
        let right = panel.origin.x.0 + panel.size.width.0 - pad;

        let mut cy = panel.origin.y.0 + pad;
        let add_row = |hits: &mut Vec<(TuningHit, Rect)>,
                       cy: f32,
                       left: f32,
                       right: f32,
                       row_h: f32,
                       btn: f32,
                       dec: TuningHit,
                       inc: TuningHit| {
            let y = cy + 0.5 * (row_h - btn).max(0.0);
            let dec_rect = Rect::new(
                Point::new(Px(right - 2.0 * btn - 6.0), Px(y)),
                Size::new(Px(btn), Px(btn)),
            );
            let inc_rect = Rect::new(
                Point::new(Px(right - btn), Px(y)),
                Size::new(Px(btn), Px(btn)),
            );
            let label_rect = Rect::new(
                Point::new(Px(left), Px(cy)),
                Size::new(Px((right - left - 2.0 * btn - 12.0).max(0.0)), Px(row_h)),
            );
            hits.push((dec, dec_rect));
            hits.push((inc, inc_rect));
            (label_rect, dec_rect, inc_rect)
        };

        // Row 0: mode
        let mode_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::ToggleMode, mode_btn));
        cy += row_h + gap;

        // Row 1: connection_radius
        let _ = add_row(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::ConnRadiusDec,
            TuningHit::ConnRadiusInc,
        );
        cy += row_h + gap;

        // Row 2: edge_interaction_width
        let _ = add_row(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::EdgeWidthDec,
            TuningHit::EdgeWidthInc,
        );
        cy += row_h + gap;

        // Row 3: connection_drag_threshold
        let _ = add_row(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::ConnThresholdDec,
            TuningHit::ConnThresholdInc,
        );
        cy += row_h + gap;

        // Row 4: node_drag_threshold
        let _ = add_row(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::NodeDragThresholdDec,
            TuningHit::NodeDragThresholdInc,
        );
        cy += row_h + gap;

        // Row 5: node_click_distance
        let _ = add_row(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::NodeClickDistanceDec,
            TuningHit::NodeClickDistanceInc,
        );
        cy += row_h + gap;

        // Row 6: auto_pan.margin
        let _ = add_row(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::AutoPanMarginDec,
            TuningHit::AutoPanMarginInc,
        );
        cy += row_h + gap;

        // Row 7: auto_pan.speed
        let _ = add_row(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::AutoPanSpeedDec,
            TuningHit::AutoPanSpeedInc,
        );

        TuningLayout { panel, hits }
    }

    fn hit_at(&self, bounds: Rect, position: Point) -> Option<TuningHit> {
        let layout = self.compute_layout(bounds);
        for (hit, rect) in layout.hits {
            if rect.contains(position) {
                return Some(hit);
            }
        }
        None
    }

    fn step_for(event: &Event) -> f32 {
        let mods = match event {
            Event::Pointer(fret_core::PointerEvent::Down { modifiers, .. }) => *modifiers,
            Event::Pointer(fret_core::PointerEvent::Up { modifiers, .. }) => *modifiers,
            _ => Default::default(),
        };
        if mods.shift {
            5.0
        } else if mods.ctrl || mods.meta {
            0.25
        } else {
            1.0
        }
    }

    fn apply_hit(&mut self, host: &mut impl UiHost, hit: TuningHit, step_scale: f32) {
        let _ = self.view_state.update(host, |s, _cx| match hit {
            TuningHit::ToggleMode => {
                s.interaction.connection_mode = match s.interaction.connection_mode {
                    NodeGraphConnectionMode::Strict => NodeGraphConnectionMode::Loose,
                    NodeGraphConnectionMode::Loose => NodeGraphConnectionMode::Strict,
                };
            }
            TuningHit::ConnRadiusDec => {
                s.interaction.connection_radius =
                    (s.interaction.connection_radius - 1.0 * step_scale).clamp(0.0, 96.0);
            }
            TuningHit::ConnRadiusInc => {
                s.interaction.connection_radius =
                    (s.interaction.connection_radius + 1.0 * step_scale).clamp(0.0, 96.0);
            }
            TuningHit::EdgeWidthDec => {
                s.interaction.edge_interaction_width =
                    (s.interaction.edge_interaction_width - 1.0 * step_scale).clamp(1.0, 96.0);
            }
            TuningHit::EdgeWidthInc => {
                s.interaction.edge_interaction_width =
                    (s.interaction.edge_interaction_width + 1.0 * step_scale).clamp(1.0, 96.0);
            }
            TuningHit::ConnThresholdDec => {
                s.interaction.connection_drag_threshold =
                    (s.interaction.connection_drag_threshold - 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::ConnThresholdInc => {
                s.interaction.connection_drag_threshold =
                    (s.interaction.connection_drag_threshold + 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::NodeDragThresholdDec => {
                s.interaction.node_drag_threshold =
                    (s.interaction.node_drag_threshold - 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::NodeDragThresholdInc => {
                s.interaction.node_drag_threshold =
                    (s.interaction.node_drag_threshold + 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::NodeClickDistanceDec => {
                s.interaction.node_click_distance =
                    (s.interaction.node_click_distance - 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::NodeClickDistanceInc => {
                s.interaction.node_click_distance =
                    (s.interaction.node_click_distance + 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::AutoPanMarginDec => {
                s.interaction.auto_pan.margin =
                    (s.interaction.auto_pan.margin - 2.0 * step_scale).clamp(0.0, 128.0);
            }
            TuningHit::AutoPanMarginInc => {
                s.interaction.auto_pan.margin =
                    (s.interaction.auto_pan.margin + 2.0 * step_scale).clamp(0.0, 128.0);
            }
            TuningHit::AutoPanSpeedDec => {
                s.interaction.auto_pan.speed =
                    (s.interaction.auto_pan.speed - 50.0 * step_scale).clamp(0.0, 4000.0);
            }
            TuningHit::AutoPanSpeedInc => {
                s.interaction.auto_pan.speed =
                    (s.interaction.auto_pan.speed + 50.0 * step_scale).clamp(0.0, 4000.0);
            }
        });
    }

    fn draw_text(
        &mut self,
        cx: &mut PaintCx<'_, impl UiHost>,
        order: u32,
        origin: Point,
        text: &str,
        color: Color,
    ) {
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let (id, metrics) =
            cx.services
                .text()
                .prepare(text, &self.style.context_menu_text_style, constraints);
        self.text_blobs.push(id);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(order),
            origin: Point::new(origin.x, Px(origin.y.0 + metrics.baseline.0)),
            text: id,
            color,
        });
    }
}

impl<H: UiHost> Widget<H> for NodeGraphTuningOverlay {
    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        self.compute_layout(bounds).panel.contains(position)
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.hit_at(cx.bounds, *position);
                if hovered.is_some() {
                    cx.set_cursor_icon(CursorIcon::Pointer);
                }
                if hovered != self.hovered {
                    self.hovered = hovered;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                let Some(hit) = self.hit_at(cx.bounds, *position) else {
                    return;
                };
                self.pressed = Some(hit);
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                let pressed = self.pressed.take();
                cx.release_pointer_capture();
                if pressed.is_some() {
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                let Some(pressed) = pressed else {
                    return;
                };
                if self.hit_at(cx.bounds, *position) != Some(pressed) {
                    return;
                }

                let step = Self::step_for(event);
                self.apply_hit(cx.app, pressed, step);
                cx.request_focus(self.canvas_node);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            Event::KeyDown { key, .. } => {
                if *key == fret_core::KeyCode::Escape {
                    cx.request_focus(self.canvas_node);
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.view_state, Invalidation::Paint);
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        let state = self
            .view_state
            .read_ref(cx.app, |s| s.clone())
            .ok()
            .unwrap_or_default();
        let layout = self.compute_layout(cx.bounds);

        let corner = self.style.context_menu_corner_radius;
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_000),
            rect: layout.panel,
            background: self.style.context_menu_background,
            border: Edges::all(Px(1.0)),
            border_color: self.style.context_menu_border,
            corner_radii: Corners::all(Px(corner)),
        });

        let pad = self.style.context_menu_padding.max(8.0);
        let gap = 6.0;
        let row_h = self.style.context_menu_item_height.max(22.0);
        let btn = self.style.controls_button_size.max(22.0);

        let left = layout.panel.origin.x.0 + pad;
        let right = layout.panel.origin.x.0 + layout.panel.size.width.0 - pad;

        let mut cy = layout.panel.origin.y.0 + pad;

        let row = |label: &str, value: &str| -> String { format!("{label}: {value}") };

        let mode_str = match state.interaction.connection_mode {
            NodeGraphConnectionMode::Strict => "strict",
            NodeGraphConnectionMode::Loose => "loose",
        };
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Connect mode", mode_str),
            self.style.context_menu_text,
        );
        // Mode button
        let mode_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let mode_bg = if self.hovered == Some(TuningHit::ToggleMode) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: mode_btn_rect,
            background: mode_bg,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        let mode_short = match state.interaction.connection_mode {
            NodeGraphConnectionMode::Strict => "S",
            NodeGraphConnectionMode::Loose => "L",
        };
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(mode_btn_rect.origin.x.0 + 0.5 * (btn - 8.0)),
                Px(mode_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            mode_short,
            self.style.controls_text,
        );
        cy += row_h + gap;

        let draw_step_row = |this: &mut Self,
                             cx: &mut PaintCx<'_, H>,
                             y: f32,
                             label: &str,
                             value: String,
                             dec: TuningHit,
                             inc: TuningHit| {
            let text = row(label, &value);
            this.draw_text(
                cx,
                22_010,
                Point::new(Px(left), Px(y)),
                &text,
                this.style.context_menu_text,
            );

            let dec_rect = Rect::new(
                Point::new(
                    Px(right - 2.0 * btn - 6.0),
                    Px(y + 0.5 * (row_h - btn).max(0.0)),
                ),
                Size::new(Px(btn), Px(btn)),
            );
            let inc_rect = Rect::new(
                Point::new(Px(right - btn), Px(y + 0.5 * (row_h - btn).max(0.0))),
                Size::new(Px(btn), Px(btn)),
            );

            let bg = |hit: TuningHit| {
                if this.hovered == Some(hit) {
                    this.style.controls_hover_background
                } else {
                    Color::TRANSPARENT
                }
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(22_020),
                rect: dec_rect,
                background: bg(dec),
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(corner.max(4.0))),
            });
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(22_020),
                rect: inc_rect,
                background: bg(inc),
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(corner.max(4.0))),
            });

            this.draw_text(
                cx,
                22_021,
                Point::new(
                    Px(dec_rect.origin.x.0 + 0.5 * (btn - 8.0)),
                    Px(dec_rect.origin.y.0 + 0.5 * (btn - 12.0)),
                ),
                "–",
                this.style.controls_text,
            );
            this.draw_text(
                cx,
                22_021,
                Point::new(
                    Px(inc_rect.origin.x.0 + 0.5 * (btn - 8.0)),
                    Px(inc_rect.origin.y.0 + 0.5 * (btn - 12.0)),
                ),
                "+",
                this.style.controls_text,
            );
        };

        draw_step_row(
            self,
            cx,
            cy,
            "Conn radius",
            format!("{:.1}", state.interaction.connection_radius),
            TuningHit::ConnRadiusDec,
            TuningHit::ConnRadiusInc,
        );
        cy += row_h + gap;

        draw_step_row(
            self,
            cx,
            cy,
            "Edge hit width",
            format!("{:.1}", state.interaction.edge_interaction_width),
            TuningHit::EdgeWidthDec,
            TuningHit::EdgeWidthInc,
        );
        cy += row_h + gap;

        draw_step_row(
            self,
            cx,
            cy,
            "Conn drag threshold",
            format!("{:.2}", state.interaction.connection_drag_threshold),
            TuningHit::ConnThresholdDec,
            TuningHit::ConnThresholdInc,
        );
        cy += row_h + gap;

        draw_step_row(
            self,
            cx,
            cy,
            "Node drag threshold",
            format!("{:.2}", state.interaction.node_drag_threshold),
            TuningHit::NodeDragThresholdDec,
            TuningHit::NodeDragThresholdInc,
        );
        cy += row_h + gap;

        draw_step_row(
            self,
            cx,
            cy,
            "Node click distance",
            format!("{:.2}", state.interaction.node_click_distance),
            TuningHit::NodeClickDistanceDec,
            TuningHit::NodeClickDistanceInc,
        );
        cy += row_h + gap;

        draw_step_row(
            self,
            cx,
            cy,
            "Auto-pan margin",
            format!("{:.1}", state.interaction.auto_pan.margin),
            TuningHit::AutoPanMarginDec,
            TuningHit::AutoPanMarginInc,
        );
        cy += row_h + gap;

        draw_step_row(
            self,
            cx,
            cy,
            "Auto-pan speed",
            format!("{:.0}", state.interaction.auto_pan.speed),
            TuningHit::AutoPanSpeedDec,
            TuningHit::AutoPanSpeedInc,
        );
    }

    fn command(&mut self, _cx: &mut CommandCx<'_, H>, _command: &CommandId) -> bool {
        false
    }
}
