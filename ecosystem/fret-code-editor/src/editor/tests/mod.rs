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
mod caret_navigation;
mod display_navigation;
mod feature_payloads;
mod geometry;
mod keyboard_commands;
mod platform_text_input;
mod platform_text_input_roundtrip;
mod pointer_helpers;
mod pointer_selection;
mod preedit_paint;
mod row_geom_cache;
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
