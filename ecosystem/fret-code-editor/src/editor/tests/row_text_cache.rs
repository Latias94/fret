use super::*;
#[cfg(feature = "syntax-rust")]
use crate::editor::syntax::ensure_syntax_row_cache_fresh;

#[cfg(test)]
fn test_text_blob_id(raw: u64) -> fret_core::TextBlobId {
    fret_core::TextBlobId::from(slotmap::KeyData::from_ffi(raw))
}

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
fn single_line_edit_preserves_unaffected_row_text_cache_entries() {
    let handle = CodeEditorHandle::new("hello\nworld\nagain");

    let before = {
        let mut st = handle.state.borrow_mut();
        (0..3)
            .map(|row| {
                let (range, text, _folds, _preedit, _spans) =
                    paint::cached_row_text_with_range(&mut st, row, 64);
                (row, range, text)
            })
            .collect::<Vec<_>>()
    };

    let row_text_resets_before = handle.cache_stats().row_text_resets;
    {
        let mut st = handle.state.borrow_mut();
        crate::editor::input::apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            Edit::Insert {
                at: 0,
                text: "!".to_string(),
            },
            Selection {
                anchor: 1,
                focus: 1,
            },
        )
        .expect("edit should apply");

        assert_eq!(st.row_text_cache_rev, st.buffer.revision());
        assert_eq!(
            st.cache_stats.row_text_resets, row_text_resets_before,
            "single-line edits should delta-update row text cache instead of forcing a reset"
        );
        assert!(
            !st.row_text_cache.contains_key(&0),
            "edited row text must be rebuilt"
        );

        for (row, old_range, old_text) in before.into_iter().skip(1) {
            let (snapshot, _) = st
                .row_text_cache
                .get(&row)
                .expect("unaffected row should remain cached");
            assert!(
                Arc::ptr_eq(&snapshot.text, &old_text),
                "unaffected row {row} should keep its text allocation"
            );
            assert_eq!(
                snapshot.range,
                (old_range.start + 1)..(old_range.end + 1),
                "unaffected row {row} range should shift by inserted bytes"
            );
        }
    }
}

#[derive(Debug)]
struct SeededPlainRowScene {
    row: usize,
    range: std::ops::Range<usize>,
    text: Arc<str>,
    key: RowSceneKey,
    tick: u64,
    chunk_fingerprint: u64,
    hosted_text_blobs: Vec<fret_core::TextBlobId>,
}

#[test]
fn single_line_edit_preserves_unaffected_row_scene_cache_entries() {
    let handle = CodeEditorHandle::new("hello\nworld\nagain");
    let fg = Color {
        r: 0.2,
        g: 0.3,
        b: 0.4,
        a: 1.0,
    };

    let before = {
        let mut st = handle.state.borrow_mut();
        st.sync_row_scene_cache_epoch();
        (0..3)
            .map(|row| {
                let (range, text, _folds, _preedit, _spans) =
                    paint::cached_row_text_with_range(&mut st, row, 64);
                seed_plain_row_scene_cache_entry(&mut st, row, range, text, fg, row as u64 + 1)
            })
            .collect::<Vec<_>>()
    };

    let row_scene_resets_before = handle.cache_stats().row_scene_resets;
    {
        let mut st = handle.state.borrow_mut();
        crate::editor::input::apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            Edit::Insert {
                at: 0,
                text: "!".to_string(),
            },
            Selection {
                anchor: 1,
                focus: 1,
            },
        )
        .expect("edit should apply");

        assert_eq!(st.row_scene_cache_rev, st.buffer.revision());
        assert_eq!(
            st.cache_stats.row_scene_resets, row_scene_resets_before,
            "safe single-line edits should delta-update row scene cache instead of resetting it"
        );
        assert!(
            !st.row_scene_cache.contains_key(&0),
            "edited row scene must be rebuilt"
        );
        assert_eq!(
            st.row_scene_cache_scene_ops_len_total, 2,
            "only unaffected row scene chunks should remain resident"
        );
        #[cfg(feature = "syntax")]
        assert!(
            st.row_scene_replay_plan_cache.is_none(),
            "window-level replay plans include revision/display epochs and must be rebuilt"
        );

        for seeded in before.into_iter().skip(1) {
            let (entry, tick) = st
                .row_scene_cache
                .get(&seeded.row)
                .expect("unaffected row scene should remain cached");
            assert_eq!(*tick, seeded.tick);
            assert_eq!(
                entry.key, seeded.key,
                "unaffected row scene key should stay stable"
            );
            assert!(
                Arc::ptr_eq(&entry.retained.content.text, &seeded.text),
                "unaffected row scene text allocation should stay stable"
            );
            assert_eq!(
                entry.retained.content.range,
                (seeded.range.start + 1)..(seeded.range.end + 1),
                "unaffected row scene content range should shift by inserted bytes"
            );
            assert_eq!(
                entry.retained.geom.row_range,
                (seeded.range.start + 1)..(seeded.range.end + 1),
                "unaffected row scene geometry range should shift by inserted bytes"
            );
            assert_eq!(
                entry.retained.chunk.fingerprint(),
                seeded.chunk_fingerprint,
                "unaffected row scene chunk identity should stay stable"
            );
            assert_eq!(
                entry.retained.hosted_resources.text_blob_ids(),
                seeded.hosted_text_blobs.as_slice(),
                "unaffected row scene hosted resources should stay stable"
            );
            #[cfg(feature = "syntax")]
            assert!(entry.syntax_replay_key.is_none());
        }
    }
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

