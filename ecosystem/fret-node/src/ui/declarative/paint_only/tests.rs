use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_canvas::view::PanZoom2D;
use fret_core::scene::DashPatternV1;
use fret_core::{
    AppWindowId, Color, MaterialDescriptor, MaterialId, MaterialRegistrationError, Modifiers,
    MouseButton, MouseButtons, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, PointerId, PointerType, Px, Rect, SemanticsRole, Size, SvgId, SvgService,
    TextConstraints, TextMetrics, TextService,
};
use fret_runtime::{
    ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, Effect,
    EffectSink, FrameId, GlobalsHost, ImageUploadToken, Model, ModelHost, ModelId, ModelStore,
    ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
};
use fret_ui::action::UiActionHost;
use fret_ui::element::{
    ContainerProps, LayoutStyle, Length, PointerRegionProps, PressableProps, SizeStyle,
};

use super::hover_anchor::{HoverTooltipAnchorSource, hovered_canvas_anchor_rect_for_surface};

use super::overlay_elements::{
    build_hover_tooltip_overlay_spec, clamp_marquee_overlay_rect_to_bounds,
};
use super::overlays::push_overlay_layer_if_needed;
use super::pointer_down::read_left_pointer_down_snapshot_action_host;
use super::surface_support::collect_node_label_and_ports;
use super::{
    AuthoritativeSurfaceBoundarySnapshot, DeclarativeDiagKeyAction, DeclarativeDiagViewPreset,
    DeclarativeKeyboardZoomAction, DerivedGeometryCacheState, DragState, GridPaintCacheState,
    HoverAnchorStore, Invalidation, KeyHandlerParams, LeftPointerDownOutcome,
    LeftPointerDownSnapshot, LeftPointerReleaseOutcome, MarqueeDragState,
    MarqueePointerMoveOutcome, NodeDragPhase, NodeDragPointerMoveOutcome, NodeDragReleaseOutcome,
    NodeDragState, NodeGraphDeclarativeEdgeLabelRenderer, NodeGraphDeclarativeInteractionContext,
    NodeGraphDeclarativeInteractionHook, NodeGraphDeclarativeInteractionOutcome,
    NodeGraphDeclarativePortalRenderer, NodeGraphDiagnosticsConfig, NodeGraphEdgeLabelHitTestMode,
    NodeGraphEdgeLabelLayout, NodeGraphVisibleSubsetPortalConfig, NodeRectDraw,
    PaintOnlyInteractionFrameInputs, PendingSelectionState, PointerDownHandlerParams,
    PortalBoundsStore, PortalDebugFlags, PortalMeasuredGeometryState, ReconnectDragState,
    apply_declarative_diag_view_preset_action_host, authoritative_surface_boundary_snapshot,
    begin_left_pointer_down_action_host, begin_pan_pointer_down_action_host,
    build_click_selection_preview_edges, build_click_selection_preview_nodes,
    build_diag_normalize_visible_node_transaction, build_diag_nudge_visible_node_transaction,
    build_edge_spatial_rect_overrides, build_edges_draws_paint_only,
    build_key_down_capture_handler, build_marquee_preview_selected_nodes,
    build_node_drag_transaction, build_pointer_down_handler, collect_edge_update_anchor_infos,
    collect_portal_label_infos_for_visible_subset, commit_edge_click_selection_action_host,
    commit_graph_transaction, commit_marquee_selection_action_host, commit_node_drag_transaction,
    commit_pending_selection_action_host, complete_left_pointer_release_action_host,
    complete_node_drag_release_action_host, derived_geometry_cache_key,
    edge_reconnect_endpoint_enabled, edge_stroke_width_mul_for_selection, edges_cache_key,
    effective_selected_nodes_for_paint, escape_cancel_declarative_interactions_action_host,
    flush_portal_measured_geometry_state, grid_cache_key, handle_declarative_diag_key_action_host,
    handle_declarative_keyboard_zoom_action_host, handle_declarative_pointer_cancel_action_host,
    handle_declarative_pointer_up_action_host, handle_marquee_left_pointer_release_action_host,
    handle_marquee_pointer_move_action_host, handle_node_drag_left_pointer_release_action_host,
    handle_node_drag_pointer_move_action_host,
    handle_pending_selection_left_pointer_release_action_host, hit_test_edge_at_canvas_point,
    hit_test_edge_update_anchor_at_window_point, node_drag_commit_delta, nodes_cache_key,
    plan_paint_only_interaction_frame, pointer_cancel_declarative_interactions_action_host,
    pointer_crossed_threshold, read_authoritative_view_state_in_models,
    record_portal_measured_node_size_in_state, resolve_hover_tooltip_anchor, stable_hash_u64,
    sync_authoritative_surface_boundary_in_models, sync_hover_anchor_store_in_models,
    sync_portal_canvas_bounds_in_models, update_hovered_node_pointer_move_action_host,
    update_view_state_action_host, view_from_state,
};
use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, EdgeReconnectable,
    EdgeReconnectableEndpoint, Graph, GraphId, Group, GroupId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ops::{EdgeEndpoints, GraphOp, GraphTransaction};
use crate::rules::EdgeEndpoint;
use crate::runtime::callbacks::{
    ConnectDragKind, ConnectEnd, ConnectEndOutcome, ConnectStart, NodeGraphCommitCallbacks,
    NodeGraphGestureCallbacks, NodeGraphViewCallbacks, SelectionChange, install_callbacks,
};
use crate::runtime::changes::NodeGraphPatch;
use crate::runtime::store::NodeGraphStore;
use crate::ui::internals::NodeGraphInternalsSnapshot;
use crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX;
use crate::ui::paint_overrides::{NodeGraphPaintOverrides, NodeGraphPaintOverridesMap};
use crate::ui::{
    EdgeChromeHint, EdgeCustomPath, EdgeRouteKind, EdgeTypeKey, MeasuredGeometryStore,
    NodeGraphController, NodeGraphDeclarativePortalCommandHandler, NodeGraphEdgeTypes,
    NodeGraphNodeTypes, NodeGraphSkin, NodeGraphSurfaceBinding, PortalCommandOutcome,
    PortalNumberEditHandler, PortalNumberEditSpec, PortalNumberEditSubmit, PortalTextCommand,
    PortalTextEditHandler, PortalTextEditSpec, PortalTextEditSubmit, PortalTextStepMode,
    node_graph_surface, node_graph_surface_with_edge_label_renderer,
    node_graph_surface_with_portal_renderer, portal_cancel_text_command,
    portal_step_text_command_with_mode, portal_submit_text_command,
};
use serde_json::Value;

#[derive(Default)]
struct TestActionHostImpl {
    models: ModelStore,
    effects: Vec<Effect>,
    next_timer_token: u64,
    next_clipboard_token: u64,
    next_share_sheet_token: u64,
    redraw_requests: Vec<AppWindowId>,
    notifications: Vec<fret_ui::action::ActionCx>,
    invalidations: Vec<Invalidation>,
    capture_pointer_count: usize,
    release_pointer_capture_count: usize,
    requested_focus: Vec<fret_ui::GlobalElementId>,
    cursor_icons: Vec<fret_core::CursorIcon>,
    prevented_defaults: Vec<fret_runtime::DefaultAction>,
    bounds: Rect,
    globals: HashMap<TypeId, Box<dyn Any>>,
    commands: CommandRegistry,
    tick_id: TickId,
    frame_id: FrameId,
    next_image_upload_token: u64,
}

impl GlobalsHost for TestActionHostImpl {
    fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        let type_id = TypeId::of::<T>();
        let existing = self.globals.remove(&type_id);
        let mut value = existing
            .and_then(|value| value.downcast::<T>().ok().map(|value| *value))
            .unwrap_or_else(init);
        let out = f(&mut value, self);
        self.globals.insert(type_id, Box::new(value));
        out
    }
}

impl ModelHost for TestActionHostImpl {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}

impl ModelsHost for TestActionHostImpl {
    fn take_changed_models(&mut self) -> Vec<ModelId> {
        self.models.take_changed_models()
    }
}

impl CommandsHost for TestActionHostImpl {
    fn commands(&self) -> &CommandRegistry {
        &self.commands
    }
}

impl EffectSink for TestActionHostImpl {
    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw_requests.push(window);
    }

    fn push_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

impl TimeHost for TestActionHostImpl {
    fn tick_id(&self) -> TickId {
        self.tick_id
    }

    fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    fn next_timer_token(&mut self) -> TimerToken {
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        TimerToken(self.next_timer_token)
    }

    fn next_clipboard_token(&mut self) -> ClipboardToken {
        self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
        ClipboardToken(self.next_clipboard_token)
    }

    fn next_share_sheet_token(&mut self) -> ShareSheetToken {
        self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
        ShareSheetToken(self.next_share_sheet_token)
    }

    fn next_image_upload_token(&mut self) -> ImageUploadToken {
        self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
        ImageUploadToken(self.next_image_upload_token)
    }
}

impl DragHost for TestActionHostImpl {
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
        _desc: MaterialDescriptor,
    ) -> Result<MaterialId, MaterialRegistrationError> {
        Err(MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: MaterialId) -> bool {
        false
    }
}

impl UiActionHost for TestActionHostImpl {
    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }

    fn push_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw_requests.push(window);
    }

    fn notify(&mut self, cx: fret_ui::action::ActionCx) {
        self.notifications.push(cx);
    }

    fn next_timer_token(&mut self) -> TimerToken {
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        TimerToken(self.next_timer_token)
    }

    fn next_clipboard_token(&mut self) -> ClipboardToken {
        self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
        ClipboardToken(self.next_clipboard_token)
    }

    fn next_share_sheet_token(&mut self) -> ShareSheetToken {
        self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
        ShareSheetToken(self.next_share_sheet_token)
    }

    fn record_pending_action_payload(
        &mut self,
        _cx: fret_ui::action::ActionCx,
        _action: &fret_runtime::ActionId,
        _payload: Box<dyn Any + Send + Sync>,
    ) {
    }
}

impl fret_ui::action::UiFocusActionHost for TestActionHostImpl {
    fn request_focus(&mut self, target: fret_ui::GlobalElementId) {
        self.requested_focus.push(target);
    }
}

impl fret_ui::action::UiDragActionHost for TestActionHostImpl {
    fn begin_drag_with_kind(
        &mut self,
        _pointer_id: PointerId,
        _kind: DragKindId,
        _source_window: AppWindowId,
        _start: Point,
    ) {
    }

    fn begin_cross_window_drag_with_kind(
        &mut self,
        _pointer_id: PointerId,
        _kind: DragKindId,
        _source_window: AppWindowId,
        _start: Point,
    ) {
    }

    fn drag(&self, _pointer_id: PointerId) -> Option<&DragSession> {
        None
    }

    fn drag_mut(&mut self, _pointer_id: PointerId) -> Option<&mut DragSession> {
        None
    }

    fn cancel_drag(&mut self, _pointer_id: PointerId) {}
}

impl fret_ui::action::UiPointerActionHost for TestActionHostImpl {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn capture_pointer(&mut self) {
        self.capture_pointer_count = self.capture_pointer_count.saturating_add(1);
    }

    fn release_pointer_capture(&mut self) {
        self.release_pointer_capture_count = self.release_pointer_capture_count.saturating_add(1);
    }

    fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
        self.cursor_icons.push(icon);
    }

    fn prevent_default(&mut self, action: fret_runtime::DefaultAction) {
        self.prevented_defaults.push(action);
    }

    fn invalidate(&mut self, invalidation: Invalidation) {
        self.invalidations.push(invalidation);
    }
}

fn test_pointer_move(
    position: Point,
    buttons: MouseButtons,
    modifiers: Modifiers,
) -> fret_ui::action::PointerMoveCx {
    fret_ui::action::PointerMoveCx {
        pointer_id: PointerId::default(),
        position,
        position_local: position,
        position_window: Some(position),
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        velocity_window: None,
        buttons,
        modifiers,
        pointer_type: PointerType::Mouse,
    }
}

fn test_pointer_down(
    button: MouseButton,
    position: Point,
    modifiers: Modifiers,
) -> fret_ui::action::PointerDownCx {
    fret_ui::action::PointerDownCx {
        pointer_id: PointerId::default(),
        position,
        position_local: position,
        position_window: Some(position),
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        button,
        modifiers,
        click_count: 1,
        pointer_type: PointerType::Mouse,
        hit_is_text_input: false,
        hit_is_pressable: false,
        hit_pressable_target: None,
        hit_pressable_target_in_descendant_subtree: false,
    }
}

fn test_action_cx() -> fret_ui::action::ActionCx {
    fret_ui::action::ActionCx {
        window: AppWindowId::default(),
        target: fret_ui::GlobalElementId(1),
    }
}

fn test_pointer_up(
    button: MouseButton,
    position: Point,
    modifiers: Modifiers,
) -> fret_ui::action::PointerUpCx {
    fret_ui::action::PointerUpCx {
        pointer_id: PointerId::default(),
        position,
        position_local: position,
        position_window: Some(position),
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        velocity_window: None,
        button,
        modifiers,
        is_click: false,
        click_count: 1,
        pointer_type: PointerType::Mouse,
        down_hit_pressable_target: None,
        down_hit_pressable_target_in_descendant_subtree: false,
    }
}

fn test_pointer_cancel() -> fret_ui::action::PointerCancelCx {
    fret_ui::action::PointerCancelCx {
        pointer_id: PointerId::default(),
        position: None,
        position_local: None,
        position_window: None,
        tick_id: TickId(0),
        pixels_per_point: 1.0,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_type: PointerType::Mouse,
        reason: fret_core::PointerCancelReason::LeftWindow,
    }
}

fn test_node(pos: CanvasPoint) -> Node {
    Node {
        kind: NodeKindKey::new("test.node"),
        kind_version: 1,
        pos,
        selectable: None,
        draggable: None,
        connectable: None,
        deletable: None,
        parent: None,
        extent: None,
        expand_parent: None,
        size: None,
        hidden: false,
        collapsed: false,
        ports: Vec::new(),
        data: Value::Null,
    }
}

fn test_marquee_geometry() -> (Graph, crate::ui::canvas::CanvasGeometry, NodeId, NodeId) {
    let mut graph = Graph::new(GraphId::from_u128(91));
    let node_a = NodeId::from_u128(9101);
    let node_b = NodeId::from_u128(9102);
    let mut node_a_value = test_node(CanvasPoint { x: 0.0, y: 0.0 });
    node_a_value.size = Some(CanvasSize {
        width: 100.0,
        height: 60.0,
    });
    let mut node_b_value = test_node(CanvasPoint { x: 140.0, y: 0.0 });
    node_b_value.size = Some(CanvasSize {
        width: 100.0,
        height: 60.0,
    });
    graph.nodes.insert(node_a, node_a_value);
    graph.nodes.insert(node_b, node_b_value);

    let draw_order = vec![node_a, node_b];
    let style = crate::ui::style::NodeGraphStyle::default();
    let mut presenter = crate::ui::presenter::DefaultNodeGraphPresenter::default();
    let geom = crate::ui::canvas::CanvasGeometry::build_with_presenter(
        &graph,
        &draw_order,
        &style,
        1.0,
        crate::io::NodeGraphNodeOrigin::default(),
        &mut presenter,
        None,
    );
    (graph, geom, node_a, node_b)
}

#[test]
fn build_node_drag_transaction_uses_set_node_pos_ops() {
    let mut graph = Graph::new(GraphId::from_u128(1));
    let node_a = NodeId::from_u128(11);
    let node_b = NodeId::from_u128(22);
    let missing = NodeId::from_u128(33);
    graph
        .nodes
        .insert(node_a, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
    graph
        .nodes
        .insert(node_b, test_node(CanvasPoint { x: -5.0, y: 7.5 }));

    let tx = build_node_drag_transaction(&graph, &[node_a, missing, node_b], 12.0, -4.5);

    assert_eq!(tx.label.as_deref(), Some("Move Nodes"));
    assert_eq!(tx.ops.len(), 2);
    assert!(matches!(
        tx.ops[0],
        GraphOp::SetNodePos {
            id,
            from: CanvasPoint { x: 10.0, y: 20.0 },
            to: CanvasPoint { x: 22.0, y: 15.5 },
        } if id == node_a
    ));
    assert!(matches!(
        tx.ops[1],
        GraphOp::SetNodePos {
            id,
            from: CanvasPoint { x: -5.0, y: 7.5 },
            to: CanvasPoint { x: 7.0, y: 3.0 },
        } if id == node_b
    ));
}

#[test]
fn build_node_drag_transaction_returns_empty_for_noops() {
    let mut graph = Graph::new(GraphId::from_u128(2));
    let node = NodeId::from_u128(44);
    graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 3.0, y: 9.0 }));

    let tx = build_node_drag_transaction(&graph, &[node], 0.0, 0.0);

    assert!(tx.is_empty());
    assert_eq!(tx.label, None);
}

#[test]
fn build_diag_nudge_visible_node_transaction_uses_set_node_pos() {
    let mut graph = Graph::new(GraphId::from_u128(3));
    let hidden = NodeId::from_u128(55);
    let visible = NodeId::from_u128(66);
    let mut hidden_node = test_node(CanvasPoint { x: 1.0, y: 2.0 });
    hidden_node.hidden = true;
    graph.nodes.insert(hidden, hidden_node);
    graph
        .nodes
        .insert(visible, test_node(CanvasPoint { x: 10.0, y: 20.0 }));

    let tx = build_diag_nudge_visible_node_transaction(&graph);

    assert_eq!(tx.label.as_deref(), Some("Diag Nudge Visible Node"));
    assert_eq!(tx.ops.len(), 1);
    assert!(matches!(
        tx.ops[0],
        GraphOp::SetNodePos {
            id,
            from: CanvasPoint { x: 10.0, y: 20.0 },
            to: CanvasPoint { x: 11.0, y: 20.0 },
        } if id == visible
    ));
}

