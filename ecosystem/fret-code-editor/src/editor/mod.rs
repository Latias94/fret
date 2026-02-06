//! Implementation details for the Fret code editor surface.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use fret_code_editor_buffer::{DocId, Edit, TextBuffer, TextBufferTransaction, TextBufferTx};
use fret_code_editor_view::{
    DisplayMap, DisplayPoint, move_word_left_in_buffer, move_word_right_in_buffer,
    select_word_range_in_buffer,
};
use fret_core::{
    AttributedText, CaretAffinity, Color, Corners, CursorIcon, DecorationLineStyle, DrawOrder,
    Edges, FontId, KeyCode, Modifiers, MouseButton, Px, Rect, SceneOp, Size, TextOverflow,
    TextPaintStyle, TextSpan, TextStyle, TextWrap, UnderlineStyle,
};
use fret_runtime::{ClipboardToken, Effect, TextBoundaryMode};
use fret_ui::action::{ActionCx, KeyDownCx, UiActionHost, UiPointerActionHost};
use fret_ui::canvas::CanvasTextConstraints;
use fret_ui::element::AnyElement;
use fret_ui::element::{
    CanvasCachePolicy, CanvasCacheTuning, Length, Overflow, PointerRegionProps, SemanticsProps,
    TextInputRegionProps,
};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::windowed_rows_surface::{
    OnWindowedRowsPaintFrame, OnWindowedRowsPointerCancel, OnWindowedRowsPointerDown,
    OnWindowedRowsPointerMove, OnWindowedRowsPointerUp, WindowedRowsPaintFrame,
    WindowedRowsSurfacePointerHandlers, WindowedRowsSurfaceProps,
    windowed_rows_surface_with_pointer_region,
};
use fret_undo::{CoalesceKey, InvertibleTransaction, UndoHistory, UndoRecord};

mod a11y;
mod geom;
mod input;
mod paint;
#[cfg(test)]
mod tests;

use a11y::{a11y_composed_text_window, a11y_text_window_bounds, map_a11y_offset_to_buffer};
use geom::{
    RowGeom, RowPreeditMapping, caret_for_pointer, caret_rect_for_selection,
    caret_x_for_buffer_byte_in_row, caret_x_for_index, hit_test_index_from_caret_stops,
    map_row_local_to_buffer_byte, preedit_cursor_offset_bytes, preedit_cursor_offset_cols,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    pub anchor: usize,
    pub focus: usize,
}

impl Selection {
    pub fn is_caret(self) -> bool {
        self.anchor == self.focus
    }

    pub fn normalized(self) -> Range<usize> {
        if self.anchor <= self.focus {
            self.anchor..self.focus
        } else {
            self.focus..self.anchor
        }
    }

