use std::collections::HashSet;

use crate::{AppWindowId, NodeId, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsRole {
    Generic,
    Window,
    Panel,
    Group,
    Toolbar,
    Heading,
    Dialog,
    AlertDialog,
    Alert,
    Button,
    Link,
    Image,
    Checkbox,
    Switch,
    Slider,
    SpinButton,
    ProgressBar,
    Meter,
    ScrollBar,
    Splitter,
    ComboBox,
    RadioGroup,
    RadioButton,
    TabList,
    Tab,
    TabPanel,
    MenuBar,
    Menu,
    MenuItem,
    MenuItemCheckbox,
    MenuItemRadio,
    Tooltip,
    Text,
    TextField,
    List,
    ListItem,
    Separator,
    ListBox,
    ListBoxOption,
    TreeItem,
    Viewport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SemanticsActions {
    pub focus: bool,
    pub invoke: bool,
    pub set_value: bool,
    /// Decrement a numeric value by one step.
    pub decrement: bool,
    /// Increment a numeric value by one step.
    pub increment: bool,
    pub scroll_by: bool,
    pub set_text_selection: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsCheckedState {
    False,
    True,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsPressedState {
    False,
    True,
    Mixed,
}

/// Indicates if a form control has invalid input (ARIA `aria-invalid` class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsInvalid {
    True,
    Grammar,
    Spelling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsLive {
    Off,
    Polite,
    Assertive,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SemanticsFlags {
    pub focused: bool,
    pub captured: bool,
    pub disabled: bool,
    pub read_only: bool,
    /// Exclude this node (and its subtree) from the accessibility tree presented to assistive
    /// technologies.
    ///
    /// This is a portable approximation of ARIA `aria-hidden`.
    pub hidden: bool,
    /// Indicates that a link has been visited.
    ///
    /// This is a portable approximation of the "visited link" concept in HTML.
    pub visited: bool,
    /// Indicates that this collection supports selecting multiple items.
    ///
    /// This is a portable approximation of ARIA `aria-multiselectable`.
    pub multiselectable: bool,
    /// When set, indicates that this node is a live region (ARIA `aria-live`).
    ///
    /// `None` means no live region semantics are requested.
    pub live: Option<SemanticsLive>,
    /// When true, indicates that updates to this live region should be presented atomically
    /// (ARIA `aria-atomic`).
    pub live_atomic: bool,
    pub selected: bool,
    pub expanded: bool,
    /// Legacy binary checked state.
    ///
    /// Prefer `checked_state` for tri-state widgets.
    pub checked: Option<bool>,
    /// Tri-state checked state (None = not checkable / unknown).
    pub checked_state: Option<SemanticsCheckedState>,
    /// Tri-state pressed state for toggle-button-like widgets (None = not a toggle / unknown).
    pub pressed_state: Option<SemanticsPressedState>,
    /// Indicates that a form field is required to be filled in.
    pub required: bool,
    /// Indicates that a form control has invalid input.
    pub invalid: Option<SemanticsInvalid>,
    /// Indicates that this node (and typically its subtree) is currently busy (e.g. loading).
    ///
    /// This is a portable approximation of ARIA `aria-busy`.
    pub busy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticsInlineSpan {
    /// UTF-8 byte range `(start, end)` into `SemanticsNode::value`.
    pub range_utf8: (u32, u32),
    pub role: SemanticsRole,
    /// Opaque, component-defined tag (e.g. a URL for markdown links).
    pub tag: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SemanticsNumeric {
    pub value: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub jump: Option<f64>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SemanticsScroll {
    pub x: Option<f64>,
    pub x_min: Option<f64>,
    pub x_max: Option<f64>,
    pub y: Option<f64>,
    pub y_min: Option<f64>,
    pub y_max: Option<f64>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SemanticsNodeExtra {
    pub placeholder: Option<String>,
    pub url: Option<String>,
    /// Optional hierarchy level for outline/tree semantics (1-based).
    pub level: Option<u32>,
    pub orientation: Option<SemanticsOrientation>,
    pub numeric: SemanticsNumeric,
    pub scroll: SemanticsScroll,
}

#[derive(Debug, Clone)]
pub struct SemanticsNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub role: SemanticsRole,
    pub bounds: Rect,
    pub flags: SemanticsFlags,
    /// Debug/test-only identifier for deterministic automation.
    ///
    /// This MUST NOT be mapped into platform accessibility name/label fields by default.
    pub test_id: Option<String>,
    /// When this node retains actual keyboard focus but another descendant is the current
    /// "active item" (e.g. composite widgets using `aria-activedescendant`), this points to that
    /// active descendant node.
    pub active_descendant: Option<NodeId>,
    /// 1-based position of this node within a logical collection (e.g. listbox/menu items).
    ///
    /// This is used to support accessible large/virtualized collections where only a window of
    /// items is present in the semantics snapshot.
    pub pos_in_set: Option<u32>,
    /// Total number of items in the logical collection that this node belongs to.
    ///
    /// This is used to support accessible large/virtualized collections where only a window of
    /// items is present in the semantics snapshot.
    pub set_size: Option<u32>,
    /// Human-readable name/label for assistive technologies.
    pub label: Option<String>,
    /// Value text, typically for text fields and sliders.
    pub value: Option<String>,
    pub extra: SemanticsNodeExtra,
    /// Text selection in UTF-8 byte offsets within `value` (ADR 0071).
    ///
    /// This is `(anchor, focus)` to preserve selection direction for assistive technologies.
    pub text_selection: Option<(u32, u32)>,
    /// IME composition range in UTF-8 byte offsets within `value` (ADR 0071).
    ///
    /// This is a best-effort signal for accessibility and may be omitted by implementations that
    /// cannot represent composition distinctly.
    pub text_composition: Option<(u32, u32)>,
    /// Supported actions for assistive technologies and automation.
    pub actions: SemanticsActions,
    /// Nodes which provide this node's accessible name.
    ///
    /// This is a portable approximation of relations such as `aria-labelledby`.
    pub labelled_by: Vec<NodeId>,
    /// Nodes which provide this node's accessible description.
    ///
    /// This is a portable approximation of relations such as `aria-describedby`.
    pub described_by: Vec<NodeId>,
    /// Nodes which this node controls.
    ///
    /// This is a portable approximation of relations such as `aria-controls`.
    pub controls: Vec<NodeId>,
    /// Inline semantics spans within this node's `value` (v1 metadata-only surface).
    pub inline_spans: Vec<SemanticsInlineSpan>,
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
    /// The root of the topmost focus-blocking layer (if any).
    ///
    /// This is intentionally decoupled from `barrier_root`: some overlay close transitions keep a
    /// pointer barrier active while releasing focus containment.
    pub focus_barrier_root: Option<NodeId>,
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub nodes: Vec<SemanticsNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticsValidationField {
    TextSelection,
    TextComposition,
    InlineSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticsNumericField {
    Value,
    Min,
    Max,
    Step,
    Jump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticsScrollField {
    X,
    XMin,
    XMax,
    Y,
    YMin,
    YMax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticsScrollAxis {
    X,
    Y,
}

#[derive(Debug, Clone)]
pub enum SemanticsValidationErrorKind {
    MissingValueForTextRange {
        field: SemanticsValidationField,
    },
    RangeOutOfBounds {
        field: SemanticsValidationField,
        start: u32,
        end: u32,
        len: u32,
    },
    RangeNotCharBoundary {
        field: SemanticsValidationField,
        offset: u32,
    },
    InvalidRangeOrder {
        field: SemanticsValidationField,
        start: u32,
        end: u32,
    },
    DuplicateNodeId {
        id: NodeId,
    },
    MissingReferencedNode {
        field: SemanticsReferenceField,
        referenced: NodeId,
    },
    InvalidCollectionMetadata {
        pos_in_set: Option<u32>,
        set_size: Option<u32>,
    },
    InvalidHierarchyLevel {
        level: u32,
    },
    NonFiniteNumeric {
        field: SemanticsNumericField,
        value: f64,
    },
    InvalidNumericBounds {
        min: f64,
        max: f64,
    },
    NumericValueOutOfBounds {
        value: f64,
        min: f64,
        max: f64,
    },
    InvalidNumericStep {
        step: f64,
    },
    InvalidNumericJump {
        jump: f64,
    },
    NonFiniteScroll {
        field: SemanticsScrollField,
        value: f64,
    },
    InvalidScrollBounds {
        axis: SemanticsScrollAxis,
        min: f64,
        max: f64,
    },
    ScrollValueOutOfBounds {
        axis: SemanticsScrollAxis,
        value: f64,
        min: f64,
        max: f64,
    },
}

impl PartialEq for SemanticsValidationErrorKind {
    fn eq(&self, other: &Self) -> bool {
        use SemanticsValidationErrorKind::*;
        match (self, other) {
            (MissingValueForTextRange { field: a }, MissingValueForTextRange { field: b }) => {
                a == b
            }
            (
                RangeOutOfBounds {
                    field: a_field,
                    start: a_start,
                    end: a_end,
                    len: a_len,
                },
                RangeOutOfBounds {
                    field: b_field,
                    start: b_start,
                    end: b_end,
                    len: b_len,
                },
            ) => a_field == b_field && a_start == b_start && a_end == b_end && a_len == b_len,
            (
                RangeNotCharBoundary {
                    field: a_field,
                    offset: a_offset,
                },
                RangeNotCharBoundary {
                    field: b_field,
                    offset: b_offset,
                },
            ) => a_field == b_field && a_offset == b_offset,
            (
                InvalidRangeOrder {
                    field: a_field,
                    start: a_start,
                    end: a_end,
                },
                InvalidRangeOrder {
                    field: b_field,
                    start: b_start,
                    end: b_end,
                },
            ) => a_field == b_field && a_start == b_start && a_end == b_end,
            (DuplicateNodeId { id: a }, DuplicateNodeId { id: b }) => a == b,
            (
                MissingReferencedNode {
                    field: a_field,
                    referenced: a_referenced,
                },
                MissingReferencedNode {
                    field: b_field,
                    referenced: b_referenced,
                },
            ) => a_field == b_field && a_referenced == b_referenced,
            (
                InvalidCollectionMetadata {
                    pos_in_set: a_pos_in_set,
                    set_size: a_set_size,
                },
                InvalidCollectionMetadata {
                    pos_in_set: b_pos_in_set,
                    set_size: b_set_size,
                },
            ) => a_pos_in_set == b_pos_in_set && a_set_size == b_set_size,
            (InvalidHierarchyLevel { level: a }, InvalidHierarchyLevel { level: b }) => a == b,
            (
                NonFiniteNumeric {
                    field: a_field,
                    value: a_value,
                },
                NonFiniteNumeric {
                    field: b_field,
                    value: b_value,
                },
            ) => a_field == b_field && a_value.to_bits() == b_value.to_bits(),
            (
                InvalidNumericBounds {
                    min: a_min,
                    max: a_max,
                },
                InvalidNumericBounds {
                    min: b_min,
                    max: b_max,
                },
            ) => a_min.to_bits() == b_min.to_bits() && a_max.to_bits() == b_max.to_bits(),
            (
                NumericValueOutOfBounds {
                    value: a_value,
                    min: a_min,
                    max: a_max,
                },
                NumericValueOutOfBounds {
                    value: b_value,
                    min: b_min,
                    max: b_max,
                },
            ) => {
                a_value.to_bits() == b_value.to_bits()
                    && a_min.to_bits() == b_min.to_bits()
                    && a_max.to_bits() == b_max.to_bits()
            }
            (InvalidNumericStep { step: a }, InvalidNumericStep { step: b }) => {
                a.to_bits() == b.to_bits()
            }
            (InvalidNumericJump { jump: a }, InvalidNumericJump { jump: b }) => {
                a.to_bits() == b.to_bits()
            }
            (
                NonFiniteScroll {
                    field: a_field,
                    value: a_value,
                },
                NonFiniteScroll {
                    field: b_field,
                    value: b_value,
                },
            ) => a_field == b_field && a_value.to_bits() == b_value.to_bits(),
            (
                InvalidScrollBounds {
                    axis: a_axis,
                    min: a_min,
                    max: a_max,
                },
                InvalidScrollBounds {
                    axis: b_axis,
                    min: b_min,
                    max: b_max,
                },
            ) => {
                a_axis == b_axis
                    && a_min.to_bits() == b_min.to_bits()
                    && a_max.to_bits() == b_max.to_bits()
            }
            (
                ScrollValueOutOfBounds {
                    axis: a_axis,
                    value: a_value,
                    min: a_min,
                    max: a_max,
                },
                ScrollValueOutOfBounds {
                    axis: b_axis,
                    value: b_value,
                    min: b_min,
                    max: b_max,
                },
            ) => {
                a_axis == b_axis
                    && a_value.to_bits() == b_value.to_bits()
                    && a_min.to_bits() == b_min.to_bits()
                    && a_max.to_bits() == b_max.to_bits()
            }
            _ => false,
        }
    }
}

impl Eq for SemanticsValidationErrorKind {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticsReferenceField {
    Root,
    BarrierRoot,
    FocusBarrierRoot,
    Focus,
    Captured,
    Parent,
    ActiveDescendant,
    LabelledBy,
    DescribedBy,
    Controls,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticsValidationError {
    pub node: NodeId,
    pub kind: SemanticsValidationErrorKind,
}

impl SemanticsNode {
    pub fn validate(&self) -> Result<(), SemanticsValidationError> {
        validate_text_ranges(
            self.id,
            self.value.as_deref(),
            self.text_selection,
            self.text_composition,
        )?;
        validate_inline_spans(self.id, self.value.as_deref(), &self.inline_spans)?;
        validate_extra(self.id, &self.extra)?;
        Ok(())
    }
}

impl SemanticsSnapshot {
    pub fn validate(&self) -> Result<(), SemanticsValidationError> {
        let mut ids = HashSet::with_capacity(self.nodes.len());
        for node in &self.nodes {
            if !ids.insert(node.id) {
                return Err(SemanticsValidationError {
                    node: node.id,
                    kind: SemanticsValidationErrorKind::DuplicateNodeId { id: node.id },
                });
            }
        }

        let check_ref = |node: NodeId,
                         field: SemanticsReferenceField,
                         referenced: NodeId,
                         ids: &HashSet<NodeId>|
         -> Result<(), SemanticsValidationError> {
            if ids.contains(&referenced) {
                return Ok(());
            }
            Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::MissingReferencedNode { field, referenced },
            })
        };

        for root in &self.roots {
            check_ref(root.root, SemanticsReferenceField::Root, root.root, &ids)?;
        }
        if let Some(barrier_root) = self.barrier_root {
            check_ref(
                barrier_root,
                SemanticsReferenceField::BarrierRoot,
                barrier_root,
                &ids,
            )?;
        }
        if let Some(focus_barrier_root) = self.focus_barrier_root {
            check_ref(
                focus_barrier_root,
                SemanticsReferenceField::FocusBarrierRoot,
                focus_barrier_root,
                &ids,
            )?;
        }
        if let Some(focus) = self.focus {
            check_ref(focus, SemanticsReferenceField::Focus, focus, &ids)?;
        }
        if let Some(captured) = self.captured {
            check_ref(captured, SemanticsReferenceField::Captured, captured, &ids)?;
        }

        for node in &self.nodes {
            node.validate()?;

            if node.pos_in_set.is_some() ^ node.set_size.is_some() {
                return Err(SemanticsValidationError {
                    node: node.id,
                    kind: SemanticsValidationErrorKind::InvalidCollectionMetadata {
                        pos_in_set: node.pos_in_set,
                        set_size: node.set_size,
                    },
                });
            }
            if let (Some(pos_in_set), Some(set_size)) = (node.pos_in_set, node.set_size)
                && (pos_in_set == 0 || set_size == 0 || pos_in_set > set_size)
            {
                return Err(SemanticsValidationError {
                    node: node.id,
                    kind: SemanticsValidationErrorKind::InvalidCollectionMetadata {
                        pos_in_set: Some(pos_in_set),
                        set_size: Some(set_size),
                    },
                });
            }

            if let Some(parent) = node.parent {
                check_ref(node.id, SemanticsReferenceField::Parent, parent, &ids)?;
            }
            if let Some(active) = node.active_descendant {
                check_ref(
                    node.id,
                    SemanticsReferenceField::ActiveDescendant,
                    active,
                    &ids,
                )?;
            }
            for id in &node.labelled_by {
                check_ref(node.id, SemanticsReferenceField::LabelledBy, *id, &ids)?;
            }
            for id in &node.described_by {
                check_ref(node.id, SemanticsReferenceField::DescribedBy, *id, &ids)?;
            }
            for id in &node.controls {
                check_ref(node.id, SemanticsReferenceField::Controls, *id, &ids)?;
            }
        }
        Ok(())
    }
}

fn validate_extra(
    node: NodeId,
    extra: &SemanticsNodeExtra,
) -> Result<(), SemanticsValidationError> {
    if let Some(level) = extra.level
        && level == 0
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::InvalidHierarchyLevel { level },
        });
    }

    validate_numeric(node, extra.numeric)?;
    validate_scroll(node, extra.scroll)?;
    Ok(())
}

fn validate_numeric(
    node: NodeId,
    numeric: SemanticsNumeric,
) -> Result<(), SemanticsValidationError> {
    let check_finite =
        |field: SemanticsNumericField, value: f64| -> Result<(), SemanticsValidationError> {
            if value.is_finite() {
                Ok(())
            } else {
                Err(SemanticsValidationError {
                    node,
                    kind: SemanticsValidationErrorKind::NonFiniteNumeric { field, value },
                })
            }
        };

    if let Some(value) = numeric.value {
        check_finite(SemanticsNumericField::Value, value)?;
    }
    if let Some(min) = numeric.min {
        check_finite(SemanticsNumericField::Min, min)?;
    }
    if let Some(max) = numeric.max {
        check_finite(SemanticsNumericField::Max, max)?;
    }
    if let Some(step) = numeric.step {
        check_finite(SemanticsNumericField::Step, step)?;
    }
    if let Some(jump) = numeric.jump {
        check_finite(SemanticsNumericField::Jump, jump)?;
    }

    if let (Some(min), Some(max)) = (numeric.min, numeric.max)
        && min > max
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::InvalidNumericBounds { min, max },
        });
    }

    if let Some(step) = numeric.step
        && step <= 0.0
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::InvalidNumericStep { step },
        });
    }
    if let Some(jump) = numeric.jump
        && jump <= 0.0
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::InvalidNumericJump { jump },
        });
    }

    if let (Some(value), Some(min), Some(max)) = (numeric.value, numeric.min, numeric.max)
        && (value < min || value > max)
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::NumericValueOutOfBounds { value, min, max },
        });
    }

    Ok(())
}

