use super::*;
use crate::editor::{a11y, geom};

#[test]
fn platform_text_input_bounds_and_index_roundtrip_under_preedit_replacement_and_wrap() {
    let text = "a😀bcdefghij";
    let handle = CodeEditorHandle::new(text);
    handle.set_soft_wrap_cols(Some(6));

    let replace_start = text.find('b').expect("expected 'b' in text");
    let replace_end = replace_start.saturating_add("bc".len()).min(text.len());
    handle.set_selection(Selection {
        anchor: replace_start,
        focus: replace_end,
    });
    handle.debug_platform_set_marked_text_for_selection("XY");

    let scroll = fret_ui::scroll::ScrollHandle::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );
    let row_h = Px(20.0);
    let cell_w = Px(10.0);

    let mut st = handle.state.borrow_mut();
    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    let after_preedit = value
        .find("XY")
        .map(|start| start + "XY".len())
        .expect("expected preedit text in composed a11y value");

    let end_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
        value.as_str(),
        after_preedit,
        fret_core::utf::UtfIndexClamp::Down,
    ) as u32;
    let (_, end_byte) = fret_core::utf::utf16_range_to_utf8_byte_range(
        value.as_str(),
        usize::try_from(end_utf16).unwrap_or(usize::MAX),
        usize::try_from(end_utf16).unwrap_or(usize::MAX),
    );
    let end_byte = u32::try_from(end_byte.min(value.len())).unwrap_or(u32::MAX);

    let buf_byte = a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, end_byte);

    // Seed geometry cache for the row so platform queries exercise the cached caret-stop path.
    let pt = st.display_map.byte_to_display_point(&st.buffer, buf_byte);
    let (row_range, row_text, fold_map, preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, pt.row, 1024);

    let mut caret_stops: Vec<(usize, Px)> = Vec::with_capacity(row_text.len().saturating_add(2));
    caret_stops.push((0, Px(0.0)));
    for (i, _) in row_text.char_indices() {
        caret_stops.push((i, Px(i as f32 * cell_w.0)));
    }
    caret_stops.push((row_text.len(), Px(row_text.len() as f32 * cell_w.0)));
    caret_stops.sort_by_key(|(i, _)| *i);
    caret_stops.dedup_by_key(|(i, _)| *i);

    st.row_geom_cache.insert(
        pt.row,
        (
            RowGeom {
                row_range,
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: Some(Px(0.0)),
                caret_rect_height: Some(row_h),
                has_preedit: preedit_range.is_some(),
                preedit: None,
            },
            1,
        ),
    );

    // BoundsForRange-like path: map a11y value UTF-16 -> a11y UTF-8 -> buffer byte -> caret rect.
    let rect =
        geom::caret_rect_for_buffer_byte_boundary(&st, row_h, cell_w, bounds, &scroll, buf_byte)
            .expect("caret rect for buffer byte boundary");

    // CharacterIndexForPoint-like path: hit test back to a UTF-16 index and ensure it round-trips.
    let point = Point::new(
        Px(rect.origin.x.0 + 1.0),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    );
    let local_y = (point.y.0 - bounds.origin.y.0 + scroll.offset().y.0).max(0.0);
    let row = (local_y / row_h.0).floor().max(0.0) as usize;

    let hit_byte = geom::caret_for_pointer(&mut st, row, bounds, point, cell_w);
    assert_eq!(
        hit_byte, buf_byte,
        "expected hit-testing inside the caret rect to map back to the same buffer byte boundary"
    );

    let a11y_offset = a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, hit_byte);
    let got_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
        value.as_str(),
        usize::try_from(a11y_offset).unwrap_or(usize::MAX),
        fret_core::utf::UtfIndexClamp::Down,
    ) as u32;
    assert_eq!(got_utf16, end_utf16);
}

