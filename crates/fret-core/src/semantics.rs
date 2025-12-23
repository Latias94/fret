use crate::{AppWindowId, NodeId, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsRole {
    Generic,
    Window,
    Panel,
    Button,
    Tab,
    Menu,
    MenuItem,
    Text,
    TextField,
    List,
    ListItem,
    Viewport,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SemanticsFlags {
    pub focused: bool,
    pub captured: bool,
    pub disabled: bool,
    pub selected: bool,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub struct SemanticsNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub role: SemanticsRole,
    pub bounds: Rect,
    pub flags: SemanticsFlags,
}

#[derive(Debug, Clone)]
pub struct SemanticsRoot {
    pub root: NodeId,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    /// Paint order index within the window (0 = back/bottom).
    pub z_index: u32,
}

#[derive(Debug, Default, Clone)]
pub struct SemanticsSnapshot {
    pub window: AppWindowId,
    pub roots: Vec<SemanticsRoot>,
    /// The root of the topmost modal layer (if any), matching ADR 0011/0033 semantics gating.
    pub barrier_root: Option<NodeId>,
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub nodes: Vec<SemanticsNode>,
}
