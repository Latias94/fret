use super::*;
use super::{input, paint};
use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, MaterialRegistrationError, MaterialService, Modifiers,
    PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point, Px, Rect,
    Size, SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::TickId;
use fret_ui::tree::UiTree;
use fret_ui_kit::declarative::windowed_rows_surface::{
    WindowedRowsSurfaceDiagnosticsStore, WindowedRowsSurfaceWindowTelemetry,
};
use std::sync::Arc;

mod accessibility;
mod feature_payloads;
mod geometry;
mod platform_text_input;
mod platform_text_input_roundtrip;
mod pointer_helpers;
mod pointer_selection;
mod row_text_cache;
mod support;
#[cfg(feature = "syntax-rust")]
mod syntax;
mod word_navigation;

use support::*;

#[derive(Default)]
struct TestHost {
    models: fret_runtime::ModelStore,
    next_timer: u64,
    next_clipboard: u64,
    next_share_sheet: u64,
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

    fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
        self.next_share_sheet = self.next_share_sheet.saturating_add(1);
        fret_runtime::ShareSheetToken(self.next_share_sheet)
    }
}

impl fret_ui::action::UiFocusActionHost for TestHost {
    fn request_focus(&mut self, _target: fret_ui::GlobalElementId) {}
}

#[derive(Default)]
struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(16.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
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
        st.drag_autoscroll_timer = Some(TimerToken(123));
        st.drag_autoscroll_viewport_pos = Some(fret_core::Point::new(Px(12.0), Px(34.0)));
        st.row_text_cache.insert(
            0,
            (
                RowTextCacheEntry {
                    text: Arc::from("hello"),
                    range: 0..5,
                    fold_map: None,
                    preedit_range: None,
                    row_spans: Arc::from([]),
                },
                1,
            ),
        );
        st.row_text_cache_queue.push_back((0, 1));
        st.row_geom_cache.insert(
            0,
            (
                RowGeom {
                    row_range: 0..5,
                    key: row_geom_key_for_tests(&Arc::from("hello")),
                    caret_stops: vec![(0, Px(0.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: false,
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
    assert!(st.drag_autoscroll_timer.is_none());
    assert!(st.drag_autoscroll_viewport_pos.is_none());
    assert_eq!(st.row_text_cache.len(), 0);
    assert_eq!(st.row_text_cache_queue.len(), 0);
    assert_eq!(st.row_geom_cache.len(), 0);
    assert_eq!(st.row_geom_cache_queue.len(), 0);
}

#[cfg(feature = "syntax")]
#[test]
fn syntax_prefetch_visible_window_uses_display_map_lines_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef\nuvwxyz\n123456");
    handle.set_soft_wrap_cols(Some(2));

    let visible_lines = {
        let st = handle.state.borrow();
        crate::editor::syntax::syntax_prefetch_visible_line_window(
            &st,
            WindowedRowsPaintFrame {
                viewport_height: Px(64.0),
                offset_y: Px(0.0),
                visible_start: 2,
                visible_end: 4,
            },
        )
    };

    assert_eq!(
        visible_lines,
        Some((0, 1)),
        "syntax prefetch must translate display rows to physical buffer lines before chunking"
    );
}

#[test]
fn paint_source_does_not_materialize_whole_buffer_string() {
    // Regression guard: the editor paint path should never call `TextBuffer::text_string()`.
    // Materializing the entire rope would scale with document size and defeat row virtualization.
    const SRC: &str = include_str!("../paint/mod.rs");
    assert!(
        !SRC.contains(".text_string("),
        "paint/mod.rs must not call TextBuffer::text_string()"
    );
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
fn text_boundary_mode_override_can_be_cleared() {
    let handle = CodeEditorHandle::new("hello");
    assert_eq!(
        handle.text_boundary_mode_override(),
        Some(TextBoundaryMode::Identifier)
    );

    handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);
    assert_eq!(
        handle.text_boundary_mode_override(),
        Some(TextBoundaryMode::UnicodeWord)
    );

    handle.set_text_boundary_mode_override(None);
    assert_eq!(handle.text_boundary_mode_override(), None);
    assert_eq!(handle.text_boundary_mode(), TextBoundaryMode::UnicodeWord);
}

#[test]
fn caret_left_right_skips_folded_ranges() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }],
    );

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 1,
        };
        input::move_caret_right(&mut st, false);
        assert_eq!(st.selection.caret(), 4);

        input::move_caret_left(&mut st, false);
        assert_eq!(st.selection.caret(), 1);
    }
}

