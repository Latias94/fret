use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::Arc;

use fret_code_editor_buffer::{Revision, Selection, TextBuffer};

use crate::{DisplayMap, DisplayPoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorAssistKind {
    Completion,
    Hover,
    CodeAction,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorAssistTrigger {
    #[default]
    Invoked,
    Character(Arc<str>),
    Hover,
    Diagnostics,
    SelectionChanged,
    Custom(Arc<str>),
}

/// Revision-aware request facts for editor assist features.
///
/// This is a view-layer data contract only. It deliberately does not encode overlay placement,
/// dismissal, focus trap/restore, listbox navigation, hover intent, or command execution policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorAssistRequest {
    pub kind: EditorAssistKind,
    pub revision: Revision,
    pub selection: Selection,
    pub buffer_range: Range<usize>,
    pub display_point: DisplayPoint,
    pub trigger: EditorAssistTrigger,
    pub anchor_id: Option<Arc<str>>,
}

impl EditorAssistRequest {
    pub fn new(
        kind: EditorAssistKind,
        revision: Revision,
        selection: Selection,
        buffer_range: Range<usize>,
        display_point: DisplayPoint,
    ) -> Self {
        Self {
            kind,
            revision,
            selection,
            buffer_range,
            display_point,
            trigger: EditorAssistTrigger::Invoked,
            anchor_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAssistRequestError {
    RangeStartAfterEnd,
    RangeOutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    RangeNotCharBoundary {
        start: usize,
        end: usize,
    },
    SelectionOutOfBounds {
        anchor: usize,
        focus: usize,
        len: usize,
    },
    SelectionNotCharBoundary {
        anchor: usize,
        focus: usize,
    },
    DisplayRowRequiresDisplayMap {
        row: usize,
    },
    DisplayRowOutOfBounds {
        row: usize,
        row_count: usize,
    },
    EmptyAnchorId,
    EmptyTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CompletionCandidateKind {
    #[default]
    Text,
    Keyword,
    Function,
    Method,
    Variable,
    Field,
    Module,
    Snippet,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CompletionCommitKind {
    #[default]
    Insert,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub id: Arc<str>,
    pub label: Arc<str>,
    pub kind: CompletionCandidateKind,
    pub detail: Option<Arc<str>>,
    pub documentation: Option<Arc<str>>,
    pub filter_text: Option<Arc<str>>,
    pub sort_text: Option<Arc<str>>,
    pub insert_text: Option<Arc<str>>,
    pub replace_range: Option<Range<usize>>,
    pub commit_kind: CompletionCommitKind,
    pub commit_characters: Vec<Arc<str>>,
    pub command_id: Option<Arc<str>>,
}

impl CompletionCandidate {
    pub fn new(id: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: CompletionCandidateKind::Text,
            detail: None,
            documentation: None,
            filter_text: None,
            sort_text: None,
            insert_text: None,
            replace_range: None,
            commit_kind: CompletionCommitKind::Insert,
            commit_characters: Vec::new(),
            command_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionList {
    pub request_id: Arc<str>,
    pub revision: Revision,
    pub candidates: Vec<CompletionCandidate>,
    pub active_id: Option<Arc<str>>,
}

impl CompletionList {
    pub fn new(
        request_id: impl Into<Arc<str>>,
        revision: Revision,
        candidates: Vec<CompletionCandidate>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            revision,
            candidates,
            active_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionListError {
    EmptyRequestId,
    EmptyCandidateId,
    EmptyCandidateLabel,
    DuplicateCandidateId(Arc<str>),
    ActiveCandidateMissing(Arc<str>),
    ReplaceRangeStartAfterEnd,
    ReplaceRangeOutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    ReplaceRangeNotCharBoundary {
        start: usize,
        end: usize,
    },
    EmptyCommitCharacter,
    EmptyCommandId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverPayload {
    pub id: Arc<str>,
    pub revision: Revision,
    pub range: Range<usize>,
    pub contents: Arc<str>,
    pub source: Option<Arc<str>>,
    pub related_command_ids: Vec<Arc<str>>,
}

impl HoverPayload {
    pub fn new(
        id: impl Into<Arc<str>>,
        revision: Revision,
        range: Range<usize>,
        contents: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            id: id.into(),
            revision,
            range,
            contents: contents.into(),
            source: None,
            related_command_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverPayloadError {
    EmptyId,
    EmptyContents,
    RangeStartAfterEnd,
    RangeOutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    RangeNotCharBoundary {
        start: usize,
        end: usize,
    },
    EmptyCommandId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CodeActionKind {
    #[default]
    QuickFix,
    Refactor,
    Source,
    OrganizeImports,
    Custom(Arc<str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeAction {
    pub id: Arc<str>,
    pub title: Arc<str>,
    pub kind: CodeActionKind,
    pub command_id: Arc<str>,
    pub related_diagnostic_ids: Vec<Arc<str>>,
    pub is_preferred: bool,
    pub disabled_reason: Option<Arc<str>>,
}

impl CodeAction {
    pub fn new(
        id: impl Into<Arc<str>>,
        title: impl Into<Arc<str>>,
        command_id: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            kind: CodeActionKind::QuickFix,
            command_id: command_id.into(),
            related_diagnostic_ids: Vec::new(),
            is_preferred: false,
            disabled_reason: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeActionList {
    pub request_id: Arc<str>,
    pub revision: Revision,
    pub range: Range<usize>,
    pub actions: Vec<CodeAction>,
}

impl CodeActionList {
    pub fn new(
        request_id: impl Into<Arc<str>>,
        revision: Revision,
        range: Range<usize>,
        actions: Vec<CodeAction>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            revision,
            range,
            actions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeActionListError {
    EmptyRequestId,
    RangeStartAfterEnd,
    RangeOutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    RangeNotCharBoundary {
        start: usize,
        end: usize,
    },
    EmptyActionId,
    EmptyActionTitle,
    EmptyCommandId,
    DuplicateActionId(Arc<str>),
    EmptyDiagnosticId,
    EmptyCustomKind,
}

pub fn validate_editor_assist_request(
    buf: &TextBuffer,
    display_map: Option<&DisplayMap>,
    request: &EditorAssistRequest,
) -> Result<(), EditorAssistRequestError> {
    validate_buffer_range(buf, request.buffer_range.clone()).map_err(|err| match err {
        RangeValidationError::StartAfterEnd => EditorAssistRequestError::RangeStartAfterEnd,
        RangeValidationError::OutOfBounds { start, end, len } => {
            EditorAssistRequestError::RangeOutOfBounds { start, end, len }
        }
        RangeValidationError::NotCharBoundary { start, end } => {
            EditorAssistRequestError::RangeNotCharBoundary { start, end }
        }
    })?;

    let len = buf.len_bytes();
    if request.selection.anchor > len || request.selection.focus > len {
        return Err(EditorAssistRequestError::SelectionOutOfBounds {
            anchor: request.selection.anchor,
            focus: request.selection.focus,
            len,
        });
    }
    if !buf.is_char_boundary(request.selection.anchor)
        || !buf.is_char_boundary(request.selection.focus)
    {
        return Err(EditorAssistRequestError::SelectionNotCharBoundary {
            anchor: request.selection.anchor,
            focus: request.selection.focus,
        });
    }

    match display_map {
        Some(display_map) => {
            let row_count = display_map.row_count();
            if request.display_point.row >= row_count {
                return Err(EditorAssistRequestError::DisplayRowOutOfBounds {
                    row: request.display_point.row,
                    row_count,
                });
            }
        }
        None => {
            return Err(EditorAssistRequestError::DisplayRowRequiresDisplayMap {
                row: request.display_point.row,
            });
        }
    }

    if request.anchor_id.as_deref().is_some_and(str::is_empty) {
        return Err(EditorAssistRequestError::EmptyAnchorId);
    }
    if matches!(&request.trigger, EditorAssistTrigger::Custom(v) if v.is_empty())
        || matches!(&request.trigger, EditorAssistTrigger::Character(v) if v.is_empty())
    {
        return Err(EditorAssistRequestError::EmptyTrigger);
    }

    Ok(())
}

pub fn validate_completion_list(
    buf: &TextBuffer,
    list: &CompletionList,
) -> Result<(), CompletionListError> {
    if list.request_id.is_empty() {
        return Err(CompletionListError::EmptyRequestId);
    }

    let mut ids = BTreeSet::<Arc<str>>::new();
    for candidate in &list.candidates {
        if candidate.id.is_empty() {
            return Err(CompletionListError::EmptyCandidateId);
        }
        if !ids.insert(candidate.id.clone()) {
            return Err(CompletionListError::DuplicateCandidateId(
                candidate.id.clone(),
            ));
        }
        if candidate.label.is_empty() {
            return Err(CompletionListError::EmptyCandidateLabel);
        }
        if let Some(range) = &candidate.replace_range {
            validate_buffer_range(buf, range.clone()).map_err(|err| match err {
                RangeValidationError::StartAfterEnd => {
                    CompletionListError::ReplaceRangeStartAfterEnd
                }
                RangeValidationError::OutOfBounds { start, end, len } => {
                    CompletionListError::ReplaceRangeOutOfBounds { start, end, len }
                }
                RangeValidationError::NotCharBoundary { start, end } => {
                    CompletionListError::ReplaceRangeNotCharBoundary { start, end }
                }
            })?;
        }
        if candidate.commit_characters.iter().any(|v| v.is_empty()) {
            return Err(CompletionListError::EmptyCommitCharacter);
        }
        if candidate.command_id.as_deref().is_some_and(str::is_empty) {
            return Err(CompletionListError::EmptyCommandId);
        }
    }

    if let Some(active_id) = &list.active_id
        && !ids.contains(active_id)
    {
        return Err(CompletionListError::ActiveCandidateMissing(
            active_id.clone(),
        ));
    }

    Ok(())
}

pub fn validate_hover_payload(
    buf: &TextBuffer,
    payload: &HoverPayload,
) -> Result<(), HoverPayloadError> {
    if payload.id.is_empty() {
        return Err(HoverPayloadError::EmptyId);
    }
    if payload.contents.is_empty() {
        return Err(HoverPayloadError::EmptyContents);
    }
    validate_buffer_range(buf, payload.range.clone()).map_err(|err| match err {
        RangeValidationError::StartAfterEnd => HoverPayloadError::RangeStartAfterEnd,
        RangeValidationError::OutOfBounds { start, end, len } => {
            HoverPayloadError::RangeOutOfBounds { start, end, len }
        }
        RangeValidationError::NotCharBoundary { start, end } => {
            HoverPayloadError::RangeNotCharBoundary { start, end }
        }
    })?;
    if payload.related_command_ids.iter().any(|v| v.is_empty()) {
        return Err(HoverPayloadError::EmptyCommandId);
    }
    Ok(())
}

pub fn validate_code_action_list(
    buf: &TextBuffer,
    list: &CodeActionList,
) -> Result<(), CodeActionListError> {
    if list.request_id.is_empty() {
        return Err(CodeActionListError::EmptyRequestId);
    }
    validate_buffer_range(buf, list.range.clone()).map_err(|err| match err {
        RangeValidationError::StartAfterEnd => CodeActionListError::RangeStartAfterEnd,
        RangeValidationError::OutOfBounds { start, end, len } => {
            CodeActionListError::RangeOutOfBounds { start, end, len }
        }
        RangeValidationError::NotCharBoundary { start, end } => {
            CodeActionListError::RangeNotCharBoundary { start, end }
        }
    })?;

    let mut ids = BTreeSet::<Arc<str>>::new();
    for action in &list.actions {
        if action.id.is_empty() {
            return Err(CodeActionListError::EmptyActionId);
        }
        if !ids.insert(action.id.clone()) {
            return Err(CodeActionListError::DuplicateActionId(action.id.clone()));
        }
        if action.title.is_empty() {
            return Err(CodeActionListError::EmptyActionTitle);
        }
        if action.command_id.is_empty() {
            return Err(CodeActionListError::EmptyCommandId);
        }
        if action.related_diagnostic_ids.iter().any(|id| id.is_empty()) {
            return Err(CodeActionListError::EmptyDiagnosticId);
        }
        if matches!(&action.kind, CodeActionKind::Custom(kind) if kind.is_empty()) {
            return Err(CodeActionListError::EmptyCustomKind);
        }
    }

    Ok(())
}

enum RangeValidationError {
    StartAfterEnd,
    OutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    NotCharBoundary {
        start: usize,
        end: usize,
    },
}

fn validate_buffer_range(
    buf: &TextBuffer,
    range: Range<usize>,
) -> Result<(), RangeValidationError> {
    let len = buf.len_bytes();
    if range.start > range.end {
        return Err(RangeValidationError::StartAfterEnd);
    }
    if range.start > len || range.end > len {
        return Err(RangeValidationError::OutOfBounds {
            start: range.start,
            end: range.end,
            len,
        });
    }
    if !buf.is_char_boundary(range.start) || !buf.is_char_boundary(range.end) {
        return Err(RangeValidationError::NotCharBoundary {
            start: range.start,
            end: range.end,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_code_editor_buffer::{DocId, Revision, Selection};

    fn buffer(text: &str) -> TextBuffer {
        TextBuffer::new(DocId::new(), text.to_string()).unwrap()
    }

    #[test]
    fn assist_request_validates_buffer_selection_and_display_point() {
        let buf = buffer("alpha\nbeta");
        let map = DisplayMap::new(&buf, Some(3));
        let request = EditorAssistRequest::new(
            EditorAssistKind::Completion,
            buf.revision(),
            Selection {
                anchor: 2,
                focus: 2,
            },
            2..2,
            DisplayPoint::new(0, 2),
        );

        assert_eq!(
            validate_editor_assist_request(&buf, Some(&map), &request),
            Ok(())
        );
    }

    #[test]
    fn assist_request_requires_display_map_for_display_anchor_validation() {
        let buf = buffer("alpha");
        let request = EditorAssistRequest::new(
            EditorAssistKind::Hover,
            Revision(0),
            Selection::default(),
            0..0,
            DisplayPoint::new(0, 0),
        );

        assert_eq!(
            validate_editor_assist_request(&buf, None, &request),
            Err(EditorAssistRequestError::DisplayRowRequiresDisplayMap { row: 0 })
        );
    }

    #[test]
    fn assist_request_rejects_non_char_boundary_selection() {
        let buf = buffer("aé");
        let map = DisplayMap::new(&buf, None);
        let request = EditorAssistRequest::new(
            EditorAssistKind::Hover,
            Revision(0),
            Selection {
                anchor: 2,
                focus: 3,
            },
            0..0,
            DisplayPoint::new(0, 0),
        );

        assert_eq!(
            validate_editor_assist_request(&buf, Some(&map), &request),
            Err(EditorAssistRequestError::SelectionNotCharBoundary {
                anchor: 2,
                focus: 3
            })
        );
    }

    #[test]
    fn completion_list_validates_active_candidate_and_replace_ranges() {
        let buf = buffer("abcdef");
        let mut item = CompletionCandidate::new("item.1", "abcdef");
        item.replace_range = Some(1..4);
        item.commit_kind = CompletionCommitKind::Replace;

        let mut list = CompletionList::new("req.1", buf.revision(), vec![item]);
        list.active_id = Some("item.1".into());

        assert_eq!(validate_completion_list(&buf, &list), Ok(()));
    }

    #[test]
    fn completion_list_rejects_missing_active_candidate() {
        let buf = buffer("abcdef");
        let mut list = CompletionList::new(
            "req.1",
            buf.revision(),
            vec![CompletionCandidate::new("item.1", "abc")],
        );
        list.active_id = Some("missing".into());

        assert_eq!(
            validate_completion_list(&buf, &list),
            Err(CompletionListError::ActiveCandidateMissing(
                "missing".into()
            ))
        );
    }

    #[test]
    fn completion_list_rejects_duplicate_candidate_ids() {
        let buf = buffer("abcdef");
        let list = CompletionList::new(
            "req.1",
            buf.revision(),
            vec![
                CompletionCandidate::new("dup", "a"),
                CompletionCandidate::new("dup", "b"),
            ],
        );

        assert_eq!(
            validate_completion_list(&buf, &list),
            Err(CompletionListError::DuplicateCandidateId("dup".into()))
        );
    }

    #[test]
    fn hover_payload_keeps_commands_as_data_only() {
        let buf = buffer("abcdef");
        let mut hover = HoverPayload::new("hover.1", buf.revision(), 1..4, "symbol docs");
        hover
            .related_command_ids
            .push("editor.open_symbol_docs".into());

        assert_eq!(validate_hover_payload(&buf, &hover), Ok(()));
    }

    #[test]
    fn hover_payload_rejects_empty_contents() {
        let buf = buffer("abcdef");
        let hover = HoverPayload::new("hover.1", buf.revision(), 1..4, "");

        assert_eq!(
            validate_hover_payload(&buf, &hover),
            Err(HoverPayloadError::EmptyContents)
        );
    }

    #[test]
    fn code_action_list_validates_command_ids_without_menu_policy() {
        let buf = buffer("abcdef");
        let mut action = CodeAction::new("action.1", "Apply fix", "editor.apply_fix");
        action.related_diagnostic_ids.push("diag.1".into());
        action.is_preferred = true;

        let list = CodeActionList::new("req.1", buf.revision(), 1..4, vec![action]);

        assert_eq!(validate_code_action_list(&buf, &list), Ok(()));
    }

    #[test]
    fn code_action_list_rejects_empty_custom_kind() {
        let buf = buffer("abcdef");
        let mut action = CodeAction::new("action.1", "Apply fix", "editor.apply_fix");
        action.kind = CodeActionKind::Custom("".into());
        let list = CodeActionList::new("req.1", buf.revision(), 1..4, vec![action]);

        assert_eq!(
            validate_code_action_list(&buf, &list),
            Err(CodeActionListError::EmptyCustomKind)
        );
    }
}
