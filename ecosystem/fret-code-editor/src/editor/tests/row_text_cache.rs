use super::*;
#[cfg(feature = "syntax-rust")]
use crate::editor::syntax::ensure_syntax_row_cache_fresh;

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

#[test]
fn paint_perf_records_windowed_surface_diagnostics() {
    let handle = CodeEditorHandle::new("hello\nworld");

    let perf_frame = {
        let mut st = handle.state.borrow_mut();
        st.paint_perf_enabled = true;
        st.begin_paint_frame(WindowedRowsPaintFrame {
            viewport_height: Px(64.0),
            offset_y: Px(0.0),
            row_height: Px(16.0),
            row_stride: Px(16.0),
            gap: Px(0.0),
            scroll_margin: Px(0.0),
            visible_start: 0,
            visible_end: 3,
        });
        st.paint_perf_frame.us_total = 80;
        st.paint_perf_frame.ns_total = 80_000;
        st.record_windowed_rows_paint_diagnostics(WindowedRowsPaintDiagnostics {
            visible_start: 0,
            visible_end: 3,
            visible_rows: 4,
            rows_iterated: 4,
            rows_with_rect: 4,
            us_paint_callback: 130,
            us_frame_lookup: 2,
            us_on_paint_frame: 7,
            us_row_loop: 110,
            us_row_rect: 3,
            us_row_paint: 95,
            us_non_row: 35,
            ns_paint_callback: 130_000,
            ns_frame_lookup: 2_000,
            ns_on_paint_frame: 7_000,
            ns_row_loop: 110_000,
            ns_row_rect: 3_000,
            ns_row_paint: 95_000,
            ns_non_row: 35_000,
        });
        st.paint_perf_frame
    };

    assert_eq!(perf_frame.surface_rows_iterated, 4);
    assert_eq!(perf_frame.surface_rows_with_rect, 4);
    assert_eq!(perf_frame.us_windowed_surface_paint_callback, 130);
    assert_eq!(perf_frame.us_windowed_surface_row_paint, 95);
    assert_eq!(perf_frame.us_windowed_surface_non_row, 35);
    assert_eq!(perf_frame.us_windowed_surface_row_callback_gap, 15);
    assert_eq!(perf_frame.ns_windowed_surface_row_callback_gap, 15_000);
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

    let resized_bounds = Rect::new(bounds.origin, Size::new(Px(704.0), bounds.size.height));
    let (resized_visible_start, resized_visible_end) = {
        let st = handle.state.borrow();
        let start = *st
            .row_scene_cache
            .keys()
            .min()
            .expect("first frame should seed row scene cache entries");
        let end = *st
            .row_scene_cache
            .keys()
            .max()
            .expect("first frame should seed row scene cache entries");
        (start, end)
    };
    {
        let mut st = handle.state.borrow_mut();
        for row in [resized_visible_start, resized_visible_end] {
            if let Some((old, _)) = st.row_scene_cache.remove(&row) {
                st.row_scene_cache_scene_ops_len_total = st
                    .row_scene_cache_scene_ops_len_total
                    .saturating_sub(old.ops.len() as u64);
            }
        }
    }
    let before = handle.cache_stats();
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
    assert_eq!(
        perf.us_row_content_resolve, 0,
        "planned replay rows without overlays should not be attributed to row content resolve"
    );
    assert_eq!(
        perf.us_row_geom_resolve, 0,
        "planned replay rows without overlays should not re-resolve row geometry in paint"
    );
    assert_eq!(
        perf.us_row_overlay, 0,
        "planned replay rows without overlays should not run row overlay paint"
    );
    assert_eq!(
        perf.rows_scene_prepaint_skip_no_cache, 0,
        "prepaint should seed newly exposed edge rows before replay planning observes no-cache skips"
    );
    assert_eq!(
        perf.rows_scene_fast_miss_no_entry, 0,
        "paint should not miss the fast row scene cache because the edge row had no entry"
    );
    assert_eq!(
        perf.rows_scene_full_miss_no_entry, 0,
        "paint should not run the full row scene path because the edge row had no entry"
    );
    assert_eq!(
        perf.rows_scene_stored_at_visible_start, 0,
        "paint should not store the newly exposed visible-start row"
    );
    assert_eq!(
        perf.rows_scene_stored_at_visible_end, 0,
        "paint should not store the newly exposed visible-end row"
    );
    assert_eq!(
        perf.rows_scene_prepaint_edge_stored, 2,
        "prepaint should explicitly prebuild both missing visible edge rows"
    );
    assert!(
        perf.row_scene_prepaint_edge_ops_stored > 0,
        "prepaint edge storage should retain replayable scene ops"
    );
    assert_eq!(
        row_text_delta, perf.rows_scene_prepaint_edge_stored,
        "only prepaint edge prebuild should resolve row content for the missing cache entry"
    );
    assert!(
        scene_hits_delta >= perf.rows_scene_prepaint_planned,
        "prepaint planning should account for row scene cache hits"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn planned_replay_rows_with_selection_still_paint_overlay() {
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

    handle.set_selection(Selection {
        anchor: 0,
        focus: 8,
    });
    let before = handle.cache_stats();
    let _ = render_code_editor_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        handle.clone(),
        Rect::new(bounds.origin, Size::new(Px(704.0), bounds.size.height)),
    );
    let after = handle.cache_stats();
    let perf = handle
        .paint_perf_frame()
        .expect("paint perf frame should be enabled in tests that set the env");

    assert!(
        perf.rows_scene_prepaint_planned > 0,
        "expected prepaint to plan cached row scene entries"
    );
    assert_eq!(
        perf.rows_scene_prepaint_plan_used, perf.rows_scene_prepaint_planned,
        "paint should consume each planned row scene entry"
    );
    assert!(
        perf.quads_selection > 0,
        "selected planned-replay rows must still paint the selection overlay"
    );
    assert_eq!(
        after
            .row_text_get_calls
            .saturating_sub(before.row_text_get_calls),
        0,
        "selection overlay should reuse planned replay content snapshots"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows() {
    let text = "fn main() {\n    let x = 1;\n}\n".repeat(64);
    let handle = CodeEditorHandle::new(text);
    handle.set_language(Some(Arc::<str>::from("rust")));
    handle.debug_set_compose_inline_preedit(true);
    handle.set_selection(Selection {
        anchor: "fn main() {\n    let ".len(),
        focus: "fn main() {\n    let ".len(),
    });
    handle.set_preedit_debug("xy", Some((1, 1)));
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

    let before = handle.cache_stats();
    let _ = render_code_editor_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        handle.clone(),
        bounds,
    );
    let after = handle.cache_stats();
    let perf = handle
        .paint_perf_frame()
        .expect("paint perf frame should be enabled in tests that set the env");

    assert!(
        perf.rows_scene_prepaint_candidates > perf.rows_scene_prepaint_skip_preedit,
        "preedit should not suppress planning for unrelated visible rows"
    );
    assert!(
        perf.rows_scene_prepaint_skip_preedit > 0,
        "the composed preedit row must stay on the paint-time path"
    );
    assert!(
        perf.rows_scene_prepaint_planned > 0,
        "non-preedit rows should still use the prepaint replay plan"
    );
    assert_eq!(
        perf.rows_scene_prepaint_plan_used, perf.rows_scene_prepaint_planned,
        "paint should consume each planned non-preedit row"
    );
    assert!(
        after
            .row_scene_hits
            .saturating_sub(before.row_scene_hits)
            .saturating_add(
                after
                    .row_scene_fast_hits
                    .saturating_sub(before.row_scene_fast_hits),
            )
            >= perf.rows_scene_prepaint_planned,
        "prepaint planning should account for retained row-scene hits"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn prepaint_row_scene_replay_plan_uses_cached_syntax_replay_context() {
    let handle = CodeEditorHandle::new("fn main() {}\n");
    handle.set_language(Some(Arc::<str>::from("rust")));
    handle.state.borrow_mut().paint_perf_enabled = true;

    let mut st = handle.state.borrow_mut();
    ensure_syntax_row_cache_fresh(&mut st);
    st.sync_row_scene_cache_epoch();

    let fg = Color {
        r: 0.2,
        g: 0.3,
        b: 0.4,
        a: 1.0,
    };
    let theme_revision = 17;
    let scale_factor = 1.0;
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: Px(14.0),
        ..Default::default()
    };
    let content_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(640.0), Px(16.0)));
    let cell_w = Px(8.0);
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(4096.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let line = Arc::<str>::from("fn main() {}");
    let row_range = 0..line.len();
    let row_spans = Arc::<[fret_code_editor_view::DisplayRowSpan]>::from([]);
    let syntax_spans = Arc::<[SyntaxSpan]>::from(vec![SyntaxSpan {
        range: 0..2,
        highlight: "keyword",
    }]);
    let content = Arc::new(RowContentSnapshot {
        text: Arc::clone(&line),
        range: row_range.clone(),
        fold_map: None,
        preedit_range: None,
        row_spans: Arc::clone(&row_spans),
    });
    let rich = AttributedText::new(
        Arc::clone(&line),
        vec![TextSpan {
            len: line.len(),
            shaping: st.code_font_shaping_style.clone(),
            paint: Default::default(),
        }],
    );
    let row_geom_key = geom::RowGeomKey::for_attributed(
        &rich,
        &text_style,
        (
            constraints.max_width,
            constraints.wrap,
            constraints.overflow,
            fret_core::TextAlign::Start,
            scale_factor,
        ),
        st.font_stack_key,
    );
    let syntax_replay_key = RowSceneSyntaxReplayKey::new(
        row_range.clone(),
        Arc::clone(&line),
        Arc::clone(&row_spans),
        Arc::clone(&syntax_spans),
        &text_style,
        constraints,
        st.font_stack_key,
        scale_factor,
        theme_revision,
        st.code_font_feature_policy_rev,
        fg,
    );
    let row_scene_key = RowSceneKey::syntax(row_geom_key.clone(), fg, theme_revision);
    let geom = RowGeom {
        row_range,
        key: row_geom_key,
        caret_stops: Vec::new(),
        fold_map: None,
        caret_rect_top: None,
        caret_rect_height: None,
        has_preedit: false,
        preedit: None,
    };
    let ops = Arc::<[SceneOp]>::from(vec![SceneOp::Quad {
        order: DrawOrder(2),
        rect: content_bounds,
        background: fret_core::Paint::Solid(fg).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    }]);
    let hosted_resources = fret_ui::canvas::CanvasHostedResources::from_scene_ops(ops.as_ref());
    st.row_scene_cache.insert(
        0,
        (
            RowSceneCacheEntry {
                key: row_scene_key,
                content,
                origin: content_bounds.origin,
                geom,
                is_rich: true,
                ops,
                hosted_resources,
                syntax_replay_key: Some(syntax_replay_key),
            },
            1,
        ),
    );
    st.row_scene_cache_tick = 1;
    st.cache_stats.syntax_get_calls = 0;
    st.paint_perf_frame = CodeEditorPaintPerfFrame::default();

    let frame = WindowedRowsPaintFrame {
        viewport_height: Px(16.0),
        offset_y: Px(0.0),
        row_height: Px(16.0),
        row_stride: Px(16.0),
        gap: Px(0.0),
        scroll_margin: Px(0.0),
        visible_start: 0,
        visible_end: 0,
    };
    st.begin_paint_frame(frame);
    let plan = paint::prepaint_row_scene_replay_plan_for_frame(
        &mut st,
        frame,
        content_bounds,
        cell_w,
        64,
        &text_style,
        fg,
        theme_revision,
        scale_factor,
    );

    assert_eq!(plan.entries.len(), 1);
    assert_eq!(
        st.cache_stats.syntax_get_calls, 0,
        "planner should trust cached replay context instead of looking up syntax spans"
    );
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_candidates, 1);
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_planned, 1);
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_skip_key_mismatch, 0);
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_skip_no_cache, 0);
}