fn test_retained_row_scene_fragment(row: usize) -> Arc<RowSceneRetainedFragment> {
    let text = Arc::<str>::from("x");
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: Px(14.0),
        ..Default::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(256.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let row_geom_key = geom::RowGeomKey::for_plain(
        &text,
        &text_style,
        (
            constraints.max_width,
            constraints.wrap,
            constraints.overflow,
            fret_core::TextAlign::Start,
            1.0,
        ),
        fret_runtime::TextFontStackKey::default(),
    );
    let local_bounds = Rect::new(
        Point::new(Px(0.0), Px(row as f32 * 16.0)),
        Size::new(Px(80.0), Px(16.0)),
    );
    Arc::new(RowSceneRetainedFragment {
        content: Arc::new(RowContentSnapshot {
            text,
            range: row..row.saturating_add(1),
            fold_map: None,
            preedit_range: None,
            row_spans: Arc::from([]),
        }),
        local_bounds,
        origin: local_bounds.origin,
        geom: RowGeom {
            row_range: row..row.saturating_add(1),
            key: row_geom_key,
            caret_stops: Vec::new(),
            fold_map: None,
            caret_rect_top: None,
            caret_rect_height: None,
            has_preedit: false,
            preedit: None,
        },
        is_rich: false,
        chunk: fret_core::SceneChunk::default(),
        hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
    })
}

fn seed_plain_row_scene_cache_entry(
    st: &mut CodeEditorState,
    row: usize,
    range: std::ops::Range<usize>,
    text: Arc<str>,
    fg: Color,
    tick: u64,
) -> SeededPlainRowScene {
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: Px(14.0),
        ..Default::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(256.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let row_geom_key = geom::RowGeomKey::for_plain(
        &text,
        &text_style,
        (
            constraints.max_width,
            constraints.wrap,
            constraints.overflow,
            fret_core::TextAlign::Start,
            1.0,
        ),
        st.font_stack_key,
    );
    let key = RowSceneKey::plain(row_geom_key.clone(), fg);
    let rect = Rect::new(
        Point::new(Px(0.0), Px(row as f32 * 16.0)),
        Size::new(Px(80.0), Px(16.0)),
    );
    let text_blob = test_text_blob_id(row as u64 + 1);
    let chunk = fret_core::SceneChunk::from_ops(Arc::from(vec![SceneOp::Text {
        order: DrawOrder(2),
        origin: rect.origin,
        text: text_blob,
        paint: fret_core::Paint::Solid(fg).into(),
        outline: None,
        shadow: None,
    }]));
    let hosted_resources = fret_ui::canvas::CanvasHostedResources::from_scene_ops(chunk.ops());
    let content = Arc::new(RowContentSnapshot {
        text: Arc::clone(&text),
        range: range.clone(),
        fold_map: None,
        preedit_range: None,
        row_spans: Arc::from([]),
    });
    let retained = Arc::new(RowSceneRetainedFragment {
        content,
        local_bounds: rect,
        origin: rect.origin,
        geom: RowGeom {
            row_range: range.clone(),
            key: row_geom_key,
            caret_stops: Vec::new(),
            fold_map: None,
            caret_rect_top: None,
            caret_rect_height: None,
            has_preedit: false,
            preedit: None,
        },
        is_rich: false,
        chunk,
        hosted_resources,
    });
    let chunk_fingerprint = retained.chunk.fingerprint();
    let hosted_text_blobs = retained.hosted_resources.text_blob_ids().to_vec();
    st.row_scene_cache.insert(
        row,
        (
            RowSceneCacheEntry {
                key: key.clone(),
                retained,
                #[cfg(feature = "syntax")]
                syntax_replay_key: None,
            },
            tick,
        ),
    );
    st.row_scene_cache_queue.push_back((row, tick));
    st.row_scene_cache_tick = st.row_scene_cache_tick.max(tick);
    st.row_scene_cache_scene_ops_len_total =
        st.row_scene_cache_scene_ops_len_total.saturating_add(1);

    SeededPlainRowScene {
        row,
        range,
        text,
        key,
        tick,
        chunk_fingerprint,
        hosted_text_blobs,
    }
}

#[test]
fn retained_row_scene_origin_preserves_bounds_offset() {
    let retained = RowSceneRetainedFragment {
        content: Arc::new(RowContentSnapshot {
            text: Arc::<str>::from("x"),
            range: 0..1,
            fold_map: None,
            preedit_range: None,
            row_spans: Arc::from([]),
        }),
        local_bounds: Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(80.0), Px(16.0)),
        ),
        origin: Point::new(Px(14.0), Px(31.0)),
        geom: RowGeom {
            row_range: 0..1,
            key: geom::RowGeomKey::for_plain(
                &Arc::<str>::from("x"),
                &TextStyle {
                    font: FontId::monospace(),
                    size: Px(14.0),
                    ..Default::default()
                },
                (
                    Some(Px(80.0)),
                    TextWrap::None,
                    TextOverflow::Clip,
                    fret_core::TextAlign::Start,
                    1.0,
                ),
                fret_runtime::TextFontStackKey::default(),
            ),
            caret_stops: Vec::new(),
            fold_map: None,
            caret_rect_top: None,
            caret_rect_height: None,
            has_preedit: false,
            preedit: None,
        },
        is_rich: false,
        chunk: fret_core::SceneChunk::default(),
        hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
    };

    let next = retained.origin_for_local_bounds(Rect::new(
        Point::new(Px(30.0), Px(100.0)),
        Size::new(Px(80.0), Px(16.0)),
    ));

    assert_eq!(next, Point::new(Px(34.0), Px(111.0)));
}

