//! Node-graph editor overlays (UI-only).
//!
//! Overlays are transient, screen-space affordances that should not be serialized into the graph
//! asset. They are hosted outside the canvas render transform (ADR 0135) so they can use regular
//! `fret-ui` widgets (focus, IME, clipboard, semantics).

use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, Size, TextBlobId, TextConstraints, TextOverflow, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::{UiHost, retained_bridge::*};

use super::edit_queue::NodeGraphEditQueue;
use super::internals::NodeGraphInternalsStore;
use super::style::NodeGraphStyle;
use crate::core::GroupId;
use crate::interaction::NodeGraphConnectionMode;
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphTransaction};
use crate::runtime::store::NodeGraphStore;
use crate::ui::commands::{
    CMD_NODE_GRAPH_FRAME_ALL, CMD_NODE_GRAPH_FRAME_SELECTION, CMD_NODE_GRAPH_RESET_VIEW,
    CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE, CMD_NODE_GRAPH_ZOOM_IN, CMD_NODE_GRAPH_ZOOM_OUT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayPlacement {
    /// Positions itself within the canvas bounds (legacy / backwards-compatible).
    FloatingInCanvas,
    /// Treats `cx.bounds` as the overlay's own panel bounds (for `NodeGraphPanel` composition).
    PanelBounds,
}

/// UI-only overlay state for a node graph editor instance.
#[derive(Debug, Default, Clone)]
pub struct NodeGraphOverlayState {
    pub group_rename: Option<GroupRenameOverlay>,
}

#[derive(Debug, Clone)]
pub struct GroupRenameOverlay {
    pub group: GroupId,
    pub invoked_at_window: Point,
}

/// Overlay host that provides a TextInput-backed group rename UI.
///
/// Expected children:
/// - child 0: a `BoundTextInput` bound to `group_rename_text`.
pub struct NodeGraphOverlayHost {
    graph: Model<crate::Graph>,
    edits: Model<NodeGraphEditQueue>,
    overlays: Model<NodeGraphOverlayState>,
    group_rename_text: Model<String>,
    canvas_node: fret_core::NodeId,
    style: NodeGraphStyle,

    last_opened_group: Option<GroupId>,
    group_rename_bounds: Option<Rect>,
    active: bool,
}

impl NodeGraphOverlayHost {
    pub fn new(
        graph: Model<crate::Graph>,
        edits: Model<NodeGraphEditQueue>,
        overlays: Model<NodeGraphOverlayState>,
        group_rename_text: Model<String>,
        canvas_node: fret_core::NodeId,
        style: NodeGraphStyle,
    ) -> Self {
        Self {
            graph,
            edits,
            overlays,
            group_rename_text,
            canvas_node,
            style,
            last_opened_group: None,
            group_rename_bounds: None,
            active: false,
        }
    }

    fn current_group_rename<H: UiHost>(&self, host: &H) -> Option<GroupRenameOverlay> {
        self.overlays
            .read_ref(host, |s| s.group_rename.clone())
            .ok()
            .flatten()
    }

    fn close_group_rename<H: UiHost>(&mut self, host: &mut H) {
        let _ = self.overlays.update(host, |s, _cx| {
            s.group_rename = None;
        });
    }

    fn commit_group_rename<H: UiHost>(&mut self, host: &mut H, group: GroupId) {
        let to = self
            .group_rename_text
            .read_ref(host, |t| t.clone())
            .ok()
            .unwrap_or_default();
        let from = self
            .graph
            .read_ref(host, |g| g.groups.get(&group).map(|gg| gg.title.clone()))
            .ok()
            .flatten()
            .unwrap_or_default();

        if from == to {
            return;
        }

        let tx = GraphTransaction {
            label: Some("Rename Group".to_string()),
            ops: vec![GraphOp::SetGroupTitle {
                id: group,
                from,
                to,
            }],
        };
        let _ = self.edits.update(host, |q, _cx| {
            q.push(tx);
        });
    }

    fn clamp_overlay_rect(&self, desired_origin: Point, bounds: Rect) -> Rect {
        let w = self.style.context_menu_width.max(40.0);
        let h = (self.style.context_menu_item_height.max(20.0)
            + 2.0 * self.style.context_menu_padding)
            .max(24.0);

        let min_x = bounds.origin.x.0;
        let min_y = bounds.origin.y.0;
        let max_x = bounds.origin.x.0 + (bounds.size.width.0 - w).max(0.0);
        let max_y = bounds.origin.y.0 + (bounds.size.height.0 - h).max(0.0);

        let x = desired_origin.x.0.clamp(min_x, max_x);
        let y = desired_origin.y.0.clamp(min_y, max_y);
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }
}

impl<H: UiHost> Widget<H> for NodeGraphOverlayHost {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.active
            && self
                .group_rename_bounds
                .is_some_and(|r| r.contains(position))
    }

    fn semantics_present(&self) -> bool {
        self.active
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &fret_core::Event) {
        let Some(rename) = self.current_group_rename(&*cx.app) else {
            return;
        };

        match event {
            fret_core::Event::KeyDown { key, .. } => match *key {
                KeyCode::Escape => {
                    self.close_group_rename(cx.app);
                    cx.request_focus(self.canvas_node);
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Layout);
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    self.commit_group_rename(cx.app, rename.group);
                    self.close_group_rename(cx.app);
                    cx.request_focus(self.canvas_node);
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Layout);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.overlays, Invalidation::Layout);
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.group_rename_text, Invalidation::Layout);

        let child = cx.children.get(0).copied();
        let rename = self.current_group_rename(&*cx.app);
        self.active = rename.is_some();

        if let Some(rename) = rename {
            if self.last_opened_group != Some(rename.group) {
                self.last_opened_group = Some(rename.group);
                let title = self
                    .graph
                    .read_ref(cx.app, |g| {
                        g.groups.get(&rename.group).map(|gg| gg.title.clone())
                    })
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let _ = self.group_rename_text.update(cx.app, |t, _cx| {
                    *t = title;
                });
                if let Some(child) = child {
                    cx.tree.set_focus(Some(child));
                }
            }

            let rect = self.clamp_overlay_rect(rename.invoked_at_window, cx.bounds);
            self.group_rename_bounds = Some(rect);
            if let Some(child) = child {
                cx.layout_in(child, rect);
            }
        } else {
            self.last_opened_group = None;
            self.group_rename_bounds = None;
            if let Some(child) = child {
                cx.layout_in(
                    child,
                    Rect::new(cx.bounds.origin, Size::new(Px(0.0), Px(0.0))),
                );
                if cx.focus == Some(child) {
                    cx.tree.set_focus(Some(self.canvas_node));
                }
            }
        }

        cx.bounds.size
    }
}

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
                let Some(btn) = self.button_at(cx.bounds, *position) else {
                    return;
                };
                self.pressed = Some(btn);
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
                if self.button_at(cx.bounds, *position) == Some(pressed) {
                    self.dispatch_button(cx, pressed);
                }
            }
            _ => {}
        }
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
            let hovered = self.hovered == Some(*btn);
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
                .prepare(fret_core::TextInput::plain(label, &text_style), constraints);
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

    drag: Option<MiniMapDragState>,
    placement: OverlayPlacement,
}

