//! Node graph controls overlay (UI-only).

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, Size, TextBlobId, TextConstraints, TextOverflow, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::{UiHost, retained_bridge::*};

use crate::interaction::NodeGraphConnectionMode;
use crate::io::NodeGraphViewState;
use crate::ui::NodeGraphStyle;
use crate::ui::commands::{
    CMD_NODE_GRAPH_FRAME_ALL, CMD_NODE_GRAPH_FRAME_SELECTION, CMD_NODE_GRAPH_RESET_VIEW,
    CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE, CMD_NODE_GRAPH_ZOOM_IN, CMD_NODE_GRAPH_ZOOM_OUT,
};

use super::OverlayPlacement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlsButton {
    ToggleConnectionMode,
    ZoomIn,
    ZoomOut,
    FrameAll,
    FrameSelection,
    ResetView,
}

struct ControlsLayout {
    panel: Rect,
    buttons: Vec<(ControlsButton, Rect)>,
}

pub struct NodeGraphControlsOverlay {
    canvas_node: fret_core::NodeId,
    view_state: Model<NodeGraphViewState>,
    style: NodeGraphStyle,
    hovered: Option<ControlsButton>,
    pressed: Option<ControlsButton>,
    keyboard_active: Option<ControlsButton>,
    text_blobs: Vec<TextBlobId>,
    placement: OverlayPlacement,
}

impl NodeGraphControlsOverlay {
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
            keyboard_active: None,
            text_blobs: Vec::new(),
            placement: OverlayPlacement::FloatingInCanvas,
        }
    }

    /// Switches to "panel bounds" mode for `NodeGraphPanel` composition.
    pub fn in_panel_bounds(mut self) -> Self {
        self.placement = OverlayPlacement::PanelBounds;
        self
    }

    fn panel_size_px(&self) -> (f32, f32) {
        let pad = self.style.controls_padding.max(0.0);
        let gap = self.style.controls_gap.max(0.0);
        let button = self.style.controls_button_size.max(10.0);

        let items = [
            ControlsButton::ToggleConnectionMode,
            ControlsButton::ZoomIn,
            ControlsButton::ZoomOut,
            ControlsButton::FrameAll,
            ControlsButton::FrameSelection,
            ControlsButton::ResetView,
        ];

        let panel_w = button + 2.0 * pad;
        let panel_h =
            (items.len() as f32) * button + ((items.len() as f32 - 1.0) * gap) + 2.0 * pad;
        (panel_w, panel_h)
    }

    fn compute_layout(&self, bounds: Rect) -> ControlsLayout {
        let margin = self.style.controls_margin.max(0.0);
        let pad = self.style.controls_padding.max(0.0);
        let gap = self.style.controls_gap.max(0.0);
        let button = self.style.controls_button_size.max(10.0);

        let items = [
            ControlsButton::ToggleConnectionMode,
            ControlsButton::ZoomIn,
            ControlsButton::ZoomOut,
            ControlsButton::FrameAll,
            ControlsButton::FrameSelection,
            ControlsButton::ResetView,
        ];

        let (panel_w, panel_h) = self.panel_size_px();

        let x = bounds.origin.x.0 + (bounds.size.width.0 - margin - panel_w).max(0.0);
        let y = bounds.origin.y.0 + margin;
        let panel = match self.placement {
            OverlayPlacement::FloatingInCanvas => Rect::new(
                Point::new(Px(x), Px(y)),
                Size::new(Px(panel_w), Px(panel_h)),
            ),
            OverlayPlacement::PanelBounds => bounds,
        };

        let mut buttons = Vec::with_capacity(items.len());
        let mut cy = panel.origin.y.0 + pad;
        for item in items {
            let rect = Rect::new(
                Point::new(Px(panel.origin.x.0 + pad), Px(cy)),
                Size::new(Px(button), Px(button)),
            );
            buttons.push((item, rect));
            cy += button + gap;
        }

        ControlsLayout { panel, buttons }
    }

    fn button_at(&self, bounds: Rect, position: Point) -> Option<ControlsButton> {
        let layout = self.compute_layout(bounds);
        for (btn, rect) in layout.buttons {
            if rect.contains(position) {
                return Some(btn);
            }
        }
        None
    }

    fn dispatch_button<H: UiHost>(&self, cx: &mut EventCx<'_, H>, btn: ControlsButton) {
        cx.request_focus(self.canvas_node);
        let id = match btn {
            ControlsButton::ToggleConnectionMode => {
                CommandId::from(CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE)
            }
            ControlsButton::ZoomIn => CommandId::from(CMD_NODE_GRAPH_ZOOM_IN),
            ControlsButton::ZoomOut => CommandId::from(CMD_NODE_GRAPH_ZOOM_OUT),
            ControlsButton::FrameAll => CommandId::from(CMD_NODE_GRAPH_FRAME_ALL),
            ControlsButton::FrameSelection => CommandId::from(CMD_NODE_GRAPH_FRAME_SELECTION),
            ControlsButton::ResetView => CommandId::from(CMD_NODE_GRAPH_RESET_VIEW),
        };
        cx.dispatch_command(id);
        cx.request_redraw();
    }

    fn items() -> &'static [ControlsButton] {
        &[
            ControlsButton::ToggleConnectionMode,
            ControlsButton::ZoomIn,
            ControlsButton::ZoomOut,
            ControlsButton::FrameAll,
            ControlsButton::FrameSelection,
            ControlsButton::ResetView,
        ]
    }

    fn next_button(current: Option<ControlsButton>, dir: i32) -> ControlsButton {
        let items = Self::items();
        let idx = current
            .and_then(|c| items.iter().position(|b| *b == c))
            .unwrap_or(0);
        let len = items.len().max(1);
        let idx_i32 = idx as i32;
        let len_i32 = len as i32;
        let mut next = idx_i32 + dir;
        next = ((next % len_i32) + len_i32) % len_i32;
        items[next as usize]
    }

    fn a11y_button_label(btn: ControlsButton) -> &'static str {
        match btn {
            ControlsButton::ToggleConnectionMode => "Toggle connection mode",
            ControlsButton::ZoomIn => "Zoom in",
            ControlsButton::ZoomOut => "Zoom out",
            ControlsButton::FrameAll => "Frame all",
            ControlsButton::FrameSelection => "Frame selection",
            ControlsButton::ResetView => "Reset view",
        }
    }

    fn label_for(btn: ControlsButton, mode: NodeGraphConnectionMode) -> &'static str {
        match btn {
            ControlsButton::ToggleConnectionMode => match mode {
                NodeGraphConnectionMode::Strict => "S",
                NodeGraphConnectionMode::Loose => "L",
            },
            ControlsButton::ZoomIn => "+",
            ControlsButton::ZoomOut => "–",
            ControlsButton::FrameAll => "Fit",
            ControlsButton::FrameSelection => "Sel",
            ControlsButton::ResetView => "1:1",
        }
    }
}

