//! Code editor surface (UI integration) for Fret.
//!
//! This is a v1 MVP: fixed row height and a monospace "cell width" heuristic for caret/selection
//! geometry. Optional soft-wrap is supported via the view-layer `DisplayMap`.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use fret_code_editor_buffer::{DocId, Edit, TextBuffer, TextBufferTransaction, TextBufferTx};
use fret_code_editor_view::{
    DisplayMap, DisplayPoint, move_word_left, move_word_right, next_char_boundary,
    prev_char_boundary, select_word_range,
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

#[derive(Debug, Clone)]
struct CodeEditorState {
    buffer: TextBuffer,
    selection: Selection,
    preedit: Option<PreeditState>,
    text_boundary_mode: TextBoundaryMode,
    display_wrap_cols: Option<usize>,
    display_map: DisplayMap,
    undo: UndoHistory<CodeEditorTx>,
    undo_group: Option<UndoGroup>,
    dragging: bool,
    drag_pointer: Option<fret_core::PointerId>,
    last_bounds: Option<Rect>,
    row_text_cache_rev: fret_code_editor_buffer::Revision,
    row_text_cache_wrap_cols: Option<usize>,
    row_text_cache_tick: u64,
    row_text_cache: HashMap<usize, (Arc<str>, u64)>,
    row_text_cache_queue: VecDeque<(usize, u64)>,
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
                undo: UndoHistory::with_limit(512),
                undo_group: None,
                dragging: false,
                drag_pointer: None,
                last_bounds: None,
                row_text_cache_rev: fret_code_editor_buffer::Revision(0),
                row_text_cache_wrap_cols: None,
                row_text_cache_tick: 0,
                row_text_cache: HashMap::new(),
                row_text_cache_queue: VecDeque::new(),
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

    pub fn text_boundary_mode(&self) -> TextBoundaryMode {
        self.state.borrow().text_boundary_mode
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
        st.undo = UndoHistory::with_limit(512);
        st.undo_group = None;
        st.dragging = false;
        st.drag_pointer = None;
        st.last_bounds = None;
        st.refresh_display_map();
        st.row_text_cache_rev = st.buffer.revision();
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
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

                    let caret = caret_for_pointer(
                        &st.buffer,
                        &st.display_map,
                        row,
                        bounds,
                        down.position,
                        cell_w,
                    );
                    match down.click_count {
                        2 => {
                            let (start, end) =
                                select_word_range(st.buffer.text(), caret, st.text_boundary_mode);
                            st.selection = Selection {
                                anchor: start,
                                focus: end,
                            };
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
                        }
                    }

                    let caret_rect = caret_rect_for_selection(
                        &st.buffer,
                        &st.display_map,
                        st.selection,
                        st.preedit.as_ref(),
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
                    let caret = caret_for_pointer(
                        &st.buffer,
                        &st.display_map,
                        row,
                        bounds,
                        mv.position,
                        cell_w,
                    );
                    st.selection.focus = caret;

                    let caret_rect = caret_rect_for_selection(
                        &st.buffer,
                        &st.display_map,
                        st.selection,
                        st.preedit.as_ref(),
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

                        let origin = fret_core::Point::new(Px(8.0), Px(offset.y.0 + 8.0));
                        painter.scene().push(SceneOp::Quad {
                            order: DrawOrder(100),
                            rect: Rect::new(origin, Size::new(Px(420.0), Px(24.0))),
                            background: overlay_bg,
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: Corners::all(Px(6.0)),
                        });

                        let label = format!(
                            "rows={}-{} offset_y={:.1}/{:.1} cache_max={}",
                            frame.visible_start,
                            frame.visible_end,
                            offset.y.0,
                            max.y.0,
                            text_cache_max_entries
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
                                max_width: Some(Px(400.0)),
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

                                let start = prev_char_boundary(st.buffer.text(), start);
                                let end = next_char_boundary(st.buffer.text(), end);
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

                        let text = st.buffer.text();
                        let caret = fret_code_editor_view::clamp_to_char_boundary(
                            text,
                            st.selection.caret().min(text.len()),
                        );
                        let (start, end) = a11y_text_window_bounds(text, caret);

                        let new_anchor = map_a11y_offset_to_buffer(text, start, end, anchor);
                        let new_focus = map_a11y_offset_to_buffer(text, start, end, focus);

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
    let text = st.buffer.text();
    let caret =
        fret_code_editor_view::clamp_to_char_boundary(text, st.selection.caret().min(text.len()));

    let (start, end) = a11y_text_window_bounds(text, caret);

    let before = text.get(start..caret).unwrap_or("");
    let after = text.get(caret..end).unwrap_or("");

    if let Some(preedit) = st.preedit.as_ref() {
        let mut display = String::with_capacity(before.len() + preedit.text.len() + after.len());
        display.push_str(before);
        display.push_str(preedit.text.as_str());
        display.push_str(after);

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
    display.push_str(before);
    display.push_str(after);

    let map = |offset: usize| -> u32 {
        let offset = offset.min(end).max(start);
        let offset = fret_code_editor_view::clamp_to_char_boundary(text, offset);
        u32::try_from(offset.saturating_sub(start)).unwrap_or(u32::MAX)
    };
    let selection = Some((map(st.selection.anchor), map(st.selection.focus)));

    (display, selection, None)
}

fn a11y_text_window_bounds(text: &str, caret: usize) -> (usize, usize) {
    let caret = fret_code_editor_view::clamp_to_char_boundary(text, caret).min(text.len());
    let start = fret_code_editor_view::clamp_to_char_boundary(
        text,
        caret.saturating_sub(A11Y_WINDOW_BYTES_BEFORE),
    );
    let end = fret_code_editor_view::clamp_to_char_boundary(
        text,
        caret
            .saturating_add(A11Y_WINDOW_BYTES_AFTER)
            .min(text.len()),
    );
    (start, end)
}

fn map_a11y_offset_to_buffer(
    text: &str,
    window_start: usize,
    window_end: usize,
    offset: u32,
) -> usize {
    let window_start = window_start.min(text.len());
    let window_end = window_end.min(text.len()).max(window_start);
    let window_len = window_end.saturating_sub(window_start);
    let offset = usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(window_len);
    let buf = window_start.saturating_add(offset).min(window_end);
    fret_code_editor_view::clamp_to_char_boundary(text, buf).min(text.len())
}

fn line_len_cols(line: &str) -> usize {
    line.chars().count()
}

fn caret_for_pointer(
    buf: &TextBuffer,
    map: &DisplayMap,
    row: usize,
    bounds: Rect,
    position: fret_core::Point,
    cell_w: Px,
) -> usize {
    let local_x = Px(position.x.0 - bounds.origin.x.0);
    let col = if cell_w.0 > 0.0 {
        (local_x.0 / cell_w.0).floor().max(0.0) as usize
    } else {
        0
    };
    map.display_point_to_byte(buf, DisplayPoint::new(row, col))
}

fn caret_rect_for_selection(
    buf: &TextBuffer,
    map: &DisplayMap,
    sel: Selection,
    preedit: Option<&PreeditState>,
    row_h: Px,
    cell_w: Px,
    bounds: Rect,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) -> Option<Rect> {
    if !sel.is_caret() {
        return None;
    }

    let caret = sel.caret().min(buf.len_bytes());
    let pt = map.byte_to_display_point(buf, caret);
    let offset = scroll_handle.offset();
    let y = Px(bounds.origin.y.0 + (pt.row as f32 * row_h.0) - offset.y.0);
    let mut col = pt.col;
    if let Some(preedit) = preedit {
        col = col.saturating_add(preedit_cursor_offset_cols(preedit));
    }
    let x = Px(bounds.origin.x.0 + col as f32 * cell_w.0);
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
    if let Some(rect) = caret_rect_for_selection(
        &st.buffer,
        &st.display_map,
        st.selection,
        st.preedit.as_ref(),
        row_h,
        cell_w,
        bounds,
        scroll_handle,
    ) {
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
    if st.preedit.is_some() {
        match key {
            KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::Backspace
            | KeyCode::Delete
            | KeyCode::Enter
            | KeyCode::Tab => {
                st.preedit = None;
            }
            _ => {}
        }
    }
    let cell_w_px = cell_w.get();

    let shift = modifiers.shift;
    let ctrl_or_meta = modifiers.ctrl || modifiers.meta;

    match key {
        KeyCode::ArrowLeft => {
            move_caret_left(&mut st, shift);
            st.undo_group = None;
        }
        KeyCode::ArrowRight => {
            move_caret_right(&mut st, shift);
            st.undo_group = None;
        }
        KeyCode::ArrowUp => {
            move_caret_vertical(&mut st, -1, shift);
            st.undo_group = None;
        }
        KeyCode::ArrowDown => {
            move_caret_vertical(&mut st, 1, shift);
            st.undo_group = None;
        }
        KeyCode::Backspace => {
            delete_backward(&mut st);
        }
        KeyCode::Delete => {
            delete_forward(&mut st);
        }
        KeyCode::Enter => {
            let _ = insert_text(&mut st, "\n");
        }
        KeyCode::Tab => {
            let _ = insert_text(&mut st, "\t");
        }
        KeyCode::KeyC if ctrl_or_meta => {
            copy_selection(host, &st);
        }
        KeyCode::KeyV if ctrl_or_meta => {
            request_paste(host, action_cx);
        }
        _ => return false,
    }

    push_caret_rect_effect(host, action_cx, &st, row_h, cell_w_px, scroll_handle);

    host.notify(action_cx);
    host.request_redraw(action_cx.window);
    true
}

fn copy_selection(host: &mut dyn UiActionHost, st: &CodeEditorState) {
    let range = st.selection.normalized();
    if range.is_empty() {
        return;
    }
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let Some(text) = st.buffer.text().get(start..end) else {
        return;
    };
    host.push_effect(Effect::ClipboardSetText {
        text: text.to_string(),
    });
}

fn request_paste(host: &mut dyn UiActionHost, action_cx: ActionCx) {
    let token = host.next_clipboard_token();
    host.push_effect(Effect::ClipboardGetText {
        window: action_cx.window,
        token,
    });
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
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    if caret == 0 {
        return;
    }
    let prev = prev_char_boundary(st.buffer.text(), caret);
    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::Backspace,
        Edit::Delete { range: prev..caret },
        Selection {
            anchor: prev,
            focus: prev,
        },
    );
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
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let next = next_char_boundary(st.buffer.text(), caret);
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
}

fn move_caret_left(st: &mut CodeEditorState, extend: bool) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let new = prev_char_boundary(st.buffer.text(), caret);
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
    let new = next_char_boundary(st.buffer.text(), caret);
    if extend {
        st.selection.focus = new;
    } else {
        st.selection = Selection {
            anchor: new,
            focus: new,
        };
    }
}

fn move_caret_vertical(st: &mut CodeEditorState, delta: i32, extend: bool) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let pt = st.display_map.byte_to_display_point(&st.buffer, caret);
    let next_row = if delta < 0 {
        pt.row.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        pt.row.saturating_add(delta as usize)
    };
    let max_row = st.display_map.row_count().saturating_sub(1);
    let next_row = next_row.min(max_row);
    let next = st
        .display_map
        .display_point_to_byte(&st.buffer, DisplayPoint::new(next_row, pt.col));
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
    st.refresh_display_map();
    #[cfg(feature = "syntax")]
    invalidate_syntax_row_cache_for_delta(st, delta);
    #[cfg(not(feature = "syntax"))]
    let _ = delta;
    st.selection = next_selection;

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
    let text = st.buffer.text();
    let mode = st.text_boundary_mode;
    st.undo_group = None;

    let (sel_start, sel_end) = {
        let r = st.selection.normalized();
        (r.start, r.end)
    };
    let mut caret = st.selection.caret().min(st.buffer.len_bytes());
    if !st.selection.is_caret() && !extend {
        caret = if dir < 0 { sel_start } else { sel_end };
    }

    let next = if dir < 0 {
        move_word_left(text, caret, mode)
    } else {
        move_word_right(text, caret, mode)
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
    apply_and_record_edit(
        st,
        UndoGroupKind::Cut,
        Edit::Delete { range: start..end },
        Selection {
            anchor: start,
            focus: start,
        },
    )
    .is_some()
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

    let sel = st.selection.normalized();
    if !sel.is_empty() {
        let start_pt = st.display_map.byte_to_display_point(&st.buffer, sel.start);
        let end_pt = st.display_map.byte_to_display_point(&st.buffer, sel.end);
        if row >= start_pt.row && row <= end_pt.row {
            let line_cols = line_len_cols(&line);
            let start_col = if row == start_pt.row { start_pt.col } else { 0 };
            let end_col = if row == end_pt.row {
                end_pt.col
            } else {
                line_cols
            };
            if start_col != end_col {
                let x0 = Px(rect.origin.x.0 + start_col as f32 * cell_w.0);
                let x1 = Px(rect.origin.x.0 + end_col as f32 * cell_w.0);
                let sel_rect = Rect::new(
                    fret_core::Point::new(x0, rect.origin.y),
                    Size::new(Px((x1.0 - x0.0).max(0.0)), row_h),
                );
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

    let origin = fret_core::Point::new(rect.origin.x, rect.origin.y);
    let scope = painter.key_scope(&"fret-code-editor-row-text");
    let key: u64 = painter.child_key(scope, &(row, 0u8)).into();
    let constraints = CanvasTextConstraints {
        max_width: Some(rect.size.width),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let mut drew_rich = false;

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
            let _ = painter.rich_text(
                key,
                DrawOrder(2),
                origin,
                rich,
                text_style.clone(),
                fg,
                constraints,
                painter.scale_factor(),
            );
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
                    let _ = painter.rich_text(
                        key,
                        DrawOrder(2),
                        origin,
                        rich,
                        text_style.clone(),
                        fg,
                        constraints,
                        painter.scale_factor(),
                    );
                    drew_rich = true;
                }
            }
        }
    }

    if !drew_rich {
        let _ = painter.text(
            key,
            DrawOrder(2),
            origin,
            Arc::clone(&line),
            text_style.clone(),
            fg,
            constraints,
            painter.scale_factor(),
        );
    }

    if st.selection.is_caret() {
        let caret_pt = st
            .display_map
            .byte_to_display_point(&st.buffer, st.selection.caret());
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

fn cached_row_text(st: &mut CodeEditorState, row: usize, max_entries: usize) -> Arc<str> {
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    if st.row_text_cache_rev != rev || st.row_text_cache_wrap_cols != wrap_cols {
        st.row_text_cache_rev = rev;
        st.row_text_cache_wrap_cols = wrap_cols;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
    }

    st.row_text_cache_tick = st.row_text_cache_tick.saturating_add(1);
    let tick = st.row_text_cache_tick;

    if let Some((text, last_used)) = st.row_text_cache.get_mut(&row) {
        *last_used = tick;
        st.row_text_cache_queue.push_back((row, tick));
        return Arc::clone(text);
    }

    let range = st.display_map.display_row_byte_range(&st.buffer, row);
    let text = st.buffer.text().get(range).unwrap_or("").to_string().into();
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

    if st.syntax_row_cache.len() != before_len {
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
    let rev = st.buffer.revision();
    if st.syntax_row_cache_rev != rev || st.syntax_row_cache_language != st.language {
        st.syntax_row_cache_rev = rev;
        st.syntax_row_cache_language = st.language.clone();
        st.syntax_row_cache_tick = 0;
        st.syntax_row_cache.clear();
        st.syntax_row_cache_queue.clear();
    }

    st.syntax_row_cache_tick = st.syntax_row_cache_tick.saturating_add(1);
    let tick = st.syntax_row_cache_tick;

    if let Some((spans, last_used)) = st.syntax_row_cache.get_mut(&row) {
        *last_used = tick;
        st.syntax_row_cache_queue.push_back((row, tick));
        return Arc::clone(spans);
    }

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

    let Some(slice) = st.buffer.text().get(start_byte..end_byte) else {
        return;
    };

    let Ok(spans) = fret_syntax::highlight(slice, language) else {
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
        }

        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, "world".to_string()).unwrap();
        handle.replace_buffer(buffer);

        let st = handle.state.borrow();
        assert_eq!(st.buffer.text(), "world");
        assert_eq!(st.selection, Selection::default());
        assert_eq!(st.preedit, None);
        assert!(st.undo_group.is_none());
        assert!(!st.dragging);
        assert_eq!(st.drag_pointer, None);
        assert_eq!(st.row_text_cache.len(), 0);
        assert_eq!(st.row_text_cache_queue.len(), 0);
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
    fn caret_rect_offsets_for_preedit_cursor() {
        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, "hello".to_string()).unwrap();
        let sel = Selection {
            anchor: 0,
            focus: 0,
        };
        let preedit = PreeditState {
            text: "ab".to_string(),
            cursor: Some((0, 2)),
        };

        let scroll = fret_ui::scroll::ScrollHandle::default();
        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(500.0)),
        );

        let map = DisplayMap::new(&buffer, None);
        let rect = caret_rect_for_selection(
            &buffer,
            &map,
            sel,
            Some(&preedit),
            Px(20.0),
            Px(10.0),
            bounds,
            &scroll,
        )
        .expect("caret rect");

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

        let text = st.buffer.text();
        let caret = fret_code_editor_view::clamp_to_char_boundary(
            text,
            st.selection.caret().min(text.len()),
        );
        let (start, end) = a11y_text_window_bounds(text, caret);
        assert_eq!(start, 0);
        assert_eq!(end, text.len());

        let anchor = 0u32;
        let focus = u32::try_from("hello".len()).unwrap();
        let new_anchor = map_a11y_offset_to_buffer(text, start, end, anchor);
        let new_focus = map_a11y_offset_to_buffer(text, start, end, focus);
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
        move_caret_vertical(&mut st, 1, false);
        assert_eq!(st.selection.caret(), 2);

        // Row 1 col 0 -> Down => row 2 col 0 (next logical line "ef").
        move_caret_vertical(&mut st, 1, false);
        assert_eq!(st.selection.caret(), 5);

        // Row 2 is the last display row; another Down should clamp.
        move_caret_vertical(&mut st, 1, false);
        assert_eq!(st.selection.caret(), 5);
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
