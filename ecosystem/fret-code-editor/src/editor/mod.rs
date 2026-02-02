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
    AttributedText, CaretAffinity, Color, Corners, DecorationLineStyle, DrawOrder, Edges, FontId,
    KeyCode, Modifiers, MouseButton, Px, Rect, SceneOp, Size, TextOverflow, TextPaintStyle,
    TextSpan, TextStyle, TextWrap, UnderlineStyle,
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
    let mut caret_rect_top = None::<Px>;
    let mut caret_rect_height = None::<Px>;
    if let Some(blob) = row_blob {
        let (services, _) = painter.services_and_scene();
        services.text().caret_stops(blob, &mut caret_stops);

        let caret_rect = services
            .text()
            .caret_rect(blob, 0, CaretAffinity::Downstream);
        if caret_rect.size.height.0 > 0.0 {
            caret_rect_top = Some(caret_rect.origin.y);
            caret_rect_height = Some(caret_rect.size.height);
        }
    }

    let row_geom = RowGeom {
        row_range: row_range.clone(),
        caret_stops,
        caret_rect_top,
        caret_rect_height,
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
                let (caret_top, caret_h) = if let (Some(top), Some(h)) =
                    (row_geom.caret_rect_top, row_geom.caret_rect_height)
                    && h.0 > 0.0
                {
                    (top, Px(h.0.min(row_h.0)))
                } else {
                    (Px(0.0), row_h)
                };
                let caret_rect = Rect::new(
                    fret_core::Point::new(
                        Px(rect.origin.x.0 + x0.0),
                        Px(rect.origin.y.0 + caret_top.0),
                    ),
                    Size::new(Px(1.0), caret_h),
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
