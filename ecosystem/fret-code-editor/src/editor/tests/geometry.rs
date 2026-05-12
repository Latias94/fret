use super::support::*;
use super::*;
use crate::editor::geom::map_row_display_local_to_buffer_byte;

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
fn caret_stops_hit_test_handles_decreasing_x() {
    let stops = vec![(0, Px(30.0)), (1, Px(20.0)), (2, Px(10.0)), (3, Px(0.0))];
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(35.0)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(30.0)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(24.0)), 1);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(15.0)), 1);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(6.0)), 2);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(-5.0)), 3);
}

#[test]
fn caret_stops_hit_test_handles_non_monotonic_x() {
    // Non-monotonic caret stops can happen on mixed-direction lines (bidi).
    let stops = vec![(0, Px(0.0)), (1, Px(30.0)), (2, Px(10.0)), (3, Px(20.0))];
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(-100.0)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(9.0)), 2);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(11.0)), 2);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(19.0)), 3);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(21.0)), 3);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(999.0)), 1);
}

#[test]
fn map_row_display_local_to_buffer_byte_snaps_inside_preedit() {
    let doc = DocId::new();
    let buffer = TextBuffer::new(doc, "hello".to_string()).unwrap();
    let geom = RowGeom {
        row_range: 0..buffer.len_bytes(),
        key: row_geom_key_for_tests(&Arc::from("hello")),
        caret_stops: Vec::new(),
        fold_map: None,
        caret_rect_top: None,
        caret_rect_height: None,
        has_preedit: true,
        preedit: Some(RowPreeditMapping {
            insert_at: 2,
            preedit_len: 2,
        }),
    };

    // Before the injection point maps 1:1.
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 0), 0);
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 2), 2);

    // Inside the injected preedit snaps to the injection point.
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 3), 2);

    // After the injected preedit shifts by `preedit_len`.
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 4), 2);
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 5), 3);
}

#[test]
fn row_fold_map_maps_between_buffer_and_display() {
    let map = geom::RowFoldMap::new(vec![geom::RowFoldSpan {
        buffer_range: 1..4,
        // U+2026 is 3 bytes in UTF-8, so a placeholder at offset 1 occupies [1,4).
        display_range: 1..4,
    }]);

    assert_eq!(map.buffer_local_to_display_local(0), 0);
    assert_eq!(map.buffer_local_to_display_local(1), 1);
    assert_eq!(map.buffer_local_to_display_local(2), 1);
    assert_eq!(map.buffer_local_to_display_local(3), 1);
    assert_eq!(map.buffer_local_to_display_local(4), 4);
    assert_eq!(map.buffer_local_to_display_local(5), 5);

    assert_eq!(map.display_local_to_buffer_local(0), 0);
    assert_eq!(map.display_local_to_buffer_local(1), 1);
    assert_eq!(map.display_local_to_buffer_local(2), 1);
    assert_eq!(map.display_local_to_buffer_local(3), 1);
    assert_eq!(map.display_local_to_buffer_local(4), 4);
    assert_eq!(map.display_local_to_buffer_local(5), 5);
}

#[test]
fn row_fold_map_handles_inlay_insertions() {
    let map = geom::RowFoldMap::new(vec![geom::RowFoldSpan {
        buffer_range: 2..2,
        display_range: 2..6,
    }]);

    assert_eq!(map.buffer_local_to_display_local(2), 2);
    assert_eq!(map.buffer_local_to_display_local(3), 7);

    assert_eq!(map.display_local_to_buffer_local(2), 2);
    assert_eq!(map.display_local_to_buffer_local(3), 2);
    assert_eq!(map.display_local_to_buffer_local(6), 2);
    assert_eq!(map.display_local_to_buffer_local(7), 3);
}

#[test]
fn row_geom_key_ignores_paint_only_changes() {
    let text: Arc<str> = Arc::<str>::from("let x = 1;");
    let base = TextStyle::default();
    let constraints = (
        Some(Px(200.0)),
        TextWrap::None,
        TextOverflow::Clip,
        fret_core::TextAlign::Start,
        1.0,
    );
    let font_stack_key = fret_runtime::TextFontStackKey(7);

    let mk_rich = |kw: Color, ident: Color| {
        let spans = vec![
            TextSpan {
                len: "let".len(),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    fg: Some(kw),
                    ..Default::default()
                },
            },
            TextSpan::new(" ".len()),
            TextSpan {
                len: "x".len(),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    fg: Some(ident),
                    ..Default::default()
                },
            },
            TextSpan::new(" = 1;".len()),
        ];
        AttributedText::new(Arc::clone(&text), Arc::<[TextSpan]>::from(spans))
    };

    let rich_a = mk_rich(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    );
    let rich_b = mk_rich(
        Color {
            r: 0.2,
            g: 0.2,
            b: 1.0,
            a: 1.0,
        },
        Color {
            r: 0.8,
            g: 0.8,
            b: 0.0,
            a: 1.0,
        },
    );

    assert!(
        rich_a.shaping_eq(&rich_b),
        "sanity: shaping_eq should ignore paint-only changes"
    );

    let key_a = geom::RowGeomKey::for_attributed(&rich_a, &base, constraints, font_stack_key);
    let key_b = geom::RowGeomKey::for_attributed(&rich_b, &base, constraints, font_stack_key);
    assert_eq!(
        key_a, key_b,
        "row geometry cache key must ignore paint-only changes"
    );

    let mut spans_c = rich_b.spans.as_ref().to_vec();
    spans_c[0].shaping = spans_c[0]
        .shaping
        .clone()
        .with_weight(fret_core::FontWeight(700));
    let rich_c = AttributedText::new(Arc::clone(&text), Arc::<[TextSpan]>::from(spans_c));
    let key_c = geom::RowGeomKey::for_attributed(&rich_c, &base, constraints, font_stack_key);
    assert_ne!(key_a, key_c, "shaping changes must affect geometry key");
}

#[test]
fn row_geom_key_buckets_max_width_for_unwrapped_start_aligned_rows() {
    let text: Arc<str> = Arc::<str>::from("hello");
    let style = TextStyle::default();
    let font_stack_key = fret_runtime::TextFontStackKey(1);

    let key_a = geom::RowGeomKey::for_plain(
        &text,
        &style,
        (
            Some(Px(100.0)),
            TextWrap::None,
            TextOverflow::Clip,
            fret_core::TextAlign::Start,
            1.0,
        ),
        font_stack_key,
    );
    let key_b = geom::RowGeomKey::for_plain(
        &text,
        &style,
        (
            Some(Px(120.0)),
            TextWrap::None,
            TextOverflow::Clip,
            fret_core::TextAlign::Start,
            1.0,
        ),
        font_stack_key,
    );
    assert_eq!(
        key_a, key_b,
        "expected small max_width changes to be bucketed for unwrapped rows"
    );

    let key_c = geom::RowGeomKey::for_plain(
        &text,
        &style,
        (
            Some(Px(120.0)),
            TextWrap::None,
            TextOverflow::Clip,
            fret_core::TextAlign::Center,
            1.0,
        ),
        font_stack_key,
    );
    assert_ne!(
        key_a, key_c,
        "expected non-start alignment to preserve exact max_width bits"
    );
}