#[test]
fn platform_text_input_bounds_and_index_roundtrip_under_inline_preedit_composed_window_and_wrap() {
    let text = "a😀bcdefghij";
    let handle = CodeEditorHandle::new(text);
    handle.set_soft_wrap_cols(Some(6));
    handle.set_compose_inline_preedit(true);

    let caret = text.find('b').expect("expected 'b' in text");
    handle.set_caret(caret);
    handle.set_preedit_debug("XY", Some((1, 1)));

    let scroll = fret_ui::scroll::ScrollHandle::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );
    let row_h = Px(20.0);
    let cell_w = Px(10.0);

    let mut st = handle.state.borrow_mut();
    assert!(
        st.compose_inline_preedit,
        "expected inline preedit composition enabled"
    );
    assert!(
        st.preedit_replace_range
            .as_ref()
            .map_or(true, |r| r.is_empty()),
        "expected non-replacing preedit so a11y uses the composed display window"
    );

    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    assert!(
        value.contains("XY"),
        "expected preedit injection in a11y value"
    );

    // Pick a boundary that is after the injected preedit segment so the mapping must account
    // for the global offset shift (bytes after caret include the preedit string).
    let after_target = value
        .find('h')
        .map(|start| start + 'h'.len_utf8())
        .expect("expected 'h' in composed a11y value");

    let end_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
        value.as_str(),
        after_target,
        fret_core::utf::UtfIndexClamp::Down,
    ) as u32;
    let (_, end_byte) = fret_core::utf::utf16_range_to_utf8_byte_range(
        value.as_str(),
        usize::try_from(end_utf16).unwrap_or(usize::MAX),
        usize::try_from(end_utf16).unwrap_or(usize::MAX),
    );
    let end_byte = u32::try_from(end_byte.min(value.len())).unwrap_or(u32::MAX);

    let buf_byte = a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, end_byte);
    let expected_buf_byte = text
        .find('h')
        .map(|start| start + 'h'.len_utf8())
        .expect("expected 'h' in base buffer text");
    assert_eq!(
        buf_byte, expected_buf_byte,
        "expected mapping from composed a11y offset to land on the matching buffer boundary"
    );

    // Seed geometry cache for the row so platform queries exercise the cached caret-stop path.
    let pt = st.display_map.byte_to_display_point(&st.buffer, buf_byte);
    let (row_range, row_text, fold_map, preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, pt.row, 1024);

    let mut caret_stops: Vec<(usize, Px)> = Vec::with_capacity(row_text.len().saturating_add(2));
    caret_stops.push((0, Px(0.0)));
    for (i, _) in row_text.char_indices() {
        caret_stops.push((i, Px(i as f32 * cell_w.0)));
    }
    caret_stops.push((row_text.len(), Px(row_text.len() as f32 * cell_w.0)));
    caret_stops.sort_by_key(|(i, _)| *i);
    caret_stops.dedup_by_key(|(i, _)| *i);

    st.row_geom_cache.insert(
        pt.row,
        (
            RowGeom {
                row_range,
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: Some(Px(0.0)),
                caret_rect_height: Some(row_h),
                has_preedit: preedit_range.is_some(),
                preedit: None,
            },
            1,
        ),
    );

    // BoundsForRange-like path: map a11y value UTF-16 -> a11y UTF-8 -> buffer byte -> caret rect.
    let rect =
        geom::caret_rect_for_buffer_byte_boundary(&st, row_h, cell_w, bounds, &scroll, buf_byte)
            .expect("caret rect for buffer byte boundary");

    // CharacterIndexForPoint-like path: hit test back to a UTF-16 index and ensure it round-trips.
    let point = Point::new(
        Px(rect.origin.x.0 + 1.0),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    );
    let local_y = (point.y.0 - bounds.origin.y.0 + scroll.offset().y.0).max(0.0);
    let row = (local_y / row_h.0).floor().max(0.0) as usize;

    let hit_byte = geom::caret_for_pointer(&mut st, row, bounds, point, cell_w);
    assert_eq!(
        hit_byte, buf_byte,
        "expected hit-testing inside the caret rect to map back to the same buffer byte boundary"
    );

    let a11y_offset = a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, hit_byte);
    let got_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
        value.as_str(),
        usize::try_from(a11y_offset).unwrap_or(usize::MAX),
        fret_core::utf::UtfIndexClamp::Down,
    ) as u32;
    assert_eq!(got_utf16, end_utf16);
}