#[cfg(feature = "syntax-rust")]
#[test]
fn prepaint_row_scene_replay_plan_handles_plain_cached_rows() {
    let handle = CodeEditorHandle::new("    \n".repeat(256));
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

    let plain_seed_entries = {
        let st = handle.state.borrow();
        st.row_scene_cache
            .values()
            .filter(|(entry, _)| {
                entry.syntax_replay_key.is_none()
                    && matches!(entry.key.paint_key, RowScenePaintKey::Plain { .. })
            })
            .count()
    };
    assert!(
        plain_seed_entries > 0,
        "expected first frame to seed plain row scene cache entries"
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
        "expected prepaint to plan cached plain row scene entries"
    );
    assert_eq!(
        perf.rows_scene_prepaint_plan_used, perf.rows_scene_prepaint_planned,
        "paint should consume each planned plain row scene entry"
    );
    assert_eq!(
        row_text_delta, 0,
        "prepaint planning should reuse cached plain row content snapshots"
    );
    assert!(
        scene_hits_delta >= perf.rows_scene_prepaint_planned,
        "prepaint planning should account for plain row scene cache hits"
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

#[test]
fn code_font_feature_policy_is_idempotent_for_same_value() {
    let handle = CodeEditorHandle::new("hello\nworld");
    let policy = CodeFontFeaturePolicy::default();
    handle.set_code_font_feature_policy(policy.clone());

    let (_range, cached_text, policy_rev_before, stats_before) = {
        let mut st = handle.state.borrow_mut();
        let (range, text, _, _, _) = paint::cached_row_text_with_range(&mut st, 0, 64);
        (range, text, st.code_font_feature_policy_rev, st.cache_stats)
    };

    handle.set_code_font_feature_policy(policy);

    let (_range_after, cached_text_after, policy_rev_after, stats_after) = {
        let mut st = handle.state.borrow_mut();
        let (range, text, _, _, _) = paint::cached_row_text_with_range(&mut st, 0, 64);
        (range, text, st.code_font_feature_policy_rev, st.cache_stats)
    };

    assert_eq!(
        policy_rev_after, policy_rev_before,
        "idempotent set_code_font_feature_policy must not bump the policy revision"
    );
    assert_eq!(
        stats_after.row_text_resets, stats_before.row_text_resets,
        "idempotent set_code_font_feature_policy must not reset row text caches"
    );
    assert_eq!(
        stats_after.row_scene_resets, stats_before.row_scene_resets,
        "idempotent set_code_font_feature_policy must not reset row scene caches"
    );
    assert!(
        Arc::ptr_eq(&cached_text, &cached_text_after),
        "idempotent set_code_font_feature_policy must preserve cached row text"
    );
}
