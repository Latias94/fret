//! Node graph minimap overlay (UI-only).

use std::sync::Arc;

use fret_canvas::view::{PanZoom2D, screen_rect_to_canvas_rect, visible_canvas_rect};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect, SceneOp, Size,
};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};

use crate::io::NodeGraphViewState;
use crate::runtime::store::NodeGraphStore;
use crate::ui::view_queue::{
    NodeGraphSetViewportOptions, NodeGraphViewQueue, NodeGraphViewRequest,
};
use crate::ui::{NodeGraphInternalsSnapshot, NodeGraphInternalsStore, NodeGraphStyle};

use super::OverlayPlacement;

#[derive(Debug, Clone)]
struct MiniMapDragState {
    start_canvas: fret_core::Point,
    start_pan: crate::core::CanvasPoint,
}

/// Navigation wiring knobs for the minimap overlay.
///
/// This is intentionally policy-light: it only affects how viewport updates are emitted.
#[derive(Clone)]
pub enum NodeGraphMiniMapNavigationBinding {
    /// Uses the overlay's default behavior (updates `NodeGraphViewState`, and `NodeGraphStore` when attached).
    Default,
    /// Disables navigation (no viewport updates).
    Disabled,
    /// Sends viewport updates through a UI-side view queue.
    ///
    /// This is useful for B-layer controlled integrations that want the canvas to consume a
    /// message surface rather than allowing widgets to mutate the view model directly.
    ViewQueue(Model<NodeGraphViewQueue>),
}

#[derive(Clone)]
pub struct NodeGraphMiniMapBindings {
    pub navigation: NodeGraphMiniMapNavigationBinding,
}

impl Default for NodeGraphMiniMapBindings {
    fn default() -> Self {
        Self {
            navigation: NodeGraphMiniMapNavigationBinding::Default,
        }
    }
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

    pub fn with_view_queue(mut self, queue: Model<NodeGraphViewQueue>) -> Self {
        self.bindings.navigation = NodeGraphMiniMapNavigationBinding::ViewQueue(queue);
        self
    }

    fn minimap_rect(&self, bounds: Rect) -> Rect {
        if self.placement == OverlayPlacement::PanelBounds {
            return bounds;
        }
        let w = self.style.minimap_width.max(40.0);
        let h = self.style.minimap_height.max(30.0);
        let margin = self.style.minimap_margin.max(0.0);

        let x = bounds.origin.x.0 + (bounds.size.width.0 - margin - w).max(0.0);
        let y = bounds.origin.y.0 + (bounds.size.height.0 - margin - h).max(0.0);
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
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
        fn rect_union(a: Rect, b: Rect) -> Rect {
            let x0 = a.origin.x.0.min(b.origin.x.0);
            let y0 = a.origin.y.0.min(b.origin.y.0);
            let x1 = (a.origin.x.0 + a.size.width.0).max(b.origin.x.0 + b.size.width.0);
            let y1 = (a.origin.y.0 + a.size.height.0).max(b.origin.y.0 + b.size.height.0);
            Rect::new(
                Point::new(Px(x0), Px(y0)),
                Size::new(Px((x1 - x0).max(1.0)), Px((y1 - y0).max(1.0))),
            )
        }

        let mut out: Option<Rect> = None;
        for rect in snapshot.nodes_window.values().copied() {
            let r = self.invert_window_rect_to_canvas(rect, snapshot);
            out = Some(match out {
                Some(prev) => rect_union(prev, r),
                None => r,
            });
        }

        let viewport = self.canvas_bounds_from_internals_and_view(canvas_bounds, snapshot);
        out = Some(match out {
            Some(prev) => rect_union(prev, viewport),
            None => viewport,
        });

        let mut out = out.unwrap_or(viewport);
        let pad = self.style.minimap_world_padding.max(0.0);
        out.origin.x.0 -= pad;
        out.origin.y.0 -= pad;
        out.size.width.0 += 2.0 * pad;
        out.size.height.0 += 2.0 * pad;
        out
    }

    fn project_rect(minimap: Rect, world: Rect, r: Rect) -> Rect {
        let ww = world.size.width.0.max(1.0e-6);
        let wh = world.size.height.0.max(1.0e-6);
        let sx = minimap.size.width.0 / ww;
        let sy = minimap.size.height.0 / wh;
        let scale = sx.min(sy);

        let ox = minimap.origin.x.0 + 0.5 * (minimap.size.width.0 - world.size.width.0 * scale)
            - world.origin.x.0 * scale;
        let oy = minimap.origin.y.0 + 0.5 * (minimap.size.height.0 - world.size.height.0 * scale)
            - world.origin.y.0 * scale;

        Rect::new(
            Point::new(Px(ox + r.origin.x.0 * scale), Px(oy + r.origin.y.0 * scale)),
            Size::new(
                Px((r.size.width.0 * scale).max(1.0)),
                Px((r.size.height.0 * scale).max(1.0)),
            ),
        )
    }