#[test]
fn platform_text_input_bounds_and_index_roundtrip_under_inline_preedit_composed_window_with_decorations_and_wrap()
 {
    let text = "ab_cd_efghij😀kl";
    let handle = CodeEditorHandle::new(text);
    handle.set_soft_wrap_cols(Some(6));
    handle.set_compose_inline_preedit(true);
    handle.set_allow_decorations_under_inline_preedit(true);
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 2..5,
            placeholder: Arc::<str>::from("…"),
        }],
    );
    handle.set_line_inlays(
        0,
        vec![InlaySpan {
            byte: 1,
            text: Arc::<str>::from("<inlay>"),
        }],
    );

    let caret = text.find('e').expect("expected 'e' in text");
    handle.set_caret(caret);
    handle.set_preedit_debug("XY", Some((1, 1)));

    let scroll = fret_ui::scroll::ScrollHandle::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );
    let row_h = Px(20.0);
    let cell_w = Px(10.0);

    let mut st = handle.state.borrow_mut();
    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    assert!(value.contains("<inlay>"));
    assert!(value.contains("…"));
    assert!(value.contains("XY"));

    let buf_byte = text
        .find('h')
        .map(|start| start + 'h'.len_utf8())
        .expect("expected 'h' in base buffer text");

    let a11y_offset = a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, buf_byte);
    let end_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
        value.as_str(),
        usize::try_from(a11y_offset).unwrap_or(usize::MAX),
        fret_core::utf::UtfIndexClamp::Down,
    ) as u32;
    let (_, end_byte) = fret_core::utf::utf16_range_to_utf8_byte_range(
        value.as_str(),
        usize::try_from(end_utf16).unwrap_or(usize::MAX),
        usize::try_from(end_utf16).unwrap_or(usize::MAX),
    );
    let end_byte = u32::try_from(end_byte.min(value.len())).unwrap_or(u32::MAX);

    let mapped_buf = a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, end_byte);
    assert_eq!(
        mapped_buf, buf_byte,
        "expected composed a11y offset mapping to round-trip for buffer boundaries beyond folds/inlays/preedit"
    );

    // Seed geometry cache for the row so platform queries exercise the cached caret-stop path.
    let pt = st.display_map.byte_to_display_point(&st.buffer, buf_byte);
    let (row_range, row_text, fold_map, preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, pt.row, 1024);

    let mut caret_stops: Vec<(usize, Px)> = Vec::with_capacity(row_text.len().saturating_add(2));
    caret_stops.push((0, Px(0.0)));
    for (i, _) in row_text.char_indices() {
        caret_stops.push((i, Px(i as f32 * cell_w.0)));
    }
    caret_stops.push((row_text.len(), Px(row_text.len() as f32 * cell_w.0)));
    caret_stops.sort_by_key(|(i, _)| *i);
    caret_stops.dedup_by_key(|(i, _)| *i);

    st.row_geom_cache.insert(
        pt.row,
        (
            RowGeom {
                row_range,
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: Some(Px(0.0)),
                caret_rect_height: Some(row_h),
                has_preedit: preedit_range.is_some(),
                preedit: None,
            },
            1,
        ),
    );

    let rect =
        geom::caret_rect_for_buffer_byte_boundary(&st, row_h, cell_w, bounds, &scroll, buf_byte)
            .expect("caret rect for buffer byte boundary");
    let point = Point::new(
        Px(rect.origin.x.0 + 1.0),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    );
    let local_y = (point.y.0 - bounds.origin.y.0 + scroll.offset().y.0).max(0.0);
    let row = (local_y / row_h.0).floor().max(0.0) as usize;

    let hit_byte = geom::caret_for_pointer(&mut st, row, bounds, point, cell_w);
    assert_eq!(hit_byte, buf_byte);

    let got_a11y = a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, hit_byte);
    let got_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
        value.as_str(),
        usize::try_from(got_a11y).unwrap_or(usize::MAX),
        fret_core::utf::UtfIndexClamp::Down,
    ) as u32;
    assert_eq!(got_utf16, end_utf16);
}
