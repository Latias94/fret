use fret_core::{Point, Rect};
use fret_runtime::{Model, ModelStore};
use fret_ui::action::UiActionHost;
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::core::{CanvasPoint, EdgeId, Graph, GroupId, NodeId};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ops::GraphTransaction;
use crate::runtime::lookups::HandleConnection;
use crate::runtime::store::{DispatchOutcome, NodeGraphStore};

use super::controller::{
    NodeGraphController, NodeGraphControllerError, NodeGraphEdgeUpdate,
    NodeGraphNodeConnectionsQuery, NodeGraphNodeUpdate, NodeGraphPortConnectionsQuery,
};
use super::declarative::NodeGraphSurfaceProps;
use super::viewport_options::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions};

#[path = "binding_queries.rs"]
mod binding_queries;
#[path = "binding_store_sync.rs"]
mod binding_store_sync;
#[path = "binding_viewport.rs"]
mod binding_viewport;

/// Canonical app-facing binding bundle for the declarative node-graph surface.
///
/// This keeps the controller-first public story explicit while avoiding repeated
/// `graph + view_state + controller` triplets in app code.
#[derive(Debug, Clone)]
pub struct NodeGraphSurfaceBinding {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    editor_config: Model<NodeGraphEditorConfig>,
    store: Model<NodeGraphStore>,
}

impl NodeGraphSurfaceBinding {
    /// Creates a store-backed binding with an explicit editor configuration payload.
    pub fn new(
        models: &mut ModelStore,
        graph: Graph,
        view_state: NodeGraphViewState,
        editor_config: NodeGraphEditorConfig,
    ) -> Self {
        let graph_model = models.insert(graph.clone());
        let view_state_model = models.insert(view_state.clone());
        let editor_config_model = models.insert(editor_config.clone());
        let store = models.insert(NodeGraphStore::new(graph, view_state, editor_config));
        Self::from_models_and_controller(
            graph_model,
            view_state_model,
            editor_config_model,
            NodeGraphController::new(store),
        )
    }

    /// Creates a declarative surface binding from an already-configured store.
    pub fn from_store(models: &mut ModelStore, store: NodeGraphStore) -> Self {
        let graph = store.graph().clone();
        let view_state = store.view_state().clone();
        let editor_config = store.editor_config();
        let store_model = models.insert(store);
        let graph_model = models.insert(graph);
        let view_state_model = models.insert(view_state);
        let editor_config_model = models.insert(editor_config);
        Self::from_models_and_controller(
            graph_model,
            view_state_model,
            editor_config_model,
            NodeGraphController::new(store_model),
        )
    }

    /// Advanced seam for callers that already own explicit graph/view mirrors and controller state.
    pub fn from_models_and_controller(
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        editor_config: Model<NodeGraphEditorConfig>,
        controller: NodeGraphController,
    ) -> Self {
        Self {
            graph,
            view_state,
            editor_config,
            store: controller.store(),
        }
    }

    pub fn graph_model(&self) -> Model<Graph> {
        self.graph.clone()
    }

    pub fn view_state_model(&self) -> Model<NodeGraphViewState> {
        self.view_state.clone()
    }

    pub fn editor_config_model(&self) -> Model<NodeGraphEditorConfig> {
        self.editor_config.clone()
    }

    /// Advanced lower-level seam for callers that need explicit controller ownership.
    ///
    /// Typical app code should stay on the binding surface. Retained/compat composition can derive
    /// a fresh `NodeGraphController` explicitly from this store handle.
    pub fn store_model(&self) -> Model<NodeGraphStore> {
        self.store.clone()
    }

    pub fn surface_props(&self) -> NodeGraphSurfaceProps {
        NodeGraphSurfaceProps::new(self.clone())
    }

    fn controller(&self) -> NodeGraphController {
        NodeGraphController::new(self.store.clone())
    }

