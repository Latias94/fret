//! Implementation details for the Fret code editor surface.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
#[cfg(feature = "syntax")]
use std::sync::Mutex;
use std::time::{Duration, Instant};

use fret_code_editor_buffer::{
    DocId, Edit, Selection, TextBuffer, TextBufferTransaction, TextBufferTx,
};
use fret_code_editor_view::code_wrap_policy::{CodeWrapPolicy, CodeWrapPreset};
use fret_code_editor_view::{
    DiagnosticLineSummary, DiagnosticSpan, DiagnosticSpanError, DisplayMap, DisplayPoint, FoldSpan,
    GutterMarker, GutterMarkerError, InlaySpan, InlinePreedit, RangeDecoration,
    RangeDecorationError, SemanticToken, SemanticTokenError, diagnostic_line_summaries,
    move_word_left_in_buffer, move_word_right_in_buffer, normalized_diagnostic_spans,
    normalized_gutter_markers, normalized_range_decorations, normalized_semantic_tokens,
    select_word_range_in_buffer, validate_gutter_markers,
};
use fret_core::{
    AttributedText, CaretAffinity, Color, Corners, CursorIcon, DecorationLineStyle, DrawOrder,
    Edges, FontId, KeyCode, Modifiers, MouseButton, Point, PointerType, Px, Rect, SceneOp, Size,
    TextFontFeatureSetting, TextOverflow, TextPaintStyle, TextShapingStyle, TextSpan, TextStyle,
    TextWrap, UnderlineStyle,
};
use fret_runtime::{ClipboardToken, Effect, TextBoundaryMode, TimerToken};
#[cfg(feature = "syntax")]
use fret_runtime::{DispatcherHandle, ExecBackgroundWork, ExecWake};
use fret_ui::Invalidation;
use fret_ui::action::{ActionCx, KeyDownCx, OnTimer, UiActionHost, UiPointerActionHost};
use fret_ui::canvas::CanvasTextConstraints;
use fret_ui::element::AnyElement;
use fret_ui::element::{
    CanvasCachePolicy, CanvasCacheTuning, Length, Overflow, PointerRegionProps,
    SemanticsDecoration, TextInputRegionProps,
};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::windowed_rows_surface::{
    OnWindowedRowsPaintFrame, OnWindowedRowsPointerCancel, OnWindowedRowsPointerDown,
    OnWindowedRowsPointerMove, OnWindowedRowsPointerUp, OnWindowedRowsPrepaintFrame,
    WindowedRowsPaintDiagnostics, WindowedRowsPaintFrame, WindowedRowsSurfacePointerHandlers,
    WindowedRowsSurfaceProps, windowed_rows_surface_with_pointer_region,
};
use fret_undo::{CoalesceKey, InvertibleTransaction, UndoHistory, UndoRecord};

mod a11y;
mod diagnostics;
mod feature_payloads;
mod geom;
mod handle;
mod input;
mod paint;
mod state;
mod syntax;
#[cfg(test)]
mod tests;

use a11y::{a11y_composed_text_window, map_a11y_offsets_to_buffer_composed};
pub use diagnostics::{
    CodeEditorCacheSizeSnapshotV1, CodeEditorCacheStats, CodeEditorMemorySnapshotV1,
    CodeEditorPaintPerfFrame,
};
use diagnostics::{
    estimate_text_buffer_tx_text_bytes_and_edits, normalized_paint_frame_visible_window,
    paint_frame_cache_min_entries, paint_frame_visible_row_count, paint_perf_enabled_from_env,
};
pub use feature_payloads::CodeEditorFeaturePayloadSnapshotV1;
use feature_payloads::CodeEditorFeaturePayloadStore;
#[cfg(test)]
use geom::caret_rect_for_selection;
use geom::{
    RowGeom, RowPreeditMapping, caret_for_pointer, caret_x_for_buffer_byte_in_row,
    caret_x_for_index, hit_test_index_from_caret_stops, preedit_cursor_offset_bytes,
    preedit_cursor_offset_cols,
};
pub use handle::CodeEditorHandle;
use state::*;
#[cfg(feature = "syntax")]
use syntax::{SyntaxPrefetchRuntime, SyntaxSpan};

const DRAG_AUTOSCROLL_TICK: Duration = Duration::from_millis(16);
const CODE_EDITOR_ROW_CACHE_MIN_ENTRIES: usize = 256;
const CODE_EDITOR_ROW_CACHE_MAX_ENTRIES: usize = 8_192;

pub(super) fn preedit_cursor_bytes_for_marked_range_utf16(
    insertion_start_utf16: u32,
    marked: fret_runtime::Utf16Range,
    text: &str,
) -> (usize, usize) {
    let text_len_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
        text,
        text.len(),
        fret_core::utf::UtfIndexClamp::Down,
    );

    let base = usize::try_from(insertion_start_utf16).unwrap_or(usize::MAX);
    let marked = marked.normalized();
    let rel_start = usize::try_from(marked.start)
        .unwrap_or(usize::MAX)
        .saturating_sub(base)
        .min(text_len_utf16);
    let rel_end = usize::try_from(marked.end)
        .unwrap_or(usize::MAX)
        .saturating_sub(base)
        .min(text_len_utf16);

    fret_core::utf::utf16_range_to_utf8_byte_range(text, rel_start, rel_end)
}

fn platform_replace_and_mark_text_in_range_utf16(
    st: &mut CodeEditorState,
    text_cache_max_entries: usize,
    value: &str,
    range: fret_runtime::Utf16Range,
    text: &str,
    marked: Option<fret_runtime::Utf16Range>,
    selected: Option<fret_runtime::Utf16Range>,
) -> bool {
    if !st.interaction.enabled || !st.interaction.editable {
        st.set_preedit(None);
        st.undo_group = None;
        return false;
    }

    let range = range.normalized();
    let (start_byte, end_byte) = fret_core::utf::utf16_range_to_utf8_byte_range(
        value,
        usize::try_from(range.start).unwrap_or(usize::MAX),
        usize::try_from(range.end).unwrap_or(usize::MAX),
    );
    let start_offset = u32::try_from(start_byte.min(value.len())).unwrap_or(u32::MAX);
    let end_offset = u32::try_from(end_byte.min(value.len())).unwrap_or(u32::MAX);

    let start =
        a11y::map_a11y_offset_to_buffer_in_current_window(st, text_cache_max_entries, start_offset);
    let end =
        a11y::map_a11y_offset_to_buffer_in_current_window(st, text_cache_max_entries, end_offset);
    let start = start.min(st.buffer.len_bytes());
    let end = end.min(st.buffer.len_bytes());

    if marked.is_none() {
        let start = start.min(end);
        let end = start.max(end);

        st.preedit_replace_range = None;
        st.preedit_saved_selection = None;
        let caret = start.saturating_add(text.len()).min(st.buffer.len_bytes());
        return input::apply_and_record_edit(
            st,
            UndoGroupKind::Typing,
            Edit::Replace {
                range: start..end,
                text: text.to_string(),
            },
            Selection {
                anchor: caret,
                focus: caret,
            },
        )
        .is_some();
    }

    let start = start.min(end);
    let mut end = start.max(end);

    // Staging contract: selection-replacing composition is best-effort and currently treated as
    // single-line. Clamp any multi-line replacement range to the anchor logical line so the view
    // model remains deterministic while we stage multi-line composition support.
    let anchor_line = st.buffer.line_index_at_byte(start);
    if let Some(line_range) = st.buffer.line_byte_range(anchor_line) {
        end = end.min(line_range.end);
    }

    let mut did = false;
    st.undo_group = None;

    if st.preedit_saved_selection.is_none() {
        st.preedit_saved_selection = Some(st.selection);
        did = true;
    }

    // Treat an empty composition update as a cancel/unmark event (best-effort).
    // Restore the selection that existed when composition began.
    if text.is_empty() {
        let had_preedit_state = st.preedit.is_some()
            || st
                .preedit_replace_range
                .as_ref()
                .is_some_and(|r| !r.is_empty())
            || st.preedit_saved_selection.is_some();
        let restore = st.preedit_saved_selection.unwrap_or(Selection {
            anchor: start,
            focus: end,
        });
        if st.selection != restore {
            st.selection = restore;
            did = true;
        }
        st.set_preedit(None);
        return did || had_preedit_state;
    }

    let next_replace = (start != end).then_some(start..end);
    if st.preedit_replace_range != next_replace {
        st.preedit_replace_range = next_replace;
        did = true;
    }

    let target = Selection {
        anchor: start,
        focus: start,
    };
    if st.selection != target {
        st.selection = target;
        did = true;
    }

    let Some(marked) = marked else {
        return did;
    };

    let cursor_range = selected.unwrap_or(marked);
    let (bs, be) = preedit_cursor_bytes_for_marked_range_utf16(range.start, cursor_range, text);
    let next = (!text.is_empty()).then_some(PreeditState {
        text: text.to_string(),
        cursor: Some((bs, be)),
    });
    if st.preedit != next {
        did = true;
    }
    st.set_preedit(next);

    did
}

