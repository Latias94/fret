use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) const REROUTE_INPUTS: usize = 1;
    pub(in super::super) const REROUTE_OUTPUTS: usize = 1;
    pub(in super::super) const AUTO_PAN_TICK_HZ: f32 = 60.0;
    pub(in super::super) const AUTO_PAN_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::AUTO_PAN_TICK_HZ) as u64);
    pub(in super::super) const PAN_INERTIA_TICK_HZ: f32 = 60.0;
    pub(in super::super) const PAN_INERTIA_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::PAN_INERTIA_TICK_HZ) as u64);
    pub(in super::super) const VIEWPORT_MOVE_END_DEBOUNCE: Duration = Duration::from_millis(180);
    pub(in super::super) const EDGE_FOCUS_ANCHOR_SIZE_SCREEN: f32 = 16.0;
    pub(in super::super) const EDGE_FOCUS_ANCHOR_PAD_SCREEN: f32 = 1.0;
    pub(in super::super) const EDGE_FOCUS_ANCHOR_BORDER_SCREEN: f32 = 2.0;
    pub(in super::super) const EDGE_FOCUS_ANCHOR_OFFSET_SCREEN: f32 = 18.0;
    pub(in super::super) const GRID_TILE_SIZE_SCREEN_PX: f32 = 2048.0;
    pub(in super::super) const GRID_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(32, 8);
    pub(in super::super) const EDGE_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(4, 1);
    pub(in super::super) const EDGE_LABEL_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(2, 1);
    pub(in super::super) const EDGE_WIRE_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(in super::super) const EDGE_WIRE_HIGHLIGHT_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(in super::super) const EDGE_WIRE_OUTLINE_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(in super::super) const EDGE_MARKER_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(96, 24);
    pub(in super::super) const EDGE_LABEL_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(16, 4);
    pub(in super::super) const STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES: u64 = 60 * 30;
    pub(in super::super) const STATIC_SCENE_TILE_CACHE_MAX_ENTRIES: usize = 16;
}