    fn unproject_point(minimap: Rect, world: Rect, p: Point) -> Option<fret_core::Point> {
        let ww = world.size.width.0.max(1.0e-6);
        let wh = world.size.height.0.max(1.0e-6);
        let sx = minimap.size.width.0 / ww;
        let sy = minimap.size.height.0 / wh;
        let scale = sx.min(sy);
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }

        let ox = minimap.origin.x.0 + 0.5 * (minimap.size.width.0 - world.size.width.0 * scale)
            - world.origin.x.0 * scale;
        let oy = minimap.origin.y.0 + 0.5 * (minimap.size.height.0 - world.size.height.0 * scale)
            - world.origin.y.0 * scale;

        let x = (p.x.0 - ox) / scale;
        let y = (p.y.0 - oy) / scale;
        Some(Point::new(Px(x), Px(y)))
    }

    fn pan_to_center_point(
        bounds: Rect,
        zoom: f32,
        canvas_center: fret_core::Point,
    ) -> crate::core::CanvasPoint {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let cx = 0.5 * bounds.size.width.0;
        let cy = 0.5 * bounds.size.height.0;
        crate::core::CanvasPoint {
            x: cx / zoom - canvas_center.x.0,
            y: cy / zoom - canvas_center.y.0,
        }
    }

    fn update_pan<H: UiHost>(&self, host: &mut H, pan: crate::core::CanvasPoint) {
        let zoom = self
            .view_state
            .read_ref(host, |s| s.zoom)
            .ok()
            .unwrap_or(1.0);
        self.update_viewport(host, pan, zoom);
    }

    fn update_viewport<H: UiHost>(&self, host: &mut H, pan: crate::core::CanvasPoint, zoom: f32) {
        let z = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };

        match &self.bindings.navigation {
            NodeGraphMiniMapNavigationBinding::Disabled => {}
            NodeGraphMiniMapNavigationBinding::ViewQueue(queue) => {
                let _ = queue.update(host, |q, _cx| {
                    q.push(NodeGraphViewRequest::SetViewport {
                        pan,
                        zoom: z,
                        options: NodeGraphSetViewportOptions::default(),
                    });
                });
            }
            NodeGraphMiniMapNavigationBinding::Default => {
                let _ = self.view_state.update(host, |s, _cx| {
                    s.pan = pan;
                    s.zoom = z;
                });

                if let Some(store) = self.store.as_ref() {
                    let _ = store.update(host, |store, _cx| {
                        store.set_viewport(pan, z);
                    });
                }
            }
        }
    }
}

impl<H: UiHost> Widget<H> for NodeGraphMiniMapOverlay {
    fn is_focusable(&self) -> bool {
        true
    }