#[test]
fn caret_left_right_skips_folded_ranges_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(2));
    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }],
    );

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 1,
        };
        input::move_caret_right(&mut st, false);
        assert_eq!(st.selection.caret(), 4);

        input::move_caret_left(&mut st, false);
        assert_eq!(st.selection.caret(), 1);
    }
}

#[test]
fn enabling_folds_snaps_caret_out_of_folded_range() {
    let handle = CodeEditorHandle::new("abcdef");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };
    }

    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }],
    );

    let st = handle.state.borrow();
    assert_eq!(st.selection.caret(), 1);
}

#[test]
fn enabling_folds_snaps_caret_out_of_folded_range_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(2));
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };
    }

    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }],
    );

    let st = handle.state.borrow();
    assert_eq!(st.selection.caret(), 1);
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
                    key: row_geom_key_for_tests(&Arc::from("aaaa")),
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: false,
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
                    key: row_geom_key_for_tests(&Arc::from("bbbb")),
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: false,
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
                    key: row_geom_key_for_tests(&Arc::from("cccc")),
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: false,
                    preedit: None,
                },
                1,
            ),
        );

        input::move_caret_vertical(&mut st, 1, false, Px(8.0));
        assert_eq!(st.selection.caret(), 7, "row 1, local index 2");
        assert_eq!(st.caret_preferred_x, Some(Px(20.0)));

        input::move_caret_vertical(&mut st, 1, false, Px(8.0));
        assert_eq!(st.selection.caret(), 12, "row 2, local index 2");
        assert_eq!(st.caret_preferred_x, Some(Px(20.0)));
    }
}

#[test]
fn row_geom_cache_is_shifted_across_single_line_soft_wrap_edits() {
    let handle = CodeEditorHandle::new("aaaaaa\nbbbbbb\ncccccc");
    handle.set_soft_wrap_cols(Some(4));

    let before = {
        let mut st = handle.state.borrow_mut();
        assert_eq!(
            st.display_map.row_count(),
            6,
            "3 lines * 2 wrapped rows each"
        );

        // Seed geometry for the second and third logical lines (rows 2..6).
        for row in 2..6 {
            let range = st.display_map.display_row_byte_range(&st.buffer, row);
            st.row_geom_cache.insert(
                row,
                (
                    RowGeom {
                        row_range: range.clone(),
                        key: row_geom_key_for_tests(&Arc::from("")),
                        caret_stops: vec![(0, Px(0.0))],
                        fold_map: None,
                        caret_rect_top: None,
                        caret_rect_height: None,
                        has_preedit: false,
                        preedit: None,
                    },
                    1,
                ),
            );
            st.row_geom_cache_queue.push_back((row, 1));
        }

        // Capture the original ranges so we can assert on the shifted values after the edit.
        (2..6)
            .map(|row| {
                (
                    row,
                    st.row_geom_cache.get(&row).unwrap().0.row_range.clone(),
                )
            })
            .collect::<Vec<_>>()
    };

    {
        let mut st = handle.state.borrow_mut();
        let at = 6; // End of the first logical line.
        let edit = Edit::Insert {
            at,
            text: "zzzz".to_string(),
        };
        let caret = at + 4;
        input::apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            edit,
            Selection {
                anchor: caret,
                focus: caret,
            },
        )
        .expect("edit must apply");
        assert_eq!(
            st.display_map.row_count(),
            7,
            "inserting four chars grows the first line from 2 -> 3 wrapped rows"
        );
    }

    let st = handle.state.borrow();
    assert_eq!(
        st.row_geom_cache.len(),
        4,
        "unaffected lines keep row geometry cached"
    );

    for (old_row, old_range) in before {
        let new_row = old_row + 1;
        let (geom, _) = st
            .row_geom_cache
            .get(&new_row)
            .expect("shifted row present");
        assert_eq!(
            geom.row_range,
            (old_range.start + 4)..(old_range.end + 4),
            "byte ranges shift by the inserted text length"
        );
    }
}

