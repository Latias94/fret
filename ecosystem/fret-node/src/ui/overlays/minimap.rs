//! Node graph minimap overlay (UI-only).

use std::sync::Arc;

use fret_canvas::view::{PanZoom2D, screen_rect_to_canvas_rect, visible_canvas_rect};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp, Size,
};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};

use crate::io::NodeGraphViewState;
use crate::runtime::store::NodeGraphStore;
use crate::ui::controller::NodeGraphController;
use crate::ui::screen_space_placement::{AxisAlign, rect_in_bounds};
use crate::ui::{NodeGraphInternalsSnapshot, NodeGraphInternalsStore, NodeGraphStyle};

use super::OverlayPlacement;
use super::minimap_navigation_policy::{
    NodeGraphMiniMapBindings, NodeGraphMiniMapNavigationBinding, apply_minimap_viewport_update,
    normalize_minimap_navigation_zoom,
};
use super::minimap_policy::{
    MiniMapKeyboardAction, minimap_keyboard_action_from_key, plan_minimap_keyboard_pan,
    plan_minimap_keyboard_zoom,
};
use super::minimap_projection::{
    minimap_world_bounds, pan_to_center_canvas_point, project_world_rect_to_minimap,
    unproject_minimap_point,
};

#[derive(Debug, Clone)]
struct MiniMapDragState {
    start_canvas: fret_core::Point,
    start_pan: crate::core::CanvasPoint,
}

pub struct NodeGraphMiniMapOverlay {
    canvas_node: fret_core::NodeId,
    graph: Model<crate::Graph>,
    view_state: Model<NodeGraphViewState>,
    store: Option<Model<NodeGraphStore>>,
    internals: Arc<NodeGraphInternalsStore>,
    style: NodeGraphStyle,
    bindings: NodeGraphMiniMapBindings,

    drag: Option<MiniMapDragState>,
    placement: OverlayPlacement,
}

impl NodeGraphMiniMapOverlay {
    const KEYBOARD_PAN_STEP_SCREEN_PX: f32 = 24.0;
    const KEYBOARD_ZOOM_STEP_MUL: f32 = 1.1;

    pub fn new(
        canvas_node: fret_core::NodeId,
        graph: Model<crate::Graph>,
        view_state: Model<NodeGraphViewState>,
        internals: Arc<NodeGraphInternalsStore>,
        style: NodeGraphStyle,
    ) -> Self {
        Self {
            canvas_node,
            graph,
            view_state,
            store: None,
            internals,
            style,
            bindings: NodeGraphMiniMapBindings::default(),
            drag: None,
            placement: OverlayPlacement::FloatingInCanvas,
        }
    }

    /// Switches to "panel bounds" mode for `NodeGraphPanel` composition.
    pub fn in_panel_bounds(mut self) -> Self {
        self.placement = OverlayPlacement::PanelBounds;
        self
    }

    /// Attaches a B-layer runtime store (optional).
    ///
    /// When set, minimap-driven panning also updates the store view-state.
    pub fn with_store(mut self, store: Model<NodeGraphStore>) -> Self {
        self.store = Some(store);
        self
    }

    pub fn with_bindings(mut self, bindings: NodeGraphMiniMapBindings) -> Self {
        self.bindings = bindings;
        self
    }

    /// Routes retained minimap navigation through a store-backed controller.
    ///
    /// This is the public advanced retained seam. Raw viewport transport remains
    /// crate-internal compatibility plumbing behind the controller/store path.
    pub fn with_controller(mut self, controller: NodeGraphController) -> Self {
        self.store = Some(controller.store());
        self.bindings.navigation = NodeGraphMiniMapNavigationBinding::Controller(controller);
        self
    }

    fn minimap_rect(&self, bounds: Rect) -> Rect {
        if self.placement == OverlayPlacement::PanelBounds {
            return bounds;
        }
        let w = self.style.paint.minimap_width.max(40.0);
        let h = self.style.paint.minimap_height.max(30.0);
        let margin = self.style.paint.minimap_margin.max(0.0);

        rect_in_bounds(
            bounds,
            Size::new(Px(w), Px(h)),
            AxisAlign::End,
            AxisAlign::End,
            margin,
            Point::new(Px(0.0), Px(0.0)),
        )
    }

    fn canvas_bounds_from_internals(snapshot: &NodeGraphInternalsSnapshot) -> Rect {
        Rect::new(
            snapshot.transform.bounds_origin,
            snapshot.transform.bounds_size,
        )
    }