#[test]
fn build_diag_normalize_visible_node_transaction_hides_other_nodes() {
    let mut graph = Graph::new(GraphId::from_u128(4));
    let first = NodeId::from_u128(77);
    let other = NodeId::from_u128(88);
    graph
        .nodes
        .insert(first, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
    graph
        .nodes
        .insert(other, test_node(CanvasPoint { x: -5.0, y: 7.5 }));

    let tx = build_diag_normalize_visible_node_transaction(&graph);

    assert_eq!(tx.label.as_deref(), Some("Diag Normalize Visible Node"));
    assert!(tx.ops.iter().any(|op| matches!(
        op,
        GraphOp::SetNodePos {
            id,
            from: CanvasPoint { x: 10.0, y: 20.0 },
            to: CanvasPoint { x: 0.0, y: 0.0 },
        } if *id == first
    )));
    assert!(tx.ops.iter().any(|op| matches!(
        op,
        GraphOp::SetNodeSize {
            id,
            from,
            to: Some(CanvasSize {
                width: 220.0,
                height: 140.0,
            }),
        } if *id == first && from.is_none()
    )));
    assert!(tx.ops.iter().any(|op| matches!(
        op,
        GraphOp::SetNodeHidden {
            id,
            from: false,
            to: true,
        } if *id == other
    )));
}

#[test]
fn commit_graph_transaction_syncs_graph_and_view_models_through_binding() {
    let mut host = TestActionHostImpl::default();
    let mut graph_value = Graph::new(GraphId::from_u128(5));
    let node = NodeId::from_u128(99);
    graph_value
        .nodes
        .insert(node, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
    let graph = host.models.insert(graph_value.clone());
    let view_state = host.models.insert(NodeGraphViewState::default());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        NodeGraphViewState::default(),
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store.clone());
    let binding = test_binding(&mut host, &graph, &view_state, &controller);

    let tx = host
        .models
        .read(&graph, |graph| {
            build_node_drag_transaction(graph, &[node], 5.0, -2.0)
        })
        .expect("build transaction");

    assert!(commit_graph_transaction(&mut host, &binding, &tx));

    let graph_pos = host
        .models
        .read(&graph, |graph| graph.nodes.get(&node).map(|node| node.pos))
        .ok()
        .flatten()
        .expect("graph node pos");
    let store_pos = host
        .models
        .read(&store, |store| {
            store.graph().nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("store node pos");
    let synced_zoom = host
        .models
        .read(&view_state, |state| state.zoom)
        .expect("view-state model readable");

    assert_eq!(graph_pos, CanvasPoint { x: 15.0, y: 18.0 });
    assert_eq!(store_pos, CanvasPoint { x: 15.0, y: 18.0 });
    assert_eq!(synced_zoom, 1.0);
}

#[test]
fn commit_node_drag_transaction_notifies_store_callbacks_through_binding() {
    #[derive(Clone)]
    struct Recorder {
        commits: Rc<RefCell<Vec<(Option<String>, usize)>>>,
    }

    impl NodeGraphCommitCallbacks for Recorder {
        fn on_graph_commit(&mut self, patch: &NodeGraphPatch) {
            let changes = patch.node_edge_changes();
            self.commits
                .borrow_mut()
                .push((patch.transaction.label.clone(), changes.nodes.len()));
        }
    }

    impl NodeGraphViewCallbacks for Recorder {}

    impl NodeGraphGestureCallbacks for Recorder {}

    let mut host = TestActionHostImpl::default();
    let mut graph_value = Graph::new(GraphId::from_u128(6));
    let node = NodeId::from_u128(199);
    graph_value
        .nodes
        .insert(node, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
    let graph = host.models.insert(graph_value.clone());
    let view_state = host.models.insert(NodeGraphViewState::default());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        NodeGraphViewState::default(),
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store.clone());
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let commits: Rc<RefCell<Vec<(Option<String>, usize)>>> = Rc::new(RefCell::new(Vec::new()));
    let _callbacks_token = host
        .models
        .update(&store, |store| {
            install_callbacks(
                store,
                Recorder {
                    commits: commits.clone(),
                },
            )
        })
        .expect("install callbacks");

    let tx = host
        .models
        .read(&graph, |graph| {
            build_node_drag_transaction(graph, &[node], 5.0, -2.0)
        })
        .expect("build transaction");

    assert!(commit_node_drag_transaction(&mut host, &binding, &tx));

    let callback_commits = commits.borrow();
    assert_eq!(callback_commits.len(), 1);
    assert_eq!(callback_commits[0].0.as_deref(), Some("Move Node"));
    assert_eq!(callback_commits[0].1, 1);
}

#[test]
fn declarative_node_drag_commit_supports_undo_and_redo_through_binding() {
    let node_a = NodeId::from_u128(601);
    let node_b = NodeId::from_u128(602);
    let mut fixture = DeclarativeControllerFixture::new_two_nodes(
        601,
        node_a,
        CanvasPoint { x: 10.0, y: 20.0 },
        node_b,
        CanvasPoint { x: 40.0, y: 20.0 },
        NodeGraphViewState::default(),
    );

    let tx = fixture
        .host
        .models
        .read(&fixture.graph, |graph| {
            build_node_drag_transaction(graph, &[node_a], 5.0, -2.0)
        })
        .expect("build transaction");

    assert!(commit_node_drag_transaction(
        &mut fixture.host,
        &fixture.binding,
        &tx,
    ));

    let committed_pos = fixture
        .host
        .models
        .read(&fixture.graph, |graph| {
            graph.nodes.get(&node_a).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("graph node pos after commit");
    assert_eq!(committed_pos, CanvasPoint { x: 15.0, y: 18.0 });

    let undo = fixture
        .binding
        .undo_action_host(&mut fixture.host)
        .unwrap()
        .expect("did undo");
    assert!(!undo.patch.ops().is_empty());

    let undone_pos = fixture
        .host
        .models
        .read(&fixture.graph, |graph| {
            graph.nodes.get(&node_a).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("graph node pos after undo");
    let store_flags = fixture
        .host
        .models
        .read(&fixture.store, |store| (store.can_undo(), store.can_redo()))
        .ok()
        .expect("history flags after undo");
    assert_eq!(undone_pos, CanvasPoint { x: 10.0, y: 20.0 });
    assert_eq!(store_flags, (false, true));

    let redo = fixture
        .binding
        .redo_action_host(&mut fixture.host)
        .unwrap()
        .expect("did redo");
    assert!(!redo.patch.ops().is_empty());

    let redone_pos = fixture
        .host
        .models
        .read(&fixture.graph, |graph| {
            graph.nodes.get(&node_a).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("graph node pos after redo");
    let store_flags = fixture
        .host
        .models
        .read(&fixture.store, |store| (store.can_undo(), store.can_redo()))
        .ok()
        .expect("history flags after redo");
    assert_eq!(redone_pos, CanvasPoint { x: 15.0, y: 18.0 });
    assert_eq!(store_flags, (true, false));
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct DeclarativeCallbackTrace {
    commit_labels: Vec<Option<String>>,
    selection_changes: Vec<SelectionChange>,
    reconnects: Vec<(EdgeId, EdgeEndpoints, EdgeEndpoints)>,
    edge_updates: Vec<(EdgeId, EdgeEndpoints, EdgeEndpoints)>,
    connect_starts: Vec<ConnectStart>,
    connect_ends: Vec<ConnectEnd>,
    reconnect_starts: Vec<ConnectStart>,
    reconnect_ends: Vec<ConnectEnd>,
    edge_update_starts: Vec<ConnectStart>,
    edge_update_ends: Vec<ConnectEnd>,
}

#[derive(Clone)]
struct DeclarativeCallbackRecorder {
    trace: Rc<RefCell<DeclarativeCallbackTrace>>,
}

impl NodeGraphCommitCallbacks for DeclarativeCallbackRecorder {
    fn on_graph_commit(&mut self, patch: &NodeGraphPatch) {
        self.trace
            .borrow_mut()
            .commit_labels
            .push(patch.transaction.label.clone());
    }

    fn on_reconnect(&mut self, edge: EdgeId, from: EdgeEndpoints, to: EdgeEndpoints) {
        self.trace.borrow_mut().reconnects.push((edge, from, to));
    }

    fn on_edge_update(&mut self, edge: EdgeId, from: EdgeEndpoints, to: EdgeEndpoints) {
        self.trace.borrow_mut().edge_updates.push((edge, from, to));
    }
}

impl NodeGraphViewCallbacks for DeclarativeCallbackRecorder {
    fn on_selection_change(&mut self, sel: SelectionChange) {
        self.trace.borrow_mut().selection_changes.push(sel);
    }
}

impl NodeGraphGestureCallbacks for DeclarativeCallbackRecorder {
    fn on_connect_start(&mut self, ev: ConnectStart) {
        self.trace.borrow_mut().connect_starts.push(ev);
    }

    fn on_connect_end(&mut self, ev: ConnectEnd) {
        self.trace.borrow_mut().connect_ends.push(ev);
    }

    fn on_reconnect_start(&mut self, ev: ConnectStart) {
        self.trace.borrow_mut().reconnect_starts.push(ev);
    }

    fn on_reconnect_end(&mut self, ev: ConnectEnd) {
        self.trace.borrow_mut().reconnect_ends.push(ev);
    }

    fn on_edge_update_start(&mut self, ev: ConnectStart) {
        self.trace.borrow_mut().edge_update_starts.push(ev);
    }

    fn on_edge_update_end(&mut self, ev: ConnectEnd) {
        self.trace.borrow_mut().edge_update_ends.push(ev);
    }
}

fn install_declarative_callback_trace(
    host: &mut TestActionHostImpl,
    store: &Model<NodeGraphStore>,
) -> Rc<RefCell<DeclarativeCallbackTrace>> {
    let trace: Rc<RefCell<DeclarativeCallbackTrace>> =
        Rc::new(RefCell::new(DeclarativeCallbackTrace::default()));
    let _callbacks_token = host
        .models
        .update(store, |store| {
            install_callbacks(
                store,
                DeclarativeCallbackRecorder {
                    trace: trace.clone(),
                },
            )
        })
        .expect("install callbacks");
    trace
}

struct DeclarativeControllerFixture {
    host: TestActionHostImpl,
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    store: Model<NodeGraphStore>,
    binding: NodeGraphSurfaceBinding,
    controller: NodeGraphController,
}

impl DeclarativeControllerFixture {
    fn new_two_nodes(
        graph_id: u128,
        node_a: NodeId,
        node_a_pos: CanvasPoint,
        node_b: NodeId,
        node_b_pos: CanvasPoint,
        initial_view: NodeGraphViewState,
    ) -> Self {
        let mut host = TestActionHostImpl::default();
        let mut graph_value = Graph::new(GraphId::from_u128(graph_id));
        graph_value.nodes.insert(node_a, test_node(node_a_pos));
        graph_value.nodes.insert(node_b, test_node(node_b_pos));
        let graph = host.models.insert(graph_value.clone());
        let view_state = host.models.insert(initial_view.clone());
        let editor_config = host.models.insert(default_editor_config());
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            initial_view,
            default_editor_config(),
        ));
        let controller = NodeGraphController::new(store.clone());
        let binding = NodeGraphSurfaceBinding::from_models_and_controller(
            graph.clone(),
            view_state.clone(),
            editor_config.clone(),
            controller.clone(),
        );
        Self {
            host,
            graph,
            view_state,
            store,
            binding,
            controller,
        }
    }

    fn install_trace(&mut self) -> Rc<RefCell<DeclarativeCallbackTrace>> {
        install_declarative_callback_trace(&mut self.host, &self.store)
    }
}

fn assert_single_selection_change(
    trace: &Rc<RefCell<DeclarativeCallbackTrace>>,
    expected_nodes: Vec<NodeId>,
) {
    let got = trace.borrow();
    assert!(got.commit_labels.is_empty());
    assert_eq!(
        got.selection_changes,
        vec![SelectionChange {
            nodes: expected_nodes,
            edges: Vec::new(),
            groups: Vec::new(),
        }]
    );
}

fn assert_pointer_session_finished(
    host: &TestActionHostImpl,
    action_cx: fret_ui::action::ActionCx,
) {
    assert_eq!(host.release_pointer_capture_count, 1);
    assert_eq!(host.invalidations, vec![Invalidation::Layout]);
    assert_eq!(host.notifications, vec![action_cx]);
    assert_eq!(host.redraw_requests, vec![action_cx.window]);
}

fn test_binding(
    host: &mut TestActionHostImpl,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: &NodeGraphController,
) -> NodeGraphSurfaceBinding {
    let editor_config = controller.store();
    let editor_config = host
        .models
        .read(&editor_config, |store| store.editor_config())
        .expect("binding test store readable");
    let editor_config = host.models.insert(editor_config);
    NodeGraphSurfaceBinding::from_models_and_controller(
        graph.clone(),
        view_state.clone(),
        editor_config,
        controller.clone(),
    )
}

fn test_editor_config(f: impl FnOnce(&mut NodeGraphEditorConfig)) -> NodeGraphEditorConfig {
    let mut editor_config = NodeGraphEditorConfig::default();
    f(&mut editor_config);
    editor_config
}

fn default_editor_config() -> NodeGraphEditorConfig {
    NodeGraphEditorConfig::default()
}

fn empty_test_binding(host: &mut TestActionHostImpl, graph_id: u128) -> NodeGraphSurfaceBinding {
    NodeGraphSurfaceBinding::new(
        &mut host.models,
        Graph::new(GraphId::from_u128(graph_id)),
        NodeGraphViewState::default(),
        default_editor_config(),
    )
}

fn test_node_graph_surface_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

fn make_port(
    node: NodeId,
    key: &str,
    dir: PortDirection,
    kind: PortKind,
    capacity: PortCapacity,
) -> Port {
    Port {
        node,
        key: PortKey::new(key),
        dir,
        kind,
        capacity,
        connectable: None,
        connectable_start: None,
        connectable_end: None,
        ty: None,
        data: Value::Null,
    }
}

fn make_graph_two_nodes_with_ports() -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
    let mut graph = Graph::new(GraphId::from_u128(0xA11));
    let a = NodeId::from_u128(0xA110);
    let a_in = PortId::from_u128(0xA111);
    let a_out = PortId::from_u128(0xA112);
    let b = NodeId::from_u128(0xA120);
    let b_in = PortId::from_u128(0xA121);

    let mut node_a = test_node(CanvasPoint { x: 10.0, y: 20.0 });
    node_a.ports = vec![a_in, a_out];
    let mut node_b = test_node(CanvasPoint { x: 160.0, y: 20.0 });
    node_b.ports = vec![b_in];
    graph.nodes.insert(a, node_a);
    graph.nodes.insert(b, node_b);
    graph.ports.insert(
        a_in,
        make_port(
            a,
            "in",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );
    graph.ports.insert(
        a_out,
        make_port(
            a,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        b_in,
        make_port(
            b,
            "in",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );

    (graph, a, a_in, a_out, b, b_in)
}

fn make_graph_two_nodes_with_edge() -> (Graph, Vec<NodeId>, EdgeId) {
    let (mut graph, a, _a_in, a_out, b, b_in) = make_graph_two_nodes_with_ports();
    let edge = EdgeId::from_u128(0xA130);
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    (graph, vec![a, b], edge)
}

fn build_test_canvas_geometry(
    graph: &Graph,
    draw_order: &[NodeId],
) -> crate::ui::canvas::CanvasGeometry {
    let style = crate::ui::style::NodeGraphStyle::default();
    let mut presenter = crate::ui::DefaultNodeGraphPresenter::default();
    crate::ui::canvas::CanvasGeometry::build_with_presenter(
        graph,
        draw_order,
        &style,
        1.0,
        crate::io::NodeGraphNodeOrigin::default(),
        &mut presenter,
        None,
    )
}

fn render_surface_semantics_snapshot(
    graph: Graph,
    view_state: NodeGraphViewState,
) -> fret_core::SemanticsSnapshot {
    render_surface_semantics_snapshot_with_props(graph, view_state, |binding| {
        binding.surface_props()
    })
}

fn render_surface_semantics_snapshot_with_props(
    graph: Graph,
    view_state: NodeGraphViewState,
    props_for_binding: impl FnOnce(&NodeGraphSurfaceBinding) -> super::NodeGraphSurfaceProps,
) -> fret_core::SemanticsSnapshot {
    render_surface_semantics_snapshot_with_editor_config_and_props(
        graph,
        view_state,
        default_editor_config(),
        props_for_binding,
    )
}

fn render_surface_semantics_snapshot_with_editor_config(
    graph: Graph,
    view_state: NodeGraphViewState,
    editor_config: NodeGraphEditorConfig,
) -> fret_core::SemanticsSnapshot {
    render_surface_semantics_snapshot_with_editor_config_and_props(
        graph,
        view_state,
        editor_config,
        |binding| binding.surface_props(),
    )
}

fn render_surface_semantics_snapshot_with_editor_config_and_props(
    graph: Graph,
    view_state: NodeGraphViewState,
    editor_config: NodeGraphEditorConfig,
    props_for_binding: impl FnOnce(&NodeGraphSurfaceBinding) -> super::NodeGraphSurfaceProps,
) -> fret_core::SemanticsSnapshot {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(&mut host.models, graph, view_state, editor_config);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "node-graph-surface-a11y",
        |cx| {
            let props = props_for_binding(&binding);
            vec![node_graph_surface(cx, props)]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut host, &mut services, bounds, 1.0);
    ui.semantics_snapshot()
        .cloned()
        .expect("semantics snapshot")
}

fn surface_snapshot_node_id(
    snapshot: &fret_core::SemanticsSnapshot,
    test_id: &str,
) -> fret_core::NodeId {
    snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| {
            let available = snapshot
                .nodes
                .iter()
                .filter_map(|node| node.test_id.as_deref())
                .collect::<Vec<_>>();
            panic!("missing semantics node with test id {test_id}; available={available:?}")
        })
        .id
}

fn maybe_surface_snapshot_node_id(
    snapshot: &fret_core::SemanticsSnapshot,
    test_id: &str,
) -> Option<fret_core::NodeId> {
    snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .map(|node| node.id)
}

fn render_surface_frame_for_binding(
    ui: &mut fret_ui::UiTree<TestActionHostImpl>,
    host: &mut TestActionHostImpl,
    services: &mut FakeUiServices,
    window: AppWindowId,
    bounds: Rect,
    binding: &NodeGraphSurfaceBinding,
    props_for_binding: impl FnOnce(&NodeGraphSurfaceBinding) -> super::NodeGraphSurfaceProps,
) -> fret_core::SemanticsSnapshot {
    let root = fret_ui::declarative::render_root(
        ui,
        host,
        services,
        window,
        bounds,
        "node-graph-surface-frame",
        |cx| {
            let props = props_for_binding(binding);
            vec![node_graph_surface(cx, props)]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(host, services, bounds, 1.0);
    let mut scene = fret_core::Scene::default();
    ui.paint_all(host, services, bounds, &mut scene, 1.0);
    host.frame_id = fret_runtime::FrameId(host.frame_id.0.saturating_add(1));
    ui.semantics_snapshot()
        .cloned()
        .expect("semantics snapshot")
}

fn render_default_surface_frame_until_test_id(
    ui: &mut fret_ui::UiTree<TestActionHostImpl>,
    host: &mut TestActionHostImpl,
    services: &mut FakeUiServices,
    window: AppWindowId,
    bounds: Rect,
    binding: &NodeGraphSurfaceBinding,
    test_id: &str,
) -> fret_core::SemanticsSnapshot {
    let mut last = None;
    for _ in 0..4 {
        let snapshot = render_surface_frame_for_binding(
            ui,
            host,
            services,
            window,
            bounds,
            binding,
            |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
        );
        if snapshot
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(test_id))
        {
            return snapshot;
        }
        last = Some(snapshot);
    }
    last.expect("at least one frame rendered")
}

fn canvas_semantics_value(snapshot: &fret_core::SemanticsSnapshot) -> &str {
    snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("node_graph.canvas"))
        .and_then(|node| node.value.as_deref())
        .expect("canvas semantics value")
}

fn render_surface_frame_with_portal_renderer_for_binding(
    ui: &mut fret_ui::UiTree<TestActionHostImpl>,
    host: &mut TestActionHostImpl,
    services: &mut FakeUiServices,
    window: AppWindowId,
    bounds: Rect,
    binding: &NodeGraphSurfaceBinding,
    props_for_binding: impl FnOnce(&NodeGraphSurfaceBinding) -> super::NodeGraphSurfaceProps,
    portal_renderer: &mut dyn NodeGraphDeclarativePortalRenderer<TestActionHostImpl>,
) -> fret_core::SemanticsSnapshot {
    let root = fret_ui::declarative::render_root(
        ui,
        host,
        services,
        window,
        bounds,
        "node-graph-surface-custom-portal-frame",
        |cx| {
            let props = props_for_binding(binding);
            vec![node_graph_surface_with_portal_renderer(
                cx,
                props,
                portal_renderer,
            )]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(host, services, bounds, 1.0);
    let mut scene = fret_core::Scene::default();
    ui.paint_all(host, services, bounds, &mut scene, 1.0);
    host.frame_id = fret_runtime::FrameId(host.frame_id.0.saturating_add(1));
    ui.semantics_snapshot()
        .cloned()
        .expect("semantics snapshot")
}

fn render_surface_frame_with_edge_label_renderer_for_binding(
    ui: &mut fret_ui::UiTree<TestActionHostImpl>,
    host: &mut TestActionHostImpl,
    services: &mut FakeUiServices,
    window: AppWindowId,
    bounds: Rect,
    binding: &NodeGraphSurfaceBinding,
    props_for_binding: impl FnOnce(&NodeGraphSurfaceBinding) -> super::NodeGraphSurfaceProps,
    edge_label_renderer: &mut dyn NodeGraphDeclarativeEdgeLabelRenderer<TestActionHostImpl>,
) -> fret_core::SemanticsSnapshot {
    let root = fret_ui::declarative::render_root(
        ui,
        host,
        services,
        window,
        bounds,
        "node-graph-surface-edge-label-renderer-frame",
        |cx| {
            let props = props_for_binding(binding);
            vec![node_graph_surface_with_edge_label_renderer(
                cx,
                props,
                edge_label_renderer,
            )]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(host, services, bounds, 1.0);
    let mut scene = fret_core::Scene::default();
    ui.paint_all(host, services, bounds, &mut scene, 1.0);
    host.frame_id = fret_runtime::FrameId(host.frame_id.0.saturating_add(1));
    ui.semantics_snapshot()
        .cloned()
        .expect("semantics snapshot")
}

fn render_until_surface_test_id_for_binding(
    ui: &mut fret_ui::UiTree<TestActionHostImpl>,
    host: &mut TestActionHostImpl,
    services: &mut FakeUiServices,
    window: AppWindowId,
    bounds: Rect,
    binding: &NodeGraphSurfaceBinding,
    test_id: &str,
    props_for_binding: impl Copy + FnOnce(&NodeGraphSurfaceBinding) -> super::NodeGraphSurfaceProps,
) -> (fret_core::SemanticsSnapshot, fret_core::NodeId) {
    let mut last = None;
    for _ in 0..4 {
        let snapshot = render_surface_frame_for_binding(
            ui,
            host,
            services,
            window,
            bounds,
            binding,
            props_for_binding,
        );
        if let Some(id) = maybe_surface_snapshot_node_id(&snapshot, test_id) {
            return (snapshot, id);
        }
        last = Some(snapshot);
    }

    let snapshot = last.expect("at least one frame rendered");
    let _ = surface_snapshot_node_id(&snapshot, test_id);
    unreachable!()
}

fn assert_canvas_active_descendant_label(snapshot: &fret_core::SemanticsSnapshot, expected: &str) {
    let canvas = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("node_graph.canvas"))
        .expect("node graph canvas semantics node");
    assert_eq!(canvas.role, SemanticsRole::Viewport);
    assert_eq!(canvas.label.as_deref(), Some("Node Graph Canvas"));
    let active = canvas
        .active_descendant
        .and_then(|id| snapshot.nodes.iter().find(|node| node.id == id))
        .expect("active descendant semantics node");
    assert_eq!(active.label.as_deref(), Some(expected));
}

fn assert_canvas_has_no_active_descendant(snapshot: &fret_core::SemanticsSnapshot) {
    let canvas = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("node_graph.canvas"))
        .expect("node graph canvas semantics node");
    assert_eq!(canvas.role, SemanticsRole::Viewport);
    assert_eq!(canvas.label.as_deref(), Some("Node Graph Canvas"));
    assert!(
        canvas.active_descendant.is_none(),
        "canvas should not expose an active descendant for missing graph items"
    );
}

#[test]
fn node_graph_surface_active_descendant_points_to_focused_port_semantics_node() {
    let (graph, a, _a_in, _a_out, _b, _b_in) = make_graph_two_nodes_with_ports();
    let snapshot = render_surface_semantics_snapshot(
        graph,
        NodeGraphViewState {
            selected_nodes: vec![a],
            ..Default::default()
        },
    );

    assert_canvas_active_descendant_label(&snapshot, "Port in");
}

#[test]
fn node_graph_surface_props_new_wires_default_active_descendant_internals() {
    let (graph, a, _a_in, _a_out, _b, _b_in) = make_graph_two_nodes_with_ports();
    let snapshot = render_surface_semantics_snapshot_with_props(
        graph,
        NodeGraphViewState {
            selected_nodes: vec![a],
            ..Default::default()
        },
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );

    assert_canvas_active_descendant_label(&snapshot, "Port in");
}

#[test]
fn node_graph_surface_disable_keyboard_a11y_suppresses_active_descendant() {
    let (graph, a, _a_in, _a_out, _b, _b_in) = make_graph_two_nodes_with_ports();
    let snapshot = render_surface_semantics_snapshot_with_editor_config(
        graph,
        NodeGraphViewState {
            selected_nodes: vec![a],
            ..Default::default()
        },
        test_editor_config(|config| {
            config.interaction.disable_keyboard_a11y = true;
        }),
    );

    assert_canvas_has_no_active_descendant(&snapshot);
}

#[test]
fn node_graph_surface_active_descendant_prefers_focused_port_over_edge_and_node() {
    let (mut graph, a, _a_in, a_out, _b, b_in) = make_graph_two_nodes_with_ports();
    let edge = EdgeId::from_u128(0xA130);
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    let snapshot = render_surface_semantics_snapshot(
        graph,
        NodeGraphViewState {
            selected_nodes: vec![a],
            selected_edges: vec![edge],
            ..Default::default()
        },
    );

    assert_canvas_active_descendant_label(&snapshot, "Port in");
}

#[test]
fn node_graph_surface_active_descendant_points_to_focused_node_without_ports() {
    let mut graph = Graph::new(GraphId::from_u128(0xA140));
    let node = NodeId::from_u128(0xA141);
    graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
    let snapshot = render_surface_semantics_snapshot(
        graph,
        NodeGraphViewState {
            selected_nodes: vec![node],
            ..Default::default()
        },
    );

    assert_canvas_active_descendant_label(&snapshot, "Node test.node");
}

#[test]
fn node_graph_surface_active_descendant_points_to_focused_edge_semantics_node() {
    let (mut graph, _a, _a_in, a_out, _b, b_in) = make_graph_two_nodes_with_ports();
    let edge = EdgeId::from_u128(0xA130);
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    let snapshot = render_surface_semantics_snapshot(
        graph,
        NodeGraphViewState {
            selected_edges: vec![edge],
            ..Default::default()
        },
    );

    assert_canvas_active_descendant_label(&snapshot, &format!("Edge {edge:?}"));
}

#[test]
fn node_graph_surface_semantics_reports_selected_edges_count() {
    let (mut graph, _a, _a_in, a_out, _b, b_in) = make_graph_two_nodes_with_ports();
    let edge = EdgeId::from_u128(0xA132);
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    let snapshot = render_surface_semantics_snapshot(
        graph,
        NodeGraphViewState {
            selected_edges: vec![edge],
            ..Default::default()
        },
    );
    let canvas = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("node_graph.canvas"))
        .expect("node graph canvas semantics node");

    assert!(
        canvas
            .value
            .as_deref()
            .is_some_and(|value| value.contains("selected_edges:1;")),
        "canvas semantics should expose selected edge count for diagnostics"
    );
    assert!(
        canvas
            .value
            .as_deref()
            .is_some_and(|value| value.contains("edge_update_anchors:2;")),
        "selected reconnectable edge should plan source and target update anchors"
    );
}

#[test]
fn node_graph_surface_active_descendant_ignores_missing_selected_node() {
    let graph = Graph::new(GraphId::from_u128(0xA150));
    let missing = NodeId::from_u128(0xA151);
    let snapshot = render_surface_semantics_snapshot(
        graph,
        NodeGraphViewState {
            selected_nodes: vec![missing],
            ..Default::default()
        },
    );

    assert_canvas_has_no_active_descendant(&snapshot);
}

#[test]
fn commit_pending_selection_action_host_notifies_selection_callbacks_through_binding() {
    let node_a = NodeId::from_u128(9801);
    let node_b = NodeId::from_u128(9802);
    let initial_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..Default::default()
    };
    let mut fixture = DeclarativeControllerFixture::new_two_nodes(
        7,
        node_a,
        CanvasPoint { x: 0.0, y: 0.0 },
        node_b,
        CanvasPoint { x: 40.0, y: 20.0 },
        initial_view,
    );
    let trace = fixture.install_trace();
    let pending = PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    };

    assert!(commit_pending_selection_action_host(
        &mut fixture.host,
        &fixture.binding,
        &pending,
    ));

    assert_single_selection_change(&trace, vec![node_b]);
}

#[test]
fn commit_marquee_selection_action_host_notifies_selection_callbacks_through_binding() {
    let node_a = NodeId::from_u128(9901);
    let node_b = NodeId::from_u128(9902);
    let edge = EdgeId::new();
    let group = GroupId::new();
    let initial_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        selected_edges: vec![edge],
        selected_groups: vec![group],
        ..Default::default()
    };
    let mut fixture = DeclarativeControllerFixture::new_two_nodes(
        8,
        node_a,
        CanvasPoint { x: 0.0, y: 0.0 },
        node_b,
        CanvasPoint { x: 60.0, y: 20.0 },
        initial_view,
    );
    let trace = fixture.install_trace();
    let marquee = MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        active: true,
        toggle: false,
        base_selected_nodes: Arc::from([node_a]),
        preview_selected_nodes: Arc::from([node_b]),
    };

    assert!(commit_marquee_selection_action_host(
        &mut fixture.host,
        &fixture.binding,
        &marquee,
    ));

    assert_single_selection_change(&trace, vec![node_b]);
}

#[test]
fn handle_declarative_pointer_up_action_host_left_release_finishes_pointer_session_when_handled() {
    let action_cx = test_action_cx();
    let node_a = NodeId::from_u128(9935);
    let node_b = NodeId::from_u128(9936);
    let initial_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..Default::default()
    };
    let mut fixture = DeclarativeControllerFixture::new_two_nodes(
        120,
        node_a,
        CanvasPoint { x: 0.0, y: 0.0 },
        node_b,
        CanvasPoint { x: 40.0, y: 20.0 },
        initial_view,
    );
    let drag = fixture.host.models.insert(None::<DragState>);
    let marquee = fixture.host.models.insert(None::<MarqueeDragState>);
    let node_drag = fixture.host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(12.0), Px(0.0)),
        phase: NodeDragPhase::Armed,
        nodes_sorted: Arc::from([node_b]),
    }));
    let pending = fixture.host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    }));
    let trace = fixture.install_trace();

    assert!(handle_declarative_pointer_up_action_host(
        &mut fixture.host,
        action_cx,
        test_pointer_up(
            MouseButton::Left,
            Point::new(Px(12.0), Px(0.0)),
            Modifiers::default(),
        ),
        MouseButton::Middle,
        &drag,
        &marquee,
        &node_drag,
        &pending,
        &fixture.binding,
    ));
    assert_pointer_session_finished(&fixture.host, action_cx);
    assert_single_selection_change(&trace, vec![node_b]);
}

#[test]
fn handle_declarative_pointer_up_action_host_ignores_non_left_non_pan_buttons() {
    let mut host = TestActionHostImpl::default();
    let action_cx = test_action_cx();
    let drag = host.models.insert(Some(DragState {
        button: MouseButton::Middle,
        last_pos: Point::new(Px(3.0), Px(4.0)),
    }));
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let node_drag = host.models.insert(None::<NodeDragState>);
    let pending = host.models.insert(None::<PendingSelectionState>);
    let graph = host.models.insert(Graph::new(GraphId::from_u128(121)));
    let view_value = NodeGraphViewState::default();
    let view_state = host.models.insert(view_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(121)),
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);

    assert!(!handle_declarative_pointer_up_action_host(
        &mut host,
        action_cx,
        test_pointer_up(
            MouseButton::Right,
            Point::new(Px(0.0), Px(0.0)),
            Modifiers::default(),
        ),
        MouseButton::Middle,
        &drag,
        &marquee,
        &node_drag,
        &pending,
        &binding,
    ));
    assert!(
        host.models
            .read(&drag, |state| state.is_some())
            .expect("drag readable")
    );
    assert_eq!(host.release_pointer_capture_count, 0);
    assert!(host.invalidations.is_empty());
    assert!(host.notifications.is_empty());
    assert!(host.redraw_requests.is_empty());
}

#[test]
fn handle_declarative_pointer_up_action_host_pan_release_clears_drag_and_finishes_session() {
    let mut host = TestActionHostImpl::default();
    let action_cx = test_action_cx();
    let drag = host.models.insert(Some(DragState {
        button: MouseButton::Middle,
        last_pos: Point::new(Px(3.0), Px(4.0)),
    }));
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let node_drag = host.models.insert(None::<NodeDragState>);
    let pending = host.models.insert(None::<PendingSelectionState>);
    let graph = host.models.insert(Graph::new(GraphId::from_u128(122)));
    let view_value = NodeGraphViewState::default();
    let view_state = host.models.insert(view_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(122)),
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);

    assert!(handle_declarative_pointer_up_action_host(
        &mut host,
        action_cx,
        test_pointer_up(
            MouseButton::Middle,
            Point::new(Px(0.0), Px(0.0)),
            Modifiers::default(),
        ),
        MouseButton::Middle,
        &drag,
        &marquee,
        &node_drag,
        &pending,
        &binding,
    ));
    assert!(
        host.models
            .read(&drag, |state| state.is_none())
            .expect("drag readable")
    );
    assert_pointer_session_finished(&host, action_cx);
}

#[test]
fn handle_declarative_pointer_cancel_action_host_finishes_session_even_without_transients() {
    let mut host = TestActionHostImpl::default();
    let action_cx = test_action_cx();
    let drag = host.models.insert(None::<DragState>);
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let node_drag = host.models.insert(None::<NodeDragState>);
    let pending = host.models.insert(None::<PendingSelectionState>);

    assert!(handle_declarative_pointer_cancel_action_host(
        &mut host,
        action_cx,
        test_pointer_cancel(),
        &drag,
        &marquee,
        &node_drag,
        &pending,
    ));
    assert_pointer_session_finished(&host, action_cx);
}

#[test]
fn complete_left_pointer_release_action_host_pending_selection_clears_transient_and_notifies_selection()
 {
    let node_a = NodeId::from_u128(9941);
    let node_b = NodeId::from_u128(9942);
    let initial_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..Default::default()
    };
    let mut fixture = DeclarativeControllerFixture::new_two_nodes(
        10,
        node_a,
        CanvasPoint { x: 0.0, y: 0.0 },
        node_b,
        CanvasPoint { x: 40.0, y: 20.0 },
        initial_view,
    );
    let node_drag = fixture.host.models.insert(None::<NodeDragState>);
    let marquee = fixture.host.models.insert(None::<MarqueeDragState>);
    let pending = fixture.host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    }));
    let trace = fixture.install_trace();

    let outcome = complete_left_pointer_release_action_host(
        &mut fixture.host,
        &node_drag,
        &pending,
        &marquee,
        &fixture.binding,
    );

    assert_eq!(
        outcome,
        LeftPointerReleaseOutcome::PendingSelection {
            selection_committed: true,
        }
    );
    assert!(
        fixture
            .host
            .models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
    assert_single_selection_change(&trace, vec![node_b]);
}

#[test]
fn complete_left_pointer_release_action_host_inactive_toggle_marquee_skips_selection_commit() {
    let node_a = NodeId::from_u128(9943);
    let node_b = NodeId::from_u128(9944);
    let initial_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..Default::default()
    };
    let mut fixture = DeclarativeControllerFixture::new_two_nodes(
        11,
        node_a,
        CanvasPoint { x: 0.0, y: 0.0 },
        node_b,
        CanvasPoint { x: 40.0, y: 20.0 },
        initial_view,
    );
    let node_drag = fixture.host.models.insert(None::<NodeDragState>);
    let marquee = fixture.host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(5.0), Px(5.0)),
        active: false,
        toggle: true,
        base_selected_nodes: Arc::from([node_a]),
        preview_selected_nodes: Arc::from([node_b]),
    }));
    let pending = fixture.host.models.insert(None::<PendingSelectionState>);
    let trace = fixture.install_trace();

    let outcome = complete_left_pointer_release_action_host(
        &mut fixture.host,
        &node_drag,
        &pending,
        &marquee,
        &fixture.binding,
    );

    assert_eq!(
        outcome,
        LeftPointerReleaseOutcome::Marquee {
            selection_committed: false,
        }
    );
    assert!(
        fixture
            .host
            .models
            .read(&marquee, |state| state.is_none())
            .expect("marquee readable")
    );
    let got = trace.borrow();
    assert!(got.commit_labels.is_empty());
    assert!(got.selection_changes.is_empty());
}

#[test]
fn complete_left_pointer_release_action_host_none_when_no_left_release_state_exists() {
    let mut host = TestActionHostImpl::default();
    let graph = host.models.insert(Graph::new(GraphId::from_u128(12)));
    let view_value = NodeGraphViewState::default();
    let view_state = host.models.insert(view_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(12)),
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let node_drag = host.models.insert(None::<NodeDragState>);
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let pending = host.models.insert(None::<PendingSelectionState>);

    let outcome = complete_left_pointer_release_action_host(
        &mut host, &node_drag, &pending, &marquee, &binding,
    );

    assert_eq!(outcome, LeftPointerReleaseOutcome::None);
}

#[test]
fn handle_node_drag_left_pointer_release_action_host_clears_drag_and_pending_selection() {
    let node_a = NodeId::from_u128(9945);
    let node_b = NodeId::from_u128(9946);
    let initial_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..Default::default()
    };
    let mut fixture = DeclarativeControllerFixture::new_two_nodes(
        13,
        node_a,
        CanvasPoint { x: 0.0, y: 0.0 },
        node_b,
        CanvasPoint { x: 40.0, y: 20.0 },
        initial_view,
    );
    let node_drag = fixture.host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(12.0), Px(0.0)),
        phase: NodeDragPhase::Armed,
        nodes_sorted: Arc::from([node_b]),
    }));
    let pending = fixture.host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    }));
    let trace = fixture.install_trace();

    let outcome = handle_node_drag_left_pointer_release_action_host(
        &mut fixture.host,
        &node_drag,
        &pending,
        &fixture.binding,
    );

    assert_eq!(
        outcome,
        Some(LeftPointerReleaseOutcome::NodeDrag(
            NodeDragReleaseOutcome {
                selection_committed: true,
                drag_committed: false,
            }
        ))
    );
    assert!(
        fixture
            .host
            .models
            .read(&node_drag, |state| state.is_none())
            .expect("node drag readable")
    );
    assert!(
        fixture
            .host
            .models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
    assert_single_selection_change(&trace, vec![node_b]);
}

#[test]
fn handle_pending_selection_left_pointer_release_action_host_commits_and_clears_pending_only() {
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9947);
    let node_b = NodeId::from_u128(9948);
    let view_value = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let mut graph_value = Graph::new(GraphId::from_u128(9947));
    graph_value
        .nodes
        .insert(node_a, test_node(CanvasPoint { x: 0.0, y: 0.0 }));
    graph_value
        .nodes
        .insert(node_b, test_node(CanvasPoint { x: 40.0, y: 20.0 }));
    let graph = host.models.insert(graph_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: true,
        clear_groups: true,
    }));

    let outcome =
        handle_pending_selection_left_pointer_release_action_host(&mut host, &pending, &binding);

    assert_eq!(
        outcome,
        Some(LeftPointerReleaseOutcome::PendingSelection {
            selection_committed: true,
        })
    );
    assert!(
        host.models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
    assert_eq!(
        host.models
            .read(&view_state, |state| state.selected_nodes.clone())
            .expect("view state readable"),
        vec![node_b]
    );
}

#[test]
fn handle_marquee_left_pointer_release_action_host_clears_pending_and_marquee_without_commit() {
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9949);
    let node_b = NodeId::from_u128(9950);
    let view_value = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let graph = host.models.insert(Graph::new(GraphId::from_u128(9949)));
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(9949)),
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(4.0), Px(4.0)),
        active: false,
        toggle: true,
        base_selected_nodes: Arc::from([node_a]),
        preview_selected_nodes: Arc::from([node_b]),
    }));
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    }));

    let outcome =
        handle_marquee_left_pointer_release_action_host(&mut host, &marquee, &pending, &binding);

    assert_eq!(
        outcome,
        Some(LeftPointerReleaseOutcome::Marquee {
            selection_committed: false,
        })
    );
    assert!(
        host.models
            .read(&marquee, |state| state.is_none())
            .expect("marquee readable")
    );
    assert!(
        host.models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
    assert_eq!(
        host.models
            .read(&view_state, |state| state.selected_nodes.clone())
            .expect("view state readable"),
        vec![node_a]
    );
}

#[test]
fn complete_node_drag_release_action_host_selection_only_release_notifies_selection_without_drag_commit()
 {
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9951);
    let node_b = NodeId::from_u128(9952);
    let mut graph_value = Graph::new(GraphId::from_u128(9));
    graph_value
        .nodes
        .insert(node_a, test_node(CanvasPoint { x: 0.0, y: 0.0 }));
    graph_value
        .nodes
        .insert(node_b, test_node(CanvasPoint { x: 40.0, y: 20.0 }));
    let graph = host.models.insert(graph_value.clone());
    let initial_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..Default::default()
    };
    let view_state = host.models.insert(initial_view.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        initial_view.clone(),
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store.clone());
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let trace = install_declarative_callback_trace(&mut host, &store);
    let node_drag = NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(12.0), Px(0.0)),
        phase: NodeDragPhase::Armed,
        nodes_sorted: Arc::from([node_b]),
    };
    let pending = PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    };

    let outcome =
        complete_node_drag_release_action_host(&mut host, &binding, &node_drag, Some(&pending));

    assert!(outcome.selection_committed);
    assert!(!outcome.drag_committed);
    let got = trace.borrow();
    assert!(got.commit_labels.is_empty());
    assert_eq!(
        got.selection_changes,
        vec![SelectionChange {
            nodes: vec![node_b],
            edges: Vec::new(),
            groups: Vec::new(),
        }]
    );
}