    pub fn caret(self) -> usize {
        self.focus
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreeditState {
    pub text: String,
    pub cursor: Option<(usize, usize)>,
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
#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntaxSpan {
    /// Range within the row text (UTF-8 byte indices).
    range: Range<usize>,
    highlight: &'static str,
}

/// Lightweight counters for editor-local caches (bundle-friendly, no allocations).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CodeEditorCacheStats {
    pub row_text_get_calls: u64,
    pub row_text_hits: u64,
    pub row_text_misses: u64,
    pub row_text_evictions: u64,
    pub row_text_resets: u64,

    pub syntax_get_calls: u64,
    pub syntax_hits: u64,
    pub syntax_misses: u64,
    pub syntax_evictions: u64,
    pub syntax_resets: u64,
}

#[derive(Debug, Clone)]
struct CodeEditorState {
    buffer: TextBuffer,
    selection: Selection,
    preedit: Option<PreeditState>,
    text_boundary_mode: TextBoundaryMode,
    display_wrap_cols: Option<usize>,
    display_map: DisplayMap,
    caret_preferred_x: Option<Px>,
    undo: UndoHistory<CodeEditorTx>,
    undo_group: Option<UndoGroup>,
    dragging: bool,
    drag_pointer: Option<fret_core::PointerId>,
    last_bounds: Option<Rect>,
    cache_stats: CodeEditorCacheStats,
    row_text_cache_rev: fret_code_editor_buffer::Revision,
    row_text_cache_wrap_cols: Option<usize>,
    row_text_cache_tick: u64,
    row_text_cache: HashMap<usize, (RowTextCacheEntry, u64)>,
    row_text_cache_queue: VecDeque<(usize, u64)>,
    row_geom_cache_rev: fret_code_editor_buffer::Revision,
    row_geom_cache_wrap_cols: Option<usize>,
    row_geom_cache_tick: u64,
    row_geom_cache: HashMap<usize, (RowGeom, u64)>,
    row_geom_cache_queue: VecDeque<(usize, u64)>,
    selection_rect_scratch: Vec<Rect>,
    baseline_measure_cache: Option<BaselineMeasureCache>,
    #[cfg(feature = "syntax")]
    language: Option<Arc<str>>,
    #[cfg(feature = "syntax")]
    syntax_row_cache_rev: fret_code_editor_buffer::Revision,
    #[cfg(feature = "syntax")]
    syntax_row_cache_tick: u64,
    #[cfg(feature = "syntax")]
    syntax_row_cache_language: Option<Arc<str>>,
    #[cfg(feature = "syntax")]
    syntax_row_cache: HashMap<usize, (Arc<[SyntaxSpan]>, u64)>,
    #[cfg(feature = "syntax")]
    syntax_row_cache_queue: VecDeque<(usize, u64)>,
}

#[derive(Debug, Clone)]
struct RowTextCacheEntry {
    text: Arc<str>,
    range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
struct BaselineMeasureCache {
    max_width: Px,
    row_h: Px,
    scale_bits: u32,
    text_style: TextStyle,
    metrics: fret_core::TextMetrics,
    measured_h: Px,
}

impl CodeEditorState {
    fn refresh_display_map(&mut self) {
        self.display_map = DisplayMap::new(&self.buffer, self.display_wrap_cols);
    }
}

#[derive(Clone)]
pub struct CodeEditorHandle {
    state: Rc<RefCell<CodeEditorState>>,
}

impl CodeEditorHandle {
    pub fn new(text: impl Into<String>) -> Self {
        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, text.into()).unwrap_or_else(|_| {
            TextBuffer::new(doc, String::new()).expect("empty buffer must be valid")
        });
        let display_map = DisplayMap::new(&buffer, None);
        Self {
            state: Rc::new(RefCell::new(CodeEditorState {
                buffer,
                selection: Selection::default(),
                preedit: None,
                text_boundary_mode: TextBoundaryMode::Identifier,
                display_wrap_cols: None,
                display_map,
                caret_preferred_x: None,
                undo: UndoHistory::with_limit(512),
                undo_group: None,
                dragging: false,
                drag_pointer: None,
                last_bounds: None,
                cache_stats: CodeEditorCacheStats::default(),
                row_text_cache_rev: fret_code_editor_buffer::Revision(0),
                row_text_cache_wrap_cols: None,
                row_text_cache_tick: 0,
                row_text_cache: HashMap::new(),
                row_text_cache_queue: VecDeque::new(),
                row_geom_cache_rev: fret_code_editor_buffer::Revision(0),
                row_geom_cache_wrap_cols: None,
                row_geom_cache_tick: 0,
                row_geom_cache: HashMap::new(),
                row_geom_cache_queue: VecDeque::new(),
                selection_rect_scratch: Vec::new(),
                baseline_measure_cache: None,
                #[cfg(feature = "syntax")]
                language: None,
                #[cfg(feature = "syntax")]
                syntax_row_cache_rev: fret_code_editor_buffer::Revision(0),
                #[cfg(feature = "syntax")]
                syntax_row_cache_tick: 0,
                #[cfg(feature = "syntax")]
                syntax_row_cache_language: None,
                #[cfg(feature = "syntax")]
                syntax_row_cache: HashMap::new(),
                #[cfg(feature = "syntax")]
                syntax_row_cache_queue: VecDeque::new(),
            })),
        }
    }