fn validate_scroll(node: NodeId, scroll: SemanticsScroll) -> Result<(), SemanticsValidationError> {
    const EPS: f64 = 1e-9;

    let check_finite =
        |field: SemanticsScrollField, value: f64| -> Result<(), SemanticsValidationError> {
            if value.is_finite() {
                Ok(())
            } else {
                Err(SemanticsValidationError {
                    node,
                    kind: SemanticsValidationErrorKind::NonFiniteScroll { field, value },
                })
            }
        };

    if let Some(x) = scroll.x {
        check_finite(SemanticsScrollField::X, x)?;
    }
    if let Some(x_min) = scroll.x_min {
        check_finite(SemanticsScrollField::XMin, x_min)?;
    }
    if let Some(x_max) = scroll.x_max {
        check_finite(SemanticsScrollField::XMax, x_max)?;
    }
    if let Some(y) = scroll.y {
        check_finite(SemanticsScrollField::Y, y)?;
    }
    if let Some(y_min) = scroll.y_min {
        check_finite(SemanticsScrollField::YMin, y_min)?;
    }
    if let Some(y_max) = scroll.y_max {
        check_finite(SemanticsScrollField::YMax, y_max)?;
    }

    if let (Some(min), Some(max)) = (scroll.x_min, scroll.x_max)
        && min > max
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::InvalidScrollBounds {
                axis: SemanticsScrollAxis::X,
                min,
                max,
            },
        });
    }
    if let (Some(value), Some(min), Some(max)) = (scroll.x, scroll.x_min, scroll.x_max)
        && (value < min - EPS || value > max + EPS)
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::ScrollValueOutOfBounds {
                axis: SemanticsScrollAxis::X,
                value,
                min,
                max,
            },
        });
    }

    if let (Some(min), Some(max)) = (scroll.y_min, scroll.y_max)
        && min > max
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::InvalidScrollBounds {
                axis: SemanticsScrollAxis::Y,
                min,
                max,
            },
        });
    }
    if let (Some(value), Some(min), Some(max)) = (scroll.y, scroll.y_min, scroll.y_max)
        && (value < min - EPS || value > max + EPS)
    {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::ScrollValueOutOfBounds {
                axis: SemanticsScrollAxis::Y,
                value,
                min,
                max,
            },
        });
    }

    Ok(())
}

