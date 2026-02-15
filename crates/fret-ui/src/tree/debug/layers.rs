use super::super::*;

/// Controls whether an overlay layer prevents pointer interactions from reaching layers beneath it.
///
/// This is a *mechanism* only. Policy lives in ecosystem crates (e.g. `fret-ui-kit`), which decide
/// when to enable occlusion (Radix `disableOutsidePointerEvents` outcomes, editor interaction
/// arbitration, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerOcclusion {
    /// No occlusion; pointer events route normally via hit-testing across layers.
    #[default]
    None,
    /// Blocks pointer interaction (hover/move/down/up) for layers beneath the occluding layer.
    BlockMouse,
    /// Blocks pointer interaction for layers beneath the occluding layer, but allows scroll wheel
    /// to route to underlay scroll targets.
    BlockMouseExceptScroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiInputArbitrationSnapshot {
    pub modal_barrier_root: Option<NodeId>,
    pub focus_barrier_root: Option<NodeId>,
    pub pointer_occlusion: PointerOcclusion,
    pub pointer_occlusion_layer: Option<UiLayerId>,
    pub pointer_capture_active: bool,
    /// When all captured pointers belong to the same layer, this reports that layer.
    ///
    /// If captures span multiple layers (or a captured node cannot be mapped to a layer), this is
    /// `None` and `pointer_capture_multiple_layers=true`.
    pub pointer_capture_layer: Option<UiLayerId>,
    pub pointer_capture_multiple_layers: bool,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayerInfo {
    pub id: UiLayerId,
    pub root: NodeId,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    pub pointer_occlusion: PointerOcclusion,
    pub wants_pointer_down_outside_events: bool,
    pub consume_pointer_down_outside_events: bool,
    pub pointer_down_outside_branches: Vec<NodeId>,
    pub wants_pointer_move_events: bool,
    pub wants_timer_events: bool,
}

#[derive(Debug, Clone)]
pub struct UiDebugHitTest {
    pub hit: Option<NodeId>,
    pub active_layer_roots: Vec<NodeId>,
    pub barrier_root: Option<NodeId>,
}