#[test]
fn row_geom_cache_is_byte_shifted_for_single_line_non_wrap_edits() {
    let handle = CodeEditorHandle::new("hello\nworld\nagain");

    let before = {
        let mut st = handle.state.borrow_mut();
        assert_eq!(st.display_map.row_count(), 3);

        for row in 1..3 {
            let range = st.display_map.display_row_byte_range(&st.buffer, row);
            st.row_geom_cache.insert(
                row,
                (
                    RowGeom {
                        row_range: range.clone(),
                        key: row_geom_key_for_tests(&Arc::from("")),
                        caret_stops: vec![(0, Px(0.0))],
                        fold_map: None,
                        caret_rect_top: None,
                        caret_rect_height: None,
                        has_preedit: false,
                        preedit: None,
                    },
                    1,
                ),
            );
            st.row_geom_cache_queue.push_back((row, 1));
        }
        (1..3)
            .map(|row| {
                (
                    row,
                    st.row_geom_cache.get(&row).unwrap().0.row_range.clone(),
                )
            })
            .collect::<Vec<_>>()
    };

    {
        let mut st = handle.state.borrow_mut();
        let edit = Edit::Insert {
            at: 0,
            text: "123".to_string(),
        };
        input::apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            edit,
            Selection {
                anchor: 3,
                focus: 3,
            },
        )
        .expect("edit must apply");
        assert_eq!(
            st.display_map.row_count(),
            3,
            "non-wrapping edits keep row count stable"
        );
    }

    let st = handle.state.borrow();
    assert_eq!(st.row_geom_cache.len(), 2);
    for (row, old_range) in before {
        let (geom, _) = st.row_geom_cache.get(&row).expect("row present");
        assert_eq!(
            geom.row_range,
            (old_range.start + 3)..(old_range.end + 3),
            "byte ranges shift by the inserted text length"
        );
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

    let handled = input::handle_key_down(
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
fn ctrl_a_selects_all() {
    let handle = CodeEditorHandle::new("hello\nworld");
    handle.set_caret(3);

    let mut host = TestHost::default();
    let action_cx = ActionCx {
        window: fret_core::AppWindowId::default(),
        target: fret_ui::GlobalElementId(0),
    };
    let scroll = fret_ui::scroll::ScrollHandle::default();
    let cell_w = Cell::new(Px(10.0));

    let handled = input::handle_key_down(
        &mut host,
        action_cx,
        &handle.state,
        Px(16.0),
        &scroll,
        &cell_w,
        KeyCode::KeyA,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    assert!(handled);

    let st = handle.state.borrow();
    assert_eq!(st.selection.anchor, 0);
    assert_eq!(st.selection.focus, st.buffer.len_bytes());
}

#[test]
fn read_only_allows_navigation_but_blocks_edits() {
    let handle = CodeEditorHandle::new("hello");
    handle.set_caret(5);
    handle.set_interaction(CodeEditorInteractionOptions::read_only());

    let mut host = TestHost::default();
    let action_cx = ActionCx {
        window: fret_core::AppWindowId::default(),
        target: fret_ui::GlobalElementId(0),
    };
    let scroll = fret_ui::scroll::ScrollHandle::default();
    let cell_w = Cell::new(Px(10.0));

    let handled = input::handle_key_down(
        &mut host,
        action_cx,
        &handle.state,
        Px(16.0),
        &scroll,
        &cell_w,
        KeyCode::Backspace,
        Modifiers::default(),
    );
    assert!(handled);
    assert_eq!(handle.with_buffer(|b| b.text_string()), "hello");
    assert_eq!(handle.selection().caret(), 5);

    let handled = input::handle_key_down(
        &mut host,
        action_cx,
        &handle.state,
        Px(16.0),
        &scroll,
        &cell_w,
        KeyCode::ArrowLeft,
        Modifiers::default(),
    );
    assert!(handled);
    assert_eq!(handle.selection().caret(), 4);

    {
        let mut st = handle.state.borrow_mut();
        assert!(input::insert_text(&mut st, "x").is_none());
        assert!(!input::undo(&mut st));
        assert!(!input::redo(&mut st));
    }
    assert_eq!(handle.with_buffer(|b| b.text_string()), "hello");
}

#[test]
fn font_stack_key_change_clears_geometry_caches() {
    let handle = CodeEditorHandle::new("hello");
    let mut st = handle.state.borrow_mut();

    st.font_stack_key = fret_runtime::TextFontStackKey(1);
    st.row_geom_cache.insert(
        0,
        (
            RowGeom {
                row_range: 0..5,
                key: row_geom_key_for_tests(&Arc::from("hello")),
                caret_stops: vec![(0, Px(0.0)), (5, Px(50.0))],
                fold_map: None,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: false,
                preedit: None,
            },
            1,
        ),
    );
    st.baseline_measure_cache = Some(BaselineMeasureCache {
        max_width: Px(100.0),
        row_h: Px(20.0),
        scale_bits: 0,
        text_style: TextStyle {
            font: FontId::monospace(),
            size: Px(12.0),
            ..Default::default()
        },
        metrics: fret_core::TextMetrics {
            size: Size::new(Px(0.0), Px(0.0)),
            baseline: Px(0.0),
        },
        measured_h: Px(0.0),
    });

    st.update_font_stack_key(fret_runtime::TextFontStackKey(1));
    assert!(!st.row_geom_cache.is_empty());
    assert!(st.baseline_measure_cache.is_some());

    st.update_font_stack_key(fret_runtime::TextFontStackKey(2));
    assert!(st.row_geom_cache.is_empty());
    assert!(st.baseline_measure_cache.is_none());
}

#[test]
fn caret_rect_ignores_stale_row_geom_with_preedit_mapping() {
    let handle = CodeEditorHandle::new("abc");
    let scroll = fret_ui::scroll::ScrollHandle::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 1,
        };
        st.preedit = None;
        st.row_geom_cache.insert(
            0,
            (
                RowGeom {
                    row_range: 0..3,
                    key: row_geom_key_for_tests(&Arc::from("abc")),
                    caret_stops: vec![(0, Px(0.0)), (1, Px(100.0)), (2, Px(200.0)), (3, Px(300.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: true,
                    preedit: Some(RowPreeditMapping {
                        insert_at: 0,
                        preedit_len: 2,
                    }),
                },
                1,
            ),
        );
    }

    let mut st = handle.state.borrow_mut();
    let rect =
        caret_rect_for_selection(&mut st, Px(20.0), Px(10.0), bounds, &scroll).expect("caret rect");
    assert_eq!(rect.origin.x, Px(10.0), "fallback col (1) * cell_w (10px)");
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

    let shaping = fret_core::TextShapingStyle::default();
    let rich = paint::materialize_preedit_rich_text(
        "hello".into(),
        2,
        &shaping,
        &preedit,
        fg,
        selection_bg,
    );
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
fn preedit_rich_text_applies_code_shaping_to_all_spans() {
    let preedit = PreeditState {
        text: "ab".to_string(),
        cursor: None,
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

    let shaping = fret_core::TextShapingStyle::default()
        .with_feature("liga", 0)
        .with_feature("calt", 0);
    let rich = paint::materialize_preedit_rich_text(
        "hello".into(),
        2,
        &shaping,
        &preedit,
        fg,
        selection_bg,
    );
    assert!(rich.is_valid());
    assert!(
        rich.spans.iter().all(|s| s
            .shaping
            .features
            .iter()
            .any(|f| f.tag == "liga" && f.value == 0)),
        "expected `liga=0` to be applied to every span"
    );
    assert!(
        rich.spans.iter().all(|s| s
            .shaping
            .features
            .iter()
            .any(|f| f.tag == "calt" && f.value == 0)),
        "expected `calt=0` to be applied to every span"
    );
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
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), 2);

    // Row 1 col 0 -> Down => row 2 col 0 (next logical line "ef").
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), 5);

    // Row 2 is the last display row; another Down should clamp.
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), 5);
}