#[test]
fn row_scene_replay_plan_rejects_stale_frame_and_skipped_rows() {
    let entry0 = RowSceneReplayPlanEntry {
        row: 0,
        retained: test_retained_row_scene_fragment(0),
        local_bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(16.0))),
    };
    let entry2 = RowSceneReplayPlanEntry {
        row: 2,
        retained: test_retained_row_scene_fragment(2),
        local_bounds: Rect::new(Point::new(Px(0.0), Px(32.0)), Size::new(Px(80.0), Px(16.0))),
    };

    let mut stale = RowSceneReplayPlan {
        frame_seq: 7,
        entries: std::collections::VecDeque::from([entry0.clone()]),
        hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
        hosted_resources_touched: false,
    };
    let (entry, rejected, reason) = paint::take_row_scene_replay_plan_entry(Some(&mut stale), 8, 0);
    assert!(entry.is_none());
    assert_eq!(rejected, 1);
    assert_eq!(reason, Some("frame_seq_mismatch"));
    assert!(stale.entries.is_empty());

    let mut advanced = RowSceneReplayPlan {
        frame_seq: 9,
        entries: std::collections::VecDeque::from([entry0, entry2]),
        hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
        hosted_resources_touched: false,
    };
    let (entry, rejected, reason) =
        paint::take_row_scene_replay_plan_entry(Some(&mut advanced), 9, 1);
    assert!(entry.is_none());
    assert_eq!(rejected, 1);
    assert_eq!(reason, Some("row_advanced_past_entry"));

    let (entry, rejected, reason) =
        paint::take_row_scene_replay_plan_entry(Some(&mut advanced), 9, 2);
    assert_eq!(entry.as_ref().map(|entry| entry.row), Some(2));
    assert_eq!(rejected, 0);
    assert_eq!(reason, None);
}

