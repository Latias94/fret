use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    Size, TextBlobId, TextConstraints, TextOverflow, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::{UiHost, retained_bridge::*};

use fret_node::io::{
    NodeGraphConnectionMode, NodeGraphDragHandleMode, NodeGraphViewState,
    NodeGraphZoomActivationKey,
};
use fret_node::runtime::store::NodeGraphStore;
use fret_node::ui::style::NodeGraphStyle;

#[derive(Debug, Clone)]
pub struct NodeGraphTuningCommands {
    pub reset_graph: CommandId,
    pub spawn_stress_1k: CommandId,
    pub spawn_stress_5k: CommandId,
    pub spawn_stress_10k: CommandId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TuningHit {
    ToggleMode,
    TogglePanOnScroll,
    TogglePanInertia,
    ToggleZoomOnScroll,
    CycleZoomActivationKey,
    ConnRadiusDec,
    ConnRadiusInc,
    EdgeWidthDec,
    EdgeWidthInc,
    ConnThresholdDec,
    ConnThresholdInc,
    ToggleConnectOnClick,
    NodeDragThresholdDec,
    NodeDragThresholdInc,
    ToggleNodeDragHandle,
    NodeClickDistanceDec,
    NodeClickDistanceInc,
    ToggleTranslateExtent,
    TranslateExtentDec,
    TranslateExtentInc,
    ToggleNodeExtent,
    NodeExtentDec,
    NodeExtentInc,
    AutoPanMarginDec,
    AutoPanMarginInc,
    AutoPanSpeedDec,
    AutoPanSpeedInc,
    ToggleEdgesReconnectable,
    ResetGraph,
    SpawnStress1k,
    SpawnStress5k,
    SpawnStress10k,
}

#[derive(Debug, Clone)]
struct TuningLayout {
    panel: Rect,
    hits: Vec<(TuningHit, Rect)>,
}

pub struct NodeGraphTuningOverlay {
    canvas_node: fret_core::NodeId,
    view_state: Model<NodeGraphViewState>,
    store: Option<Model<NodeGraphStore>>,
    style: NodeGraphStyle,
    commands: Option<NodeGraphTuningCommands>,

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
            store: None,
            style,
            commands: None,
            hovered: None,
            pressed: None,
            text_blobs: Vec::new(),
        }
    }

    pub fn with_store(mut self, store: Model<NodeGraphStore>) -> Self {
        self.store = Some(store);
        self
    }

    pub fn with_commands(mut self, commands: NodeGraphTuningCommands) -> Self {
        self.commands = Some(commands);
        self
    }

    fn compute_layout(&self, bounds: Rect) -> TuningLayout {
        let margin = self.style.minimap_margin.max(10.0);
        let pad = self.style.context_menu_padding.max(8.0);
        let gap = 6.0;
        let row_h = self.style.context_menu_item_height.max(22.0);
        let btn = self.style.controls_button_size.max(22.0);

        let panel_w = 340.0;
        let mut rows = 17.0;
        if self.commands.is_some() {
            rows += 4.0;
        }
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

        let add_row3 = |hits: &mut Vec<(TuningHit, Rect)>,
                        cy: f32,
                        left: f32,
                        right: f32,
                        row_h: f32,
                        btn: f32,
                        toggle: TuningHit,
                        dec: TuningHit,
                        inc: TuningHit| {
            let y = cy + 0.5 * (row_h - btn).max(0.0);
            let inc_rect = Rect::new(
                Point::new(Px(right - btn), Px(y)),
                Size::new(Px(btn), Px(btn)),
            );
            let dec_rect = Rect::new(
                Point::new(Px(right - 2.0 * btn - 6.0), Px(y)),
                Size::new(Px(btn), Px(btn)),
            );
            let toggle_rect = Rect::new(
                Point::new(Px(right - 3.0 * btn - 12.0), Px(y)),
                Size::new(Px(btn), Px(btn)),
            );
            let label_rect = Rect::new(
                Point::new(Px(left), Px(cy)),
                Size::new(Px((right - left - 3.0 * btn - 18.0).max(0.0)), Px(row_h)),
            );
            hits.push((toggle, toggle_rect));
            hits.push((dec, dec_rect));
            hits.push((inc, inc_rect));
            (label_rect, toggle_rect, dec_rect, inc_rect)
        };

        // Row 0: mode
        let mode_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::ToggleMode, mode_btn));
        cy += row_h + gap;

        // Row 1: wheel pan_on_scroll
        let pan_on_scroll_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::TogglePanOnScroll, pan_on_scroll_btn));
        cy += row_h + gap;

        // Row 2: pan inertia
        let pan_inertia_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::TogglePanInertia, pan_inertia_btn));
        cy += row_h + gap;

        // Row 3: wheel zoom_on_scroll
        let zoom_on_scroll_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::ToggleZoomOnScroll, zoom_on_scroll_btn));
        cy += row_h + gap;

        // Row 4: wheel zoom_activation_key
        let zoom_key_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::CycleZoomActivationKey, zoom_key_btn));
        cy += row_h + gap;

        // Row 4: connection_radius
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

        // Row 5: edge_interaction_width
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

        // Row 6: connection_drag_threshold
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

        // Row 7: connect_on_click
        let connect_on_click_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::ToggleConnectOnClick, connect_on_click_btn));
        cy += row_h + gap;

        // Row 8: node_drag_threshold
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

        // Row 9: node_drag_handle_mode
        let drag_handle_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::ToggleNodeDragHandle, drag_handle_btn));
        cy += row_h + gap;

        // Row 10: node_click_distance
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

        // Row 11: translate_extent
        let _ = add_row3(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::ToggleTranslateExtent,
            TuningHit::TranslateExtentDec,
            TuningHit::TranslateExtentInc,
        );
        cy += row_h + gap;

        // Row 12: node_extent
        let _ = add_row3(
            &mut hits,
            cy,
            left,
            right,
            row_h,
            btn,
            TuningHit::ToggleNodeExtent,
            TuningHit::NodeExtentDec,
            TuningHit::NodeExtentInc,
        );
        cy += row_h + gap;

        // Row 13: auto_pan.margin
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

        // Row 14: auto_pan.speed
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
        cy += row_h + gap;

        // Row 15: edges_reconnectable
        let edges_reconnectable_btn = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        hits.push((TuningHit::ToggleEdgesReconnectable, edges_reconnectable_btn));
        cy += row_h + gap;

        if self.commands.is_some() {
            let reset_btn = Rect::new(
                Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
                Size::new(Px(btn), Px(btn)),
            );
            hits.push((TuningHit::ResetGraph, reset_btn));
            cy += row_h + gap;

            let stress_1k_btn = Rect::new(
                Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
                Size::new(Px(btn), Px(btn)),
            );
            hits.push((TuningHit::SpawnStress1k, stress_1k_btn));
            cy += row_h + gap;

            let stress_5k_btn = Rect::new(
                Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
                Size::new(Px(btn), Px(btn)),
            );
            hits.push((TuningHit::SpawnStress5k, stress_5k_btn));
            cy += row_h + gap;

            let stress_10k_btn = Rect::new(
                Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
                Size::new(Px(btn), Px(btn)),
            );
            hits.push((TuningHit::SpawnStress10k, stress_10k_btn));
        }

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
        fn default_translate_extent() -> fret_node::core::CanvasRect {
            fret_node::core::CanvasRect {
                origin: fret_node::core::CanvasPoint {
                    x: -1200.0,
                    y: -900.0,
                },
                size: fret_node::core::CanvasSize {
                    width: 2400.0,
                    height: 1800.0,
                },
            }
        }

        fn default_node_extent() -> fret_node::core::CanvasRect {
            fret_node::core::CanvasRect {
                origin: fret_node::core::CanvasPoint {
                    x: -600.0,
                    y: -450.0,
                },
                size: fret_node::core::CanvasSize {
                    width: 2400.0,
                    height: 1800.0,
                },
            }
        }

        fn resize_extent(
            rect: fret_node::core::CanvasRect,
            delta: f32,
        ) -> fret_node::core::CanvasRect {
            let w = rect.size.width.max(0.0);
            let h = rect.size.height.max(0.0);
            let cx = rect.origin.x + 0.5 * w;
            let cy = rect.origin.y + 0.5 * h;

            let next_w = (w + 2.0 * delta).max(200.0);
            let next_h = (h + 2.0 * delta).max(200.0);

            fret_node::core::CanvasRect {
                origin: fret_node::core::CanvasPoint {
                    x: cx - 0.5 * next_w,
                    y: cy - 0.5 * next_h,
                },
                size: fret_node::core::CanvasSize {
                    width: next_w,
                    height: next_h,
                },
            }
        }

        let apply = |s: &mut NodeGraphViewState| match hit {
            TuningHit::ToggleMode => {
                s.interaction.connection_mode = match s.interaction.connection_mode {
                    NodeGraphConnectionMode::Strict => NodeGraphConnectionMode::Loose,
                    NodeGraphConnectionMode::Loose => NodeGraphConnectionMode::Strict,
                };
            }
            TuningHit::TogglePanOnScroll => {
                s.interaction.pan_on_scroll = !s.interaction.pan_on_scroll;
            }
            TuningHit::TogglePanInertia => {
                s.interaction.pan_inertia.enabled = !s.interaction.pan_inertia.enabled;
            }
            TuningHit::ToggleZoomOnScroll => {
                s.interaction.zoom_on_scroll = !s.interaction.zoom_on_scroll;
            }
            TuningHit::CycleZoomActivationKey => {
                s.interaction.zoom_activation_key = match s.interaction.zoom_activation_key {
                    NodeGraphZoomActivationKey::CtrlOrMeta => NodeGraphZoomActivationKey::None,
                    NodeGraphZoomActivationKey::None => NodeGraphZoomActivationKey::Shift,
                    NodeGraphZoomActivationKey::Shift => NodeGraphZoomActivationKey::Alt,
                    NodeGraphZoomActivationKey::Alt => NodeGraphZoomActivationKey::CtrlOrMeta,
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
            TuningHit::ToggleConnectOnClick => {
                s.interaction.connect_on_click = !s.interaction.connect_on_click;
            }
            TuningHit::NodeDragThresholdDec => {
                s.interaction.node_drag_threshold =
                    (s.interaction.node_drag_threshold - 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::NodeDragThresholdInc => {
                s.interaction.node_drag_threshold =
                    (s.interaction.node_drag_threshold + 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::ToggleNodeDragHandle => {
                s.interaction.node_drag_handle_mode = match s.interaction.node_drag_handle_mode {
                    NodeGraphDragHandleMode::Any => NodeGraphDragHandleMode::Header,
                    NodeGraphDragHandleMode::Header => NodeGraphDragHandleMode::Any,
                };
            }
            TuningHit::NodeClickDistanceDec => {
                s.interaction.node_click_distance =
                    (s.interaction.node_click_distance - 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::NodeClickDistanceInc => {
                s.interaction.node_click_distance =
                    (s.interaction.node_click_distance + 0.5 * step_scale).clamp(0.0, 24.0);
            }
            TuningHit::ToggleTranslateExtent => {
                s.interaction.translate_extent = if s.interaction.translate_extent.is_some() {
                    None
                } else {
                    Some(default_translate_extent())
                };
            }
            TuningHit::TranslateExtentDec => {
                let delta = 200.0 * step_scale;
                let cur = s
                    .interaction
                    .translate_extent
                    .unwrap_or_else(default_translate_extent);
                s.interaction.translate_extent = Some(resize_extent(cur, -delta));
            }
            TuningHit::TranslateExtentInc => {
                let delta = 200.0 * step_scale;
                let cur = s
                    .interaction
                    .translate_extent
                    .unwrap_or_else(default_translate_extent);
                s.interaction.translate_extent = Some(resize_extent(cur, delta));
            }
            TuningHit::ToggleNodeExtent => {
                s.interaction.node_extent = if s.interaction.node_extent.is_some() {
                    None
                } else {
                    Some(default_node_extent())
                };
            }
            TuningHit::NodeExtentDec => {
                let delta = 200.0 * step_scale;
                let cur = s
                    .interaction
                    .node_extent
                    .unwrap_or_else(default_node_extent);
                s.interaction.node_extent = Some(resize_extent(cur, -delta));
            }
            TuningHit::NodeExtentInc => {
                let delta = 200.0 * step_scale;
                let cur = s
                    .interaction
                    .node_extent
                    .unwrap_or_else(default_node_extent);
                s.interaction.node_extent = Some(resize_extent(cur, delta));
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
            TuningHit::ToggleEdgesReconnectable => {
                s.interaction.edges_reconnectable = !s.interaction.edges_reconnectable;
            }
            TuningHit::ResetGraph
            | TuningHit::SpawnStress1k
            | TuningHit::SpawnStress5k
            | TuningHit::SpawnStress10k => {}
        };

        if let Some(store) = self.store.as_ref() {
            let _ = store.update(host, |store, _cx| {
                store.update_view_state(apply);
            });
        } else {
            let _ = self.view_state.update(host, |s, _cx| apply(s));
        }
    }

    fn command_for_hit(&self, hit: TuningHit) -> Option<CommandId> {
        let cmds = self.commands.as_ref()?;
        match hit {
            TuningHit::ResetGraph => Some(cmds.reset_graph.clone()),
            TuningHit::SpawnStress1k => Some(cmds.spawn_stress_1k.clone()),
            TuningHit::SpawnStress5k => Some(cmds.spawn_stress_5k.clone()),
            TuningHit::SpawnStress10k => Some(cmds.spawn_stress_10k.clone()),
            _ => None,
        }
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
                .prepare_str(text, &self.style.context_menu_text_style, constraints);
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
                if let Some(cmd) = self.command_for_hit(pressed) {
                    cx.dispatch_command(cmd);
                } else {
                    self.apply_hit(cx.app, pressed, step);
                }
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
            background: fret_core::Paint::Solid(self.style.context_menu_background),

            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(self.style.context_menu_border),

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
            background: fret_core::Paint::Solid(mode_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

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
                background: fret_core::Paint::Solid(bg(dec)),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(Px(corner.max(4.0))),
            });
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(22_020),
                rect: inc_rect,
                background: fret_core::Paint::Solid(bg(inc)),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

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

        let wheel_pan_text = if state.interaction.pan_on_scroll {
            "On"
        } else {
            "Off"
        };
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Pan on scroll", wheel_pan_text),
            self.style.context_menu_text,
        );
        let wheel_pan_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let wheel_pan_bg = if self.hovered == Some(TuningHit::TogglePanOnScroll) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: wheel_pan_btn_rect,
            background: fret_core::Paint::Solid(wheel_pan_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(wheel_pan_btn_rect.origin.x.0 + 4.0),
                Px(wheel_pan_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            wheel_pan_text,
            self.style.controls_text,
        );
        cy += row_h + gap;

        let pan_inertia_text = if state.interaction.pan_inertia.enabled {
            "On"
        } else {
            "Off"
        };
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Pan inertia", pan_inertia_text),
            self.style.context_menu_text,
        );
        let pan_inertia_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let pan_inertia_bg = if self.hovered == Some(TuningHit::TogglePanInertia) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: pan_inertia_btn_rect,
            background: fret_core::Paint::Solid(pan_inertia_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(pan_inertia_btn_rect.origin.x.0 + 4.0),
                Px(pan_inertia_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            pan_inertia_text,
            self.style.controls_text,
        );
        cy += row_h + gap;

        let wheel_zoom_text = if state.interaction.zoom_on_scroll {
            "On"
        } else {
            "Off"
        };
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Zoom on scroll", wheel_zoom_text),
            self.style.context_menu_text,
        );
        let wheel_zoom_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let wheel_zoom_bg = if self.hovered == Some(TuningHit::ToggleZoomOnScroll) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: wheel_zoom_btn_rect,
            background: fret_core::Paint::Solid(wheel_zoom_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(wheel_zoom_btn_rect.origin.x.0 + 4.0),
                Px(wheel_zoom_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            wheel_zoom_text,
            self.style.controls_text,
        );
        cy += row_h + gap;

        let zoom_key_str = match state.interaction.zoom_activation_key {
            NodeGraphZoomActivationKey::None => "none",
            NodeGraphZoomActivationKey::CtrlOrMeta => "ctrl",
            NodeGraphZoomActivationKey::Shift => "shift",
            NodeGraphZoomActivationKey::Alt => "alt",
        };
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Zoom activation", zoom_key_str),
            self.style.context_menu_text,
        );
        let zoom_key_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let zoom_key_bg = if self.hovered == Some(TuningHit::CycleZoomActivationKey) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: zoom_key_btn_rect,
            background: fret_core::Paint::Solid(zoom_key_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        let zoom_key_short = match state.interaction.zoom_activation_key {
            NodeGraphZoomActivationKey::None => "N",
            NodeGraphZoomActivationKey::CtrlOrMeta => "C",
            NodeGraphZoomActivationKey::Shift => "S",
            NodeGraphZoomActivationKey::Alt => "A",
        };
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(zoom_key_btn_rect.origin.x.0 + 0.5 * (btn - 8.0)),
                Px(zoom_key_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            zoom_key_short,
            self.style.controls_text,
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

        let connect_on_click_text = if state.interaction.connect_on_click {
            "On"
        } else {
            "Off"
        };
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Connect on click", connect_on_click_text),
            self.style.context_menu_text,
        );
        let connect_on_click_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let connect_on_click_bg = if self.hovered == Some(TuningHit::ToggleConnectOnClick) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: connect_on_click_btn_rect,
            background: fret_core::Paint::Solid(connect_on_click_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(connect_on_click_btn_rect.origin.x.0 + 4.0),
                Px(connect_on_click_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            connect_on_click_text,
            self.style.controls_text,
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

        let drag_handle_str = match state.interaction.node_drag_handle_mode {
            NodeGraphDragHandleMode::Any => "any",
            NodeGraphDragHandleMode::Header => "header",
        };
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Node drag handle", drag_handle_str),
            self.style.context_menu_text,
        );
        let drag_handle_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let drag_handle_bg = if self.hovered == Some(TuningHit::ToggleNodeDragHandle) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: drag_handle_btn_rect,
            background: fret_core::Paint::Solid(drag_handle_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        let drag_handle_short = match state.interaction.node_drag_handle_mode {
            NodeGraphDragHandleMode::Any => "A",
            NodeGraphDragHandleMode::Header => "H",
        };
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(drag_handle_btn_rect.origin.x.0 + 0.5 * (btn - 8.0)),
                Px(drag_handle_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            drag_handle_short,
            self.style.controls_text,
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

        let draw_extent_row = |this: &mut Self,
                               cx: &mut PaintCx<'_, H>,
                               y: f32,
                               label: &str,
                               enabled: bool,
                               value: String,
                               toggle: TuningHit,
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

            let toggle_rect = Rect::new(
                Point::new(
                    Px(right - 3.0 * btn - 12.0),
                    Px(y + 0.5 * (row_h - btn).max(0.0)),
                ),
                Size::new(Px(btn), Px(btn)),
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
                rect: toggle_rect,
                background: fret_core::Paint::Solid(bg(toggle)),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(Px(corner.max(4.0))),
            });
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(22_020),
                rect: dec_rect,
                background: fret_core::Paint::Solid(bg(dec)),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(Px(corner.max(4.0))),
            });
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(22_020),
                rect: inc_rect,
                background: fret_core::Paint::Solid(bg(inc)),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(Px(corner.max(4.0))),
            });

            let toggle_text = if enabled { "On" } else { "Off" };
            this.draw_text(
                cx,
                22_021,
                Point::new(
                    Px(toggle_rect.origin.x.0 + 4.0),
                    Px(toggle_rect.origin.y.0 + 0.5 * (btn - 12.0)),
                ),
                toggle_text,
                this.style.controls_text,
            );

            this.draw_text(
                cx,
                22_021,
                Point::new(
                    Px(dec_rect.origin.x.0 + 0.5 * (btn - 8.0)),
                    Px(dec_rect.origin.y.0 + 0.5 * (btn - 12.0)),
                ),
                "−",
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

        let translate_enabled = state.interaction.translate_extent.is_some();
        let translate_value = state
            .interaction
            .translate_extent
            .map(|r| format!("{:.0}×{:.0}", r.size.width.max(0.0), r.size.height.max(0.0)))
            .unwrap_or_else(|| "Off".to_string());
        draw_extent_row(
            self,
            cx,
            cy,
            "Translate extent",
            translate_enabled,
            translate_value,
            TuningHit::ToggleTranslateExtent,
            TuningHit::TranslateExtentDec,
            TuningHit::TranslateExtentInc,
        );
        cy += row_h + gap;

        let node_extent_enabled = state.interaction.node_extent.is_some();
        let node_extent_value = state
            .interaction
            .node_extent
            .map(|r| format!("{:.0}×{:.0}", r.size.width.max(0.0), r.size.height.max(0.0)))
            .unwrap_or_else(|| "Off".to_string());
        draw_extent_row(
            self,
            cx,
            cy,
            "Node extent",
            node_extent_enabled,
            node_extent_value,
            TuningHit::ToggleNodeExtent,
            TuningHit::NodeExtentDec,
            TuningHit::NodeExtentInc,
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
        cy += row_h + gap;

        let edges_reconnectable_text = if state.interaction.edges_reconnectable {
            "On"
        } else {
            "Off"
        };
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Edges reconnect", edges_reconnectable_text),
            self.style.context_menu_text,
        );
        let edges_reconnectable_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let edges_reconnectable_bg = if self.hovered == Some(TuningHit::ToggleEdgesReconnectable) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: edges_reconnectable_btn_rect,
            background: fret_core::Paint::Solid(edges_reconnectable_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(edges_reconnectable_btn_rect.origin.x.0 + 4.0),
                Px(edges_reconnectable_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            edges_reconnectable_text,
            self.style.controls_text,
        );

        if self.commands.is_none() {
            return;
        }
        cy += row_h + gap;

        // Extra rows for demo harness commands.
        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Graph", "reset"),
            self.style.context_menu_text,
        );
        let reset_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let reset_bg = if self.hovered == Some(TuningHit::ResetGraph) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: reset_btn_rect,
            background: fret_core::Paint::Solid(reset_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(reset_btn_rect.origin.x.0 + 4.0),
                Px(reset_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            "Go",
            self.style.controls_text,
        );
        cy += row_h + gap;

        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Stress graph", "1k nodes"),
            self.style.context_menu_text,
        );
        let stress_1k_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let stress_1k_bg = if self.hovered == Some(TuningHit::SpawnStress1k) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: stress_1k_btn_rect,
            background: fret_core::Paint::Solid(stress_1k_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(stress_1k_btn_rect.origin.x.0 + 4.0),
                Px(stress_1k_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            "Go",
            self.style.controls_text,
        );
        cy += row_h + gap;

        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Stress graph", "5k nodes"),
            self.style.context_menu_text,
        );
        let stress_5k_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let stress_5k_bg = if self.hovered == Some(TuningHit::SpawnStress5k) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: stress_5k_btn_rect,
            background: fret_core::Paint::Solid(stress_5k_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(stress_5k_btn_rect.origin.x.0 + 4.0),
                Px(stress_5k_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            "Go",
            self.style.controls_text,
        );
        cy += row_h + gap;

        self.draw_text(
            cx,
            22_010,
            Point::new(Px(left), Px(cy)),
            &row("Stress graph", "10k nodes"),
            self.style.context_menu_text,
        );
        let stress_10k_btn_rect = Rect::new(
            Point::new(Px(right - btn), Px(cy + 0.5 * (row_h - btn).max(0.0))),
            Size::new(Px(btn), Px(btn)),
        );
        let stress_10k_bg = if self.hovered == Some(TuningHit::SpawnStress10k) {
            self.style.controls_hover_background
        } else {
            Color::TRANSPARENT
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(22_020),
            rect: stress_10k_btn_rect,
            background: fret_core::Paint::Solid(stress_10k_bg),

            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(corner.max(4.0))),
        });
        self.draw_text(
            cx,
            22_021,
            Point::new(
                Px(stress_10k_btn_rect.origin.x.0 + 4.0),
                Px(stress_10k_btn_rect.origin.y.0 + 0.5 * (btn - 12.0)),
            ),
            "Go",
            self.style.controls_text,
        );
    }

    fn command(&mut self, _cx: &mut CommandCx<'_, H>, _command: &CommandId) -> bool {
        false
    }
}
