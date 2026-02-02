//! Code editor surface (UI integration) for Fret.
//!
//! This is a v1 MVP: fixed row height and a monospace "cell width" fallback for caret/selection
//! geometry while the surface migrates to pixel-accurate text geometry queries. Optional soft-wrap
//! is supported via the view-layer `DisplayMap`.

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
    AttributedText, Color, Corners, DecorationLineStyle, DrawOrder, Edges, FontId, KeyCode,
    Modifiers, MouseButton, Px, Rect, SceneOp, Size, TextOverflow, TextPaintStyle, TextSpan,
    TextStyle, TextWrap, UnderlineStyle,
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

const A11Y_WINDOW_BYTES_BEFORE: usize = 4096;
const A11Y_WINDOW_BYTES_AFTER: usize = 4096;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RowPreeditMapping {
    /// Byte offset within the row slice where preedit is injected.
    insert_at: usize,
    /// UTF-8 byte length of the injected preedit text.
    preedit_len: usize,
}

#[derive(Debug, Clone)]
struct RowGeom {
    /// Display-row range within the buffer (UTF-8 byte indices).
    row_range: Range<usize>,
    /// Caret stop table for the displayed row text (byte index -> x offset).
    caret_stops: Vec<(usize, Px)>,
    /// Mapping needed when the displayed row includes an injected preedit string.
    preedit: Option<RowPreeditMapping>,
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
    row_text_cache: HashMap<usize, (Arc<str>, u64)>,
    row_text_cache_queue: VecDeque<(usize, u64)>,
    row_geom_cache_rev: fret_code_editor_buffer::Revision,
    row_geom_cache_wrap_cols: Option<usize>,
    row_geom_cache_tick: u64,
    row_geom_cache: HashMap<usize, (RowGeom, u64)>,
    row_geom_cache_queue: VecDeque<(usize, u64)>,
    selection_rect_scratch: Vec<Rect>,
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
        }
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
        let cell_w = cx.with_state(|| Cell::new(Px(0.0)), |c| c.clone());
        let scroll_dir = cx.with_state(|| Cell::new(1i32), |c| c.clone());

        let editor_state = self.handle.state.clone();
        let overscan = self.overscan;
        let torture = self.torture;
        let soft_wrap_cols = self.soft_wrap_cols;
        let a11y_label: Arc<str> = Arc::from("Code editor");

        cx.keyed("code-editor", move |cx| {
            let theme = cx.theme().clone();
            let region_id = cx.root_id();
            let is_focused = cx.is_focused_element(region_id);

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
                if !is_focused {
                    (content_len, boundary_mode, None, None, None)
                } else {
                    let (value, selection, composition) = a11y_composed_text_window(&st);
                    (
                        content_len,
                        boundary_mode,
                        Some(Arc::<str>::from(value)),
                        selection,
                        composition,
                    )
                }
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

            let on_pointer_down_state = editor_state.clone();
            let on_pointer_down_cell_w = cell_w.clone();
            let on_pointer_down_scroll = scroll_handle.clone();
            let on_pointer_down: OnWindowedRowsPointerDown = Arc::new(
                move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, row, down| {
                    if down.button != MouseButton::Left {
                        return false;
                    }

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
                            let (start, end) =
                                select_word_range_in_buffer(&st.buffer, caret, st.text_boundary_mode);
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
                            if let Some(range) = st.buffer.line_byte_range_including_newline(line) {
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

                    let caret_rect =
                        caret_rect_for_selection(&st, row_h, cell_w, bounds, &on_pointer_down_scroll);
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
                    let Some(row) = row else {
                        return false;
                    };
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

                    let cell_w = on_pointer_move_cell_w.get();
                    let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
                    let caret = caret_for_pointer(&st, row, bounds, mv.position, cell_w);
                    st.selection.focus = caret;
                    st.caret_preferred_x = None;

                    let caret_rect =
                        caret_rect_for_selection(&st, row_h, cell_w, bounds, &on_pointer_move_scroll);
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

            let key_state = editor_state.clone();
            let key_scroll = scroll_handle.clone();
            let key_cell_w = cell_w.clone();
            cx.key_on_key_down_for(
                region_id,
                Arc::new(
                    move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                          action_cx: ActionCx,
                          down: KeyDownCx| {
                        if handle_key_down(
                            host,
                            action_cx,
                            &key_state,
                            row_h,
                            &key_scroll,
                            &key_cell_w,
                            down.key,
                            down.modifiers,
                        ) {
                            return true;
                        }
                        false
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
                                did = undo(&mut st);
                            }
                            "edit.redo" => {
                                did = redo(&mut st);
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
                                copy_selection(host, &st);
                                did = true;
                            }
                            "text.cut" => {
                                if cut_selection(host, &mut st) {
                                    did = true;
                                }
                            }
                            "text.paste" => {
                                request_paste(host, action_cx);
                                did = true;
                            }
                            "text.move_word_left" => {
                                st.preedit = None;
                                did = move_word(&mut st, -1, false);
                            }
                            "text.move_word_right" => {
                                st.preedit = None;
                                did = move_word(&mut st, 1, false);
                            }
                            "text.select_word_left" => {
                                st.preedit = None;
                                did = move_word(&mut st, -1, true);
                            }
                            "text.select_word_right" => {
                                st.preedit = None;
                                did = move_word(&mut st, 1, true);
                            }
                            _ => return false,
                        }

                        if did {
                            push_caret_rect_effect(
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

            let handlers = WindowedRowsSurfacePointerHandlers {
                on_pointer_down: Some(on_pointer_down),
                on_pointer_move: Some(on_pointer_move),
                on_pointer_up: Some(on_pointer_up),
                on_pointer_cancel: Some(on_pointer_cancel),
            };

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

                        let (stats, delta) = {
                            let stats = editor_state.borrow().cache_stats;
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
                            (stats, delta)
                        };

                        let origin = fret_core::Point::new(Px(8.0), Px(offset.y.0 + 8.0));
                        painter.scene().push(SceneOp::Quad {
                            order: DrawOrder(100),
                            rect: Rect::new(origin, Size::new(Px(620.0), Px(24.0))),
                            background: overlay_bg,
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: Corners::all(Px(6.0)),
                        });

                        let label = format!(
                            "rows={}-{} y={:.0}/{:.0} max={} text {}/{}/{} (+{}/{}/{}) syn {}/{}/{} (+{}/{}/{})",
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
                            delta.syntax_misses
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
                let text_state = editor_state.clone();
                let text_scroll = scroll_handle.clone();
                let text_cell_w = cell_w.clone();
                cx.text_input_region_on_text_input(Arc::new(
                    move |host: &mut dyn UiActionHost, action_cx: ActionCx, text: &str| {
                        let mut st = text_state.borrow_mut();
                        st.preedit = None;
                        if insert_text(&mut st, text).is_some() {
                            push_caret_rect_effect(
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
                                let _ = insert_text_with_kind(
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
                                    let _ = apply_and_record_edit(
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

                        push_caret_rect_effect(
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

                        push_caret_rect_effect(
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
                        let _ = insert_text_with_kind(&mut st, text, UndoGroupKind::Paste);
                        push_caret_rect_effect(
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
                        paint_row(
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

fn a11y_composed_text_window(
    st: &CodeEditorState,
) -> (String, Option<(u32, u32)>, Option<(u32, u32)>) {
    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(st.buffer.len_bytes()));

    let (start, end) = a11y_text_window_bounds(&st.buffer, caret);

    let before = st.buffer.slice_to_string(start..caret).unwrap_or_default();
    let after = st.buffer.slice_to_string(caret..end).unwrap_or_default();

    if let Some(preedit) = st.preedit.as_ref() {
        let mut display = String::with_capacity(before.len() + preedit.text.len() + after.len());
        display.push_str(before.as_str());
        display.push_str(preedit.text.as_str());
        display.push_str(after.as_str());

        let before_len: u32 = before.len().try_into().unwrap_or(u32::MAX);
        let preedit_len: u32 = preedit.text.len().try_into().unwrap_or(u32::MAX);

        let composition = Some((before_len, before_len.saturating_add(preedit_len)));

        let (mut a, mut b) = preedit
            .cursor
            .unwrap_or_else(|| (preedit.text.len(), preedit.text.len()));
        a = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, a).min(preedit.text.len());
        b = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, b).min(preedit.text.len());
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }

        let selection = Some((
            before_len.saturating_add(a as u32),
            before_len.saturating_add(b as u32),
        ));

        return (display, selection, composition);
    }

    let mut display = String::with_capacity(before.len() + after.len());
    display.push_str(before.as_str());
    display.push_str(after.as_str());

    let map = |offset: usize| -> u32 {
        let offset = offset.min(end).max(start);
        let offset = st.buffer.clamp_to_char_boundary_left(offset);
        u32::try_from(offset.saturating_sub(start)).unwrap_or(u32::MAX)
    };
    let selection = Some((map(st.selection.anchor), map(st.selection.focus)));

    (display, selection, None)
}

fn a11y_text_window_bounds(buf: &TextBuffer, caret: usize) -> (usize, usize) {
    let caret = buf.clamp_to_char_boundary_left(caret.min(buf.len_bytes()));
    let start = buf.clamp_to_char_boundary_left(caret.saturating_sub(A11Y_WINDOW_BYTES_BEFORE));
    let end = buf.clamp_to_char_boundary_left(
        caret
            .saturating_add(A11Y_WINDOW_BYTES_AFTER)
            .min(buf.len_bytes()),
    );
    (start, end)
}

fn map_a11y_offset_to_buffer(
    buf: &TextBuffer,
    window_start: usize,
    window_end: usize,
    offset: u32,
) -> usize {
    let window_start = window_start.min(buf.len_bytes());
    let window_end = window_end.min(buf.len_bytes()).max(window_start);
    let window_len = window_end.saturating_sub(window_start);
    let offset = usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(window_len);
    let byte = window_start.saturating_add(offset).min(window_end);
    buf.clamp_to_char_boundary_left(byte).min(buf.len_bytes())
}

fn caret_x_for_index(stops: &[(usize, Px)], index: usize) -> Px {
    if stops.is_empty() {
        return Px(0.0);
    }
    // Prefer an exact index match, otherwise clamp to the nearest representable caret stop.
    let mut lo = 0usize;
    let mut hi = stops.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if stops[mid].0 < index {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo < stops.len() && stops[lo].0 == index {
        return stops[lo].1;
    }
    if lo == 0 {
        return stops[0].1;
    }
    stops[lo.saturating_sub(1)].1
}

fn hit_test_index_from_caret_stops(stops: &[(usize, Px)], x: Px) -> usize {
    if stops.is_empty() {
        return 0;
    }
    if stops.len() == 1 {
        return stops[0].0;
    }
    // Caret stops are expected to be monotonically increasing in X for a single-line blob.
    let x = x.0;
    if x <= stops[0].1.0 {
        return stops[0].0;
    }
    if x >= stops[stops.len() - 1].1.0 {
        return stops[stops.len() - 1].0;
    }
    let mut lo = 0usize;
    let mut hi = stops.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if stops[mid].1.0 < x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let right = lo.min(stops.len() - 1);
    let left = right.saturating_sub(1);
    let (li, lx) = stops[left];
    let (ri, rx) = stops[right];
    if (x - lx.0).abs() <= (rx.0 - x).abs() {
        li
    } else {
        ri
    }
}

fn map_row_local_to_buffer_byte(buf: &TextBuffer, geom: &RowGeom, local: usize) -> usize {
    let row_start = geom.row_range.start.min(buf.len_bytes());
    let row_end = geom.row_range.end.min(buf.len_bytes()).max(row_start);
    let max_local = row_end.saturating_sub(row_start);

    let mut local = local;
    if let Some(preedit) = geom.preedit {
        let insert_at = preedit.insert_at.min(max_local);
        let preedit_len = preedit.preedit_len;
        if local <= insert_at {
            local = local.min(insert_at);
            return row_start.saturating_add(local).min(row_end);
        }
        let after_insert = insert_at.saturating_add(preedit_len);
        if local >= after_insert {
            let base_local = local.saturating_sub(preedit_len);
            return row_start
                .saturating_add(base_local.min(max_local))
                .min(row_end);
        }
        // Inside the injected preedit: snap to the injection point in the base buffer.
        return row_start.saturating_add(insert_at).min(row_end);
    }

    row_start.saturating_add(local.min(max_local)).min(row_end)
}

fn caret_for_pointer(
    st: &CodeEditorState,
    row: usize,
    bounds: Rect,
    position: fret_core::Point,
    cell_w: Px,
) -> usize {
    let local_x = Px(position.x.0 - bounds.origin.x.0);
    if let Some((geom, _)) = st.row_geom_cache.get(&row)
        && !geom.caret_stops.is_empty()
    {
        let local = hit_test_index_from_caret_stops(&geom.caret_stops, local_x);
        let byte = map_row_local_to_buffer_byte(&st.buffer, geom, local);
        return st
            .buffer
            .clamp_to_char_boundary_left(byte.min(st.buffer.len_bytes()));
    }

    // Fallback to the MVP monospace heuristic when geometry hasn't been cached yet.
    let col = if cell_w.0 > 0.0 {
        (local_x.0 / cell_w.0).floor().max(0.0) as usize
    } else {
        0
    };
    st.display_map
        .display_point_to_byte(&st.buffer, DisplayPoint::new(row, col))
}

fn caret_rect_for_selection(
    st: &CodeEditorState,
    row_h: Px,
    cell_w: Px,
    bounds: Rect,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) -> Option<Rect> {
    if !st.selection.is_caret() {
        return None;
    }

    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(st.buffer.len_bytes()));
    let pt = st.display_map.byte_to_display_point(&st.buffer, caret);
    let offset = scroll_handle.offset();
    let y = Px(bounds.origin.y.0 + (pt.row as f32 * row_h.0) - offset.y.0);

    let x = if let Some((geom, _)) = st.row_geom_cache.get(&pt.row)
        && !geom.caret_stops.is_empty()
        && caret >= geom.row_range.start
    {
        let mut local = caret.saturating_sub(geom.row_range.start);
        if let Some(preedit) = st.preedit.as_ref()
            && geom.preedit.is_some()
        {
            local = local.saturating_add(preedit_cursor_offset_bytes(preedit));
        }
        let cx = caret_x_for_index(&geom.caret_stops, local);
        Px(bounds.origin.x.0 + cx.0)
    } else {
        let mut col = pt.col;
        if let Some(preedit) = st.preedit.as_ref() {
            col = col.saturating_add(preedit_cursor_offset_cols(preedit));
        }
        Px(bounds.origin.x.0 + col as f32 * cell_w.0)
    };

    Some(Rect::new(
        fret_core::Point::new(x, y),
        Size::new(Px(1.0), row_h),
    ))
}

fn push_caret_rect_effect(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    st: &CodeEditorState,
    row_h: Px,
    cell_w: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) {
    let Some(bounds) = st.last_bounds else {
        return;
    };
    let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
    if let Some(rect) = caret_rect_for_selection(st, row_h, cell_w, bounds, scroll_handle) {
        host.push_effect(Effect::ImeSetCursorArea {
            window: action_cx.window,
            rect,
        });
    }
}

fn preedit_cursor_offset_cols(preedit: &PreeditState) -> usize {
    let mut end = preedit
        .cursor
        .map(|(_, end)| end)
        .unwrap_or_else(|| preedit.text.len());
    end = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, end).min(preedit.text.len());
    preedit.text[..end].chars().count()
}

fn preedit_cursor_offset_bytes(preedit: &PreeditState) -> usize {
    let mut end = preedit
        .cursor
        .map(|(_, end)| end)
        .unwrap_or_else(|| preedit.text.len());
    end = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, end).min(preedit.text.len());
    end
}

fn insert_text(st: &mut CodeEditorState, text: &str) -> Option<()> {
    insert_text_with_kind(st, text, UndoGroupKind::Typing)
}

fn insert_text_with_kind(st: &mut CodeEditorState, text: &str, kind: UndoGroupKind) -> Option<()> {
    if text.is_empty() {
        return None;
    }
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let caret = start.saturating_add(text.len()).min(st.buffer.len_bytes());
    apply_and_record_edit(
        st,
        kind,
        Edit::Replace {
            range: start..end,
            text: text.to_string(),
        },
        Selection {
            anchor: caret,
            focus: caret,
        },
    )?;
    st.caret_preferred_x = None;
    Some(())
}

#[allow(clippy::too_many_arguments)]
fn handle_key_down(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: ActionCx,
    state: &Rc<RefCell<CodeEditorState>>,
    row_h: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
    cell_w: &Cell<Px>,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    let mut st = state.borrow_mut();
    let shift = modifiers.shift;
    let ctrl_or_meta = modifiers.ctrl || modifiers.meta;
    let word = modifiers.ctrl || modifiers.alt;
    let meta = modifiers.meta;

    if st.preedit.is_some() {
        let cancel_preedit = match key {
            KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::Backspace
            | KeyCode::Delete
            | KeyCode::Enter
            | KeyCode::Tab => true,
            KeyCode::PageUp | KeyCode::PageDown => !ctrl_or_meta,
            _ => false,
        };
        if cancel_preedit {
            st.preedit = None;
        }
    }

    // Let workspace keymaps handle global page navigation (e.g. tab switching).
    if ctrl_or_meta && matches!(key, KeyCode::PageUp | KeyCode::PageDown) {
        return false;
    }

    let cell_w_px = cell_w.get();

    match key {
        KeyCode::ArrowLeft => {
            if meta {
                move_caret_home_end(&mut st, true, false, shift);
            } else if word {
                move_word(&mut st, -1, shift);
            } else {
                move_caret_left(&mut st, shift);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowRight => {
            if meta {
                move_caret_home_end(&mut st, false, false, shift);
            } else if word {
                move_word(&mut st, 1, shift);
            } else {
                move_caret_right(&mut st, shift);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowUp => {
            if meta {
                move_caret_home_end(&mut st, true, true, shift);
            } else {
                move_caret_vertical(&mut st, -1, shift, cell_w_px);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowDown => {
            if meta {
                move_caret_home_end(&mut st, false, true, shift);
            } else {
                move_caret_vertical(&mut st, 1, shift, cell_w_px);
            }
            st.undo_group = None;
        }
        KeyCode::Home => {
            move_caret_home_end(&mut st, true, ctrl_or_meta, shift);
            st.undo_group = None;
        }
        KeyCode::End => {
            move_caret_home_end(&mut st, false, ctrl_or_meta, shift);
            st.undo_group = None;
        }
        KeyCode::PageUp => {
            move_caret_page(&mut st, -1, shift, row_h, scroll_handle, cell_w_px);
            st.undo_group = None;
        }
        KeyCode::PageDown => {
            move_caret_page(&mut st, 1, shift, row_h, scroll_handle, cell_w_px);
            st.undo_group = None;
        }
        KeyCode::Backspace => {
            if word {
                delete_word_backward(&mut st);
            } else {
                delete_backward(&mut st);
            }
        }
        KeyCode::Delete => {
            if word {
                delete_word_forward(&mut st);
            } else {
                delete_forward(&mut st);
            }
        }
        KeyCode::Enter => {
            let _ = insert_text(&mut st, "\n");
        }
        KeyCode::Tab => {
            let _ = insert_text(&mut st, "\t");
        }
        KeyCode::KeyC if ctrl_or_meta => copy_selection(host, &st),
        KeyCode::KeyV if ctrl_or_meta => request_paste(host, action_cx),
        _ => return false,
    }

    push_caret_rect_effect(host, action_cx, &st, row_h, cell_w_px, scroll_handle);

    host.notify(action_cx);
    host.request_redraw(action_cx.window);
    true
}

fn page_rows(row_h: Px, scroll_handle: &fret_ui::scroll::ScrollHandle) -> usize {
    if row_h.0 <= 0.0 {
        return 1;
    }
    let viewport = scroll_handle.viewport_size();
    ((viewport.height.0 / row_h.0).floor() as usize).max(1)
}

fn move_caret_page(
    st: &mut CodeEditorState,
    pages: i32,
    extend: bool,
    row_h: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
    cell_w: Px,
) {
    let rows = page_rows(row_h, scroll_handle);
    let delta = pages.saturating_mul(rows as i32);
    if delta != 0 {
        move_caret_vertical(st, delta, extend, cell_w);
    }

    // Keep the viewport moving with the caret for page navigation.
    let offset = scroll_handle.offset();
    let dy = row_h.0 * rows as f32;
    let next_y = if pages < 0 {
        offset.y.0 - dy * pages.unsigned_abs() as f32
    } else {
        offset.y.0 + dy * pages as f32
    };
    scroll_handle.scroll_to_offset(fret_core::Point::new(offset.x, Px(next_y)));
}

fn move_caret_home_end(st: &mut CodeEditorState, home: bool, ctrl_or_meta: bool, extend: bool) {
    let sel = st.selection.normalized();
    let mut caret = st.selection.caret().min(st.buffer.len_bytes());
    if !st.selection.is_caret() && !extend {
        caret = if home { sel.start } else { sel.end };
    }

    let target = if ctrl_or_meta {
        if home { 0 } else { st.buffer.len_bytes() }
    } else {
        let row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
        let row_range = st.display_map.display_row_byte_range(&st.buffer, row);
        if home { row_range.start } else { row_range.end }
    };

    st.caret_preferred_x = None;
    if extend {
        if st.selection.is_caret() {
            st.selection.anchor = caret;
        }
        st.selection.focus = target;
    } else {
        st.selection = Selection {
            anchor: target,
            focus: target,
        };
    }
}

fn copy_selection(host: &mut dyn UiActionHost, st: &CodeEditorState) {
    let range = st.selection.normalized();
    if range.is_empty() {
        return;
    }
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let Some(text) = st.buffer.slice_to_string(start..end) else {
        return;
    };
    host.push_effect(Effect::ClipboardSetText { text });
}

fn request_paste(host: &mut dyn UiActionHost, action_cx: ActionCx) {
    let token = host.next_clipboard_token();
    host.push_effect(Effect::ClipboardGetText {
        window: action_cx.window,
        token,
    });
}

fn delete_word_backward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = apply_and_record_edit(
            st,
            UndoGroupKind::Backspace,
            Edit::Delete { range: start..end },
            Selection {
                anchor: start,
                focus: start,
            },
        );
        st.caret_preferred_x = None;
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    if caret == 0 {
        return;
    }

    let prev = move_word_left_in_buffer(&st.buffer, caret, st.text_boundary_mode).min(caret);
    if prev == caret {
        return;
    }

    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::Backspace,
        Edit::Delete { range: prev..caret },
        Selection {
            anchor: prev,
            focus: prev,
        },
    );
    st.caret_preferred_x = None;
}

fn delete_word_forward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = apply_and_record_edit(
            st,
            UndoGroupKind::DeleteForward,
            Edit::Delete { range: start..end },
            Selection {
                anchor: start,
                focus: start,
            },
        );
        st.caret_preferred_x = None;
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let next = move_word_right_in_buffer(&st.buffer, caret, st.text_boundary_mode)
        .max(caret)
        .min(st.buffer.len_bytes());
    if next == caret {
        return;
    }

    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::DeleteForward,
        Edit::Delete { range: caret..next },
        Selection {
            anchor: caret,
            focus: caret,
        },
    );
    st.caret_preferred_x = None;
}

fn delete_backward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = apply_and_record_edit(
            st,
            UndoGroupKind::Backspace,
            Edit::Delete { range: start..end },
            Selection {
                anchor: start,
                focus: start,
            },
        );
        st.caret_preferred_x = None;
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    if caret == 0 {
        return;
    }
    let prev = st.buffer.prev_char_boundary(caret);
    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::Backspace,
        Edit::Delete { range: prev..caret },
        Selection {
            anchor: prev,
            focus: prev,
        },
    );
    st.caret_preferred_x = None;
}

fn delete_forward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = apply_and_record_edit(
            st,
            UndoGroupKind::DeleteForward,
            Edit::Delete { range: start..end },
            Selection {
                anchor: start,
                focus: start,
            },
        );
        st.caret_preferred_x = None;
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let next = st.buffer.next_char_boundary(caret);
    if next == caret {
        return;
    }
    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::DeleteForward,
        Edit::Delete { range: caret..next },
        Selection {
            anchor: caret,
            focus: caret,
        },
    );
    st.caret_preferred_x = None;
}

fn move_caret_left(st: &mut CodeEditorState, extend: bool) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let new = st.buffer.prev_char_boundary(caret);
    st.caret_preferred_x = None;
    if extend {
        st.selection.focus = new;
    } else {
        st.selection = Selection {
            anchor: new,
            focus: new,
        };
    }
}

fn move_caret_right(st: &mut CodeEditorState, extend: bool) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let new = st.buffer.next_char_boundary(caret);
    st.caret_preferred_x = None;
    if extend {
        st.selection.focus = new;
    } else {
        st.selection = Selection {
            anchor: new,
            focus: new,
        };
    }
}

fn caret_x_for_buffer_byte_in_row(st: &CodeEditorState, row: usize, caret: usize) -> Option<Px> {
    let Some((geom, _)) = st.row_geom_cache.get(&row) else {
        return None;
    };
    if geom.caret_stops.is_empty() {
        return None;
    }
    let row_start = geom.row_range.start.min(st.buffer.len_bytes());
    if caret < row_start {
        return None;
    }
    let mut local = caret.saturating_sub(row_start);
    if let Some(preedit) = st.preedit.as_ref()
        && geom.preedit.is_some()
    {
        local = local.saturating_add(preedit_cursor_offset_bytes(preedit));
    }
    Some(caret_x_for_index(&geom.caret_stops, local))
}

fn move_caret_vertical(st: &mut CodeEditorState, delta: i32, extend: bool, cell_w: Px) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let pt = st.display_map.byte_to_display_point(&st.buffer, caret);

    let desired_x = st
        .caret_preferred_x
        .or_else(|| caret_x_for_buffer_byte_in_row(st, pt.row, caret))
        .unwrap_or_else(|| Px(pt.col as f32 * cell_w.0));
    st.caret_preferred_x = Some(desired_x);

    let next_row = if delta < 0 {
        pt.row.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        pt.row.saturating_add(delta as usize)
    };
    let max_row = st.display_map.row_count().saturating_sub(1);
    let next_row = next_row.min(max_row);
    let next = if let Some((geom, _)) = st.row_geom_cache.get(&next_row)
        && !geom.caret_stops.is_empty()
    {
        let local = hit_test_index_from_caret_stops(&geom.caret_stops, desired_x);
        let byte = map_row_local_to_buffer_byte(&st.buffer, geom, local);
        st.buffer
            .clamp_to_char_boundary_left(byte.min(st.buffer.len_bytes()))
    } else {
        st.display_map
            .display_point_to_byte(&st.buffer, DisplayPoint::new(next_row, pt.col))
    };
    if extend {
        st.selection.focus = next;
    } else {
        st.selection = Selection {
            anchor: next,
            focus: next,
        };
    }
}

fn apply_and_record_edit(
    st: &mut CodeEditorState,
    kind: UndoGroupKind,
    edit: Edit,
    next_selection: Selection,
) -> Option<()> {
    if !st.selection.is_caret() {
        st.undo_group = None;
    }
    if st.undo_group.as_ref().is_none_or(|g| g.kind != kind) {
        st.undo_group = Some(UndoGroup {
            kind,
            before_selection: st.selection,
            tx: TextBufferTransaction::default(),
            coalesce_key: kind.coalesce_key(),
        });
    }

    st.preedit = None;
    let delta = {
        let group = st.undo_group.as_mut().expect("undo group must exist");
        st.buffer.apply_in_transaction(&mut group.tx, edit).ok()?
    };
    if st.display_wrap_cols.is_some() || delta.lines.old_count != delta.lines.new_count {
        st.refresh_display_map();
    }
    #[cfg(feature = "syntax")]
    invalidate_syntax_row_cache_for_delta(st, delta);
    #[cfg(not(feature = "syntax"))]
    let _ = delta;
    st.selection = next_selection;
    st.caret_preferred_x = None;
    st.row_geom_cache_rev = st.buffer.revision();
    st.row_geom_cache_wrap_cols = st.display_wrap_cols;
    st.row_geom_cache_tick = 0;
    st.row_geom_cache.clear();
    st.row_geom_cache_queue.clear();

    let (buffer_tx, inverse_selection, coalesce_key) = {
        let group = st.undo_group.as_ref().expect("undo group must exist");
        (
            group.tx.snapshot(),
            group.before_selection,
            group.coalesce_key.clone(),
        )
    };
    let record = UndoRecord::new(CodeEditorTx {
        buffer_tx,
        selection: next_selection,
        inverse_selection,
    })
    .coalesce_key(coalesce_key);
    st.undo.record_or_coalesce(record);
    Some(())
}

fn undo(st: &mut CodeEditorState) -> bool {
    st.undo_group = None;
    st.caret_preferred_x = None;
    let (buffer, selection, preedit, history) = (
        &mut st.buffer,
        &mut st.selection,
        &mut st.preedit,
        &mut st.undo,
    );
    let mut applied = false;
    let _ = history.undo_invertible(|record| {
        *preedit = None;
        if buffer.apply_tx(&record.tx.buffer_tx).is_ok() {
            *selection = record.tx.selection;
            applied = true;
        }
        Ok::<_, ()>(())
    });
    if applied {
        st.refresh_display_map();
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
    }
    #[cfg(feature = "syntax")]
    {
        if applied {
            st.syntax_row_cache_rev = st.buffer.revision();
            st.syntax_row_cache_tick = 0;
            st.syntax_row_cache.clear();
            st.syntax_row_cache_queue.clear();
        }
    }
    applied
}

fn redo(st: &mut CodeEditorState) -> bool {
    st.undo_group = None;
    st.caret_preferred_x = None;
    let (buffer, selection, preedit, history) = (
        &mut st.buffer,
        &mut st.selection,
        &mut st.preedit,
        &mut st.undo,
    );
    let mut applied = false;
    let _ = history.redo_invertible(|record| {
        *preedit = None;
        if buffer.apply_tx(&record.tx.buffer_tx).is_ok() {
            *selection = record.tx.selection;
            applied = true;
        }
        Ok::<_, ()>(())
    });
    if applied {
        st.refresh_display_map();
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
    }
    #[cfg(feature = "syntax")]
    {
        if applied {
            st.syntax_row_cache_rev = st.buffer.revision();
            st.syntax_row_cache_tick = 0;
            st.syntax_row_cache.clear();
            st.syntax_row_cache_queue.clear();
        }
    }
    applied
}

fn move_word(st: &mut CodeEditorState, dir: i32, extend: bool) -> bool {
    let mode = st.text_boundary_mode;
    st.undo_group = None;
    st.caret_preferred_x = None;

    let (sel_start, sel_end) = {
        let r = st.selection.normalized();
        (r.start, r.end)
    };
    let mut caret = st.selection.caret().min(st.buffer.len_bytes());
    if !st.selection.is_caret() && !extend {
        caret = if dir < 0 { sel_start } else { sel_end };
    }

    let next = if dir < 0 {
        move_word_left_in_buffer(&st.buffer, caret, mode)
    } else {
        move_word_right_in_buffer(&st.buffer, caret, mode)
    };

    if extend {
        if st.selection.is_caret() {
            st.selection.anchor = caret;
        }
        st.selection.focus = next;
    } else {
        st.selection = Selection {
            anchor: next,
            focus: next,
        };
    }
    st.preedit = None;
    true
}

fn cut_selection(host: &mut dyn UiActionHost, st: &mut CodeEditorState) -> bool {
    let range = st.selection.normalized();
    if range.is_empty() {
        return false;
    }
    copy_selection(host, st);
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let out = apply_and_record_edit(
        st,
        UndoGroupKind::Cut,
        Edit::Delete { range: start..end },
        Selection {
            anchor: start,
            focus: start,
        },
    )
    .is_some();
    if out {
        st.caret_preferred_x = None;
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn paint_row(
    painter: &mut fret_ui::canvas::CanvasPainter<'_>,
    st: &mut CodeEditorState,
    row: usize,
    rect: Rect,
    row_h: Px,
    cell_w: Px,
    text_cache_max_entries: usize,
    text_style: &TextStyle,
    fg: Color,
    selection_bg: Color,
    caret_color: Color,
) {
    let row_range = st.display_map.display_row_byte_range(&st.buffer, row);
    let line = cached_row_text(st, row, text_cache_max_entries);
    painter.scene().push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Color::TRANSPARENT,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let origin = fret_core::Point::new(rect.origin.x, rect.origin.y);
    let scope = painter.key_scope(&"fret-code-editor-row-text");
    let key: u64 = painter.child_key(scope, &(row, 0u8)).into();
    let constraints = CanvasTextConstraints {
        max_width: Some(rect.size.width),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let mut drew_rich = false;
    let mut row_preedit = None::<RowPreeditMapping>;
    let mut row_blob = None::<fret_core::TextBlobId>;

    if let Some(preedit) = &st.preedit {
        let caret = st.selection.caret().min(st.buffer.len_bytes());
        let caret_pt = st.display_map.byte_to_display_point(&st.buffer, caret);
        if caret_pt.row == row {
            let mut caret_in_line = caret.saturating_sub(row_range.start).min(line.len());
            caret_in_line =
                fret_code_editor_view::clamp_to_char_boundary(line.as_ref(), caret_in_line);

            let rich = materialize_preedit_rich_text(
                Arc::clone(&line),
                caret_in_line,
                preedit,
                fg,
                selection_bg,
            );
            let key: u64 = painter.child_key(scope, &(row, 2u8)).into();
            let (blob, _) = painter.rich_text_with_blob(
                key,
                DrawOrder(2),
                origin,
                rich,
                text_style.clone(),
                fg,
                constraints,
                painter.scale_factor(),
            );
            row_preedit = Some(RowPreeditMapping {
                insert_at: caret_in_line,
                preedit_len: preedit.text.len(),
            });
            row_blob = Some(blob);
            drew_rich = true;
        }
    }
    #[cfg(feature = "syntax")]
    {
        if !drew_rich {
            let line_idx = st.display_map.display_row_line(row);
            let spans = cached_row_syntax_spans(st, line_idx, text_cache_max_entries);
            if !spans.is_empty() {
                let seg_start_in_line = row_range
                    .start
                    .saturating_sub(st.buffer.line_start(line_idx).unwrap_or(row_range.start));
                let seg_end_in_line = seg_start_in_line.saturating_add(line.len());

                let mut clipped: Vec<SyntaxSpan> = Vec::new();
                for span in spans.as_ref() {
                    let start = span.range.start.max(seg_start_in_line);
                    let end = span.range.end.min(seg_end_in_line);
                    if start >= end {
                        continue;
                    }
                    clipped.push(SyntaxSpan {
                        range: (start - seg_start_in_line)..(end - seg_start_in_line),
                        highlight: span.highlight,
                    });
                }

                if !clipped.is_empty() {
                    clipped.sort_by_key(|s| s.range.start);
                    clipped.dedup_by(|a, b| a.range == b.range && a.highlight == b.highlight);
                    let mut merged: Vec<SyntaxSpan> = Vec::new();
                    for span in clipped {
                        if let Some(last) = merged.last_mut()
                            && last.highlight == span.highlight
                            && last.range.end == span.range.start
                        {
                            last.range.end = span.range.end;
                            continue;
                        }
                        merged.push(span);
                    }

                    let theme = painter.theme().clone();
                    let rich =
                        materialize_row_rich_text(&theme, Arc::clone(&line), merged.as_ref());
                    let (blob, _) = painter.rich_text_with_blob(
                        key,
                        DrawOrder(2),
                        origin,
                        rich,
                        text_style.clone(),
                        fg,
                        constraints,
                        painter.scale_factor(),
                    );
                    row_blob = Some(blob);
                    drew_rich = true;
                }
            }
        }
    }

    if !drew_rich {
        let (blob, _) = painter.text_with_blob(
            key,
            DrawOrder(2),
            origin,
            Arc::clone(&line),
            text_style.clone(),
            fg,
            constraints,
            painter.scale_factor(),
        );
        row_blob = Some(blob);
    }

    let mut caret_stops: Vec<(usize, Px)> = Vec::new();
    if let Some(blob) = row_blob {
        let (services, _) = painter.services_and_scene();
        services.text().caret_stops(blob, &mut caret_stops);
    }

    let row_geom = RowGeom {
        row_range: row_range.clone(),
        caret_stops,
        preedit: row_preedit,
    };

    let sel = st.selection.normalized();
    let mut drew_selection = false;
    if !sel.is_empty() {
        let global_start = sel.start.max(row_range.start).min(row_range.end);
        let global_end = sel.end.max(row_range.start).min(row_range.end);
        if global_start < global_end
            && let Some(blob) = row_blob
        {
            let local_start = global_start.saturating_sub(row_range.start).min(line.len());
            let local_end = global_end.saturating_sub(row_range.start).min(line.len());
            if local_start < local_end {
                let (services, _) = painter.services_and_scene();
                st.selection_rect_scratch.clear();
                services.text().selection_rects(
                    blob,
                    (local_start, local_end),
                    &mut st.selection_rect_scratch,
                );

                for local_rect in st.selection_rect_scratch.iter().copied() {
                    let x0 = local_rect.origin.x.0;
                    let x1 = x0 + local_rect.size.width.0;
                    let x0 = x0.clamp(0.0, rect.size.width.0);
                    let x1 = x1.clamp(0.0, rect.size.width.0);
                    let w = (x1 - x0).max(0.0);
                    if w <= 0.0 {
                        continue;
                    }
                    let sel_rect = Rect::new(
                        fret_core::Point::new(Px(rect.origin.x.0 + x0), rect.origin.y),
                        Size::new(Px(w), row_h),
                    );
                    painter.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: sel_rect,
                        background: selection_bg,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                    drew_selection = true;
                }
            }
        }
    }

    if !row_geom.caret_stops.is_empty() {
        // Draw selection using caret stops so that selection geometry matches hit-testing.
        if !drew_selection && !sel.is_empty() {
            let global_start = sel.start.max(row_range.start).min(row_range.end);
            let global_end = sel.end.max(row_range.start).min(row_range.end);
            if global_start < global_end {
                let local_start = global_start.saturating_sub(row_range.start);
                let local_end = global_end.saturating_sub(row_range.start);
                let mut ranges: Vec<(usize, usize)> = Vec::new();
                if let Some(preedit) = row_geom.preedit {
                    if local_end <= preedit.insert_at {
                        ranges.push((local_start, local_end));
                    } else if local_start >= preedit.insert_at {
                        ranges.push((
                            local_start.saturating_add(preedit.preedit_len),
                            local_end.saturating_add(preedit.preedit_len),
                        ));
                    } else {
                        ranges.push((local_start, preedit.insert_at));
                        ranges.push((
                            preedit.insert_at.saturating_add(preedit.preedit_len),
                            local_end.saturating_add(preedit.preedit_len),
                        ));
                    }
                } else {
                    ranges.push((local_start, local_end));
                }

                for (a, b) in ranges {
                    if a >= b {
                        continue;
                    }
                    let x0 = caret_x_for_index(&row_geom.caret_stops, a);
                    let x1 = caret_x_for_index(&row_geom.caret_stops, b);
                    if x0.0 == x1.0 {
                        continue;
                    }
                    let x = Px(rect.origin.x.0 + x0.0.min(x1.0));
                    let w = Px((x1.0 - x0.0).abs());
                    let sel_rect =
                        Rect::new(fret_core::Point::new(x, rect.origin.y), Size::new(w, row_h));
                    painter.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: sel_rect,
                        background: selection_bg,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }
        }

        // Draw caret using caret stops so that caret geometry matches hit-testing and IME anchoring.
        if st.selection.is_caret() {
            let caret = st.selection.caret().min(st.buffer.len_bytes());
            let caret_pt = st.display_map.byte_to_display_point(&st.buffer, caret);
            if caret_pt.row == row {
                let mut local = caret.saturating_sub(row_range.start);
                if let Some(preedit) = &st.preedit
                    && row_geom.preedit.is_some()
                {
                    local = local.saturating_add(preedit_cursor_offset_bytes(preedit));
                }
                let x0 = caret_x_for_index(&row_geom.caret_stops, local);
                let caret_rect = Rect::new(
                    fret_core::Point::new(Px(rect.origin.x.0 + x0.0), rect.origin.y),
                    Size::new(Px(1.0), row_h),
                );
                painter.scene().push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: caret_rect,
                    background: caret_color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }
    } else {
        // Fallback to the MVP monospace heuristic if caret stops are unavailable.
        if !drew_selection && !sel.is_empty() {
            let start_pt = st.display_map.byte_to_display_point(&st.buffer, sel.start);
            let end_pt = st.display_map.byte_to_display_point(&st.buffer, sel.end);
            if row >= start_pt.row && row <= end_pt.row {
                let line_cols = line.chars().count();
                let start_col = if row == start_pt.row { start_pt.col } else { 0 };
                let end_col = if row == end_pt.row {
                    end_pt.col
                } else {
                    line_cols
                };
                if start_col != end_col {
                    let x0 = Px(rect.origin.x.0 + start_col as f32 * cell_w.0);
                    let x1 = Px(rect.origin.x.0 + end_col as f32 * cell_w.0);
                    let x = Px(x0.0.min(x1.0));
                    let w = Px((x1.0 - x0.0).abs());
                    let sel_rect =
                        Rect::new(fret_core::Point::new(x, rect.origin.y), Size::new(w, row_h));
                    painter.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: sel_rect,
                        background: selection_bg,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }
        }

        if st.selection.is_caret() {
            let caret = st.selection.caret().min(st.buffer.len_bytes());
            let caret_pt = st.display_map.byte_to_display_point(&st.buffer, caret);
            if caret_pt.row == row {
                let mut col = caret_pt.col;
                if let Some(preedit) = &st.preedit {
                    col = col.saturating_add(preedit_cursor_offset_cols(preedit));
                }
                let x = Px(rect.origin.x.0 + col as f32 * cell_w.0);
                let caret_rect = Rect::new(
                    fret_core::Point::new(x, rect.origin.y),
                    Size::new(Px(1.0), row_h),
                );
                painter.scene().push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: caret_rect,
                    background: caret_color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }
    }

    // Cache row geometry for pointer hit-testing / IME cursor-area anchoring in event handlers.
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    if st.row_geom_cache_rev != rev || st.row_geom_cache_wrap_cols != wrap_cols {
        st.row_geom_cache_rev = rev;
        st.row_geom_cache_wrap_cols = wrap_cols;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
    }

    st.row_geom_cache_tick = st.row_geom_cache_tick.saturating_add(1);
    let tick = st.row_geom_cache_tick;
    st.row_geom_cache.insert(row, (row_geom, tick));
    st.row_geom_cache_queue.push_back((row, tick));
    while st.row_geom_cache.len() > text_cache_max_entries {
        let Some((victim, victim_tick)) = st.row_geom_cache_queue.pop_front() else {
            break;
        };
        let remove = st
            .row_geom_cache
            .get(&victim)
            .is_some_and(|(_, last_used)| *last_used == victim_tick);
        if remove {
            st.row_geom_cache.remove(&victim);
        }
    }
}

fn cached_row_text(st: &mut CodeEditorState, row: usize, max_entries: usize) -> Arc<str> {
    st.cache_stats.row_text_get_calls = st.cache_stats.row_text_get_calls.saturating_add(1);
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    if st.row_text_cache_rev != rev || st.row_text_cache_wrap_cols != wrap_cols {
        st.row_text_cache_rev = rev;
        st.row_text_cache_wrap_cols = wrap_cols;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.cache_stats.row_text_resets = st.cache_stats.row_text_resets.saturating_add(1);
    }

    st.row_text_cache_tick = st.row_text_cache_tick.saturating_add(1);
    let tick = st.row_text_cache_tick;

    if let Some((text, last_used)) = st.row_text_cache.get_mut(&row) {
        *last_used = tick;
        st.row_text_cache_queue.push_back((row, tick));
        st.cache_stats.row_text_hits = st.cache_stats.row_text_hits.saturating_add(1);
        return Arc::clone(text);
    }
    st.cache_stats.row_text_misses = st.cache_stats.row_text_misses.saturating_add(1);

    let range = st.display_map.display_row_byte_range(&st.buffer, row);
    let text: Arc<str> = st.buffer.slice_to_string(range).unwrap_or_default().into();
    st.row_text_cache.insert(row, (Arc::clone(&text), tick));
    st.row_text_cache_queue.push_back((row, tick));

    while st.row_text_cache.len() > max_entries {
        let Some((victim, victim_tick)) = st.row_text_cache_queue.pop_front() else {
            break;
        };
        let remove = st
            .row_text_cache
            .get(&victim)
            .is_some_and(|(_, last_used)| *last_used == victim_tick);
        if remove {
            st.row_text_cache.remove(&victim);
            st.cache_stats.row_text_evictions = st.cache_stats.row_text_evictions.saturating_add(1);
        }
    }

    text
}

fn materialize_preedit_rich_text(
    line: Arc<str>,
    caret_in_line: usize,
    preedit: &PreeditState,
    fg: Color,
    selection_bg: Color,
) -> AttributedText {
    let caret_in_line = caret_in_line.min(line.len());
    let before = line.get(..caret_in_line).unwrap_or("");
    let after = line.get(caret_in_line..).unwrap_or("");

    let mut display = String::with_capacity(before.len() + preedit.text.len() + after.len());
    display.push_str(before);
    display.push_str(preedit.text.as_str());
    display.push_str(after);

    let before_len = before.len();
    let preedit_len = preedit.text.len();
    let after_len = after.len();

    let underline = UnderlineStyle {
        color: Some(fg),
        style: DecorationLineStyle::Solid,
    };

    let cursor_range = preedit.cursor.and_then(|(a, b)| {
        let a = fret_code_editor_view::clamp_to_char_boundary(preedit.text.as_str(), a)
            .min(preedit.text.len());
        let b = fret_code_editor_view::clamp_to_char_boundary(preedit.text.as_str(), b)
            .min(preedit.text.len());
        if a == b {
            return None;
        }
        Some(if a <= b { a..b } else { b..a })
    });

    let mut spans: Vec<TextSpan> = Vec::new();
    if before_len > 0 {
        spans.push(TextSpan::new(before_len));
    }

    if let Some(cursor) = cursor_range {
        let pre_a = cursor.start.min(preedit_len);
        let pre_b = cursor.end.min(preedit_len);
        if pre_a > 0 {
            spans.push(TextSpan {
                len: pre_a,
                shaping: Default::default(),
                paint: TextPaintStyle {
                    underline: Some(underline.clone()),
                    ..Default::default()
                },
            });
        }
        spans.push(TextSpan {
            len: pre_b.saturating_sub(pre_a),
            shaping: Default::default(),
            paint: TextPaintStyle {
                bg: Some(selection_bg),
                underline: Some(underline.clone()),
                ..Default::default()
            },
        });
        if pre_b < preedit_len {
            spans.push(TextSpan {
                len: preedit_len - pre_b,
                shaping: Default::default(),
                paint: TextPaintStyle {
                    underline: Some(underline),
                    ..Default::default()
                },
            });
        }
    } else {
        spans.push(TextSpan {
            len: preedit_len,
            shaping: Default::default(),
            paint: TextPaintStyle {
                underline: Some(underline),
                ..Default::default()
            },
        });
    }

    if after_len > 0 {
        spans.push(TextSpan::new(after_len));
    }

    AttributedText::new(display, spans)
}

#[cfg(feature = "syntax")]
const SYNTAX_CACHE_LOOKBACK_ROWS: usize = 64;

#[cfg(feature = "syntax")]
const SYNTAX_CACHE_LOOKAHEAD_ROWS: usize = 64;

#[cfg(feature = "syntax")]
fn invalidate_syntax_row_cache_for_delta(
    st: &mut CodeEditorState,
    delta: fret_code_editor_buffer::BufferDelta,
) {
    // Keep the revision in sync so cached-row requests don't force a full cache clear.
    st.syntax_row_cache_rev = delta.after;
    if st.syntax_row_cache.is_empty() {
        return;
    }

    let start = delta.lines.start.saturating_sub(SYNTAX_CACHE_LOOKBACK_ROWS);
    let line_count = st.buffer.line_count();
    let before_len = st.syntax_row_cache.len();

    if delta.lines.old_count != delta.lines.new_count {
        // Line count changed: row indices at/after the edit point may have shifted.
        // Keep only entries that are strictly before the invalidation start.
        st.syntax_row_cache.retain(|row, _| *row < start);
    } else {
        let affected_end = delta
            .lines
            .start
            .saturating_add(delta.lines.new_count.saturating_sub(1));
        let end = affected_end
            .saturating_add(SYNTAX_CACHE_LOOKAHEAD_ROWS)
            .min(line_count.saturating_sub(1));
        st.syntax_row_cache
            .retain(|row, _| *row < start || *row > end);
    }

    let after_len = st.syntax_row_cache.len();
    if after_len != before_len {
        let removed = before_len.saturating_sub(after_len);
        st.cache_stats.syntax_evictions = st
            .cache_stats
            .syntax_evictions
            .saturating_add(removed as u64);
        rebuild_syntax_row_cache_queue(st);
    }
}

#[cfg(feature = "syntax")]
fn rebuild_syntax_row_cache_queue(st: &mut CodeEditorState) {
    let mut entries: Vec<(usize, u64)> = st
        .syntax_row_cache
        .iter()
        .map(|(row, (_, tick))| (*row, *tick))
        .collect();
    entries.sort_by_key(|(_, tick)| *tick);
    st.syntax_row_cache_queue = entries.into();
}

#[cfg(feature = "syntax")]
fn cached_row_syntax_spans(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> Arc<[SyntaxSpan]> {
    st.cache_stats.syntax_get_calls = st.cache_stats.syntax_get_calls.saturating_add(1);
    let rev = st.buffer.revision();
    if st.syntax_row_cache_rev != rev || st.syntax_row_cache_language != st.language {
        st.syntax_row_cache_rev = rev;
        st.syntax_row_cache_language = st.language.clone();
        st.syntax_row_cache_tick = 0;
        st.syntax_row_cache.clear();
        st.syntax_row_cache_queue.clear();
        st.cache_stats.syntax_resets = st.cache_stats.syntax_resets.saturating_add(1);
    }

    st.syntax_row_cache_tick = st.syntax_row_cache_tick.saturating_add(1);
    let tick = st.syntax_row_cache_tick;

    if let Some((spans, last_used)) = st.syntax_row_cache.get_mut(&row) {
        *last_used = tick;
        st.syntax_row_cache_queue.push_back((row, tick));
        st.cache_stats.syntax_hits = st.cache_stats.syntax_hits.saturating_add(1);
        return Arc::clone(spans);
    }
    st.cache_stats.syntax_misses = st.cache_stats.syntax_misses.saturating_add(1);

    let language = st.language.clone();
    let Some(language) = language.as_deref() else {
        return Arc::<[SyntaxSpan]>::from([]);
    };

    let line_count = st.buffer.line_count();
    if line_count == 0 {
        return Arc::<[SyntaxSpan]>::from([]);
    }

    let chunk_start = row.saturating_sub(SYNTAX_CACHE_LOOKBACK_ROWS);
    let chunk_end = row
        .saturating_add(SYNTAX_CACHE_LOOKAHEAD_ROWS)
        .min(line_count.saturating_sub(1));
    populate_syntax_row_cache_for_chunk(st, chunk_start, chunk_end, language, max_entries, tick);

    st.syntax_row_cache
        .get(&row)
        .map(|(spans, _)| Arc::clone(spans))
        .unwrap_or_else(|| Arc::<[SyntaxSpan]>::from([]))
}

#[cfg(feature = "syntax")]
fn populate_syntax_row_cache_for_chunk(
    st: &mut CodeEditorState,
    chunk_start: usize,
    chunk_end: usize,
    language: &str,
    max_entries: usize,
    tick: u64,
) {
    let line_count = st.buffer.line_count();
    if line_count == 0 || chunk_start > chunk_end {
        return;
    }

    let start_byte = st
        .buffer
        .line_start(chunk_start)
        .unwrap_or(0)
        .min(st.buffer.len_bytes());
    let end_byte = if chunk_end.saturating_add(1) < line_count {
        st.buffer
            .line_start(chunk_end.saturating_add(1))
            .unwrap_or(st.buffer.len_bytes())
            .min(st.buffer.len_bytes())
    } else {
        st.buffer.len_bytes()
    };

    if start_byte >= end_byte {
        return;
    }

    let Some(slice) = st.buffer.slice_to_string(start_byte..end_byte) else {
        return;
    };

    let Ok(spans) = fret_syntax::highlight(slice.as_str(), language) else {
        return;
    };

    let mut row_ranges = Vec::with_capacity(chunk_end - chunk_start + 1);
    for row in chunk_start..=chunk_end {
        row_ranges.push(st.buffer.line_byte_range(row).unwrap_or(0..0));
    }

    let mut per_row = vec![Vec::<SyntaxSpan>::new(); row_ranges.len()];
    for span in spans {
        let Some(highlight) = span.highlight else {
            continue;
        };

        let global_start = start_byte.saturating_add(span.range.start);
        let global_end = start_byte.saturating_add(span.range.end);
        if global_start >= global_end {
            continue;
        }

        let global_end_for_row = global_end.saturating_sub(1);
        let start_row = st.buffer.line_index_at_byte(global_start);
        let end_row = st.buffer.line_index_at_byte(global_end_for_row);

        for row in start_row..=end_row {
            if row < chunk_start || row > chunk_end {
                continue;
            }
            let row_idx = row - chunk_start;
            let row_range = &row_ranges[row_idx];
            let inter_start = global_start.max(row_range.start);
            let inter_end = global_end.min(row_range.end);
            if inter_start >= inter_end {
                continue;
            }
            per_row[row_idx].push(SyntaxSpan {
                range: (inter_start - row_range.start)..(inter_end - row_range.start),
                highlight,
            });
        }
    }

    for (i, spans) in per_row.into_iter().enumerate() {
        let row = chunk_start + i;

        let mut spans = spans;
        spans.sort_by_key(|s| s.range.start);
        spans.dedup_by(|a, b| a.range == b.range && a.highlight == b.highlight);

        let mut merged: Vec<SyntaxSpan> = Vec::new();
        for span in spans {
            if let Some(last) = merged.last_mut()
                && last.highlight == span.highlight
                && last.range.end == span.range.start
            {
                last.range.end = span.range.end;
                continue;
            }
            merged.push(span);
        }

        let spans: Arc<[SyntaxSpan]> = Arc::from(merged);
        st.syntax_row_cache.insert(row, (Arc::clone(&spans), tick));
        st.syntax_row_cache_queue.push_back((row, tick));

        while st.syntax_row_cache.len() > max_entries {
            let Some((victim, victim_tick)) = st.syntax_row_cache_queue.pop_front() else {
                break;
            };
            let remove = st
                .syntax_row_cache
                .get(&victim)
                .is_some_and(|(_, last_used)| *last_used == victim_tick);
            if remove {
                st.syntax_row_cache.remove(&victim);
                st.cache_stats.syntax_evictions = st.cache_stats.syntax_evictions.saturating_add(1);
            }
        }
    }
}

#[cfg(feature = "syntax")]
fn syntax_color(theme: &fret_ui::Theme, highlight: &str) -> Option<Color> {
    let mut key = String::with_capacity("color.syntax.".len() + highlight.len());
    key.push_str("color.syntax.");
    key.push_str(highlight);
    if let Some(c) = theme.color_by_key(key.as_str()) {
        return Some(c);
    }

    let fallback = highlight.split('.').next().unwrap_or(highlight);
    if fallback != highlight {
        let mut key = String::with_capacity("color.syntax.".len() + fallback.len());
        key.push_str("color.syntax.");
        key.push_str(fallback);
        if let Some(c) = theme.color_by_key(key.as_str()) {
            return Some(c);
        }
    }

    match fallback {
        "comment" => Some(theme.color_required("muted-foreground")),
        "keyword" | "operator" => Some(theme.color_required("primary")),
        "property" | "variable" => Some(theme.color_required("foreground")),
        "punctuation" => Some(theme.color_required("muted-foreground")),

        "string" => Some(theme.color_required("foreground")),
        "number" | "boolean" | "constant" => Some(theme.color_required("primary")),
        "type" | "constructor" | "function" => Some(theme.color_required("foreground")),
        _ => None,
    }
}

#[cfg(feature = "syntax")]
fn materialize_row_rich_text(
    theme: &fret_ui::Theme,
    line: Arc<str>,
    spans: &[SyntaxSpan],
) -> AttributedText {
    let mut out: Vec<TextSpan> = Vec::new();
    let mut cursor = 0usize;
    let max = line.len();

    for span in spans {
        let start = span.range.start.min(max);
        let end = span.range.end.min(max);
        if start >= end || start < cursor {
            continue;
        }

        if start > cursor {
            out.push(TextSpan {
                len: start - cursor,
                ..Default::default()
            });
        }

        let fg = syntax_color(theme, span.highlight);
        out.push(TextSpan {
            len: end - start,
            shaping: Default::default(),
            paint: TextPaintStyle {
                fg,
                ..Default::default()
            },
        });
        cursor = end;
    }

    if cursor < max {
        out.push(TextSpan {
            len: max - cursor,
            ..Default::default()
        });
    }

    AttributedText::new(line, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestHost {
        models: fret_runtime::ModelStore,
        next_timer: u64,
        next_clipboard: u64,
    }

    impl fret_ui::action::UiActionHost for TestHost {
        fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, _effect: fret_runtime::Effect) {}

        fn request_redraw(&mut self, _window: fret_core::AppWindowId) {}

        fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
            self.next_timer = self.next_timer.saturating_add(1);
            fret_runtime::TimerToken(self.next_timer)
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.next_clipboard = self.next_clipboard.saturating_add(1);
            fret_runtime::ClipboardToken(self.next_clipboard)
        }
    }

    impl fret_ui::action::UiFocusActionHost for TestHost {
        fn request_focus(&mut self, _target: fret_ui::GlobalElementId) {}
    }

    #[test]
    fn replace_buffer_resets_state() {
        let handle = CodeEditorHandle::new("hello");

        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: 1,
                focus: 3,
            };
            st.dragging = true;
            st.drag_pointer = Some(fret_core::PointerId(1));
            st.row_text_cache.insert(0, ("hello".into(), 1));
            st.row_text_cache_queue.push_back((0, 1));
            st.row_geom_cache.insert(
                0,
                (
                    RowGeom {
                        row_range: 0..5,
                        caret_stops: vec![(0, Px(0.0))],
                        preedit: None,
                    },
                    1,
                ),
            );
            st.row_geom_cache_queue.push_back((0, 1));
        }

        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, "world".to_string()).unwrap();
        handle.replace_buffer(buffer);

        let st = handle.state.borrow();
        assert_eq!(st.buffer.text_string(), "world");
        assert_eq!(st.selection, Selection::default());
        assert_eq!(st.preedit, None);
        assert!(st.undo_group.is_none());
        assert!(!st.dragging);
        assert_eq!(st.drag_pointer, None);
        assert_eq!(st.row_text_cache.len(), 0);
        assert_eq!(st.row_text_cache_queue.len(), 0);
        assert_eq!(st.row_geom_cache.len(), 0);
        assert_eq!(st.row_geom_cache_queue.len(), 0);
    }

    #[test]
    fn replace_buffer_preserves_text_boundary_mode() {
        let handle = CodeEditorHandle::new("hello");
        handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);

        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, "world".to_string()).unwrap();
        handle.replace_buffer(buffer);

        assert_eq!(handle.text_boundary_mode(), TextBoundaryMode::UnicodeWord);
    }

    #[test]
    fn caret_stops_hit_test_picks_nearest_stop() {
        let stops = vec![(0, Px(0.0)), (1, Px(10.0)), (2, Px(20.0)), (3, Px(30.0))];
        assert_eq!(hit_test_index_from_caret_stops(&stops, Px(-5.0)), 0);
        assert_eq!(hit_test_index_from_caret_stops(&stops, Px(0.0)), 0);
        assert_eq!(hit_test_index_from_caret_stops(&stops, Px(4.9)), 0);
        assert_eq!(hit_test_index_from_caret_stops(&stops, Px(5.1)), 1);
        assert_eq!(hit_test_index_from_caret_stops(&stops, Px(14.9)), 1);
        assert_eq!(hit_test_index_from_caret_stops(&stops, Px(15.1)), 2);
        assert_eq!(hit_test_index_from_caret_stops(&stops, Px(999.0)), 3);
    }

    #[test]
    fn map_row_local_to_buffer_byte_snaps_inside_preedit() {
        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, "hello".to_string()).unwrap();
        let geom = RowGeom {
            row_range: 0..buffer.len_bytes(),
            caret_stops: Vec::new(),
            preedit: Some(RowPreeditMapping {
                insert_at: 2,
                preedit_len: 2,
            }),
        };

        // Before the injection point maps 1:1.
        assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 0), 0);
        assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 2), 2);

        // Inside the injected preedit snaps to the injection point.
        assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 3), 2);

        // After the injected preedit shifts by `preedit_len`.
        assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 4), 2);
        assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 5), 3);
    }

    #[test]
    fn caret_preferred_x_is_preserved_across_vertical_moves() {
        let handle = CodeEditorHandle::new("aaaa\nbbbb\ncccc");
        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: 2,
                focus: 2,
            };

            // Synthetic caret stops: 10px per byte.
            st.row_geom_cache.insert(
                0,
                (
                    RowGeom {
                        row_range: 0..4,
                        caret_stops: vec![
                            (0, Px(0.0)),
                            (1, Px(10.0)),
                            (2, Px(20.0)),
                            (3, Px(30.0)),
                            (4, Px(40.0)),
                        ],
                        preedit: None,
                    },
                    1,
                ),
            );
            st.row_geom_cache.insert(
                1,
                (
                    RowGeom {
                        row_range: 5..9,
                        caret_stops: vec![
                            (0, Px(0.0)),
                            (1, Px(10.0)),
                            (2, Px(20.0)),
                            (3, Px(30.0)),
                            (4, Px(40.0)),
                        ],
                        preedit: None,
                    },
                    1,
                ),
            );
            st.row_geom_cache.insert(
                2,
                (
                    RowGeom {
                        row_range: 10..14,
                        caret_stops: vec![
                            (0, Px(0.0)),
                            (1, Px(10.0)),
                            (2, Px(20.0)),
                            (3, Px(30.0)),
                            (4, Px(40.0)),
                        ],
                        preedit: None,
                    },
                    1,
                ),
            );

            move_caret_vertical(&mut st, 1, false, Px(8.0));
            assert_eq!(st.selection.caret(), 7, "row 1, local index 2");
            assert_eq!(st.caret_preferred_x, Some(Px(20.0)));

            move_caret_vertical(&mut st, 1, false, Px(8.0));
            assert_eq!(st.selection.caret(), 12, "row 2, local index 2");
            assert_eq!(st.caret_preferred_x, Some(Px(20.0)));
        }
    }

    #[test]
    fn row_text_cache_stats_tracks_hits_and_misses() {
        let handle = CodeEditorHandle::new("hello\nworld");
        handle.reset_cache_stats();

        {
            let mut st = handle.state.borrow_mut();
            assert_eq!(st.cache_stats.row_text_get_calls, 0);
            assert_eq!(st.cache_stats.row_text_hits, 0);
            assert_eq!(st.cache_stats.row_text_misses, 0);

            let a = cached_row_text(&mut st, 0, 8);
            let b = cached_row_text(&mut st, 0, 8);

            assert_eq!(a.as_ref(), "hello");
            assert_eq!(b.as_ref(), "hello");
            assert_eq!(st.cache_stats.row_text_get_calls, 2);
            assert_eq!(st.cache_stats.row_text_hits, 1);
            assert_eq!(st.cache_stats.row_text_misses, 1);
        }
    }

    #[test]
    fn ctrl_page_down_bubbles_and_keeps_preedit() {
        let handle = CodeEditorHandle::new("hello\nworld");
        let preedit = PreeditState {
            text: "世界".to_string(),
            cursor: Some((0, "世".len())),
        };
        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: 2,
                focus: 2,
            };
            st.preedit = Some(preedit.clone());
        }

        let mut host = TestHost::default();
        let action_cx = ActionCx {
            window: fret_core::AppWindowId::default(),
            target: fret_ui::GlobalElementId(0),
        };
        let scroll = fret_ui::scroll::ScrollHandle::default();
        let cell_w = Cell::new(Px(10.0));

        let handled = handle_key_down(
            &mut host,
            action_cx,
            &handle.state,
            Px(16.0),
            &scroll,
            &cell_w,
            KeyCode::PageDown,
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );

        assert!(!handled);
        let st = handle.state.borrow();
        assert_eq!(st.preedit, Some(preedit));
        assert_eq!(
            st.selection,
            Selection {
                anchor: 2,
                focus: 2
            }
        );
    }

    #[test]
    fn caret_rect_offsets_for_preedit_cursor() {
        let handle = CodeEditorHandle::new("hello");
        let preedit = PreeditState {
            text: "ab".to_string(),
            cursor: Some((0, 2)),
        };
        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: 0,
                focus: 0,
            };
            st.preedit = Some(preedit.clone());
        }

        let scroll = fret_ui::scroll::ScrollHandle::default();
        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(500.0)),
        );

        let st = handle.state.borrow();
        let rect =
            caret_rect_for_selection(&st, Px(20.0), Px(10.0), bounds, &scroll).expect("caret rect");

        assert_eq!(rect.origin.x, Px(20.0), "2 cols * 10px");
        assert_eq!(rect.origin.y, Px(0.0));
    }

    #[test]
    fn preedit_rich_text_inserts_and_underlines() {
        let preedit = PreeditState {
            text: "世界".to_string(),
            cursor: Some((0, "世".len())),
        };
        let fg = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let selection_bg = Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        };

        let rich = materialize_preedit_rich_text("hello".into(), 2, &preedit, fg, selection_bg);
        assert_eq!(rich.text.as_ref(), "he世界llo");
        assert!(rich.is_valid());
        assert!(
            rich.spans.iter().any(|s| s.paint.underline.is_some()),
            "expected preedit spans to be underlined"
        );
        assert!(
            rich.spans.iter().any(|s| s.paint.bg.is_some()),
            "expected cursor range to be highlighted"
        );
    }

    #[test]
    fn a11y_window_maps_offsets_back_to_buffer_selection() {
        let handle = CodeEditorHandle::new("hello 😀 world");
        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: "hello 😀 ".len(),
                focus: "hello 😀 ".len(),
            };
            st.preedit = None;
        }

        let st = handle.state.borrow();
        let (value, selection, composition) = a11y_composed_text_window(&st);
        assert_eq!(composition, None);
        assert_eq!(value.as_str(), "hello 😀 world");
        assert_eq!(
            selection,
            Some(("hello 😀 ".len() as u32, "hello 😀 ".len() as u32))
        );

        let text_len = st.buffer.len_bytes();
        let caret = st
            .buffer
            .clamp_to_char_boundary_left(st.selection.caret().min(text_len));
        let (start, end) = a11y_text_window_bounds(&st.buffer, caret);
        assert_eq!(start, 0);
        assert_eq!(end, text_len);

        let anchor = 0u32;
        let focus = u32::try_from("hello".len()).unwrap();
        let new_anchor = map_a11y_offset_to_buffer(&st.buffer, start, end, anchor);
        let new_focus = map_a11y_offset_to_buffer(&st.buffer, start, end, focus);
        assert_eq!(new_anchor, 0);
        assert_eq!(new_focus, "hello".len());
    }

    #[test]
    fn move_caret_vertical_clamps_in_display_row_space_when_wrapped() {
        let handle = CodeEditorHandle::new("abcd\nef");
        handle.set_soft_wrap_cols(Some(2));

        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };

        // Row 0 col 0 -> Down => row 1 col 0 (within the wrapped "abcd").
        move_caret_vertical(&mut st, 1, false, Px(10.0));
        assert_eq!(st.selection.caret(), 2);

        // Row 1 col 0 -> Down => row 2 col 0 (next logical line "ef").
        move_caret_vertical(&mut st, 1, false, Px(10.0));
        assert_eq!(st.selection.caret(), 5);

        // Row 2 is the last display row; another Down should clamp.
        move_caret_vertical(&mut st, 1, false, Px(10.0));
        assert_eq!(st.selection.caret(), 5);
    }

    #[test]
    fn apply_and_record_edit_refreshes_display_map_only_when_needed() {
        let handle = CodeEditorHandle::new("ab\nc");

        {
            let mut st = handle.state.borrow_mut();
            assert_eq!(st.display_wrap_cols, None);
            assert_eq!(st.display_map.row_count(), 2);

            // No newline, no wrap => row_count should remain correct without forcing a refresh.
            apply_and_record_edit(
                &mut st,
                UndoGroupKind::Typing,
                Edit::Insert {
                    at: 0,
                    text: "x".to_string(),
                },
                Selection {
                    anchor: 1,
                    focus: 1,
                },
            )
            .expect("apply edit");
            assert_eq!(st.buffer.line_count(), 2);
            assert_eq!(st.display_map.row_count(), 2);

            // Newline => line count changes, so the map must refresh.
            let insert_at = st.buffer.text_string().find('\n').unwrap_or(0);
            apply_and_record_edit(
                &mut st,
                UndoGroupKind::Typing,
                Edit::Insert {
                    at: insert_at,
                    text: "\n".to_string(),
                },
                Selection {
                    anchor: insert_at + 1,
                    focus: insert_at + 1,
                },
            )
            .expect("apply edit");
            assert_eq!(st.buffer.line_count(), 3);
            assert_eq!(st.display_map.row_count(), 3);
        }

        // With wrap enabled, edits can change display rows even if line count is stable.
        let handle = CodeEditorHandle::new("ab");
        handle.set_soft_wrap_cols(Some(2));
        {
            let mut st = handle.state.borrow_mut();
            assert_eq!(st.display_map.row_count(), 1);

            apply_and_record_edit(
                &mut st,
                UndoGroupKind::Typing,
                Edit::Insert {
                    at: 2,
                    text: "c".to_string(),
                },
                Selection {
                    anchor: 3,
                    focus: 3,
                },
            )
            .expect("apply edit");
            assert_eq!(st.display_map.row_count(), 2);
        }
    }

    #[test]
    fn home_end_move_within_wrapped_display_rows() {
        let handle = CodeEditorHandle::new("abcd\nef");
        handle.set_soft_wrap_cols(Some(2));

        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 3,
            focus: 3,
        };

        // caret at byte 3 is in the second wrapped row ("cd"): row start is byte 2, end is byte 4.
        move_caret_home_end(&mut st, true, false, false);
        assert_eq!(st.selection.caret(), 2);

        st.selection = Selection {
            anchor: 3,
            focus: 3,
        };
        move_caret_home_end(&mut st, false, false, false);
        assert_eq!(st.selection.caret(), 4);

        // Ctrl+Home/End should clamp to document bounds.
        st.selection = Selection {
            anchor: 3,
            focus: 3,
        };
        move_caret_home_end(&mut st, true, true, false);
        assert_eq!(st.selection.caret(), 0);

        st.selection = Selection {
            anchor: 3,
            focus: 3,
        };
        move_caret_home_end(&mut st, false, true, false);
        assert_eq!(st.selection.caret(), st.buffer.len_bytes());
    }

    #[test]
    fn page_down_moves_by_viewport_rows_and_scrolls() {
        let handle = CodeEditorHandle::new("abcd\nefgh\nijkl\nmnop\nqrst\n");
        handle.set_soft_wrap_cols(Some(2));

        let scroll = fret_ui::scroll::ScrollHandle::default();
        let row_h = Px(10.0);
        scroll.set_viewport_size(Size::new(Px(100.0), Px(25.0))); // 2 rows
        scroll.set_content_size(Size::new(Px(100.0), Px(10_000.0)));

        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };

        move_caret_page(&mut st, 1, false, row_h, &scroll, Px(10.0));

        let expected = st
            .display_map
            .display_point_to_byte(&st.buffer, DisplayPoint::new(2, 0));
        assert_eq!(st.selection.caret(), expected);
        assert_eq!(scroll.offset().y, Px(20.0));
    }

    #[test]
    fn delete_word_backward_removes_previous_word() {
        let handle = CodeEditorHandle::new("hello world");
        handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);

        let mut st = handle.state.borrow_mut();
        let end = st.buffer.len_bytes();
        st.selection = Selection {
            anchor: end,
            focus: end,
        };

        delete_word_backward(&mut st);
        assert_eq!(st.buffer.text_string(), "hello ");
        assert_eq!(st.selection.caret(), "hello ".len());
    }

    #[test]
    fn delete_word_forward_removes_next_word() {
        let handle = CodeEditorHandle::new("hello world");
        handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);

        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };

        delete_word_forward(&mut st);
        assert_eq!(st.buffer.text_string(), " world");
        assert_eq!(st.selection.caret(), 0);
    }

    #[cfg(feature = "syntax-rust")]
    #[test]
    fn rust_syntax_spans_are_materialized_for_rows() {
        let handle = CodeEditorHandle::new("fn main() {\n    let x = 1;\n}\n");
        handle.set_language(Some(Arc::<str>::from("rust")));

        let mut st = handle.state.borrow_mut();
        let line_count = st.buffer.line_count();
        assert!(line_count > 0);

        let mut any_highlight = false;
        for row in 0..line_count {
            let spans = cached_row_syntax_spans(&mut st, row, 256);
            if !spans.is_empty() {
                any_highlight = true;
                break;
            }
        }
        assert!(
            any_highlight,
            "expected at least one highlighted span for rust"
        );
    }

    #[cfg(feature = "syntax-rust")]
    #[test]
    fn syntax_cache_invalidation_preserves_far_rows_on_inline_edit() {
        let mut text = String::new();
        for _ in 0..200 {
            text.push_str("fn main() {}\n");
        }

        let handle = CodeEditorHandle::new(text.as_str());
        handle.set_language(Some(Arc::<str>::from("rust")));

        let mut st = handle.state.borrow_mut();
        let max_entries = 4096;
        let _ = cached_row_syntax_spans(&mut st, 0, max_entries);
        let _ = cached_row_syntax_spans(&mut st, 150, max_entries);
        assert!(
            st.syntax_row_cache.contains_key(&150),
            "expected far-row cache entries to be populated"
        );

        apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            Edit::Insert {
                at: 0,
                text: "x".to_string(),
            },
            Selection {
                anchor: 1,
                focus: 1,
            },
        )
        .expect("apply edit");

        assert!(
            st.syntax_row_cache.contains_key(&150),
            "expected far-row cache entries to survive inline edit invalidation"
        );
        assert!(
            !st.syntax_row_cache.contains_key(&0),
            "expected near-row cache entries to be invalidated"
        );
    }
}