#[test]
fn row_scene_replay_plan_reports_scene_chunk_debug_metadata() {
    use fret_ui::tree::BoundarySceneFragmentDebug;

    let empty = RowSceneReplayPlan {
        frame_seq: 9,
        entries: std::collections::VecDeque::from([RowSceneReplayPlanEntry {
            row: 0,
            retained: test_retained_row_scene_fragment(0),
            local_bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(16.0))),
        }]),
        hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
        hosted_resources_touched: false,
    };
    assert_eq!(empty.boundary_scene_fragment_entry_count(), 1);
    assert_eq!(empty.boundary_scene_fragment_chunk_count(), 0);
    assert_eq!(empty.boundary_scene_fragment_fingerprint(), 0);
    let mut empty_manifest = fret_ui::tree::BoundarySceneChunkManifest::default();
    empty.append_boundary_scene_fragment_chunks(&mut empty_manifest);
    assert!(empty_manifest.is_empty());

    let retained_bounds = Rect::new(Point::new(Px(0.0), Px(16.0)), Size::new(Px(80.0), Px(16.0)));
    let replay_bounds = Rect::new(Point::new(Px(4.0), Px(32.0)), Size::new(Px(80.0), Px(16.0)));
    let fg = Color {
        r: 0.2,
        g: 0.8,
        b: 0.3,
        a: 1.0,
    };
    let chunk = fret_core::SceneChunk::from_ops(Arc::from(vec![SceneOp::Quad {
        order: DrawOrder(2),
        rect: retained_bounds,
        background: fret_core::Paint::Solid(fg).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    }]));
    let chunk_fingerprint = chunk.fingerprint();
    let mut retained = (*test_retained_row_scene_fragment(1)).clone();
    retained.chunk = chunk;
    retained.local_bounds = retained_bounds;
    retained.origin = Point::new(Px(10.0), Px(20.0));
    let plan = RowSceneReplayPlan {
        frame_seq: 9,
        entries: std::collections::VecDeque::from([RowSceneReplayPlanEntry {
            row: 1,
            retained: Arc::new(retained),
            local_bounds: replay_bounds,
        }]),
        hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
        hosted_resources_touched: false,
    };

    assert_eq!(plan.boundary_scene_fragment_entry_count(), 1);
    assert_eq!(plan.boundary_scene_fragment_chunk_count(), 1);
    assert_ne!(plan.boundary_scene_fragment_fingerprint(), 0);
    assert_ne!(
        plan.boundary_scene_fragment_fingerprint(),
        chunk_fingerprint,
        "row-scene diagnostics include row identity, not only chunk bytes"
    );
    let mut manifest = fret_ui::tree::BoundarySceneChunkManifest::default();
    plan.append_boundary_scene_fragment_chunks(&mut manifest);
    assert_eq!(manifest.len(), 1);
    assert_ne!(manifest.chunks()[0].fingerprint(), chunk_fingerprint);
    assert_eq!(manifest.chunks()[0].local_bounds(), replay_bounds);
    assert_eq!(
        manifest.chunks()[0].scene_origin(),
        Point::new(Px(14.0), Px(36.0))
    );
    assert_eq!(
        manifest.chunks()[0].fingerprint(),
        fret_core::SceneChunkManifestEntry::new(
            manifest.chunks()[0].chunk().clone(),
            replay_bounds,
            Point::new(Px(14.0), Px(36.0)),
        )
        .fingerprint()
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn prepaint_row_scene_replay_plan_aggregates_hosted_resources_once() {
    let handle = CodeEditorHandle::new("row0\nrow1\n");
    let mut st = handle.state.borrow_mut();
    st.paint_perf_enabled = true;

    let fg = Color {
        r: 0.2,
        g: 0.8,
        b: 0.3,
        a: 1.0,
    };
    let text_style = TextStyle {
        size: Px(14.0),
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(4096.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let frame = WindowedRowsPaintFrame {
        viewport_height: Px(32.0),
        offset_y: Px(0.0),
        row_height: Px(16.0),
        row_stride: Px(16.0),
        gap: Px(0.0),
        scroll_margin: Px(0.0),
        visible_start: 0,
        visible_end: 1,
    };
    let content_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(32.0)));

    for row in 0usize..=1 {
        let text = Arc::<str>::from(format!("row{row}"));
        let range = row..row.saturating_add(1);
        let content = Arc::new(RowContentSnapshot {
            text: Arc::clone(&text),
            range: range.clone(),
            fold_map: None,
            preedit_range: None,
            row_spans: Arc::from([]),
        });
        let row_geom_key = geom::RowGeomKey::for_plain(
            &text,
            &text_style,
            (
                constraints.max_width,
                constraints.wrap,
                constraints.overflow,
                fret_core::TextAlign::Start,
                1.0,
            ),
            st.font_stack_key,
        );
        let rect = frame
            .row_rect(content_bounds, row)
            .expect("test row should be visible");
        let text_blob = test_text_blob_id((row + 1) as u64);
        let chunk = fret_core::SceneChunk::from_ops(Arc::from(vec![SceneOp::Text {
            order: DrawOrder(2),
            origin: rect.origin,
            text: text_blob,
            paint: fret_core::Paint::Solid(fg).into(),
            outline: None,
            shadow: None,
        }]));
        let hosted_resources = fret_ui::canvas::CanvasHostedResources::from_scene_ops(chunk.ops());
        st.row_scene_cache.insert(
            row,
            (
                RowSceneCacheEntry {
                    key: RowSceneKey::plain(row_geom_key.clone(), fg),
                    retained: Arc::new(RowSceneRetainedFragment {
                        content,
                        local_bounds: rect,
                        origin: rect.origin,
                        geom: RowGeom {
                            row_range: range,
                            key: row_geom_key,
                            caret_stops: Vec::new(),
                            fold_map: None,
                            caret_rect_top: None,
                            caret_rect_height: None,
                            has_preedit: false,
                            preedit: None,
                        },
                        is_rich: false,
                        chunk,
                        hosted_resources,
                    }),
                    syntax_replay_key: None,
                },
                row as u64 + 1,
            ),
        );
    }

    st.begin_paint_frame(frame);
    let mut plan = paint::prepaint_row_scene_replay_plan_for_frame(
        &mut st,
        frame,
        content_bounds,
        Px(8.0),
        64,
        &text_style,
        fg,
        0,
        1.0,
    );

    assert_eq!(plan.entries.len(), 2);
    assert_eq!(
        plan.hosted_resources.text_blob_ids(),
        &[test_text_blob_id(1), test_text_blob_id(2)],
        "planned replay should aggregate retained row text blobs once per plan"
    );

    let (_entry0, _rejected0, _reason0) =
        paint::take_row_scene_replay_plan_entry(Some(&mut plan), st.paint_perf_frame.frame_seq, 0);
    let first_resources = paint::take_row_scene_replay_plan_hosted_resources_once(Some(&mut plan));
    assert_eq!(
        first_resources
            .as_ref()
            .map(|resources| resources.text_blob_ids()),
        Some(&[test_text_blob_id(1), test_text_blob_id(2)][..]),
        "the first actual planned replay should carry the aggregate hosted resources"
    );

    let (_entry1, _rejected1, _reason1) =
        paint::take_row_scene_replay_plan_entry(Some(&mut plan), st.paint_perf_frame.frame_seq, 1);
    let second_resources = paint::take_row_scene_replay_plan_hosted_resources_once(Some(&mut plan));
    assert!(
        second_resources.is_none(),
        "later plan entries should not touch plan resources again"
    );
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
    let offscreen_caret = text.len();
    let handle = CodeEditorHandle::new(text);
    handle.set_language(Some(Arc::<str>::from("rust")));
    handle.set_caret(offscreen_caret);
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
                    .saturating_sub(old.retained.chunk.ops_len() as u64);
            }
        }
        st.baseline_measure_cache = None;
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
    assert!(
        perf.ns_row_scene_replay_setup > 0,
        "planned replay rows should record replay setup attribution"
    );
    assert_eq!(
        perf.us_baseline_measure, 0,
        "no-overlay planned replay rows should not remeasure the text baseline even if the cache is cold"
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
fn prepaint_row_scene_replay_plan_reuses_stable_window_plan() {
    let handle = CodeEditorHandle::new("row0\nrow1\n");
    handle.state.borrow_mut().paint_perf_enabled = true;

    let fg = Color {
        r: 0.2,
        g: 0.3,
        b: 0.4,
        a: 1.0,
    };
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: Px(14.0),
        ..Default::default()
    };
    let content_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(640.0), Px(32.0)));
    let frame = WindowedRowsPaintFrame {
        viewport_height: Px(32.0),
        offset_y: Px(0.0),
        row_height: Px(16.0),
        row_stride: Px(16.0),
        gap: Px(0.0),
        scroll_margin: Px(0.0),
        visible_start: 0,
        visible_end: 1,
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(4096.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };

    let mut st = handle.state.borrow_mut();
    st.sync_row_scene_cache_epoch();
    for (row, text, range) in [
        (0usize, Arc::<str>::from("row0"), 0..4),
        (1usize, Arc::<str>::from("row1"), 5..9),
    ] {
        let rect = frame
            .row_rect(content_bounds, row)
            .expect("seed row should be visible");
        let content = Arc::new(RowContentSnapshot {
            text: Arc::clone(&text),
            range: range.clone(),
            fold_map: None,
            preedit_range: None,
            row_spans: Arc::from([]),
        });
        let row_geom_key = geom::RowGeomKey::for_plain(
            &text,
            &text_style,
            (
                constraints.max_width,
                constraints.wrap,
                constraints.overflow,
                fret_core::TextAlign::Start,
                1.0,
            ),
            st.font_stack_key,
        );
        let row_scene_key = RowSceneKey::plain(row_geom_key.clone(), fg);
        let chunk = fret_core::SceneChunk::from_ops(Arc::from(vec![SceneOp::Quad {
            order: DrawOrder(2),
            rect,
            background: fret_core::Paint::Solid(fg).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        }]));
        st.row_scene_cache.insert(
            row,
            (
                RowSceneCacheEntry {
                    key: row_scene_key,
                    retained: Arc::new(RowSceneRetainedFragment {
                        content,
                        local_bounds: rect,
                        origin: rect.origin,
                        geom: RowGeom {
                            row_range: range,
                            key: row_geom_key,
                            caret_stops: Vec::new(),
                            fold_map: None,
                            caret_rect_top: None,
                            caret_rect_height: None,
                            has_preedit: false,
                            preedit: None,
                        },
                        is_rich: false,
                        hosted_resources: fret_ui::canvas::CanvasHostedResources::from_scene_ops(
                            chunk.ops(),
                        ),
                        chunk,
                    }),
                    syntax_replay_key: None,
                },
                row as u64 + 1,
            ),
        );
    }
    st.row_scene_cache_tick = 2;
    st.row_scene_cache_scene_ops_len_total = 2;

    st.begin_paint_frame(frame);
    let validation_plan = paint::prepaint_row_scene_replay_plan_for_frame(
        &mut st,
        frame,
        content_bounds,
        Px(8.0),
        64,
        &text_style,
        fg,
        0,
        1.0,
    );
    assert_eq!(validation_plan.entries.len(), 2);
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_candidates, 2);
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_planned, 2);
    assert_eq!(
        st.row_scene_replay_plan_cache
            .as_ref()
            .map(|cache| cache.entries.len()),
        Some(2),
        "the validation frame should save every planned row for stable-window reuse"
    );

    let before_reuse = st.cache_stats;
    st.begin_paint_frame(frame);
    let reuse_plan = paint::prepaint_row_scene_replay_plan_for_frame(
        &mut st,
        frame,
        content_bounds,
        Px(8.0),
        64,
        &text_style,
        fg,
        0,
        1.0,
    );
    let after_reuse = st.cache_stats;
    let reuse_perf = st.paint_perf_frame;

    assert_eq!(
        reuse_plan.entries.len(),
        validation_plan.entries.len(),
        "stable-window plan reuse should preserve the planned row count"
    );
    assert_eq!(
        reuse_perf.rows_scene_prepaint_candidates, 0,
        "stable-window plan reuse should skip per-row candidate probing"
    );
    assert_eq!(
        reuse_perf.us_row_scene_prepaint_probe, 0,
        "stable-window plan reuse should skip per-row cache probes"
    );
    assert_eq!(
        reuse_perf.us_row_scene_prepaint_key_compare, 0,
        "stable-window plan reuse should skip per-row key comparisons"
    );
    assert_eq!(
        after_reuse
            .row_scene_fast_get_calls
            .saturating_sub(before_reuse.row_scene_fast_get_calls),
        0,
        "stable-window plan reuse should not record synthetic row scene cache probes"
    );
    assert_eq!(
        after_reuse
            .row_text_get_calls
            .saturating_sub(before_reuse.row_text_get_calls),
        0,
        "stable-window plan reuse should keep row content out of paint"
    );

    let shifted_frame = WindowedRowsPaintFrame {
        viewport_height: Px(32.0),
        offset_y: Px(16.0),
        row_height: Px(16.0),
        row_stride: Px(16.0),
        gap: Px(0.0),
        scroll_margin: Px(0.0),
        visible_start: 1,
        visible_end: 1,
    };
    let before_shifted_reuse = st.cache_stats;
    st.begin_paint_frame(shifted_frame);
    let shifted_plan = paint::prepaint_row_scene_replay_plan_for_frame(
        &mut st,
        shifted_frame,
        content_bounds,
        Px(8.0),
        64,
        &text_style,
        fg,
        0,
        1.0,
    );
    let after_shifted_reuse = st.cache_stats;
    let shifted_perf = st.paint_perf_frame;

    assert_eq!(
        shifted_plan.entries.len(),
        1,
        "a sliding visible window should reuse the overlapping retained row"
    );
    assert_eq!(
        shifted_plan.entries.front().map(|entry| entry.row),
        Some(1),
        "overlap reuse should keep the row identity intact"
    );
    assert_eq!(
        shifted_perf.rows_scene_prepaint_plan_cache_hits, 1,
        "overlap reuse should be visible in paint diagnostics"
    );
    assert_eq!(
        shifted_perf.rows_scene_prepaint_candidates, 0,
        "overlap reuse should skip per-row probing for retained rows"
    );
    assert_eq!(
        shifted_perf.us_row_scene_prepaint_probe, 0,
        "overlap reuse should skip cache probe timing"
    );
    assert_eq!(
        shifted_perf.us_row_scene_prepaint_key_compare, 0,
        "overlap reuse should skip key comparison timing"
    );
    assert_eq!(
        after_shifted_reuse
            .row_scene_fast_get_calls
            .saturating_sub(before_shifted_reuse.row_scene_fast_get_calls),
        0,
        "overlap reuse should not count as a synthetic row-scene probe"
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
fn prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit() {
    let handle = CodeEditorHandle::new("row0\nrow1\n");
    handle.state.borrow_mut().paint_perf_enabled = true;

    let fg = Color {
        r: 0.2,
        g: 0.3,
        b: 0.4,
        a: 1.0,
    };
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: Px(14.0),
        ..Default::default()
    };
    let content_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(640.0), Px(32.0)));
    let frame = WindowedRowsPaintFrame {
        viewport_height: Px(32.0),
        offset_y: Px(0.0),
        row_height: Px(16.0),
        row_stride: Px(16.0),
        gap: Px(0.0),
        scroll_margin: Px(0.0),
        visible_start: 0,
        visible_end: 1,
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(4096.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };

    let mut st = handle.state.borrow_mut();
    st.preedit = Some(PreeditState {
        text: "xy".to_string(),
        cursor: Some((1, 1)),
    });
    st.sync_row_scene_cache_epoch();
    for (row, text, range) in [
        (0usize, Arc::<str>::from("row0"), 0..4),
        (1usize, Arc::<str>::from("row1"), 5..9),
    ] {
        let rect = frame
            .row_rect(content_bounds, row)
            .expect("seed row should be visible");
        let content = Arc::new(RowContentSnapshot {
            text: Arc::clone(&text),
            range: range.clone(),
            fold_map: None,
            preedit_range: None,
            row_spans: Arc::from([]),
        });
        let row_geom_key = geom::RowGeomKey::for_plain(
            &text,
            &text_style,
            (
                constraints.max_width,
                constraints.wrap,
                constraints.overflow,
                fret_core::TextAlign::Start,
                1.0,
            ),
            st.font_stack_key,
        );
        let row_scene_key = RowSceneKey::plain(row_geom_key.clone(), fg);
        let chunk = fret_core::SceneChunk::from_ops(Arc::from(vec![SceneOp::Quad {
            order: DrawOrder(2),
            rect,
            background: fret_core::Paint::Solid(fg).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        }]));
        st.row_scene_cache.insert(
            row,
            (
                RowSceneCacheEntry {
                    key: row_scene_key,
                    retained: Arc::new(RowSceneRetainedFragment {
                        content,
                        local_bounds: rect,
                        origin: rect.origin,
                        geom: RowGeom {
                            row_range: range,
                            key: row_geom_key,
                            caret_stops: Vec::new(),
                            fold_map: None,
                            caret_rect_top: None,
                            caret_rect_height: None,
                            has_preedit: false,
                            preedit: None,
                        },
                        is_rich: false,
                        hosted_resources: fret_ui::canvas::CanvasHostedResources::from_scene_ops(
                            chunk.ops(),
                        ),
                        chunk,
                    }),
                    syntax_replay_key: None,
                },
                row as u64 + 1,
            ),
        );
    }
    st.row_scene_cache_tick = 2;
    st.row_scene_cache_scene_ops_len_total = 2;

    st.begin_paint_frame(frame);
    st.paint_frame_overlay.caret = Some(PaintFrameCaretOverlay {
        byte: 0,
        row: 0,
        col: 0,
    });
    let warm_plan = paint::prepaint_row_scene_replay_plan_for_frame(
        &mut st,
        frame,
        content_bounds,
        Px(8.0),
        64,
        &text_style,
        fg,
        0,
        1.0,
    );
    assert_eq!(
        warm_plan
            .entries
            .iter()
            .map(|entry| entry.row)
            .collect::<Vec<_>>(),
        vec![1],
        "the first active-preedit frame should plan only unrelated rows"
    );
    assert_eq!(
        st.paint_perf_frame.rows_scene_prepaint_skip_preedit, 1,
        "the first active-preedit frame should skip the actual preedit row"
    );
    assert_eq!(
        st.row_scene_replay_plan_cache
            .as_ref()
            .map(|cache| cache.entries.len()),
        Some(1),
        "the first active-preedit frame should save a partial non-preedit replay plan"
    );

    let before_reuse = st.cache_stats;
    st.begin_paint_frame(frame);
    st.paint_frame_overlay.caret = Some(PaintFrameCaretOverlay {
        byte: 0,
        row: 0,
        col: 0,
    });
    let reuse_plan = paint::prepaint_row_scene_replay_plan_for_frame(
        &mut st,
        frame,
        content_bounds,
        Px(8.0),
        64,
        &text_style,
        fg,
        0,
        1.0,
    );
    let after_reuse = st.cache_stats;
    let perf = st.paint_perf_frame;

    assert!(
        perf.rows_scene_prepaint_plan_cache_hits > 0,
        "active preedit should still reuse cached replay-plan entries for unrelated rows"
    );
    assert_eq!(
        reuse_plan
            .entries
            .iter()
            .map(|entry| entry.row)
            .collect::<Vec<_>>(),
        vec![1],
        "active preedit should only replay-plan non-preedit rows"
    );
    assert!(
        perf.rows_scene_prepaint_skip_preedit > 0,
        "the actual preedit row must stay on the paint-time path"
    );
    assert_eq!(
        perf.rows_scene_prepaint_candidates, perf.rows_scene_prepaint_skip_preedit,
        "cached non-preedit rows should avoid per-row candidate probing while preedit is active"
    );
    assert_eq!(
        perf.us_row_scene_prepaint_probe, 0,
        "cached non-preedit rows should avoid cache probe timing while preedit is active"
    );
    assert_eq!(
        perf.us_row_scene_prepaint_key_compare, 0,
        "cached non-preedit rows should avoid key comparison timing while preedit is active"
    );
    assert_eq!(
        after_reuse
            .row_scene_fast_get_calls
            .saturating_sub(before_reuse.row_scene_fast_get_calls),
        0,
        "plan-cache reuse should not record synthetic row-scene probes"
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
    let chunk = fret_core::SceneChunk::from_ops(Arc::from(vec![SceneOp::Quad {
        order: DrawOrder(2),
        rect: content_bounds,
        background: fret_core::Paint::Solid(fg).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    }]));
    let hosted_resources = fret_ui::canvas::CanvasHostedResources::from_scene_ops(chunk.ops());
    st.row_scene_cache.insert(
        0,
        (
            RowSceneCacheEntry {
                key: row_scene_key,
                retained: Arc::new(RowSceneRetainedFragment {
                    content,
                    local_bounds: content_bounds,
                    origin: content_bounds.origin,
                    geom,
                    is_rich: true,
                    chunk,
                    hosted_resources,
                }),
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
    let retained = Arc::clone(
        &st.row_scene_cache
            .get(&0)
            .expect("seeded row scene entry")
            .0
            .retained,
    );
    let planned = plan.entries.front().expect("planned retained row entry");
    assert!(
        Arc::ptr_eq(&planned.retained, &retained),
        "prepaint plan should point at the retained row fragment instead of cloning it"
    );
    assert_eq!(planned.local_bounds, content_bounds);
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
fn prepaint_row_scene_replay_plan_rejects_plain_rows_when_fg_changes() {
    let handle = CodeEditorHandle::new("plain\n");
    handle.state.borrow_mut().paint_perf_enabled = true;

    let mut st = handle.state.borrow_mut();
    ensure_syntax_row_cache_fresh(&mut st);
    st.sync_row_scene_cache_epoch();

    let old_fg = Color {
        r: 0.2,
        g: 0.3,
        b: 0.4,
        a: 1.0,
    };
    let new_fg = Color {
        r: 0.8,
        g: 0.3,
        b: 0.4,
        a: 1.0,
    };
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: Px(14.0),
        ..Default::default()
    };
    let content_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(640.0), Px(16.0)));
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(4096.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let line = Arc::<str>::from("plain");
    let row_range = 0..line.len();
    let content = Arc::new(RowContentSnapshot {
        text: Arc::clone(&line),
        range: row_range.clone(),
        fold_map: None,
        preedit_range: None,
        row_spans: Arc::from([]),
    });
    let row_geom_key = geom::RowGeomKey::for_plain(
        &line,
        &text_style,
        (
            constraints.max_width,
            constraints.wrap,
            constraints.overflow,
            fret_core::TextAlign::Start,
            1.0,
        ),
        st.font_stack_key,
    );
    let row_scene_key = RowSceneKey::plain(row_geom_key.clone(), old_fg);
    st.row_scene_cache.insert(
        0,
        (
            RowSceneCacheEntry {
                key: row_scene_key,
                retained: Arc::new(RowSceneRetainedFragment {
                    content,
                    local_bounds: content_bounds,
                    origin: content_bounds.origin,
                    geom: RowGeom {
                        row_range,
                        key: row_geom_key,
                        caret_stops: Vec::new(),
                        fold_map: None,
                        caret_rect_top: None,
                        caret_rect_height: None,
                        has_preedit: false,
                        preedit: None,
                    },
                    is_rich: false,
                    chunk: fret_core::SceneChunk::default(),
                    hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
                }),
                syntax_replay_key: None,
            },
            1,
        ),
    );
    st.row_scene_cache_tick = 1;

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
        Px(8.0),
        64,
        &text_style,
        new_fg,
        0,
        1.0,
    );

    assert!(plan.entries.is_empty());
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_candidates, 1);
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_planned, 0);
    assert_eq!(st.paint_perf_frame.rows_scene_prepaint_skip_key_mismatch, 1);
    assert_eq!(st.cache_stats.row_scene_fast_misses, 1);
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
