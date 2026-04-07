//! Blackboard (symbols) overlay (UI-only).
//!
//! This is a window-space overlay hosted outside the canvas render transform (ADR 0126).

use std::collections::BTreeMap;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextBlobId, TextConstraints, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::retained_bridge::*;

use crate::core::{Graph, Symbol, SymbolId};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::ui::compat_transport::NodeGraphEditQueue;
use crate::ui::controller::NodeGraphController;
use crate::ui::screen_space_placement::{AxisAlign, rect_in_bounds};
use crate::ui::style::NodeGraphStyle;

use super::NodeGraphOverlayState;
use super::SymbolRenameOverlay;
use super::blackboard_policy::{
    BlackboardAction, BlackboardActionPlan, blackboard_action_a11y_label,
    blackboard_action_button_label, blackboard_actions_in_order, next_blackboard_action,
    plan_blackboard_action,
};

const PANEL_MARGIN_PX: f32 = 12.0;
const BUTTON_GAP_PX: f32 = 6.0;
const LABEL_PADDING_PX: f32 = 4.0;

#[derive(Debug, Clone)]
struct BlackboardRowLayout {
    symbol: SymbolId,
    label: Rect,
    insert_ref: Rect,
    rename: Rect,
    delete: Rect,
}

#[derive(Debug, Clone)]
struct BlackboardLayout {
    panel: Rect,
    header: Rect,
    add_button: Rect,
    rows: Vec<BlackboardRowLayout>,
}

/// Window-space blackboard (symbols) overlay.
pub struct NodeGraphBlackboardOverlay {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    edits: Option<Model<NodeGraphEditQueue>>,
    controller: Option<NodeGraphController>,
    overlays: Model<NodeGraphOverlayState>,
    canvas_node: fret_core::NodeId,
    style: NodeGraphStyle,

    hovered: Option<BlackboardAction>,
    pressed: Option<BlackboardAction>,
    keyboard_active: Option<BlackboardAction>,
    text_blobs: Vec<TextBlobId>,
    last_layout: Option<BlackboardLayout>,
}