fn scale_vertical_mouse_autoscroll_delta(delta_px: f32) -> f32 {
    (delta_px.max(0.0).powf(1.2) / 100.0).min(3.0)
}

fn drag_autoscroll_delta_y(viewport_h: Px, row_h: Px, viewport_y: Px) -> Px {
    if viewport_h.0 <= 0.0 {
        return Px(0.0);
    }
    let vertical_margin = Px(row_h.0.min(viewport_h.0 / 3.0));
    if vertical_margin.0 <= 0.0 {
        return Px(0.0);
    }

    let top = vertical_margin.0;
    let bottom = viewport_h.0 - vertical_margin.0;

    if viewport_y.0 < top {
        Px(-scale_vertical_mouse_autoscroll_delta(top - viewport_y.0))
    } else if viewport_y.0 > bottom {
        Px(scale_vertical_mouse_autoscroll_delta(viewport_y.0 - bottom))
    } else {
        Px(0.0)
    }
}

#[cfg(test)]
fn display_row_for_pointer_y(bounds: Rect, row_h: Px, pointer_y: Px, rows: usize) -> Option<usize> {
    if rows == 0 || row_h.0 <= 0.0 {
        return None;
    }

    let local_y = (pointer_y.0 - bounds.origin.y.0) / row_h.0;
    if !local_y.is_finite() {
        return None;
    }

    let mut row = local_y.floor() as isize;
    if row < 0 {
        row = 0;
    }
    let max_row = rows.saturating_sub(1) as isize;
    if row > max_row {
        row = max_row;
    }
    Some(row as usize)
}

fn viewport_pos_for_pointer(
    bounds: Rect,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
    pos: fret_core::Point,
) -> fret_core::Point {
    let offset = scroll_handle.offset();
    let viewport = scroll_handle.viewport_size();
    let viewport_w = Px(viewport.width.0.max(0.0));
    let viewport_h = Px(viewport.height.0.max(0.0));

    let local_x = Px(pos.x.0 - bounds.origin.x.0);
    let local_y = Px(pos.y.0 - bounds.origin.y.0);

    let y_viewport = local_y;
    let y_content = Px(local_y.0 - offset.y.0);

    // Pointer event positions are mapped through transforms. Within scroll containers, descendants
    // typically see "content space" coordinates already. Prefer the interpretation that places the
    // cursor position closer to the viewport.
    let range_min = -viewport_h.0;
    let range_max = viewport_h.0 * 2.0;
    let plausible = |y: Px| y.0 >= range_min && y.0 <= range_max;
    let score = |y: Px| (y.0 - (viewport_h.0 / 2.0)).abs();

    let y = match (plausible(y_viewport), plausible(y_content)) {
        (true, false) => y_viewport,
        (false, true) => y_content,
        _ => {
            if score(y_content) < score(y_viewport) {
                y_content
            } else {
                y_viewport
            }
        }
    };

    let x = if viewport_w.0 > 0.0 {
        Px(local_x.0.clamp(0.0, viewport_w.0))
    } else {
        local_x
    };

    fret_core::Point::new(x, y)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreeditState {
    pub text: String,
    pub cursor: Option<(usize, usize)>,
}

/// Editor-owned OpenType feature policy for code surfaces.
///
/// This is intentionally an ecosystem-layer surface: mechanism-layer text types expose a generic
/// feature representation (`TextShapingStyle.features`), and editors/components decide defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeFontFeaturePreset {
    /// Do not override font feature defaults.
    PreserveDefaults,
    /// Common editor baseline: disable standard ligatures (`liga`) and contextual alternates
    /// (`calt`), best-effort.
    EditorDefault,
    /// Disable standard ligatures (`liga`) only, best-effort.
    NoLigatures,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeFontFeaturePolicy {
    pub preset: CodeFontFeaturePreset,
    /// Additional feature overrides applied after the preset (best-effort).
    pub overrides: Vec<TextFontFeatureSetting>,
}

impl Default for CodeFontFeaturePolicy {
    fn default() -> Self {
        Self {
            preset: CodeFontFeaturePreset::EditorDefault,
            overrides: Vec::new(),
        }
    }
}

impl CodeFontFeaturePolicy {
    fn shaping_style(&self) -> TextShapingStyle {
        let mut out = TextShapingStyle::default();
        match self.preset {
            CodeFontFeaturePreset::PreserveDefaults => {}
            CodeFontFeaturePreset::EditorDefault => {
                out.features.push(TextFontFeatureSetting {
                    tag: "liga".into(),
                    value: 0,
                });
                out.features.push(TextFontFeatureSetting {
                    tag: "calt".into(),
                    value: 0,
                });
            }
            CodeFontFeaturePreset::NoLigatures => {
                out.features.push(TextFontFeatureSetting {
                    tag: "liga".into(),
                    value: 0,
                });
            }
        }

        if !self.overrides.is_empty() {
            out.features.extend(self.overrides.iter().cloned());
        }

        out
    }
}

/// Controls how the code editor surface participates in focus, selection, and editing.
///
/// This is intentionally an ecosystem-layer policy surface (ADR 0066).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeEditorInteractionOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub selectable: bool,
    pub editable: bool,
}

impl CodeEditorInteractionOptions {
    pub fn editor() -> Self {
        Self {
            enabled: true,
            focusable: true,
            selectable: true,
            editable: true,
        }
    }

    pub fn read_only() -> Self {
        Self {
            enabled: true,
            focusable: true,
            selectable: true,
            editable: false,
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            focusable: false,
            selectable: false,
            editable: false,
        }
    }
}

impl Default for CodeEditorInteractionOptions {
    fn default() -> Self {
        Self::editor()
    }
}

#[derive(Debug, Clone)]
struct CodeEditorTx {
    buffer_tx: TextBufferTx,
    selection: Selection,
    inverse_selection: Selection,
}

impl InvertibleTransaction for CodeEditorTx {
    fn invert(&self) -> Self {
        Self {
            buffer_tx: self.buffer_tx.invert(),
            selection: self.inverse_selection,
            inverse_selection: self.selection,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UndoGroupKind {
    Typing,
    Paste,
    Cut,
    Backspace,
    DeleteForward,
}

impl UndoGroupKind {
    fn coalesce_key(self) -> CoalesceKey {
        match self {
            Self::Typing => CoalesceKey::from("code-editor.typing"),
            Self::Paste => CoalesceKey::from("code-editor.paste"),
            Self::Cut => CoalesceKey::from("code-editor.cut"),
            Self::Backspace => CoalesceKey::from("code-editor.backspace"),
            Self::DeleteForward => CoalesceKey::from("code-editor.delete_forward"),
        }
    }
}

#[derive(Debug, Clone)]
struct UndoGroup {
    kind: UndoGroupKind,
    before_selection: Selection,
    tx: TextBufferTransaction,
    coalesce_key: CoalesceKey,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone)]
struct RowRichPrefetchKey {
    doc: DocId,
    rev: fret_code_editor_buffer::Revision,
    language: Arc<str>,
    row: usize,
    row_range: Range<usize>,
    theme_revision: u64,
    code_font_feature_policy_rev: u64,
    line: Arc<str>,
    syntax_spans: Arc<[SyntaxSpan]>,
    row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
}

#[cfg(feature = "syntax")]
impl RowRichPrefetchKey {
    fn new(
        doc: DocId,
        rev: fret_code_editor_buffer::Revision,
        language: Arc<str>,
        row: usize,
        row_range: Range<usize>,
        theme_revision: u64,
        code_font_feature_policy_rev: u64,
        line: Arc<str>,
        syntax_spans: Arc<[SyntaxSpan]>,
        row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
    ) -> Self {
        Self {
            doc,
            rev,
            language,
            row,
            row_range,
            theme_revision,
            code_font_feature_policy_rev,
            line,
            syntax_spans,
            row_spans,
        }
    }

    fn matches(&self, other: &Self) -> bool {
        self.doc == other.doc
            && self.rev == other.rev
            && self.language.as_ref() == other.language.as_ref()
            && self.row == other.row
            && self.row_range == other.row_range
            && self.theme_revision == other.theme_revision
            && self.code_font_feature_policy_rev == other.code_font_feature_policy_rev
            && Arc::ptr_eq(&self.line, &other.line)
            && Arc::ptr_eq(&self.syntax_spans, &other.syntax_spans)
            && Arc::ptr_eq(&self.row_spans, &other.row_spans)
    }
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone)]
struct RowRichPrefetchChunk {
    key: RowRichPrefetchKey,
    rich: AttributedText,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Default)]