    pub fn set_language(&self, language: Option<impl Into<Arc<str>>>) {
        #[cfg(feature = "syntax")]
        {
            let mut st = self.state.borrow_mut();
            st.language = language.map(Into::into);
            st.cache_stats.syntax_resets = st.cache_stats.syntax_resets.saturating_add(1);
            st.syntax_row_cache_language = None;
            st.syntax_row_cache_tick = 0;
            st.syntax_row_cache.clear();
            st.syntax_row_cache_queue.clear();
        }
        #[cfg(not(feature = "syntax"))]
        {
            let _ = language;
        }
    }

    pub fn selection(&self) -> Selection {
        self.state.borrow().selection
    }

    pub fn set_selection(&self, selection: Selection) {
        let mut st = self.state.borrow_mut();
        let max = st.buffer.len_bytes();
        let anchor = selection.anchor.min(max);
        let focus = selection.focus.min(max);
        st.selection = Selection { anchor, focus };
        st.preedit = None;
        st.caret_preferred_x = None;
        st.undo_group = None;
        st.dragging = false;
        st.drag_pointer = None;
    }

    pub fn set_caret(&self, caret: usize) {
        let caret = caret.min(self.state.borrow().buffer.len_bytes());
        self.set_selection(Selection {
            anchor: caret,
            focus: caret,
        });
    }

    pub fn text_boundary_mode(&self) -> TextBoundaryMode {
        self.state.borrow().text_boundary_mode
    }

    pub fn cache_stats(&self) -> CodeEditorCacheStats {
        self.state.borrow().cache_stats
    }

    pub fn reset_cache_stats(&self) {
        self.state.borrow_mut().cache_stats = CodeEditorCacheStats::default();
    }

    pub fn set_text_boundary_mode(&self, mode: TextBoundaryMode) {
        let mut st = self.state.borrow_mut();
        if st.text_boundary_mode == mode {
            return;
        }
        st.text_boundary_mode = mode;
        st.undo_group = None;
    }

    pub fn replace_buffer(&self, buffer: TextBuffer) {
        let mut st = self.state.borrow_mut();
        st.buffer = buffer;
        st.selection = Selection::default();
        st.preedit = None;
        st.caret_preferred_x = None;
        st.undo = UndoHistory::with_limit(512);
        st.undo_group = None;
        st.dragging = false;
        st.drag_pointer = None;
        st.last_bounds = None;
        st.cache_stats = CodeEditorCacheStats::default();
        st.refresh_display_map();
        st.row_text_cache_rev = st.buffer.revision();
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
        #[cfg(feature = "syntax")]
        {
            st.syntax_row_cache_rev = st.buffer.revision();
            st.syntax_row_cache_language = st.language.clone();
            st.syntax_row_cache_tick = 0;
            st.syntax_row_cache.clear();
            st.syntax_row_cache_queue.clear();
        }
    }

    pub fn set_text(&self, text: impl Into<String>) {
        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, text.into()).unwrap_or_else(|_| {
            TextBuffer::new(doc, String::new()).expect("empty buffer must be valid")
        });
        self.replace_buffer(buffer);
    }

    pub fn with_buffer<R>(&self, f: impl FnOnce(&TextBuffer) -> R) -> R {
        let st = self.state.borrow();
        f(&st.buffer)
    }