impl NodeGraphMiniMapOverlay {
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

    fn canvas_bounds_from_internals(
        snapshot: &super::internals::NodeGraphInternalsSnapshot,
    ) -> Rect {
        Rect::new(
            snapshot.transform.bounds_origin,
            snapshot.transform.bounds_size,
        )
    }

    fn canvas_bounds_from_internals_and_view(
        &self,
        canvas_bounds: Rect,
        snapshot: &super::internals::NodeGraphInternalsSnapshot,
    ) -> Rect {
        let t = snapshot.transform;
        let zoom = if t.zoom.is_finite() && t.zoom > 0.0 {
            t.zoom
        } else {
            1.0
        };
        let viewport_w = canvas_bounds.size.width.0 / zoom;
        let viewport_h = canvas_bounds.size.height.0 / zoom;
        Rect::new(
            Point::new(Px(-t.pan.x), Px(-t.pan.y)),
            Size::new(Px(viewport_w), Px(viewport_h)),
        )
    }

    fn invert_window_rect_to_canvas(
        &self,
        r: Rect,
        snapshot: &super::internals::NodeGraphInternalsSnapshot,
    ) -> Rect {
        let t = snapshot.transform;
        let zoom = if t.zoom.is_finite() && t.zoom > 0.0 {
            t.zoom
        } else {
            1.0
        };
        let ox = t.bounds_origin.x.0;
        let oy = t.bounds_origin.y.0;

        let x = (r.origin.x.0 - ox) / zoom - t.pan.x;
        let y = (r.origin.y.0 - oy) / zoom - t.pan.y;
        let w = r.size.width.0 / zoom;
        let h = r.size.height.0 / zoom;
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    fn compute_world_bounds(
        &self,
        canvas_bounds: Rect,
        snapshot: &super::internals::NodeGraphInternalsSnapshot,
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
        let _ = self.view_state.update(host, |s, _cx| {
            s.pan = pan;
        });

        let Some(store) = self.store.as_ref() else {
            return;
        };
        let _ = store.update(host, |store, _cx| {
            let zoom = store.view_state().zoom;
            store.set_viewport(pan, zoom);
        });
    }
}

impl<H: UiHost> Widget<H> for NodeGraphMiniMapOverlay {
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
            background: self.style.context_menu_background,
            border: Edges::all(Px(1.0)),
            border_color: self.style.context_menu_border,
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
                background: node_fill,
                border: Edges::all(Px(0.5)),
                border_color: node_border,
                corner_radii: Corners::all(Px(2.0)),
            });
        }

        let viewport = self.canvas_bounds_from_internals_and_view(canvas_bounds, &snapshot);
        let rr = Self::project_rect(minimap, world, viewport);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(20_002),
            rect: rr,
            background: Color {
                a: 0.12,
                ..self.style.node_border_selected
            },
            border: Edges::all(Px(1.0)),
            border_color: self.style.node_border_selected,
            corner_radii: Corners::all(Px(2.0)),
        });
    }
}
