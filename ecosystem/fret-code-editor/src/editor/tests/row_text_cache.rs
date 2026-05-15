use super::*;

#[test]
fn cached_row_text_hits_and_reuses_arc_for_repeated_calls() {
    let handle = CodeEditorHandle::new("hello\nworld");

    let (a, b, stats) = {
        let mut st = handle.state.borrow_mut();
        let (_range_a, a, _folds_a, _preedit_a, _spans_a) =
            paint::cached_row_text_with_range(&mut st, 0, 64);
        let (_range_b, b, _folds_b, _preedit_b, _spans_b) =
            paint::cached_row_text_with_range(&mut st, 0, 64);
        (a, b, st.cache_stats)
    };

    assert!(
        Arc::ptr_eq(&a, &b),
        "expected row text cache to reuse Arc<str>"
    );
    assert_eq!(stats.row_text_misses, 1);
    assert_eq!(stats.row_text_hits, 1);
}

#[test]
fn cached_row_text_invalidates_on_buffer_revision_change() {
    let handle = CodeEditorHandle::new("hello\nworld");

    let (before, after, resets) = {
        let mut st = handle.state.borrow_mut();
        let (_range, before, _folds, _preedit, _spans) =
            paint::cached_row_text_with_range(&mut st, 0, 64);

        let mut tx = st.buffer.transaction_begin();
        st.buffer
            .transaction_update(
                &mut tx,
                Edit::Insert {
                    at: 0,
                    text: "!".to_string(),
                },
            )
            .expect("edit");
        let _ = st.buffer.transaction_commit(tx);
        st.refresh_display_map();

        let (_range, after, _folds, _preedit, _spans) =
            paint::cached_row_text_with_range(&mut st, 0, 64);
        (before, after, st.cache_stats.row_text_resets)
    };

    assert!(
        !Arc::ptr_eq(&before, &after),
        "expected row text cache to invalidate when buffer revision changes"
    );
    assert!(resets > 0, "expected row text cache resets to be recorded");
}

#[test]
fn cached_row_text_lru_eviction_rebuilds_evicted_rows() {
    let handle = CodeEditorHandle::new("hello\nworld");

    let (first0, first1, second0, stats) = {
        let mut st = handle.state.borrow_mut();
        let (_range0, first0, _folds0, _preedit0, _spans0) =
            paint::cached_row_text_with_range(&mut st, 0, 1);
        let (_range1, first1, _folds1, _preedit1, _spans1) =
            paint::cached_row_text_with_range(&mut st, 1, 1);
        let (_range0, second0, _folds0, _preedit0, _spans0) =
            paint::cached_row_text_with_range(&mut st, 0, 1);
        (first0, first1, second0, st.cache_stats)
    };

    assert_eq!(first0.as_ref(), "hello");
    assert_eq!(first1.as_ref(), "world");
    assert!(
        !Arc::ptr_eq(&first0, &second0),
        "expected row 0 to be rebuilt after eviction under max_entries=1"
    );
    assert!(
        stats.row_text_evictions > 0,
        "expected at least one eviction"
    );
}

#[test]
fn paint_frame_cache_min_entries_tracks_visible_window_union() {
    assert_eq!(
        super::paint_frame_cache_min_entries(None, Some((10, 20))),
        11
    );
    assert_eq!(
        super::paint_frame_cache_min_entries(Some((10, 20)), Some((15, 25))),
        16
    );
    assert_eq!(
        super::paint_frame_cache_min_entries(Some((20, 30)), Some((10, 20))),
        21
    );
    assert_eq!(
        super::paint_frame_cache_min_entries(Some((0, 10)), Some((100, 110))),
        22
    );
    assert_eq!(super::paint_frame_cache_min_entries(Some((0, 10)), None), 0);
}

#[test]
fn begin_paint_frame_sets_cache_floor_from_actual_visible_rows() {
    let handle = CodeEditorHandle::new("hello\nworld");

    let (first, second, perf_frame) = {
        let mut st = handle.state.borrow_mut();
        st.paint_perf_enabled = true;
        st.begin_paint_frame(WindowedRowsPaintFrame {
            viewport_height: Px(520.0),
            offset_y: Px(0.0),
            row_height: Px(16.0),
            row_stride: Px(16.0),
            gap: Px(0.0),
            scroll_margin: Px(0.0),
            visible_start: 10,
            visible_end: 298,
        });
        let first = st.paint_frame_cache_min_entries;

        st.begin_paint_frame(WindowedRowsPaintFrame {
            viewport_height: Px(520.0),
            offset_y: Px(0.0),
            row_height: Px(16.0),
            row_stride: Px(16.0),
            gap: Px(0.0),
            scroll_margin: Px(0.0),
            visible_start: 0,
            visible_end: 288,
        });
        (first, st.paint_frame_cache_min_entries, st.paint_perf_frame)
    };

    assert_eq!(first, 289);
    assert_eq!(
        second, 299,
        "reverse scrolling must keep the previous and current visible windows resident"
    );
    assert_eq!(perf_frame.visible_rows, 289);
    assert_eq!(perf_frame.cache_frame_min_entries, 299);
}