    /// v1 soft-wrap seam.
    ///
    /// This controls the view-layer `DisplayMap` and therefore affects:
    /// - rendered row splitting (logical lines -> display rows),
    /// - caret/selection geometry (byte ↔ display point).
    pub fn set_soft_wrap_cols(&self, cols: Option<usize>) {
        let mut st = self.state.borrow_mut();
        let cols = cols.filter(|v| *v > 0);
        if st.display_wrap_cols == cols {
            return;
        }
        st.display_wrap_cols = cols;
        st.refresh_display_map();
        st.caret_preferred_x = None;
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
    }
}

pub struct CodeEditor {
    handle: CodeEditorHandle,
    overscan: usize,
    torture: Option<CodeEditorTorture>,
    soft_wrap_cols: Option<usize>,
    key: u64,
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
            key: 0,
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

    pub fn torture(mut self, torture: CodeEditorTorture) -> Self {
        self.torture = Some(torture);
        self
    }

    pub fn viewport_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
        let cell_w = cx.with_state(|| Cell::new(Px(0.0)), |c| c.clone());
        let scroll_dir = cx.with_state(|| Cell::new(1i32), |c| c.clone());

        let editor_state = self.handle.state.clone();
        let overscan = self.overscan;
        let torture = self.torture;
        let soft_wrap_cols = self.soft_wrap_cols;
        let key = self.key;
        let viewport_test_id = self.viewport_test_id;
        let a11y_label: Arc<str> = Arc::from("Code editor");