#[test]
fn complete_node_drag_release_action_host_uses_authoritative_store_graph_when_bound_graph_is_stale()
{
    let node_a = NodeId::from_u128(9953);
    let node_b = NodeId::from_u128(9954);
    let mut fixture = DeclarativeControllerFixture::new_two_nodes(
        15,
        node_a,
        CanvasPoint { x: 100.0, y: 0.0 },
        node_b,
        CanvasPoint { x: 40.0, y: 20.0 },
        NodeGraphViewState::default(),
    );
    let _ = fixture.host.models.update(&fixture.graph, |graph| {
        graph.nodes.get_mut(&node_a).expect("bound mirror node").pos =
            CanvasPoint { x: 0.0, y: 0.0 };
    });
    let node_drag = NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(10.0), Px(0.0)),
        phase: NodeDragPhase::Active,
        nodes_sorted: Arc::from([node_a]),
    };

    let outcome = complete_node_drag_release_action_host(
        &mut fixture.host,
        &fixture.binding,
        &node_drag,
        None,
    );

    assert_eq!(
        outcome,
        NodeDragReleaseOutcome {
            selection_committed: false,
            drag_committed: true,
        }
    );
    let pos = fixture
        .host
        .models
        .read(&fixture.binding.graph_model(), |graph| {
            graph.nodes.get(&node_a).map(|node| node.pos)
        })
        .expect("graph readable")
        .expect("node position");
    assert_eq!(pos, CanvasPoint { x: 110.0, y: 0.0 });
}

#[test]
fn escape_cancel_declarative_interactions_action_host_handles_pending_selection_only() {
    let mut host = TestActionHostImpl::default();
    let drag = host.models.insert(None::<DragState>);
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let node_drag = host.models.insert(None::<NodeDragState>);
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([NodeId::from_u128(9961)]),
        clear_edges: true,
        clear_groups: true,
    }));

    assert!(escape_cancel_declarative_interactions_action_host(
        &mut host, &drag, &marquee, &node_drag, &pending,
    ));
    assert!(
        host.models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
}

#[test]
fn begin_pan_pointer_down_action_host_clears_transients_and_starts_drag() {
    let mut host = TestActionHostImpl::default();
    let drag = host.models.insert(None::<DragState>);
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(1.0), Px(2.0)),
        current_screen: Point::new(Px(3.0), Px(4.0)),
        active: false,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([]),
    }));
    let node_drag = host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(5.0), Px(6.0)),
        current_screen: Point::new(Px(7.0), Px(8.0)),
        phase: NodeDragPhase::Armed,
        nodes_sorted: Arc::from([NodeId::from_u128(9966)]),
    }));
    let down = test_pointer_down(
        MouseButton::Middle,
        Point::new(Px(10.0), Px(11.0)),
        Modifiers::default(),
    );

    assert!(begin_pan_pointer_down_action_host(
        &mut host, &drag, &marquee, &node_drag, down,
    ));
    assert!(
        host.models
            .read(&marquee, |state| state.is_none())
            .expect("marquee readable")
    );
    assert!(
        host.models
            .read(&node_drag, |state| state.is_none())
            .expect("node drag readable")
    );
    host.models
        .read(&drag, |state| {
            let state = state.expect("drag armed");
            assert_eq!(state.button, MouseButton::Middle);
            assert_eq!(state.last_pos, Point::new(Px(10.0), Px(11.0)));
        })
        .expect("drag readable");
}

#[test]
fn begin_left_pointer_down_action_host_hit_node_selectable_arms_pending_selection_and_drag() {
    let mut host = TestActionHostImpl::default();
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(1.0), Px(2.0)),
        current_screen: Point::new(Px(3.0), Px(4.0)),
        active: false,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([]),
    }));
    let node_drag = host.models.insert(None::<NodeDragState>);
    let pending = host.models.insert(None::<PendingSelectionState>);
    let hovered = host.models.insert(None::<NodeId>);
    let hit = NodeId::from_u128(9967);
    let snapshot = LeftPointerDownSnapshot {
        interaction: crate::io::NodeGraphInteractionConfig {
            elements_selectable: true,
            nodes_draggable: true,
            ..Default::default()
        },
        base_selection: vec![NodeId::from_u128(9968)],
        hit: Some(hit),
        edge_hit: None,
    };
    let down = test_pointer_down(
        MouseButton::Left,
        Point::new(Px(12.0), Px(13.0)),
        Modifiers::default(),
    );
    let binding = empty_test_binding(&mut host, 9967);

    let outcome = begin_left_pointer_down_action_host(
        &mut host, &marquee, &node_drag, &pending, &hovered, &binding, down, &snapshot,
    );

    assert_eq!(
        outcome,
        LeftPointerDownOutcome::HitNode {
            capture_pointer: true,
        }
    );
    assert!(outcome.capture_pointer());
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        Some(hit)
    );
    host.models
        .read(&pending, |state| {
            let state = state.as_ref().expect("pending armed");
            assert_eq!(state.nodes.as_ref(), &[hit]);
            assert!(!state.clear_edges);
            assert!(!state.clear_groups);
        })
        .expect("pending readable");
    host.models
        .read(&node_drag, |state| {
            let state = state.as_ref().expect("node drag armed");
            assert_eq!(state.phase, NodeDragPhase::Armed);
            assert_eq!(state.nodes_sorted.as_ref(), &[hit]);
        })
        .expect("node drag readable");
    assert!(
        host.models
            .read(&marquee, |state| state.is_none())
            .expect("marquee readable")
    );
}

#[test]
fn begin_left_pointer_down_action_host_empty_space_arms_marquee() {
    let mut host = TestActionHostImpl::default();
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let node_drag = host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(1.0), Px(1.0)),
        current_screen: Point::new(Px(2.0), Px(2.0)),
        phase: NodeDragPhase::Armed,
        nodes_sorted: Arc::from([NodeId::from_u128(9969)]),
    }));
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([NodeId::from_u128(9970)]),
        clear_edges: false,
        clear_groups: false,
    }));
    let hovered = host.models.insert(Some(NodeId::from_u128(9971)));
    let snapshot = LeftPointerDownSnapshot {
        interaction: crate::io::NodeGraphInteractionConfig {
            elements_selectable: true,
            selection_on_drag: true,
            ..Default::default()
        },
        base_selection: vec![NodeId::from_u128(9972)],
        hit: None,
        edge_hit: None,
    };
    let down = test_pointer_down(
        MouseButton::Left,
        Point::new(Px(20.0), Px(21.0)),
        Modifiers::default(),
    );
    let binding = empty_test_binding(&mut host, 9969);

    let outcome = begin_left_pointer_down_action_host(
        &mut host, &marquee, &node_drag, &pending, &hovered, &binding, down, &snapshot,
    );

    assert_eq!(outcome, LeftPointerDownOutcome::Marquee);
    assert!(outcome.capture_pointer());
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        None
    );
    assert!(
        host.models
            .read(&node_drag, |state| state.is_none())
            .expect("node drag readable")
    );
    assert!(
        host.models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
    host.models
        .read(&marquee, |state| {
            let state = state.as_ref().expect("marquee armed");
            assert_eq!(state.start_screen, Point::new(Px(20.0), Px(21.0)));
            assert_eq!(state.preview_selected_nodes.len(), 0);
        })
        .expect("marquee readable");
}

#[test]
fn begin_left_pointer_down_action_host_empty_space_clear_arms_pending_clear() {
    let mut host = TestActionHostImpl::default();
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let node_drag = host.models.insert(None::<NodeDragState>);
    let pending = host.models.insert(None::<PendingSelectionState>);
    let hovered = host.models.insert(Some(NodeId::from_u128(9973)));
    let snapshot = LeftPointerDownSnapshot {
        interaction: crate::io::NodeGraphInteractionConfig {
            elements_selectable: true,
            ..Default::default()
        },
        base_selection: Vec::new(),
        hit: None,
        edge_hit: None,
    };
    let down = test_pointer_down(
        MouseButton::Left,
        Point::new(Px(30.0), Px(31.0)),
        Modifiers::default(),
    );
    let binding = empty_test_binding(&mut host, 9973);

    let outcome = begin_left_pointer_down_action_host(
        &mut host, &marquee, &node_drag, &pending, &hovered, &binding, down, &snapshot,
    );

    assert_eq!(outcome, LeftPointerDownOutcome::EmptySpaceClear);
    assert!(outcome.capture_pointer());
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        None
    );
    host.models
        .read(&pending, |state| {
            let state = state.as_ref().expect("pending clear armed");
            assert!(state.nodes.is_empty());
            assert!(state.clear_edges);
            assert!(state.clear_groups);
        })
        .expect("pending readable");
}

#[test]
fn read_authoritative_view_state_in_models_uses_store_when_bound_view_is_stale() {
    let mut host = TestActionHostImpl::default();
    let node = NodeId::from_u128(99730);
    let mut authoritative_graph = Graph::new(GraphId::from_u128(99729));
    authoritative_graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 8.0, y: 16.0 }));
    let authoritative_view = NodeGraphViewState {
        pan: CanvasPoint { x: 12.0, y: 24.0 },
        zoom: 2.0,
        selected_nodes: vec![node],
        ..Default::default()
    };
    let stale_view = NodeGraphViewState {
        pan: CanvasPoint { x: -40.0, y: -80.0 },
        zoom: 0.5,
        selected_nodes: Vec::new(),
        ..Default::default()
    };
    let view_state = host.models.insert(stale_view);
    let graph = host.models.insert(Graph::new(GraphId::from_u128(99729)));
    let store = host.models.insert(NodeGraphStore::new(
        authoritative_graph,
        authoritative_view,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);

    let projection = read_authoritative_view_state_in_models(&mut host.models, &binding, |state| {
        (state.pan, state.zoom, state.selected_nodes.clone())
    })
    .expect("projection readable");

    assert_eq!(projection.0, CanvasPoint { x: 12.0, y: 24.0 });
    assert_eq!(projection.1, 2.0);
    assert_eq!(projection.2, vec![node]);
}

#[test]
fn read_left_pointer_down_snapshot_action_host_uses_authoritative_store_view_state_when_bound_view_is_stale()
 {
    let mut host = TestActionHostImpl::default();
    let (graph_value, geom, node_a, node_b) = test_marquee_geometry();
    let spatial =
        crate::ui::canvas::CanvasSpatialDerived::build(&graph_value, &geom, 1.0, 0.0, 64.0);
    let derived_cache = host.models.insert(DerivedGeometryCacheState {
        key: None,
        rebuilds: 1,
        geom: Some(Arc::new(geom)),
        index: Some(Arc::new(spatial)),
    });
    let authoritative_view = NodeGraphViewState {
        pan: CanvasPoint { x: 0.0, y: 0.0 },
        zoom: 1.0,
        selected_nodes: vec![node_b],
        ..Default::default()
    };
    let stale_view = NodeGraphViewState {
        pan: CanvasPoint { x: 400.0, y: 300.0 },
        zoom: 1.0,
        selected_nodes: Vec::new(),
        ..Default::default()
    };
    let view_state = host.models.insert(stale_view);
    let graph = host.models.insert(Graph::new(graph_value.graph_id));
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        authoritative_view,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let hit_scratch = host.models.insert(Vec::<NodeId>::new());
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(400.0), Px(200.0)),
    );
    let down = test_pointer_down(
        MouseButton::Left,
        Point::new(Px(20.0), Px(20.0)),
        Modifiers::default(),
    );

    let snapshot = read_left_pointer_down_snapshot_action_host(
        &mut host,
        &binding,
        &derived_cache,
        &hit_scratch,
        &crate::ui::style::NodeGraphStyle::default(),
        None,
        down,
        bounds,
    );

    assert_eq!(snapshot.hit, Some(node_a));
    assert_eq!(snapshot.edge_hit, None);
    assert_eq!(snapshot.base_selection, vec![node_b]);
}

#[test]
fn handle_node_drag_pointer_move_action_host_activation_commits_pending_selection_and_requests_capture()
 {
    let mut host = TestActionHostImpl::default();
    let view_value = NodeGraphViewState::default();
    let editor_config = test_editor_config(|state| {
        state.interaction.node_drag_threshold = 4.0;
    });
    let view_state = host.models.insert(view_value.clone());
    let mut graph_value = Graph::new(GraphId::from_u128(9973));
    graph_value.nodes.insert(
        NodeId::from_u128(9974),
        test_node(CanvasPoint { x: 0.0, y: 0.0 }),
    );
    graph_value.nodes.insert(
        NodeId::from_u128(9975),
        test_node(CanvasPoint { x: 40.0, y: 20.0 }),
    );
    let store = host
        .models
        .insert(NodeGraphStore::new(graph_value, view_value, editor_config));
    let controller = NodeGraphController::new(store);
    let graph = host.models.insert(Graph::new(GraphId::from_u128(9973)));
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let node_drag = host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        phase: NodeDragPhase::Armed,
        nodes_sorted: Arc::from([NodeId::from_u128(9974)]),
    }));
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([NodeId::from_u128(9975)]),
        clear_edges: false,
        clear_groups: false,
    }));
    let hovered = host.models.insert(Some(NodeId::from_u128(9976)));
    let mv = test_pointer_move(
        Point::new(Px(10.0), Px(0.0)),
        MouseButtons {
            left: true,
            right: false,
            middle: false,
        },
        Modifiers::default(),
    );

    let outcome = handle_node_drag_pointer_move_action_host(
        &mut host, &node_drag, &pending, &hovered, &binding, mv,
    );

    assert_eq!(
        outcome,
        Some(NodeDragPointerMoveOutcome {
            capture_pointer: true,
            needs_layout_redraw: true,
        })
    );
    assert!(
        host.models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        None
    );
    host.models
        .read(&view_state, |state| {
            assert_eq!(state.selected_nodes, vec![NodeId::from_u128(9975)]);
        })
        .expect("view readable");
    host.models
        .read(&node_drag, |state| {
            let state = state.as_ref().expect("node drag readable");
            assert!(state.is_active());
            assert_eq!(state.current_screen, Point::new(Px(10.0), Px(0.0)));
        })
        .expect("node drag readable");
}

#[test]
fn handle_node_drag_pointer_move_action_host_uses_authoritative_store_interaction() {
    let mut host = TestActionHostImpl::default();
    let authoritative_view = NodeGraphViewState::default();
    let editor_config = test_editor_config(|state| {
        state.interaction.node_drag_threshold = 4.0;
    });
    let view_state = host.models.insert(NodeGraphViewState::default());
    let mut graph_value = Graph::new(GraphId::from_u128(99731));
    graph_value.nodes.insert(
        NodeId::from_u128(99741),
        test_node(CanvasPoint { x: 0.0, y: 0.0 }),
    );
    graph_value.nodes.insert(
        NodeId::from_u128(99751),
        test_node(CanvasPoint { x: 40.0, y: 20.0 }),
    );
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        authoritative_view,
        editor_config,
    ));
    let controller = NodeGraphController::new(store);
    let graph = host.models.insert(Graph::new(GraphId::from_u128(99731)));
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let node_drag = host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        phase: NodeDragPhase::Armed,
        nodes_sorted: Arc::from([NodeId::from_u128(99741)]),
    }));
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([NodeId::from_u128(99751)]),
        clear_edges: false,
        clear_groups: false,
    }));
    let hovered = host.models.insert(Some(NodeId::from_u128(99761)));
    let mv = test_pointer_move(
        Point::new(Px(10.0), Px(0.0)),
        MouseButtons {
            left: true,
            right: false,
            middle: false,
        },
        Modifiers::default(),
    );

    let outcome = handle_node_drag_pointer_move_action_host(
        &mut host, &node_drag, &pending, &hovered, &binding, mv,
    );

    assert_eq!(
        outcome,
        Some(NodeDragPointerMoveOutcome {
            capture_pointer: true,
            needs_layout_redraw: true,
        })
    );
    host.models
        .read(&node_drag, |state| {
            let state = state.as_ref().expect("node drag readable");
            assert!(state.is_active());
            assert_eq!(state.current_screen, Point::new(Px(10.0), Px(0.0)));
        })
        .expect("node drag readable");
}

#[test]
fn handle_node_drag_pointer_move_action_host_canceled_session_clears_hover_without_redraw() {
    let mut host = TestActionHostImpl::default();
    let view_value = NodeGraphViewState::default();
    let view_state = host.models.insert(view_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(9976)),
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let graph = host.models.insert(Graph::new(GraphId::from_u128(9976)));
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let node_drag = host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        phase: NodeDragPhase::Canceled,
        nodes_sorted: Arc::from([NodeId::from_u128(9977)]),
    }));
    let pending = host.models.insert(None::<PendingSelectionState>);
    let hovered = host.models.insert(Some(NodeId::from_u128(9978)));
    let mv = test_pointer_move(
        Point::new(Px(2.0), Px(0.0)),
        MouseButtons {
            left: true,
            right: false,
            middle: false,
        },
        Modifiers::default(),
    );

    let outcome = handle_node_drag_pointer_move_action_host(
        &mut host, &node_drag, &pending, &hovered, &binding, mv,
    );

    assert_eq!(
        outcome,
        Some(NodeDragPointerMoveOutcome {
            capture_pointer: false,
            needs_layout_redraw: false,
        })
    );
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        None
    );
}

#[test]
fn handle_marquee_pointer_move_action_host_non_selectable_clears_session_without_touching_hover() {
    let mut host = TestActionHostImpl::default();
    let view_value = NodeGraphViewState::default();
    let editor_config = test_editor_config(|state| {
        state.interaction.elements_selectable = false;
    });
    let view_state = host.models.insert(view_value.clone());
    let graph = host.models.insert(Graph::new(GraphId::from_u128(9979)));
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(9979)),
        view_value,
        editor_config,
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        active: false,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([]),
    }));
    let hovered = host.models.insert(Some(NodeId::from_u128(9979)));
    let derived_cache = host.models.insert(DerivedGeometryCacheState::default());
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(400.0), Px(200.0)),
    );
    let mv = test_pointer_move(
        Point::new(Px(10.0), Px(10.0)),
        MouseButtons::default(),
        Modifiers::default(),
    );

    let outcome = handle_marquee_pointer_move_action_host(
        &mut host,
        &marquee,
        &hovered,
        &binding,
        &derived_cache,
        mv,
        bounds,
    );

    assert_eq!(
        outcome,
        Some(MarqueePointerMoveOutcome::ReleaseCaptureRedrawOnly)
    );
    assert!(
        host.models
            .read(&marquee, |state| state.is_none())
            .expect("marquee readable")
    );
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        Some(NodeId::from_u128(9979))
    );
}

#[test]
fn handle_marquee_pointer_move_action_host_updates_preview_and_clears_hover() {
    let mut host = TestActionHostImpl::default();
    let (graph, geom, node_a, _node_b) = test_marquee_geometry();
    let spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
    let derived_cache = host.models.insert(DerivedGeometryCacheState {
        key: None,
        rebuilds: 1,
        geom: Some(Arc::new(geom)),
        index: Some(Arc::new(spatial)),
    });
    let view_value = NodeGraphViewState::default();
    let editor_config = test_editor_config(|state| {
        state.interaction.elements_selectable = true;
        state.interaction.selection_mode = crate::io::NodeGraphSelectionMode::Partial;
    });
    let view_state = host.models.insert(view_value.clone());
    let graph_model = host.models.insert(graph.clone());
    let store = host
        .models
        .insert(NodeGraphStore::new(graph, view_value, editor_config));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph_model, &view_state, &controller);
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        active: false,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([]),
    }));
    let hovered = host.models.insert(Some(NodeId::from_u128(9980)));
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(400.0), Px(200.0)),
    );
    let mv = test_pointer_move(
        Point::new(Px(80.0), Px(40.0)),
        MouseButtons::default(),
        Modifiers::default(),
    );

    let outcome = handle_marquee_pointer_move_action_host(
        &mut host,
        &marquee,
        &hovered,
        &binding,
        &derived_cache,
        mv,
        bounds,
    );

    assert_eq!(outcome, Some(MarqueePointerMoveOutcome::NotifyRedraw));
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        None
    );
    host.models
        .read(&marquee, |state| {
            let state = state.as_ref().expect("marquee readable");
            assert!(state.active);
            assert_eq!(state.current_screen, Point::new(Px(80.0), Px(40.0)));
            assert_eq!(state.preview_selected_nodes.as_ref(), &[node_a]);
        })
        .expect("marquee readable");
}

#[test]
fn handle_marquee_pointer_move_action_host_uses_authoritative_store_view_when_bound_view_is_stale()
{
    let mut host = TestActionHostImpl::default();
    let (graph_value, geom, node_a, _node_b) = test_marquee_geometry();
    let spatial =
        crate::ui::canvas::CanvasSpatialDerived::build(&graph_value, &geom, 1.0, 0.0, 64.0);
    let derived_cache = host.models.insert(DerivedGeometryCacheState {
        key: None,
        rebuilds: 1,
        geom: Some(Arc::new(geom)),
        index: Some(Arc::new(spatial)),
    });
    let authoritative_view = NodeGraphViewState {
        pan: CanvasPoint { x: 0.0, y: 0.0 },
        zoom: 1.0,
        ..Default::default()
    };
    let stale_view = NodeGraphViewState {
        pan: CanvasPoint { x: 400.0, y: 300.0 },
        zoom: 1.0,
        ..Default::default()
    };
    let editor_config = test_editor_config(|state| {
        state.interaction.elements_selectable = true;
        state.interaction.selection_mode = crate::io::NodeGraphSelectionMode::Partial;
    });
    let view_state = host.models.insert(stale_view);
    let graph = host.models.insert(Graph::new(graph_value.graph_id));
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        authoritative_view,
        editor_config,
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        active: false,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([]),
    }));
    let hovered = host.models.insert(Some(NodeId::from_u128(99801)));
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(400.0), Px(200.0)),
    );
    let mv = test_pointer_move(
        Point::new(Px(80.0), Px(40.0)),
        MouseButtons::default(),
        Modifiers::default(),
    );

    let outcome = handle_marquee_pointer_move_action_host(
        &mut host,
        &marquee,
        &hovered,
        &binding,
        &derived_cache,
        mv,
        bounds,
    );

    assert_eq!(outcome, Some(MarqueePointerMoveOutcome::NotifyRedraw));
    host.models
        .read(&marquee, |state| {
            let state = state.as_ref().expect("marquee readable");
            assert!(state.active);
            assert_eq!(state.preview_selected_nodes.as_ref(), &[node_a]);
        })
        .expect("marquee readable");
}

#[test]
fn update_hovered_node_pointer_move_action_host_sets_hit_node_from_geometry() {
    let mut host = TestActionHostImpl::default();
    let (graph, geom, _node_a, node_b) = test_marquee_geometry();
    let spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
    let derived_cache = host.models.insert(DerivedGeometryCacheState {
        key: None,
        rebuilds: 1,
        geom: Some(Arc::new(geom)),
        index: Some(Arc::new(spatial)),
    });
    let view_value = NodeGraphViewState::default();
    let view_state = host.models.insert(view_value.clone());
    let graph_model = host.models.insert(graph.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph,
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph_model, &view_state, &controller);
    let hovered = host.models.insert(None::<NodeId>);
    let hit_scratch = host.models.insert(Vec::<NodeId>::new());
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(400.0), Px(200.0)),
    );
    let mv = test_pointer_move(
        Point::new(Px(160.0), Px(20.0)),
        MouseButtons::default(),
        Modifiers::default(),
    );

    assert!(update_hovered_node_pointer_move_action_host(
        &mut host,
        &hovered,
        &binding,
        &derived_cache,
        &hit_scratch,
        mv,
        bounds,
    ));
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        Some(node_b)
    );
}

#[test]
fn update_hovered_node_pointer_move_action_host_uses_authoritative_store_view_when_bound_view_is_stale()
 {
    let mut host = TestActionHostImpl::default();
    let (graph_value, geom, _node_a, node_b) = test_marquee_geometry();
    let spatial =
        crate::ui::canvas::CanvasSpatialDerived::build(&graph_value, &geom, 1.0, 0.0, 64.0);
    let derived_cache = host.models.insert(DerivedGeometryCacheState {
        key: None,
        rebuilds: 1,
        geom: Some(Arc::new(geom)),
        index: Some(Arc::new(spatial)),
    });
    let authoritative_view = NodeGraphViewState::default();
    let stale_view = NodeGraphViewState {
        pan: CanvasPoint { x: 400.0, y: 300.0 },
        zoom: 1.0,
        ..Default::default()
    };
    let editor_config = test_editor_config(|state| {
        state.interaction.node_click_distance = 6.0;
    });
    let view_state = host.models.insert(stale_view);
    let graph = host.models.insert(Graph::new(graph_value.graph_id));
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        authoritative_view,
        editor_config,
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let hovered = host.models.insert(None::<NodeId>);
    let hit_scratch = host.models.insert(Vec::<NodeId>::new());
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(400.0), Px(200.0)),
    );
    let mv = test_pointer_move(
        Point::new(Px(160.0), Px(20.0)),
        MouseButtons::default(),
        Modifiers::default(),
    );

    assert!(update_hovered_node_pointer_move_action_host(
        &mut host,
        &hovered,
        &binding,
        &derived_cache,
        &hit_scratch,
        mv,
        bounds,
    ));
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        Some(node_b)
    );
}

#[test]
fn declarative_diag_key_action_from_key_gates_on_diag_toggle() {
    assert_eq!(
        DeclarativeDiagKeyAction::from_key(false, fret_core::KeyCode::Digit3),
        None
    );
    assert_eq!(
        DeclarativeDiagKeyAction::from_key(true, fret_core::KeyCode::Digit3),
        Some(DeclarativeDiagKeyAction::NudgeVisibleNode)
    );
    assert_eq!(
        DeclarativeKeyboardZoomAction::from_key(fret_core::KeyCode::Digit0),
        Some(DeclarativeKeyboardZoomAction::Reset)
    );
}

#[test]
fn declarative_interaction_hook_commits_only_through_binding_dispatch_context() {
    struct MoveNodeOnKey {
        node: NodeId,
        calls: Rc<RefCell<usize>>,
    }

    impl NodeGraphDeclarativeInteractionHook for MoveNodeOnKey {
        fn handle_key_down(
            &mut self,
            ctx: &mut NodeGraphDeclarativeInteractionContext<'_>,
            key: fret_ui::action::KeyDownCx,
        ) -> NodeGraphDeclarativeInteractionOutcome {
            if key.key != fret_core::KeyCode::KeyN {
                return NodeGraphDeclarativeInteractionOutcome::NotHandled;
            }
            *self.calls.borrow_mut() += 1;

            let graph = ctx.graph_snapshot().expect("hook graph snapshot");
            let node = graph.nodes.get(&self.node).expect("node exists");
            let mut tx = GraphTransaction::new().with_label("Hook Move Node");
            tx.push(GraphOp::SetNodePos {
                id: self.node,
                from: node.pos,
                to: CanvasPoint {
                    x: node.pos.x + 8.0,
                    y: node.pos.y + 4.0,
                },
            });
            let outcome = ctx.dispatch_transaction(&tx).expect("hook dispatch");
            assert_eq!(outcome.committed().ops.len(), 1);
            ctx.request_focus_to_surface();
            ctx.request_redraw();
            ctx.notify();
            NodeGraphDeclarativeInteractionOutcome::Handled
        }
    }

    let mut host = TestActionHostImpl::default();
    let node = NodeId::from_u128(99641);
    let mut graph_value = Graph::new(GraphId::from_u128(99641));
    graph_value
        .nodes
        .insert(node, test_node(CanvasPoint { x: 2.0, y: 3.0 }));
    let graph = host.models.insert(graph_value.clone());
    let view_state = host.models.insert(NodeGraphViewState::default());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        NodeGraphViewState::default(),
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store.clone());
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let hook_calls = Rc::new(RefCell::new(0));
    let hook = Rc::new(RefCell::new(MoveNodeOnKey {
        node,
        calls: hook_calls.clone(),
    }));
    let handler = build_key_down_capture_handler(KeyHandlerParams {
        drag: host.models.insert(None::<DragState>),
        marquee_drag: host.models.insert(None::<MarqueeDragState>),
        node_drag: host.models.insert(None::<NodeDragState>),
        reconnect_drag: host.models.insert(None::<ReconnectDragState>),
        pending_selection: host.models.insert(None::<PendingSelectionState>),
        binding: binding.clone(),
        portal_bounds_store: host.models.insert(PortalBoundsStore::default()),
        portal_debug_flags: host.models.insert(PortalDebugFlags::default()),
        diagnostics: NodeGraphDiagnosticsConfig::default(),
        diag_paint_overrides_value: Arc::new(NodeGraphPaintOverridesMap::default()),
        diag_paint_overrides_enabled: host.models.insert(false),
        interaction_hook: Some(hook),
        min_zoom: 0.1,
        max_zoom: 8.0,
    });

    let handled = handler(
        &mut host,
        test_action_cx(),
        fret_ui::action::KeyDownCx {
            key: fret_core::KeyCode::KeyN,
            modifiers: Modifiers::default(),
            repeat: false,
            ime_composing: false,
        },
    );

    assert!(handled);
    assert_eq!(*hook_calls.borrow(), 1);
    let projection_pos = host
        .models
        .read(&graph, |graph| graph.nodes.get(&node).map(|node| node.pos))
        .ok()
        .flatten()
        .expect("projection node pos");
    let store_pos = host
        .models
        .read(&store, |store| {
            store.graph().nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("store node pos");
    assert_eq!(projection_pos, CanvasPoint { x: 10.0, y: 7.0 });
    assert_eq!(store_pos, projection_pos);
    assert_eq!(host.requested_focus, vec![test_action_cx().target]);
    assert_eq!(host.redraw_requests, vec![test_action_cx().window]);
    assert_eq!(host.notifications, vec![test_action_cx()]);
}

#[test]
fn apply_declarative_diag_view_preset_action_host_offset_partial_marquee_clears_selection() {
    let mut host = TestActionHostImpl::default();
    let view_value = NodeGraphViewState {
        zoom: 2.5,
        selected_nodes: vec![NodeId::from_u128(9964)],
        selected_edges: vec![EdgeId::new()],
        selected_groups: vec![GroupId::new()],
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let graph = host.models.insert(Graph::new(GraphId::from_u128(9964)));
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(9964)),
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store.clone());
    let binding = test_binding(&mut host, &graph, &view_state, &controller);

    assert!(apply_declarative_diag_view_preset_action_host(
        &mut host,
        &binding,
        DeclarativeDiagViewPreset::OffsetPartialMarquee,
    ));
    host.models
        .read(&view_state, |state| {
            assert_eq!(state.pan.x, 540.0);
            assert_eq!(state.pan.y, 290.0);
            assert_eq!(state.zoom, 1.0);
            assert!(state.selected_nodes.is_empty());
            assert!(state.selected_edges.is_empty());
            assert!(state.selected_groups.is_empty());
        })
        .expect("view readable");
    host.models
        .read(&store, |state| {
            assert!(state.interaction().selection_on_drag);
            assert_eq!(
                state.interaction().selection_mode,
                crate::io::NodeGraphSelectionMode::Partial
            );
        })
        .expect("store readable");
}

