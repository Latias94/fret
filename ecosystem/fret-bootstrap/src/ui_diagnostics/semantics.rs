use serde::{Deserialize, Serialize};

use super::*;

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiCheckedStateV1 {
    False,
    True,
    Mixed,
}

impl UiCheckedStateV1 {
    fn from_semantics_checked_state(v: fret_core::SemanticsCheckedState) -> Self {
        match v {
            fret_core::SemanticsCheckedState::False => Self::False,
            fret_core::SemanticsCheckedState::True => Self::True,
            fret_core::SemanticsCheckedState::Mixed => Self::Mixed,
            _ => Self::Mixed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPressedStateV1 {
    False,
    True,
    Mixed,
}

impl UiPressedStateV1 {
    fn from_semantics_pressed_state(v: fret_core::SemanticsPressedState) -> Self {
        match v {
            fret_core::SemanticsPressedState::False => Self::False,
            fret_core::SemanticsPressedState::True => Self::True,
            fret_core::SemanticsPressedState::Mixed => Self::Mixed,
            _ => Self::Mixed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInvalidV1 {
    True,
    Grammar,
    Spelling,
}

impl UiInvalidV1 {
    fn from_semantics_invalid(v: fret_core::SemanticsInvalid) -> Self {
        match v {
            fret_core::SemanticsInvalid::True => Self::True,
            fret_core::SemanticsInvalid::Grammar => Self::Grammar,
            fret_core::SemanticsInvalid::Spelling => Self::Spelling,
            _ => Self::True,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLiveV1 {
    Off,
    Polite,
    Assertive,
}

impl UiLiveV1 {
    fn from_semantics_live(v: fret_core::SemanticsLive) -> Option<Self> {
        match v {
            fret_core::SemanticsLive::Off => Some(Self::Off),
            fret_core::SemanticsLive::Polite => Some(Self::Polite),
            fret_core::SemanticsLive::Assertive => Some(Self::Assertive),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsSnapshotV1 {
    pub window: u64,
    pub roots: Vec<UiSemanticsRootV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barrier_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_barrier_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured: Option<u64>,
    pub nodes: Vec<UiSemanticsNodeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsRootV1 {
    pub root: u64,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    pub z_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsNodeV1 {
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<u64>,
    pub role: String,
    pub bounds: RectV1,
    #[serde(default, skip_serializing_if = "UiSemanticsFlagsV1::is_default")]
    pub flags: UiSemanticsFlagsV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_descendant: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pos_in_set: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_selection: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_composition: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "UiSemanticsActionsV1::is_default")]
    pub actions: UiSemanticsActionsV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labelled_by: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub described_by: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inline_spans: Vec<UiSemanticsInlineSpanV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsInlineSpanV1 {
    pub range_utf8: (u32, u32),
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSemanticsFlagsV1 {
    #[serde(default, skip_serializing_if = "is_false")]
    pub focused: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub captured: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub visited: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub multiselectable: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub busy: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub selected: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub expanded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_state: Option<UiCheckedStateV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressed_state: Option<UiPressedStateV1>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalid: Option<UiInvalidV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live: Option<UiLiveV1>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub live_atomic: bool,
}

impl UiSemanticsFlagsV1 {
    fn is_default(v: &Self) -> bool {
        !v.focused
            && !v.captured
            && !v.disabled
            && !v.hidden
            && !v.visited
            && !v.multiselectable
            && !v.busy
            && !v.selected
            && !v.expanded
            && v.checked.is_none()
            && v.checked_state.is_none()
            && v.pressed_state.is_none()
            && !v.required
            && v.invalid.is_none()
            && v.live.is_none()
            && !v.live_atomic
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSemanticsActionsV1 {
    #[serde(default, skip_serializing_if = "is_false")]
    pub focus: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub invoke: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub set_value: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub scroll_by: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub set_text_selection: bool,
}

impl UiSemanticsActionsV1 {
    fn is_default(v: &Self) -> bool {
        !v.focus && !v.invoke && !v.set_value && !v.scroll_by && !v.set_text_selection
    }
}

impl UiSemanticsInlineSpanV1 {
    fn from_span(
        span: &fret_core::SemanticsInlineSpan,
        redact_text: bool,
        max_string_bytes: usize,
    ) -> Self {
        let mut tag = span
            .tag
            .as_deref()
            .map(|s| maybe_redact_string(s, redact_text));
        if let Some(s) = &mut tag {
            truncate_string_bytes(s, max_string_bytes);
        }

        Self {
            range_utf8: span.range_utf8,
            role: semantics_role_label(span.role).to_string(),
            tag,
        }
    }
}

impl UiSemanticsSnapshotV1 {
    pub(super) fn from_snapshot(
        snapshot: &fret_core::SemanticsSnapshot,
        redact_text: bool,
        max_string_bytes: usize,
        max_nodes: usize,
        test_ids_only: bool,
    ) -> Self {
        Self {
            window: snapshot.window.data().as_ffi(),
            roots: snapshot
                .roots
                .iter()
                .map(|r| UiSemanticsRootV1 {
                    root: key_to_u64(r.root),
                    visible: r.visible,
                    blocks_underlay_input: r.blocks_underlay_input,
                    hit_testable: r.hit_testable,
                    z_index: r.z_index,
                })
                .collect(),
            barrier_root: snapshot.barrier_root.map(key_to_u64),
            focus_barrier_root: snapshot.focus_barrier_root.map(key_to_u64),
            focus: snapshot.focus.map(key_to_u64),
            captured: snapshot.captured.map(key_to_u64),
            nodes: snapshot
                .nodes
                .iter()
                .filter(|n| !test_ids_only || n.test_id.is_some())
                .take(max_nodes)
                .map(|n| UiSemanticsNodeV1::from_node(n, redact_text, max_string_bytes))
                .collect(),
        }
    }
}

impl UiSemanticsNodeV1 {
    pub(crate) fn from_node(
        node: &fret_core::SemanticsNode,
        redact_text: bool,
        max_string_bytes: usize,
    ) -> Self {
        let mut label = node
            .label
            .as_deref()
            .map(|s| maybe_redact_string(s, redact_text));
        let mut value = node
            .value
            .as_deref()
            .map(|s| maybe_redact_string(s, redact_text));
        let mut test_id = node.test_id.clone();

        if let Some(s) = &mut label {
            truncate_string_bytes(s, max_string_bytes);
        }
        if let Some(s) = &mut value {
            truncate_string_bytes(s, max_string_bytes);
        }
        if let Some(s) = &mut test_id {
            truncate_string_bytes(s, max_string_bytes);
        }

        Self {
            id: key_to_u64(node.id),
            parent: node.parent.map(key_to_u64),
            role: semantics_role_label(node.role).to_string(),
            bounds: RectV1::from(node.bounds),
            flags: UiSemanticsFlagsV1 {
                focused: node.flags.focused,
                captured: node.flags.captured,
                disabled: node.flags.disabled,
                hidden: node.flags.hidden,
                visited: node.flags.visited,
                multiselectable: node.flags.multiselectable,
                busy: node.flags.busy,
                selected: node.flags.selected,
                expanded: node.flags.expanded,
                checked: node.flags.checked,
                checked_state: node
                    .flags
                    .checked_state
                    .map(UiCheckedStateV1::from_semantics_checked_state),
                pressed_state: node
                    .flags
                    .pressed_state
                    .map(UiPressedStateV1::from_semantics_pressed_state),
                required: node.flags.required,
                invalid: node.flags.invalid.map(UiInvalidV1::from_semantics_invalid),
                live: node.flags.live.and_then(UiLiveV1::from_semantics_live),
                live_atomic: node.flags.live_atomic,
            },
            test_id,
            active_descendant: node.active_descendant.map(key_to_u64),
            pos_in_set: node.pos_in_set,
            set_size: node.set_size,
            label,
            value,
            text_selection: node.text_selection,
            text_composition: node.text_composition,
            actions: UiSemanticsActionsV1 {
                focus: node.actions.focus,
                invoke: node.actions.invoke,
                set_value: node.actions.set_value,
                scroll_by: node.actions.scroll_by,
                set_text_selection: node.actions.set_text_selection,
            },
            labelled_by: node.labelled_by.iter().copied().map(key_to_u64).collect(),
            described_by: node.described_by.iter().copied().map(key_to_u64).collect(),
            controls: node.controls.iter().copied().map(key_to_u64).collect(),
            inline_spans: node
                .inline_spans
                .iter()
                .map(|span| UiSemanticsInlineSpanV1::from_span(span, redact_text, max_string_bytes))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantics_node_omits_default_flags_actions_and_empty_vecs() {
        let node = UiSemanticsNodeV1 {
            id: 1,
            parent: None,
            role: "button".to_string(),
            bounds: RectV1 {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 10.0,
            },
            flags: UiSemanticsFlagsV1::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            text_selection: None,
            text_composition: None,
            actions: UiSemanticsActionsV1::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        };

        let v = serde_json::to_value(&node).expect("serialize");
        assert!(v.get("flags").is_none());
        assert!(v.get("actions").is_none());
        assert!(v.get("labelled_by").is_none());
        assert!(v.get("described_by").is_none());
        assert!(v.get("controls").is_none());
        assert!(v.get("inline_spans").is_none());
        assert!(v.get("test_id").is_none());
    }

    #[test]
    fn semantics_node_missing_flags_actions_deserialize_to_default() {
        let v = serde_json::json!({
            "id": 1,
            "role": "button",
            "bounds": {"x":0.0,"y":0.0,"w":10.0,"h":10.0}
        });
        let parsed: UiSemanticsNodeV1 = serde_json::from_value(v).expect("deserialize");
        assert!(UiSemanticsFlagsV1::is_default(&parsed.flags));
        assert!(UiSemanticsActionsV1::is_default(&parsed.actions));
        assert!(parsed.inline_spans.is_empty());
    }

    #[test]
    fn semantics_node_exports_inline_spans() {
        let node = fret_core::SemanticsNode {
            id: fret_core::NodeId::from(slotmap::KeyData::from_ffi(1)),
            parent: None,
            role: fret_core::SemanticsRole::Text,
            bounds: fret_core::Rect::new(
                fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                fret_core::Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
            ),
            flags: fret_core::SemanticsFlags::default(),
            test_id: Some("description".to_string()),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: Some("Open Settings to continue.".to_string()),
            extra: fret_core::SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: fret_core::SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: vec![fret_core::SemanticsInlineSpan {
                range_utf8: (5, 13),
                role: fret_core::SemanticsRole::Link,
                tag: Some("settings://workspace".to_string()),
            }],
        };

        let exported = UiSemanticsNodeV1::from_node(&node, false, 512);
        assert_eq!(exported.inline_spans.len(), 1);
        assert_eq!(exported.inline_spans[0].range_utf8, (5, 13));
        assert_eq!(exported.inline_spans[0].role, "link");
        assert_eq!(
            exported.inline_spans[0].tag.as_deref(),
            Some("settings://workspace")
        );
    }
}