        cx.keyed(("code-editor", key), move |cx| {
            let theme = cx.theme().clone();

            let row_h = theme.metric_required("metric.font.mono_line_height");
            let font_size = theme.metric_required("metric.font.mono_size");
            let fg = theme.color_required("foreground");
            let selection_bg = theme.color_required("selection.background");
            let caret_color = fg;
            let overlay_bg = theme.color_required("muted");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: font_size,
                ..Default::default()
            };

            let (
                content_len,
                boundary_mode,
                a11y_value,
                a11y_text_selection,
                a11y_text_composition,
            ) = {
                let mut st = editor_state.borrow_mut();
                if st.display_wrap_cols != soft_wrap_cols {
                    st.display_wrap_cols = soft_wrap_cols;
                    st.refresh_display_map();
                }
                let content_len = st.display_map.row_count();
                let boundary_mode = st.text_boundary_mode;
                let (value, selection, composition) = a11y_composed_text_window(&st);
                (
                    content_len,
                    boundary_mode,
                    Some(Arc::<str>::from(value)),
                    selection,
                    composition,
                )
            };

            let mut region_layout = fret_ui::element::LayoutStyle::default();
            region_layout.size.width = Length::Fill;
            region_layout.size.height = Length::Fill;
            region_layout.overflow = Overflow::Clip;

            let region_props = TextInputRegionProps {
                layout: region_layout,
                enabled: true,
                text_boundary_mode_override: Some(boundary_mode),
                a11y_label: Some(Arc::clone(&a11y_label)),
                a11y_value,
                a11y_text_selection,
                a11y_text_composition,
            };

            let mut pointer_props = PointerRegionProps::default();
            pointer_props.layout.size.width = Length::Fill;
            pointer_props.layout.size.height = Length::Fill;

            let mut surface_props = WindowedRowsSurfaceProps::default();
            surface_props.scroll.layout.size.width = Length::Fill;
            surface_props.scroll.layout.size.height = Length::Fill;
            surface_props.scroll.layout.overflow = Overflow::Clip;
            surface_props.len = content_len;
            surface_props.row_height = row_h;
            surface_props.overscan = overscan;
            surface_props.scroll_handle = scroll_handle.clone();
            let viewport_rows = if row_h.0 > 0.0 {
                (cx.bounds.size.height.0 / row_h.0).ceil() as usize
            } else {
                0
            };
            let text_cache_max_entries = viewport_rows
                .saturating_add(overscan.saturating_mul(2))
                .saturating_add(128)
                .clamp(256, 8_192);
            surface_props.canvas.cache_policy = CanvasCachePolicy {
                text: CanvasCacheTuning {
                    keep_frames: 60,
                    max_entries: text_cache_max_entries,
                },
                shared_text: CanvasCacheTuning::transient(),
                path: CanvasCacheTuning::transient(),
                svg: CanvasCacheTuning::transient(),
            };
            surface_props.on_paint_frame = torture.map(|torture| {
                let scroll_handle = scroll_handle.clone();
                let scroll_dir = scroll_dir.clone();
                let text_style = text_style.clone();
                let editor_state = editor_state.clone();
                let prev_stats = Rc::new(Cell::new(CodeEditorCacheStats::default()));
                let hook: OnWindowedRowsPaintFrame = Arc::new(
                    move |painter: &mut fret_ui::canvas::CanvasPainter<'_>,
                          frame: WindowedRowsPaintFrame| {
                        if !torture.auto_scroll {
                            return;
                        }

                        let max = scroll_handle.max_offset();
                        if max.y.0 <= 0.0 {
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
                        painter.request_animation_frame();

                        if !torture.show_overlay {
                            return;
                        }

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
                                syntax_get_calls: stats
                                    .syntax_get_calls
                                    .saturating_sub(prev.syntax_get_calls),
                                syntax_hits: stats.syntax_hits.saturating_sub(prev.syntax_hits),
                                syntax_misses: stats.syntax_misses.saturating_sub(prev.syntax_misses),
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
                            rect: Rect::new(origin, Size::new(Px(620.0), Px(24.0))),
                            background: overlay_bg,
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: Corners::all(Px(6.0)),
                        });

                        let label = format!(
                            "rows={}-{} y={:.0}/{:.0} max={} text {}/{}/{} (+{}/{}/{}) syn {}/{}/{} (+{}/{}/{}) geom row={} pref_x={:?} stops={:?} cache={}",
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
                                max_width: Some(Px(600.0)),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            },
                            painter.scale_factor(),
                        );
                    },
                );
                hook
            });

            cx.text_input_region(region_props, |cx| {
                // `TextInputRegion` creates its own element id scope. All focus/key/command hooks
                // must target this id (not the outer keyed scope), otherwise Web/WASM input routing
                // will never attach to the focused text region.
                let region_id = cx.root_id();

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
                let cmd_cell_w = cell_w.clone();
                cx.command_on_command_for(
                    region_id,
                    Arc::new(
                        move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                              action_cx: ActionCx,
                              command| {
                            let mut st = cmd_state.borrow_mut();
                            let mut did = false;
                            match command.as_str() {
                                "edit.undo" => {
                                    did = input::undo(&mut st);
                                }
                                "edit.redo" => {
                                    did = input::redo(&mut st);
                                }
                                "text.select_all" => {
                                    let end = st.buffer.len_bytes();
                                    st.selection = Selection {
                                        anchor: 0,
                                        focus: end,
                                    };
                                    st.preedit = None;
                                    st.undo_group = None;
                                    did = true;
                                }
                                "text.copy" => {
                                    input::copy_selection(host, &st);
                                    did = true;
                                }
                                "text.cut" => {
                                    if input::cut_selection(host, &mut st) {
                                        did = true;
                                    }
                                }
                                "text.paste" => {
                                    input::request_paste(host, action_cx);
                                    did = true;
                                }
                                "text.move_word_left" => {
                                    st.preedit = None;
                                    did = input::move_word(&mut st, -1, false);
                                }
                                "text.move_word_right" => {
                                    st.preedit = None;
                                    did = input::move_word(&mut st, 1, false);
                                }
                                "text.select_word_left" => {
                                    st.preedit = None;
                                    did = input::move_word(&mut st, -1, true);
                                }
                                "text.select_word_right" => {
                                    st.preedit = None;
                                    did = input::move_word(&mut st, 1, true);
                                }
                                _ => return false,
                            }

                            if did {
                                input::scroll_caret_into_view(&st, row_h, &cmd_scroll);
                                input::push_caret_rect_effect(
                                    host,
                                    action_cx,
                                    &st,
                                    row_h,
                                    cmd_cell_w.get(),
                                    &cmd_scroll,
                                );
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            }
                            true
                        },
                    ),
                );

                let on_pointer_down_state = editor_state.clone();
                let on_pointer_down_cell_w = cell_w.clone();
                let on_pointer_down_scroll = scroll_handle.clone();
                let on_pointer_down: OnWindowedRowsPointerDown = Arc::new(
                    move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, row, down| {
                        if down.button != MouseButton::Left {
                            return false;
                        }

                        host.set_cursor_icon(CursorIcon::Text);
                        host.request_focus(region_id);
                        host.capture_pointer();

                        let bounds = host.bounds();
                        let cell_w = on_pointer_down_cell_w.get();
                        let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };

                        let mut st = on_pointer_down_state.borrow_mut();
                        st.last_bounds = Some(bounds);
                        st.dragging = true;
                        st.drag_pointer = Some(down.pointer_id);
                        st.undo_group = None;
                        st.preedit = None;

                        let caret = caret_for_pointer(&st, row, bounds, down.position, cell_w);
                        match down.click_count {
                            2 => {
                                let (start, end) = select_word_range_in_buffer(
                                    &st.buffer,
                                    caret,
                                    st.text_boundary_mode,
                                );
                                st.selection = Selection {
                                    anchor: start,
                                    focus: end,
                                };
                                st.caret_preferred_x = None;
                            }
                            3 => {
                                let start = st
                                    .display_map
                                    .display_point_to_byte(&st.buffer, DisplayPoint::new(row, 0));
                                let line = st.buffer.line_index_at_byte(start);
                                if let Some(range) = st.buffer.line_byte_range_including_newline(line)
                                {
                                    st.selection = Selection {
                                        anchor: range.start,
                                        focus: range.end,
                                    };
                                }
                                st.caret_preferred_x = None;
                            }
                            _ => {
                                if down.modifiers.shift {
                                    st.selection.focus = caret;
                                } else {
                                    st.selection = Selection {
                                        anchor: caret,
                                        focus: caret,
                                    };
                                }
                                st.caret_preferred_x = None;
                            }
                        }

                        let caret_rect = caret_rect_for_selection(
                            &st,
                            row_h,
                            cell_w,
                            bounds,
                            &on_pointer_down_scroll,
                        );
                        if let Some(rect) = caret_rect {
                            host.push_effect(Effect::ImeSetCursorArea {
                                window: action_cx.window,
                                rect,
                            });
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
                    move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, row, mv| {
                        // Show an I-beam cursor while hovering the editor surface, even when not dragging.
                        host.set_cursor_icon(CursorIcon::Text);
                        if !mv.buttons.left {
                            return false;
                        }
                        let mut st = on_pointer_move_state.borrow_mut();
                        if !st.dragging {
                            return false;
                        }
                        st.undo_group = None;

                        let bounds = host.bounds();
                        st.last_bounds = Some(bounds);

                        let Some(row) = row else {
                            return false;
                        };

                        // Drag-to-select autoscroll: when the pointer is near/over the viewport edges,
                        // scroll so selection can extend beyond the visible window (Zed-style).
                        //
                        // Pointer positions are mapped through transforms; within scroll containers this
                        // typically means "content space". Use the scroll offset + viewport size to
                        // compute edge proximity in the same space.
                        let mut changed = false;
                        let viewport_h = Px(on_pointer_move_scroll.viewport_size().height.0.max(0.0));
                        if viewport_h.0 > 0.0 {
                            let offset = on_pointer_move_scroll.offset();
                            let vertical_margin = Px(row_h.0.min(viewport_h.0 / 3.0));
                            let top = Px(offset.y.0 + vertical_margin.0);
                            let bottom = Px(offset.y.0 + viewport_h.0 - vertical_margin.0);

                            let scale_vertical_delta = |delta_px: f32| -> f32 {
                                (delta_px.max(0.0).powf(1.2) / 100.0).min(3.0)
                            };

                            let mut scroll_delta_y = 0.0f32;
                            if mv.position.y.0 < top.0 {
                                scroll_delta_y = -scale_vertical_delta(top.0 - mv.position.y.0);
                            } else if mv.position.y.0 > bottom.0 {
                                scroll_delta_y = scale_vertical_delta(mv.position.y.0 - bottom.0);
                            }

                            if scroll_delta_y.abs() > 0.0 {
                                on_pointer_move_scroll.set_offset(fret_core::Point::new(
                                    offset.x,
                                    Px(offset.y.0 + scroll_delta_y),
                                ));
                                changed = true;
                            }
                        }

                        let cell_w = on_pointer_move_cell_w.get();
                        let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
                        let caret = caret_for_pointer(&st, row, bounds, mv.position, cell_w);
                        if caret != st.selection.focus {
                            st.selection.focus = caret;
                            st.caret_preferred_x = None;
                            changed = true;
                        }

                        let caret_rect = caret_rect_for_selection(
                            &st,
                            row_h,
                            cell_w,
                            bounds,
                            &on_pointer_move_scroll,
                        );
                        if let Some(rect) = caret_rect {
                            host.push_effect(Effect::ImeSetCursorArea {
                                window: action_cx.window,
                                rect,
                            });
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
                        st.dragging = false;
                        st.drag_pointer = None;
                        st.undo_group = None;
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
                        if st.drag_pointer == Some(cancel.pointer_id) {
                            st.dragging = false;
                            st.drag_pointer = None;
                        }
                        st.undo_group = None;
                        host.release_pointer_capture();
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        false
                    },
                );

                let handlers = WindowedRowsSurfacePointerHandlers {
                    on_pointer_down: Some(on_pointer_down),
                    on_pointer_move: Some(on_pointer_move),
                    on_pointer_up: Some(on_pointer_up),
                    on_pointer_cancel: Some(on_pointer_cancel),
                };

                let text_state = editor_state.clone();
                let text_scroll = scroll_handle.clone();
                let text_cell_w = cell_w.clone();
                cx.text_input_region_on_text_input(Arc::new(
                    move |host: &mut dyn UiActionHost, action_cx: ActionCx, text: &str| {
                        let mut st = text_state.borrow_mut();
                        st.preedit = None;
                        if input::insert_text(&mut st, text).is_some() {
                            input::scroll_caret_into_view(&st, row_h, &text_scroll);
                            input::push_caret_rect_effect(
                                host,
                                action_cx,
                                &st,
                                row_h,
                                text_cell_w.get(),
                                &text_scroll,
                            );
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            return true;
                        }
                        false
                    },
                ));

                let ime_state = editor_state.clone();
                let ime_scroll = scroll_handle.clone();
                let ime_cell_w = cell_w.clone();
                cx.text_input_region_on_ime(Arc::new(
                    move |host: &mut dyn UiActionHost,
                          action_cx: ActionCx,
                          ime: &fret_core::ImeEvent| {
                        let mut st = ime_state.borrow_mut();
                        match ime {
                            fret_core::ImeEvent::Enabled => return false,
                            fret_core::ImeEvent::Disabled => {
                                st.preedit = None;
                            }
                            fret_core::ImeEvent::Commit(text) => {
                                let _ = input::insert_text_with_kind(
                                    &mut st,
                                    text.as_str(),
                                    UndoGroupKind::Typing,
                                );
                                st.preedit = None;
                            }
                            fret_core::ImeEvent::Preedit { text, cursor } => {
                                if text.is_empty() {
                                    st.preedit = None;
                                } else {
                                    st.preedit = Some(PreeditState {
                                        text: text.clone(),
                                        cursor: *cursor,
                                    });
                                }
                            }
                            fret_core::ImeEvent::DeleteSurrounding {
                                before_bytes,
                                after_bytes,
                            } => {
                                let range = st.selection.normalized();
                                let caret = st.selection.caret().min(st.buffer.len_bytes());
                                let start = if range.is_empty() {
                                    caret.saturating_sub(*before_bytes)
                                } else {
                                    range.start
                                }
                                .min(st.buffer.len_bytes());
                                let end = if range.is_empty() {
                                    caret.saturating_add(*after_bytes)
                                } else {
                                    range.end
                                }
                                .min(st.buffer.len_bytes());

                                let start = st.buffer.prev_char_boundary(start);
                                let end = st.buffer.next_char_boundary(end);
                                if start < end {
                                    let kind = if *before_bytes > 0 {
                                        UndoGroupKind::Backspace
                                    } else {
                                        UndoGroupKind::DeleteForward
                                    };
                                    let _ = input::apply_and_record_edit(
                                        &mut st,
                                        kind,
                                        Edit::Delete { range: start..end },
                                        Selection {
                                            anchor: start,
                                            focus: start,
                                        },
                                    );
                                }
                            }
                        }

                        input::scroll_caret_into_view(&st, row_h, &ime_scroll);
                        input::push_caret_rect_effect(
                            host,
                            action_cx,
                            &st,
                            row_h,
                            ime_cell_w.get(),
                            &ime_scroll,
                        );
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    },
                ));

                let sel_state = editor_state.clone();
                let sel_scroll = scroll_handle.clone();
                let sel_cell_w = cell_w.clone();
                cx.text_input_region_on_set_selection(Arc::new(
                    move |host: &mut dyn UiActionHost, action_cx: ActionCx, anchor, focus| {
                        let mut st = sel_state.borrow_mut();
                        if st.preedit.is_some() {
                            return false;
                        }

                        let caret = st
                            .buffer
                            .clamp_to_char_boundary_left(st.selection.caret().min(st.buffer.len_bytes()));
                        let (start, end) = a11y_text_window_bounds(&st.buffer, caret);

                        let new_anchor = map_a11y_offset_to_buffer(&st.buffer, start, end, anchor);
                        let new_focus = map_a11y_offset_to_buffer(&st.buffer, start, end, focus);

                        st.selection = Selection {
                            anchor: new_anchor,
                            focus: new_focus,
                        };
                        st.undo_group = None;

                        input::scroll_caret_into_view(&st, row_h, &sel_scroll);
                        input::push_caret_rect_effect(
                            host,
                            action_cx,
                            &st,
                            row_h,
                            sel_cell_w.get(),
                            &sel_scroll,
                        );
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    },
                ));

                let clipboard_state = editor_state.clone();
                let clipboard_scroll = scroll_handle.clone();
                let clipboard_cell_w = cell_w.clone();
                cx.text_input_region_on_clipboard_text(Arc::new(
                    move |host: &mut dyn UiActionHost,
                          action_cx: ActionCx,
                          _token: ClipboardToken,
                          text: &str| {
                        let mut st = clipboard_state.borrow_mut();
                        let _ = input::insert_text_with_kind(&mut st, text, UndoGroupKind::Paste);
                        input::scroll_caret_into_view(&st, row_h, &clipboard_scroll);
                        input::push_caret_rect_effect(
                            host,
                            action_cx,
                            &st,
                            row_h,
                            clipboard_cell_w.get(),
                            &clipboard_scroll,
                        );
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    },
                ));

                vec![windowed_rows_surface_with_pointer_region(
                    cx,
                    surface_props,
                    pointer_props,
                    handlers,
                    Some(SemanticsProps {
                        role: fret_core::SemanticsRole::Viewport,
                        label: Some(Arc::<str>::from("Editor viewport")),
                        test_id: viewport_test_id.clone(),
                        ..Default::default()
                    }),
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
                )]
            })
        })
    }
}
