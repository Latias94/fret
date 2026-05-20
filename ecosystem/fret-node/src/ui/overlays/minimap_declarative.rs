use fret_canvas::view::{PanZoom2D, screen_rect_to_canvas_rect, visible_canvas_rect};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, Paint, Point, Px, Rect, SceneOp, SemanticsRole, Size,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CanvasProps, InsetEdge, Length, ManagedSurfaceProps, PositionStyle,
    SemanticsDecoration, SemanticsProps,
};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;
use crate::runtime::store::NodeGraphStore;
use crate::ui::NodeGraphStyle;

use super::minimap_drag_policy::plan_minimap_drag_pan;
use super::minimap_interaction_policy::{
    MiniMapKeyboardInteractionPlan, plan_minimap_keyboard_interaction,
    plan_minimap_pointer_down_interaction, plan_minimap_pointer_up_interaction,
};
use super::minimap_navigation_policy::{
    NodeGraphMiniMapBindings, apply_minimap_viewport_update,
    apply_minimap_viewport_update_action_host, normalize_minimap_navigation_zoom,
};
use super::minimap_projection::{minimap_world_bounds, project_world_rect_to_minimap};

#[derive(Debug, Clone)]
pub(super) struct NodeGraphMiniMapSnapshot {
    pub(super) canvas_bounds: Rect,
    pub(super) pan: CanvasPoint,
    pub(super) zoom: f32,
    pub(super) nodes_window: Vec<Rect>,
}

#[derive(Debug, Clone)]
pub(super) struct NodeGraphMiniMapOverlayElementProps {
    pub(super) style: NodeGraphStyle,
    pub(super) snapshot: NodeGraphMiniMapSnapshot,
    pub(super) view_state: Option<Model<NodeGraphViewState>>,
    pub(super) store: Option<Model<NodeGraphStore>>,
    pub(super) bindings: NodeGraphMiniMapBindings,
    pub(super) focus_target: Option<GlobalElementId>,
}

#[derive(Debug, Clone, Copy)]
struct DeclarativeMiniMapDragState {
    start_canvas: Point,
    start_pan: CanvasPoint,
}

