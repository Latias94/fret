use fret_core::NodeId;

/// Window-level pointer occlusion mode published by the UI runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowPointerOcclusion {
    /// No occlusion; pointer events route normally via hit-testing.
    #[default]
    None,
    /// Blocks pointer interaction (hover/move/down/up) for underlay roots.
    BlockMouse,
    /// Blocks pointer interaction for underlay roots, but allows scroll wheel forwarding.
    BlockMouseExceptScroll,
}

/// Window-scoped input arbitration snapshot published by the UI runtime.
///
/// This is a data-only integration seam used by policy-heavy ecosystem crates and runner/platform
/// layers that need to observe modal/capture/occlusion state without depending on `fret-ui`
/// internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowInputArbitrationSnapshot {
    pub modal_barrier_root: Option<NodeId>,
    pub pointer_occlusion: WindowPointerOcclusion,
    pub pointer_occlusion_root: Option<NodeId>,
    pub pointer_capture_active: bool,
    /// When all captured pointers belong to the same layer root, this reports that root.
    pub pointer_capture_root: Option<NodeId>,
    /// `true` when capture spans multiple roots or cannot be mapped to a single root.
    pub pointer_capture_multiple_roots: bool,
}
