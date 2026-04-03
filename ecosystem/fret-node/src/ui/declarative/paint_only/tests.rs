use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_canvas::view::PanZoom2D;
use fret_core::{
    AppWindowId, Modifiers, MouseButton, MouseButtons, Point, PointerId, PointerType, Px, Rect,
};
use fret_runtime::{
    ClipboardToken, DragKindId, DragSession, Effect, Model, ModelStore, ShareSheetToken, TickId,
    TimerToken,
};
use fret_ui::action::UiActionHost;

use super::hover_anchor::{HoverTooltipAnchorSource, hovered_canvas_anchor_rect_for_surface};

use super::overlay_elements::{
    build_hover_tooltip_overlay_spec, clamp_marquee_overlay_rect_to_bounds,
};
use super::pointer_down::read_left_pointer_down_snapshot_action_host;
use super::{
    AuthoritativeSurfaceBoundarySnapshot, DeclarativeDiagKeyAction, DeclarativeDiagViewPreset,
    DeclarativeKeyboardZoomAction, DerivedGeometryCacheState, DragState, HoverAnchorStore,
    Invalidation, LeftPointerDownOutcome, LeftPointerDownSnapshot, LeftPointerReleaseOutcome,
    MarqueeDragState, MarqueePointerMoveOutcome, NodeDragPhase, NodeDragPointerMoveOutcome,
    NodeDragReleaseOutcome, NodeDragState, NodeRectDraw, PendingSelectionState, PortalBoundsStore,
    PortalDebugFlags, PortalMeasuredGeometryState, apply_declarative_diag_view_preset_action_host,
    authoritative_surface_boundary_snapshot, begin_left_pointer_down_action_host,
    begin_pan_pointer_down_action_host, build_click_selection_preview_nodes,
    build_diag_normalize_visible_node_transaction, build_diag_nudge_visible_node_transaction,
    build_marquee_preview_selected_nodes, build_node_drag_transaction,
    collect_portal_label_infos_for_visible_subset, commit_graph_transaction,
    commit_marquee_selection_action_host, commit_node_drag_transaction,
    commit_pending_selection_action_host, complete_left_pointer_release_action_host,
    complete_node_drag_release_action_host, derived_geometry_cache_key, edges_cache_key,
    effective_selected_nodes_for_paint, escape_cancel_declarative_interactions_action_host,
    flush_portal_measured_geometry_state, grid_cache_key, handle_declarative_diag_key_action_host,
    handle_declarative_keyboard_zoom_action_host, handle_declarative_pointer_cancel_action_host,
    handle_declarative_pointer_up_action_host, handle_marquee_left_pointer_release_action_host,
    handle_marquee_pointer_move_action_host, handle_node_drag_left_pointer_release_action_host,
    handle_node_drag_pointer_move_action_host,
    handle_pending_selection_left_pointer_release_action_host, node_drag_commit_delta,
    nodes_cache_key, pointer_cancel_declarative_interactions_action_host,
    pointer_crossed_threshold, read_authoritative_view_state_in_models,
    record_portal_measured_node_size_in_state, resolve_hover_tooltip_anchor, stable_hash_u64,
    sync_authoritative_surface_boundary_in_models, sync_hover_anchor_store_in_models,
    sync_portal_canvas_bounds_in_models, update_hovered_node_pointer_move_action_host,
    update_view_state_action_host, view_from_state,
};
use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group, GroupId,
    Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ops::GraphOp;
use crate::runtime::callbacks::{
    NodeGraphCommitCallbacks, NodeGraphGestureCallbacks, NodeGraphViewCallbacks, SelectionChange,
    install_callbacks,
};
use crate::runtime::changes::NodeGraphChanges;
use crate::runtime::store::NodeGraphStore;
use crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX;
use crate::ui::paint_overrides::{NodeGraphPaintOverrides, NodeGraphPaintOverridesMap};
use crate::ui::{MeasuredGeometryStore, NodeGraphController, NodeGraphSurfaceBinding};
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
        fn on_graph_commit(
            &mut self,
            committed: &crate::ops::GraphTransaction,
            changes: &NodeGraphChanges,
        ) {
            self.commits
                .borrow_mut()
                .push((committed.label.clone(), changes.nodes.len()));
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
    assert!(!undo.committed.ops.is_empty());

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
    assert!(!redo.committed.ops.is_empty());

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
}

#[derive(Clone)]
struct DeclarativeCallbackRecorder {
    trace: Rc<RefCell<DeclarativeCallbackTrace>>,
}

impl NodeGraphCommitCallbacks for DeclarativeCallbackRecorder {
    fn on_graph_commit(
        &mut self,
        committed: &crate::ops::GraphTransaction,
        _changes: &NodeGraphChanges,
    ) {
        self.trace
            .borrow_mut()
            .commit_labels
            .push(committed.label.clone());
    }
}

impl NodeGraphViewCallbacks for DeclarativeCallbackRecorder {
    fn on_selection_change(&mut self, sel: SelectionChange) {
        self.trace.borrow_mut().selection_changes.push(sel);
    }
}

impl NodeGraphGestureCallbacks for DeclarativeCallbackRecorder {}

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
    };
    let down = test_pointer_down(
        MouseButton::Left,
        Point::new(Px(12.0), Px(13.0)),
        Modifiers::default(),
    );

    let outcome = begin_left_pointer_down_action_host(
        &mut host, &marquee, &node_drag, &pending, &hovered, down, &snapshot,
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
    };
    let down = test_pointer_down(
        MouseButton::Left,
        Point::new(Px(20.0), Px(21.0)),
        Modifiers::default(),
    );

    let outcome = begin_left_pointer_down_action_host(
        &mut host, &marquee, &node_drag, &pending, &hovered, down, &snapshot,
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
    };
    let down = test_pointer_down(
        MouseButton::Left,
        Point::new(Px(30.0), Px(31.0)),
        Modifiers::default(),
    );

    let outcome = begin_left_pointer_down_action_host(
        &mut host, &marquee, &node_drag, &pending, &hovered, down, &snapshot,
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
        down,
        bounds,
    );

    assert_eq!(snapshot.hit, Some(node_a));
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

fn declarative_paint_only_runtime_sources() -> [(&'static str, &'static str); 21] {
    [
        ("paint_only.rs", include_str!("../paint_only.rs")),
        ("cache.rs", include_str!("cache.rs")),
        ("diag.rs", include_str!("diag.rs")),
        ("hover_anchor.rs", include_str!("hover_anchor.rs")),
        ("input_handlers.rs", include_str!("input_handlers.rs")),
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
        0.0,
    );

    assert_ne!(derived_a, derived_b);
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
    );
    let edges_b = edges_cache_key(
        graph_rev,
        selection_only_view.zoom,
        node_origin,
        draw_order_hash_b,
        derived_b.0,
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
