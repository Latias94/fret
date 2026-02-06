use super::*;
use fret_core::HitTestResult;
use slotmap::KeyData;
use std::collections::HashMap;

#[derive(Default)]
struct PlatformTextTestServices {
    next_blob: u64,
    by_blob_text: HashMap<fret_core::TextBlobId, String>,
}

impl fret_core::TextService for PlatformTextTestServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let blob = fret_core::TextBlobId::from(KeyData::from_ffi(self.next_blob));
        self.next_blob = self.next_blob.wrapping_add(1);
        self.by_blob_text.insert(blob, input.text().to_string());

        let w = constraints
            .max_width
            .map(|w| w.0.max(0.0))
            .unwrap_or(1000.0);
        (
            blob,
            TextMetrics {
                size: Size::new(Px(w), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn caret_rect(
        &mut self,
        _blob: fret_core::TextBlobId,
        index: usize,
        _affinity: fret_core::CaretAffinity,
    ) -> Rect {
        Rect::new(
            Point::new(Px(index as f32), Px(0.0)),
            Size::new(Px(1.0), Px(10.0)),
        )
    }

    fn selection_rects(
        &mut self,
        _blob: fret_core::TextBlobId,
        (a, b): (usize, usize),
        out: &mut Vec<Rect>,
    ) {
        out.clear();
        let (start, end) = (a.min(b), a.max(b));
        out.push(Rect::new(
            Point::new(Px(start as f32), Px(0.0)),
            Size::new(Px((end - start) as f32), Px(10.0)),
        ));
    }

    fn hit_test_point(&mut self, blob: fret_core::TextBlobId, point: Point) -> HitTestResult {
        let len = self.by_blob_text.get(&blob).map(|s| s.len()).unwrap_or(0);

        let mut idx = point.x.0.floor().max(0.0) as usize;
        idx = idx.min(len);

        let text = self
            .by_blob_text
            .get(&blob)
            .map(|s| s.as_str())
            .unwrap_or("");
        while idx > 0 && !text.is_char_boundary(idx) {
            idx = idx.saturating_sub(1);
        }

        HitTestResult {
            index: idx,
            affinity: fret_core::CaretAffinity::Downstream,
        }
    }

    fn release(&mut self, blob: fret_core::TextBlobId) {
        self.by_blob_text.remove(&blob);
    }
}

impl fret_core::PathService for PlatformTextTestServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for PlatformTextTestServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

#[test]
fn platform_text_input_query_can_get_text_ranges_bounds_and_replace() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut input = crate::text_input::TextInput::new().with_text("a😀b");
    let style = crate::TextInputStyle {
        padding: Edges::all(Px(0.0)),
        ..Default::default()
    };
    input.set_chrome_style(style);

    let root = ui.create_node(TestStack);
    let text = ui.create_node(input);
    ui.add_child(root, text);
    ui.set_root(root);
    ui.set_focus(Some(text));

    let mut services = PlatformTextTestServices::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(20.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    // Select the emoji (UTF-8 bytes [1,5)).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::SetTextSelection {
            anchor: 1,
            focus: 1 + "😀".len() as u32,
        },
    );

    let selected = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::SelectedTextRange,
    );
    assert_eq!(
        selected,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            1, 3
        )))
    );

    let marked = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::MarkedTextRange,
    );
    assert_eq!(
        marked,
        fret_runtime::PlatformTextInputQueryResult::Range(None)
    );

    let text_for = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::TextForRange {
            range: fret_runtime::Utf16Range::new(1, 3),
        },
    );
    assert_eq!(
        text_for,
        fret_runtime::PlatformTextInputQueryResult::Text(Some("😀".to_string()))
    );

    let bounds_for = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::BoundsForRange {
            range: fret_runtime::Utf16Range::new(1, 1),
        },
    );
    let fret_runtime::PlatformTextInputQueryResult::Bounds(Some(rect)) = bounds_for else {
        panic!("expected Bounds(Some(_)), got {bounds_for:?}");
    };
    assert!((rect.origin.x.0 - 1.0).abs() < 0.01);
    assert!((rect.origin.y.0 - 5.0).abs() < 0.01);

    let idx = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::CharacterIndexForPoint {
            point: Point::new(Px(2.0), Px(6.0)),
        },
    );
    assert_eq!(
        idx,
        fret_runtime::PlatformTextInputQueryResult::Index(Some(1))
    );

    assert!(ui.platform_text_input_replace_text_in_range_utf16(
        &mut app,
        &mut services,
        1.0,
        fret_runtime::Utf16Range::new(1, 3),
        "X"
    ));

    let full = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::TextForRange {
            range: fret_runtime::Utf16Range::new(0, 1000),
        },
    );
    assert_eq!(
        full,
        fret_runtime::PlatformTextInputQueryResult::Text(Some("aXb".to_string()))
    );
}

#[test]
fn platform_text_input_replace_and_mark_can_drive_caret_anchored_preedit() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut input = crate::text_input::TextInput::new().with_text("abc");
    let style = crate::TextInputStyle {
        padding: Edges::all(Px(0.0)),
        ..Default::default()
    };
    input.set_chrome_style(style);

    let root = ui.create_node(TestStack);
    let text = ui.create_node(input);
    ui.add_child(root, text);
    ui.set_root(root);
    ui.set_focus(Some(text));

    let mut services = PlatformTextTestServices::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(20.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::SetTextSelection {
            anchor: 1,
            focus: 1,
        },
    );

    assert!(ui.platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut app,
        &mut services,
        1.0,
        fret_runtime::Utf16Range::new(1, 1),
        "X",
        Some(fret_runtime::Utf16Range::new(1, 2))
    ));

    let marked = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::MarkedTextRange,
    );
    assert_eq!(
        marked,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            1, 2
        )))
    );

    let selected = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::SelectedTextRange,
    );
    assert_eq!(
        selected,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            2, 2
        )))
    );

    let text_for = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::TextForRange {
            range: fret_runtime::Utf16Range::new(1, 2),
        },
    );
    assert_eq!(
        text_for,
        fret_runtime::PlatformTextInputQueryResult::Text(Some("X".to_string()))
    );

    assert!(ui.platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut app,
        &mut services,
        1.0,
        fret_runtime::Utf16Range::new(1, 2),
        "XY",
        Some(fret_runtime::Utf16Range::new(1, 3))
    ));

    let marked = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::MarkedTextRange,
    );
    assert_eq!(
        marked,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            1, 3
        )))
    );

    let selected = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::SelectedTextRange,
    );
    assert_eq!(
        selected,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            3, 3
        )))
    );

    assert!(ui.platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut app,
        &mut services,
        1.0,
        fret_runtime::Utf16Range::new(1, 3),
        "Z",
        None
    ));

    let full = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::TextForRange {
            range: fret_runtime::Utf16Range::new(0, 1000),
        },
    );
    assert_eq!(
        full,
        fret_runtime::PlatformTextInputQueryResult::Text(Some("aZbc".to_string()))
    );

    let marked = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::MarkedTextRange,
    );
    assert_eq!(
        marked,
        fret_runtime::PlatformTextInputQueryResult::Range(None)
    );
}