impl NodeGraphBlackboardOverlay {
    pub fn new(
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        overlays: Model<NodeGraphOverlayState>,
        canvas_node: fret_core::NodeId,
        style: NodeGraphStyle,
    ) -> Self {
        Self {
            graph,
            view_state,
            edits: None,
            controller: None,
            overlays,
            canvas_node,
            style,
            hovered: None,
            pressed: None,
            keyboard_active: None,
            text_blobs: Vec::new(),
            last_layout: None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn with_edit_queue(mut self, edits: Model<NodeGraphEditQueue>) -> Self {
        self.edits = Some(edits);
        self
    }

    /// Routes retained blackboard actions through a store-backed controller.
    ///
    /// This is the public advanced retained seam. Raw edit-queue fallback remains
    /// crate-internal compatibility plumbing for focused retained tests and temporary
    /// migration harnesses.
    pub fn with_controller(mut self, controller: NodeGraphController) -> Self {
        self.controller = Some(controller);
        self
    }

    fn submit_transaction<H: fret_ui::UiHost>(&self, host: &mut H, tx: &GraphTransaction) {
        if let Some(controller) = &self.controller {
            let _ = controller.submit_transaction_and_sync_graph_model(host, &self.graph, tx);
            return;
        }

        if let Some(edits) = &self.edits {
            let _ = edits.update(host, |q, _cx| {
                q.push(tx.clone());
            });
        }
    }

    fn row_height_px(&self) -> f32 {
        self.style.paint.context_menu_item_height.max(20.0)
    }

    fn panel_width_px(&self) -> f32 {
        self.style.paint.context_menu_width.max(120.0)
    }

    fn header_height_px(&self) -> f32 {
        self.row_height_px()
    }

    fn panel_size_px_for_rows(&self, rows: usize) -> Size {
        let pad = self.style.paint.context_menu_padding.max(0.0);
        let w = self.panel_width_px();
        let h =
            (self.header_height_px() + rows as f32 * self.row_height_px() + 2.0 * pad).max(24.0);
        Size::new(Px(w), Px(h))
    }

    fn snapshot_symbols<H: fret_ui::UiHost>(&self, host: &H) -> BTreeMap<SymbolId, Symbol> {
        self.graph
            .read_ref(host, |g| g.symbols.clone())
            .ok()
            .unwrap_or_default()
    }

    fn compute_layout(
        &self,
        bounds: Rect,
        symbols: &BTreeMap<SymbolId, Symbol>,
    ) -> BlackboardLayout {
        let size = self.panel_size_px_for_rows(symbols.len());
        let panel = rect_in_bounds(
            bounds,
            size,
            AxisAlign::Start,
            AxisAlign::Start,
            PANEL_MARGIN_PX,
            Point::new(Px(0.0), Px(0.0)),
        );

        let pad = self.style.paint.context_menu_padding.max(0.0);
        let row_h = self.row_height_px();
        let header_h = self.header_height_px();

        let inner_x = panel.origin.x.0 + pad;
        let inner_y = panel.origin.y.0 + pad;
        let inner_w = (panel.size.width.0 - 2.0 * pad).max(0.0);

        let header = Rect::new(
            Point::new(Px(inner_x), Px(inner_y)),
            Size::new(Px(inner_w), Px(header_h)),
        );

        let button_w = row_h.max(18.0);
        let add_button = Rect::new(
            Point::new(
                Px(header.origin.x.0 + (header.size.width.0 - button_w).max(0.0)),
                header.origin.y,
            ),
            Size::new(Px(button_w), Px(header_h)),
        );

        let mut rows = Vec::new();
        let mut y = inner_y + header_h;
        for symbol in symbols.keys().copied() {
            let row = Rect::new(
                Point::new(Px(inner_x), Px(y)),
                Size::new(Px(inner_w), Px(row_h)),
            );

            let btn_w = button_w;
            let delete = Rect::new(
                Point::new(
                    Px(row.origin.x.0 + (row.size.width.0 - btn_w).max(0.0)),
                    row.origin.y,
                ),
                Size::new(Px(btn_w), Px(row_h)),
            );
            let rename = Rect::new(
                Point::new(Px(delete.origin.x.0 - BUTTON_GAP_PX - btn_w), row.origin.y),
                Size::new(Px(btn_w), Px(row_h)),
            );
            let insert_ref = Rect::new(
                Point::new(Px(rename.origin.x.0 - BUTTON_GAP_PX - btn_w), row.origin.y),
                Size::new(Px(btn_w), Px(row_h)),
            );
            let label = Rect::new(
                row.origin,
                Size::new(
                    Px((insert_ref.origin.x.0 - row.origin.x.0 - BUTTON_GAP_PX).max(0.0)),
                    row.size.height,
                ),
            );

            rows.push(BlackboardRowLayout {
                symbol,
                label,
                insert_ref,
                rename,
                delete,
            });

            y += row_h;
        }

        BlackboardLayout {
            panel,
            header,
            add_button,
            rows,
        }
    }

    fn action_at(&self, position: Point) -> Option<BlackboardAction> {
        let layout = self.last_layout.as_ref()?;
        if layout.add_button.contains(position) {
            return Some(BlackboardAction::AddSymbol);
        }
        for row in &layout.rows {
            if row.insert_ref.contains(position) {
                return Some(BlackboardAction::InsertRef { symbol: row.symbol });
            }
            if row.rename.contains(position) {
                return Some(BlackboardAction::Rename { symbol: row.symbol });
            }
            if row.delete.contains(position) {
                return Some(BlackboardAction::Delete { symbol: row.symbol });
            }
        }
        None
    }

    fn dispatch_action<H: fret_ui::UiHost>(
        &mut self,
        host: &mut H,
        bounds: Rect,
        action: BlackboardAction,
        invoked_at_window: Point,
    ) {
        let graph = self
            .graph
            .read_ref(host, |graph| graph.clone())
            .ok()
            .unwrap_or_else(|| Graph::new(crate::core::GraphId::new()));
        let view_state = self
            .view_state
            .read_ref(host, |state| state.clone())
            .ok()
            .unwrap_or_default();

        let Some(plan) =
            plan_blackboard_action(&graph, &view_state, bounds, action, invoked_at_window)
        else {
            return;
        };

        match plan {
            BlackboardActionPlan::Transaction(tx) => self.submit_transaction(host, &tx),
            BlackboardActionPlan::OpenSymbolRename(SymbolRenameOverlay {
                symbol,
                invoked_at_window,
            }) => {
                let _ = self.overlays.update(host, |s, _cx| {
                    s.group_rename = None;
                    s.symbol_rename = Some(SymbolRenameOverlay {
                        symbol,
                        invoked_at_window,
                    });
                });
            }
        }
    }

    fn text_style(&self) -> TextStyle {
        self.style.geometry.context_menu_text_style.clone()
    }
}

impl<H: fret_ui::UiHost> Widget<H> for NodeGraphBlackboardOverlay {
    fn is_focusable(&self) -> bool {
        true
    }

    fn measure(&mut self, cx: &mut MeasureCx<'_, H>) -> Size {
        cx.observe_model(&self.graph, Invalidation::Layout);
        let rows = self
            .graph
            .read_ref(cx.app, |g| g.symbols.len())
            .ok()
            .unwrap_or_default();
        self.panel_size_px_for_rows(rows)
    }

    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.last_layout
            .as_ref()
            .is_some_and(|layout| layout.panel.contains(position))
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Panel);
        cx.set_label("Blackboard");
        cx.set_test_id("node_graph.blackboard");
        cx.set_focusable(true);

        let Some(layout) = self.last_layout.as_ref() else {
            return;
        };
        let symbols = self.snapshot_symbols(&*cx.app);
        let items = blackboard_actions_in_order(&symbols);
        let active = self
            .keyboard_active
            .or_else(|| items.first().copied())
            .unwrap_or(BlackboardAction::AddSymbol);
        let _ = layout;
        cx.set_value(blackboard_action_a11y_label(active));
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown { key, .. } => {
                let Some(layout) = self.last_layout.as_ref() else {
                    return;
                };
                let symbols = self.snapshot_symbols(&*cx.app);
                let items = blackboard_actions_in_order(&symbols);
                let _ = layout;

                match *key {
                    KeyCode::ArrowDown => {
                        self.hovered = None;
                        self.pressed = None;
                        self.keyboard_active =
                            next_blackboard_action(self.keyboard_active, 1, &items);
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    KeyCode::ArrowUp => {
                        self.hovered = None;
                        self.pressed = None;
                        self.keyboard_active =
                            next_blackboard_action(self.keyboard_active, -1, &items);
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    KeyCode::Home => {
                        self.hovered = None;
                        self.pressed = None;
                        self.keyboard_active = items.first().copied();
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    KeyCode::End => {
                        self.hovered = None;
                        self.pressed = None;
                        self.keyboard_active = items.last().copied();
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                        let action = self
                            .keyboard_active
                            .or_else(|| items.first().copied())
                            .unwrap_or(BlackboardAction::AddSymbol);
                        self.dispatch_action(
                            cx.app,
                            cx.bounds,
                            action,
                            Point::new(Px(0.0), Px(0.0)),
                        );
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    KeyCode::Escape => {
                        self.hovered = None;
                        self.pressed = None;
                        self.keyboard_active = None;
                        cx.request_focus(self.canvas_node);
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    _ => {}
                }
            }
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.action_at(*position);
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
                let Some(layout) = self.last_layout.as_ref() else {
                    return;
                };
                if !layout.panel.contains(*position) {
                    return;
                }

                // Ensure keyboard focus can be acquired even when clicking on non-button areas.
                cx.request_focus(cx.node);
                cx.stop_propagation();

                let action = self.action_at(*position);
                self.keyboard_active = action.or(self.keyboard_active);
                let Some(action) = action else {
                    return;
                };
                self.pressed = Some(action);
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
                let Some(layout) = self.last_layout.as_ref() else {
                    return;
                };
                if layout.panel.contains(*position) {
                    cx.stop_propagation();
                }

                let pressed = self.pressed.take();
                cx.release_pointer_capture();
                if pressed.is_some() {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                let Some(pressed) = pressed else {
                    return;
                };
                if self.action_at(*position) == Some(pressed) {
                    self.dispatch_action(cx.app, cx.bounds, pressed, *position);
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.graph, Invalidation::Layout);
        let symbols = self.snapshot_symbols(&*cx.app);
        self.last_layout = Some(self.compute_layout(cx.bounds, &symbols));
        cx.bounds.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
        cx.observe_model(&self.overlays, Invalidation::Paint);

        let symbols = self.snapshot_symbols(&*cx.app);
        let layout = self.compute_layout(cx.bounds, &symbols);
        self.last_layout = Some(layout.clone());

        let bg = self.style.paint.context_menu_background;
        let border = self.style.paint.context_menu_border;
        let hover_bg = self.style.paint.context_menu_hover_background;
        let text_color = self.style.paint.context_menu_text;
        let corner = self.style.paint.context_menu_corner_radius;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(20_900),
            rect: layout.panel,
            background: fret_core::Paint::Solid(bg).into(),

            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(border).into(),

            corner_radii: Corners::all(Px(corner)),
        });

        let text_style = self.text_style();
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };

        // Header title.
        {
            let (id, metrics) = cx
                .services
                .text()
                .prepare_str("Symbols", &text_style, constraints);
            self.text_blobs.push(id);
            let tx = layout.header.origin.x.0 + LABEL_PADDING_PX;
            let ty = layout.header.origin.y.0
                + 0.5 * (layout.header.size.height.0 - metrics.size.height.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(20_901),
                text: id,
                origin: Point::new(Px(tx), Px(ty)),
                paint: (text_color).into(),
                outline: None,
                shadow: None,
            });
        }

        // Add button.
        {
            let hovered = self.hovered == Some(BlackboardAction::AddSymbol);
            let pressed = self.pressed == Some(BlackboardAction::AddSymbol);
            let keyboard = self.keyboard_active == Some(BlackboardAction::AddSymbol);
            let button_bg = if pressed || hovered || keyboard {
                hover_bg
            } else {
                Color::TRANSPARENT
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(20_901),
                rect: layout.add_button,
                background: fret_core::Paint::Solid(button_bg).into(),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),

                corner_radii: Corners::all(Px(corner.max(4.0))),
            });

            let (id, metrics) = cx.services.text().prepare_str(
                blackboard_action_button_label(BlackboardAction::AddSymbol),
                &text_style,
                constraints,
            );
            self.text_blobs.push(id);
            let tx = layout.add_button.origin.x.0
                + 0.5 * (layout.add_button.size.width.0 - metrics.size.width.0);
            let ty = layout.add_button.origin.y.0
                + 0.5 * (layout.add_button.size.height.0 - metrics.size.height.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(20_902),
                text: id,
                origin: Point::new(Px(tx), Px(ty)),
                paint: (text_color).into(),
                outline: None,
                shadow: None,
            });
        }

