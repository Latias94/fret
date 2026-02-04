use crate::{AppWindowId, NodeId, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticsRole {
    Generic,
    Window,
    Panel,
    Group,
    Toolbar,
    Dialog,
    AlertDialog,
    Alert,
    Button,
    Checkbox,
    Switch,
    Slider,
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
    ListBox,
    ListBoxOption,
    TreeItem,
    Viewport,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SemanticsActions {
    pub focus: bool,
    pub invoke: bool,
    pub set_value: bool,
    pub set_text_selection: bool,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        )
    }
}

impl SemanticsSnapshot {
    pub fn validate(&self) -> Result<(), SemanticsValidationError> {
        for node in &self.nodes {
            node.validate()?;
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    fn node(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
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
            text_selection: Some((0, 4)),
            text_composition: Some((0, 4)),
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
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
            text_selection: Some((0, 0)),
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
        };
        let err = n.validate().expect_err("range without value should fail");
        assert!(matches!(
            err.kind,
            SemanticsValidationErrorKind::MissingValueForTextRange { .. }
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
            text_selection: Some((0, 4)),
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
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
            text_selection: None,
            text_composition: Some((2, 1)),
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
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
}
