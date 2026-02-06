//! Blackboard (symbols) overlay (UI-only).
//!
//! This is a window-space overlay hosted outside the canvas render transform (ADR 0135).

use std::collections::BTreeMap;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextBlobId, TextConstraints, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::retained_bridge::*;

use crate::core::{
    CanvasPoint, CanvasSize, Graph, Node, NodeId, NodeKindKey, SYMBOL_REF_NODE_KIND, Symbol,
    SymbolId, is_symbol_ref_node, symbol_ref_node_data, symbol_ref_target_symbol_id,
};
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphOpBuilderExt as _, GraphTransaction};
use crate::ui::{NodeGraphEditQueue, NodeGraphOverlayState, NodeGraphStyle};

use super::{SymbolRenameOverlay, clamp_rect_to_bounds};

const PANEL_MARGIN_PX: f32 = 12.0;
const BUTTON_GAP_PX: f32 = 6.0;
const LABEL_PADDING_PX: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlackboardAction {
    AddSymbol,
    InsertRef { symbol: SymbolId },
    Rename { symbol: SymbolId },
    Delete { symbol: SymbolId },
}

impl BlackboardAction {
    fn a11y_label(&self) -> &'static str {
        match self {
            BlackboardAction::AddSymbol => "Add symbol",
            BlackboardAction::InsertRef { .. } => "Insert symbol reference",
            BlackboardAction::Rename { .. } => "Rename symbol",
            BlackboardAction::Delete { .. } => "Delete symbol",
        }
    }
}

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
    edits: Model<NodeGraphEditQueue>,
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
        edits: Model<NodeGraphEditQueue>,
        overlays: Model<NodeGraphOverlayState>,
        canvas_node: fret_core::NodeId,
        style: NodeGraphStyle,
    ) -> Self {
        Self {
            graph,
            view_state,
            edits,
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

    fn row_height_px(&self) -> f32 {
        self.style.context_menu_item_height.max(20.0)
    }

    fn panel_width_px(&self) -> f32 {
        self.style.context_menu_width.max(120.0)
    }

    fn header_height_px(&self) -> f32 {
        self.row_height_px()
    }

    fn panel_size_px_for_rows(&self, rows: usize) -> Size {
        let pad = self.style.context_menu_padding.max(0.0);
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

    fn actions_in_order(layout: &BlackboardLayout) -> Vec<BlackboardAction> {
        let mut out = Vec::new();
        out.push(BlackboardAction::AddSymbol);
        for row in &layout.rows {
            out.push(BlackboardAction::InsertRef { symbol: row.symbol });
            out.push(BlackboardAction::Rename { symbol: row.symbol });
            out.push(BlackboardAction::Delete { symbol: row.symbol });
        }
        out
    }

    fn next_action(
        current: Option<BlackboardAction>,
        delta: i32,
        items: &[BlackboardAction],
    ) -> Option<BlackboardAction> {
        if items.is_empty() {
            return None;
        }

        let len = items.len() as i32;
        let idx0 = current
            .and_then(|a| items.iter().position(|x| *x == a))
            .unwrap_or(0) as i32;
        let mut next = idx0 + delta;
        next = ((next % len) + len) % len;
        Some(items[next as usize])
    }

    fn compute_layout(
        &self,
        bounds: Rect,
        symbols: &BTreeMap<SymbolId, Symbol>,
    ) -> BlackboardLayout {
        let size = self.panel_size_px_for_rows(symbols.len());
        let desired_origin = Point::new(
            Px(bounds.origin.x.0 + PANEL_MARGIN_PX),
            Px(bounds.origin.y.0 + PANEL_MARGIN_PX),
        );
        let panel = clamp_rect_to_bounds(Rect::new(desired_origin, size), bounds);

        let pad = self.style.context_menu_padding.max(0.0);
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

    fn default_symbol_name(symbols: &BTreeMap<SymbolId, Symbol>) -> String {
        if !symbols.values().any(|s| s.name == "Symbol") {
            return "Symbol".to_string();
        }

        for i in 2..=9999 {
            let candidate = format!("Symbol {i}");
            if !symbols.values().any(|s| s.name == candidate) {
                return candidate;
            }
        }

        "Symbol".to_string()
    }

    fn viewport_center_canvas_point<H: fret_ui::UiHost>(
        host: &H,
        view: &Model<NodeGraphViewState>,
        bounds: Rect,
    ) -> CanvasPoint {
        let (pan, zoom) = view
            .read_ref(host, |s| (s.pan, s.zoom))
            .ok()
            .unwrap_or_default();
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };

        CanvasPoint {
            x: 0.5 * bounds.size.width.0 / zoom - pan.x,
            y: 0.5 * bounds.size.height.0 / zoom - pan.y,
        }
    }

    fn dispatch_action<H: fret_ui::UiHost>(
        &mut self,
        host: &mut H,
        bounds: Rect,
        action: BlackboardAction,
        invoked_at_window: Point,
    ) {
        match action {
            BlackboardAction::AddSymbol => {
                let symbols = self.snapshot_symbols(host);
                let id = SymbolId::new();
                let symbol = Symbol {
                    name: Self::default_symbol_name(&symbols),
                    ty: None,
                    default_value: None,
                    meta: serde_json::Value::Null,
                };
                let tx = GraphTransaction {
                    label: Some("Add Symbol".to_string()),
                    ops: vec![GraphOp::AddSymbol { id, symbol }],
                };
                let _ = self.edits.update(host, |q, _cx| {
                    q.push(tx);
                });
            }
            BlackboardAction::InsertRef { symbol } => {
                let center = Self::viewport_center_canvas_point(host, &self.view_state, bounds);
                let node = Node {
                    kind: NodeKindKey::new(SYMBOL_REF_NODE_KIND),
                    kind_version: 1,
                    pos: center,
                    selectable: None,
                    draggable: None,
                    connectable: None,
                    deletable: None,
                    parent: None,
                    extent: None,
                    expand_parent: None,
                    size: Some(CanvasSize {
                        width: 140.0,
                        height: 40.0,
                    }),
                    hidden: false,
                    collapsed: false,
                    ports: Vec::new(),
                    data: symbol_ref_node_data(symbol),
                };
                let id = NodeId::new();
                let tx = GraphTransaction {
                    label: Some("Insert Symbol Ref".to_string()),
                    ops: vec![GraphOp::AddNode { id, node }],
                };
                let _ = self.edits.update(host, |q, _cx| {
                    q.push(tx);
                });
            }
            BlackboardAction::Rename { symbol } => {
                let _ = self.overlays.update(host, |s, _cx| {
                    s.group_rename = None;
                    s.symbol_rename = Some(SymbolRenameOverlay {
                        symbol,
                        invoked_at_window,
                    });
                });
            }
            BlackboardAction::Delete { symbol } => {
                let graph = self
                    .graph
                    .read_ref(host, |g| g.clone())
                    .ok()
                    .unwrap_or_else(|| Graph::new(crate::core::GraphId::new()));
                let Some(symbol_value) = graph.symbols.get(&symbol).cloned() else {
                    return;
                };

                let mut ref_nodes: Vec<NodeId> = graph
                    .nodes
                    .iter()
                    .filter_map(|(node_id, node)| {
                        if !is_symbol_ref_node(node) {
                            return None;
                        }
                        let Ok(Some(target)) = symbol_ref_target_symbol_id(*node_id, node) else {
                            return None;
                        };
                        (target == symbol).then_some(*node_id)
                    })
                    .collect();
                ref_nodes.sort();

                let mut ops = Vec::new();
                for node_id in ref_nodes {
                    if let Some(op) = graph.build_remove_node_op(node_id) {
                        ops.push(op);
                    }
                }
                ops.push(GraphOp::RemoveSymbol {
                    id: symbol,
                    symbol: symbol_value,
                });

                let tx = GraphTransaction {
                    label: Some("Delete Symbol".to_string()),
                    ops,
                };
                let _ = self.edits.update(host, |q, _cx| {
                    q.push(tx);
                });
            }
        }
    }

    fn text_style(&self) -> TextStyle {
        self.style.context_menu_text_style.clone()
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
        let items = Self::actions_in_order(layout);
        let active = self
            .keyboard_active
            .or_else(|| items.first().copied())
            .unwrap_or(BlackboardAction::AddSymbol);
        cx.set_value(active.a11y_label());
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown { key, .. } => {
                let Some(layout) = self.last_layout.as_ref() else {
                    return;
                };
                let items = Self::actions_in_order(layout);

                match *key {
                    KeyCode::ArrowDown => {
                        self.hovered = None;
                        self.pressed = None;
                        self.keyboard_active = Self::next_action(self.keyboard_active, 1, &items);
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                    }
                    KeyCode::ArrowUp => {
                        self.hovered = None;
                        self.pressed = None;
                        self.keyboard_active = Self::next_action(self.keyboard_active, -1, &items);
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

        let bg = self.style.context_menu_background;
        let border = self.style.context_menu_border;
        let hover_bg = self.style.context_menu_hover_background;
        let text_color = self.style.context_menu_text;
        let corner = self.style.context_menu_corner_radius;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(20_900),
            rect: layout.panel,
            background: bg,
            border: Edges::all(Px(1.0)),
            border_color: border,
            corner_radii: Corners::all(Px(corner)),
        });

        let text_style = self.text_style();
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
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
                color: text_color,
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
                background: button_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(corner.max(4.0))),
            });

            let (id, metrics) = cx
                .services
                .text()
                .prepare_str("+", &text_style, constraints);
            self.text_blobs.push(id);
            let tx = layout.add_button.origin.x.0
                + 0.5 * (layout.add_button.size.width.0 - metrics.size.width.0);
            let ty = layout.add_button.origin.y.0
                + 0.5 * (layout.add_button.size.height.0 - metrics.size.height.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(20_902),
                text: id,
                origin: Point::new(Px(tx), Px(ty)),
                color: text_color,
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
                        background: button_bg,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
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
                        color: text_color,
                    });
                };

            draw_button(
                cx,
                row.insert_ref,
                BlackboardAction::InsertRef { symbol: row.symbol },
                "R",
            );
            draw_button(
                cx,
                row.rename,
                BlackboardAction::Rename { symbol: row.symbol },
                "E",
            );
            draw_button(
                cx,
                row.delete,
                BlackboardAction::Delete { symbol: row.symbol },
                "X",
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
                color: text_color,
            });
        }
    }
}
