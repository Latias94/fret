//! Blackboard (symbols) overlay (UI-only).
//!
//! This is a window-space overlay hosted outside the canvas render transform (ADR 0126).

use std::collections::BTreeMap;

use fret_core::{CursorIcon, Event, MouseButton, Point, Px, Rect, SemanticsRole, Size, TextBlobId};
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
use super::blackboard_paint::{BlackboardPaintState, paint_blackboard_overlay};
use super::blackboard_policy::{
    BlackboardAction, BlackboardActionPlan, blackboard_action_a11y_label,
    blackboard_actions_in_order, plan_blackboard_action,
};
use super::open_symbol_rename_session;
use super::panel_item_state::{clear_panel_item_state, select_panel_keyboard_item};
use super::panel_navigation_policy::{PanelKeyboardAction, panel_keyboard_action};
use super::panel_pointer_policy::{begin_panel_press, release_panel_press, sync_panel_hover};

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
                    open_symbol_rename_session(
                        s,
                        SymbolRenameOverlay {
                            symbol,
                            invoked_at_window,
                        },
                    );
                });
            }
        }
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

                let action = self.action_at(*position);
                begin_panel_press(cx, &mut self.keyboard_active, &mut self.pressed, action);
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
        paint_blackboard_overlay(
            cx,
            &mut self.text_blobs,
            &self.style,
            &layout,
            &symbols,
            BlackboardPaintState {
                hovered: self.hovered,
                pressed: self.pressed,
                keyboard_active: self.keyboard_active,
            },
        );
    }
}
