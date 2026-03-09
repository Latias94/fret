use super::*;

#[path = "widget_surface/builders.rs"]
mod builders;
#[path = "widget_surface/construct.rs"]
mod construct;
#[path = "widget_surface/fit_view.rs"]
mod fit_view;
#[path = "widget_surface/runtime.rs"]
mod runtime;
#[path = "widget_surface/sync.rs"]
mod sync;

impl NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
    pub fn new(graph: Model<Graph>, view_state: Model<NodeGraphViewState>) -> Self {
        Self::new_with_middleware(graph, view_state, NoopNodeGraphCanvasMiddleware)
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) const REROUTE_INPUTS: usize = 1;
    pub(super) const REROUTE_OUTPUTS: usize = 1;
    pub(super) const AUTO_PAN_TICK_HZ: f32 = 60.0;
    pub(super) const AUTO_PAN_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::AUTO_PAN_TICK_HZ) as u64);
    pub(super) const PAN_INERTIA_TICK_HZ: f32 = 60.0;
    pub(super) const PAN_INERTIA_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::PAN_INERTIA_TICK_HZ) as u64);
    pub(super) const VIEWPORT_MOVE_END_DEBOUNCE: Duration = Duration::from_millis(180);
    pub(super) const EDGE_FOCUS_ANCHOR_SIZE_SCREEN: f32 = 16.0;
    pub(super) const EDGE_FOCUS_ANCHOR_PAD_SCREEN: f32 = 1.0;
    pub(super) const EDGE_FOCUS_ANCHOR_BORDER_SCREEN: f32 = 2.0;
    pub(super) const EDGE_FOCUS_ANCHOR_OFFSET_SCREEN: f32 = 18.0;
    pub(super) const GRID_TILE_SIZE_SCREEN_PX: f32 = 2048.0;
    pub(super) const GRID_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(32, 8);
    pub(super) const EDGE_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(4, 1);
    pub(super) const EDGE_LABEL_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(2, 1);
    pub(super) const EDGE_WIRE_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(super) const EDGE_WIRE_HIGHLIGHT_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(super) const EDGE_WIRE_OUTLINE_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(super) const EDGE_MARKER_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(96, 24);
    pub(super) const EDGE_LABEL_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(16, 4);
    pub(super) const STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES: u64 = 60 * 30;
    pub(super) const STATIC_SCENE_TILE_CACHE_MAX_ENTRIES: usize = 16;

    pub fn with_presenter(mut self, presenter: impl NodeGraphPresenter + 'static) -> Self {
        self.presenter = Box::new(FallbackMeasuredNodeGraphPresenter::new(
            presenter,
            self.auto_measured.clone(),
        ));
        self
    }

    pub fn with_skin(mut self, skin: NodeGraphSkinRef) -> Self {
        self.skin = Some(skin);
        self.skin_last_rev = None;
        self
    }

    pub fn with_paint_overrides(mut self, overrides: NodeGraphPaintOverridesRef) -> Self {
        self.paint_overrides = Some(overrides);
        self.paint_overrides_last_rev = None;
        self
    }

    pub fn with_edge_types(mut self, edge_types: NodeGraphEdgeTypes) -> Self {
        self.edge_types = Some(edge_types);
        self
    }

    pub fn with_callbacks(mut self, callbacks: impl NodeGraphCallbacks) -> Self {
        self.callbacks = Some(Box::new(callbacks));
        self
    }

    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = Some(command);
        self
    }
}
