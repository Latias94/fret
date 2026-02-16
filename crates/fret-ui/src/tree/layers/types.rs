use super::super::*;

slotmap::new_key_type! {
    pub struct UiLayerId;
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