#[test]
fn handle_declarative_diag_key_action_host_disable_portals_clears_pending_fit_and_bounds() {
    let mut host = TestActionHostImpl::default();
    let graph = host.models.insert(Graph::default());
    let view_value = NodeGraphViewState::default();
    let view_state = host.models.insert(view_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(9965)),
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let mut portal_bounds_state = PortalBoundsStore::default();
    portal_bounds_state.pending_fit_to_portals = true;
    portal_bounds_state.nodes_canvas_bounds.insert(
        NodeId::from_u128(9965),
        Rect::new(
            Point::new(Px(1.0), Px(2.0)),
            fret_core::Size::new(Px(3.0), Px(4.0)),
        ),
    );
    let portal_bounds = host.models.insert(portal_bounds_state);
    let portal_debug = host.models.insert(PortalDebugFlags::default());
    let diag_paint_overrides_enabled = host.models.insert(false);
    let diag_paint_overrides = Arc::new(NodeGraphPaintOverridesMap::default());

    assert!(handle_declarative_diag_key_action_host(
        &mut host,
        DeclarativeDiagKeyAction::DisablePortals,
        &binding,
        &portal_bounds,
        &portal_debug,
        &diag_paint_overrides,
        &diag_paint_overrides_enabled,
    ));
    assert!(
        host.models
            .read(&portal_debug, |state| state.disable_portals)
            .expect("portal debug readable")
    );
    host.models
        .read(&portal_bounds, |state| {
            assert!(!state.pending_fit_to_portals);
            assert!(state.nodes_canvas_bounds.is_empty());
        })
        .expect("portal bounds readable");
}

#[test]
fn handle_declarative_keyboard_zoom_action_host_reset_normalizes_zoom() {
    let mut host = TestActionHostImpl::default();
    let view_value = NodeGraphViewState {
        zoom: 2.5,
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let graph = host.models.insert(Graph::new(GraphId::from_u128(9966)));
    let store = host.models.insert(NodeGraphStore::new(
        Graph::new(GraphId::from_u128(9966)),
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);

    assert!(handle_declarative_keyboard_zoom_action_host(
        &mut host,
        DeclarativeKeyboardZoomAction::Reset,
        &binding,
        0.1,
        8.0,
    ));
    assert_eq!(
        host.models
            .read(&view_state, |state| state.zoom)
            .expect("view readable"),
        1.0
    );
}

#[test]
fn handle_declarative_diag_key_action_host_toggle_paint_overrides_sets_first_edge_override() {
    let mut host = TestActionHostImpl::default();
    let edge_id = EdgeId::new();
    let graph = host.models.insert(Graph::new(GraphId::from_u128(9967)));
    let mut authoritative_graph = Graph::new(GraphId::from_u128(9967));
    authoritative_graph.edges.insert(
        edge_id,
        crate::core::Edge {
            kind: crate::core::EdgeKind::Data,
            from: crate::core::PortId::new(),
            to: crate::core::PortId::new(),
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    let view_value = NodeGraphViewState::default();
    let view_state = host.models.insert(view_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        authoritative_graph,
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let portal_bounds = host.models.insert(PortalBoundsStore::default());
    let portal_debug = host.models.insert(PortalDebugFlags::default());
    let diag_paint_overrides_enabled = host.models.insert(false);
    let diag_paint_overrides = Arc::new(NodeGraphPaintOverridesMap::default());

    assert!(handle_declarative_diag_key_action_host(
        &mut host,
        DeclarativeDiagKeyAction::TogglePaintOverrides,
        &binding,
        &portal_bounds,
        &portal_debug,
        &diag_paint_overrides,
        &diag_paint_overrides_enabled,
    ));
    assert!(
        host.models
            .read(&diag_paint_overrides_enabled, |state| *state)
            .expect("flag readable")
    );
    assert!(diag_paint_overrides.edge_paint_override(edge_id).is_some());
}

#[test]
fn escape_cancel_declarative_interactions_action_host_ignores_already_canceled_node_drag() {
    let mut host = TestActionHostImpl::default();
    let drag = host.models.insert(None::<DragState>);
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let node_drag = host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(4.0), Px(0.0)),
        phase: NodeDragPhase::Canceled,
        nodes_sorted: Arc::from([NodeId::from_u128(9962)]),
    }));
    let pending = host.models.insert(None::<PendingSelectionState>);

    assert!(!escape_cancel_declarative_interactions_action_host(
        &mut host, &drag, &marquee, &node_drag, &pending,
    ));
    assert!(
        host.models
            .read(&node_drag, |state| {
                state.as_ref().is_some_and(NodeDragState::is_canceled)
            })
            .expect("node drag readable")
    );
}

#[test]
fn pointer_cancel_declarative_interactions_action_host_clears_already_canceled_node_drag() {
    let mut host = TestActionHostImpl::default();
    let drag = host.models.insert(None::<DragState>);
    let marquee = host.models.insert(None::<MarqueeDragState>);
    let node_drag = host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(4.0), Px(0.0)),
        phase: NodeDragPhase::Canceled,
        nodes_sorted: Arc::from([NodeId::from_u128(9963)]),
    }));
    let pending = host.models.insert(None::<PendingSelectionState>);

    assert!(pointer_cancel_declarative_interactions_action_host(
        &mut host, &drag, &marquee, &node_drag, &pending,
    ));
    assert!(
        host.models
            .read(&node_drag, |state| state.is_none())
            .expect("node drag readable")
    );
}

#[test]
fn pointer_cancel_declarative_interactions_action_host_clears_transients_without_callbacks() {
    let mut host = TestActionHostImpl::default();
    let drag = host.models.insert(Some(DragState {
        button: MouseButton::Left,
        last_pos: Point::new(Px(2.0), Px(3.0)),
    }));
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(10.0), Px(10.0)),
        active: true,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([]),
    }));
    let node_drag = host.models.insert(Some(NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(8.0), Px(0.0)),
        phase: NodeDragPhase::Active,
        nodes_sorted: Arc::from([NodeId::from_u128(9971)]),
    }));
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([NodeId::from_u128(9972)]),
        clear_edges: false,
        clear_groups: false,
    }));

    assert!(pointer_cancel_declarative_interactions_action_host(
        &mut host, &drag, &marquee, &node_drag, &pending,
    ));
    assert!(
        host.models
            .read(&drag, |state| state.is_none())
            .expect("drag readable")
    );
    assert!(
        host.models
            .read(&marquee, |state| state.is_none())
            .expect("marquee readable")
    );
    assert!(
        host.models
            .read(&node_drag, |state| state.is_none())
            .expect("node drag readable")
    );
    assert!(
        host.models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
}

#[test]
fn build_click_selection_preview_nodes_single_click_replaces_base_selection() {
    let node_a = NodeId::from_u128(9401);
    let node_b = NodeId::from_u128(9402);

    let preview = build_click_selection_preview_nodes(&[node_a], node_b, false);

    assert_eq!(preview.as_ref(), &[node_b]);
}

#[test]
fn build_click_selection_preview_nodes_multi_click_toggles_hit_membership() {
    let node_a = NodeId::from_u128(9501);
    let node_b = NodeId::from_u128(9502);

    let added = build_click_selection_preview_nodes(&[node_a], node_b, true);
    let removed = build_click_selection_preview_nodes(&[node_a, node_b], node_b, true);

    assert_eq!(added.as_ref(), &[node_a, node_b]);
    assert_eq!(removed.as_ref(), &[node_a]);
}

#[test]
fn build_click_selection_preview_edges_multi_click_toggles_hit_membership() {
    let edge_a = EdgeId::from_u128(9503);
    let edge_b = EdgeId::from_u128(9504);

    let added = build_click_selection_preview_edges(&[edge_a], edge_b, true);
    let removed = build_click_selection_preview_edges(&[edge_a, edge_b], edge_b, true);
    let replaced = build_click_selection_preview_edges(&[edge_a], edge_b, false);

    assert_eq!(added.as_ref(), &[edge_a, edge_b]);
    assert_eq!(removed.as_ref(), &[edge_a]);
    assert_eq!(replaced.as_ref(), &[edge_b]);
}

#[test]
fn edge_reconnect_endpoint_enabled_resolves_global_and_per_edge_overrides() {
    assert!(edge_reconnect_endpoint_enabled(
        None,
        true,
        EdgeEndpoint::From
    ));
    assert!(!edge_reconnect_endpoint_enabled(
        None,
        false,
        EdgeEndpoint::From
    ));
    assert!(!edge_reconnect_endpoint_enabled(
        Some(EdgeReconnectable::Bool(false)),
        true,
        EdgeEndpoint::From
    ));
    assert!(edge_reconnect_endpoint_enabled(
        Some(EdgeReconnectable::Bool(true)),
        false,
        EdgeEndpoint::To
    ));
    assert!(edge_reconnect_endpoint_enabled(
        Some(EdgeReconnectable::Endpoint(
            EdgeReconnectableEndpoint::Source
        )),
        false,
        EdgeEndpoint::From
    ));
    assert!(!edge_reconnect_endpoint_enabled(
        Some(EdgeReconnectable::Endpoint(
            EdgeReconnectableEndpoint::Source
        )),
        true,
        EdgeEndpoint::To
    ));
    assert!(edge_reconnect_endpoint_enabled(
        Some(EdgeReconnectable::Endpoint(
            EdgeReconnectableEndpoint::Target
        )),
        false,
        EdgeEndpoint::To
    ));
}

#[test]
fn collect_edge_update_anchor_infos_uses_selected_and_focused_edges_with_port_centers() {
    let (mut graph, _a, _a_in, a_out, _b, b_in) = make_graph_two_nodes_with_ports();
    let selected_edge = EdgeId::from_u128(0xA155);
    let focused_edge = EdgeId::from_u128(0xA156);
    for edge in [selected_edge, focused_edge] {
        graph.edges.insert(
            edge,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
    }
    let view_state = NodeGraphViewState {
        selected_edges: vec![selected_edge],
        ..Default::default()
    };
    let mut internals = NodeGraphInternalsSnapshot {
        focused_edge: Some(focused_edge),
        ..Default::default()
    };
    let source_center = Point::new(Px(12.0), Px(34.0));
    let target_center = Point::new(Px(98.0), Px(76.0));
    internals.port_centers_window.insert(a_out, source_center);
    internals.port_centers_window.insert(b_in, target_center);
    let mut interaction = default_editor_config().resolved_interaction_state();
    interaction.edges_reconnectable = true;
    interaction.reconnect_radius = 11.0;

    let anchors = collect_edge_update_anchor_infos(&graph, &view_state, &internals, &interaction);

    assert_eq!(anchors.len(), 4);
    let selected_source = anchors
        .iter()
        .find(|anchor| anchor.edge == selected_edge && anchor.endpoint == EdgeEndpoint::From)
        .expect("selected source anchor");
    assert_eq!(selected_source.anchor_port, a_out);
    assert_eq!(selected_source.opposite_port, b_in);
    assert_eq!(selected_source.center_window, source_center);
    assert_eq!(selected_source.radius, 11.0);
    assert!(
        anchors
            .iter()
            .any(|anchor| anchor.edge == focused_edge && anchor.endpoint == EdgeEndpoint::To)
    );
}

#[test]
fn collect_edge_update_anchor_infos_respects_endpoint_override_missing_centers_and_radius() {
    let (mut graph, _a, _a_in, a_out, _b, b_in) = make_graph_two_nodes_with_ports();
    let edge = EdgeId::from_u128(0xA157);
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: Some(EdgeReconnectable::Endpoint(
                EdgeReconnectableEndpoint::Target,
            )),
        },
    );
    let view_state = NodeGraphViewState {
        selected_edges: vec![edge],
        ..Default::default()
    };
    let mut internals = NodeGraphInternalsSnapshot::default();
    let target_center = Point::new(Px(88.0), Px(42.0));
    internals.port_centers_window.insert(b_in, target_center);
    let mut interaction = default_editor_config().resolved_interaction_state();
    interaction.edges_reconnectable = false;
    interaction.reconnect_radius = 9.0;

    let anchors = collect_edge_update_anchor_infos(&graph, &view_state, &internals, &interaction);

    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors[0].endpoint, EdgeEndpoint::To);
    assert_eq!(anchors[0].anchor_port, b_in);
    assert_eq!(anchors[0].opposite_port, a_out);
    assert_eq!(anchors[0].center_window, target_center);

    interaction.reconnect_radius = f32::NAN;
    assert!(
        collect_edge_update_anchor_infos(&graph, &view_state, &internals, &interaction).is_empty()
    );
}

#[test]
fn edge_update_anchor_controls_render_and_intercept_before_surface_pointer_down() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let edge_ref = graph.edges.get(&edge).expect("test edge");
    let source_center = geom.port_center(edge_ref.from).expect("source port center");
    let target_center = geom.port_center(edge_ref.to).expect("target port center");

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            selected_edges: vec![edge],
            ..Default::default()
        },
        default_editor_config(),
    );

    let source_test_id = format!("node_graph.edge_update_anchor.{}.source", edge.0);
    let target_test_id = format!("node_graph.edge_update_anchor.{}.target", edge.0);
    let mut last = None;
    for _ in 0..4 {
        let snapshot = render_surface_frame_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
        );
        if snapshot
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(source_test_id.as_str()))
        {
            last = Some(snapshot);
            break;
        }
        last = Some(snapshot);
    }
    let snapshot = last.expect("at least one frame rendered");
    let source = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(source_test_id.as_str()))
        .unwrap_or_else(|| {
            let available = snapshot
                .nodes
                .iter()
                .filter_map(|node| node.test_id.as_deref())
                .collect::<Vec<_>>();
            panic!("source update anchor control; available={available:?}")
        });
    let target = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(target_test_id.as_str()))
        .unwrap_or_else(|| {
            let available = snapshot
                .nodes
                .iter()
                .filter_map(|node| node.test_id.as_deref())
                .collect::<Vec<_>>();
            panic!("target update anchor control; available={available:?}")
        });

    assert_eq!(source.role, SemanticsRole::Button);
    assert_eq!(target.role, SemanticsRole::Button);
    assert!(source.value.as_deref().is_some_and(|value| {
        value.contains("endpoint=source") && value.contains("anchor_port=")
    }));
    assert_point_near(
        Point::new(
            Px(source.bounds.origin.x.0 + source.bounds.size.width.0 / 2.0),
            Px(source.bounds.origin.y.0 + source.bounds.size.height.0 / 2.0),
        ),
        source_center,
        0.5,
    );
    assert_point_near(
        Point::new(
            Px(target.bounds.origin.x.0 + target.bounds.size.width.0 / 2.0),
            Px(target.bounds.origin.y.0 + target.bounds.size.height.0 / 2.0),
        ),
        target_center,
        0.5,
    );

    let anchors = host
        .models
        .read(&binding.store_model(), |store| {
            collect_edge_update_anchor_infos(
                store.graph(),
                store.view_state(),
                &binding.internals_store().snapshot(),
                &store.resolved_interaction_state(),
            )
        })
        .expect("read store anchors");
    let source_hit = hit_test_edge_update_anchor_at_window_point(&anchors, source_center)
        .expect("source anchor pure hit");
    assert_eq!(source_hit.edge, edge);
    assert_eq!(source_hit.endpoint, EdgeEndpoint::From);

    assert_eq!(ui.debug_hit_test(source_center).hit, Some(source.id));
    assert_eq!(
        ui.debug_hit_test_routing(source_center).hit,
        Some(source.id)
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: source_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(source.id),
        "anchor pointer down should focus the update-anchor control instead of the canvas surface"
    );

    let outside_anchor = Point::new(Px(bounds.origin.x.0 + 12.0), Px(bounds.origin.y.0 + 12.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(1),
            position: outside_anchor,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        ui.focus().is_some() && ui.focus() != Some(source.id),
        "outside update-anchor controls should fall through to the canvas surface pointer path"
    );
}

#[test]
fn edge_update_anchor_controls_respect_endpoint_reconnectable_gate() {
    let (mut graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let edge_ref = graph.edges.get_mut(&edge).expect("test edge");
    let source_port = edge_ref.from;
    edge_ref.reconnectable = Some(EdgeReconnectable::Endpoint(
        EdgeReconnectableEndpoint::Target,
    ));
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let source_center = geom.port_center(source_port).expect("source port center");

    let mut editor_config = default_editor_config();
    editor_config.interaction.edges_reconnectable = false;
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            selected_edges: vec![edge],
            ..Default::default()
        },
        editor_config,
    );

    let source_test_id = format!("node_graph.edge_update_anchor.{}.source", edge.0);
    let target_test_id = format!("node_graph.edge_update_anchor.{}.target", edge.0);
    let mut last = None;
    for _ in 0..4 {
        let snapshot = render_surface_frame_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
        );
        if snapshot
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(target_test_id.as_str()))
        {
            last = Some(snapshot);
            break;
        }
        last = Some(snapshot);
    }
    let snapshot = last.expect("at least one frame rendered");

    assert!(
        maybe_surface_snapshot_node_id(&snapshot, &source_test_id).is_none(),
        "source endpoint override should suppress the source update-anchor control"
    );
    assert!(
        maybe_surface_snapshot_node_id(&snapshot, &target_test_id).is_some(),
        "target endpoint override should still render the target update-anchor control"
    );
    assert!(
        hit_test_edge_update_anchor_at_window_point(
            &host
                .models
                .read(&binding.store_model(), |store| {
                    collect_edge_update_anchor_infos(
                        store.graph(),
                        store.view_state(),
                        &binding.internals_store().snapshot(),
                        &store.resolved_interaction_state(),
                    )
                })
                .expect("read store anchors"),
            source_center,
        )
        .is_none(),
        "missing source control should not create a pure update-anchor hit"
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: source_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        ui.focus().is_some(),
        "pointer down where the source anchor is gated off should fall through to the canvas"
    );
}

#[test]
fn edge_update_anchor_drag_uses_connection_threshold_before_active_reconnect() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let source_center = graph
        .edges
        .get(&edge)
        .and_then(|edge| geom.port_center(edge.from))
        .expect("source port center");

    let mut editor_config = default_editor_config();
    editor_config.interaction.connection_drag_threshold = 6.0;
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            selected_edges: vec![edge],
            ..Default::default()
        },
        editor_config,
    );

    let source_test_id = format!("node_graph.edge_update_anchor.{}.source", edge.0);
    let snapshot = render_default_surface_frame_until_test_id(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        &source_test_id,
    );
    let source = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(source_test_id.as_str()))
        .expect("source update anchor control");

    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: source_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(source.id));

    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    let value = canvas_semantics_value(&snapshot);
    assert!(value.contains("reconnect_drag_armed:true;"));
    assert!(value.contains("reconnect_dragging:false;"));
    assert!(!binding.internals_store().a11y_snapshot().connecting);

    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: PointerId(0),
            position: Point::new(Px(source_center.x.0 + 3.0), source_center.y),
            buttons: MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    let value = canvas_semantics_value(&snapshot);
    assert!(value.contains("reconnect_drag_armed:true;"));
    assert!(value.contains("reconnect_dragging:false;"));
    assert!(!binding.internals_store().a11y_snapshot().connecting);

    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: PointerId(0),
            position: Point::new(Px(source_center.x.0 + 8.0), source_center.y),
            buttons: MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    let value = canvas_semantics_value(&snapshot);
    assert!(value.contains("reconnect_drag_armed:false;"));
    assert!(value.contains("reconnect_dragging:true;"));
    assert!(binding.internals_store().a11y_snapshot().connecting);

    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: Point::new(Px(source_center.x.0 + 8.0), source_center.y),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    let value = canvas_semantics_value(&snapshot);
    assert!(value.contains("reconnect_drag_armed:false;"));
    assert!(value.contains("reconnect_dragging:false;"));
    assert!(!binding.internals_store().a11y_snapshot().connecting);
}

#[test]
fn edge_update_anchor_reconnect_drag_cancel_paths_clear_transient() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let edge_before = graph.edges.get(&edge).expect("edge present").clone();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let source_center = graph
        .edges
        .get(&edge)
        .and_then(|edge| geom.port_center(edge.from))
        .expect("source port center");

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            selected_edges: vec![edge],
            ..Default::default()
        },
        default_editor_config(),
    );
    let trace = install_declarative_callback_trace(&mut host, &binding.store_model());
    let mode = default_editor_config()
        .resolved_interaction_state()
        .connection_mode;
    let reconnect_kind = ConnectDragKind::Reconnect {
        edge,
        endpoint: EdgeEndpoint::From,
        fixed: edge_before.to,
    };
    let expected_start = ConnectStart {
        kind: reconnect_kind.clone(),
        mode,
    };
    let expected_canceled_end = ConnectEnd {
        kind: reconnect_kind,
        mode,
        target: None,
        outcome: ConnectEndOutcome::Canceled,
    };

    let source_test_id = format!("node_graph.edge_update_anchor.{}.source", edge.0);
    let arm_and_activate = |ui: &mut fret_ui::UiTree<TestActionHostImpl>,
                            host: &mut TestActionHostImpl,
                            services: &mut FakeUiServices| {
        let _snapshot = render_default_surface_frame_until_test_id(
            ui,
            host,
            services,
            window,
            bounds,
            &binding,
            &source_test_id,
        );
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: PointerId(0),
                position: source_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: PointerId(0),
                position: Point::new(Px(source_center.x.0 + 12.0), source_center.y),
                buttons: MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
        let snapshot = render_surface_frame_for_binding(
            ui,
            host,
            services,
            window,
            bounds,
            &binding,
            |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
        );
        assert!(canvas_semantics_value(&snapshot).contains("reconnect_dragging:true;"));
    };

    arm_and_activate(&mut ui, &mut host, &mut services);
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    assert!(canvas_semantics_value(&snapshot).contains("reconnect_dragging:false;"));
    assert!(!binding.internals_store().a11y_snapshot().connecting);

    arm_and_activate(&mut ui, &mut host, &mut services);
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: PointerId(0),
            position: Some(Point::new(Px(source_center.x.0 + 12.0), source_center.y)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );
    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    assert!(canvas_semantics_value(&snapshot).contains("reconnect_dragging:false;"));
    assert!(!binding.internals_store().a11y_snapshot().connecting);

    arm_and_activate(&mut ui, &mut host, &mut services);
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: PointerId(0),
            position: Point::new(Px(source_center.x.0 + 12.0), source_center.y),
            buttons: MouseButtons {
                left: false,
                right: false,
                middle: false,
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    assert!(canvas_semantics_value(&snapshot).contains("reconnect_dragging:false;"));
    assert!(!binding.internals_store().a11y_snapshot().connecting);

    let got = trace.borrow();
    assert_eq!(
        got.connect_starts,
        vec![
            expected_start.clone(),
            expected_start.clone(),
            expected_start.clone()
        ]
    );
    assert_eq!(&got.reconnect_starts, &got.connect_starts);
    assert_eq!(&got.edge_update_starts, &got.connect_starts);
    assert_eq!(
        got.connect_ends,
        vec![
            expected_canceled_end.clone(),
            expected_canceled_end.clone(),
            expected_canceled_end.clone()
        ]
    );
    assert_eq!(&got.reconnect_ends, &got.connect_ends);
    assert_eq!(&got.edge_update_ends, &got.connect_ends);
}

#[test]
fn edge_update_anchor_reconnect_drop_on_valid_port_commits_store_transaction_and_callbacks() {
    let (mut graph, mut draw_order, edge) = make_graph_two_nodes_with_edge();
    let edge_before = graph.edges.get(&edge).expect("edge present").clone();
    let new_source_node = NodeId::from_u128(0xA140);
    let new_source_port = PortId::from_u128(0xA141);
    let mut node_c = test_node(CanvasPoint { x: 320.0, y: 20.0 });
    node_c.ports = vec![new_source_port];
    graph.nodes.insert(new_source_node, node_c);
    graph.ports.insert(
        new_source_port,
        make_port(
            new_source_node,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    draw_order.push(new_source_node);

    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let source_center = geom
        .port_center(edge_before.from)
        .expect("source port center");
    let new_source_center = geom
        .port_center(new_source_port)
        .expect("new source port center");

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            selected_edges: vec![edge],
            ..Default::default()
        },
        default_editor_config(),
    );
    let trace = install_declarative_callback_trace(&mut host, &binding.store_model());

    let source_test_id = format!("node_graph.edge_update_anchor.{}.source", edge.0);
    let _snapshot = render_default_surface_frame_until_test_id(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        &source_test_id,
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: source_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: PointerId(0),
            position: new_source_center,
            buttons: MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: new_source_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    let edge_after = host
        .models
        .read(&binding.store_model(), |store| {
            store
                .graph()
                .edges
                .get(&edge)
                .expect("edge after reconnect")
                .clone()
        })
        .expect("store readable");
    assert_eq!(edge_after.from, new_source_port);
    assert_eq!(edge_after.to, edge_before.to);

    let before = EdgeEndpoints {
        from: edge_before.from,
        to: edge_before.to,
    };
    let after = EdgeEndpoints {
        from: new_source_port,
        to: edge_before.to,
    };
    let mode = default_editor_config()
        .resolved_interaction_state()
        .connection_mode;
    let reconnect_kind = ConnectDragKind::Reconnect {
        edge,
        endpoint: EdgeEndpoint::From,
        fixed: edge_before.to,
    };
    let expected_start = ConnectStart {
        kind: reconnect_kind.clone(),
        mode,
    };
    let expected_end = ConnectEnd {
        kind: reconnect_kind,
        mode,
        target: Some(new_source_port),
        outcome: ConnectEndOutcome::Committed,
    };
    let got = trace.borrow();
    assert_eq!(got.commit_labels, vec![Some("Reconnect Edge".to_string())]);
    assert_eq!(got.reconnects, vec![(edge, before, after)]);
    assert_eq!(got.edge_updates, vec![(edge, before, after)]);
    assert_eq!(got.connect_starts, vec![expected_start.clone()]);
    assert_eq!(got.reconnect_starts, vec![expected_start.clone()]);
    assert_eq!(got.edge_update_starts, vec![expected_start]);
    assert_eq!(got.connect_ends, vec![expected_end.clone()]);
    assert_eq!(got.reconnect_ends, vec![expected_end.clone()]);
    assert_eq!(got.edge_update_ends, vec![expected_end]);
}

#[test]
fn edge_update_anchor_reconnect_drop_on_non_start_connectable_port_clears_without_commit() {
    let (mut graph, mut draw_order, edge) = make_graph_two_nodes_with_edge();
    let edge_before = graph.edges.get(&edge).expect("edge present").clone();
    let new_source_node = NodeId::from_u128(0xA150);
    let new_source_port = PortId::from_u128(0xA151);
    let mut node_c = test_node(CanvasPoint { x: 320.0, y: 20.0 });
    node_c.ports = vec![new_source_port];
    graph.nodes.insert(new_source_node, node_c);
    let mut port = make_port(
        new_source_node,
        "out",
        PortDirection::Out,
        PortKind::Data,
        PortCapacity::Multi,
    );
    port.connectable_start = Some(false);
    graph.ports.insert(new_source_port, port);
    draw_order.push(new_source_node);

    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let source_center = geom
        .port_center(edge_before.from)
        .expect("source port center");
    let new_source_center = geom
        .port_center(new_source_port)
        .expect("new source port center");

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            selected_edges: vec![edge],
            ..Default::default()
        },
        default_editor_config(),
    );
    let trace = install_declarative_callback_trace(&mut host, &binding.store_model());

    let source_test_id = format!("node_graph.edge_update_anchor.{}.source", edge.0);
    let _snapshot = render_default_surface_frame_until_test_id(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        &source_test_id,
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: source_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: PointerId(0),
            position: new_source_center,
            buttons: MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: new_source_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    let edge_after = host
        .models
        .read(&binding.store_model(), |store| {
            store
                .graph()
                .edges
                .get(&edge)
                .expect("edge after rejected drop")
                .clone()
        })
        .expect("store readable");
    assert_eq!(edge_after.from, edge_before.from);
    assert_eq!(edge_after.to, edge_before.to);

    let got = trace.borrow();
    assert!(got.commit_labels.is_empty());
    assert!(got.reconnects.is_empty());
    assert!(got.edge_updates.is_empty());
    let mode = default_editor_config()
        .resolved_interaction_state()
        .connection_mode;
    let reconnect_kind = ConnectDragKind::Reconnect {
        edge,
        endpoint: EdgeEndpoint::From,
        fixed: edge_before.to,
    };
    let expected_start = ConnectStart {
        kind: reconnect_kind.clone(),
        mode,
    };
    let expected_end = ConnectEnd {
        kind: reconnect_kind,
        mode,
        target: Some(new_source_port),
        outcome: ConnectEndOutcome::Rejected,
    };
    assert_eq!(got.connect_starts, vec![expected_start.clone()]);
    assert_eq!(got.reconnect_starts, vec![expected_start.clone()]);
    assert_eq!(got.edge_update_starts, vec![expected_start]);
    assert_eq!(got.connect_ends, vec![expected_end.clone()]);
    assert_eq!(got.reconnect_ends, vec![expected_end.clone()]);
    assert_eq!(got.edge_update_ends, vec![expected_end]);
    drop(got);

    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    let value = canvas_semantics_value(&snapshot);
    assert!(value.contains("reconnect_drag_armed:false;"));
    assert!(value.contains("reconnect_dragging:false;"));
    assert!(!binding.internals_store().a11y_snapshot().connecting);
}

