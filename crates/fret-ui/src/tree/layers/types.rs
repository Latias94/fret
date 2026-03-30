use super::super::*;

slotmap::new_key_type! {
    pub struct UiLayerId;
}

/// Stable mechanism contract for installing an overlay root into a window-scoped [`UiTree`].
///
/// The runtime only owns the layer/root substrate. Higher-level overlay policy stays in
/// `ecosystem/*` and can refine the layer after installation (for example by changing focus or
/// pointer-occlusion behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayRootOptions {
    /// Whether this layer blocks input to underlay roots when visible.
    ///
    /// On install, this also seeds `blocks_underlay_focus` to the same value. Policy code can
    /// diverge the focus barrier later via `UiTree::set_layer_blocks_underlay_focus(...)`.
    pub blocks_underlay_input: bool,
    /// Whether this layer participates in hit testing inside its own subtree.
    pub hit_testable: bool,
}

impl OverlayRootOptions {
    pub const fn new(blocks_underlay_input: bool) -> Self {
        Self {
            blocks_underlay_input,
            hit_testable: true,
        }
    }
}

impl Default for OverlayRootOptions {
    fn default() -> Self {
        Self::new(false)
    }
}

#[derive(Debug, Clone)]
pub(in crate::tree) struct UiLayer {
    pub(in crate::tree) root: NodeId,
    pub(in crate::tree) visible: bool,
    pub(in crate::tree) blocks_underlay_input: bool,
    pub(in crate::tree) blocks_underlay_focus: bool,
    pub(in crate::tree) hit_testable: bool,
    pub(in crate::tree) pointer_occlusion: PointerOcclusion,
    pub(in crate::tree) wants_pointer_down_outside_events: bool,
    pub(in crate::tree) consume_pointer_down_outside_events: bool,
    pub(in crate::tree) pointer_down_outside_branches: Vec<NodeId>,
    /// Elements that should cause this overlay to dismiss when they are inside the scroll-event
    /// target (Radix Tooltip "close on scroll" outcome).
    pub(in crate::tree) scroll_dismiss_elements: Vec<crate::GlobalElementId>,
    pub(in crate::tree) wants_pointer_move_events: bool,
    pub(in crate::tree) wants_timer_events: bool,
}