fn validate_text_ranges(
    node: NodeId,
    value: Option<&str>,
    text_selection: Option<(u32, u32)>,
    text_composition: Option<(u32, u32)>,
) -> Result<(), SemanticsValidationError> {
    if text_selection.is_none() && text_composition.is_none() {
        return Ok(());
    }

    let Some(value) = value else {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::MissingValueForTextRange {
                field: if text_selection.is_some() {
                    SemanticsValidationField::TextSelection
                } else {
                    SemanticsValidationField::TextComposition
                },
            },
        });
    };

    let len_u32 = u32::try_from(value.len()).unwrap_or(u32::MAX);

    let check_range = |field: SemanticsValidationField,
                       start: u32,
                       end: u32|
     -> Result<(), SemanticsValidationError> {
        if start > end {
            return Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::InvalidRangeOrder { field, start, end },
            });
        }
        if start > len_u32 || end > len_u32 {
            return Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::RangeOutOfBounds {
                    field,
                    start,
                    end,
                    len: len_u32,
                },
            });
        }

        let start_usize = start as usize;
        let end_usize = end as usize;
        if !value.is_char_boundary(start_usize) {
            return Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::RangeNotCharBoundary {
                    field,
                    offset: start,
                },
            });
        }
        if !value.is_char_boundary(end_usize) {
            return Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::RangeNotCharBoundary { field, offset: end },
            });
        }
        Ok(())
    };

    if let Some((anchor, focus)) = text_selection {
        let (start, end) = if anchor <= focus {
            (anchor, focus)
        } else {
            (focus, anchor)
        };
        check_range(SemanticsValidationField::TextSelection, start, end)?;
    }

    if let Some((start, end)) = text_composition {
        check_range(SemanticsValidationField::TextComposition, start, end)?;
    }

    Ok(())
}

