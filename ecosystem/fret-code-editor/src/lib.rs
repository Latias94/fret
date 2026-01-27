//! Code editor surface (UI integration) for Fret.
//!
//! This is a v1 MVP: fixed row height, no soft wrap, and a monospace "cell width" heuristic for
//! caret/selection geometry.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use fret_code_editor_buffer::{DocId, Edit, TextBuffer};
use fret_code_editor_view::{DisplayPoint, byte_to_display_point, display_point_to_byte};
use fret_core::{
    Color, Corners, DrawOrder, Edges, FontId, KeyCode, Modifiers, MouseButton, Px, Rect, SceneOp,
    Size, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{ClipboardToken, Effect};
use fret_ui::action::{ActionCx, KeyDownCx, UiActionHost, UiPointerActionHost};
use fret_ui::canvas::CanvasTextConstraints;
use fret_ui::element::AnyElement;
use fret_ui::element::{
    CanvasCachePolicy, CanvasCacheTuning, Length, Overflow, PointerRegionProps,
    TextInputRegionProps,
};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::windowed_rows_surface::{
    OnWindowedRowsPaintFrame, OnWindowedRowsPointerCancel, OnWindowedRowsPointerDown,
    OnWindowedRowsPointerMove, OnWindowedRowsPointerUp, WindowedRowsPaintFrame,
    WindowedRowsSurfacePointerHandlers, WindowedRowsSurfaceProps,
    windowed_rows_surface_with_pointer_region,
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
struct CodeEditorState {
    buffer: TextBuffer,
    selection: Selection,
    preedit: Option<PreeditState>,
    dragging: bool,
    drag_pointer: Option<fret_core::PointerId>,
    last_bounds: Option<Rect>,
    row_text_cache_rev: fret_code_editor_buffer::Revision,
    row_text_cache_tick: u64,
    row_text_cache: HashMap<usize, (Arc<str>, u64)>,
    row_text_cache_queue: VecDeque<(usize, u64)>,
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
        Self {
            state: Rc::new(RefCell::new(CodeEditorState {
                buffer,
                selection: Selection::default(),
                preedit: None,
                dragging: false,
                drag_pointer: None,
                last_bounds: None,
                row_text_cache_rev: fret_code_editor_buffer::Revision(0),
                row_text_cache_tick: 0,
                row_text_cache: HashMap::new(),
                row_text_cache_queue: VecDeque::new(),
            })),
        }
    }

    pub fn with_buffer<R>(&self, f: impl FnOnce(&TextBuffer) -> R) -> R {
        let st = self.state.borrow();
        f(&st.buffer)
    }
}

pub struct CodeEditor {
    handle: CodeEditorHandle,
    overscan: usize,
    torture: Option<CodeEditorTorture>,
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
        }
    }

    pub fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan.max(1);
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

        cx.keyed("code-editor", move |cx| {
            let theme = cx.theme().clone();
            let region_id = cx.root_id();

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

            let content_len = editor_state.borrow().buffer.line_count();

            let mut region_layout = fret_ui::element::LayoutStyle::default();
            region_layout.size.width = Length::Fill;
            region_layout.size.height = Length::Fill;
            region_layout.overflow = Overflow::Clip;

            let region_props = TextInputRegionProps {
                layout: region_layout,
                enabled: true,
                text_boundary_mode_override: Some(fret_runtime::TextBoundaryMode::Identifier),
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

                    let caret = caret_for_pointer(&st.buffer, row, bounds, down.position, cell_w);
                    if down.modifiers.shift {
                        st.selection.focus = caret;
                    } else {
                        st.selection = Selection {
                            anchor: caret,
                            focus: caret,
                        };
                    }
                    st.preedit = None;

                    let caret_rect = caret_rect_for_selection(
                        &st.buffer,
                        st.selection,
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

                    let bounds = host.bounds();
                    st.last_bounds = Some(bounds);

                    let cell_w = on_pointer_move_cell_w.get();
                    let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
                    let caret = caret_for_pointer(&st.buffer, row, bounds, mv.position, cell_w);
                    st.selection.focus = caret;

                    let caret_rect = caret_rect_for_selection(
                        &st.buffer,
                        st.selection,
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
                .max(256)
                .min(8_192);
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
                let overlay_bg = overlay_bg;
                let text_cache_max_entries = text_cache_max_entries;
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
                                let _ = insert_text(&mut st, text.as_str());
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

                let clipboard_state = editor_state.clone();
                let clipboard_scroll = scroll_handle.clone();
                let clipboard_cell_w = cell_w.clone();
                cx.text_input_region_on_clipboard_text(Arc::new(
                    move |host: &mut dyn UiActionHost,
                          action_cx: ActionCx,
                          _token: ClipboardToken,
                          text: &str| {
                        let mut st = clipboard_state.borrow_mut();
                        let _ = insert_text(&mut st, text);
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

fn line_len_cols(line: &str) -> usize {
    line.chars().count()
}

fn caret_for_pointer(
    buf: &TextBuffer,
    row: usize,
    bounds: Rect,
    position: fret_core::Point,
    cell_w: Px,
) -> usize {
    let line = buf.line_text(row).unwrap_or("");
    let col_max = line_len_cols(line);
    let local_x = Px(position.x.0 - bounds.origin.x.0);
    let col = if cell_w.0 > 0.0 {
        (local_x.0 / cell_w.0).floor().max(0.0) as usize
    } else {
        0
    };
    let col = col.min(col_max);
    display_point_to_byte(buf, DisplayPoint::new(row, col))
}

fn caret_rect_for_selection(
    buf: &TextBuffer,
    sel: Selection,
    row_h: Px,
    cell_w: Px,
    bounds: Rect,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) -> Option<Rect> {
    if !sel.is_caret() {
        return None;
    }

    let caret = sel.caret().min(buf.len_bytes());
    let pt = byte_to_display_point(buf, caret);
    let offset = scroll_handle.offset();
    let y = Px(bounds.origin.y.0 + (pt.row as f32 * row_h.0) - offset.y.0);
    let x = Px(bounds.origin.x.0 + pt.col as f32 * cell_w.0);
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
        st.selection,
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

fn insert_text(st: &mut CodeEditorState, text: &str) -> Option<()> {
    if text.is_empty() {
        return None;
    }
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let _ = st
        .buffer
        .apply(Edit::Replace {
            range: start..end,
            text: text.to_string(),
        })
        .ok()?;

    let caret = start.saturating_add(text.len()).min(st.buffer.len_bytes());
    st.selection = Selection {
        anchor: caret,
        focus: caret,
    };
    Some(())
}

fn prev_char_boundary(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    if idx == 0 {
        return 0;
    }
    idx = idx.saturating_sub(1);
    while idx > 0 && !text.is_char_boundary(idx) {
        idx = idx.saturating_sub(1);
    }
    idx
}

fn next_char_boundary(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    if idx >= text.len() {
        return text.len();
    }
    idx = idx.saturating_add(1).min(text.len());
    while idx < text.len() && !text.is_char_boundary(idx) {
        idx = idx.saturating_add(1).min(text.len());
    }
    idx
}

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
    let cell_w_px = cell_w.get();

    let shift = modifiers.shift;
    let ctrl_or_meta = modifiers.ctrl || modifiers.meta;

    match key {
        KeyCode::ArrowLeft => {
            move_caret_left(&mut st, shift);
        }
        KeyCode::ArrowRight => {
            move_caret_right(&mut st, shift);
        }
        KeyCode::ArrowUp => {
            move_caret_vertical(&mut st, -1, shift);
        }
        KeyCode::ArrowDown => {
            move_caret_vertical(&mut st, 1, shift);
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
        let _ = st.buffer.apply(Edit::Delete { range: start..end });
        st.selection = Selection {
            anchor: start,
            focus: start,
        };
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    if caret == 0 {
        return;
    }
    let prev = prev_char_boundary(st.buffer.text(), caret);
    let _ = st.buffer.apply(Edit::Delete { range: prev..caret });
    st.selection = Selection {
        anchor: prev,
        focus: prev,
    };
}

fn delete_forward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = st.buffer.apply(Edit::Delete { range: start..end });
        st.selection = Selection {
            anchor: start,
            focus: start,
        };
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let next = next_char_boundary(st.buffer.text(), caret);
    if next == caret {
        return;
    }
    let _ = st.buffer.apply(Edit::Delete { range: caret..next });
    st.selection = Selection {
        anchor: caret,
        focus: caret,
    };
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
    let pt = byte_to_display_point(&st.buffer, caret);
    let next_row = if delta < 0 {
        pt.row.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        pt.row.saturating_add(delta as usize)
    };
    let next_row = next_row.min(st.buffer.line_count().saturating_sub(1));
    let next = display_point_to_byte(&st.buffer, DisplayPoint::new(next_row, pt.col));
    if extend {
        st.selection.focus = next;
    } else {
        st.selection = Selection {
            anchor: next,
            focus: next,
        };
    }
}

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
        let start_pt = byte_to_display_point(&st.buffer, sel.start);
        let end_pt = byte_to_display_point(&st.buffer, sel.end);
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

    if st.selection.is_caret() {
        let caret_pt = byte_to_display_point(&st.buffer, st.selection.caret());
        if caret_pt.row == row {
            let x = Px(rect.origin.x.0 + caret_pt.col as f32 * cell_w.0);
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

    if let Some(preedit) = &st.preedit {
        let caret_pt = byte_to_display_point(&st.buffer, st.selection.caret());
        if caret_pt.row == row {
            let x = Px(rect.origin.x.0 + caret_pt.col as f32 * cell_w.0);
            let origin = fret_core::Point::new(x, rect.origin.y);
            let scope = painter.key_scope(&"fret-code-editor-row-text");
            let key: u64 = painter.child_key(scope, &(row, 1u8)).into();
            let _ = painter.text(
                key,
                DrawOrder(4),
                origin,
                preedit.text.as_str(),
                text_style.clone(),
                fg,
                constraints,
                painter.scale_factor(),
            );
        }
    }
}

fn cached_row_text(st: &mut CodeEditorState, row: usize, max_entries: usize) -> Arc<str> {
    let rev = st.buffer.revision();
    if st.row_text_cache_rev != rev {
        st.row_text_cache_rev = rev;
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

    let text = st.buffer.line_text(row).unwrap_or("").to_string().into();
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