#[cfg(feature = "syntax-rust")]
#[test]
fn prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint() {
    let text = "fn main() {\n    let x = 1;\n}\n".repeat(64);
    let handle = CodeEditorHandle::new(text);
    handle.set_language(Some(Arc::<str>::from("rust")));
    handle.state.borrow_mut().paint_perf_enabled = true;

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let bounds = editor_ui_bounds();
    let mut services = FakeServices::default();

    let _ = render_code_editor_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        handle.clone(),
        bounds,
    );

    let after_seed = handle.cache_size_snapshot();
    #[cfg(feature = "syntax")]
    let syntax_replayable_seed_entries = {
        let st = handle.state.borrow();
        st.row_scene_cache
            .values()
            .filter(|(entry, _)| entry.syntax_replay_key.is_some())
            .count()
    };
    assert!(
        after_seed.row_scene_cache_entries > 0,
        "expected first frame to seed replayable row scene cache"
    );
    #[cfg(feature = "syntax")]
    assert!(
        syntax_replayable_seed_entries > 0,
        "expected first frame to seed syntax-replayable row scene cache; sizes={after_seed:?}"
    );

    let before = handle.cache_stats();
    let resized_bounds = Rect::new(bounds.origin, Size::new(Px(704.0), bounds.size.height));
    let _ = render_code_editor_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        handle.clone(),
        resized_bounds,
    );
    let after = handle.cache_stats();

    let row_text_delta = after
        .row_text_get_calls
        .saturating_sub(before.row_text_get_calls);
    let scene_hits_delta = after
        .row_scene_hits
        .saturating_sub(before.row_scene_hits)
        .saturating_add(
            after
                .row_scene_fast_hits
                .saturating_sub(before.row_scene_fast_hits),
        );
    let perf = handle
        .paint_perf_frame()
        .expect("paint perf frame should be enabled in tests that set the env");

    assert!(
        perf.rows_scene_prepaint_planned > 0,
        "expected prepaint to create row scene replay plans"
    );
    assert_eq!(
        perf.rows_scene_prepaint_plan_used, perf.rows_scene_prepaint_planned,
        "paint should consume the prepaint replay plan for each planned row"
    );
    assert_eq!(
        perf.us_row_text, 0,
        "paint should not redo row text work for planned rows"
    );
    assert_eq!(
        perf.us_syntax_spans, 0,
        "paint should not redo syntax-span lookup for planned replay rows"
    );
    assert_eq!(
        perf.us_row_rich_cache_compare, 0,
        "paint should not probe rich row content for planned replay rows"
    );
    assert!(
        row_text_delta >= perf.rows_scene_prepaint_planned,
        "prepaint planning should account for row text work before paint"
    );
    assert!(
        scene_hits_delta >= perf.rows_scene_prepaint_planned,
        "prepaint planning should account for row scene cache hits"
    );
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

        let a = paint::cached_row_text(&mut st, 0, 8);
        let b = paint::cached_row_text(&mut st, 0, 8);

        assert_eq!(a.as_ref(), "hello");
        assert_eq!(b.as_ref(), "hello");
        assert_eq!(st.cache_stats.row_text_get_calls, 2);
        assert_eq!(st.cache_stats.row_text_hits, 1);
        assert_eq!(st.cache_stats.row_text_misses, 1);
    }
}

#[test]
fn code_wrap_policy_change_invalidates_row_text_cache() {
    let handle = CodeEditorHandle::new("ab_cd_ef");
    handle.set_soft_wrap_cols(Some(4));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Conservative,
        ),
    ));

    {
        let mut st = handle.state.borrow_mut();
        let (range, text, _, _, _) = paint::cached_row_text_with_range(&mut st, 0, 8);
        assert_eq!(range, 0..4);
        assert_eq!(
            text.as_ref(),
            "ab_c",
            "conservative wrap falls back to grapheme boundaries (no identifier knob)"
        );
    }

    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    {
        let mut st = handle.state.borrow_mut();
        let (range, text, _, _, _) = paint::cached_row_text_with_range(&mut st, 0, 8);
        assert_eq!(range, 0..3);
        assert_eq!(
            text.as_ref(),
            "ab_",
            "balanced wrap prefers breaking after '_' when near the wrap width"
        );
    }
}