fn validate_inline_spans(
    node: NodeId,
    value: Option<&str>,
    spans: &[SemanticsInlineSpan],
) -> Result<(), SemanticsValidationError> {
    if spans.is_empty() {
        return Ok(());
    }

    let Some(value) = value else {
        return Err(SemanticsValidationError {
            node,
            kind: SemanticsValidationErrorKind::MissingValueForTextRange {
                field: SemanticsValidationField::InlineSpan,
            },
        });
    };

    let len_u32 = u32::try_from(value.len()).unwrap_or(u32::MAX);
    for span in spans {
        let (start, end) = span.range_utf8;
        if start > end {
            return Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::InvalidRangeOrder {
                    field: SemanticsValidationField::InlineSpan,
                    start,
                    end,
                },
            });
        }
        if start > len_u32 || end > len_u32 {
            return Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::RangeOutOfBounds {
                    field: SemanticsValidationField::InlineSpan,
                    start,
                    end,
                    len: len_u32,
                },
            });
        }

        let start_usize = start as usize;
        let end_usize = end as usize;
        if !value.is_char_boundary(start_usize) {
            return Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::RangeNotCharBoundary {
                    field: SemanticsValidationField::InlineSpan,
                    offset: start,
                },
            });
        }
        if !value.is_char_boundary(end_usize) {
            return Err(SemanticsValidationError {
                node,
                kind: SemanticsValidationErrorKind::RangeNotCharBoundary {
                    field: SemanticsValidationField::InlineSpan,
                    offset: end,
                },
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    fn node(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    fn snapshot_with_nodes(nodes: Vec<SemanticsNode>) -> SemanticsSnapshot {
        SemanticsSnapshot {
            window: AppWindowId::default(),
            roots: vec![SemanticsRoot {
                root: nodes.first().expect("at least one node").id,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes,
        }
    }

    fn base_node(extra: SemanticsNodeExtra) -> SemanticsNode {
        SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::Slider,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra,
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        }
    }

    #[test]
    fn validates_utf8_char_boundaries_for_text_ranges() {
        let n = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::TextField,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: Some("😀".to_string()), // 4 bytes
            extra: SemanticsNodeExtra::default(),
            text_selection: Some((0, 4)),
            text_composition: Some((0, 4)),
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };
        n.validate().expect("valid ranges should pass");

        let bad = SemanticsNode {
            text_selection: Some((0, 2)),
            ..n
        };
        let err = bad.validate().expect_err("non-boundary should fail");
        assert_eq!(err.node, node(1));
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::RangeNotCharBoundary {
                field: SemanticsValidationField::TextSelection,
                offset: 2
            }
        ));
    }

    #[test]
    fn rejects_ranges_without_value() {
        let n = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::TextField,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: Some((0, 0)),
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };
        let err = n.validate().expect_err("range without value should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::MissingValueForTextRange { .. }
        ));
    }

    #[test]
    fn rejects_inline_spans_without_value() {
        let n = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::Text,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: vec![SemanticsInlineSpan {
                range_utf8: (0, 0),
                role: SemanticsRole::Link,
                tag: None,
            }],
        };
        let err = n
            .validate()
            .expect_err("inline spans without value should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::MissingValueForTextRange {
                field: SemanticsValidationField::InlineSpan
            }
        ));
    }

    #[test]
    fn validates_extra_numeric_and_scroll_metadata() {
        let n = base_node(SemanticsNodeExtra {
            level: Some(1),
            numeric: SemanticsNumeric {
                value: Some(5.0),
                min: Some(0.0),
                max: Some(10.0),
                step: Some(1.0),
                jump: Some(5.0),
            },
            scroll: SemanticsScroll {
                x: Some(0.0),
                x_min: Some(0.0),
                x_max: Some(0.0),
                y: Some(10.0),
                y_min: Some(0.0),
                y_max: Some(10.0),
            },
            ..SemanticsNodeExtra::default()
        });
        n.validate().expect("valid extra metadata should pass");
    }

    #[test]
    fn rejects_invalid_hierarchy_level() {
        let n = base_node(SemanticsNodeExtra {
            level: Some(0),
            ..SemanticsNodeExtra::default()
        });
        let err = n.validate().expect_err("level=0 should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::InvalidHierarchyLevel { level: 0 }
        ));
    }

    #[test]
    fn rejects_non_finite_numeric_metadata() {
        let n = base_node(SemanticsNodeExtra {
            numeric: SemanticsNumeric {
                value: Some(f64::NAN),
                ..SemanticsNumeric::default()
            },
            ..SemanticsNodeExtra::default()
        });
        let err = n.validate().expect_err("NaN should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::NonFiniteNumeric {
                field: SemanticsNumericField::Value,
                ..
            }
        ));
    }

    #[test]
    fn rejects_invalid_numeric_bounds() {
        let n = base_node(SemanticsNodeExtra {
            numeric: SemanticsNumeric {
                min: Some(10.0),
                max: Some(5.0),
                ..SemanticsNumeric::default()
            },
            ..SemanticsNodeExtra::default()
        });
        let err = n.validate().expect_err("min > max should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::InvalidNumericBounds { .. }
        ));
    }

    #[test]
    fn rejects_numeric_value_out_of_bounds() {
        let n = base_node(SemanticsNodeExtra {
            numeric: SemanticsNumeric {
                value: Some(11.0),
                min: Some(0.0),
                max: Some(10.0),
                ..SemanticsNumeric::default()
            },
            ..SemanticsNodeExtra::default()
        });
        let err = n.validate().expect_err("out-of-bounds should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::NumericValueOutOfBounds { .. }
        ));
    }

    #[test]
    fn rejects_non_positive_numeric_step_and_jump() {
        let step = base_node(SemanticsNodeExtra {
            numeric: SemanticsNumeric {
                step: Some(0.0),
                ..SemanticsNumeric::default()
            },
            ..SemanticsNodeExtra::default()
        });
        let err = step.validate().expect_err("step <= 0 should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::InvalidNumericStep { .. }
        ));

        let jump = base_node(SemanticsNodeExtra {
            numeric: SemanticsNumeric {
                jump: Some(-1.0),
                ..SemanticsNumeric::default()
            },
            ..SemanticsNodeExtra::default()
        });
        let err = jump.validate().expect_err("jump <= 0 should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::InvalidNumericJump { .. }
        ));
    }

    #[test]
    fn rejects_non_finite_scroll_metadata() {
        let n = base_node(SemanticsNodeExtra {
            scroll: SemanticsScroll {
                y: Some(f64::INFINITY),
                ..SemanticsScroll::default()
            },
            ..SemanticsNodeExtra::default()
        });
        let err = n.validate().expect_err("Infinity should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::NonFiniteScroll {
                field: SemanticsScrollField::Y,
                ..
            }
        ));
    }

    #[test]
    fn rejects_invalid_scroll_bounds_and_value_out_of_bounds() {
        let bounds = base_node(SemanticsNodeExtra {
            scroll: SemanticsScroll {
                x_min: Some(10.0),
                x_max: Some(5.0),
                ..SemanticsScroll::default()
            },
            ..SemanticsNodeExtra::default()
        });
        let err = bounds.validate().expect_err("x_min > x_max should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::InvalidScrollBounds {
                axis: SemanticsScrollAxis::X,
                ..
            }
        ));

        let oob = base_node(SemanticsNodeExtra {
            scroll: SemanticsScroll {
                y: Some(11.0),
                y_min: Some(0.0),
                y_max: Some(10.0),
                ..SemanticsScroll::default()
            },
            ..SemanticsNodeExtra::default()
        });
        let err = oob.validate().expect_err("y out-of-bounds should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::ScrollValueOutOfBounds {
                axis: SemanticsScrollAxis::Y,
                ..
            }
        ));
    }

    #[test]
    fn validates_utf8_char_boundaries_for_inline_spans() {
        let n = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::Text,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: Some("😀".to_string()), // 4 bytes
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: vec![SemanticsInlineSpan {
                range_utf8: (0, 4),
                role: SemanticsRole::Link,
                tag: None,
            }],
        };
        n.validate()
            .expect("inline span on a utf-8 boundary should pass");

        let bad = SemanticsNode {
            inline_spans: vec![SemanticsInlineSpan {
                range_utf8: (0, 2),
                role: SemanticsRole::Link,
                tag: None,
            }],
            ..n
        };
        let err = bad.validate().expect_err("non-boundary should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::RangeNotCharBoundary {
                field: SemanticsValidationField::InlineSpan,
                offset: 2
            }
        ));
    }

    #[test]
    fn rejects_out_of_bounds_ranges() {
        let n = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::TextField,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: Some("abc".to_string()),
            extra: SemanticsNodeExtra::default(),
            text_selection: Some((0, 4)),
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };
        let err = n.validate().expect_err("oob should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::RangeOutOfBounds { .. }
        ));
    }

    #[test]
    fn rejects_invalid_composition_order() {
        let n = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::TextField,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: Some("abc".to_string()),
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: Some((2, 1)),
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };
        let err = n.validate().expect_err("invalid order should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::InvalidRangeOrder {
                field: SemanticsValidationField::TextComposition,
                ..
            }
        ));
    }

    #[test]
    fn rejects_duplicate_node_ids_in_snapshot() {
        let n1 = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::Window,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };
        let snap = snapshot_with_nodes(vec![n1.clone(), n1]);
        let err = snap.validate().expect_err("duplicate node ids should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::DuplicateNodeId { .. }
        ));
    }

    #[test]
    fn rejects_missing_references() {
        let root = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::Window,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };
        let child = SemanticsNode {
            id: node(2),
            parent: Some(node(999)),
            role: SemanticsRole::Group,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: Some(node(998)),
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: vec![node(997)],
            described_by: vec![node(996)],
            controls: vec![node(995)],
            inline_spans: Vec::new(),
        };

        let snap = snapshot_with_nodes(vec![root, child]);
        let err = snap.validate().expect_err("missing references should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::MissingReferencedNode { .. }
        ));
    }

    #[test]
    fn rejects_invalid_collection_metadata() {
        let root = SemanticsNode {
            id: node(1),
            parent: None,
            role: SemanticsRole::Window,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };

        let bad_missing_peer = SemanticsNode {
            id: node(2),
            parent: Some(node(1)),
            role: SemanticsRole::ListItem,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: Some(1),
            set_size: None,
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };
        let snap = snapshot_with_nodes(vec![root.clone(), bad_missing_peer]);
        let err = snap.validate().expect_err("missing set_size should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::InvalidCollectionMetadata { .. }
        ));

        let bad_bounds = SemanticsNode {
            id: node(2),
            parent: Some(node(1)),
            role: SemanticsRole::ListItem,
            bounds: Rect::default(),
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: Some(2),
            set_size: Some(1),
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };
        let snap = snapshot_with_nodes(vec![root, bad_bounds]);
        let err = snap
            .validate()
            .expect_err("pos_in_set > set_size should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::InvalidCollectionMetadata {
                pos_in_set: Some(2),
                set_size: Some(1),
            }
        ));
    }
}