#[test]
fn edge_update_anchor_reconnect_drop_on_empty_space_clears_without_commit() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let edge_before = graph.edges.get(&edge).expect("edge present").clone();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let source_center = geom
        .port_center(edge_before.from)
        .expect("source port center");

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            selected_edges: vec![edge],
            ..Default::default()
        },
        default_editor_config(),
    );
    let trace = install_declarative_callback_trace(&mut host, &binding.store_model());

    let source_test_id = format!("node_graph.edge_update_anchor.{}.source", edge.0);
    let _snapshot = render_default_surface_frame_until_test_id(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        &source_test_id,
    );

    let empty_space = Point::new(Px(bounds.origin.x.0 + 760.0), Px(bounds.origin.y.0 + 540.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: source_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: PointerId(0),
            position: empty_space,
            buttons: MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: empty_space,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    let edge_after = host
        .models
        .read(&binding.store_model(), |store| {
            store
                .graph()
                .edges
                .get(&edge)
                .expect("edge after empty-space drop")
                .clone()
        })
        .expect("store readable");
    assert_eq!(edge_after.from, edge_before.from);
    assert_eq!(edge_after.to, edge_before.to);

    let got = trace.borrow();
    assert!(got.commit_labels.is_empty());
    assert!(got.reconnects.is_empty());
    assert!(got.edge_updates.is_empty());
    let mode = default_editor_config()
        .resolved_interaction_state()
        .connection_mode;
    let reconnect_kind = ConnectDragKind::Reconnect {
        edge,
        endpoint: EdgeEndpoint::From,
        fixed: edge_before.to,
    };
    let expected_start = ConnectStart {
        kind: reconnect_kind.clone(),
        mode,
    };
    let expected_end = ConnectEnd {
        kind: reconnect_kind,
        mode,
        target: None,
        outcome: ConnectEndOutcome::NoOp,
    };
    assert_eq!(got.connect_starts, vec![expected_start.clone()]);
    assert_eq!(got.reconnect_starts, vec![expected_start.clone()]);
    assert_eq!(got.edge_update_starts, vec![expected_start]);
    assert_eq!(got.connect_ends, vec![expected_end.clone()]);
    assert_eq!(got.reconnect_ends, vec![expected_end.clone()]);
    assert_eq!(got.edge_update_ends, vec![expected_end]);
    drop(got);

    let snapshot = render_surface_frame_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        |binding| super::NodeGraphSurfaceProps::new(binding.clone()),
    );
    let value = canvas_semantics_value(&snapshot);
    assert!(value.contains("reconnect_drag_armed:false;"));
    assert!(value.contains("reconnect_dragging:false;"));
    assert!(!binding.internals_store().a11y_snapshot().connecting);
}

#[test]
fn edge_stroke_width_mul_for_selection_applies_selected_edge_width_token() {
    let mut style = crate::ui::style::NodeGraphStyle::default();
    style.paint.wire_width_selected_mul = 1.75;

    assert_eq!(edge_stroke_width_mul_for_selection(2.0, false, &style), 2.0);
    assert_eq!(edge_stroke_width_mul_for_selection(2.0, true, &style), 3.5);

    style.paint.wire_width_selected_mul = 0.0;
    assert_eq!(edge_stroke_width_mul_for_selection(2.0, true, &style), 2.0);
    assert_eq!(
        edge_stroke_width_mul_for_selection(f32::NAN, true, &style),
        1.0
    );
}

#[test]
fn commit_edge_click_selection_action_host_multi_toggles_edge_without_clearing_other_kinds() {
    let mut host = TestActionHostImpl::default();
    let node = NodeId::from_u128(9505);
    let edge = EdgeId::from_u128(9506);
    let group = GroupId::from_u128(9507);
    let view_value = NodeGraphViewState {
        selected_nodes: vec![node],
        selected_edges: vec![edge],
        selected_groups: vec![group],
        ..Default::default()
    };
    let mut graph_value = Graph::new(GraphId::from_u128(9505));
    graph_value
        .nodes
        .insert(node, test_node(CanvasPoint { x: 0.0, y: 0.0 }));
    graph_value.groups.insert(
        group,
        Group {
            title: "group".into(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 80.0,
                },
            },
            color: None,
        },
    );
    graph_value.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: PortId::new(),
            to: PortId::new(),
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph_value,
        view_value,
        default_editor_config(),
    );

    assert!(commit_edge_click_selection_action_host(
        &mut host, &binding, edge, true,
    ));

    let selection = host
        .models
        .read(&binding.view_state_model(), |state| {
            (
                state.selected_nodes.clone(),
                state.selected_edges.clone(),
                state.selected_groups.clone(),
            )
        })
        .expect("read view state");
    assert_eq!(selection.0, vec![node]);
    assert!(selection.1.is_empty());
    assert_eq!(selection.2, vec![group]);
}

#[test]
fn commit_pending_selection_action_host_preserves_edges_and_groups_when_not_requested() {
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9601);
    let node_b = NodeId::from_u128(9602);
    let edge = EdgeId::new();
    let group = GroupId::new();
    let view_value = NodeGraphViewState {
        selected_nodes: vec![node_a],
        selected_edges: vec![edge],
        selected_groups: vec![group],
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let mut graph_value = Graph::new(GraphId::from_u128(9601));
    let from_port = PortId::new();
    let to_port = PortId::new();
    let mut node_a_value = test_node(CanvasPoint { x: 0.0, y: 0.0 });
    node_a_value.ports = vec![from_port];
    let mut node_b_value = test_node(CanvasPoint { x: 40.0, y: 20.0 });
    node_b_value.ports = vec![to_port];
    graph_value.nodes.insert(node_a, node_a_value);
    graph_value.nodes.insert(node_b, node_b_value);
    graph_value.ports.insert(
        from_port,
        Port {
            node: node_a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.ports.insert(
        to_port,
        Port {
            node: node_b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: from_port,
            to: to_port,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph_value.groups.insert(
        group,
        Group {
            title: "test group".into(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 1.0,
                    height: 1.0,
                },
            },
            color: None,
        },
    );
    let graph = host.models.insert(graph_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let pending = PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    };

    assert!(commit_pending_selection_action_host(
        &mut host, &binding, &pending,
    ));

    let selection = host
        .models
        .read(&view_state, |state| {
            (
                state.selected_nodes.clone(),
                state.selected_edges.clone(),
                state.selected_groups.clone(),
            )
        })
        .expect("read view state");
    assert_eq!(selection.0, vec![node_b]);
    assert_eq!(selection.1, vec![edge]);
    assert_eq!(selection.2, vec![group]);
}

#[test]
fn commit_pending_selection_action_host_preserves_authoritative_selection_when_bound_view_is_stale()
{
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9611);
    let node_b = NodeId::from_u128(9612);
    let edge = EdgeId::new();
    let group = GroupId::new();
    let view_value = NodeGraphViewState {
        selected_nodes: vec![node_a],
        selected_edges: vec![edge],
        selected_groups: vec![group],
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let mut graph_value = Graph::new(GraphId::from_u128(9611));
    let from_port = PortId::new();
    let to_port = PortId::new();
    let mut node_a_value = test_node(CanvasPoint { x: 0.0, y: 0.0 });
    node_a_value.ports = vec![from_port];
    let mut node_b_value = test_node(CanvasPoint { x: 40.0, y: 20.0 });
    node_b_value.ports = vec![to_port];
    graph_value.nodes.insert(node_a, node_a_value);
    graph_value.nodes.insert(node_b, node_b_value);
    graph_value.ports.insert(
        from_port,
        Port {
            node: node_a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.ports.insert(
        to_port,
        Port {
            node: node_b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: from_port,
            to: to_port,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph_value.groups.insert(
        group,
        Group {
            title: "test group".into(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 1.0,
                    height: 1.0,
                },
            },
            color: None,
        },
    );
    let graph = host.models.insert(graph_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let _ = host.models.update(&view_state, |state| {
        state.selected_nodes = vec![node_a];
        state.selected_edges.clear();
        state.selected_groups.clear();
    });
    let pending = PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    };

    assert!(commit_pending_selection_action_host(
        &mut host, &binding, &pending,
    ));

    let selection = host
        .models
        .read(&view_state, |state| {
            (
                state.selected_nodes.clone(),
                state.selected_edges.clone(),
                state.selected_groups.clone(),
            )
        })
        .expect("read view state");
    assert_eq!(selection.0, vec![node_b]);
    assert_eq!(selection.1, vec![edge]);
    assert_eq!(selection.2, vec![group]);
}

#[test]
fn commit_pending_selection_action_host_can_clear_all_selection_kinds() {
    let mut host = TestActionHostImpl::default();
    let node = NodeId::from_u128(9701);
    let other = NodeId::from_u128(9702);
    let edge = EdgeId::new();
    let group = GroupId::new();
    let view_value = NodeGraphViewState {
        selected_nodes: vec![node],
        selected_edges: vec![edge],
        selected_groups: vec![group],
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let mut graph_value = Graph::new(GraphId::from_u128(9701));
    let from_port = PortId::new();
    let to_port = PortId::new();
    let mut node_value = test_node(CanvasPoint { x: 0.0, y: 0.0 });
    node_value.ports = vec![from_port];
    let mut other_value = test_node(CanvasPoint { x: 40.0, y: 20.0 });
    other_value.ports = vec![to_port];
    graph_value.nodes.insert(node, node_value);
    graph_value.nodes.insert(other, other_value);
    graph_value.ports.insert(
        from_port,
        Port {
            node,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.ports.insert(
        to_port,
        Port {
            node: other,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: from_port,
            to: to_port,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph_value.groups.insert(
        group,
        Group {
            title: "test group".into(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 1.0,
                    height: 1.0,
                },
            },
            color: None,
        },
    );
    let graph = host.models.insert(graph_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let pending = PendingSelectionState {
        nodes: Arc::from([]),
        clear_edges: true,
        clear_groups: true,
    };

    assert!(commit_pending_selection_action_host(
        &mut host, &binding, &pending,
    ));

    let selection = host
        .models
        .read(&view_state, |state| {
            (
                state.selected_nodes.clone(),
                state.selected_edges.clone(),
                state.selected_groups.clone(),
            )
        })
        .expect("read view state");
    assert!(selection.0.is_empty());
    assert!(selection.1.is_empty());
    assert!(selection.2.is_empty());
}

#[test]
fn update_view_state_action_host_uses_authoritative_store_view_state_when_bound_view_is_stale() {
    let mut host = TestActionHostImpl::default();
    let node = NodeId::from_u128(97031);
    let graph = host.models.insert(Graph::new(GraphId::from_u128(9703)));
    let mut authoritative_graph = Graph::new(GraphId::from_u128(9703));
    authoritative_graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 8.0, y: 16.0 }));
    let authoritative = NodeGraphViewState {
        pan: CanvasPoint { x: 12.0, y: 24.0 },
        zoom: 2.5,
        selected_nodes: vec![node],
        ..Default::default()
    };
    let stale = NodeGraphViewState {
        pan: CanvasPoint { x: -5.0, y: -7.0 },
        zoom: 0.5,
        selected_nodes: Vec::new(),
        ..Default::default()
    };
    let view_state = host.models.insert(stale);
    let store = host.models.insert(NodeGraphStore::new(
        authoritative_graph,
        authoritative,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);

    assert!(update_view_state_action_host(
        &mut host,
        &binding,
        |state| state.pan.x += 5.0,
    ));

    let updated = host
        .models
        .read(&binding.view_state_model(), |state| state.clone())
        .expect("view state readable");
    let synced_node = host
        .models
        .read(&binding.graph_model(), |graph| {
            graph.nodes.get(&node).map(|node| node.pos)
        })
        .expect("graph readable");
    assert_eq!(updated.pan, CanvasPoint { x: 17.0, y: 24.0 });
    assert_eq!(updated.zoom, 2.5);
    assert_eq!(updated.selected_nodes, vec![node]);
    assert_eq!(synced_node, Some(CanvasPoint { x: 8.0, y: 16.0 }));
}

fn declarative_paint_only_runtime_sources() -> [(&'static str, &'static str); 24] {
    [
        ("paint_only.rs", include_str!("../paint_only.rs")),
        ("cache.rs", include_str!("cache.rs")),
        ("diag.rs", include_str!("diag.rs")),
        ("frame_plan.rs", include_str!("frame_plan.rs")),
        ("hover_anchor.rs", include_str!("hover_anchor.rs")),
        ("input_handlers.rs", include_str!("input_handlers.rs")),
        ("interaction_hooks.rs", include_str!("interaction_hooks.rs")),
        ("overlay_elements.rs", include_str!("overlay_elements.rs")),
        ("overlays.rs", include_str!("overlays.rs")),
        ("pointer_down.rs", include_str!("pointer_down.rs")),
        ("pointer_move.rs", include_str!("pointer_move.rs")),
        ("pointer_session.rs", include_str!("pointer_session.rs")),
        (
            "portal_measurement.rs",
            include_str!("portal_measurement.rs"),
        ),
        ("portals.rs", include_str!("portals.rs")),
        ("selection.rs", include_str!("selection.rs")),
        ("semantics.rs", include_str!("semantics.rs")),
        ("surface_content.rs", include_str!("surface_content.rs")),
        ("surface_frame.rs", include_str!("surface_frame.rs")),
        ("surface_math.rs", include_str!("surface_math.rs")),
        ("surface_models.rs", include_str!("surface_models.rs")),
        ("surface_shell.rs", include_str!("surface_shell.rs")),
        ("surface_state.rs", include_str!("surface_state.rs")),
        ("surface_support.rs", include_str!("surface_support.rs")),
        ("transactions.rs", include_str!("transactions.rs")),
    ]
}

#[test]
fn declarative_paint_only_runtime_uses_authoritative_store_models_instead_of_bound_mirrors() {
    for (path, source) in declarative_paint_only_runtime_sources() {
        assert!(
            !source.contains("binding.graph_model()"),
            "{path} must not read/write the bound graph mirror; use binding.store_model() instead",
        );
        assert!(
            !source.contains("binding.view_state_model()"),
            "{path} must not read/write the bound view-state mirror; use binding.store_model() instead",
        );
        assert!(
            !source.contains("binding.editor_config_model()"),
            "{path} must not read/write the bound editor-config mirror; use binding.store_model() instead",
        );
    }
}

#[test]
fn declarative_paint_only_graph_edit_paths_stay_on_transactions_seam() {
    for (path, source) in declarative_paint_only_runtime_sources() {
        assert!(
            !source.contains("replace_graph("),
            "{path} must not replace the graph document directly; graph-edit gestures must stay transaction-backed",
        );
        assert!(
            !source.contains("replace_document("),
            "{path} must not replace the authoritative document directly; graph-edit gestures must stay transaction-backed",
        );

        if path == "transactions.rs" || path == "interaction_hooks.rs" {
            continue;
        }

        assert!(
            !source.contains("dispatch_transaction_action_host("),
            "{path} must not dispatch graph transactions directly; route graph commits through paint_only/transactions.rs",
        );
        assert!(
            !source.contains("submit_transaction_action_host("),
            "{path} must not submit graph transactions directly; route graph commits through paint_only/transactions.rs",
        );
    }
}

#[test]
fn visible_subset_portal_hosting_config_defaults_to_enabled_capped_layer() {
    let config = NodeGraphVisibleSubsetPortalConfig::default();
    assert!(config.enabled);
    assert_eq!(config.max_nodes, 32);
}

#[test]
fn node_graph_surface_props_declare_visible_subset_portal_hosting_config() {
    let source = include_str!("../paint_only.rs");
    assert!(source.contains("pub portal_hosting: NodeGraphVisibleSubsetPortalConfig"));
    assert!(!source.contains("pub portals_enabled: bool"));
    assert!(!source.contains("pub portal_max_nodes: usize"));
}

#[test]
fn diagnostics_config_defaults_to_disabled() {
    let config = NodeGraphDiagnosticsConfig::default();
    assert!(!config.key_actions_enabled);
    assert!(!config.hover_tooltip_enabled);
}

#[test]
fn node_graph_surface_props_declare_explicit_diagnostics_config() {
    let source = include_str!("../paint_only.rs");
    assert!(source.contains("pub diagnostics: NodeGraphDiagnosticsConfig"));
    assert!(!source.contains("std::env::var(\"FRET_DIAG\")"));
}

#[test]
fn root_ui_surface_reexports_declarative_policy_configs() {
    let source = include_str!("../../mod.rs");
    assert!(source.contains("NodeGraphDiagnosticsConfig"));
    assert!(source.contains("NodeGraphVisibleSubsetPortalConfig"));
}

#[test]
fn declarative_paint_only_runtime_does_not_read_diag_env_directly() {
    for (path, source) in declarative_paint_only_runtime_sources() {
        assert!(
            !source.contains("FRET_DIAG"),
            "{path} must not read diagnostics policy from process env; use surface diagnostics config instead",
        );
    }
}

#[test]
fn declarative_overlay_runtime_does_not_depend_on_portal_hosting_module() {
    let source = include_str!("overlays.rs");
    assert!(!source.contains("use super::portals::"));
    assert!(!source.contains("collect_hovered_node_label_and_ports("));
}

fn test_node_drag_state(phase: NodeDragPhase, current_screen: Point) -> NodeDragState {
    NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen,
        phase,
        nodes_sorted: Arc::from([NodeId::from_u128(9800)]),
    }
}

#[test]
fn node_drag_phase_activation_crosses_threshold() {
    let mut drag = test_node_drag_state(NodeDragPhase::Armed, Point::new(Px(0.0), Px(0.0)));
    let next = Point::new(Px(6.0), Px(8.0));

    assert!(pointer_crossed_threshold(drag.start_screen, next, 10.0));
    assert!(drag.activate(next));
    assert!(drag.is_active());
    assert_eq!(drag.current_screen, next);
}

#[test]
fn canceled_node_drag_does_not_produce_commit_delta() {
    let view = PanZoom2D::default();
    let mut drag = test_node_drag_state(NodeDragPhase::Armed, Point::new(Px(12.0), Px(0.0)));

    assert!(drag.activate(Point::new(Px(12.0), Px(0.0))));
    drag.cancel();

    assert!(drag.is_canceled());
    assert_eq!(node_drag_commit_delta(view, &drag), None);
}

#[test]
fn active_node_drag_with_non_zero_delta_produces_commit_delta() {
    let view = PanZoom2D {
        pan: Point::new(Px(0.0), Px(0.0)),
        zoom: 2.0,
    };
    let drag = test_node_drag_state(NodeDragPhase::Active, Point::new(Px(8.0), Px(-6.0)));

    assert_eq!(node_drag_commit_delta(view, &drag), Some((4.0, -3.0)));
}

#[test]
fn armed_node_drag_release_keeps_drag_commit_local() {
    let view = PanZoom2D::default();
    let drag = test_node_drag_state(NodeDragPhase::Armed, Point::new(Px(14.0), Px(0.0)));

    assert_eq!(node_drag_commit_delta(view, &drag), None);
}

#[test]
fn build_marquee_preview_selected_nodes_non_toggle_uses_current_candidates() {
    let (graph, geom, node_a, node_b) = test_marquee_geometry();
    let spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
    let marquee = MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        active: true,
        toggle: false,
        base_selected_nodes: Arc::from([node_b]),
        preview_selected_nodes: Arc::from([]),
    };
    let rect = Rect::new(
        Point::new(Px(-10.0), Px(-10.0)),
        fret_core::Size::new(Px(120.0), Px(80.0)),
    );

    let preview = build_marquee_preview_selected_nodes(
        &marquee,
        rect,
        crate::io::NodeGraphSelectionMode::Partial,
        &geom,
        &spatial,
    );

    assert_eq!(preview.as_ref(), &[node_a]);
}

#[test]
fn build_marquee_preview_selected_nodes_toggle_flips_against_base_selection() {
    let (graph, geom, node_a, node_b) = test_marquee_geometry();
    let spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
    let marquee = MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        active: true,
        toggle: true,
        base_selected_nodes: Arc::from([node_a]),
        preview_selected_nodes: Arc::from([]),
    };
    let rect = Rect::new(
        Point::new(Px(-10.0), Px(-10.0)),
        fret_core::Size::new(Px(260.0), Px(80.0)),
    );

    let preview = build_marquee_preview_selected_nodes(
        &marquee,
        rect,
        crate::io::NodeGraphSelectionMode::Partial,
        &geom,
        &spatial,
    );

    assert_eq!(preview.as_ref(), &[node_b]);
}

#[test]
fn effective_selected_nodes_for_paint_prefers_active_marquee_preview() {
    let node_a = NodeId::from_u128(9001);
    let node_b = NodeId::from_u128(9002);
    let node_c = NodeId::from_u128(9003);
    let view_state = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..NodeGraphViewState::default()
    };
    let pending = PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: true,
        clear_groups: true,
    };
    let marquee = MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(10.0), Px(10.0)),
        active: true,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([node_c]),
    };

    let effective = effective_selected_nodes_for_paint(&view_state, Some(&marquee), Some(&pending));

    assert_eq!(effective, vec![node_c]);
}

#[test]
fn effective_selected_nodes_for_paint_falls_back_from_inactive_marquee_to_pending_then_view() {
    let node_a = NodeId::from_u128(9011);
    let node_b = NodeId::from_u128(9012);
    let view_state = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..NodeGraphViewState::default()
    };
    let pending = PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    };
    let inactive_marquee = MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(10.0), Px(10.0)),
        active: false,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([NodeId::from_u128(9013)]),
    };

    let from_pending =
        effective_selected_nodes_for_paint(&view_state, Some(&inactive_marquee), Some(&pending));
    let from_view = effective_selected_nodes_for_paint(&view_state, None, None);

    assert_eq!(from_pending, vec![node_b]);
    assert_eq!(from_view, vec![node_a]);
}

#[test]
fn paint_only_interaction_frame_plan_is_pure_snapshot_state() {
    let node_a = NodeId::from_u128(9021);
    let node_b = NodeId::from_u128(9022);
    let node_c = NodeId::from_u128(9023);
    let view_state = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..NodeGraphViewState::default()
    };
    let drag = DragState {
        button: MouseButton::Middle,
        last_pos: Point::new(Px(0.0), Px(0.0)),
    };
    let pending = PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    };
    let marquee = MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(10.0), Px(10.0)),
        active: false,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([node_c]),
    };
    let node_drag = NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(2.0), Px(3.0)),
        phase: NodeDragPhase::Armed,
        nodes_sorted: Arc::from([node_a]),
    };

    let plan = plan_paint_only_interaction_frame(PaintOnlyInteractionFrameInputs {
        view_state: &view_state,
        drag: Some(drag),
        marquee: Some(&marquee),
        node_drag: Some(&node_drag),
        reconnect_drag: None,
        pending_selection: Some(&pending),
        hovered_node: Some(node_c),
    });

    assert!(plan.panning);
    assert!(!plan.marquee_active);
    assert!(plan.node_drag_armed);
    assert!(!plan.node_dragging);
    assert!(!plan.reconnect_drag_armed);
    assert!(!plan.reconnect_dragging);
    assert!(plan.hovered);
    assert_eq!(plan.hovered_node, Some(node_c));
    assert_eq!(plan.effective_selected_nodes, vec![node_b]);
    assert_eq!(plan.selected_nodes_len(), 1);
}

#[test]
fn collect_node_label_and_ports_reads_kind_and_direction_counts() {
    let node = NodeId::from_u128(9097);
    let port_in = PortId::from_u128(9098);
    let port_out_a = PortId::from_u128(9099);
    let port_out_b = PortId::from_u128(9100);
    let mut graph = Graph::new(GraphId::from_u128(9096));
    graph.nodes.insert(
        node,
        Node {
            kind: NodeKindKey::new("test.portal.summary"),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![port_in, port_out_a, port_out_b],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        port_in,
        Port {
            node,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    for (port, key) in [(port_out_a, "out_a"), (port_out_b, "out_b")] {
        graph.ports.insert(
            port,
            Port {
                node,
                key: PortKey::new(key),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
    }

    let (label, ports_in, ports_out) =
        collect_node_label_and_ports(&graph, node).expect("summary should exist");
    assert_eq!(&*label, "test.portal.summary");
    assert_eq!(ports_in, 1);
    assert_eq!(ports_out, 2);
}

#[test]
fn collect_portal_label_infos_for_visible_subset_uses_dragged_rect_for_visibility() {
    let node = NodeId::from_u128(9101);
    let mut graph = Graph::new(GraphId::from_u128(9100));
    graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 200.0, y: 0.0 }));
    let draws = vec![NodeRectDraw {
        id: node,
        rect: Rect::new(
            Point::new(Px(200.0), Px(0.0)),
            fret_core::Size::new(Px(40.0), Px(20.0)),
        ),
    }];
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(100.0), Px(100.0)),
    );
    let view = PanZoom2D::default();
    let cull = Some(Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(100.0), Px(100.0)),
    ));
    let drag = NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(-160.0), Px(0.0)),
        phase: NodeDragPhase::Active,
        nodes_sorted: Arc::from([node]),
    };

    let infos = collect_portal_label_infos_for_visible_subset(
        &graph,
        Some(draws.as_slice()),
        bounds,
        view,
        cull,
        8,
        Some(node),
        &[node],
        Some(&drag),
    );

    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].id, node);
    assert_eq!(infos[0].left, Px(40.0));
    assert!(infos[0].selected);
    assert!(infos[0].hovered);
}

#[test]
fn collect_portal_label_infos_for_visible_subset_respects_draw_order_and_cap() {
    let node_a = NodeId::from_u128(9111);
    let node_b = NodeId::from_u128(9112);
    let node_c = NodeId::from_u128(9113);
    let mut graph = Graph::new(GraphId::from_u128(9110));
    graph
        .nodes
        .insert(node_a, test_node(CanvasPoint { x: 0.0, y: 0.0 }));
    graph
        .nodes
        .insert(node_b, test_node(CanvasPoint { x: 10.0, y: 0.0 }));
    graph
        .nodes
        .insert(node_c, test_node(CanvasPoint { x: 20.0, y: 0.0 }));
    let draws = vec![
        NodeRectDraw {
            id: node_b,
            rect: Rect::new(
                Point::new(Px(10.0), Px(0.0)),
                fret_core::Size::new(Px(20.0), Px(20.0)),
            ),
        },
        NodeRectDraw {
            id: node_a,
            rect: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                fret_core::Size::new(Px(20.0), Px(20.0)),
            ),
        },
        NodeRectDraw {
            id: node_c,
            rect: Rect::new(
                Point::new(Px(20.0), Px(0.0)),
                fret_core::Size::new(Px(20.0), Px(20.0)),
            ),
        },
    ];
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(100.0)),
    );

    let infos = collect_portal_label_infos_for_visible_subset(
        &graph,
        Some(draws.as_slice()),
        bounds,
        PanZoom2D::default(),
        None,
        2,
        None,
        &[node_a],
        None,
    );

    assert_eq!(
        infos.iter().map(|info| info.id).collect::<Vec<_>>(),
        vec![node_b, node_a]
    );
    assert!(!infos[0].selected);
    assert!(infos[1].selected);
}

#[test]
fn declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let node = NodeId::from_u128(0x9201);
    let mut graph = Graph::new(GraphId::from_u128(0x9200));
    let mut value = test_node(CanvasPoint { x: 10.0, y: 20.0 });
    value.kind = NodeKindKey::new("test.portal.lifecycle.a");
    value.kind_version = 1;
    value.size = Some(CanvasSize {
        width: 140.0,
        height: 80.0,
    });
    graph.nodes.insert(node, value);

    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order: vec![node],
            ..NodeGraphViewState::default()
        },
        default_editor_config(),
    );

    let portal_props = |binding: &NodeGraphSurfaceBinding| {
        let mut props = binding.surface_props();
        props.cull_margin_screen_px = 0.0;
        props.portal_hosting = NodeGraphVisibleSubsetPortalConfig {
            enabled: true,
            max_nodes: 8,
        };
        props
    };

    let (_first, first_id) = render_until_surface_test_id_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        "node_graph.portal.node.0",
        portal_props,
    );

    let (_second, second_id) = render_until_surface_test_id_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        "node_graph.portal.node.0",
        portal_props,
    );
    assert_eq!(
        first_id, second_id,
        "portal subtree identity must persist across frames for the same node kind/version"
    );

    let mut version_changed = host
        .models
        .read(&binding.graph_model(), Clone::clone)
        .expect("graph readable");
    version_changed
        .nodes
        .get_mut(&node)
        .expect("node exists")
        .kind_version = 2;
    binding
        .replace_graph_action_host(&mut host, version_changed)
        .expect("replace graph");
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);

    let (_third, third_id) = render_until_surface_test_id_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        "node_graph.portal.node.0",
        portal_props,
    );
    assert_ne!(
        third_id, second_id,
        "portal subtree identity must reset when node kind_version changes"
    );

    let mut kind_changed = host
        .models
        .read(&binding.graph_model(), Clone::clone)
        .expect("graph readable");
    kind_changed.nodes.get_mut(&node).expect("node exists").kind =
        NodeKindKey::new("test.portal.lifecycle.b");
    binding
        .replace_graph_action_host(&mut host, kind_changed)
        .expect("replace graph");
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);

    let (_fourth, fourth_id) = render_until_surface_test_id_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        "node_graph.portal.node.0",
        portal_props,
    );
    assert_ne!(
        fourth_id, third_id,
        "portal subtree identity must reset when node kind changes"
    );
}