    fn measure(&mut self, _cx: &mut MeasureCx<'_, H>) -> Size {
        let w = self.style.minimap_width.max(0.0);
        let h = self.style.minimap_height.max(0.0);
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
            } => match *key {
                KeyCode::ArrowLeft
                | KeyCode::ArrowRight
                | KeyCode::ArrowUp
                | KeyCode::ArrowDown => {
                    let (pan, zoom) = self
                        .view_state
                        .read_ref(cx.app, |s| (s.pan, s.zoom))
                        .ok()
                        .unwrap_or_default();
                    let zoom = if zoom.is_finite() && zoom > 0.0 {
                        zoom
                    } else {
                        1.0
                    };
                    let step = Self::KEYBOARD_PAN_STEP_SCREEN_PX / zoom;
                    let mut pan = pan;
                    match *key {
                        KeyCode::ArrowLeft => pan.x += step,
                        KeyCode::ArrowRight => pan.x -= step,
                        KeyCode::ArrowUp => pan.y += step,
                        KeyCode::ArrowDown => pan.y -= step,
                        _ => {}
                    }

                    self.update_pan(cx.app, pan);
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                KeyCode::Equal | KeyCode::NumpadAdd | KeyCode::Minus | KeyCode::NumpadSubtract => {
                    let snapshot = self.internals.snapshot();
                    let canvas_bounds = Self::canvas_bounds_from_internals(&snapshot);

                    let (pan, zoom) = self
                        .view_state
                        .read_ref(cx.app, |s| (s.pan, s.zoom))
                        .ok()
                        .unwrap_or_default();
                    let zoom = if zoom.is_finite() && zoom > 0.0 {
                        zoom
                    } else {
                        1.0
                    };

                    let factor = match *key {
                        KeyCode::Equal | KeyCode::NumpadAdd => Self::KEYBOARD_ZOOM_STEP_MUL,
                        KeyCode::Minus | KeyCode::NumpadSubtract => {
                            1.0 / Self::KEYBOARD_ZOOM_STEP_MUL
                        }
                        _ => 1.0,
                    };

                    let mut new_zoom = zoom * factor;
                    let min_zoom = self.style.min_zoom;
                    let max_zoom = self.style.max_zoom;
                    if min_zoom.is_finite()
                        && max_zoom.is_finite()
                        && min_zoom > 0.0
                        && max_zoom > 0.0
                    {
                        let (min_z, max_z) = if min_zoom <= max_zoom {
                            (min_zoom, max_zoom)
                        } else {
                            (max_zoom, min_zoom)
                        };
                        new_zoom = new_zoom.clamp(min_z, max_z);
                    } else {
                        new_zoom = if new_zoom.is_finite() && new_zoom > 0.0 {
                            new_zoom
                        } else {
                            1.0
                        };
                    }

                    // Zoom about the current viewport center in canvas space.
                    let view = PanZoom2D {
                        pan: Point::new(Px(pan.x), Px(pan.y)),
                        zoom,
                    };
                    let vis = visible_canvas_rect(canvas_bounds, view);
                    let cx_canvas = vis.origin.x.0 + 0.5 * vis.size.width.0;
                    let cy_canvas = vis.origin.y.0 + 0.5 * vis.size.height.0;

                    let bw = canvas_bounds.size.width.0;
                    let bh = canvas_bounds.size.height.0;
                    let new_pan = crate::core::CanvasPoint {
                        x: bw / (2.0 * new_zoom) - cx_canvas,
                        y: bh / (2.0 * new_zoom) - cy_canvas,
                    };

                    self.update_viewport(cx.app, new_pan, new_zoom);
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
                let Some(canvas_pt) = Self::unproject_point(minimap, world, *position) else {
                    return;
                };

                let zoom = snapshot.transform.zoom;
                let viewport = self.canvas_bounds_from_internals_and_view(canvas_bounds, &snapshot);
                let viewport_rr = Self::project_rect(minimap, world, viewport);

                let current_pan = self
                    .view_state
                    .read_ref(cx.app, |s| s.pan)
                    .ok()
                    .unwrap_or_default();

                let start_pan = if viewport_rr.contains(*position) {
                    current_pan
                } else {
                    let new_pan = Self::pan_to_center_point(canvas_bounds, zoom, canvas_pt);
                    self.update_pan(cx.app, new_pan);
                    new_pan
                };
                self.drag = Some(MiniMapDragState {
                    start_canvas: canvas_pt,
                    start_pan,
                });

                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let Some(drag) = &self.drag else {
                    return;
                };

                let minimap = self.minimap_rect(cx.bounds);
                let snapshot = self.internals.snapshot();
                let canvas_bounds = Self::canvas_bounds_from_internals(&snapshot);
                let world = self.compute_world_bounds(canvas_bounds, &snapshot);
                let Some(canvas_pt) = Self::unproject_point(minimap, world, *position) else {
                    return;
                };

                let dx = canvas_pt.x.0 - drag.start_canvas.x.0;
                let dy = canvas_pt.y.0 - drag.start_canvas.y.0;
                let pan = crate::core::CanvasPoint {
                    x: drag.start_pan.x - dx,
                    y: drag.start_pan.y - dy,
                };
                self.update_pan(cx.app, pan);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if *button != MouseButton::Left {
                    return;
                }
                if self.drag.take().is_some() {
                    cx.release_pointer_capture();
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
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
        let corner = self.style.context_menu_corner_radius;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(20_000),
            rect: minimap,
            background: fret_core::Paint::Solid(self.style.context_menu_background),

            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(self.style.context_menu_border),

            corner_radii: Corners::all(Px(corner)),
        });

        let node_fill = self.style.node_background;
        let node_border = self.style.node_border;

        for rect in snapshot.nodes_window.values().copied() {
            let r = self.invert_window_rect_to_canvas(rect, &snapshot);
            let rr = Self::project_rect(minimap, world, r);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(20_001),
                rect: rr,
                background: fret_core::Paint::Solid(node_fill),

                border: Edges::all(Px(0.5)),
                border_paint: fret_core::Paint::Solid(node_border),

                corner_radii: Corners::all(Px(2.0)),
            });
        }

        let viewport = self.canvas_bounds_from_internals_and_view(canvas_bounds, &snapshot);
        let rr = Self::project_rect(minimap, world, viewport);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(20_002),
            rect: rr,
            background: fret_core::Paint::Solid(Color {
                a: 0.12,
                ..self.style.node_border_selected
            }),
            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(self.style.node_border_selected),

            corner_radii: Corners::all(Px(2.0)),
        });
    }
}