#[test]
fn move_caret_vertical_steps_through_code_wrap_policy_rows() {
    let handle = CodeEditorHandle::new("left->right->tail\nzzz");
    handle.set_soft_wrap_cols(Some(6));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    // Expect the first logical line to be split into three wrapped display rows.
    let rows: Vec<String> = (0..st.display_map.row_count())
        .map(|row| {
            let range = st.display_map.display_row_byte_range(&st.buffer, row);
            st.buffer.slice_to_string(range).unwrap_or_default()
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            "left->".to_string(),
            "right".to_string(),
            "->tail".to_string(),
            "zzz".to_string()
        ]
    );

    // Down should walk display rows (not logical lines) until it reaches the next line.
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), "left->".len());
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), "left->right".len());
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), "left->right->tail\n".len());
}

#[test]
fn apply_and_record_edit_refreshes_display_map_only_when_needed() {
    let handle = CodeEditorHandle::new("ab\nc");

    {
        let mut st = handle.state.borrow_mut();
        assert_eq!(st.display_wrap_cols, None);
        assert_eq!(st.display_map.row_count(), 2);

        // No newline, no wrap => row_count should remain correct without forcing a refresh.
        input::apply_and_record_edit(
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
        input::apply_and_record_edit(
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

        input::apply_and_record_edit(
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
    input::move_caret_home_end(&mut st, true, false, false);
    assert_eq!(st.selection.caret(), 2);

    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    input::move_caret_home_end(&mut st, false, false, false);
    assert_eq!(st.selection.caret(), 4);

    // Ctrl+Home/End should clamp to document bounds.
    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    input::move_caret_home_end(&mut st, true, true, false);
    assert_eq!(st.selection.caret(), 0);

    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    input::move_caret_home_end(&mut st, false, true, false);
    assert_eq!(st.selection.caret(), st.buffer.len_bytes());
}

#[test]
fn shift_home_end_extends_selection_within_wrapped_display_rows() {
    let handle = CodeEditorHandle::new("abcd\nef");
    handle.set_soft_wrap_cols(Some(2));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };

    input::move_caret_home_end(&mut st, true, false, true);
    assert_eq!(
        st.selection,
        Selection {
            anchor: 3,
            focus: 2,
        }
    );

    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    input::move_caret_home_end(&mut st, false, false, true);
    assert_eq!(
        st.selection,
        Selection {
            anchor: 3,
            focus: 4,
        }
    );
}

#[test]
fn shift_vertical_extends_selection_in_display_row_space_when_wrapped() {
    let handle = CodeEditorHandle::new("abcd\nef");
    handle.set_soft_wrap_cols(Some(2));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    input::move_caret_vertical(&mut st, 1, true, Px(10.0));
    assert_eq!(
        st.selection,
        Selection {
            anchor: 0,
            focus: 2,
        }
    );

    input::move_caret_vertical(&mut st, 1, true, Px(10.0));
    assert_eq!(
        st.selection,
        Selection {
            anchor: 0,
            focus: 5,
        }
    );
}

#[test]
fn home_end_respects_code_wrap_policy_row_boundaries() {
    let handle = CodeEditorHandle::new("left->right->tail");
    handle.set_soft_wrap_cols(Some(6));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    let mut st = handle.state.borrow_mut();
    // Place the caret inside the "right" segment.
    st.selection = Selection {
        anchor: "left->r".len(),
        focus: "left->r".len(),
    };

    // Sanity: End should produce a caret in the same display-map space as the policy segmentation.
    input::move_caret_home_end(&mut st, false, false, false);

    let mut rows: Vec<String> = Vec::new();
    for row in 0..st.display_map.row_count() {
        let range = st.display_map.display_row_byte_range(&st.buffer, row);
        rows.push(st.buffer.slice_to_string(range).unwrap_or_default());
    }
    assert_eq!(rows.join(""), st.buffer.text_string());

    for w in rows.windows(2) {
        assert!(
            !(w[0].ends_with('-') && w[1].starts_with('>')),
            "expected `->` to not be split across a wrap boundary: rows={rows:?}"
        );
    }
}

#[test]
fn set_code_wrap_policy_clears_row_geom_cache_when_wrapped() {
    let handle = CodeEditorHandle::new("left->right->tail");
    handle.set_soft_wrap_cols(Some(6));

    {
        let mut st = handle.state.borrow_mut();
        st.row_geom_cache.insert(
            0,
            (
                geom::RowGeom {
                    row_range: 0.."left->".len(),
                    key: row_geom_key_for_tests(&Arc::from("left->")),
                    caret_stops: vec![(0, Px(0.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: false,
                    preedit: None,
                },
                0,
            ),
        );
        assert!(!st.row_geom_cache.is_empty());
    }

    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Conservative,
        ),
    ));

    let st = handle.state.borrow();
    assert!(st.row_geom_cache.is_empty());
    assert_eq!(st.row_geom_cache_wrap_cols, st.display_wrap_cols);
}

#[test]
fn move_caret_vertical_uses_row_fold_map_for_inlay_insertions_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(3));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));
    handle.set_line_inlays(
        0,
        vec![InlaySpan {
            byte: 3,
            text: Arc::<str>::from("X"),
        }],
    );

    let mut st = handle.state.borrow_mut();
    assert_eq!(st.display_map.row_count(), 3);
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    // Prefer a stable target x so the vertical move hit-tests a specific caret stop.
    st.caret_preferred_x = Some(Px(10.0));

    // Seed geometry for the next row ("X") so the move path uses the geometry cache.
    let (row_range, row_text, fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, 1, 64);
    assert_eq!(
        row_range,
        st.display_map.display_row_byte_range(&st.buffer, 1),
        "cached row text range must match the display map"
    );
    assert_eq!(row_text.as_ref(), "X");
    assert!(
        fold_map.is_some(),
        "expected an insertion span for the inlay"
    );

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        1,
        (
            RowGeom {
                row_range: row_range.clone(),
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: false,
                preedit: None,
            },
            1,
        ),
    );

    input::move_caret_vertical(&mut st, 1, false, Px(8.0));
    assert_eq!(
        st.selection.caret(),
        3,
        "expected caret to snap to the inlay insertion point (before 'd')"
    );
}