struct KindSwitchPortalRenderer {
    custom_kind: NodeKindKey,
    next_instance: Arc<AtomicUsize>,
    last_instance: Arc<AtomicUsize>,
}

impl NodeGraphDeclarativePortalRenderer<TestActionHostImpl> for KindSwitchPortalRenderer {
    fn render_portal(
        &mut self,
        cx: &mut fret_ui::ElementContext<'_, TestActionHostImpl>,
        graph: &Graph,
        layout: crate::ui::NodeGraphPortalNodeLayout,
    ) -> fret_ui::element::Elements {
        let Some(node) = graph.nodes.get(&layout.node) else {
            return Vec::new().into();
        };
        if node.kind != self.custom_kind {
            return Vec::new().into();
        }
        let instance = cx.root_state(
            || self.next_instance.fetch_add(1, Ordering::SeqCst),
            |state| *state,
        );
        self.last_instance.store(instance, Ordering::SeqCst);

        let mut props = fret_ui::element::SemanticsProps::default();
        props.label = Some(Arc::<str>::from("Custom portal body"));
        props.value = Some(Arc::<str>::from(format!(
            "node={}; window_x={:.1}; zoom={:.1}; instance={instance}",
            layout.node.0, layout.node_window.origin.x.0, layout.zoom,
        )));
        props.layout.size.width = fret_ui::element::Length::Px(Px(64.0));
        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));

        vec![cx.semantics(props, |_cx| Vec::new()).attach_semantics(
            fret_ui::element::SemanticsDecoration::default().test_id(Arc::<str>::from(format!(
                "node_graph.portal.custom.{}",
                layout.node.0
            ))),
        )]
        .into()
    }
}

#[derive(Debug, Clone, Copy)]
struct DeclarativeCommitMoveHandler {
    node: NodeId,
}

impl NodeGraphDeclarativePortalCommandHandler for DeclarativeCommitMoveHandler {
    fn handle_portal_command(
        &mut self,
        _host: &mut dyn fret_ui::action::UiFocusActionHost,
        _cx: fret_ui::action::ActionCx,
        _graph: &Graph,
        command: PortalTextCommand,
    ) -> PortalCommandOutcome {
        match command {
            PortalTextCommand::Submit { node } if node == self.node => {
                PortalCommandOutcome::Commit(crate::ops::GraphTransaction {
                    label: Some("Move Node".to_string()),
                    ops: vec![GraphOp::SetNodePos {
                        id: node,
                        from: CanvasPoint { x: 10.0, y: 20.0 },
                        to: CanvasPoint { x: 64.0, y: 32.0 },
                    }],
                })
            }
            _ => PortalCommandOutcome::NotHandled,
        }
    }
}

#[test]
fn declarative_portal_command_host_submits_transactions_without_retained_portal_host() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let node = NodeId::from_u128(0x9341);
    let mut graph = Graph::new(GraphId::from_u128(0x9340));
    graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 10.0, y: 20.0 }));

    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order: vec![node],
            ..NodeGraphViewState::default()
        },
        default_editor_config(),
    );
    let handler = Rc::new(RefCell::new(DeclarativeCommitMoveHandler { node }));
    let mut surface_element = None;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "node-graph-surface-portal-command",
        |cx| {
            let mut props = binding.surface_props();
            props.portal_command_handler = Some(handler.clone());
            let surface = node_graph_surface(cx, props);
            surface_element = Some(surface.id);
            vec![surface]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    assert!(
        surface_element.is_some(),
        "surface element should be captured during render"
    );

    assert!(ui.dispatch_command(&mut host, &mut services, &portal_submit_text_command(node),));

    let graph_pos = host
        .models
        .read(&binding.graph_model(), |graph| {
            graph.nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("graph node pos");
    assert_eq!(graph_pos, CanvasPoint { x: 64.0, y: 32.0 });

    let store_pos = host
        .models
        .read(&binding.store_model(), |store| {
            store.graph().nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("store node pos");
    assert_eq!(store_pos, CanvasPoint { x: 64.0, y: 32.0 });

    assert!(
        !ui.dispatch_command(
            &mut host,
            &mut services,
            &portal_submit_text_command(NodeId::from_u128(0x9342)),
        ),
        "unclaimed portal commands must keep bubbling instead of being swallowed"
    );
}

#[derive(Debug, Clone, Copy)]
struct DeclarativePortalTextMoveSpec {
    node: NodeId,
}

impl PortalTextEditSpec for DeclarativePortalTextMoveSpec {
    fn initial_text(&self, _graph: &Graph, _node: NodeId) -> String {
        "start".to_string()
    }

    fn submit(&self, _graph: &Graph, node: NodeId, text: &str) -> PortalTextEditSubmit {
        if node != self.node {
            return PortalTextEditSubmit::NotHandled;
        }

        match text {
            "moved" => PortalTextEditSubmit::Commit {
                tx: crate::ops::GraphTransaction {
                    label: Some("Text Portal Move".to_string()),
                    ops: vec![GraphOp::SetNodePos {
                        id: node,
                        from: CanvasPoint { x: 10.0, y: 20.0 },
                        to: CanvasPoint { x: 96.0, y: 48.0 },
                    }],
                },
                normalized_text: Some("moved".to_string()),
            },
            _ => PortalTextEditSubmit::Handled {
                normalized_text: Some(text.to_ascii_uppercase()),
            },
        }
    }

    fn step_text_with_mode(
        &self,
        _graph: &Graph,
        node: NodeId,
        text: &str,
        delta: i32,
        mode: PortalTextStepMode,
    ) -> Option<String> {
        (node == self.node && text == "start" && delta == 1 && mode == PortalTextStepMode::Coarse)
            .then(|| "moved".to_string())
    }
}

#[test]
fn declarative_portal_text_cancel_returns_focus_to_surface_without_graph_commit() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let node = NodeId::from_u128(0x9348);
    let original_pos = CanvasPoint { x: 10.0, y: 20.0 };
    let mut graph = Graph::new(GraphId::from_u128(0x9349));
    graph.nodes.insert(node, test_node(original_pos));

    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order: vec![node],
            ..NodeGraphViewState::default()
        },
        default_editor_config(),
    );
    let handler = Rc::new(RefCell::new(PortalTextEditHandler::new(
        "node-graph-surface-text-editor-cancel-command",
        DeclarativePortalTextMoveSpec { node },
    )));
    let mut surface_element = None;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "node-graph-surface-text-editor-cancel-command",
        |cx| {
            let mut props = binding.surface_props();
            props.portal_command_handler = Some(handler.clone());
            let surface = node_graph_surface(cx, props);
            surface_element = Some(surface.id);
            vec![surface]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    let surface_element = surface_element.expect("surface element should be captured");
    let surface_node = ui
        .live_attached_node_for_element(&mut host, surface_element)
        .expect("surface element should resolve to a live node");
    assert_ne!(ui.focus(), Some(surface_node));

    let cancel = portal_cancel_text_command(node);
    assert!(
        ui.is_command_available(&mut host, &cancel),
        "cancel must be available for a live declarative portal node"
    );
    assert!(ui.dispatch_command(&mut host, &mut services, &cancel));

    assert_eq!(
        ui.focus(),
        Some(surface_node),
        "handled portal cancel must restore focus to the graph surface"
    );

    let graph_pos = host
        .models
        .read(&binding.graph_model(), |graph| {
            graph.nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("graph node pos");
    assert_eq!(graph_pos, original_pos);

    let store_pos = host
        .models
        .read(&binding.store_model(), |store| {
            store.graph().nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("store node pos");
    assert_eq!(store_pos, original_pos);

    assert!(
        !ui.is_command_available(
            &mut host,
            &portal_cancel_text_command(NodeId::from_u128(0x9350)),
        ),
        "cancel commands for missing portal nodes must not become available"
    );
}

#[test]
fn declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let node = NodeId::from_u128(0x9343);
    let mut graph = Graph::new(GraphId::from_u128(0x9344));
    graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 10.0, y: 20.0 }));

    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order: vec![node],
            ..NodeGraphViewState::default()
        },
        default_editor_config(),
    );
    let handler = Rc::new(RefCell::new(PortalTextEditHandler::new(
        "node-graph-surface-text-editor-command",
        DeclarativePortalTextMoveSpec { node },
    )));

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "node-graph-surface-text-editor-command",
        |cx| {
            let mut props = binding.surface_props();
            props.portal_command_handler = Some(handler.clone());
            vec![node_graph_surface(cx, props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    assert!(ui.dispatch_command(
        &mut host,
        &mut services,
        &portal_step_text_command_with_mode(node, 1, PortalTextStepMode::Coarse),
    ));

    let graph_pos = host
        .models
        .read(&binding.graph_model(), |graph| {
            graph.nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("graph node pos");
    let store_pos = host
        .models
        .read(&binding.store_model(), |store| {
            store.graph().nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("store node pos");

    assert_eq!(graph_pos, CanvasPoint { x: 96.0, y: 48.0 });
    assert_eq!(store_pos, CanvasPoint { x: 96.0, y: 48.0 });

    assert!(
        !ui.dispatch_command(
            &mut host,
            &mut services,
            &portal_step_text_command_with_mode(
                NodeId::from_u128(0x9345),
                1,
                PortalTextStepMode::Coarse
            ),
        ),
        "commands for nodes outside the text edit spec should keep bubbling"
    );
}

#[derive(Debug, Clone, Copy)]
struct DeclarativePortalNumberMoveSpec {
    node: NodeId,
}

impl PortalNumberEditSpec for DeclarativePortalNumberMoveSpec {
    fn initial_value(&self, _graph: &Graph, node: NodeId) -> Option<f64> {
        (node == self.node).then_some(1.0)
    }

    fn format_value(&self, value: f64) -> String {
        format!("{value:.0}")
    }

    fn submit_value(
        &self,
        _graph: &Graph,
        node: NodeId,
        value: f64,
        _text: &str,
    ) -> PortalNumberEditSubmit {
        if node != self.node {
            return PortalNumberEditSubmit::NotHandled;
        }
        if value < 2.0 {
            return PortalNumberEditSubmit::Handled {
                normalized_text: Some(format!("{value:.0}")),
            };
        }

        PortalNumberEditSubmit::Commit {
            tx: crate::ops::GraphTransaction {
                label: Some("Number Portal Move".to_string()),
                ops: vec![GraphOp::SetNodePos {
                    id: node,
                    from: CanvasPoint { x: 10.0, y: 20.0 },
                    to: CanvasPoint { x: 128.0, y: 72.0 },
                }],
            },
            normalized_text: Some(format!("{value:.0}")),
        }
    }

    fn step_size(&self, _graph: &Graph, node: NodeId, mode: PortalTextStepMode) -> Option<f64> {
        (node == self.node && mode == PortalTextStepMode::Fine).then_some(0.5)
    }
}

#[test]
fn declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let node = NodeId::from_u128(0x9346);
    let mut graph = Graph::new(GraphId::from_u128(0x9347));
    graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 10.0, y: 20.0 }));

    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order: vec![node],
            ..NodeGraphViewState::default()
        },
        default_editor_config(),
    );
    let handler = Rc::new(RefCell::new(PortalNumberEditHandler::new(
        "node-graph-surface-number-editor-command",
        DeclarativePortalNumberMoveSpec { node },
    )));

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "node-graph-surface-number-editor-command",
        |cx| {
            let mut props = binding.surface_props();
            props.portal_command_handler = Some(handler.clone());
            vec![node_graph_surface(cx, props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    assert!(ui.dispatch_command(
        &mut host,
        &mut services,
        &portal_step_text_command_with_mode(node, 2, PortalTextStepMode::Fine),
    ));

    let graph_pos = host
        .models
        .read(&binding.graph_model(), |graph| {
            graph.nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("graph node pos");
    let store_pos = host
        .models
        .read(&binding.store_model(), |store| {
            store.graph().nodes.get(&node).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("store node pos");

    assert_eq!(graph_pos, CanvasPoint { x: 128.0, y: 72.0 });
    assert_eq!(store_pos, CanvasPoint { x: 128.0, y: 72.0 });
}

#[test]
fn declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let custom_kind = NodeKindKey::new("test.portal.custom");
    let custom_node = NodeId::from_u128(0x9301);
    let fallback_node = NodeId::from_u128(0x9302);
    let mut graph = Graph::new(GraphId::from_u128(0x9300));
    let mut custom = test_node(CanvasPoint { x: 10.0, y: 20.0 });
    custom.kind = custom_kind.clone();
    custom.size = Some(CanvasSize {
        width: 140.0,
        height: 80.0,
    });
    graph.nodes.insert(custom_node, custom);
    let mut fallback = test_node(CanvasPoint { x: 240.0, y: 20.0 });
    fallback.kind = NodeKindKey::new("test.portal.fallback");
    fallback.size = Some(CanvasSize {
        width: 140.0,
        height: 80.0,
    });
    graph.nodes.insert(fallback_node, fallback);

    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order: vec![custom_node, fallback_node],
            ..NodeGraphViewState::default()
        },
        default_editor_config(),
    );
    let portal_props = |binding: &NodeGraphSurfaceBinding| {
        let mut props = binding.surface_props();
        props.cull_margin_screen_px = 0.0;
        props.portal_hosting = NodeGraphVisibleSubsetPortalConfig {
            enabled: true,
            max_nodes: 8,
        };
        props
    };
    let next_instance = Arc::new(AtomicUsize::new(0));
    let last_instance = Arc::new(AtomicUsize::new(usize::MAX));
    let mut renderer = KindSwitchPortalRenderer {
        custom_kind: custom_kind.clone(),
        next_instance: next_instance.clone(),
        last_instance: last_instance.clone(),
    };

    let custom_test_id = format!("node_graph.portal.custom.{}", custom_node.0);
    let mut snapshot = None;
    for _ in 0..4 {
        let next = render_surface_frame_with_portal_renderer_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            portal_props,
            &mut renderer,
        );
        if next
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(custom_test_id.as_str()))
        {
            snapshot = Some(next);
            break;
        }
        snapshot = Some(next);
    }
    let snapshot = snapshot.expect("surface frame rendered");
    let first_instance = last_instance.load(Ordering::SeqCst);
    let custom = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(custom_test_id.as_str()))
        .expect("custom portal subtree must be hosted");
    assert_eq!(custom.label.as_deref(), Some("Custom portal body"));
    assert!(
        custom
            .value
            .as_deref()
            .is_some_and(|value| value.contains("zoom=1.0")),
        "custom renderer should receive screen-space portal layout"
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some("node_graph.portal.node.1")),
        "empty renderer output must fall back to the default portal label"
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .all(|node| node.test_id.as_deref() != Some("node_graph.portal.node.0")),
        "custom subtree should replace the default label for the matching node kind"
    );

    let _second = render_surface_frame_with_portal_renderer_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        portal_props,
        &mut renderer,
    );
    assert_eq!(
        first_instance,
        last_instance.load(Ordering::SeqCst),
        "custom portal renderer state should persist for the same node kind/version"
    );

    let mut changed = host
        .models
        .read(&binding.graph_model(), Clone::clone)
        .expect("graph readable");
    changed
        .nodes
        .get_mut(&custom_node)
        .expect("custom node exists")
        .kind_version = 2;
    binding
        .replace_graph_action_host(&mut host, changed)
        .expect("replace graph");
    let changed_models = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed_models);

    let _third = render_surface_frame_with_portal_renderer_for_binding(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        &binding,
        portal_props,
        &mut renderer,
    );
    assert_ne!(
        first_instance,
        last_instance.load(Ordering::SeqCst),
        "custom portal renderer state should reset when node kind_version changes"
    );
}

#[test]
fn declarative_surface_hosts_node_type_registry_without_retained_portal_host() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let registered_kind = NodeKindKey::new("test.portal.registry");
    let registered_node = NodeId::from_u128(0x9311);
    let fallback_node = NodeId::from_u128(0x9312);
    let mut graph = Graph::new(GraphId::from_u128(0x9310));
    let mut registered = test_node(CanvasPoint { x: 16.0, y: 24.0 });
    registered.kind = registered_kind.clone();
    registered.size = Some(CanvasSize {
        width: 140.0,
        height: 80.0,
    });
    graph.nodes.insert(registered_node, registered);
    let mut fallback = test_node(CanvasPoint { x: 220.0, y: 24.0 });
    fallback.kind = NodeKindKey::new("test.portal.registry.fallback");
    fallback.size = Some(CanvasSize {
        width: 140.0,
        height: 80.0,
    });
    graph.nodes.insert(fallback_node, fallback);

    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order: vec![registered_node, fallback_node],
            ..NodeGraphViewState::default()
        },
        default_editor_config(),
    );
    let portal_props = |binding: &NodeGraphSurfaceBinding| {
        let mut props = binding.surface_props();
        props.cull_margin_screen_px = 0.0;
        props.portal_hosting = NodeGraphVisibleSubsetPortalConfig {
            enabled: true,
            max_nodes: 8,
        };
        props
    };
    let mut registry = NodeGraphNodeTypes::new()
        .register(registered_kind, |cx, _graph, layout| {
            let mut props = fret_ui::element::SemanticsProps {
                label: Some(Arc::<str>::from("Registered portal node")),
                ..Default::default()
            };
            props.layout.size.width = fret_ui::element::Length::Px(Px(52.0));
            props.layout.size.height = fret_ui::element::Length::Px(Px(20.0));
            vec![cx.semantics(props, |_cx| Vec::new()).attach_semantics(
                fret_ui::element::SemanticsDecoration::default().test_id(Arc::<str>::from(
                    format!("node_graph.portal.registry.{}", layout.node.0),
                )),
            )]
        })
        .with_fallback(|cx, _graph, layout| {
            let mut props = fret_ui::element::SemanticsProps {
                label: Some(Arc::<str>::from("Fallback portal node")),
                ..Default::default()
            };
            props.layout.size.width = fret_ui::element::Length::Px(Px(44.0));
            props.layout.size.height = fret_ui::element::Length::Px(Px(20.0));
            vec![cx.semantics(props, |_cx| Vec::new()).attach_semantics(
                fret_ui::element::SemanticsDecoration::default().test_id(Arc::<str>::from(
                    format!("node_graph.portal.registry.fallback.{}", layout.node.0),
                )),
            )]
        });

    let registered_test_id = format!("node_graph.portal.registry.{}", registered_node.0);
    let fallback_test_id = format!("node_graph.portal.registry.fallback.{}", fallback_node.0);
    let mut snapshot = None;
    for _ in 0..4 {
        let next = render_surface_frame_with_portal_renderer_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            portal_props,
            &mut registry,
        );
        let has_registered = next
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(registered_test_id.as_str()));
        let has_fallback = next
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(fallback_test_id.as_str()));
        if has_registered && has_fallback {
            snapshot = Some(next);
            break;
        }
        snapshot = Some(next);
    }
    let snapshot = snapshot.expect("surface frame rendered");

    assert!(
        snapshot.nodes.iter().any(|node| node.test_id.as_deref()
            == Some(registered_test_id.as_str())
            && node.label.as_deref() == Some("Registered portal node")),
        "registered node kind should use its renderer"
    );
    assert!(
        snapshot.nodes.iter().any(|node| node.test_id.as_deref()
            == Some(fallback_test_id.as_str())
            && node.label.as_deref() == Some("Fallback portal node")),
        "unregistered node kind should use the registry fallback renderer"
    );
}

#[test]
fn declarative_portal_renderer_publishes_custom_subtree_measurements() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let custom_kind = NodeKindKey::new("test.portal.measured");
    let measured_node = NodeId::from_u128(0x9321);
    let mut graph = Graph::new(GraphId::from_u128(0x9320));
    let mut node = test_node(CanvasPoint { x: 20.0, y: 28.0 });
    node.kind = custom_kind.clone();
    graph.nodes.insert(measured_node, node);

    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order: vec![measured_node],
            ..NodeGraphViewState::default()
        },
        default_editor_config(),
    );
    let measured_geometry = Arc::new(MeasuredGeometryStore::new());
    let portal_props = |binding: &NodeGraphSurfaceBinding| {
        let mut props = binding.surface_props();
        props.cull_margin_screen_px = 0.0;
        props.measured_geometry = Some(measured_geometry.clone());
        props.portal_hosting = NodeGraphVisibleSubsetPortalConfig {
            enabled: true,
            max_nodes: 4,
        };
        props
    };
    let mut registry = NodeGraphNodeTypes::new().register(custom_kind, |cx, _graph, layout| {
        let mut props = fret_ui::element::SemanticsProps {
            label: Some(Arc::<str>::from("Measured portal node")),
            ..Default::default()
        };
        props.layout.size.width = fret_ui::element::Length::Px(Px(360.0));
        props.layout.size.height = fret_ui::element::Length::Px(Px(220.0));
        vec![cx.semantics(props, |_cx| Vec::new()).attach_semantics(
            fret_ui::element::SemanticsDecoration::default().test_id(Arc::<str>::from(format!(
                "node_graph.portal.measured.{}",
                layout.node.0
            ))),
        )]
    });

    for _ in 0..6 {
        let _ = render_surface_frame_with_portal_renderer_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            portal_props,
            &mut registry,
        );
        if measured_geometry.node_size_px(measured_node).is_some() {
            break;
        }
    }

    let measured = measured_geometry
        .node_size_px(measured_node)
        .expect("custom declarative portal subtree should publish a measured node size");
    assert!(
        measured.0 >= 360.0 && measured.1 >= 220.0,
        "custom portal measurement should use the rendered subtree bounds, got {measured:?}"
    );
}

#[test]
fn sync_portal_canvas_bounds_in_models_ignores_epsilon_churn() {
    let mut host = TestActionHostImpl::default();
    let node = NodeId::from_u128(9121);
    let portal_bounds = host.models.insert(PortalBoundsStore::default());
    let initial = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        fret_core::Size::new(Px(30.0), Px(40.0)),
    );
    assert!(sync_portal_canvas_bounds_in_models(
        &mut host.models,
        &portal_bounds,
        node,
        initial,
    ));

    let near = Rect::new(
        Point::new(Px(10.1), Px(20.1)),
        fret_core::Size::new(Px(30.1), Px(40.1)),
    );
    assert!(!sync_portal_canvas_bounds_in_models(
        &mut host.models,
        &portal_bounds,
        node,
        near,
    ));
    assert!(sync_portal_canvas_bounds_in_models(
        &mut host.models,
        &portal_bounds,
        node,
        Rect::new(
            Point::new(Px(12.0), Px(24.0)),
            fret_core::Size::new(Px(30.0), Px(40.0)),
        ),
    ));
}

#[test]
fn sync_hover_anchor_store_in_models_tracks_dragged_hovered_node_rect() {
    let mut models = ModelStore::default();
    let hover_anchor = models.insert(HoverAnchorStore::default());
    let node = NodeId::from_u128(9407);
    let draws = vec![NodeRectDraw {
        id: node,
        rect: Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            fret_core::Size::new(Px(120.0), Px(60.0)),
        ),
    }];
    let drag = NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(40.0), Px(-20.0)),
        phase: NodeDragPhase::Active,
        nodes_sorted: Arc::from([node]),
    };
    let view = PanZoom2D {
        pan: Point::new(Px(0.0), Px(0.0)),
        zoom: 2.0,
    };

    assert!(sync_hover_anchor_store_in_models(
        &mut models,
        &hover_anchor,
        Some(node),
        Some(draws.as_slice()),
        view,
        Some(&drag),
    ));

    let stored = models.read(&hover_anchor, |st| st.clone()).unwrap();
    assert_eq!(stored.hovered_id, Some(node));
    assert_eq!(
        stored.hovered_canvas_bounds,
        Some(Rect::new(
            Point::new(Px(30.0), Px(10.0)),
            fret_core::Size::new(Px(120.0), Px(60.0)),
        ))
    );
}

#[test]
fn declarative_hover_tooltip_overlay_tracks_dragged_anchor_when_portals_disabled() {
    let mut models = ModelStore::default();
    let hover_anchor = models.insert(HoverAnchorStore::default());
    let node = NodeId::from_u128(9411);
    let draws = vec![NodeRectDraw {
        id: node,
        rect: Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            fret_core::Size::new(Px(120.0), Px(60.0)),
        ),
    }];
    let drag = NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(40.0), Px(-20.0)),
        phase: NodeDragPhase::Active,
        nodes_sorted: Arc::from([node]),
    };
    let view = PanZoom2D {
        pan: Point::new(Px(0.0), Px(0.0)),
        zoom: 2.0,
    };
    let bounds = Rect::new(
        Point::new(Px(100.0), Px(200.0)),
        fret_core::Size::new(Px(800.0), Px(600.0)),
    );

    assert!(sync_hover_anchor_store_in_models(
        &mut models,
        &hover_anchor,
        Some(node),
        Some(draws.as_slice()),
        view,
        Some(&drag),
    ));
    let stored = models.read(&hover_anchor, |st| st.clone()).unwrap();
    let anchor =
        resolve_hover_tooltip_anchor(bounds, view, true, None, stored.hovered_canvas_bounds)
            .expect("hover anchor should resolve when portals are disabled");
    let spec = build_hover_tooltip_overlay_spec(
        bounds,
        node,
        anchor,
        false,
        Arc::<str>::from("dragged"),
        1,
        2,
    )
    .expect("tooltip spec");

    assert_eq!(anchor.source, HoverTooltipAnchorSource::HoverAnchorStore);
    assert_eq!(anchor.origin_screen, Point::new(Px(160.0), Px(220.0)));
    assert_eq!(spec.left, Px(60.0));
    assert_eq!(
        spec.top,
        Px(26.0),
        "tooltip should flip below the drag-adjusted anchor near the top edge"
    );
    assert_eq!(spec.width, Px(240.0));
    assert!(!spec.hide_label_summary);
}

#[test]
fn build_hover_tooltip_overlay_spec_flips_below_anchor_when_needed() {
    let bounds = Rect::new(
        Point::new(Px(100.0), Px(200.0)),
        fret_core::Size::new(Px(800.0), Px(600.0)),
    );
    let spec = build_hover_tooltip_overlay_spec(
        bounds,
        NodeId::from_u128(9410),
        super::hover_anchor::HoverTooltipAnchor {
            origin_screen: Point::new(Px(120.0), Px(205.0)),
            width_screen: Px(240.0),
            source: HoverTooltipAnchorSource::PortalBoundsStore,
        },
        true,
        Arc::<str>::from("node"),
        2,
        3,
    )
    .expect("spec");

    assert_eq!(spec.left, Px(20.0));
    assert_eq!(spec.top, Px(11.0));
    assert_eq!(spec.width, Px(240.0));
    assert!(spec.hide_label_summary);
}

#[test]
fn declarative_overlay_layer_is_input_transparent_over_canvas_region() {
    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let mut services = FakeUiServices;
    let window = AppWindowId::default();
    let bounds = test_node_graph_surface_bounds();
    ui.set_window(window);
    host.bounds = bounds;

    let canvas_downs = Arc::new(AtomicUsize::new(0));
    let overlay_downs = Arc::new(AtomicUsize::new(0));
    let canvas_downs_hook = canvas_downs.clone();
    let overlay_downs_hook = overlay_downs.clone();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "node-graph-declarative-overlay-input-transparency",
        |cx| {
            let mut canvas_region = PointerRegionProps::default();
            canvas_region.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            let canvas = cx.pointer_region(canvas_region, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(move |_host, _cx, _down| {
                    canvas_downs_hook.fetch_add(1, Ordering::Relaxed);
                    true
                }));
                Vec::new()
            });

            let mut overlay_children = Vec::new();
            let mut overlay_region = PointerRegionProps::default();
            overlay_region.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            overlay_children.push(cx.pointer_region(overlay_region, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(move |_host, _cx, _down| {
                    overlay_downs_hook.fetch_add(1, Ordering::Relaxed);
                    true
                }));
                Vec::new()
            }));

            let mut out = vec![canvas];
            push_overlay_layer_if_needed(cx, &mut out, overlay_children);
            out
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    let position = Point::new(Px(320.0), Px(240.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId::default(),
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    assert_eq!(
        canvas_downs.load(Ordering::Relaxed),
        1,
        "declarative overlay layer should pass pointer input through to the canvas region"
    );
    assert_eq!(
        overlay_downs.load(Ordering::Relaxed),
        0,
        "diagnostics-only declarative overlays must not become interactive hit-test roots"
    );
}

#[test]
fn clamp_marquee_overlay_rect_to_bounds_clamps_and_rejects_empty_rects() {
    let bounds = Rect::new(
        Point::new(Px(100.0), Px(100.0)),
        fret_core::Size::new(Px(200.0), Px(160.0)),
    );
    let clamped = clamp_marquee_overlay_rect_to_bounds(
        bounds,
        Rect::new(
            Point::new(Px(50.0), Px(80.0)),
            fret_core::Size::new(Px(180.0), Px(90.0)),
        ),
    )
    .expect("clamped");
    assert_eq!(
        clamped,
        Rect::new(
            Point::new(Px(100.0), Px(100.0)),
            fret_core::Size::new(Px(130.0), Px(70.0)),
        )
    );
    assert_eq!(
        clamp_marquee_overlay_rect_to_bounds(
            bounds,
            Rect::new(
                Point::new(Px(10.0), Px(10.0)),
                fret_core::Size::new(Px(20.0), Px(20.0)),
            ),
        ),
        None
    );
}

#[test]
fn resolve_hover_tooltip_anchor_prefers_dragged_portal_bounds_over_stale_hover_anchor() {
    let node = NodeId::from_u128(9408);
    let draws = vec![NodeRectDraw {
        id: node,
        rect: Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            fret_core::Size::new(Px(120.0), Px(60.0)),
        ),
    }];
    let drag = NodeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(40.0), Px(-20.0)),
        phase: NodeDragPhase::Active,
        nodes_sorted: Arc::from([node]),
    };
    let bounds = Rect::new(
        Point::new(Px(100.0), Px(200.0)),
        fret_core::Size::new(Px(800.0), Px(600.0)),
    );
    let view = PanZoom2D {
        pan: Point::new(Px(0.0), Px(0.0)),
        zoom: 2.0,
    };
    let dragged_portal =
        hovered_canvas_anchor_rect_for_surface(node, Some(draws.as_slice()), view, Some(&drag))
            .expect("dragged rect");
    let stale_hover = draws[0].rect;

    let anchor =
        resolve_hover_tooltip_anchor(bounds, view, false, Some(dragged_portal), Some(stale_hover))
            .expect("anchor resolved");

    assert_eq!(anchor.source, HoverTooltipAnchorSource::PortalBoundsStore);
    assert_eq!(
        anchor.origin_screen,
        view.canvas_to_screen(bounds, dragged_portal.origin)
    );
    assert_eq!(anchor.width_screen, Px(240.0));
}