    fn canvas_bounds_from_internals_and_view(
        &self,
        canvas_bounds: Rect,
        snapshot: &NodeGraphInternalsSnapshot,
    ) -> Rect {
        let t = snapshot.transform;
        let view = PanZoom2D {
            pan: Point::new(Px(t.pan.x), Px(t.pan.y)),
            zoom: t.zoom,
        };
        visible_canvas_rect(canvas_bounds, view)
    }

    fn invert_window_rect_to_canvas(&self, r: Rect, snapshot: &NodeGraphInternalsSnapshot) -> Rect {
        let t = snapshot.transform;
        let bounds = Rect::new(t.bounds_origin, t.bounds_size);
        let view = PanZoom2D {
            pan: Point::new(Px(t.pan.x), Px(t.pan.y)),
            zoom: t.zoom,
        };
        screen_rect_to_canvas_rect(bounds, view, r)
    }

    fn compute_world_bounds(
        &self,
        canvas_bounds: Rect,
        snapshot: &NodeGraphInternalsSnapshot,
    ) -> Rect {
        let viewport = self.canvas_bounds_from_internals_and_view(canvas_bounds, snapshot);
        let rects = snapshot
            .nodes_window
            .values()
            .copied()
            .map(|rect| self.invert_window_rect_to_canvas(rect, snapshot));
        minimap_world_bounds(
            rects,
            viewport,
            self.style.paint.minimap_world_padding.max(0.0),
        )
    }

    fn update_pan<H: UiHost>(&self, host: &mut H, pan: crate::core::CanvasPoint) {
        let zoom = self
            .view_state
            .read_ref(host, |s| s.zoom)
            .ok()
            .map(normalize_minimap_navigation_zoom)
            .unwrap_or(1.0);
        self.update_viewport(host, pan, zoom);
    }

    fn update_viewport<H: UiHost>(&self, host: &mut H, pan: crate::core::CanvasPoint, zoom: f32) {
        apply_minimap_viewport_update(
            host,
            &self.bindings.navigation,
            &self.view_state,
            self.store.as_ref(),
            pan,
            zoom,
        );
    }
}

impl<H: UiHost> Widget<H> for NodeGraphMiniMapOverlay {
    fn is_focusable(&self) -> bool {
        true
    }

    fn measure(&mut self, _cx: &mut MeasureCx<'_, H>) -> Size {
        let w = self.style.paint.minimap_width.max(0.0);
        let h = self.style.paint.minimap_height.max(0.0);
        Size::new(Px(w), Px(h))
    }

    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        self.minimap_rect(bounds).contains(position)
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown {
                key,
                modifiers: _,
                repeat: _,
            } => {
                let Some(action) = minimap_keyboard_action_from_key(*key) else {
                    return;
                };

                match action {
                    MiniMapKeyboardAction::PanLeft
                    | MiniMapKeyboardAction::PanRight
                    | MiniMapKeyboardAction::PanUp
                    | MiniMapKeyboardAction::PanDown => {
                        let view_state = self
                            .view_state
                            .read_ref(cx.app, |state| state.clone())
                            .ok()
                            .unwrap_or_default();
                        if let Some(pan) = plan_minimap_keyboard_pan(
                            &view_state,
                            action,
                            Self::KEYBOARD_PAN_STEP_SCREEN_PX,
                        ) {
                            self.update_pan(cx.app, pan);
                            crate::ui::retained_event_tail::finish_paint_event(cx);
                        }
                    }
                    MiniMapKeyboardAction::ZoomIn | MiniMapKeyboardAction::ZoomOut => {
                        let snapshot = self.internals.snapshot();
                        let canvas_bounds = Self::canvas_bounds_from_internals(&snapshot);
                        let view_state = self
                            .view_state
                            .read_ref(cx.app, |state| state.clone())
                            .ok()
                            .unwrap_or_default();

                        if let Some((pan, zoom)) = plan_minimap_keyboard_zoom(
                            &view_state,
                            canvas_bounds,
                            self.style.geometry.min_zoom,
                            self.style.geometry.max_zoom,
                            Self::KEYBOARD_ZOOM_STEP_MUL,
                            action,
                        ) {
                            self.update_viewport(cx.app, pan, zoom);
                            crate::ui::retained_event_tail::finish_paint_event(cx);
                        }
                    }
                    MiniMapKeyboardAction::FocusCanvas => {
                        crate::ui::retained_event_tail::focus_canvas_and_finish_paint_event(
                            cx,
                            self.canvas_node,
                        );
                    }
                }
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                let minimap = self.minimap_rect(cx.bounds);
                if !minimap.contains(*position) {
                    return;
                }

                cx.request_focus(self.canvas_node);
                cx.capture_pointer(cx.node);
                cx.stop_propagation();

                let snapshot = self.internals.snapshot();
                let canvas_bounds = Self::canvas_bounds_from_internals(&snapshot);
                let world = self.compute_world_bounds(canvas_bounds, &snapshot);
                let Some(canvas_pt) = unproject_minimap_point(minimap, world, *position) else {
                    return;
                };

                let zoom = snapshot.transform.zoom;
                let viewport = self.canvas_bounds_from_internals_and_view(canvas_bounds, &snapshot);
                let viewport_rr = project_world_rect_to_minimap(minimap, world, viewport);

                let current_pan = self
                    .view_state
                    .read_ref(cx.app, |s| s.pan)
                    .ok()
                    .unwrap_or_default();

                let start_pan = if viewport_rr.contains(*position) {
                    current_pan
                } else {
                    let new_pan = pan_to_center_canvas_point(canvas_bounds, zoom, canvas_pt);
                    self.update_pan(cx.app, new_pan);
                    new_pan
                };
                self.drag = Some(MiniMapDragState {
                    start_canvas: canvas_pt,
                    start_pan,
                });

                crate::ui::retained_event_tail::request_paint_repaint(cx);
            }
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let Some(drag) = &self.drag else {
                    return;
                };

