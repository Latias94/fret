use crate::{AppWindowId, NodeId, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsRole {
    Generic,
    Window,
    Panel,
    Dialog,
    Alert,
    Button,
    Checkbox,
    Switch,
    Slider,
    ComboBox,
    Tab,
    MenuBar,
    Menu,
    MenuItem,
    Text,
    TextField,
    List,
    ListItem,
    TreeItem,
    Viewport,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SemanticsActions {
    pub focus: bool,
    pub invoke: bool,
    pub set_value: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SemanticsFlags {
    pub focused: bool,
    pub captured: bool,
    pub disabled: bool,
    pub selected: bool,
    pub expanded: bool,
    /// Tri-state checked state (None = not checkable / unknown).
    pub checked: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct SemanticsNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub role: SemanticsRole,
    pub bounds: Rect,
    pub flags: SemanticsFlags,
    /// When this node retains actual keyboard focus but another descendant is the current
    /// "active item" (e.g. composite widgets using `aria-activedescendant`), this points to that
    /// active descendant node.
    pub active_descendant: Option<NodeId>,
    /// Human-readable name/label for assistive technologies.
    pub label: Option<String>,
    /// Value text, typically for text fields and sliders.
    pub value: Option<String>,
    /// Supported actions for assistive technologies and automation.
    pub actions: SemanticsActions,
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