        for row in &layout.rows {
            let name = symbols
                .get(&row.symbol)
                .map(|s| s.name.as_str())
                .unwrap_or("<missing>");

            let mut draw_button =
                |cx: &mut PaintCx<'_, H>, rect: Rect, action: BlackboardAction, label: &str| {
                    let is_hovered = self.hovered == Some(action);
                    let is_pressed = self.pressed == Some(action);
                    let is_keyboard = self.keyboard_active == Some(action);
                    let button_bg = if is_pressed || is_hovered || is_keyboard {
                        hover_bg
                    } else {
                        Color::TRANSPARENT
                    };
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(20_901),
                        rect,
                        background: fret_core::Paint::Solid(button_bg).into(),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),

                        corner_radii: Corners::all(Px(corner.max(4.0))),
                    });

                    let (id, metrics) =
                        cx.services
                            .text()
                            .prepare_str(label, &text_style, constraints);
                    self.text_blobs.push(id);
                    let tx = rect.origin.x.0 + 0.5 * (rect.size.width.0 - metrics.size.width.0);
                    let ty = rect.origin.y.0 + 0.5 * (rect.size.height.0 - metrics.size.height.0);
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(20_902),
                        text: id,
                        origin: Point::new(Px(tx), Px(ty)),
                        paint: (text_color).into(),
                        outline: None,
                        shadow: None,
                    });
                };

            draw_button(
                cx,
                row.insert_ref,
                BlackboardAction::InsertRef { symbol: row.symbol },
                blackboard_action_button_label(BlackboardAction::InsertRef { symbol: row.symbol }),
            );
            draw_button(
                cx,
                row.rename,
                BlackboardAction::Rename { symbol: row.symbol },
                blackboard_action_button_label(BlackboardAction::Rename { symbol: row.symbol }),
            );
            draw_button(
                cx,
                row.delete,
                BlackboardAction::Delete { symbol: row.symbol },
                blackboard_action_button_label(BlackboardAction::Delete { symbol: row.symbol }),
            );

            let (id, metrics) = cx
                .services
                .text()
                .prepare_str(name, &text_style, constraints);
            self.text_blobs.push(id);
            let tx = row.label.origin.x.0 + LABEL_PADDING_PX;
            let ty = row.label.origin.y.0 + 0.5 * (row.label.size.height.0 - metrics.size.height.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(20_902),
                text: id,
                origin: Point::new(Px(tx), Px(ty)),
                paint: (text_color).into(),
                outline: None,
                shadow: None,
            });
        }
    }
}