    /// Observes the external graph/view mirrors that the declarative surface keeps in sync.
    pub fn observe<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
        cx.observe_model(&self.editor_config, Invalidation::Paint);
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use super::NodeGraphSurfaceBinding;
    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey, StickyNote,
        StickyNoteId,
    };
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ops::{GraphOp, GraphTransaction};
    use crate::runtime::fit_view::{
        FitViewComputeOptions, FitViewNodeInfo, compute_fit_view_target,
        compute_fit_view_target_for_canvas_rect,
    };
    use crate::runtime::store::NodeGraphStore;
    use crate::ui::NodeGraphFitViewOptions;
    use fret_core::AppWindowId;
    use fret_core::{Point, Px, Rect, Size};
    use fret_runtime::{
        ClipboardToken, CommandRegistry, DragKindId, DragSession, DragSessionId, Effect, FrameId,
        ModelHost, ModelStore, ShareSheetToken, TickId, TimerToken,
    };
    use serde_json::Value;

    #[derive(Default)]
    struct TestActionHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        effects: Vec<Effect>,
        drag: Option<DragSession>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
    }

    impl fret_runtime::GlobalsHost for TestActionHost {
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
            if !self.globals.contains_key(&type_id) {
                self.globals.insert(type_id, Box::new(init()));
            }
            let boxed = self
                .globals
                .remove(&type_id)
                .expect("global must exist")
                .downcast::<T>()
                .ok()
                .expect("global has wrong type");
            let mut value = *boxed;
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestActionHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl fret_runtime::ModelsHost for TestActionHost {
        fn take_changed_models(&mut self) -> Vec<fret_runtime::ModelId> {
            self.models.take_changed_models()
        }
    }

    impl fret_runtime::CommandsHost for TestActionHost {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl fret_ui::action::UiActionHost for TestActionHost {
        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn request_redraw(&mut self, _window: AppWindowId) {}

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
    }

    impl fret_runtime::EffectSink for TestActionHost {
        fn request_redraw(&mut self, _window: AppWindowId) {}

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl fret_runtime::TimeHost for TestActionHost {
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

        fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            fret_runtime::ImageUploadToken(self.next_image_upload_token)
        }
    }

    impl fret_runtime::DragHost for TestActionHost {
        fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&DragSession> {
            self.drag
                .as_ref()
                .filter(|drag| drag.pointer_id == pointer_id)
        }

        fn drag_mut(&mut self, pointer_id: fret_core::PointerId) -> Option<&mut DragSession> {
            self.drag
                .as_mut()
                .filter(|drag| drag.pointer_id == pointer_id)
        }

        fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
            if self.drag(pointer_id).is_some() {
                self.drag = None;
            }
        }

        fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
            self.drag.as_ref().is_some_and(|drag| predicate(drag))
        }

        fn find_drag_pointer_id(
            &self,
            mut predicate: impl FnMut(&DragSession) -> bool,
        ) -> Option<fret_core::PointerId> {
            self.drag
                .as_ref()
                .filter(|drag| predicate(drag))
                .map(|drag| drag.pointer_id)
        }

        fn cancel_drag_sessions(
            &mut self,
            mut predicate: impl FnMut(&DragSession) -> bool,
        ) -> Vec<fret_core::PointerId> {
            let Some(drag) = self.drag.as_ref() else {
                return Vec::new();
            };
            if !predicate(drag) {
                return Vec::new();
            }
            let pointer_id = drag.pointer_id;
            self.drag = None;
            vec![pointer_id]
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.drag = Some(DragSession::new(
                DragSessionId(1),
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ));
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.drag = Some(DragSession::new_cross_window(
                DragSessionId(1),
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ));
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

    fn test_node_with_size(pos: CanvasPoint, size: CanvasSize) -> Node {
        let mut node = test_node(pos);
        node.size = Some(size);
        node
    }

    #[test]
    fn new_binding_seeds_graph_view_and_store_models() {
        let mut models = ModelStore::default();
        let graph = Graph::new(GraphId::from_u128(0x9001));
        let view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 12.0, y: 34.0 },
            zoom: 1.5,
            ..NodeGraphViewState::default()
        };

        let binding = NodeGraphSurfaceBinding::new(
            &mut models,
            graph.clone(),
            view_state.clone(),
            NodeGraphEditorConfig::default(),
        );

        let graph_id = models
            .read(&binding.graph_model(), |value| value.graph_id)
            .unwrap();
        let pan = models
            .read(&binding.view_state_model(), |value| value.pan)
            .unwrap();
        let zoom = models
            .read(&binding.store_model(), |store| store.view_state().zoom)
            .unwrap();

        assert_eq!(graph_id, graph.graph_id);
        assert_eq!(pan, view_state.pan);
        assert_eq!(zoom, view_state.zoom);
    }

    #[test]
    fn from_store_clones_initial_store_state_into_surface_models() {
        let mut models = ModelStore::default();
        let graph = Graph::new(GraphId::from_u128(0x9002));
        let view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 88.0, y: 55.0 },
            zoom: 0.75,
            ..NodeGraphViewState::default()
        };
        let store = NodeGraphStore::new(
            graph.clone(),
            view_state.clone(),
            NodeGraphEditorConfig::default(),
        );

        let binding = NodeGraphSurfaceBinding::from_store(&mut models, store);

        let graph_id = models
            .read(&binding.graph_model(), |value| value.graph_id)
            .unwrap();
        let pan = models
            .read(&binding.view_state_model(), |value| value.pan)
            .unwrap();
        let zoom = models
            .read(&binding.store_model(), |store| store.view_state().zoom)
            .unwrap();

        assert_eq!(graph_id, graph.graph_id);
        assert_eq!(pan, view_state.pan);
        assert_eq!(zoom, view_state.zoom);
    }

    #[test]
    fn dispatch_transaction_action_host_and_history_sync_bound_models() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x9003)),
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );

        let note_id = StickyNoteId::from_u128(0x9004);
        let note = StickyNote {
            text: "note".into(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 10.0, y: 20.0 },
                size: CanvasSize {
                    width: 120.0,
                    height: 80.0,
                },
            },
            color: Some("amber".into()),
        };
        let mut tx = GraphTransaction::new();
        tx.push(GraphOp::AddStickyNote {
            id: note_id,
            note: note.clone(),
        });

        let outcome = binding
            .dispatch_transaction_action_host(&mut host, &tx)
            .expect("dispatch succeeds");
        assert_eq!(outcome.committed.ops.len(), 1);
        assert!(matches!(
            outcome.committed.ops.first(),
            Some(GraphOp::AddStickyNote { id, .. }) if *id == note_id
        ));
        assert!(outcome.changes.nodes.is_empty());
        assert!(outcome.changes.edges.is_empty());
        assert!(
            host.models
                .read(&binding.graph_model(), |graph| graph
                    .sticky_notes
                    .contains_key(&note_id))
                .expect("graph model readable")
        );

        let undo = binding.undo_action_host(&mut host).expect("undo succeeds");
        assert!(undo.is_some());
        assert!(
            !host
                .models
                .read(&binding.graph_model(), |graph| graph
                    .sticky_notes
                    .contains_key(&note_id))
                .expect("graph model readable")
        );

        let redo = binding.redo_action_host(&mut host).expect("redo succeeds");
        assert!(redo.is_some());
        assert!(
            host.models
                .read(&binding.graph_model(), |graph| graph
                    .sticky_notes
                    .contains_key(&note_id))
                .expect("graph model readable")
        );
    }

    #[test]
    fn replace_graph_and_selection_action_host_sync_bound_models() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x9005)),
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );

        let node_id = NodeId::from_u128(0x9006);
        let mut graph = Graph::new(GraphId::from_u128(0x9005));
        graph
            .nodes
            .insert(node_id, test_node(CanvasPoint { x: 64.0, y: 96.0 }));

        binding
            .replace_graph_action_host(&mut host, graph)
            .expect("replace graph succeeds");
        assert!(
            host.models
                .read(&binding.graph_model(), |value| value
                    .nodes
                    .contains_key(&node_id))
                .expect("graph model readable")
        );

        binding
            .set_selection_action_host(&mut host, vec![node_id], Vec::new(), Vec::new())
            .expect("set selection succeeds");
        assert_eq!(
            host.models
                .read(&binding.view_state_model(), |state| state
                    .selected_nodes
                    .clone())
                .expect("view model readable"),
            vec![node_id]
        );
    }

    #[test]
    fn replace_view_state_action_host_syncs_bound_view_model() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x9007)),
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );

        let next = NodeGraphViewState {
            pan: CanvasPoint { x: 21.0, y: 34.0 },
            zoom: 1.75,
            ..NodeGraphViewState::default()
        };

        binding
            .replace_view_state_action_host(&mut host, next.clone())
            .expect("replace view state succeeds");

        let (pan, zoom) = host
            .models
            .read(&binding.view_state_model(), |state| (state.pan, state.zoom))
            .expect("view model readable");
        assert_eq!(pan, next.pan);
        assert_eq!(zoom, next.zoom);
    }

    #[test]
    fn replace_document_action_host_preserves_explicit_editor_config_mirror() {
        let mut host = TestActionHost::default();
        let mut editor_config = NodeGraphEditorConfig::default();
        editor_config.interaction.selection_on_drag = true;
        editor_config.runtime_tuning.only_render_visible_elements = false;
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x9008)),
            NodeGraphViewState::default(),
            editor_config.clone(),
        );
        let next_graph = Graph::new(GraphId::from_u128(0x9009));
        let next_view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 48.0, y: 24.0 },
            zoom: 2.0,
            ..NodeGraphViewState::default()
        };

        binding
            .replace_document_action_host(&mut host, next_graph.clone(), next_view_state.clone())
            .expect("replace document succeeds");

        let store_editor_config = host
            .models
            .read(&binding.store_model(), |store| store.editor_config())
            .expect("store editor config readable");
        let bound_editor_config = host
            .models
            .read(&binding.editor_config_model(), |config| config.clone())
            .expect("binding editor config readable");
        let graph_id = host
            .models
            .read(&binding.graph_model(), |graph| graph.graph_id)
            .expect("graph model readable");
        let (pan, zoom) = host
            .models
            .read(&binding.view_state_model(), |state| (state.pan, state.zoom))
            .expect("view model readable");

        assert_eq!(store_editor_config, editor_config);
        assert_eq!(bound_editor_config, editor_config);
        assert_eq!(graph_id, next_graph.graph_id);
        assert_eq!(pan, next_view_state.pan);
        assert_eq!(zoom, next_view_state.zoom);
    }

    #[test]
    fn viewport_queries_use_authoritative_store_viewport() {
        let mut host = TestActionHost::default();
        let view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 10.0, y: 20.0 },
            zoom: 2.0,
            ..NodeGraphViewState::default()
        };
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x900a)),
            view_state,
            NodeGraphEditorConfig::default(),
        );
        host.models
            .update(&binding.view_state_model(), |state| {
                state.pan = CanvasPoint {
                    x: -999.0,
                    y: -999.0,
                };
                state.zoom = 0.5;
            })
            .expect("stale view model update");

        assert_eq!(
            binding.viewport(&host),
            (CanvasPoint { x: 10.0, y: 20.0 }, 2.0)
        );

        let bounds = Rect::new(
            Point::new(Px(100.0), Px(50.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let canvas = binding
            .screen_to_canvas(&host, bounds, Point::new(Px(300.0), Px(250.0)))
            .expect("screen projection");
        assert!((canvas.x - 90.0).abs() <= 1.0e-6);
        assert!((canvas.y - 80.0).abs() <= 1.0e-6);

        let screen = binding
            .canvas_to_screen(&host, bounds, CanvasPoint { x: 90.0, y: 80.0 })
            .expect("canvas projection");
        assert!((screen.x.0 - 300.0).abs() <= 1.0e-6);
        assert!((screen.y.0 - 250.0).abs() <= 1.0e-6);
    }

    #[test]
    fn set_viewport_action_host_syncs_bound_view_model() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x900b)),
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );

        assert!(binding.set_viewport_action_host(
            &mut host,
            CanvasPoint { x: -12.0, y: 8.0 },
            1.75,
        ));

        let (pan, zoom) = host
            .models
            .read(&binding.view_state_model(), |state| (state.pan, state.zoom))
            .expect("view model readable");
        assert_eq!(pan, CanvasPoint { x: -12.0, y: 8.0 });
        assert_eq!(zoom, 1.75);
    }

    #[test]
    fn set_center_in_bounds_action_host_syncs_bound_view_model() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x900c)),
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        assert!(binding.set_center_in_bounds_action_host(
            &mut host,
            bounds,
            CanvasPoint { x: 10.0, y: 20.0 },
        ));

        let (pan, zoom) = host
            .models
            .read(&binding.view_state_model(), |state| (state.pan, state.zoom))
            .expect("view model readable");
        assert_eq!(pan, CanvasPoint { x: 390.0, y: 280.0 });
        assert_eq!(zoom, 1.0);
    }

    #[test]
    fn fit_view_nodes_in_bounds_action_host_syncs_bound_view_model() {
        let mut host = TestActionHost::default();
        let mut graph = Graph::new(GraphId::from_u128(0x900d));
        let node_a = NodeId::from_u128(0x900e);
        let node_b = NodeId::from_u128(0x900f);
        graph.nodes.insert(
            node_a,
            test_node_with_size(
                CanvasPoint { x: 0.0, y: 0.0 },
                CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            ),
        );
        graph.nodes.insert(
            node_b,
            test_node_with_size(
                CanvasPoint { x: 200.0, y: 0.0 },
                CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            ),
        );
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            graph,
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let options = NodeGraphFitViewOptions {
            padding: Some(0.0),
            ..NodeGraphFitViewOptions::default()
        };

        assert!(binding.fit_view_nodes_in_bounds_with_options_action_host(
            &mut host,
            bounds,
            vec![node_a, node_b],
            options,
        ));

        let expected = compute_fit_view_target(
            &[
                FitViewNodeInfo {
                    pos: CanvasPoint { x: 0.0, y: 0.0 },
                    size_px: (100.0, 100.0),
                },
                FitViewNodeInfo {
                    pos: CanvasPoint { x: 200.0, y: 0.0 },
                    size_px: (100.0, 100.0),
                },
            ],
            FitViewComputeOptions {
                viewport_width_px: 800.0,
                viewport_height_px: 600.0,
                node_origin: (0.0, 0.0),
                padding: 0.0,
                margin_px_fallback: 48.0,
                min_zoom: 0.05,
                max_zoom: 64.0,
            },
        )
        .expect("fit-view target");
        let (pan, zoom) = host
            .models
            .read(&binding.view_state_model(), |state| (state.pan, state.zoom))
            .expect("view model readable");
        assert!((pan.x - expected.0.x).abs() <= 1.0e-6);
        assert!((pan.y - expected.0.y).abs() <= 1.0e-6);
        assert!((zoom - expected.1).abs() <= 1.0e-6);
    }

    #[test]
    fn fit_canvas_rect_in_bounds_action_host_syncs_bound_view_model() {
        let mut host = TestActionHost::default();
        let binding = NodeGraphSurfaceBinding::new(
            &mut host.models,
            Graph::new(GraphId::from_u128(0x9010)),
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let target_canvas = Rect::new(
            Point::new(Px(100.0), Px(50.0)),
            Size::new(Px(400.0), Px(200.0)),
        );
        let options = NodeGraphFitViewOptions {
            padding: Some(0.0),
            ..NodeGraphFitViewOptions::default()
        };

        assert!(binding.fit_canvas_rect_in_bounds_with_options_action_host(
            &mut host,
            bounds,
            target_canvas,
            options,
        ));

        let expected = compute_fit_view_target_for_canvas_rect(
            target_canvas,
            FitViewComputeOptions {
                viewport_width_px: 800.0,
                viewport_height_px: 600.0,
                node_origin: (0.0, 0.0),
                padding: 0.0,
                margin_px_fallback: 48.0,
                min_zoom: 0.05,
                max_zoom: 64.0,
            },
        )
        .expect("fit-rect target");
        let (pan, zoom) = host
            .models
            .read(&binding.view_state_model(), |state| (state.pan, state.zoom))
            .expect("view model readable");
        assert!((pan.x - expected.0.x).abs() <= 1.0e-6);
        assert!((pan.y - expected.0.y).abs() <= 1.0e-6);
        assert!((zoom - expected.1).abs() <= 1.0e-6);
    }
}