pub(super) fn node_graph_minimap_overlay_element<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphMiniMapOverlayElementProps,
) -> AnyElement {
    let panel_size = minimap_panel_size(&props.style);
    let style = props.style;
    let snapshot = props.snapshot;
    let view_state = props.view_state;
    let store = props.store;
    let bindings = props.bindings;
    let focus_target = props.focus_target;
    let semantics_value = format!("zoom {:.3}", snapshot.zoom);
    let drag = cx.local_model(|| None::<DeclarativeMiniMapDragState>);
    let layout_style = style.clone();
    let layout_snapshot = snapshot.clone();
    let paint_style = style.clone();
    let paint_snapshot = snapshot.clone();
    let child_style = style.clone();
    let child_snapshot = snapshot.clone();
    let event_style = style.clone();
    let event_snapshot = snapshot.clone();
    let event_drag = drag.clone();
    let event_bindings = bindings.clone();
    let event_view_state = view_state.clone();
    let event_store = store.clone();
    let key_style = style.clone();
    let key_snapshot = snapshot.clone();
    let key_bindings = bindings.clone();
    let key_view_state = view_state.clone();
    let key_store = store.clone();

    let mut surface = ManagedSurfaceProps::default();
    surface.layout.position = PositionStyle::Absolute;
    surface.layout.inset.left = InsetEdge::Px(Px(0.0));
    surface.layout.inset.top = InsetEdge::Px(Px(0.0));
    surface.layout.size.width = Length::Fill;
    surface.layout.size.height = Length::Fill;

    let mut element = cx.managed_surface(
        surface,
        move |cx| {
            let minimap = minimap_rect(cx.bounds(), &layout_style, panel_size);
            let Some(child) = cx.children().first().copied() else {
                cx.set_hit_test_rects([minimap]);
                return;
            };
            cx.layout_child(child, minimap);
            cx.set_hit_test_rects([minimap]);
            cx.set_output(DeclarativeMiniMapFrame {
                minimap,
                snapshot: layout_snapshot.clone(),
            });
        },
        move |cx| {
            let paint_style = paint_style.clone();
            let paint_snapshot = paint_snapshot.clone();
            let children = cx.children().to_vec();
            let frame = cx
                .output::<DeclarativeMiniMapFrame>()
                .cloned()
                .unwrap_or_else(|| DeclarativeMiniMapFrame {
                    minimap: minimap_rect(cx.bounds(), &paint_style, panel_size),
                    snapshot: paint_snapshot.clone(),
                });
            if let Some(child) = children.first().copied() {
                cx.paint_child(child, frame.minimap);
            }
        },
        move |cx| {
            let canvas_style = child_style.clone();
            let canvas_snapshot = child_snapshot.clone();
            vec![cx.canvas(CanvasProps::default(), move |p| {
                for op in build_minimap_scene_ops(p.bounds(), &canvas_style, &canvas_snapshot) {
                    p.scene().push(op);
                }
            })]
        },
    );
    let minimap_element = element.id;
    cx.managed_surface_on_event_for(minimap_element, move |host, event| {
        handle_minimap_event(
            host,
            event,
            &event_style,
            &event_snapshot,
            event_view_state.as_ref(),
            event_store.as_ref(),
            &event_bindings,
            focus_target,
            &event_drag,
        );
    });

    element = element.attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Panel)
            .label("MiniMap")
            .value(semantics_value)
            .test_id("node_graph.minimap"),
    );
    let mut semantics = SemanticsProps::default();
    semantics.layout.size.width = Length::Fill;
    semantics.layout.size.height = Length::Fill;
    semantics.role = SemanticsRole::Panel;
    semantics.label = Some("MiniMap".into());
    semantics.value = Some(format!("zoom {:.3}", snapshot.zoom).into());
    semantics.test_id = Some("node_graph.minimap".into());
    semantics.focusable = true;
    cx.semantics_with_id(semantics, move |cx, minimap_root| {
        cx.key_on_key_down_for(
            minimap_root,
            std::sync::Arc::new(move |host, action_cx, down| {
                handle_minimap_key_down_action(
                    host,
                    action_cx,
                    down,
                    &key_style,
                    &key_snapshot,
                    key_view_state.as_ref(),
                    key_store.as_ref(),
                    &key_bindings,
                    focus_target,
                )
            }),
        );
        vec![element]
    })
}

#[derive(Debug, Clone)]
struct DeclarativeMiniMapFrame {
    minimap: Rect,
    snapshot: NodeGraphMiniMapSnapshot,
}

fn minimap_panel_size(style: &NodeGraphStyle) -> Size {
    Size::new(
        Px(style.paint.minimap_width.max(0.0)),
        Px(style.paint.minimap_height.max(0.0)),
    )
}