struct RowRichPrefetchRuntimeState {
    pending: Vec<RowRichPrefetchKey>,
    ready: VecDeque<RowRichPrefetchChunk>,
    last_visible_start: Option<usize>,
}

#[cfg(feature = "syntax")]
#[derive(Clone)]
struct RowRichPrefetchRuntime {
    shared: Arc<Mutex<RowRichPrefetchRuntimeState>>,
    dispatcher: DispatcherHandle,
}

#[cfg(feature = "syntax")]
impl std::fmt::Debug for RowRichPrefetchRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RowRichPrefetchRuntime")
            .field("shared", &self.shared)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "syntax")]
impl RowRichPrefetchRuntime {
    fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            shared: Arc::new(Mutex::new(RowRichPrefetchRuntimeState::default())),
            dispatcher,
        }
    }

    fn clear(&self) {
        let mut state = self.lock_state();
        state.pending.clear();
        state.ready.clear();
        state.last_visible_start = None;
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, RowRichPrefetchRuntimeState> {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn note_visible_start(&self, visible_start: usize) -> i8 {
        let mut state = self.lock_state();
        let direction = match state.last_visible_start {
            Some(prev) if visible_start < prev => -1,
            Some(prev) if visible_start > prev => 1,
            _ => 1,
        };
        state.last_visible_start = Some(visible_start);
        direction
    }

    fn drain_ready(&self) -> Vec<RowRichPrefetchChunk> {
        let mut state = self.lock_state();
        state.ready.drain(..).collect()
    }

    fn try_mark_pending(&self, key: RowRichPrefetchKey) -> bool {
        const MAX_PENDING: usize = 3;

        let mut state = self.lock_state();
        if state.pending.iter().any(|pending| pending.matches(&key))
            || state.ready.iter().any(|chunk| chunk.key.matches(&key))
        {
            return false;
        }
        if state.pending.len() >= MAX_PENDING {
            return false;
        }
        state.pending.push(key);
        true
    }
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RowSceneTextStyleKey {
    font: FontId,
    size_bits: u32,
    weight: fret_core::FontWeight,
    slant: fret_core::TextSlant,
    line_height_bits: Option<u32>,
    letter_spacing_em_bits: Option<u32>,
}

#[cfg(feature = "syntax")]
impl RowSceneTextStyleKey {
    fn from_style(style: &TextStyle) -> Self {
        Self {
            font: style.font.clone(),
            size_bits: style.size.0.to_bits(),
            weight: style.weight,
            slant: style.slant,
            line_height_bits: style.line_height.map(|h| h.0.to_bits()),
            letter_spacing_em_bits: style.letter_spacing_em.map(f32::to_bits),
        }
    }
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RowSceneTextConstraintsKey {
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    overflow: TextOverflow,
}

#[cfg(feature = "syntax")]
impl RowSceneTextConstraintsKey {
    fn from_constraints(constraints: CanvasTextConstraints) -> Self {
        let max_width_bits = match constraints.wrap {
            TextWrap::None if constraints.overflow != TextOverflow::Ellipsis => None,
            _ => constraints.max_width.map(|w| w.0.to_bits()),
        };
        Self {
            max_width_bits,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
        }
    }
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone)]
struct RowSceneSyntaxReplayKey {
    row_range: Range<usize>,
    line: Arc<str>,
    row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
    syntax_spans: Arc<[SyntaxSpan]>,
    text_style: RowSceneTextStyleKey,
    constraints: RowSceneTextConstraintsKey,
    font_stack_key: u64,
    scale_bits: u32,
    theme_revision: u64,
    code_font_feature_policy_rev: u64,
    fg: ColorKey,
}

#[cfg(feature = "syntax")]
impl RowSceneSyntaxReplayKey {
    #[allow(clippy::too_many_arguments)]
    fn new(
        row_range: Range<usize>,
        line: Arc<str>,
        row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
        syntax_spans: Arc<[SyntaxSpan]>,
        text_style: &TextStyle,
        constraints: CanvasTextConstraints,
        font_stack_key: fret_runtime::TextFontStackKey,
        scale_factor: f32,
        theme_revision: u64,
        code_font_feature_policy_rev: u64,
        fg: Color,
    ) -> Self {
        Self {
            row_range,
            line,
            row_spans,
            syntax_spans,
            text_style: RowSceneTextStyleKey::from_style(text_style),
            constraints: RowSceneTextConstraintsKey::from_constraints(constraints),
            font_stack_key: font_stack_key.0,
            scale_bits: scale_factor.max(1.0).to_bits(),
            theme_revision,
            code_font_feature_policy_rev,
            fg: fg.into(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn matches_current(
        &self,
        row_range: &Range<usize>,
        line: &Arc<str>,
        row_spans: &Arc<[fret_code_editor_view::DisplayRowSpan]>,
        syntax_spans: &Arc<[SyntaxSpan]>,
        text_style: &TextStyle,
        constraints: CanvasTextConstraints,
        font_stack_key: fret_runtime::TextFontStackKey,
        scale_factor: f32,
        theme_revision: u64,
        code_font_feature_policy_rev: u64,
        fg: Color,
    ) -> bool {
        self.row_range == *row_range
            && (Arc::ptr_eq(&self.line, line) || self.line.as_ref() == line.as_ref())
            && (Arc::ptr_eq(&self.row_spans, row_spans)
                || self.row_spans.as_ref() == row_spans.as_ref())
            && (Arc::ptr_eq(&self.syntax_spans, syntax_spans)
                || self.syntax_spans.as_ref() == syntax_spans.as_ref())
            && self.text_style == RowSceneTextStyleKey::from_style(text_style)
            && self.constraints == RowSceneTextConstraintsKey::from_constraints(constraints)
            && self.font_stack_key == font_stack_key.0
            && self.scale_bits == scale_factor.max(1.0).to_bits()
            && self.theme_revision == theme_revision
            && self.code_font_feature_policy_rev == code_font_feature_policy_rev
            && self.fg == fg.into()
    }

    fn matches_cached_replay_context(
        &self,
        content: &RowContentSnapshot,
        text_style: &TextStyle,
        constraints: CanvasTextConstraints,
        font_stack_key: fret_runtime::TextFontStackKey,
        scale_factor: f32,
        theme_revision: u64,
        code_font_feature_policy_rev: u64,
        fg: Color,
    ) -> bool {
        self.row_range == content.range
            && (Arc::ptr_eq(&self.line, &content.text)
                || self.line.as_ref() == content.text.as_ref())
            && (Arc::ptr_eq(&self.row_spans, &content.row_spans)
                || self.row_spans.as_ref() == content.row_spans.as_ref())
            && self.text_style == RowSceneTextStyleKey::from_style(text_style)
            && self.constraints == RowSceneTextConstraintsKey::from_constraints(constraints)
            && self.font_stack_key == font_stack_key.0
            && self.scale_bits == scale_factor.max(1.0).to_bits()
            && self.theme_revision == theme_revision
            && self.code_font_feature_policy_rev == code_font_feature_policy_rev
            && self.fg == fg.into()
    }
}

pub struct CodeEditor {
    handle: CodeEditorHandle,
    overscan: usize,
    torture: Option<CodeEditorTorture>,
    soft_wrap_cols: Option<usize>,
    code_font_features: Option<CodeFontFeaturePolicy>,
    interaction: Option<CodeEditorInteractionOptions>,
    key: u64,
    a11y_label: Option<Arc<str>>,
    viewport_test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeEditorTorture {
    pub auto_scroll: bool,
    pub scroll_speed: Px,
    pub bounce: bool,
    pub show_overlay: bool,
}

impl CodeEditorTorture {
    pub fn auto_scroll_bounce(scroll_speed: Px) -> Self {
        Self {
            auto_scroll: true,
            scroll_speed,
            bounce: true,
            show_overlay: true,
        }
    }
}

impl CodeEditor {
    pub fn new(handle: CodeEditorHandle) -> Self {
        Self {
            handle,
            overscan: 16,
            torture: None,
            soft_wrap_cols: None,
            code_font_features: None,
            interaction: None,
            key: 0,
            a11y_label: None,
            viewport_test_id: None,
        }
    }

    /// Set a stable key for this editor instance.
    ///
    /// This is required when multiple `CodeEditor`s appear under the same element-id scope,
    /// because the editor uses an internal keyed scope for persistent state.
    pub fn key(mut self, key: u64) -> Self {
        self.key = key;
        self
    }

    pub fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan.max(1);
        self
    }

    pub fn soft_wrap_cols(mut self, cols: Option<usize>) -> Self {
        self.soft_wrap_cols = cols.filter(|v| *v > 0);
        self
    }

    pub fn code_font_features(mut self, policy: CodeFontFeaturePolicy) -> Self {
        self.code_font_features = Some(policy);
        self
    }

    pub fn torture(mut self, torture: CodeEditorTorture) -> Self {
        self.torture = Some(torture);
        self
    }

    pub fn interaction(mut self, interaction: CodeEditorInteractionOptions) -> Self {
        self.interaction = Some(interaction);
        self
    }

    pub fn viewport_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(test_id.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let scroll_handle = cx.slot_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
        let cell_w = cx.slot_state(|| Cell::new(Px(0.0)), |c| c.clone());
        let scroll_dir = cx.slot_state(|| Cell::new(1i32), |c| c.clone());

        let handle = self.handle.clone();
        let editor_state = self.handle.state.clone();
        let overscan = self.overscan;
        let torture = self.torture;
        let soft_wrap_cols = self.soft_wrap_cols;
        let code_font_features = self.code_font_features;
        let interaction = self.interaction;
        let key = self.key;
        let viewport_test_id = self.viewport_test_id;
        let a11y_label: Arc<str> = self.a11y_label.unwrap_or_else(|| Arc::from("Code editor"));

        cx.keyed(("code-editor", key), move |cx| {
            let active_interaction = interaction.unwrap_or_else(|| editor_state.borrow().interaction);
            let theme = cx.theme().clone();

            let row_h = theme.metric_token("metric.font.mono_line_height");
            let font_size = theme.metric_token("metric.font.mono_size");
            let fg = theme.color_token("foreground");
            let selection_bg = theme.color_token("selection.background");
            let caret_color = fg;
            let overlay_bg = theme.color_token("muted");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: font_size,
                ..Default::default()
            };

            let viewport_rows = if row_h.0 > 0.0 {
                (cx.bounds.size.height.0 / row_h.0).ceil() as usize
            } else {
                0
            };
            let text_cache_max_entries = viewport_rows
                .saturating_add(overscan.saturating_mul(2))
                .saturating_add(128)
                .clamp(
                    CODE_EDITOR_ROW_CACHE_MIN_ENTRIES,
                    CODE_EDITOR_ROW_CACHE_MAX_ENTRIES,
                );
            #[cfg(feature = "syntax")]
            let window = cx.window;

            #[cfg(feature = "syntax")]
            {
                if let Some(dispatcher) = cx.app.global::<DispatcherHandle>().cloned() {
                    let exec = dispatcher.exec_capabilities();
                    let supports_prefetch = exec.background_work != ExecBackgroundWork::None
                        && exec.wake != ExecWake::None;
                    let mut st = editor_state.borrow_mut();
                    if supports_prefetch {
                        if st.syntax_prefetch_runtime.is_none() {
                            st.syntax_prefetch_runtime =
                                Some(SyntaxPrefetchRuntime::new(dispatcher.clone()));
                        }
                        if st.row_rich_prefetch_runtime.is_none() {
                            st.row_rich_prefetch_runtime =
                                Some(RowRichPrefetchRuntime::new(dispatcher));
                        }
                    } else {
                        if let Some(runtime) = st.syntax_prefetch_runtime.take() {
                            runtime.clear();
                        }
                        if let Some(runtime) = st.row_rich_prefetch_runtime.take() {
                            runtime.clear();
                        }
                    }
                }
            }

            cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
            let font_stack_key = cx
                .app
                .global::<fret_runtime::TextFontStackKey>()
                .copied()
                .unwrap_or_default();

            let (
                content_len,
                boundary_mode,
                a11y_value,
                a11y_text_selection,
                a11y_text_composition,
                ime_cursor_area,
                ime_surrounding_text,
            ) = {
                handle.set_soft_wrap_cols(soft_wrap_cols);
                if let Some(policy) = code_font_features.clone() {
                    handle.set_code_font_feature_policy(policy);
                }
                if let Some(interaction) = interaction {
                    handle.set_interaction(interaction);
                }
                let mut st = editor_state.borrow_mut();
                st.update_font_stack_key(font_stack_key);
                let content_len = st.display_map.row_count();
                let inherited_mode = cx
                    .app
                    .global::<fret_runtime::WindowInputContextService>()
                    .and_then(|svc| svc.snapshot(cx.window))
                    .map(|snapshot| snapshot.text_boundary_mode)
                    .unwrap_or_default();
                let boundary_mode = st
                    .text_boundary_mode_override
                    .unwrap_or(inherited_mode);
                if st.active_text_boundary_mode != boundary_mode {
                    st.active_text_boundary_mode = boundary_mode;
                }
                let boundary_override = st.text_boundary_mode_override;
                let (value, selection, composition) =
                    a11y_composed_text_window(&mut st, text_cache_max_entries);
                let ime_surrounding_text = Some(st.ime_surrounding_text_best_effort_cached());

                let cell_w = cell_w.get();
                let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
                let ime_cursor_area =
                    ime_cursor_area_for_text_input_region(&mut st, row_h, cell_w, cx.bounds, &scroll_handle);
                (
                    content_len,
                    boundary_override,
                    Some(Arc::<str>::from(value)),
                    selection,
                    composition,
                    ime_cursor_area,
                    ime_surrounding_text,
                )
            };

            let mut region_layout = fret_ui::element::LayoutStyle::default();
            region_layout.size.width = Length::Fill;
            region_layout.size.height = Length::Fill;
            region_layout.overflow = Overflow::Clip;

            let region_props = TextInputRegionProps {
                layout: region_layout,
                enabled: active_interaction.enabled && active_interaction.focusable,
                text_boundary_mode_override: boundary_mode,
                ime_cursor_area,
                a11y_label: Some(Arc::clone(&a11y_label)),
                a11y_value,
                a11y_required: false,
                a11y_invalid: None,
                a11y_text_selection,
                a11y_text_composition,
                ime_surrounding_text,
            };

            let mut pointer_props = PointerRegionProps::default();
            pointer_props.layout.size.width = Length::Fill;
            pointer_props.layout.size.height = Length::Fill;
            pointer_props.enabled = active_interaction.enabled && active_interaction.selectable;

            let mut surface_props = WindowedRowsSurfaceProps::default();
            surface_props.scroll.layout.size.width = Length::Fill;
            surface_props.scroll.layout.size.height = Length::Fill;
            surface_props.scroll.layout.overflow = Overflow::Clip;
            surface_props.len = content_len;
            surface_props.row_height = row_h;
            surface_props.overscan = overscan;
            surface_props.scroll_handle = scroll_handle.clone();
            surface_props.canvas.cache_policy = CanvasCachePolicy {
                text: CanvasCacheTuning {
                    keep_frames: 60,
                    max_entries: text_cache_max_entries,
                },
                shared_text: CanvasCacheTuning::transient(),
                path: CanvasCacheTuning::transient(),
                svg: CanvasCacheTuning::transient(),
            };
            #[cfg(feature = "syntax")]
            let syntax_prefetch_hook = {
                let editor_state = editor_state.clone();
                let hook: OnWindowedRowsPrepaintFrame = Arc::new(move |_cx, frame| {
                    self::syntax::schedule_syntax_prefetch_for_frame(
                        &mut editor_state.borrow_mut(),
                        frame,
                        text_cache_max_entries,
                        window,
                    );
                });
                Some(hook)
            };
            #[cfg(not(feature = "syntax"))]
            let syntax_prefetch_hook: Option<OnWindowedRowsPrepaintFrame> = None;
            #[cfg(feature = "syntax")]
            let row_rich_prefetch_hook = {
                let editor_state = editor_state.clone();
                let hook: OnWindowedRowsPrepaintFrame = Arc::new(move |cx, frame| {
                    let theme = cx.theme().clone();
                    paint::schedule_row_rich_prefetch_for_frame(
                        &mut editor_state.borrow_mut(),
                        frame,
                        text_cache_max_entries,
                        window,
                        theme,
                    );
                });
                Some(hook)
            };
            #[cfg(not(feature = "syntax"))]
            let row_rich_prefetch_hook: Option<OnWindowedRowsPrepaintFrame> = None;
            let paint_frame_hook = {
                let editor_state = editor_state.clone();
                let hook: OnWindowedRowsPrepaintFrame = Arc::new(move |_cx, frame| {
                    editor_state.borrow_mut().begin_paint_frame(frame);
                });
                hook
            };
            let row_scene_replay_plan_hook = {
                let editor_state = editor_state.clone();
                let text_style = text_style.clone();
                let cell_w = cell_w.clone();
                let hook: OnWindowedRowsPrepaintFrame = Arc::new(move |cx, frame| {
                    let bounds = cx.bounds();
                    let theme_revision = cx.theme().revision();
                    let cell_w = cell_w.get();
                    let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
                    let mut st = editor_state.borrow_mut();
                    #[cfg(feature = "syntax")]
                    let plan =
                        paint::prepaint_row_scene_replay_plan_for_frame_with_edge_prebuild(
                            cx,
                            &mut st,
                            frame,
                            bounds,
                            cell_w,
                            text_cache_max_entries,
                            &text_style,
                            fg,
                            theme_revision,
                            cx.scale_factor(),
                        );
                    #[cfg(not(feature = "syntax"))]
                    let plan = paint::prepaint_row_scene_replay_plan_for_frame(
                        &mut st,
                        frame,
                        bounds,
                        cell_w,
                        text_cache_max_entries,
                        &text_style,
                        fg,
                        theme_revision,
                        cx.scale_factor(),
                    );
                    cx.set_scene_fragment_debug(plan);
                });
                hook
            };

            let torture_hook = torture.map(|torture| {
                let scroll_handle = scroll_handle.clone();
                let scroll_dir = scroll_dir.clone();
                let text_style = text_style.clone();
                let editor_state = editor_state.clone();
                let prev_stats = Rc::new(Cell::new(CodeEditorCacheStats::default()));
                let paint_perf_enabled = paint_perf_enabled_from_env();
                let hook: OnWindowedRowsPaintFrame = Arc::new(
                    move |painter: &mut fret_ui::canvas::CanvasPainter<'_>,
                          frame: WindowedRowsPaintFrame| {
                        if !torture.auto_scroll {
                            return;
                        }

                        let autoscroll_started = paint_perf_enabled.then(Instant::now);
                        let max = scroll_handle.max_offset();
                        if max.y.0 <= 0.0 {
                            if let Some(started) = autoscroll_started {
                                editor_state
                                    .borrow_mut()
                                    .record_torture_autoscroll_paint_elapsed(started);
                            }
                            return;
                        }

                        let offset = scroll_handle.offset();
                        let dir = scroll_dir.get();
                        let mut next_y = offset.y.0 + torture.scroll_speed.0 * dir as f32;
                        if torture.bounce && (next_y <= 0.0 || next_y >= max.y.0) {
                            scroll_dir.set(-dir);
                            next_y = next_y.clamp(0.0, max.y.0);
                        }

                        scroll_handle.set_offset(fret_core::Point::new(offset.x, Px(next_y)));
                        painter.request_animation_frame_paint_only();

                        if let Some(started) = autoscroll_started {
                            editor_state
                                .borrow_mut()
                                .record_torture_autoscroll_paint_elapsed(started);
                        }

                        if !torture.show_overlay {
                            return;
                        }

                        let overlay_started = paint_perf_enabled.then(Instant::now);
                        let (stats, delta, caret_row, caret_preferred_x, caret_stops, geom_cached) =
                            {
                                let st = editor_state.borrow();
                                let stats = st.cache_stats;
                                let caret = st.selection.caret().min(st.buffer.len_bytes());
                                let caret_row =
                                    st.display_map.byte_to_display_point(&st.buffer, caret).row;
                                let caret_preferred_x = st.caret_preferred_x;
                                let caret_stops =
                                    st.row_geom_cache.get(&caret_row).map(|(g, _)| g.caret_stops.len());
                                let geom_cached = st.row_geom_cache.len();

                                let prev = prev_stats.get();
                                prev_stats.set(stats);
                                let delta = CodeEditorCacheStats {
                                    row_text_get_calls: stats
                                        .row_text_get_calls
                                        .saturating_sub(prev.row_text_get_calls),
                                    row_text_hits: stats.row_text_hits.saturating_sub(prev.row_text_hits),
                                    row_text_misses: stats
                                        .row_text_misses
                                        .saturating_sub(prev.row_text_misses),
                                    row_text_evictions: stats
                                        .row_text_evictions
                                        .saturating_sub(prev.row_text_evictions),
                                    row_text_resets: stats
                                        .row_text_resets
                                        .saturating_sub(prev.row_text_resets),
                                    row_scene_get_calls: stats
                                        .row_scene_get_calls
                                        .saturating_sub(prev.row_scene_get_calls),
                                    row_scene_hits: stats
                                        .row_scene_hits
                                        .saturating_sub(prev.row_scene_hits),
                                    row_scene_misses: stats
                                        .row_scene_misses
                                        .saturating_sub(prev.row_scene_misses),
                                    row_scene_evictions: stats
                                        .row_scene_evictions
                                        .saturating_sub(prev.row_scene_evictions),
                                    row_scene_resets: stats
                                        .row_scene_resets
                                        .saturating_sub(prev.row_scene_resets),
                                    #[cfg(feature = "syntax")]
                                    row_scene_fast_get_calls: stats
                                        .row_scene_fast_get_calls
                                        .saturating_sub(prev.row_scene_fast_get_calls),
                                    #[cfg(feature = "syntax")]
                                    row_scene_fast_hits: stats
                                        .row_scene_fast_hits
                                        .saturating_sub(prev.row_scene_fast_hits),
                                    #[cfg(feature = "syntax")]
                                    row_scene_fast_misses: stats
                                        .row_scene_fast_misses
                                        .saturating_sub(prev.row_scene_fast_misses),

                                    #[cfg(feature = "syntax")]
                                    row_rich_get_calls: stats
                                        .row_rich_get_calls
                                        .saturating_sub(prev.row_rich_get_calls),
                                    #[cfg(feature = "syntax")]
                                    row_rich_hits: stats.row_rich_hits.saturating_sub(prev.row_rich_hits),
                                    #[cfg(feature = "syntax")]
                                    row_rich_misses: stats
                                        .row_rich_misses
                                        .saturating_sub(prev.row_rich_misses),
                                    #[cfg(feature = "syntax")]
                                    row_rich_evictions: stats
                                        .row_rich_evictions
                                        .saturating_sub(prev.row_rich_evictions),
                                    #[cfg(feature = "syntax")]
                                    row_rich_resets: stats
                                        .row_rich_resets
                                        .saturating_sub(prev.row_rich_resets),

                                    geom_pointer_hit_test_fallbacks: stats
                                        .geom_pointer_hit_test_fallbacks
                                        .saturating_sub(prev.geom_pointer_hit_test_fallbacks),
                                    geom_caret_rect_fallbacks: stats
                                        .geom_caret_rect_fallbacks
                                        .saturating_sub(prev.geom_caret_rect_fallbacks),
                                    geom_vertical_move_fallbacks: stats
                                        .geom_vertical_move_fallbacks
                                        .saturating_sub(prev.geom_vertical_move_fallbacks),
                                    syntax_get_calls: stats
                                        .syntax_get_calls
                                        .saturating_sub(prev.syntax_get_calls),
                                    syntax_hits: stats.syntax_hits.saturating_sub(prev.syntax_hits),
                                    syntax_misses: stats
                                        .syntax_misses
                                        .saturating_sub(prev.syntax_misses),
                                    syntax_evictions: stats
                                        .syntax_evictions
                                        .saturating_sub(prev.syntax_evictions),
                                    syntax_resets: stats.syntax_resets.saturating_sub(prev.syntax_resets),
                                };
                                (
                                    stats,
                                    delta,
                                    caret_row,
                                    caret_preferred_x,
                                    caret_stops,
                                    geom_cached,
                                )
                        };

                        let bounds = painter.bounds();
                        let origin = fret_core::Point::new(
                            Px(bounds.origin.x.0 + 8.0),
                            Px(bounds.origin.y.0 + offset.y.0 + 8.0),
                        );
                        painter.scene().push(SceneOp::Quad {
                            order: DrawOrder(100),
                            rect: Rect::new(origin, Size::new(Px(820.0), Px(24.0))),
                            background: fret_core::Paint::Solid(overlay_bg).into(),

                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),

                            corner_radii: Corners::all(Px(6.0)),
                        });

                        let label = format!(
                            "rows={}-{} y={:.0}/{:.0} max={} text {}/{}/{} (+{}/{}/{}) scene {}/{}/{} (+{}/{}/{}) syn {}/{}/{} (+{}/{}/{}) geom row={} pref_x={:?} stops={:?} cache={}",
                            frame.visible_start,
                            frame.visible_end,
                            offset.y.0,
                            max.y.0,
                            text_cache_max_entries,
                            stats.row_text_get_calls,
                            stats.row_text_hits,
                            stats.row_text_misses,
                            delta.row_text_get_calls,
                            delta.row_text_hits,
                            delta.row_text_misses,
                            stats.row_scene_get_calls,
                            stats.row_scene_hits,
                            stats.row_scene_misses,
                            delta.row_scene_get_calls,
                            delta.row_scene_hits,
                            delta.row_scene_misses,
                            stats.syntax_get_calls,
                            stats.syntax_hits,
                            stats.syntax_misses,
                            delta.syntax_get_calls,
                            delta.syntax_hits,
                            delta.syntax_misses,
                            caret_row,
                            caret_preferred_x.map(|v| v.0.round() as i32),
                            caret_stops,
                            geom_cached,
                        );
                        let key = painter.key(&("fret-code-editor-torture-overlay", 0u8));
                        let _ = painter.text(
                            key,
                            DrawOrder(101),
                            fret_core::Point::new(Px(origin.x.0 + 8.0), Px(origin.y.0 + 4.0)),
                            label,
                            text_style.clone(),
                            Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            },
                            CanvasTextConstraints {
                                max_width: Some(Px(800.0)),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            },
                            painter.scale_factor(),
                        );
                        if let Some(started) = overlay_started {
                            editor_state
                                .borrow_mut()
                                .record_torture_overlay_paint_elapsed(started);
                        }
                    },
                );
                hook
            });

            surface_props.on_prepaint_frame = {
                let mut hooks: Vec<OnWindowedRowsPrepaintFrame> = Vec::new();
                hooks.push(paint_frame_hook);
                if let Some(hook) = syntax_prefetch_hook {
                    hooks.push(hook);
                }
                if let Some(hook) = row_rich_prefetch_hook {
                    hooks.push(hook);
                }
                hooks.push(row_scene_replay_plan_hook);
                match hooks.len() {
                    0 => None,
                    1 => hooks.pop(),
                    _ => Some(Arc::new(move |cx, frame| {
                        for hook in &hooks {
                            hook(cx, frame);
                        }
                    })),
                }
            };
            surface_props.on_paint_frame = torture_hook;
            if paint_perf_enabled_from_env() {
                let editor_state = editor_state.clone();
                surface_props.on_paint_diagnostics = Some(Arc::new(move |diagnostics| {
                    editor_state
                        .borrow_mut()
                        .record_windowed_rows_paint_diagnostics(diagnostics);
                }));
            }

            cx.text_input_region(region_props, |cx| {
                // `TextInputRegion` creates its own element id scope. All focus/key/command hooks
                // must target this id (not the outer keyed scope), otherwise Web/WASM input routing
                // will never attach to the focused text region.
                let region_id = cx.root_id();
                editor_state.borrow_mut().region_id = Some(region_id);

                let platform_query_state = editor_state.clone();
                let platform_query_scroll = scroll_handle.clone();
                let platform_query_cell_w = cell_w.clone();
                cx.text_input_region_on_platform_text_input_query(std::sync::Arc::new(
                    move |_host,
                          _action_cx,
                          _services,
                          bounds,
                          _scale_factor,
                          props,
                          query| {
                        let Some(value) = props.a11y_value.as_deref() else {
                            return None;
                        };

                        match query {
                            fret_runtime::PlatformTextInputQuery::BoundsForRange { range } => {
                                let range = range.normalized();
                                let (_, end) = fret_core::utf::utf16_range_to_utf8_byte_range(
                                    value,
                                    usize::try_from(range.start).unwrap_or(usize::MAX),
                                    usize::try_from(range.end).unwrap_or(usize::MAX),
                                );
                                let end = u32::try_from(end.min(value.len())).unwrap_or(u32::MAX);

                                let mut st = platform_query_state.borrow_mut();
                                let byte = a11y::map_a11y_offset_to_buffer_in_current_window(
                                    &mut st,
                                    text_cache_max_entries,
                                    end,
                                );

                                let cell_w = platform_query_cell_w.get();
                                let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
                                let rect = geom::caret_rect_for_buffer_byte_boundary(
                                    &st,
                                    row_h,
                                    cell_w,
                                    bounds,
                                    &platform_query_scroll,
                                    byte,
                                );
                                Some(fret_runtime::PlatformTextInputQueryResult::Bounds(rect))
                            }
                            fret_runtime::PlatformTextInputQuery::CharacterIndexForPoint { point } => {
                                if row_h.0 <= 0.0 {
                                    return Some(fret_runtime::PlatformTextInputQueryResult::Index(
                                        None,
                                    ));
                                }

                                let mut st = platform_query_state.borrow_mut();
                                let offset = platform_query_scroll.offset();
                                let local_y =
                                    (point.y.0 - bounds.origin.y.0 + offset.y.0).max(0.0);
                                let row = (local_y / row_h.0).floor().max(0.0) as usize;
                                let row_count = st.display_map.row_count().max(1);
                                let row = row.min(row_count.saturating_sub(1));

                                let cell_w = platform_query_cell_w.get();
                                let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
                                let caret =
                                    geom::caret_for_pointer(&mut st, row, bounds, *point, cell_w);

                                let a11y_offset = a11y::map_buffer_offset_to_a11y_offset(
                                    &mut st,
                                    text_cache_max_entries,
                                    caret,
                                );
                                let idx = fret_core::utf::utf8_byte_offset_to_utf16_offset(
                                    value,
                                    usize::try_from(a11y_offset).unwrap_or(usize::MAX),
                                    fret_core::utf::UtfIndexClamp::Down,
                                );
                                let idx = u32::try_from(idx).unwrap_or(u32::MAX);
                                Some(fret_runtime::PlatformTextInputQueryResult::Index(Some(idx)))
                            }
                            _ => None,
                        }
                    },
                ));

                let platform_replace_state = editor_state.clone();
                let platform_replace_scroll = scroll_handle.clone();
                cx.text_input_region_on_platform_text_input_replace_text_in_range_utf16(
                    std::sync::Arc::new(
                        move |host,
                              action_cx,
                              _services,
                              _bounds,
                              _scale_factor,
                              props,
                              range,
                              text| {
                            let mut st = platform_replace_state.borrow_mut();
                            if !st.interaction.enabled || !st.interaction.editable {
                                st.set_preedit(None);
                                st.undo_group = None;
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                return false;
                            }

                            let Some(value) = props.a11y_value.as_deref() else {
                                return false;
                            };

                            let range = range.normalized();
                            let (start, end) = fret_core::utf::utf16_range_to_utf8_byte_range(
                                value,
                                usize::try_from(range.start).unwrap_or(usize::MAX),
                                usize::try_from(range.end).unwrap_or(usize::MAX),
                            );
                            let start =
                                u32::try_from(start.min(value.len())).unwrap_or(u32::MAX);
                            let end = u32::try_from(end.min(value.len())).unwrap_or(u32::MAX);

                            let start = a11y::map_a11y_offset_to_buffer_in_current_window(
                                &mut st,
                                text_cache_max_entries,
                                start,
                            );
                            let end = a11y::map_a11y_offset_to_buffer_in_current_window(
                                &mut st,
                                text_cache_max_entries,
                                end,
                            );

                            st.set_preedit(None);
                            st.selection = Selection {
                                anchor: start,
                                focus: end,
                            };

                            let did =
                                input::insert_text_with_kind(&mut st, text, UndoGroupKind::Typing)
                                    .is_some();
                            if did {
                                input::scroll_caret_into_view(&st, row_h, &platform_replace_scroll);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            }
                            did
                        },
                    ),
                );

                let platform_mark_state = editor_state.clone();
                let platform_mark_scroll = scroll_handle.clone();
                cx.text_input_region_on_platform_text_input_replace_and_mark_text_in_range_utf16(
                    std::sync::Arc::new(
                        move |host,
                              action_cx,
                              _services,
                              _bounds,
                              _scale_factor,
                              props,
                              range,
                              text,
                              marked,
                              selected| {
                            let mut st = platform_mark_state.borrow_mut();
                            let Some(value) = props.a11y_value.as_deref() else {
                                return false;
                            };

                            let did = platform_replace_and_mark_text_in_range_utf16(
                                &mut st,
                                text_cache_max_entries,
                                value,
                                range,
                                text,
                                marked,
                                selected,
                            );

                            if did {
                                input::scroll_caret_into_view(&st, row_h, &platform_mark_scroll);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            }
                            did
                        },
                    ),
                );

                let key_state = editor_state.clone();
                let key_scroll = scroll_handle.clone();
                let key_cell_w = cell_w.clone();
                cx.key_on_key_down_for(
                    region_id,
                    Arc::new(
                        move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                              action_cx: ActionCx,
                              down: KeyDownCx| {
                            input::handle_key_down(
                                host,
                                action_cx,
                                &key_state,
                                row_h,
                                &key_scroll,
                                &key_cell_w,
                                down.key,
                                down.modifiers,
                            )
                        },
                    ),
                );

                let cmd_state = editor_state.clone();
                let cmd_scroll = scroll_handle.clone();
                cx.command_on_command_for(
                    region_id,
                    Arc::new(
                        move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                              action_cx: ActionCx,
                              command| {
                            let mut st = cmd_state.borrow_mut();
                            let result =
                                input::handle_command(host, action_cx, &mut st, command.as_str());
                            if !result.handled {
                                return false;
                            }

                            if result.did {
                                input::scroll_caret_into_view(&st, row_h, &cmd_scroll);
                                // IME cursor positioning is driven by `TextInputRegionProps.ime_cursor_area`
                                // and the per-frame `WindowTextInputSnapshot` published by the UI tree.
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            }
                            true
                        },
                    ),
                );

                let avail_state = editor_state.clone();
                cx.command_on_command_availability_for(
                    region_id,
                    Arc::new(move |_host, acx, command| {
                        if !acx.focus_in_subtree {
                            return fret_ui::CommandAvailability::NotHandled;
                        }

                        let st = avail_state.borrow();
                        input::command_availability(&st, &acx.input_ctx, command.as_str())
                    }),
                );

                let on_pointer_down_state = editor_state.clone();
                let on_pointer_down_cell_w = cell_w.clone();
                let on_pointer_down_scroll = scroll_handle.clone();
                let on_pointer_down: OnWindowedRowsPointerDown = Arc::new(
                    move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, row, down| {
                        let mut st = on_pointer_down_state.borrow_mut();
                        if !st.interaction.enabled || !st.interaction.selectable {
                            return false;
                        }
                        if down.button != MouseButton::Left {
                            return false;
                        }

                        if down.pointer_type != PointerType::Touch {
                            host.set_cursor_icon(CursorIcon::Text);
                        }
                        if st.interaction.focusable {
                            host.request_focus(region_id);
                        }

                        let bounds = host.bounds();
                        let cell_w = on_pointer_down_cell_w.get();
                        let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };

                        st.last_bounds = Some(bounds);
                        st.undo_group = None;

                        let caret = caret_for_pointer(&mut st, row, bounds, down.position, cell_w);
                        input::apply_pointer_down_selection(
                            &mut st,
                            row,
                            caret,
                            down.click_count,
                            down.modifiers.shift,
                        );

                        if down.pointer_type == PointerType::Touch {
                            st.dragging = false;
                            st.drag_pointer = None;
                            st.drag_autoscroll_viewport_pos = None;
                        } else {
                            host.capture_pointer();
                            st.dragging = true;
                            st.drag_pointer = Some(down.pointer_id);
                            st.drag_autoscroll_viewport_pos = Some(viewport_pos_for_pointer(
                                bounds,
                                &on_pointer_down_scroll,
                                down.position,
                            ));
                        }

                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    },
                );

                let on_pointer_move_state = editor_state.clone();
                let on_pointer_move_cell_w = cell_w.clone();
                let on_pointer_move_scroll = scroll_handle.clone();
                let on_pointer_move: OnWindowedRowsPointerMove = Arc::new(
                    move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, _row, mv| {
                        // Show an I-beam cursor while hovering the editor surface, even when not dragging.
                        if mv.pointer_type != PointerType::Touch {
                            host.set_cursor_icon(CursorIcon::Text);
                        }
                        if mv.pointer_type == PointerType::Touch {
                            return false;
                        }
                        if !mv.buttons.left {
                            return false;
                        }
                        let mut st = on_pointer_move_state.borrow_mut();
                        if !st.interaction.enabled || !st.interaction.selectable {
                            return false;
                        }
                        if !st.dragging {
                            return false;
                        }
                        st.undo_group = None;

                        let bounds = host.bounds();
                        st.last_bounds = Some(bounds);

                        let mut changed = false;
                        let viewport_pos =
                            viewport_pos_for_pointer(bounds, &on_pointer_move_scroll, mv.position);
                        st.drag_autoscroll_viewport_pos = Some(viewport_pos);

                        let viewport_h = Px(on_pointer_move_scroll.viewport_size().height.0.max(0.0));
                        let scroll_delta_y = drag_autoscroll_delta_y(viewport_h, row_h, viewport_pos.y);
                        if scroll_delta_y.0 != 0.0 {
                            let offset = on_pointer_move_scroll.offset();
                            on_pointer_move_scroll.set_offset(fret_core::Point::new(
                                offset.x,
                                Px(offset.y.0 + scroll_delta_y.0),
                            ));
                            changed = true;

                            if st.drag_autoscroll_timer.is_none() {
                                let token = host.next_timer_token();
                                st.drag_autoscroll_timer = Some(token);
                                host.push_effect(Effect::SetTimer {
                                    window: Some(action_cx.window),
                                    token,
                                    after: DRAG_AUTOSCROLL_TICK,
                                    repeat: Some(DRAG_AUTOSCROLL_TICK),
                                });
                            }
                        } else if let Some(token) = st.drag_autoscroll_timer.take() {
                            st.drag_autoscroll_viewport_pos = None;
                            host.push_effect(Effect::CancelTimer { token });
                        }

                        let cell_w = on_pointer_move_cell_w.get();
                        let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };

                        let viewport_y = if viewport_h.0 > 0.0 {
                            Px(viewport_pos.y.0.clamp(0.0, viewport_h.0))
                        } else {
                            Px(0.0)
                        };
                        let offset = on_pointer_move_scroll.offset();
                        let content_y = offset.y.0 + viewport_y.0;
                        let mut row = if row_h.0 > 0.0 {
                            (content_y / row_h.0).floor().max(0.0) as usize
                        } else {
                            0
                        };
                        row = row.min(st.display_map.row_count().saturating_sub(1));

                        let caret_pos = fret_core::Point::new(
                            Px(bounds.origin.x.0 + viewport_pos.x.0),
                            Px(bounds.origin.y.0 + viewport_y.0),
                        );
                        let caret = caret_for_pointer(&mut st, row, bounds, caret_pos, cell_w);
                        if caret != st.selection.focus {
                            st.selection.focus = caret;
                            st.caret_preferred_x = None;
                            changed = true;
                        }

                        if changed {
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                        }

                        true
                    },
                );

                let on_pointer_up_state = editor_state.clone();
                let on_pointer_up: OnWindowedRowsPointerUp = Arc::new(
                    move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, _row, up| {
                        if up.button != MouseButton::Left {
                            return false;
                        }
                        let mut st = on_pointer_up_state.borrow_mut();
                        if !st.interaction.enabled || !st.interaction.selectable {
                            return false;
                        }
                        st.dragging = false;
                        st.drag_pointer = None;
                        st.undo_group = None;
                        st.drag_autoscroll_viewport_pos = None;
                        if let Some(token) = st.drag_autoscroll_timer.take() {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        host.release_pointer_capture();
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        false
                    },
                );

                let on_pointer_cancel_state = editor_state.clone();
                let on_pointer_cancel: OnWindowedRowsPointerCancel = Arc::new(
                    move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, cancel| {
                        let mut st = on_pointer_cancel_state.borrow_mut();
                        if !st.interaction.enabled || !st.interaction.selectable {
                            return false;
                        }
                        if st.drag_pointer == Some(cancel.pointer_id) {
                            st.dragging = false;
                            st.drag_pointer = None;
                        }
                        st.undo_group = None;
                        st.drag_autoscroll_viewport_pos = None;
                        if let Some(token) = st.drag_autoscroll_timer.take() {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        host.release_pointer_capture();
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        false
                    },
                );

                let on_timer_state = editor_state.clone();
                let on_timer_scroll = scroll_handle.clone();
                let on_timer_cell_w = cell_w.clone();
                let on_timer: OnTimer = Arc::new(move |host, action_cx, token| {
                    let mut st = on_timer_state.borrow_mut();
                    if st.drag_autoscroll_timer != Some(token) {
                        return false;
                    }

                    if !st.dragging {
                        st.drag_autoscroll_timer = None;
                        st.drag_autoscroll_viewport_pos = None;
                        host.push_effect(Effect::CancelTimer { token });
                        return true;
                    }

                    let Some(bounds) = st.last_bounds else {
                        st.drag_autoscroll_timer = None;
                        st.drag_autoscroll_viewport_pos = None;
                        host.push_effect(Effect::CancelTimer { token });
                        return true;
                    };

                    let Some(viewport_pos) = st.drag_autoscroll_viewport_pos else {
                        st.drag_autoscroll_timer = None;
                        host.push_effect(Effect::CancelTimer { token });
                        return true;
                    };

                    let viewport_h = Px(on_timer_scroll.viewport_size().height.0.max(0.0));
                    let scroll_delta_y = drag_autoscroll_delta_y(viewport_h, row_h, viewport_pos.y);
                    if scroll_delta_y.0 == 0.0 {
                        st.drag_autoscroll_timer = None;
                        st.drag_autoscroll_viewport_pos = None;
                        host.push_effect(Effect::CancelTimer { token });
                        return true;
                    }

                    let offset = on_timer_scroll.offset();
                    on_timer_scroll.set_offset(fret_core::Point::new(
                        offset.x,
                        Px(offset.y.0 + scroll_delta_y.0),
                    ));

                    let viewport_y = if viewport_h.0 > 0.0 {
                        Px(viewport_pos.y.0.clamp(0.0, viewport_h.0))
                    } else {
                        Px(0.0)
                    };
                    let offset = on_timer_scroll.offset();
                    let content_y = offset.y.0 + viewport_y.0;
                    let mut row = if row_h.0 > 0.0 {
                        (content_y / row_h.0).floor().max(0.0) as usize
                    } else {
                        0
                    };
                    row = row.min(st.display_map.row_count().saturating_sub(1));

                    let cell_w = on_timer_cell_w.get();
                    let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };

                    let caret_pos = fret_core::Point::new(
                        Px(bounds.origin.x.0 + viewport_pos.x.0),
                        Px(bounds.origin.y.0 + viewport_y.0),
                    );
                    let caret = caret_for_pointer(&mut st, row, bounds, caret_pos, cell_w);
                    if caret != st.selection.focus {
                        st.selection.focus = caret;
                        st.caret_preferred_x = None;
                    }

                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                    true
                });

                let handlers = WindowedRowsSurfacePointerHandlers {
                    on_pointer_down: Some(on_pointer_down),
                    on_pointer_move: Some(on_pointer_move),
                    on_pointer_up: Some(on_pointer_up),
                    on_pointer_cancel: Some(on_pointer_cancel),
                    on_timer: Some(on_timer),
                };

                let text_state = editor_state.clone();
                let text_scroll = scroll_handle.clone();
                cx.text_input_region_on_text_input(Arc::new(
                    move |host: &mut dyn UiActionHost, action_cx: ActionCx, text: &str| {
                        let mut st = text_state.borrow_mut();
                        if !st.interaction.enabled || !st.interaction.editable {
                            st.set_preedit(None);
                            st.undo_group = None;
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            return true;
                        }
                        st.set_preedit(None);
                        if input::insert_text(&mut st, text).is_some() {
                            input::scroll_caret_into_view(&st, row_h, &text_scroll);
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            return true;
                        }
                        false
                    },
                ));

                let ime_state = editor_state.clone();
                let ime_scroll = scroll_handle.clone();
                cx.text_input_region_on_ime(Arc::new(
                    move |host: &mut dyn UiActionHost,
                          action_cx: ActionCx,
                          ime: &fret_core::ImeEvent| {
                        let mut st = ime_state.borrow_mut();
                        if !st.interaction.enabled || !st.interaction.editable {
                            match ime {
                                fret_core::ImeEvent::Enabled => return false,
                                fret_core::ImeEvent::Disabled => {
                                    st.set_preedit(None);
                                }
                                _ => {
                                    st.set_preedit(None);
                                    st.undo_group = None;
                                }
                            }
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            return true;
                        }
                        match ime {
                            fret_core::ImeEvent::Enabled => return false,
                            fret_core::ImeEvent::Disabled => {
                                st.set_preedit(None);
                            }
                            fret_core::ImeEvent::Commit(text) => {
                                let _ = input::insert_text_with_kind(
                                    &mut st,
                                    text.as_str(),
                                    UndoGroupKind::Typing,
                                );
                                st.set_preedit(None);
                            }
                            fret_core::ImeEvent::Preedit { text, cursor } => {
                                st.preedit_replace_range = None;
                                let preedit = (!text.is_empty()).then_some(PreeditState {
                                    text: text.clone(),
                                    cursor: *cursor,
                                });
                                st.set_preedit(preedit);
                            }
                            fret_core::ImeEvent::DeleteSurrounding {
                                before_bytes,
                                after_bytes,
                            } => {
                                let _ = input::apply_ime_delete_surrounding(
                                    &mut st,
                                    *before_bytes,
                                    *after_bytes,
                                );
                            }
                        }

                        input::scroll_caret_into_view(&st, row_h, &ime_scroll);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    },
                ));

                let sel_state = editor_state.clone();
                let sel_scroll = scroll_handle.clone();
                cx.text_input_region_on_set_selection(Arc::new(
                    move |host: &mut dyn UiActionHost, action_cx: ActionCx, anchor, focus| {
                        let mut st = sel_state.borrow_mut();

                        let (new_anchor, new_focus) = if st.compose_inline_preedit {
                            map_a11y_offsets_to_buffer_composed(
                                &mut st,
                                text_cache_max_entries,
                                anchor,
                                focus,
                            )
                        } else {
                            (
                                a11y::map_a11y_offset_to_buffer_in_current_window(
                                    &mut st,
                                    text_cache_max_entries,
                                    anchor,
                                ),
                                a11y::map_a11y_offset_to_buffer_in_current_window(
                                    &mut st,
                                    text_cache_max_entries,
                                    focus,
                                ),
                            )
                        };

                        st.set_preedit(None);

                        st.selection = Selection {
                            anchor: new_anchor,
                            focus: new_focus,
                        };
                        st.undo_group = None;

                        input::scroll_caret_into_view(&st, row_h, &sel_scroll);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    },
                ));

                let clipboard_state = editor_state.clone();
                let clipboard_scroll = scroll_handle.clone();
                cx.text_input_region_on_clipboard_read_text(Arc::new(
                    move |host: &mut dyn UiActionHost,
                          action_cx: ActionCx,
                          _token: ClipboardToken,
                          text: &str| {
                        let mut st = clipboard_state.borrow_mut();
                        if !st.interaction.enabled || !st.interaction.editable {
                            st.set_preedit(None);
                            st.undo_group = None;
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            return true;
                        }
                        let _ = input::insert_text_with_kind(&mut st, text, UndoGroupKind::Paste);
                        input::scroll_caret_into_view(&st, row_h, &clipboard_scroll);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    },
                ));

                let surface = windowed_rows_surface_with_pointer_region(
                    cx,
                    surface_props,
                    pointer_props,
                    handlers,
                    None,
                    move |painter, row, rect| {
                        if cell_w.get().0 <= 0.0 {
                            let scope = painter.key_scope(&"fret-code-editor-cell-width");
                            let key: u64 = painter.child_key(scope, &0u8).into();
                            let metrics = painter.text(
                                key,
                                DrawOrder(0),
                                fret_core::Point::new(Px(-10_000.0), Px(-10_000.0)),
                                "M",
                                text_style.clone(),
                                Color::TRANSPARENT,
                                CanvasTextConstraints {
                                    max_width: None,
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                },
                                painter.scale_factor(),
                            );
                            let w = Px(metrics.size.width.0.max(1.0));
                            cell_w.set(w);
                        }

                        let mut st = editor_state.borrow_mut();
                        paint::paint_row(
                            painter,
                            &mut st,
                            row,
                            rect,
                            row_h,
                            cell_w.get(),
                            text_cache_max_entries,
                            &text_style,
                            fg,
                            selection_bg,
                            caret_color,
                        );
                    },
                );

                if let Some(test_id) = viewport_test_id.clone() {
                    let surface = surface.attach_semantics(
                        SemanticsDecoration::default()
                            .label("Editor viewport")
                            .test_id(test_id),
                    );
                    vec![surface]
                } else {
                    vec![surface]
                }
            })
        })
    }
}

fn ime_cursor_area_for_text_input_region(
    st: &mut CodeEditorState,
    row_h: Px,
    cell_w: Px,
    bounds: Rect,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) -> Option<fret_core::Rect> {
    geom::caret_rect_for_selection(st, row_h, cell_w, bounds, scroll_handle)
}

fn best_effort_ime_surrounding_text(
    buffer: &TextBuffer,
    selection: Selection,
) -> fret_runtime::WindowImeSurroundingText {
    fn clamp_down_to_char_boundary(buffer: &TextBuffer, idx: usize) -> usize {
        let mut idx = idx.min(buffer.len_bytes());
        while idx > 0 && !buffer.is_char_boundary(idx) {
            idx = idx.saturating_sub(1);
        }
        idx
    }

    let max = fret_runtime::WindowImeSurroundingText::MAX_TEXT_BYTES;
    let len = buffer.len_bytes();

    let cursor = clamp_down_to_char_boundary(buffer, selection.focus);
    let mut anchor = clamp_down_to_char_boundary(buffer, selection.anchor);

    let mut low = cursor.min(anchor);
    let mut high = cursor.max(anchor);
    if high.saturating_sub(low) > max {
        anchor = cursor;
        low = cursor;
        high = cursor;
    }

    let (mut start, mut end) = if len <= max {
        (0, len)
    } else {
        let needed = high.saturating_sub(low);
        let slack = max.saturating_sub(needed);
        let before = slack / 2;
        let start = low.saturating_sub(before).min(len.saturating_sub(max));
        let end = (start + max).min(len);
        (start, end)
    };

    start = clamp_down_to_char_boundary(buffer, start);
    end = clamp_down_to_char_boundary(buffer, end);
    if end < start {
        end = start;
    }

    let text = buffer.slice_to_string(start..end).unwrap_or_default();
    let cursor_rel = cursor.saturating_sub(start).min(text.len());
    let anchor_rel = anchor.saturating_sub(start).min(text.len());

    fret_runtime::WindowImeSurroundingText {
        text: Arc::<str>::from(text),
        cursor: u32::try_from(cursor_rel).unwrap_or(u32::MAX),
        anchor: u32::try_from(anchor_rel).unwrap_or(u32::MAX),
    }
}
