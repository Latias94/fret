use std::sync::Arc;

use fret_code_editor_buffer::{Revision, TextBuffer};
use fret_code_editor_view::{
    DiagnosticLineSummary, DiagnosticSpan, DisplayMap, GutterMarker, GutterMarkerAnchor,
    RangeDecoration, SemanticToken,
};

/// Feature-payload counts exposed to diagnostics bundles.
///
/// The payloads are source/display facts owned by the editor surface; presentation policy and
/// overlay behavior stay outside `fret-code-editor`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CodeEditorFeaturePayloadSnapshot {
    pub schema_version: u32,

    pub epoch: u64,
    pub buffer_revision: u64,
    pub display_map_epoch: u64,

    pub diagnostic_spans_count: u64,
    pub diagnostic_line_summaries_count: u64,
    pub range_decorations_count: u64,
    pub gutter_markers_count: u64,
    pub semantic_tokens_count: u64,
}

#[derive(Debug, Clone)]
pub(super) struct CodeEditorFeaturePayloadStore {
    buffer_revision: Revision,
    display_map_epoch: u64,
    epoch: u64,
    diagnostic_spans: Arc<[DiagnosticSpan]>,
    diagnostic_line_summaries: Arc<[DiagnosticLineSummary]>,
    range_decorations: Arc<[RangeDecoration]>,
    gutter_markers: Arc<[GutterMarker]>,
    semantic_tokens: Arc<[SemanticToken]>,
}

impl CodeEditorFeaturePayloadStore {
    pub(super) fn new(buffer_revision: Revision, display_map_epoch: u64) -> Self {
        Self {
            buffer_revision,
            display_map_epoch,
            epoch: 0,
            diagnostic_spans: Arc::from(Vec::<DiagnosticSpan>::new()),
            diagnostic_line_summaries: Arc::from(Vec::<DiagnosticLineSummary>::new()),
            range_decorations: Arc::from(Vec::<RangeDecoration>::new()),
            gutter_markers: Arc::from(Vec::<GutterMarker>::new()),
            semantic_tokens: Arc::from(Vec::<SemanticToken>::new()),
        }
    }

    pub(super) fn epoch(&self) -> u64 {
        self.epoch
    }

    pub(super) fn diagnostic_line_summaries(&self) -> &[DiagnosticLineSummary] {
        &self.diagnostic_line_summaries
    }

    fn is_empty(&self) -> bool {
        self.diagnostic_spans.is_empty()
            && self.diagnostic_line_summaries.is_empty()
            && self.range_decorations.is_empty()
            && self.gutter_markers.is_empty()
            && self.semantic_tokens.is_empty()
    }

    fn bump_epoch(&mut self) {
        self.epoch = self.epoch.saturating_add(1);
    }

    pub(super) fn clear_all_for_buffer_change(
        &mut self,
        buffer_revision: Revision,
        display_map_epoch: u64,
    ) -> bool {
        let changed = !self.is_empty();
        self.buffer_revision = buffer_revision;
        self.display_map_epoch = display_map_epoch;
        if changed {
            self.diagnostic_spans = Arc::from(Vec::<DiagnosticSpan>::new());
            self.diagnostic_line_summaries = Arc::from(Vec::<DiagnosticLineSummary>::new());
            self.range_decorations = Arc::from(Vec::<RangeDecoration>::new());
            self.gutter_markers = Arc::from(Vec::<GutterMarker>::new());
            self.semantic_tokens = Arc::from(Vec::<SemanticToken>::new());
            self.bump_epoch();
        }
        changed
    }

    pub(super) fn retain_gutter_markers_valid_for_display_map(
        &mut self,
        buf: &TextBuffer,
        display_map: &DisplayMap,
        display_map_epoch: u64,
    ) -> bool {
        self.display_map_epoch = display_map_epoch;
        if self.gutter_markers.is_empty() {
            return false;
        }

        let line_count = buf.line_count().max(1);
        let row_count = display_map.row_count();
        let retained = self
            .gutter_markers
            .iter()
            .filter(|marker| match marker.anchor {
                GutterMarkerAnchor::LogicalLine(line) => line < line_count,
                GutterMarkerAnchor::DisplayRow(row) => row < row_count,
            })
            .cloned()
            .collect::<Vec<_>>();

        if retained.len() == self.gutter_markers.len() {
            return false;
        }

        self.gutter_markers = Arc::from(retained);
        self.bump_epoch();
        true
    }

    pub(super) fn set_diagnostic_spans(
        &mut self,
        buffer_revision: Revision,
        spans: Vec<DiagnosticSpan>,
        summaries: Vec<DiagnosticLineSummary>,
    ) -> bool {
        if self.buffer_revision == buffer_revision
            && self.diagnostic_spans.as_ref() == spans.as_slice()
            && self.diagnostic_line_summaries.as_ref() == summaries.as_slice()
        {
            return false;
        }
        self.buffer_revision = buffer_revision;
        self.diagnostic_spans = Arc::from(spans);
        self.diagnostic_line_summaries = Arc::from(summaries);
        self.bump_epoch();
        true
    }

    pub(super) fn set_range_decorations(
        &mut self,
        buffer_revision: Revision,
        decorations: Vec<RangeDecoration>,
    ) -> bool {
        if self.buffer_revision == buffer_revision
            && self.range_decorations.as_ref() == decorations.as_slice()
        {
            return false;
        }
        self.buffer_revision = buffer_revision;
        self.range_decorations = Arc::from(decorations);
        self.bump_epoch();
        true
    }

    pub(super) fn set_gutter_markers(
        &mut self,
        buffer_revision: Revision,
        display_map_epoch: u64,
        markers: Vec<GutterMarker>,
    ) -> bool {
        if self.buffer_revision == buffer_revision
            && self.display_map_epoch == display_map_epoch
            && self.gutter_markers.as_ref() == markers.as_slice()
        {
            return false;
        }
        self.buffer_revision = buffer_revision;
        self.display_map_epoch = display_map_epoch;
        self.gutter_markers = Arc::from(markers);
        self.bump_epoch();
        true
    }

    pub(super) fn set_semantic_tokens(
        &mut self,
        buffer_revision: Revision,
        tokens: Vec<SemanticToken>,
    ) -> bool {
        if self.buffer_revision == buffer_revision
            && self.semantic_tokens.as_ref() == tokens.as_slice()
        {
            return false;
        }
        self.buffer_revision = buffer_revision;
        self.semantic_tokens = Arc::from(tokens);
        self.bump_epoch();
        true
    }

    pub(super) fn snapshot(&self) -> CodeEditorFeaturePayloadSnapshot {
        CodeEditorFeaturePayloadSnapshot {
            schema_version: 1,
            epoch: self.epoch,
            buffer_revision: self.buffer_revision.0,
            display_map_epoch: self.display_map_epoch,
            diagnostic_spans_count: self.diagnostic_spans.len() as u64,
            diagnostic_line_summaries_count: self.diagnostic_line_summaries.len() as u64,
            range_decorations_count: self.range_decorations.len() as u64,
            gutter_markers_count: self.gutter_markers.len() as u64,
            semantic_tokens_count: self.semantic_tokens.len() as u64,
        }
    }
}