#[test]
fn move_caret_vertical_uses_row_fold_map_for_fold_placeholders_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(2));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));
    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..5,
            placeholder: Arc::<str>::from("."),
        }],
    );

    let mut st = handle.state.borrow_mut();
    assert_eq!(st.display_map.row_count(), 2);

    // Seed geometry for the first wrapped row ("a.") so the vertical move path uses the cached
    // caret stop mapping (which must respect fold placeholders).
    let (row_range, row_text, fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, 0, 64);
    assert_eq!(
        row_range,
        st.display_map.display_row_byte_range(&st.buffer, 0),
        "cached row text range must match the display map"
    );
    assert_eq!(row_text.as_ref(), "a.");
    assert!(fold_map.is_some());

    // Start on the second wrapped row and move up. The requested x targets the placeholder in the
    // first wrapped row, which must snap to the fold start in the base buffer.
    st.selection = Selection {
        anchor: 5,
        focus: 5,
    };
    st.caret_preferred_x = Some(Px(10.0));

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        0,
        (
            RowGeom {
                row_range: row_range.clone(),
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: false,
                preedit: None,
            },
            1,
        ),
    );

    input::move_caret_vertical(&mut st, -1, false, Px(8.0));
    assert_eq!(
        st.selection.caret(),
        1,
        "expected caret to snap to the fold start when targeting the placeholder"
    );
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

    input::move_caret_page(&mut st, 1, false, row_h, &scroll, Px(10.0));

    let expected = st
        .display_map
        .display_point_to_byte(&st.buffer, DisplayPoint::new(2, 0));
    assert_eq!(st.selection.caret(), expected);
    assert_eq!(scroll.offset().y, Px(20.0));
}