                let minimap = self.minimap_rect(cx.bounds);
                let snapshot = self.internals.snapshot();
                let canvas_bounds = Self::canvas_bounds_from_internals(&snapshot);
                let world = self.compute_world_bounds(canvas_bounds, &snapshot);
                let Some(canvas_pt) = unproject_minimap_point(minimap, world, *position) else {
                    return;
                };

                let dx = canvas_pt.x.0 - drag.start_canvas.x.0;
                let dy = canvas_pt.y.0 - drag.start_canvas.y.0;
                let pan = crate::core::CanvasPoint {
                    x: drag.start_pan.x - dx,
                    y: drag.start_pan.y - dy,
                };
                self.update_pan(cx.app, pan);
                crate::ui::retained_event_tail::request_paint_repaint(cx);
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if *button != MouseButton::Left {
                    return;
                }
                if self.drag.take().is_some() {
                    cx.release_pointer_capture();
                    crate::ui::retained_event_tail::finish_paint_event(cx);
                }
            }
            _ => {}
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Panel);
        cx.set_label("MiniMap");
        cx.set_test_id("node_graph.minimap");
        cx.set_focusable(true);

        let snapshot = self.internals.snapshot();
        cx.set_value(format!("zoom {:.3}", snapshot.transform.zoom));
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);

        let minimap = self.minimap_rect(cx.bounds);
        let snapshot = self.internals.snapshot();
        let canvas_bounds = Self::canvas_bounds_from_internals(&snapshot);
        let world = self.compute_world_bounds(canvas_bounds, &snapshot);
        let corner = self.style.paint.context_menu_corner_radius;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(20_000),
            rect: minimap,
            background: fret_core::Paint::Solid(self.style.paint.context_menu_background).into(),

            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(self.style.paint.context_menu_border).into(),

            corner_radii: Corners::all(Px(corner)),
        });

        let node_fill = self.style.paint.node_background;
        let node_border = self.style.paint.node_border;

        for rect in snapshot.nodes_window.values().copied() {
            let r = self.invert_window_rect_to_canvas(rect, &snapshot);
            let rr = project_world_rect_to_minimap(minimap, world, r);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(20_001),
                rect: rr,
                background: fret_core::Paint::Solid(node_fill).into(),

                border: Edges::all(Px(0.5)),
                border_paint: fret_core::Paint::Solid(node_border).into(),

                corner_radii: Corners::all(Px(2.0)),
            });
        }

        let viewport = self.canvas_bounds_from_internals_and_view(canvas_bounds, &snapshot);
        let rr = project_world_rect_to_minimap(minimap, world, viewport);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(20_002),
            rect: rr,
            background: fret_core::Paint::Solid(Color {
                a: 0.12,
                ..self.style.paint.node_border_selected
            })
            .into(),
            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(self.style.paint.node_border_selected).into(),

            corner_radii: Corners::all(Px(2.0)),
        });
    }
}
