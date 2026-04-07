//! Blackboard (symbols) overlay (UI-only).
//!
//! This is a window-space overlay hosted outside the canvas render transform (ADR 0126).

use std::collections::BTreeMap;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    SemanticsRole, Size, TextBlobId, TextConstraints, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::retained_bridge::*;

use crate::core::{Graph, Symbol, SymbolId};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::ui::compat_transport::NodeGraphEditQueue;
use crate::ui::controller::NodeGraphController;
use crate::ui::style::NodeGraphStyle;

use super::NodeGraphOverlayState;
use super::SymbolRenameOverlay;
use super::blackboard_layout::{
    BlackboardLayout, blackboard_action_at, blackboard_panel_size, compute_blackboard_layout,
};
use super::blackboard_policy::{
    BlackboardAction, BlackboardActionPlan, blackboard_action_a11y_label,
    blackboard_action_button_label, blackboard_actions_in_order, plan_blackboard_action,
};
use super::panel_button_paint::{paint_panel_button, paint_panel_label};
use super::panel_item_state::{
    clear_panel_item_state, panel_item_visual_state, promote_pointer_target_to_keyboard_item,
    select_panel_keyboard_item,
};
use super::panel_navigation_policy::{PanelKeyboardAction, panel_keyboard_action};
use super::panel_pointer_policy::{release_panel_press, sync_panel_hover};

const LABEL_PADDING_PX: f32 = 4.0;

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
        crate::ui::retained_submit::submit_graph_transaction(
            host,
            self.controller.as_ref(),
            self.edits.as_ref(),
            &self.graph,
            tx,
        );
    }

    fn panel_size_px_for_rows(&self, rows: usize) -> Size {
        blackboard_panel_size(&self.style, rows)
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
        compute_blackboard_layout(&self.style, bounds, symbols.keys().copied())
    }

    fn action_at(&self, position: Point) -> Option<BlackboardAction> {
        self.last_layout
            .as_ref()
            .and_then(|layout| blackboard_action_at(layout, position))
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

                match panel_keyboard_action(*key, self.keyboard_active, &items) {
                    PanelKeyboardAction::Select(action) => {
                        select_panel_keyboard_item(
                            &mut self.hovered,
                            &mut self.pressed,
                            &mut self.keyboard_active,
                            action,
                        );
                        crate::ui::retained_event_tail::finish_paint_event(cx);
                    }
                    PanelKeyboardAction::Activate(action) => {
                        self.dispatch_action(
                            cx.app,
                            cx.bounds,
                            action,
                            Point::new(Px(0.0), Px(0.0)),
                        );
                        crate::ui::retained_event_tail::finish_paint_event(cx);
                    }
                    PanelKeyboardAction::FocusCanvas => {
                        clear_panel_item_state(
                            &mut self.hovered,
                            &mut self.pressed,
                            &mut self.keyboard_active,
                        );
                        crate::ui::retained_event_tail::focus_canvas_and_finish_paint_event(
                            cx,
                            self.canvas_node,
                        );
                    }
                    PanelKeyboardAction::Ignore => {}
                }
            }
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.action_at(*position);
                if hovered.is_some() {
                    cx.set_cursor_icon(CursorIcon::Pointer);
                }
                if sync_panel_hover(&mut self.hovered, hovered) {
                    crate::ui::retained_event_tail::request_paint_repaint(cx);
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
                promote_pointer_target_to_keyboard_item(&mut self.keyboard_active, action);
                let Some(action) = action else {
                    return;
                };
                self.pressed = Some(action);
                cx.capture_pointer(cx.node);
                crate::ui::retained_event_tail::request_paint_repaint(cx);
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

                let released_on = self.action_at(*position);
                let release = release_panel_press(&mut self.pressed, released_on);
                cx.release_pointer_capture();
                if release.had_pressed {
                    crate::ui::retained_event_tail::request_paint_repaint(cx);
                }
                let Some(pressed) = release.activate else {
                    return;
                };
                self.dispatch_action(cx.app, cx.bounds, pressed, *position);
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
            paint_panel_label(
                cx,
                &mut self.text_blobs,
                layout.header,
                "Symbols",
                &text_style,
                constraints,
                text_color,
                LABEL_PADDING_PX,
                DrawOrder(20_901),
            );
        }

        // Add button.
        {
            let state = panel_item_visual_state(
                BlackboardAction::AddSymbol,
                self.hovered,
                self.pressed,
                self.keyboard_active,
                true,
                false,
            );
            let button_bg = if state.active() {
                hover_bg
            } else {
                Color::TRANSPARENT
            };
            paint_panel_button(
                cx,
                &mut self.text_blobs,
                layout.add_button,
                blackboard_action_button_label(BlackboardAction::AddSymbol),
                &text_style,
                constraints,
                button_bg,
                text_color,
                corner,
                DrawOrder(20_901),
                DrawOrder(20_902),
            );
        }

        for row in &layout.rows {
            let name = symbols
                .get(&row.symbol)
                .map(|s| s.name.as_str())
                .unwrap_or("<missing>");

            let mut draw_button =
                |cx: &mut PaintCx<'_, H>, rect: Rect, action: BlackboardAction, label: &str| {
                    let state = panel_item_visual_state(
                        action,
                        self.hovered,
                        self.pressed,
                        self.keyboard_active,
                        true,
                        false,
                    );
                    let button_bg = if state.active() {
                        hover_bg
                    } else {
                        Color::TRANSPARENT
                    };
                    paint_panel_button(
                        cx,
                        &mut self.text_blobs,
                        rect,
                        label,
                        &text_style,
                        constraints,
                        button_bg,
                        text_color,
                        corner,
                        DrawOrder(20_901),
                        DrawOrder(20_902),
                    );
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

            paint_panel_label(
                cx,
                &mut self.text_blobs,
                row.label,
                name,
                &text_style,
                constraints,
                text_color,
                LABEL_PADDING_PX,
                DrawOrder(20_902),
            );
        }
    }
}