#[test]
fn editor_viewport_wheel_scroll_updates_inner_window_without_bounds_drift() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let mut services = FakeServices::default();
    let text = (0..240)
        .map(|idx| format!("fn line_{idx:03}() {{ println!(\"scroll {idx:03}\"); }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let handle = CodeEditorHandle::new(text);

    app.set_tick_id(TickId(1));
    app.set_frame_id(FrameId(1));
    render_editor_scroll_audit_frame(&mut ui, &mut app, &mut services, window, handle.clone());
    app.set_tick_id(TickId(2));
    app.set_frame_id(FrameId(2));
    render_editor_scroll_audit_frame(&mut ui, &mut app, &mut services, window, handle.clone());

    let snap = ui
        .semantics_snapshot()
        .expect("semantics snapshot before wheel");
    let before_bounds = bounds_by_test_id(&ui, &snap, "code-editor-scroll-audit-viewport");
    let before = windowed_rows_telemetry(&app, window);
    assert!(
        before.visible_count > 0,
        "expected visible rows before wheel, telemetry={before:?}"
    );
    assert!(
        before.offset_y.0.abs() <= 0.01,
        "expected initial inner viewport offset near zero, telemetry={before:?}"
    );
    assert!(
        (before_bounds.size.height.0 - before.viewport_height.0).abs() <= 0.01,
        "expected viewport test_id bounds to match visible viewport height: bounds={before_bounds:?} telemetry={before:?}"
    );
    assert!(
        before.content_height.0 > before_bounds.size.height.0 + 0.01,
        "expected content to be taller than the visible viewport in this regression fixture: bounds={before_bounds:?} telemetry={before:?}"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position: center_of(before_bounds),
            delta: Point::new(Px(0.0), Px(-120.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    app.set_tick_id(TickId(3));
    app.set_frame_id(FrameId(3));
    render_editor_scroll_audit_frame(&mut ui, &mut app, &mut services, window, handle);

    let snap = ui
        .semantics_snapshot()
        .expect("semantics snapshot after wheel");
    let after_bounds = bounds_by_test_id(&ui, &snap, "code-editor-scroll-audit-viewport");
    let after = windowed_rows_telemetry(&app, window);
    assert!(
        after.offset_y.0 > before.offset_y.0 + 0.01,
        "expected wheel to advance editor inner viewport offset: before={before:?} after={after:?}"
    );
    assert!(
        after.visible_start.unwrap_or(0) >= before.visible_start.unwrap_or(0),
        "expected visible row window to stay monotonic after wheel: before={before:?} after={after:?}"
    );
    assert!(
        (after_bounds.origin.x.0 - before_bounds.origin.x.0).abs() <= 0.01
            && (after_bounds.origin.y.0 - before_bounds.origin.y.0).abs() <= 0.01
            && (after_bounds.size.width.0 - before_bounds.size.width.0).abs() <= 0.01
            && (after_bounds.size.height.0 - before_bounds.size.height.0).abs() <= 0.01,
        "expected editor viewport bounds to stay stable while inner scroll moves: before={before_bounds:?} after={after_bounds:?}"
    );
}