#[test]
fn resolve_hover_tooltip_anchor_prefers_portal_bounds_when_available() {
    let bounds = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        fret_core::Size::new(Px(800.0), Px(600.0)),
    );
    let view = PanZoom2D {
        pan: Point::new(Px(0.0), Px(0.0)),
        zoom: 2.0,
    };
    let portal = Rect::new(
        Point::new(Px(30.0), Px(40.0)),
        fret_core::Size::new(Px(120.0), Px(60.0)),
    );
    let hover = Rect::new(
        Point::new(Px(100.0), Px(200.0)),
        fret_core::Size::new(Px(80.0), Px(50.0)),
    );

    let anchor = resolve_hover_tooltip_anchor(bounds, view, false, Some(portal), Some(hover))
        .expect("anchor resolved");

    assert_eq!(anchor.source, HoverTooltipAnchorSource::PortalBoundsStore);
    assert_eq!(
        anchor.origin_screen,
        view.canvas_to_screen(bounds, portal.origin)
    );
    assert_eq!(anchor.width_screen, Px(240.0));
}

#[test]
fn resolve_hover_tooltip_anchor_falls_back_to_hover_anchor_when_portals_disabled() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(640.0), Px(480.0)),
    );
    let view = PanZoom2D {
        pan: Point::new(Px(16.0), Px(-8.0)),
        zoom: 1.5,
    };
    let portal = Rect::new(
        Point::new(Px(30.0), Px(40.0)),
        fret_core::Size::new(Px(120.0), Px(60.0)),
    );
    let hover = Rect::new(
        Point::new(Px(22.0), Px(18.0)),
        fret_core::Size::new(Px(40.0), Px(30.0)),
    );

    let anchor = resolve_hover_tooltip_anchor(bounds, view, true, Some(portal), Some(hover))
        .expect("anchor resolved");

    assert_eq!(anchor.source, HoverTooltipAnchorSource::HoverAnchorStore);
    assert_eq!(
        anchor.origin_screen,
        view.canvas_to_screen(bounds, hover.origin)
    );
    assert_eq!(anchor.width_screen, Px(60.0));
}

#[test]
fn resolve_hover_tooltip_anchor_rejects_non_positive_width() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(640.0), Px(480.0)),
    );
    let view = PanZoom2D::default();
    let hover = Rect::new(
        Point::new(Px(22.0), Px(18.0)),
        fret_core::Size::new(Px(0.0), Px(30.0)),
    );

    assert_eq!(
        resolve_hover_tooltip_anchor(bounds, view, true, None, Some(hover)),
        None
    );
}

#[test]
fn derived_geometry_cache_key_changes_when_presenter_revision_changes() {
    let node = NodeId::from_u128(9401);
    let view_state = NodeGraphViewState {
        draw_order: vec![node],
        ..NodeGraphViewState::default()
    };
    let editor_config = NodeGraphEditorConfig::default();
    let interaction = editor_config.resolved_interaction_state();
    let node_origin = editor_config.interaction.node_origin;
    let style = crate::ui::style::NodeGraphStyle::default();

    let derived_a = derived_geometry_cache_key(
        91,
        view_state.zoom,
        node_origin,
        &view_state.draw_order,
        &interaction,
        &style,
        7,
        0,
        0,
        0.0,
    );
    let derived_b = derived_geometry_cache_key(
        91,
        view_state.zoom,
        node_origin,
        &view_state.draw_order,
        &interaction,
        &style,
        8,
        0,
        0,
        0.0,
    );

    assert_ne!(derived_a, derived_b);
}

#[test]
fn derived_geometry_cache_key_changes_when_edge_types_revision_changes() {
    let node = NodeId::from_u128(9411);
    let view_state = NodeGraphViewState {
        draw_order: vec![node],
        ..NodeGraphViewState::default()
    };
    let editor_config = NodeGraphEditorConfig::default();
    let interaction = editor_config.resolved_interaction_state();
    let node_origin = editor_config.interaction.node_origin;
    let style = crate::ui::style::NodeGraphStyle::default();

    let derived_a = derived_geometry_cache_key(
        92,
        view_state.zoom,
        node_origin,
        &view_state.draw_order,
        &interaction,
        &style,
        0,
        7,
        0,
        0.0,
    );
    let derived_b = derived_geometry_cache_key(
        92,
        view_state.zoom,
        node_origin,
        &view_state.draw_order,
        &interaction,
        &style,
        0,
        8,
        0,
        0.0,
    );

    assert_ne!(derived_a, derived_b);
}

#[test]
fn custom_edge_path_spatial_rect_overrides_feed_edge_index_candidates() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let style = crate::ui::style::NodeGraphStyle::default();
    let far = Point::new(Px(500.0), Px(500.0));
    let edge_types = NodeGraphEdgeTypes::new().register_path(
        EdgeTypeKey::new("data"),
        move |_graph, _edge_id, _style, _hint, input| {
            Some(EdgeCustomPath {
                cache_key: 88,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::LineTo(far),
                    PathCommand::LineTo(input.to),
                ],
            })
        },
    );
    let interaction = NodeGraphEditorConfig::default().resolved_interaction_state();
    let overrides = build_edge_spatial_rect_overrides(
        &graph,
        1.0,
        &geom,
        &interaction,
        &style,
        Some(&edge_types),
    );
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].0, edge);

    let mut spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
    let mut scratch = Vec::new();
    assert!(
        !spatial
            .query_edges_sorted_dedup(far, 1.0, &mut scratch)
            .contains(&edge),
        "default Bezier spatial AABB should not include the custom path excursion"
    );

    for (edge_id, rect) in overrides {
        spatial.update_edge_rect(edge_id, rect);
    }
    scratch.clear();
    assert!(
        spatial
            .query_edges_sorted_dedup(far, 1.0, &mut scratch)
            .contains(&edge),
        "custom edge path conservative AABB should feed the spatial index candidate set"
    );
}

#[test]
fn custom_edge_path_hit_testing_uses_exact_path_distance_after_spatial_candidate() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let style = crate::ui::style::NodeGraphStyle::default();
    let edge_types = NodeGraphEdgeTypes::new().register_path(
        EdgeTypeKey::new("data"),
        move |_graph, _edge_id, _style, _hint, input| {
            let corner_x = Point::new(Px(input.from.x.0 + 320.0), input.from.y);
            let corner_y = Point::new(corner_x.x, Px(input.from.y.0 + 280.0));
            let return_x = Point::new(input.to.x, corner_y.y);
            Some(EdgeCustomPath {
                cache_key: 89,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::LineTo(corner_x),
                    PathCommand::LineTo(corner_y),
                    PathCommand::LineTo(return_x),
                    PathCommand::LineTo(input.to),
                ],
            })
        },
    );
    let edge_value = graph.edges.get(&edge).expect("edge present");
    let from = geom.port_center(edge_value.from).expect("from port center");
    let hit = Point::new(Px(from.x.0 + 320.0), Px(from.y.0 + 180.0));
    let miss_inside_custom_aabb = Point::new(Px(from.x.0 + 220.0), Px(from.y.0 + 180.0));

    let mut spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
    let mut interaction = NodeGraphEditorConfig::default().resolved_interaction_state();
    interaction.edge_interaction_width = 12.0;
    interaction.bezier_hit_test_steps = 24;
    for (edge_id, rect) in build_edge_spatial_rect_overrides(
        &graph,
        1.0,
        &geom,
        &interaction,
        &style,
        Some(&edge_types),
    ) {
        spatial.update_edge_rect(edge_id, rect);
    }
    let mut scratch = Vec::new();
    assert!(
        spatial
            .query_edges_sorted_dedup(miss_inside_custom_aabb, 1.0, &mut scratch)
            .contains(&edge),
        "miss point should still be a coarse spatial candidate so the exact path filter is tested"
    );

    assert_eq!(
        hit_test_edge_at_canvas_point(
            &graph,
            1.0,
            &geom,
            &spatial,
            &interaction,
            &style,
            Some(&edge_types),
            hit,
            &mut scratch,
        )
        .map(|hit| hit.edge),
        Some(edge)
    );
    assert_eq!(
        hit_test_edge_at_canvas_point(
            &graph,
            1.0,
            &geom,
            &spatial,
            &interaction,
            &style,
            Some(&edge_types),
            miss_inside_custom_aabb,
            &mut scratch,
        ),
        None,
        "coarse AABB candidates must still be rejected when the point is outside the custom path interaction width"
    );
}

#[test]
fn custom_edge_path_click_selects_edge_via_default_declarative_pointer_down_path() {
    let mut host = TestActionHostImpl::default();
    let action_cx = test_action_cx();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(960.0), Px(720.0)),
    );
    host.bounds = bounds;

    let (graph_value, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph_value, &draw_order);
    let style = crate::ui::style::NodeGraphStyle::default();
    let edge_types = Rc::new(NodeGraphEdgeTypes::new().register_path(
        EdgeTypeKey::new("data"),
        move |_graph, _edge_id, _style, _hint, input| {
            let corner_x = Point::new(Px(input.from.x.0 + 320.0), input.from.y);
            let corner_y = Point::new(corner_x.x, Px(input.from.y.0 + 280.0));
            let return_x = Point::new(input.to.x, corner_y.y);
            Some(EdgeCustomPath {
                cache_key: 90,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::LineTo(corner_x),
                    PathCommand::LineTo(corner_y),
                    PathCommand::LineTo(return_x),
                    PathCommand::LineTo(input.to),
                ],
            })
        },
    ));
    let edge_value = graph_value.edges.get(&edge).expect("edge present");
    let from = geom.port_center(edge_value.from).expect("from port center");
    let hit = Point::new(Px(from.x.0 + 320.0), Px(from.y.0 + 180.0));

    let mut editor_config = default_editor_config();
    editor_config.interaction.edge_interaction_width = 12.0;
    editor_config.interaction.bezier_hit_test_steps = 24;
    let interaction = editor_config.resolved_interaction_state();
    let mut spatial =
        crate::ui::canvas::CanvasSpatialDerived::build(&graph_value, &geom, 1.0, 0.0, 64.0);
    for (edge_id, rect) in build_edge_spatial_rect_overrides(
        &graph_value,
        1.0,
        &geom,
        &interaction,
        &style,
        Some(edge_types.as_ref()),
    ) {
        spatial.update_edge_rect(edge_id, rect);
    }
    let derived_cache = host.models.insert(DerivedGeometryCacheState {
        key: None,
        rebuilds: 1,
        geom: Some(Arc::new(geom)),
        index: Some(Arc::new(spatial)),
    });

    let stale_edge = EdgeId::from_u128(0xBAD);
    let group = GroupId::from_u128(0xBEEF);
    let view_value = NodeGraphViewState {
        selected_nodes: vec![draw_order[0]],
        selected_edges: vec![stale_edge],
        selected_groups: vec![group],
        ..Default::default()
    };
    let graph = host.models.insert(graph_value.clone());
    let view_state = host.models.insert(view_value.clone());
    let store = host
        .models
        .insert(NodeGraphStore::new(graph_value, view_value, editor_config));
    let controller = NodeGraphController::new(store.clone());
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let handler = build_pointer_down_handler(PointerDownHandlerParams {
        focus_target: action_cx.target,
        pan_button: MouseButton::Middle,
        drag: host.models.insert(None::<DragState>),
        marquee_drag: host.models.insert(None::<MarqueeDragState>),
        node_drag: host.models.insert(None::<NodeDragState>),
        pending_selection: host.models.insert(None::<PendingSelectionState>),
        binding: binding.clone(),
        grid_cache: host.models.insert(GridPaintCacheState::default()),
        derived_cache,
        hovered_node: host.models.insert(None::<NodeId>),
        hit_scratch: host.models.insert(Vec::<NodeId>::new()),
        style_tokens: style,
        edge_types: Some(edge_types),
    });

    assert!(handler(
        &mut host,
        action_cx,
        test_pointer_down(MouseButton::Left, hit, Modifiers::default()),
    ));

    let selection = host
        .models
        .read(&store, |store| {
            (
                store.view_state().selected_nodes.clone(),
                store.view_state().selected_edges.clone(),
                store.view_state().selected_groups.clone(),
            )
        })
        .expect("store readable");
    assert!(selection.0.is_empty());
    assert_eq!(selection.1, vec![edge]);
    assert!(selection.2.is_empty());
    assert_eq!(host.capture_pointer_count, 0);
    assert_eq!(host.requested_focus, vec![action_cx.target]);
    assert_eq!(host.notifications, vec![action_cx]);
    assert_eq!(host.redraw_requests, vec![action_cx.window]);
}

fn polyline_midpoint(points: &[Point]) -> Point {
    let mut total = 0.0f32;
    for window in points.windows(2) {
        let dx = window[1].x.0 - window[0].x.0;
        let dy = window[1].y.0 - window[0].y.0;
        total += (dx * dx + dy * dy).sqrt();
    }

    let mut remaining = total * 0.5;
    for window in points.windows(2) {
        let from = window[0];
        let to = window[1];
        let dx = to.x.0 - from.x.0;
        let dy = to.y.0 - from.y.0;
        let len = (dx * dx + dy * dy).sqrt();
        if len <= 1.0e-6 {
            continue;
        }
        if remaining <= len {
            let t = (remaining / len).clamp(0.0, 1.0);
            return Point::new(Px(from.x.0 + dx * t), Px(from.y.0 + dy * t));
        }
        remaining -= len;
    }

    *points
        .last()
        .expect("polyline should contain at least one point")
}

fn assert_point_near(actual: Point, expected: Point, epsilon: f32) {
    assert!(
        (actual.x.0 - expected.x.0).abs() <= epsilon
            && (actual.y.0 - expected.y.0).abs() <= epsilon,
        "expected point near ({:.3}, {:.3}), got ({:.3}, {:.3})",
        expected.x.0,
        expected.y.0,
        actual.x.0,
        actual.y.0
    );
}

fn assert_rect_near(actual: Rect, expected: Rect, epsilon: f32) {
    assert_point_near(actual.origin, expected.origin, epsilon);
    assert!(
        (actual.size.width.0 - expected.size.width.0).abs() <= epsilon
            && (actual.size.height.0 - expected.size.height.0).abs() <= epsilon,
        "expected rect size near ({:.3}, {:.3}), got ({:.3}, {:.3})",
        expected.size.width.0,
        expected.size.height.0,
        actual.size.width.0,
        actual.size.height.0
    );
}

#[test]
fn custom_edge_path_feeds_default_declarative_edge_center_anchor() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let edge_ref = graph.edges.get(&edge).expect("test edge");
    let from = geom.port_center(edge_ref.from).expect("from port center");
    let to = geom.port_center(edge_ref.to).expect("to port center");
    let detour_y = from.y.0 + 240.0;
    let custom_points = [
        from,
        Point::new(from.x, Px(detour_y)),
        Point::new(to.x, Px(detour_y)),
        to,
    ];
    let expected_center = polyline_midpoint(&custom_points);
    let edge_types = Rc::new(NodeGraphEdgeTypes::new().register_path(
        EdgeTypeKey::new("data"),
        move |_graph, _edge_id, _style, _hint, input| {
            let detour_y = input.from.y.0 + 240.0;
            Some(EdgeCustomPath {
                cache_key: 98,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::LineTo(Point::new(input.from.x, Px(detour_y))),
                    PathCommand::LineTo(Point::new(input.to.x, Px(detour_y))),
                    PathCommand::LineTo(input.to),
                ],
            })
        },
    ));

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            selected_edges: vec![edge],
            draw_order,
            ..Default::default()
        },
        default_editor_config(),
    );

    for _ in 0..2 {
        let edge_types = edge_types.clone();
        let _ = render_surface_frame_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            move |binding| {
                let mut props = super::NodeGraphSurfaceProps::new(binding.clone());
                props.edge_types = Some(edge_types);
                props
            },
        );
    }

    let snapshot = binding.internals_store().snapshot();
    let center = snapshot
        .edge_centers_window
        .get(&edge)
        .copied()
        .expect("edge center anchor");

    assert_point_near(center, expected_center, 0.5);
}

#[test]
fn custom_edge_path_feeds_declarative_edge_toolbar_composition_anchor() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let edge_ref = graph.edges.get(&edge).expect("test edge");
    let from = geom.port_center(edge_ref.from).expect("from port center");
    let to = geom.port_center(edge_ref.to).expect("to port center");
    let detour_y = from.y.0 + 240.0;
    let custom_points = [
        from,
        Point::new(from.x, Px(detour_y)),
        Point::new(to.x, Px(detour_y)),
        to,
    ];
    let expected_center = polyline_midpoint(&custom_points);
    let edge_types = Rc::new(NodeGraphEdgeTypes::new().register_path(
        EdgeTypeKey::new("data"),
        move |_graph, _edge_id, _style, _hint, input| {
            let detour_y = input.from.y.0 + 240.0;
            Some(EdgeCustomPath {
                cache_key: 99,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::LineTo(Point::new(input.from.x, Px(detour_y))),
                    PathCommand::LineTo(Point::new(input.to.x, Px(detour_y))),
                    PathCommand::LineTo(input.to),
                ],
            })
        },
    ));

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            selected_edges: vec![edge],
            draw_order,
            ..Default::default()
        },
        default_editor_config(),
    );

    for _ in 0..2 {
        let edge_types = edge_types.clone();
        let _ = render_surface_frame_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            move |binding| {
                let mut props = super::NodeGraphSurfaceProps::new(binding.clone());
                props.edge_types = Some(edge_types);
                props
            },
        );
    }

    let toolbar_size = Size::new(Px(44.0), Px(18.0));
    let view_state = binding.view_state_model();
    let internals = binding.internals_store();
    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "edge-toolbar-custom-path-host",
        |cx| {
            vec![
                crate::ui::overlays::node_graph_edge_toolbar_host_for_internals_test(
                    cx,
                    crate::ui::overlays::NodeGraphEdgeToolbarInternalsHostTestProps {
                        view_state,
                        requested_edge: None,
                        internals: internals.clone(),
                        bounds,
                        size: toolbar_size,
                        label: Arc::from("Edge toolbar"),
                        test_id: Arc::from("node_graph.edge_toolbar"),
                    },
                    |cx| {
                        let mut child = ContainerProps::default();
                        child.layout.size.width = Length::Px(toolbar_size.width);
                        child.layout.size.height = Length::Px(toolbar_size.height);
                        vec![cx.container(child, |_| Vec::new())]
                    },
                ),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    let toolbar = ui.children(root)[0];
    let child = ui.children(toolbar)[0];
    let child_bounds = ui
        .debug_node_bounds(child)
        .expect("edge toolbar child should be laid out");
    let expected_child_bounds = Rect::new(
        Point::new(
            Px(expected_center.x.0 - toolbar_size.width.0 / 2.0),
            Px(expected_center.y.0 - toolbar_size.height.0 / 2.0),
        ),
        toolbar_size,
    );

    assert_rect_near(child_bounds, expected_child_bounds, 0.5);
}

#[test]
fn custom_edge_path_feeds_declarative_edge_label_child_layer_anchor() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let edge_ref = graph.edges.get(&edge).expect("test edge");
    let from = geom.port_center(edge_ref.from).expect("from port center");
    let to = geom.port_center(edge_ref.to).expect("to port center");
    let detour_y = from.y.0 + 240.0;
    let custom_points = [
        from,
        Point::new(from.x, Px(detour_y)),
        Point::new(to.x, Px(detour_y)),
        to,
    ];
    let expected_center = polyline_midpoint(&custom_points);
    let edge_types = Rc::new(
        NodeGraphEdgeTypes::new()
            .register(
                EdgeTypeKey::new("data"),
                |_graph, _edge, _style, mut hint| {
                    hint.label = Some(Arc::from("Custom path label"));
                    hint
                },
            )
            .register_path(
                EdgeTypeKey::new("data"),
                move |_graph, _edge_id, _style, _hint, input| {
                    let detour_y = input.from.y.0 + 240.0;
                    Some(EdgeCustomPath {
                        cache_key: 100,
                        commands: vec![
                            PathCommand::MoveTo(input.from),
                            PathCommand::LineTo(Point::new(input.from.x, Px(detour_y))),
                            PathCommand::LineTo(Point::new(input.to.x, Px(detour_y))),
                            PathCommand::LineTo(input.to),
                        ],
                    })
                },
            ),
    );

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            selected_edges: vec![edge],
            draw_order,
            ..Default::default()
        },
        default_editor_config(),
    );

    let mut last = None;
    for _ in 0..4 {
        let edge_types = edge_types.clone();
        let snapshot = render_surface_frame_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            move |binding| {
                let mut props = super::NodeGraphSurfaceProps::new(binding.clone());
                props.edge_types = Some(edge_types);
                props
            },
        );
        if snapshot
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some("node_graph.edge_label.0"))
        {
            last = Some(snapshot);
            break;
        }
        last = Some(snapshot);
    }
    let snapshot = last.expect("at least one frame rendered");
    let label = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("node_graph.edge_label.0"))
        .unwrap_or_else(|| {
            let available = snapshot
                .nodes
                .iter()
                .filter_map(|node| node.test_id.as_deref())
                .collect::<Vec<_>>();
            panic!("missing edge label child layer; available={available:?}")
        });

    assert_eq!(label.label.as_deref(), Some("Custom path label"));
    assert_point_near(
        Point::new(
            Px(label.bounds.origin.x.0 + label.bounds.size.width.0 / 2.0),
            Px(label.bounds.origin.y.0 + label.bounds.size.height.0 / 2.0),
        ),
        expected_center,
        0.5,
    );
}

#[derive(Default)]
struct RecordingEdgeLabelRenderer {
    calls: Rc<RefCell<Vec<NodeGraphEdgeLabelLayout>>>,
}

impl NodeGraphDeclarativeEdgeLabelRenderer<TestActionHostImpl> for RecordingEdgeLabelRenderer {
    fn render_edge_label(
        &mut self,
        cx: &mut fret_ui::ElementContext<'_, TestActionHostImpl>,
        graph: &Graph,
        layout: NodeGraphEdgeLabelLayout,
    ) -> fret_ui::element::Elements {
        if !graph.edges.contains_key(&layout.edge) {
            return Vec::new().into();
        }
        self.calls.borrow_mut().push(layout.clone());

        let mut props = fret_ui::element::SemanticsProps {
            label: Some(Arc::<str>::from("Custom edge label child")),
            value: Some(Arc::<str>::from(format!(
                "edge={}; center_x={:.1}; center_y={:.1}; zoom={:.1}; label={:?}",
                layout.edge.0,
                layout.edge_center_window.x.0,
                layout.edge_center_window.y.0,
                layout.zoom,
                layout.label.as_deref(),
            ))),
            ..Default::default()
        };
        props.layout.size.width = Length::Px(Px(96.0));
        props.layout.size.height = Length::Px(Px(24.0));

        vec![cx.semantics(props, |_cx| Vec::new()).attach_semantics(
            fret_ui::element::SemanticsDecoration::default().test_id(Arc::<str>::from(format!(
                "node_graph.edge_label.custom.{}",
                layout.edge.0
            ))),
        )]
        .into()
    }
}

struct InteractiveEdgeLabelRenderer {
    calls: Rc<RefCell<Vec<NodeGraphEdgeLabelLayout>>>,
    pointer_downs: Arc<AtomicUsize>,
}

impl NodeGraphDeclarativeEdgeLabelRenderer<TestActionHostImpl> for InteractiveEdgeLabelRenderer {
    fn edge_label_hit_test_mode(
        &mut self,
        _graph: &Graph,
        _layout: &NodeGraphEdgeLabelLayout,
    ) -> NodeGraphEdgeLabelHitTestMode {
        NodeGraphEdgeLabelHitTestMode::ChildBounds
    }

    fn render_edge_label(
        &mut self,
        cx: &mut fret_ui::ElementContext<'_, TestActionHostImpl>,
        graph: &Graph,
        layout: NodeGraphEdgeLabelLayout,
    ) -> fret_ui::element::Elements {
        if !graph.edges.contains_key(&layout.edge) {
            return Vec::new().into();
        }
        self.calls.borrow_mut().push(layout.clone());

        let mut pressable = PressableProps::default();
        pressable.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Px(Px(96.0)),
                height: Length::Px(Px(24.0)),
                ..Default::default()
            },
            ..Default::default()
        };
        let pointer_downs = self.pointer_downs.clone();
        let test_id = format!("node_graph.edge_label.control.{}", layout.edge.0);
        vec![cx
            .pressable(pressable, move |cx, _state| {
                cx.pressable_on_pointer_down(Arc::new(move |_host, _cx, down| {
                    if down.button == MouseButton::Left {
                        pointer_downs.fetch_add(1, Ordering::Relaxed);
                        return fret_ui::action::PressablePointerDownResult::SkipDefaultAndStopPropagation;
                    }
                    fret_ui::action::PressablePointerDownResult::Continue
                }));
                Vec::new()
            })
            .test_id(test_id)]
        .into()
    }
}

#[test]
fn custom_edge_path_feeds_declarative_edge_label_custom_renderer_anchor() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let edge_ref = graph.edges.get(&edge).expect("test edge");
    let from = geom.port_center(edge_ref.from).expect("from port center");
    let to = geom.port_center(edge_ref.to).expect("to port center");
    let detour_y = from.y.0 + 240.0;
    let custom_points = [
        from,
        Point::new(from.x, Px(detour_y)),
        Point::new(to.x, Px(detour_y)),
        to,
    ];
    let expected_center = polyline_midpoint(&custom_points);
    let edge_types = Rc::new(NodeGraphEdgeTypes::new().register_path(
        EdgeTypeKey::new("data"),
        move |_graph, _edge_id, _style, _hint, input| {
            let detour_y = input.from.y.0 + 240.0;
            Some(EdgeCustomPath {
                cache_key: 101,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::LineTo(Point::new(input.from.x, Px(detour_y))),
                    PathCommand::LineTo(Point::new(input.to.x, Px(detour_y))),
                    PathCommand::LineTo(input.to),
                ],
            })
        },
    ));

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            ..Default::default()
        },
        default_editor_config(),
    );

    let calls = Rc::new(RefCell::new(Vec::<NodeGraphEdgeLabelLayout>::new()));
    let mut renderer = RecordingEdgeLabelRenderer {
        calls: calls.clone(),
    };
    let custom_test_id = format!("node_graph.edge_label.custom.{}", edge.0);
    let mut last = None;
    for _ in 0..4 {
        let edge_types = edge_types.clone();
        let snapshot = render_surface_frame_with_edge_label_renderer_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            move |binding| {
                let mut props = super::NodeGraphSurfaceProps::new(binding.clone());
                props.edge_types = Some(edge_types);
                props
            },
            &mut renderer,
        );
        if snapshot
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(custom_test_id.as_str()))
        {
            last = Some(snapshot);
            break;
        }
        last = Some(snapshot);
    }
    let snapshot = last.expect("at least one frame rendered");
    let custom = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(custom_test_id.as_str()))
        .unwrap_or_else(|| {
            let available = snapshot
                .nodes
                .iter()
                .filter_map(|node| node.test_id.as_deref())
                .collect::<Vec<_>>();
            panic!("missing custom edge label child renderer output; available={available:?}")
        });

    assert_eq!(custom.label.as_deref(), Some("Custom edge label child"));
    assert_point_near(
        Point::new(
            Px(custom.bounds.origin.x.0 + custom.bounds.size.width.0 / 2.0),
            Px(custom.bounds.origin.y.0 + custom.bounds.size.height.0 / 2.0),
        ),
        expected_center,
        0.5,
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .all(|node| node.test_id.as_deref() != Some("node_graph.edge_label.0")),
        "custom renderer without a default label must not synthesize the default label child"
    );

    let calls = calls.borrow();
    let call = calls
        .iter()
        .find(|call| call.edge == edge)
        .expect("custom edge label renderer should be called for the edge");
    assert_eq!(call.label, None);
    assert_eq!(call.zoom, 1.0);
    assert_point_near(call.edge_center_window, expected_center, 0.5);
}