fn minimap_rect(bounds: Rect, style: &NodeGraphStyle, panel_size: Size) -> Rect {
    let w = panel_size.width.0.max(40.0);
    let h = panel_size.height.0.max(30.0);
    let margin = style.paint.minimap_margin.max(0.0);
    let x = bounds.origin.x.0 + (bounds.size.width.0 - margin - w).max(0.0);
    let y = bounds.origin.y.0 + (bounds.size.height.0 - margin - h).max(0.0);
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

fn minimap_world_rect(style: &NodeGraphStyle, snapshot: &NodeGraphMiniMapSnapshot) -> Rect {
    let viewport = minimap_viewport_rect(snapshot);
    let node_canvas_rects = minimap_node_canvas_rects(snapshot);
    minimap_world_bounds(
        node_canvas_rects.iter().copied(),
        viewport,
        style.paint.minimap_world_padding.max(0.0),
    )
}

fn minimap_viewport_rect(snapshot: &NodeGraphMiniMapSnapshot) -> Rect {
    let view = PanZoom2D {
        pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
        zoom: snapshot.zoom,
    };
    visible_canvas_rect(snapshot.canvas_bounds, view)
}

fn minimap_node_canvas_rects(snapshot: &NodeGraphMiniMapSnapshot) -> Vec<Rect> {
    let view = PanZoom2D {
        pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
        zoom: snapshot.zoom,
    };
    snapshot
        .nodes_window
        .iter()
        .copied()
        .map(|rect| screen_rect_to_canvas_rect(snapshot.canvas_bounds, view, rect))
        .collect()
}

fn build_minimap_scene_ops(
    minimap: Rect,
    style: &NodeGraphStyle,
    snapshot: &NodeGraphMiniMapSnapshot,
) -> Vec<SceneOp> {
    let viewport = minimap_viewport_rect(snapshot);
    let node_canvas_rects = minimap_node_canvas_rects(snapshot);
    let world = minimap_world_rect(style, snapshot);

    let mut ops = Vec::with_capacity(2 + node_canvas_rects.len());
    ops.push(SceneOp::Quad {
        order: DrawOrder(20_000),
        rect: minimap,
        background: Paint::Solid(style.paint.context_menu_background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(style.paint.context_menu_border).into(),
        corner_radii: Corners::all(Px(style.paint.context_menu_corner_radius)),
    });

    for rect in node_canvas_rects {
        ops.push(SceneOp::Quad {
            order: DrawOrder(20_001),
            rect: project_world_rect_to_minimap(minimap, world, rect),
            background: Paint::Solid(style.paint.node_background).into(),
            border: Edges::all(Px(0.5)),
            border_paint: Paint::Solid(style.paint.node_border).into(),
            corner_radii: Corners::all(Px(2.0)),
        });
    }

    ops.push(SceneOp::Quad {
        order: DrawOrder(20_002),
        rect: project_world_rect_to_minimap(minimap, world, viewport),
        background: Paint::Solid(Color {
            a: 0.12,
            ..style.paint.node_border_selected
        })
        .into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(style.paint.node_border_selected).into(),
        corner_radii: Corners::all(Px(2.0)),
    });

    ops
}

#[allow(clippy::too_many_arguments)]
fn handle_minimap_event<H: UiHost>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    event: &Event,
    style: &NodeGraphStyle,
    snapshot: &NodeGraphMiniMapSnapshot,
    view_state: Option<&Model<NodeGraphViewState>>,
    store: Option<&Model<NodeGraphStore>>,
    bindings: &NodeGraphMiniMapBindings,
    focus_target: Option<GlobalElementId>,
    drag: &Model<Option<DeclarativeMiniMapDragState>>,
) {
    match event {
        Event::KeyDown {
            key,
            modifiers: _,
            repeat: _,
        } => {
            let view_state_value = view_state
                .and_then(|view_state| view_state.read_ref(cx.app(), Clone::clone).ok())
                .unwrap_or_else(|| NodeGraphViewState {
                    pan: snapshot.pan,
                    zoom: snapshot.zoom,
                    ..Default::default()
                });
            match plan_minimap_keyboard_interaction(
                *key,
                &view_state_value,
                snapshot.canvas_bounds,
                style.geometry.min_zoom,
                style.geometry.max_zoom,
                24.0,
                1.1,
            ) {
                MiniMapKeyboardInteractionPlan::Viewport { pan, zoom } => {
                    if let Some(view_state) = view_state {
                        apply_minimap_viewport_update(
                            cx.app(),
                            &bindings.navigation,
                            view_state,
                            store,
                            pan,
                            zoom,
                        );
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.notify();
                    cx.stop_propagation();
                }
                MiniMapKeyboardInteractionPlan::FocusCanvas => {
                    if let Some(focus_target) = focus_target {
                        cx.request_focus_element(focus_target);
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.notify();
                    cx.stop_propagation();
                }
                MiniMapKeyboardInteractionPlan::Ignore => {}
            }
        }
        Event::Pointer(fret_core::PointerEvent::Down {
            position, button, ..
        }) => {
            let Some(minimap) = minimap_event_rect(cx.bounds(), style) else {
                return;
            };
            let world = minimap_world_rect(style, snapshot);
            let viewport = minimap_viewport_rect(snapshot);
            let current_pan = view_state
                .and_then(|view_state| view_state.read_ref(cx.app(), |s| s.pan).ok())
                .unwrap_or(snapshot.pan);
            let Some(plan) = plan_minimap_pointer_down_interaction(
                *button,
                minimap,
                world,
                viewport,
                *position,
                current_pan,
                snapshot.zoom,
                snapshot.canvas_bounds,
            ) else {
                return;
            };

            if plan.focus_canvas
                && let Some(focus_target) = focus_target
            {
                cx.request_focus_element(focus_target);
            }
            if plan.capture_pointer {
                cx.capture_pointer(cx.node());
            }
            if plan.stop_propagation {
                cx.stop_propagation();
            }
            if let Some(pan) = plan.drag.immediate_pan
                && let Some(view_state) = view_state
            {
                let zoom = view_state
                    .read_ref(cx.app(), |state| state.zoom)
                    .ok()
                    .map(normalize_minimap_navigation_zoom)
                    .unwrap_or(1.0);
                apply_minimap_viewport_update(
                    cx.app(),
                    &bindings.navigation,
                    view_state,
                    store,
                    pan,
                    zoom,
                );
            }
            let _ = drag.update(cx.app(), |state, _cx| {
                *state = Some(DeclarativeMiniMapDragState {
                    start_canvas: plan.drag.start_canvas,
                    start_pan: plan.drag.start_pan,
                });
            });
            if plan.repaint {
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.notify();
            }
        }
        Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
            let drag_state = drag.read_ref(cx.app(), |state| *state).ok().flatten();
            let Some(drag_state) = drag_state else {
                return;
            };
            let Some(minimap) = minimap_event_rect(cx.bounds(), style) else {
                return;
            };
            let world = minimap_world_rect(style, snapshot);
            let Some(pan) = plan_minimap_drag_pan(
                minimap,
                world,
                *position,
                drag_state.start_canvas,
                drag_state.start_pan,
            ) else {
                return;
            };
            if let Some(view_state) = view_state {
                let zoom = view_state
                    .read_ref(cx.app(), |state| state.zoom)
                    .ok()
                    .map(normalize_minimap_navigation_zoom)
                    .unwrap_or(1.0);
                apply_minimap_viewport_update(
                    cx.app(),
                    &bindings.navigation,
                    view_state,
                    store,
                    pan,
                    zoom,
                );
            }
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
        }
        Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
            let drag_active = drag
                .read_ref(cx.app(), Option::<DeclarativeMiniMapDragState>::is_some)
                .ok()
                .unwrap_or(false);
            if let Some(plan) = plan_minimap_pointer_up_interaction(*button, drag_active) {
                let _ = drag.update(cx.app(), |state, _cx| {
                    *state = None;
                });
                if plan.release_capture {
                    cx.release_pointer_capture();
                }
                if plan.finish_event {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.notify();
                    cx.stop_propagation();
                }
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_minimap_key_down_action(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: fret_ui::action::ActionCx,
    down: fret_ui::action::KeyDownCx,
    style: &NodeGraphStyle,
    snapshot: &NodeGraphMiniMapSnapshot,
    view_state: Option<&Model<NodeGraphViewState>>,
    store: Option<&Model<NodeGraphStore>>,
    bindings: &NodeGraphMiniMapBindings,
    focus_target: Option<GlobalElementId>,
) -> bool {
    if down.repeat || down.ime_composing {
        return false;
    }

    let view_state_value = view_state
        .and_then(|view_state| host.models_mut().read(view_state, Clone::clone).ok())
        .unwrap_or_else(|| NodeGraphViewState {
            pan: snapshot.pan,
            zoom: snapshot.zoom,
            ..Default::default()
        });

    match plan_minimap_keyboard_interaction(
        down.key,
        &view_state_value,
        snapshot.canvas_bounds,
        style.geometry.min_zoom,
        style.geometry.max_zoom,
        24.0,
        1.1,
    ) {
        MiniMapKeyboardInteractionPlan::Viewport { pan, zoom } => {
            if let Some(view_state) = view_state {
                apply_minimap_viewport_update_action_host(
                    host,
                    &bindings.navigation,
                    view_state,
                    store,
                    pan,
                    zoom,
                );
            }
            host.request_redraw(action_cx.window);
            host.notify(action_cx);
            true
        }
        MiniMapKeyboardInteractionPlan::FocusCanvas => {
            if let Some(focus_target) = focus_target {
                host.request_focus(focus_target);
            }
            host.request_redraw(action_cx.window);
            host.notify(action_cx);
            true
        }
        MiniMapKeyboardInteractionPlan::Ignore => false,
    }
}

fn minimap_event_rect(bounds: Rect, style: &NodeGraphStyle) -> Option<Rect> {
    let panel_size = minimap_panel_size(style);
    if panel_size.width.0 <= 0.0 || panel_size.height.0 <= 0.0 {
        return None;
    }
    Some(minimap_rect(bounds, style, panel_size))
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{
        AppWindowId, DrawOrder, KeyCode, Modifiers, MouseButton, MouseButtons, PathCommand,
        PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point, PointerEvent,
        PointerId, PointerType, Px, Rect, SceneOp, SemanticsRole, Size, SvgId, SvgService,
        TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, Effect,
        EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost, ModelId, ModelStore,
        ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::element::{ElementKind, Length, PointerRegionProps, StackProps};

    use crate::core::{CanvasPoint, Graph, GraphId};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::runtime::store::NodeGraphStore;
    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::minimap_declarative::{
        NodeGraphMiniMapOverlayElementProps, NodeGraphMiniMapSnapshot, build_minimap_scene_ops,
        minimap_panel_size, minimap_rect, node_graph_minimap_overlay_element,
    };

    #[derive(Default)]
    struct TestUiHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        effects: Vec<Effect>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestUiHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let existing = self.globals.remove(&type_id);
            let mut value = existing
                .and_then(|v| v.downcast::<T>().ok().map(|v| *v))
                .unwrap_or_else(init);
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestUiHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for TestUiHost {
        fn take_changed_models(&mut self) -> Vec<ModelId> {
            Vec::new()
        }
    }

    impl CommandsHost for TestUiHost {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for TestUiHost {
        fn request_redraw(&mut self, window: AppWindowId) {
            self.effects.push(Effect::Redraw(window));
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl TimeHost for TestUiHost {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_timer_token(&mut self) -> TimerToken {
            let out = TimerToken(self.next_timer_token);
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            out
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let out = ClipboardToken(self.next_clipboard_token);
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            out
        }

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            let out = ShareSheetToken(self.next_share_sheet_token);
            self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
            out
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            let out = ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            out
        }
    }

    impl DragHost for TestUiHost {
        fn drag(&self, _pointer_id: PointerId) -> Option<&DragSession> {
            None
        }

        fn drag_mut(&mut self, _pointer_id: PointerId) -> Option<&mut DragSession> {
            None
        }

        fn cancel_drag(&mut self, _pointer_id: PointerId) {}

        fn any_drag_session(&self, _predicate: impl FnMut(&DragSession) -> bool) -> bool {
            false
        }

        fn find_drag_pointer_id(
            &self,
            _predicate: impl FnMut(&DragSession) -> bool,
        ) -> Option<PointerId> {
            None
        }

        fn cancel_drag_sessions(
            &mut self,
            _predicate: impl FnMut(&DragSession) -> bool,
        ) -> Vec<PointerId> {
            Vec::new()
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }
    }

    #[derive(Default)]
    struct FakeUiServices;

    impl TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeUiServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    fn snapshot() -> NodeGraphMiniMapSnapshot {
        NodeGraphMiniMapSnapshot {
            canvas_bounds: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            pan: CanvasPoint { x: 0.0, y: 0.0 },
            zoom: 2.0,
            nodes_window: vec![Rect::new(
                Point::new(Px(20.0), Px(30.0)),
                Size::new(Px(80.0), Px(40.0)),
            )],
        }
    }

    fn render_minimap(
        style: NodeGraphStyle,
        snapshot: NodeGraphMiniMapSnapshot,
    ) -> fret_ui::element::AnyElement {
        let mut host = TestUiHost::default();
        let mut runtime = fret_ui::ElementRuntime::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );
        let mut cx = fret_ui::ElementContext::new_for_root_name(
            &mut host,
            &mut runtime,
            window,
            bounds,
            "root",
        );
        node_graph_minimap_overlay_element(
            &mut cx,
            NodeGraphMiniMapOverlayElementProps {
                style,
                snapshot,
                view_state: None,
                store: None,
                bindings: Default::default(),
                focus_target: None,
            },
        )
    }

    fn render_minimap_props(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        props: NodeGraphMiniMapOverlayElementProps,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            host,
            services,
            window,
            bounds,
            root_name,
            |cx| vec![node_graph_minimap_overlay_element(cx, props)],
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);
        root
    }

    fn render_minimap_with_surface(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        props: NodeGraphMiniMapOverlayElementProps,
        underlay_downs: fret_runtime::Model<u32>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            host,
            services,
            window,
            bounds,
            root_name,
            |cx| {
                let mut stack = StackProps::default();
                stack.layout.size.width = Length::Fill;
                stack.layout.size.height = Length::Fill;
                vec![cx.stack_props(stack, move |cx| {
                    let mut surface_props = PointerRegionProps::default();
                    surface_props.layout.size.width = Length::Fill;
                    surface_props.layout.size.height = Length::Fill;
                    let underlay_downs = underlay_downs.clone();
                    let surface_pointer = cx.pointer_region(surface_props, move |cx| {
                        cx.pointer_region_on_pointer_down(std::sync::Arc::new(
                            move |host, _action_cx, down| {
                                if down.button != MouseButton::Left {
                                    return false;
                                }
                                let _ = host.models_mut().update(&underlay_downs, |count| {
                                    *count = count.saturating_add(1);
                                });
                                true
                            },
                        ));
                        Vec::new()
                    });
                    let mut surface_props = fret_ui::element::SemanticsProps::default();
                    surface_props.layout.size.width = Length::Fill;
                    surface_props.layout.size.height = Length::Fill;
                    surface_props.role = SemanticsRole::Viewport;
                    surface_props.label = Some("Surface".into());
                    surface_props.test_id = Some("node_graph.surface".into());
                    surface_props.focusable = true;
                    let surface = cx.semantics(surface_props, move |_cx| vec![surface_pointer]);
                    let surface_target = surface.id;
                    let mut props = props;
                    props.focus_target = Some(surface_target);
                    let minimap = node_graph_minimap_overlay_element(cx, props);
                    vec![surface, minimap]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);
        root
    }

    fn test_style() -> NodeGraphStyle {
        let mut style = NodeGraphStyle::default();
        style.paint.minimap_width = 200.0;
        style.paint.minimap_height = 120.0;
        style.paint.minimap_margin = 10.0;
        style.paint.minimap_world_padding = 0.0;
        style
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        )
    }

    fn interaction_snapshot() -> NodeGraphMiniMapSnapshot {
        NodeGraphMiniMapSnapshot {
            canvas_bounds: bounds(),
            pan: CanvasPoint { x: 0.0, y: 0.0 },
            zoom: 1.0,
            nodes_window: Vec::new(),
        }
    }

    fn minimap_props(
        style: NodeGraphStyle,
        snapshot: NodeGraphMiniMapSnapshot,
        view_state: Option<fret_runtime::Model<NodeGraphViewState>>,
        store: Option<fret_runtime::Model<NodeGraphStore>>,
    ) -> NodeGraphMiniMapOverlayElementProps {
        NodeGraphMiniMapOverlayElementProps {
            style,
            snapshot,
            view_state,
            store,
            bindings: Default::default(),
            focus_target: None,
        }
    }

    fn insert_view_state(host: &mut TestUiHost) -> fret_runtime::Model<NodeGraphViewState> {
        host.models.insert(NodeGraphViewState::default())
    }

    fn insert_store(host: &mut TestUiHost) -> fret_runtime::Model<NodeGraphStore> {
        host.models.insert(NodeGraphStore::new(
            Graph::new(GraphId::new()),
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        ))
    }

    #[test]
    fn minimap_declarative_composition_builds_canvas_panel_without_retained_widget() {
        let mut style = NodeGraphStyle::default();
        style.paint.minimap_width = 180.0;
        style.paint.minimap_height = 110.0;
        let expected = minimap_panel_size(&style);

        let root = render_minimap(style, snapshot());

        let ElementKind::Semantics(semantics) = &root.kind else {
            panic!("minimap root should be a focusable declarative semantics node");
        };
        assert_eq!(semantics.layout.size.width, Length::Fill);
        assert_eq!(semantics.layout.size.height, Length::Fill);
        assert_eq!(semantics.role, SemanticsRole::Panel);
        assert_eq!(semantics.label.as_deref(), Some("MiniMap"));
        assert_eq!(semantics.value.as_deref(), Some("zoom 2.000"));
        assert_eq!(semantics.test_id.as_deref(), Some("node_graph.minimap"));
        assert!(semantics.focusable);

        assert_eq!(root.children.len(), 1);
        let ElementKind::ManagedSurface(surface) = &root.children[0].kind else {
            panic!("minimap should use a declarative managed surface host");
        };
        assert_eq!(surface.layout.size.width, Length::Fill);
        assert_eq!(surface.layout.size.height, Length::Fill);
        let surface_semantics = root.children[0]
            .semantics_decoration
            .as_ref()
            .expect("managed surface semantics");
        assert_eq!(surface_semantics.role, Some(SemanticsRole::Panel));
        assert_eq!(surface_semantics.label.as_deref(), Some("MiniMap"));
        assert_eq!(
            surface_semantics.test_id.as_deref(),
            Some("node_graph.minimap")
        );

        assert_eq!(root.children[0].children.len(), 1);
        let ElementKind::Canvas(canvas) = &root.children[0].children[0].kind else {
            panic!("minimap managed surface should contain a declarative canvas");
        };
        assert_eq!(canvas.layout.size.width, Length::Fill);
        assert_eq!(canvas.layout.size.height, Length::Fill);
        assert_eq!(expected.width.0, 180.0);
        assert_eq!(expected.height.0, 110.0);
    }

    #[test]
    fn minimap_declarative_paint_plan_emits_panel_nodes_and_viewport_markers() {
        let style = NodeGraphStyle::default();
        let minimap = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(160.0), Px(100.0)),
        );

        let ops = build_minimap_scene_ops(minimap, &style, &snapshot());

        assert_eq!(ops.len(), 3);
        let SceneOp::Quad {
            order: panel_order,
            rect: panel,
            ..
        } = ops[0]
        else {
            panic!("first minimap op should paint the panel");
        };
        assert_eq!(panel_order, DrawOrder(20_000));
        assert_eq!(panel, minimap);

        let SceneOp::Quad {
            order: node_order,
            rect: node_marker,
            ..
        } = ops[1]
        else {
            panic!("second minimap op should paint a node marker");
        };
        assert_eq!(node_order, DrawOrder(20_001));
        assert!(node_marker.size.width.0 >= 1.0);
        assert!(node_marker.size.height.0 >= 1.0);

        let SceneOp::Quad {
            order: viewport_order,
            rect: viewport_marker,
            ..
        } = ops[2]
        else {
            panic!("last minimap op should paint the viewport marker");
        };
        assert_eq!(viewport_order, DrawOrder(20_002));
        assert!(viewport_marker.size.width.0 > node_marker.size.width.0);
        assert!(viewport_marker.size.height.0 > node_marker.size.height.0);
    }

    #[test]
    fn minimap_declarative_pointer_events_fall_through_outside_rect_to_surface() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let style = test_style();
        let underlay_downs = host.models.insert(0_u32);
        let props = minimap_props(style.clone(), interaction_snapshot(), None, None);
        let root = render_minimap_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds(),
            "minimap-pointer-fallthrough",
            props,
            underlay_downs.clone(),
        );

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let surface_bounds = ui
            .debug_node_bounds(surface)
            .expect("surface should be laid out");
        let minimap = minimap_rect(bounds(), &style, minimap_panel_size(&style));
        let position = Point::new(
            Px(surface_bounds.origin.x.0 + 10.0),
            Px((minimap.origin.y.0 - 10.0).max(surface_bounds.origin.y.0 + 1.0)),
        );

        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, position);

        assert_eq!(
            underlay_downs
                .read_ref(&host, |count| *count)
                .expect("underlay counter"),
            1,
            "pointer-down outside the declarative minimap hit rect should fall through to the surface"
        );
    }

    #[test]
    fn minimap_declarative_drag_updates_view_state_and_store_without_surface_leakage() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let style = test_style();
        let minimap = minimap_rect(bounds(), &style, minimap_panel_size(&style));
        let view_state = insert_view_state(&mut host);
        let store = insert_store(&mut host);
        let underlay_downs = host.models.insert(0_u32);
        let props = minimap_props(
            style,
            interaction_snapshot(),
            Some(view_state.clone()),
            Some(store.clone()),
        );
        let root = render_minimap_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds(),
            "minimap-drag-updates-view-state-and-store",
            props,
            underlay_downs.clone(),
        );
        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let minimap_root = ui.children(stack)[1];
        let minimap_surface = ui.children(minimap_root)[0];

        ui.set_focus(Some(minimap_root));

        let start = Point::new(
            Px(minimap.origin.x.0 + 0.5 * minimap.size.width.0),
            Px(minimap.origin.y.0 + 0.5 * minimap.size.height.0),
        );
        let moved = Point::new(Px(start.x.0 + 10.0), start.y);

        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, start);
        assert_eq!(
            ui.focus(),
            Some(surface),
            "minimap pointer down should restore focus to the node graph surface target"
        );
        assert_eq!(
            ui.captured(),
            Some(minimap_surface),
            "minimap pointer down should capture pointer on the declarative managed surface"
        );
        assert_eq!(
            underlay_downs
                .read_ref(&host, |count| *count)
                .expect("underlay counter"),
            0,
            "minimap pointer down must not leak to the surface"
        );

        dispatch_pointer_move_at(&mut ui, &mut host, &mut services, moved);
        dispatch_pointer_up_at(&mut ui, &mut host, &mut services, moved);
        assert_eq!(
            ui.captured(),
            None,
            "minimap pointer up should release pointer capture"
        );

        let expected_pan_x = -50.0;
        let pan = view_state
            .read_ref(&host, |state| state.pan)
            .expect("view state pan");
        assert!(
            (pan.x - expected_pan_x).abs() <= 1.0e-4,
            "{pan:?} != {expected_pan_x}"
        );
        let store_pan = store
            .read_ref(&host, |store| store.view_state().pan)
            .expect("store view state pan");
        assert!(
            (store_pan.x - expected_pan_x).abs() <= 1.0e-4,
            "{store_pan:?} != {expected_pan_x}"
        );
    }

    #[test]
    fn minimap_declarative_keyboard_pan_zoom_and_escape_match_retained_oracle() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let style = test_style();
        let view_state = insert_view_state(&mut host);
        let store = insert_store(&mut host);
        let underlay_downs = host.models.insert(0_u32);
        let props = minimap_props(
            style,
            interaction_snapshot(),
            Some(view_state.clone()),
            Some(store.clone()),
        );
        let root = render_minimap_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds(),
            "minimap-keyboard-navigation",
            props,
            underlay_downs,
        );
        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let minimap_root = ui.children(stack)[1];
        ui.set_focus(Some(minimap_root));

        dispatch_key_down(&mut ui, &mut host, &mut services, KeyCode::ArrowRight);
        let pan = view_state
            .read_ref(&host, |state| state.pan)
            .expect("view state pan");
        assert!((pan.x + 24.0).abs() <= 1.0e-4, "{pan:?}");
        let store_pan = store
            .read_ref(&host, |store| store.view_state().pan)
            .expect("store view state pan");
        assert!((store_pan.x + 24.0).abs() <= 1.0e-4, "{store_pan:?}");

        dispatch_key_down(&mut ui, &mut host, &mut services, KeyCode::NumpadAdd);
        let zoom = view_state
            .read_ref(&host, |state| state.zoom)
            .expect("view state zoom");
        assert!((zoom - 1.1).abs() <= 1.0e-6, "{zoom}");
        let store_zoom = store
            .read_ref(&host, |store| store.view_state().zoom)
            .expect("store view state zoom");
        assert!((store_zoom - 1.1).abs() <= 1.0e-6, "{store_zoom}");

        dispatch_key_down(&mut ui, &mut host, &mut services, KeyCode::Escape);
        assert_eq!(
            ui.focus(),
            Some(surface),
            "Escape from declarative minimap should return focus to the node graph surface target"
        );
        assert!(
            !host
                .effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { .. })),
            "Escape should not dispatch commands: {:?}",
            host.effects
        );
    }

    fn dispatch_key_down(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        key: KeyCode,
    ) {
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::KeyDown {
                key,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    }

    fn dispatch_pointer_down_at(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        position: Point,
    ) {
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn dispatch_pointer_move_at(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        position: Point,
    ) {
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                position,
                buttons: MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn dispatch_pointer_up_at(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        position: Point,
    ) {
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
    }
}