impl<H: UiHost> Widget<H> for NodeGraphControlsOverlay {
    fn is_focusable(&self) -> bool {
        true
    }

    fn measure(&mut self, _cx: &mut MeasureCx<'_, H>) -> Size {
        let (w, h) = self.panel_size_px();
        Size::new(Px(w), Px(h))
    }

    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        let layout = self.compute_layout(bounds);
        layout.panel.contains(position)
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown { key, repeat: _, .. } => match *key {
                KeyCode::ArrowDown => {
                    self.hovered = None;
                    self.pressed = None;
                    self.keyboard_active = Some(Self::next_button(self.keyboard_active, 1));
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                KeyCode::ArrowUp => {
                    self.hovered = None;
                    self.pressed = None;
                    self.keyboard_active = Some(Self::next_button(self.keyboard_active, -1));
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                KeyCode::Home => {
                    self.hovered = None;
                    self.pressed = None;
                    self.keyboard_active = Self::items().first().copied();
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                KeyCode::End => {
                    self.hovered = None;
                    self.pressed = None;
                    self.keyboard_active = Self::items().last().copied();
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                    let btn = self
                        .keyboard_active
                        .or_else(|| Self::items().first().copied())
                        .expect("controls buttons");
                    self.dispatch_button(cx, btn);
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                KeyCode::Escape => {
                    cx.request_focus(self.canvas_node);
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                _ => {}
            },
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.button_at(cx.bounds, *position);
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
                cx.stop_propagation();
                let Some(btn) = self.button_at(cx.bounds, *position) else {
                    return;
                };
                self.pressed = Some(btn);
                cx.capture_pointer(cx.node);
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
                if self.button_at(cx.bounds, *position) == Some(pressed) {
                    self.dispatch_button(cx, pressed);
                }
            }
            _ => {}
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Panel);
        cx.set_label("Controls");
        cx.set_test_id("node_graph.controls");
        cx.set_focusable(true);

        let active = self
            .keyboard_active
            .or_else(|| Self::items().first().copied())
            .expect("controls buttons");
        cx.set_value(Self::a11y_button_label(active));
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        cx.observe_model(&self.view_state, Invalidation::Paint);
        let mode = self
            .view_state
            .read_ref(cx.app, |s| s.interaction.connection_mode)
            .ok()
            .unwrap_or_default();

        let layout = self.compute_layout(cx.bounds);
        let bg = self.style.context_menu_background;
        let border = self.style.context_menu_border;
        let corner = self.style.context_menu_corner_radius;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(21_000),
            rect: layout.panel,
            background: bg,
            border: Edges::all(Px(1.0)),
            border_color: border,
            corner_radii: Corners::all(Px(corner)),
        });

        let text_style = self.style.controls_text_style.clone();
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        for (btn, rect) in &layout.buttons {
            let hovered = self.hovered == Some(*btn)
                || (self.hovered.is_none()
                    && self.pressed.is_none()
                    && cx.focus == Some(cx.node)
                    && self.keyboard_active == Some(*btn));
            let pressed = self.pressed == Some(*btn);
            let button_bg = if pressed {
                self.style.controls_active_background
            } else if hovered {
                self.style.controls_hover_background
            } else {
                Color::TRANSPARENT
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(21_001),
                rect: *rect,
                background: button_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(corner.max(4.0))),
            });

            let label = Self::label_for(*btn, mode);
            let (id, metrics) = cx
                .services
                .text()
                .prepare_str(label, &text_style, constraints);
            self.text_blobs.push(id);

            let tx = rect.origin.x.0 + 0.5 * (rect.size.width.0 - metrics.size.width.0);
            let ty = rect.origin.y.0 + 0.5 * (rect.size.height.0 - metrics.size.height.0);

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(21_002),
                text: id,
                origin: Point::new(Px(tx), Px(ty)),
                color: self.style.controls_text,
            });
        }
    }
}