#[test]
fn custom_edge_label_control_intercepts_inside_and_falls_through_outside_child_bounds() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let edge_ref = graph.edges.get(&edge).expect("test edge");
    let from = geom.port_center(edge_ref.from).expect("from port center");
    let to = geom.port_center(edge_ref.to).expect("to port center");
    let detour_y = from.y.0 + 240.0;
    let custom_points = [
        from,
        Point::new(from.x, Px(detour_y)),
        Point::new(to.x, Px(detour_y)),
        to,
    ];
    let expected_center = polyline_midpoint(&custom_points);
    let edge_types = Rc::new(NodeGraphEdgeTypes::new().register_path(
        EdgeTypeKey::new("data"),
        move |_graph, _edge_id, _style, _hint, input| {
            let detour_y = input.from.y.0 + 240.0;
            Some(EdgeCustomPath {
                cache_key: 102,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::LineTo(Point::new(input.from.x, Px(detour_y))),
                    PathCommand::LineTo(Point::new(input.to.x, Px(detour_y))),
                    PathCommand::LineTo(input.to),
                ],
            })
        },
    ));

    let mut host = TestActionHostImpl::default();
    let mut ui = fret_ui::UiTree::<TestActionHostImpl>::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = test_node_graph_surface_bounds();
    host.bounds = bounds;
    let mut services = FakeUiServices;
    let binding = NodeGraphSurfaceBinding::new(
        &mut host.models,
        graph,
        NodeGraphViewState {
            draw_order,
            ..Default::default()
        },
        default_editor_config(),
    );

    let calls = Rc::new(RefCell::new(Vec::<NodeGraphEdgeLabelLayout>::new()));
    let pointer_downs = Arc::new(AtomicUsize::new(0));
    let mut renderer = InteractiveEdgeLabelRenderer {
        calls: calls.clone(),
        pointer_downs: pointer_downs.clone(),
    };
    let custom_test_id = format!("node_graph.edge_label.control.{}", edge.0);
    let mut last = None;
    for _ in 0..4 {
        let edge_types = edge_types.clone();
        let snapshot = render_surface_frame_with_edge_label_renderer_for_binding(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            &binding,
            move |binding| {
                let mut props = super::NodeGraphSurfaceProps::new(binding.clone());
                props.edge_types = Some(edge_types);
                props
            },
            &mut renderer,
        );
        if snapshot
            .nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(custom_test_id.as_str()))
        {
            last = Some(snapshot);
            break;
        }
        last = Some(snapshot);
    }
    let snapshot = last.expect("at least one frame rendered");
    let custom = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(custom_test_id.as_str()))
        .expect("custom edge label control should be present");
    let inside = Point::new(
        Px(custom.bounds.origin.x.0 + 1.0),
        Px(custom.bounds.origin.y.0 + 1.0),
    );
    let outside = Point::new(Px(bounds.origin.x.0 + 12.0), Px(bounds.origin.y.0 + 12.0));

    assert_point_near(
        Point::new(
            Px(custom.bounds.origin.x.0 + custom.bounds.size.width.0 / 2.0),
            Px(custom.bounds.origin.y.0 + custom.bounds.size.height.0 / 2.0),
        ),
        expected_center,
        0.5,
    );
    let inside_hit = ui.debug_hit_test(inside).hit;
    let inside_routing_hit = ui.debug_hit_test_routing(inside).hit;
    assert_eq!(
        inside_hit,
        Some(custom.id),
        "inside the custom edge-label control should hit the control node"
    );
    assert_eq!(
        inside_routing_hit,
        Some(custom.id),
        "pointer routing should target the custom edge-label control"
    );
    let outside_hit = ui.debug_hit_test(outside).hit;
    let outside_hit_path = outside_hit
        .map(|node| ui.debug_node_path(node))
        .unwrap_or_default();
    let outside_hit_bounds = outside_hit.and_then(|node| ui.debug_node_bounds(node));
    assert!(
        outside_hit.is_some(),
        "outside the custom edge-label control should still hit the underlying surface"
    );
    assert_ne!(
        outside_hit, inside_hit,
        "outside the custom edge-label control should not be masked by the label host"
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert_eq!(
        pointer_downs.load(Ordering::Relaxed),
        1,
        "inside pointer down should be handled by the edge-label control"
    );
    assert_eq!(
        ui.focus(),
        Some(custom.id),
        "inside pointer down should focus the edge-label control instead of the canvas pointer path"
    );

    let focus_before_outside = ui.focus();
    ui.dispatch_event(
        &mut host,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(1),
            position: outside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert_eq!(
        pointer_downs.load(Ordering::Relaxed),
        1,
        "outside pointer down should fall through instead of targeting the edge-label control"
    );
    assert!(
        ui.focus().is_some() && ui.focus() != focus_before_outside,
        "outside pointer down should reach the canvas pointer path; outside_hit={outside_hit:?}, outside_hit_path={outside_hit_path:?}, outside_hit_bounds={outside_hit_bounds:?}, focus_before={focus_before_outside:?}, focus_after={:?}",
        ui.focus()
    );

    let calls = calls.borrow();
    let call = calls
        .iter()
        .find(|call| call.edge == edge)
        .expect("custom edge label renderer should be called for the edge");
    assert_point_near(call.edge_center_window, expected_center, 0.5);
}

#[test]
fn record_portal_measured_node_size_in_state_ignores_epsilon_churn() {
    let mut models = ModelStore::default();
    let state = models.insert(PortalMeasuredGeometryState::default());
    let node = NodeId::from_u128(9402);

    assert!(record_portal_measured_node_size_in_state(
        &mut models,
        &state,
        node,
        (200.0, 120.0),
    ));
    assert!(!record_portal_measured_node_size_in_state(
        &mut models,
        &state,
        node,
        (
            200.0 + MEASURED_GEOMETRY_EPSILON_PX * 0.5,
            120.0 + MEASURED_GEOMETRY_EPSILON_PX * 0.5,
        ),
    ));
    assert!(record_portal_measured_node_size_in_state(
        &mut models,
        &state,
        node,
        (
            200.0 + MEASURED_GEOMETRY_EPSILON_PX * 2.0,
            120.0 + MEASURED_GEOMETRY_EPSILON_PX * 2.0,
        ),
    ));

    let pending = models
        .read(&state, |st| st.pending_node_sizes_px.get(&node).copied())
        .unwrap();
    assert_eq!(
        pending,
        Some((
            200.0 + MEASURED_GEOMETRY_EPSILON_PX * 2.0,
            120.0 + MEASURED_GEOMETRY_EPSILON_PX * 2.0,
        ))
    );
}

#[test]
fn flush_portal_measured_geometry_state_publishes_pending_node_size_to_store() {
    let mut graph = Graph::new(GraphId::from_u128(9403));
    let node = NodeId::from_u128(9404);
    graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 0.0, y: 0.0 }));

    let measured = MeasuredGeometryStore::new();
    let initial_revision = measured.revision();
    let mut state = PortalMeasuredGeometryState::default();
    state.pending_node_sizes_px.insert(node, (320.0, 180.0));

    let outcome = flush_portal_measured_geometry_state(
        &graph,
        &crate::ui::style::NodeGraphStyle::default(),
        &measured,
        &mut state,
    );

    assert!(outcome.store_changed);
    assert!(outcome.state_changed);
    assert!(measured.revision() > initial_revision);
    assert_eq!(measured.node_size_px(node), Some((320.0, 180.0)));
    assert_eq!(state.published_nodes, vec![node]);
    assert!(state.pending_node_sizes_px.is_empty());
}

#[test]
fn flush_portal_measured_geometry_state_skips_explicit_size_nodes() {
    let mut graph = Graph::new(GraphId::from_u128(9405));
    let node = NodeId::from_u128(9406);
    let mut value = test_node(CanvasPoint { x: 0.0, y: 0.0 });
    value.size = Some(CanvasSize {
        width: 160.0,
        height: 90.0,
    });
    graph.nodes.insert(node, value);

    let measured = MeasuredGeometryStore::new();
    let initial_revision = measured.revision();
    let mut state = PortalMeasuredGeometryState::default();
    state.pending_node_sizes_px.insert(node, (320.0, 180.0));

    let outcome = flush_portal_measured_geometry_state(
        &graph,
        &crate::ui::style::NodeGraphStyle::default(),
        &measured,
        &mut state,
    );

    assert!(!outcome.store_changed);
    assert!(outcome.state_changed);
    assert_eq!(measured.revision(), initial_revision);
    assert_eq!(measured.node_size_px(node), None);
    assert!(state.published_nodes.is_empty());
    assert!(state.pending_node_sizes_px.is_empty());
}

#[test]
fn flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes() {
    let mut graph = Graph::new(GraphId::from_u128(9407));
    let node = NodeId::from_u128(9408);
    graph
        .nodes
        .insert(node, test_node(CanvasPoint { x: 0.0, y: 0.0 }));

    let measured = MeasuredGeometryStore::new();
    let style = crate::ui::style::NodeGraphStyle::default();
    let mut state = PortalMeasuredGeometryState::default();
    state.pending_node_sizes_px.insert(node, (320.0, 180.0));

    let first = flush_portal_measured_geometry_state(&graph, &style, &measured, &mut state);
    assert!(first.store_changed);
    assert_eq!(measured.node_size_px(node), Some((320.0, 180.0)));
    assert_eq!(state.published_nodes, vec![node]);

    state.pending_node_sizes_px.insert(node, (120.0, 80.0));
    let shrink = flush_portal_measured_geometry_state(&graph, &style, &measured, &mut state);
    assert!(!shrink.store_changed);
    assert!(shrink.state_changed);
    assert_eq!(
        measured.node_size_px(node),
        Some((320.0, 180.0)),
        "portal measurements must remain growth-only hints"
    );
    assert_eq!(state.published_nodes, vec![node]);

    graph.nodes.remove(&node);
    let removed = flush_portal_measured_geometry_state(&graph, &style, &measured, &mut state);
    assert!(removed.store_changed);
    assert!(removed.state_changed);
    assert_eq!(
        measured.node_size_px(node),
        None,
        "removed graph nodes must be pruned from the measured-geometry store"
    );
    assert!(state.published_nodes.is_empty());
}

#[test]
fn edges_cache_key_changes_when_edge_types_or_skin_revision_changes() {
    let node = NodeId::from_u128(9013);
    let view_state = NodeGraphViewState {
        zoom: 1.25,
        draw_order: vec![node],
        ..NodeGraphViewState::default()
    };
    let editor_config = NodeGraphEditorConfig::default();
    let interaction = editor_config.resolved_interaction_state();
    let node_origin = editor_config.interaction.node_origin;
    let style = crate::ui::style::NodeGraphStyle::default();
    let derived = derived_geometry_cache_key(
        39,
        view_state.zoom,
        node_origin,
        &view_state.draw_order,
        &interaction,
        &style,
        0,
        0,
        0,
        0.0,
    );
    let draw_order_hash = stable_hash_u64(2, &view_state.draw_order);

    let base = edges_cache_key(
        39,
        view_state.zoom,
        node_origin,
        draw_order_hash,
        derived.0,
        0,
        0,
    );
    let edge_types_changed = edges_cache_key(
        39,
        view_state.zoom,
        node_origin,
        draw_order_hash,
        derived.0,
        1,
        0,
    );
    let skin_changed = edges_cache_key(
        39,
        view_state.zoom,
        node_origin,
        draw_order_hash,
        derived.0,
        0,
        1,
    );

    assert_ne!(base, edge_types_changed);
    assert_ne!(base, skin_changed);
}

#[test]
fn declarative_edge_types_feed_default_surface_edge_draws() {
    let (graph, draw_order, edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let style = crate::ui::style::NodeGraphStyle::default();
    let color = Color {
        r: 0.9,
        g: 0.1,
        b: 0.2,
        a: 1.0,
    };
    let dash = DashPatternV1::new(Px(8.0), Px(4.0), Px(0.0));
    let edge_types = NodeGraphEdgeTypes::new()
        .register(
            EdgeTypeKey::new("data"),
            move |_graph, edge_id, _style, mut hint| {
                assert_eq!(edge_id, edge);
                hint.color = Some(color);
                hint.width_mul = 2.0;
                hint.route = EdgeRouteKind::Straight;
                hint.dash = Some(dash);
                hint
            },
        )
        .register_path(
            EdgeTypeKey::new("data"),
            |_graph, _edge_id, _style, _hint, input| {
                Some(EdgeCustomPath {
                    cache_key: 77,
                    commands: vec![
                        PathCommand::MoveTo(input.from),
                        PathCommand::LineTo(Point::new(input.to.x, input.from.y)),
                        PathCommand::LineTo(input.to),
                    ],
                })
            },
        );

    let draws = build_edges_draws_paint_only(
        &graph,
        7,
        1.0,
        &geom,
        &style,
        edge_types.revision(),
        0,
        Some(&edge_types),
        None,
    );

    assert_eq!(draws.len(), 1);
    let draw = &draws[0];
    assert_eq!(draw.edge, edge);
    assert_eq!(draw.color, color);
    assert_eq!(draw.width_mul, 2.0);
    assert_eq!(draw.route, EdgeRouteKind::Straight);
    assert_eq!(draw.dash, Some(dash));
    assert!(matches!(
        draw.commands.as_ref(),
        [
            PathCommand::MoveTo(_),
            PathCommand::LineTo(_),
            PathCommand::LineTo(_)
        ]
    ));
}

#[derive(Debug)]
struct TestEdgeSkin {
    rev: u64,
    color: Color,
    width_mul: f32,
    dash: DashPatternV1,
}

impl NodeGraphSkin for TestEdgeSkin {
    fn revision(&self) -> u64 {
        self.rev
    }

    fn edge_chrome_hint(
        &self,
        _graph: &Graph,
        _edge: EdgeId,
        _style: &crate::ui::NodeGraphStyle,
        _selected: bool,
        _hovered: bool,
    ) -> EdgeChromeHint {
        EdgeChromeHint {
            color: Some(self.color),
            width_mul: Some(self.width_mul),
            dash: Some(self.dash),
            ..EdgeChromeHint::default()
        }
    }
}

#[test]
fn declarative_skin_refines_edge_draw_hints_after_edge_types() {
    let (graph, draw_order, _edge) = make_graph_two_nodes_with_edge();
    let geom = build_test_canvas_geometry(&graph, &draw_order);
    let style = crate::ui::style::NodeGraphStyle::default();
    let edge_types_color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let skin_color = Color {
        r: 0.0,
        g: 0.45,
        b: 1.0,
        a: 1.0,
    };
    let skin_dash = DashPatternV1::new(Px(6.0), Px(2.0), Px(1.0));
    let edge_types =
        NodeGraphEdgeTypes::new().with_fallback(move |_graph, _edge, _style, mut hint| {
            hint.color = Some(edge_types_color);
            hint.width_mul = 1.25;
            hint.route = EdgeRouteKind::Step;
            hint
        });
    let skin = TestEdgeSkin {
        rev: 5,
        color: skin_color,
        width_mul: 3.0,
        dash: skin_dash,
    };

    let draws = build_edges_draws_paint_only(
        &graph,
        8,
        1.0,
        &geom,
        &style,
        edge_types.revision(),
        skin.revision(),
        Some(&edge_types),
        Some(&skin),
    );

    assert_eq!(draws.len(), 1);
    let draw = &draws[0];
    assert_eq!(draw.color, skin_color);
    assert_eq!(draw.width_mul, 3.0);
    assert_eq!(draw.dash, Some(skin_dash));
    assert_eq!(
        draw.route,
        EdgeRouteKind::Step,
        "skin refinements must not erase the edgeTypes route"
    );
}

#[test]
fn authoritative_selection_changes_keep_paint_cache_keys_stable() {
    let node_a = NodeId::from_u128(9014);
    let node_b = NodeId::from_u128(9015);
    let edge = EdgeId::from_u128(9016);
    let group = GroupId::from_u128(9017);
    let base_view = NodeGraphViewState {
        pan: CanvasPoint { x: 120.0, y: -48.0 },
        zoom: 1.75,
        selected_nodes: vec![node_a],
        selected_edges: vec![edge],
        selected_groups: vec![group],
        draw_order: vec![node_a, node_b],
        ..NodeGraphViewState::default()
    };
    let selection_only_view = NodeGraphViewState {
        selected_nodes: vec![node_b],
        selected_edges: Vec::new(),
        selected_groups: Vec::new(),
        ..base_view.clone()
    };
    let style = crate::ui::style::NodeGraphStyle::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(1280.0), Px(720.0)),
    );
    let graph_rev = 41;
    let editor_config = NodeGraphEditorConfig::default();
    let node_origin = editor_config.interaction.node_origin;

    let grid_a = grid_cache_key(bounds, view_from_state(&base_view), &style);
    let grid_b = grid_cache_key(bounds, view_from_state(&selection_only_view), &style);
    let interaction_a = editor_config.resolved_interaction_state();
    let interaction_b = interaction_a.clone();
    let derived_a = derived_geometry_cache_key(
        graph_rev,
        base_view.zoom,
        node_origin,
        &base_view.draw_order,
        &interaction_a,
        &style,
        0,
        0,
        0,
        0.0,
    );
    let derived_b = derived_geometry_cache_key(
        graph_rev,
        selection_only_view.zoom,
        node_origin,
        &selection_only_view.draw_order,
        &interaction_b,
        &style,
        0,
        0,
        0,
        0.0,
    );
    let draw_order_hash_a = stable_hash_u64(2, &base_view.draw_order);
    let draw_order_hash_b = stable_hash_u64(2, &selection_only_view.draw_order);
    let nodes_a = nodes_cache_key(
        graph_rev,
        base_view.zoom,
        node_origin,
        draw_order_hash_a,
        derived_a.0,
    );
    let nodes_b = nodes_cache_key(
        graph_rev,
        selection_only_view.zoom,
        node_origin,
        draw_order_hash_b,
        derived_b.0,
    );
    let edges_a = edges_cache_key(
        graph_rev,
        base_view.zoom,
        node_origin,
        draw_order_hash_a,
        derived_a.0,
        0,
        0,
    );
    let edges_b = edges_cache_key(
        graph_rev,
        selection_only_view.zoom,
        node_origin,
        draw_order_hash_b,
        derived_b.0,
        0,
        0,
    );

    assert_eq!(grid_a, grid_b);
    assert_eq!(derived_a, derived_b);
    assert_eq!(nodes_a, nodes_b);
    assert_eq!(edges_a, edges_b);
}

#[test]
fn authoritative_graph_replacement_invalidates_only_graph_dependent_paint_cache_keys() {
    let node_a = NodeId::from_u128(9018);
    let node_b = NodeId::from_u128(9019);
    let view_state = NodeGraphViewState {
        pan: CanvasPoint { x: -96.0, y: 24.0 },
        zoom: 0.85,
        draw_order: vec![node_a, node_b],
        ..NodeGraphViewState::default()
    };
    let style = crate::ui::style::NodeGraphStyle::default();
    let bounds = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        fret_core::Size::new(Px(1024.0), Px(768.0)),
    );
    let editor_config = NodeGraphEditorConfig::default();
    let interaction = editor_config.resolved_interaction_state();
    let node_origin = editor_config.interaction.node_origin;
    let draw_order_hash = stable_hash_u64(2, &view_state.draw_order);

    let grid_before = grid_cache_key(bounds, view_from_state(&view_state), &style);
    let derived_before = derived_geometry_cache_key(
        73,
        view_state.zoom,
        node_origin,
        &view_state.draw_order,
        &interaction,
        &style,
        0,
        0,
        0,
        0.0,
    );
    let nodes_before = nodes_cache_key(
        73,
        view_state.zoom,
        node_origin,
        draw_order_hash,
        derived_before.0,
    );
    let edges_before = edges_cache_key(
        73,
        view_state.zoom,
        node_origin,
        draw_order_hash,
        derived_before.0,
        0,
        0,
    );

    let grid_after = grid_cache_key(bounds, view_from_state(&view_state), &style);
    let derived_after = derived_geometry_cache_key(
        74,
        view_state.zoom,
        node_origin,
        &view_state.draw_order,
        &interaction,
        &style,
        0,
        0,
        0,
        0.0,
    );
    let nodes_after = nodes_cache_key(
        74,
        view_state.zoom,
        node_origin,
        draw_order_hash,
        derived_after.0,
    );
    let edges_after = edges_cache_key(
        74,
        view_state.zoom,
        node_origin,
        draw_order_hash,
        derived_after.0,
        0,
        0,
    );

    assert_eq!(grid_before, grid_after);
    assert_ne!(derived_before, derived_after);
    assert_ne!(nodes_before, nodes_after);
    assert_ne!(edges_before, edges_after);
}

#[test]
fn sync_authoritative_surface_boundary_in_models_clears_graph_scoped_transients_on_graph_change() {
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9021);
    let node_b = NodeId::from_u128(9022);
    let previous_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..NodeGraphViewState::default()
    };
    let boundary = host
        .models
        .insert(Some(authoritative_surface_boundary_snapshot(
            GraphId::from_u128(9020),
            3,
            &previous_view,
        )));
    let drag = host.models.insert(Some(DragState {
        button: MouseButton::Middle,
        last_pos: Point::new(Px(3.0), Px(4.0)),
    }));
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(8.0), Px(8.0)),
        active: true,
        toggle: false,
        base_selected_nodes: Arc::from([]),
        preview_selected_nodes: Arc::from([node_a]),
    }));
    let node_drag = host.models.insert(Some(test_node_drag_state(
        NodeDragPhase::Active,
        Point::new(Px(16.0), Px(0.0)),
    )));
    let reconnect_drag = host.models.insert(None::<ReconnectDragState>);
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    }));
    let hovered = host.models.insert(Some(node_a));
    let hover_anchor = host.models.insert(HoverAnchorStore {
        hovered_id: Some(node_a),
        hovered_canvas_bounds: Some(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(100.0), Px(40.0)),
        )),
    });
    let mut portal_bounds_state = PortalBoundsStore::default();
    portal_bounds_state.fit_to_portals_count = 7;
    portal_bounds_state.pending_fit_to_portals = true;
    portal_bounds_state.nodes_canvas_bounds.insert(
        node_a,
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(20.0), Px(20.0)),
        ),
    );
    let portal_bounds = host.models.insert(portal_bounds_state);

    let next_view = NodeGraphViewState {
        selected_nodes: vec![node_b],
        ..NodeGraphViewState::default()
    };

    assert!(sync_authoritative_surface_boundary_in_models(
        &mut host.models,
        &boundary,
        authoritative_surface_boundary_snapshot(GraphId::from_u128(9020), 4, &next_view),
        &drag,
        &marquee,
        &node_drag,
        &reconnect_drag,
        &pending,
        &hovered,
        &hover_anchor,
        &portal_bounds,
    ));
    assert!(
        host.models
            .read(&drag, |state| state.is_none())
            .expect("drag readable")
    );
    assert!(
        host.models
            .read(&marquee, |state| state.is_none())
            .expect("marquee readable")
    );
    assert!(
        host.models
            .read(&node_drag, |state| state.is_none())
            .expect("node drag readable")
    );
    assert!(
        host.models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
    assert!(
        host.models
            .read(&hovered, |state| state.is_none())
            .expect("hovered readable")
    );
    host.models
        .read(&hover_anchor, |state| {
            assert_eq!(state.hovered_id, None);
            assert_eq!(state.hovered_canvas_bounds, None);
        })
        .expect("hover anchor readable");
    host.models
        .read(&portal_bounds, |state| {
            assert_eq!(state.fit_to_portals_count, 7);
            assert!(!state.pending_fit_to_portals);
            assert!(state.nodes_canvas_bounds.is_empty());
        })
        .expect("portal bounds readable");
}

#[test]
fn sync_authoritative_surface_boundary_in_models_keeps_pan_and_hover_on_selection_only_change() {
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9031);
    let node_b = NodeId::from_u128(9032);
    let previous_view = NodeGraphViewState {
        selected_nodes: vec![node_a],
        ..NodeGraphViewState::default()
    };
    let boundary = host
        .models
        .insert(Some(AuthoritativeSurfaceBoundarySnapshot {
            graph_id: GraphId::from_u128(9030),
            graph_rev: 9,
            selected_nodes_hash: stable_hash_u64(17, &previous_view.selected_nodes),
            selected_edges_hash: stable_hash_u64(19, &previous_view.selected_edges),
            selected_groups_hash: stable_hash_u64(23, &previous_view.selected_groups),
        }));
    let drag = host.models.insert(Some(DragState {
        button: MouseButton::Middle,
        last_pos: Point::new(Px(11.0), Px(12.0)),
    }));
    let marquee = host.models.insert(Some(MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(8.0), Px(8.0)),
        active: true,
        toggle: false,
        base_selected_nodes: Arc::from([node_a]),
        preview_selected_nodes: Arc::from([node_b]),
    }));
    let node_drag = host.models.insert(Some(test_node_drag_state(
        NodeDragPhase::Armed,
        Point::new(Px(5.0), Px(0.0)),
    )));
    let pending = host.models.insert(Some(PendingSelectionState {
        nodes: Arc::from([node_b]),
        clear_edges: false,
        clear_groups: false,
    }));
    let reconnect_drag = host.models.insert(None::<ReconnectDragState>);
    let hovered = host.models.insert(Some(node_a));
    let hover_bounds = Rect::new(
        Point::new(Px(10.0), Px(10.0)),
        fret_core::Size::new(Px(40.0), Px(20.0)),
    );
    let hover_anchor = host.models.insert(HoverAnchorStore {
        hovered_id: Some(node_a),
        hovered_canvas_bounds: Some(hover_bounds),
    });
    let mut portal_bounds_state = PortalBoundsStore::default();
    portal_bounds_state.fit_to_portals_count = 5;
    portal_bounds_state.pending_fit_to_portals = true;
    portal_bounds_state
        .nodes_canvas_bounds
        .insert(node_a, hover_bounds);
    let portal_bounds = host.models.insert(portal_bounds_state);

    let next_view = NodeGraphViewState {
        selected_nodes: vec![node_b],
        ..NodeGraphViewState::default()
    };

    assert!(sync_authoritative_surface_boundary_in_models(
        &mut host.models,
        &boundary,
        authoritative_surface_boundary_snapshot(GraphId::from_u128(9030), 9, &next_view),
        &drag,
        &marquee,
        &node_drag,
        &reconnect_drag,
        &pending,
        &hovered,
        &hover_anchor,
        &portal_bounds,
    ));
    assert!(
        host.models
            .read(&drag, |state| state.is_some())
            .expect("drag readable")
    );
    assert!(
        host.models
            .read(&marquee, |state| state.is_none())
            .expect("marquee readable")
    );
    assert!(
        host.models
            .read(&node_drag, |state| state.is_none())
            .expect("node drag readable")
    );
    assert!(
        host.models
            .read(&pending, |state| state.is_none())
            .expect("pending readable")
    );
    assert_eq!(
        host.models
            .read(&hovered, |state| *state)
            .expect("hovered readable"),
        Some(node_a)
    );
    host.models
        .read(&hover_anchor, |state| {
            assert_eq!(state.hovered_id, Some(node_a));
            assert_eq!(state.hovered_canvas_bounds, Some(hover_bounds));
        })
        .expect("hover anchor readable");
    host.models
        .read(&portal_bounds, |state| {
            assert_eq!(state.fit_to_portals_count, 5);
            assert!(state.pending_fit_to_portals);
            assert_eq!(state.nodes_canvas_bounds.get(&node_a), Some(&hover_bounds));
        })
        .expect("portal bounds readable");
}

#[test]
fn commit_marquee_selection_action_host_clears_edges_and_groups_for_non_toggle() {
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9201);
    let node_b = NodeId::from_u128(9202);
    let edge = EdgeId::new();
    let group = GroupId::new();
    let view_value = NodeGraphViewState {
        selected_nodes: vec![node_a],
        selected_edges: vec![edge],
        selected_groups: vec![group],
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let mut graph_value = Graph::new(GraphId::from_u128(9201));
    let from_port = PortId::new();
    let to_port = PortId::new();
    let mut node_a_value = test_node(CanvasPoint { x: 0.0, y: 0.0 });
    node_a_value.ports = vec![from_port];
    let mut node_b_value = test_node(CanvasPoint { x: 40.0, y: 20.0 });
    node_b_value.ports = vec![to_port];
    graph_value.nodes.insert(node_a, node_a_value);
    graph_value.nodes.insert(node_b, node_b_value);
    graph_value.ports.insert(
        from_port,
        Port {
            node: node_a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.ports.insert(
        to_port,
        Port {
            node: node_b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: from_port,
            to: to_port,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph_value.groups.insert(
        group,
        Group {
            title: "test group".into(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 1.0,
                    height: 1.0,
                },
            },
            color: None,
        },
    );
    let graph = host.models.insert(graph_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let marquee = MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        active: true,
        toggle: false,
        base_selected_nodes: Arc::from([node_a]),
        preview_selected_nodes: Arc::from([node_b]),
    };

    assert!(commit_marquee_selection_action_host(
        &mut host, &binding, &marquee,
    ));

    let selection = host
        .models
        .read(&view_state, |state| {
            (
                state.selected_nodes.clone(),
                state.selected_edges.clone(),
                state.selected_groups.clone(),
            )
        })
        .expect("read view state");
    assert_eq!(selection.0, vec![node_b]);
    assert!(selection.1.is_empty());
    assert!(selection.2.is_empty());
}

#[test]
fn commit_marquee_selection_action_host_preserves_edges_and_groups_for_toggle() {
    let mut host = TestActionHostImpl::default();
    let node_a = NodeId::from_u128(9301);
    let node_b = NodeId::from_u128(9302);
    let edge = EdgeId::new();
    let group = GroupId::new();
    let view_value = NodeGraphViewState {
        selected_nodes: vec![node_a],
        selected_edges: vec![edge],
        selected_groups: vec![group],
        ..Default::default()
    };
    let view_state = host.models.insert(view_value.clone());
    let mut graph_value = Graph::new(GraphId::from_u128(9301));
    let from_port = PortId::new();
    let to_port = PortId::new();
    let mut node_a_value = test_node(CanvasPoint { x: 0.0, y: 0.0 });
    node_a_value.ports = vec![from_port];
    let mut node_b_value = test_node(CanvasPoint { x: 40.0, y: 20.0 });
    node_b_value.ports = vec![to_port];
    graph_value.nodes.insert(node_a, node_a_value);
    graph_value.nodes.insert(node_b, node_b_value);
    graph_value.ports.insert(
        from_port,
        Port {
            node: node_a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.ports.insert(
        to_port,
        Port {
            node: node_b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: from_port,
            to: to_port,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph_value.groups.insert(
        group,
        Group {
            title: "test group".into(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 1.0,
                    height: 1.0,
                },
            },
            color: None,
        },
    );
    let graph = host.models.insert(graph_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        view_value,
        default_editor_config(),
    ));
    let controller = NodeGraphController::new(store);
    let binding = test_binding(&mut host, &graph, &view_state, &controller);
    let marquee = MarqueeDragState {
        start_screen: Point::new(Px(0.0), Px(0.0)),
        current_screen: Point::new(Px(0.0), Px(0.0)),
        active: true,
        toggle: true,
        base_selected_nodes: Arc::from([node_a]),
        preview_selected_nodes: Arc::from([node_b]),
    };

    assert!(commit_marquee_selection_action_host(
        &mut host, &binding, &marquee,
    ));

    let selection = host
        .models
        .read(&view_state, |state| {
            (
                state.selected_nodes.clone(),
                state.selected_edges.clone(),
                state.selected_groups.clone(),
            )
        })
        .expect("read view state");
    assert_eq!(selection.0, vec![node_b]);
    assert_eq!(selection.1, vec![edge]);
    assert_eq!(selection.2, vec![group]);
}
