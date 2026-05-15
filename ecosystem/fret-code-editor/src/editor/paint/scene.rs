#[cfg(feature = "syntax")]
use super::syntax::SyntaxSpan;
#[cfg(feature = "syntax")]
use crate::editor::syntax::ensure_syntax_row_cache_fresh;

use super::*;

pub(super) fn ensure_row_scene_cache_fresh(st: &mut CodeEditorState) {
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    let folds_epoch = st.folds_epoch;
    let inlays_epoch = st.inlays_epoch;
    let display_map_epoch = st.display_map_epoch;
    let feature_payload_epoch = st.feature_payloads.epoch();
    if st.row_scene_cache_rev != rev
        || st.row_scene_cache_wrap_cols != wrap_cols
        || st.row_scene_cache_folds_epoch != folds_epoch
        || st.row_scene_cache_inlays_epoch != inlays_epoch
        || st.row_scene_cache_display_map_epoch != display_map_epoch
        || st.row_scene_cache_feature_payload_epoch != feature_payload_epoch
    {
        st.invalidate_row_scene_cache();
    }
}

pub(super) fn row_scene_replay_delta(cached_origin: Point, origin: Point) -> Point {
    Point::new(
        Px(origin.x.0 - cached_origin.x.0),
        Px(origin.y.0 - cached_origin.y.0),
    )
}

#[cfg(feature = "syntax")]
pub(super) fn row_scene_cached_entry_matches_syntax(
    cached: &RowSceneCacheEntry,
    row_range: &Range<usize>,
    line: &Arc<str>,
    row_spans: &Arc<[fret_code_editor_view::DisplayRowSpan]>,
    syntax_spans: &Arc<[SyntaxSpan]>,
    text_style: &TextStyle,
    constraints: CanvasTextConstraints,
    font_stack_key: fret_runtime::TextFontStackKey,
    scale_factor: f32,
    theme_revision: u64,
    code_font_feature_policy_rev: u64,
    fg: Color,
) -> bool {
    cached.syntax_replay_key.as_ref().is_some_and(|key| {
        key.matches_current(
            row_range,
            line,
            row_spans,
            syntax_spans,
            text_style,
            constraints,
            font_stack_key,
            scale_factor,
            theme_revision,
            code_font_feature_policy_rev,
            fg,
        )
    })
}

#[cfg(not(feature = "syntax"))]
pub(super) fn replay_row_scene_plan_candidates_for_frame(
    st: &mut CodeEditorState,
    frame: WindowedRowsPaintFrame,
    content_bounds: Rect,
    _max_entries: usize,
    _text_style: &TextStyle,
    _fg: Color,
    _theme_revision: u64,
    _constraints: CanvasTextConstraints,
    _scale_factor: f32,
) -> RowSceneReplayPlan {
    let _ = content_bounds;
    let _ = frame;
    RowSceneReplayPlan {
        frame_seq: st.paint_perf_frame.frame_seq,
        entries: VecDeque::new(),
    }
}

#[cfg(feature = "syntax")]
#[allow(clippy::too_many_arguments)]
pub(super) fn replay_row_scene_plan_candidates_for_frame(
    st: &mut CodeEditorState,
    frame: WindowedRowsPaintFrame,
    content_bounds: Rect,
    max_entries: usize,
    text_style: &TextStyle,
    fg: Color,
    theme_revision: u64,
    constraints: CanvasTextConstraints,
    scale_factor: f32,
) -> RowSceneReplayPlan {
    let frame_seq = st.paint_perf_frame.frame_seq;
    let mut plan = RowSceneReplayPlan {
        frame_seq,
        entries: VecDeque::new(),
    };

    if st.preedit.is_some() {
        return plan;
    }

    ensure_row_scene_cache_fresh(st);
    ensure_syntax_row_cache_fresh(st);

    let started = st.paint_perf_enabled.then(Instant::now);
    let max_entries = frame_cache_max_entries(st, max_entries);
    let font_stack_key = st.font_stack_key;
    let code_font_feature_policy_rev = st.code_font_feature_policy_rev;
    let row_count = st.display_map.row_count();
    if row_count == 0 {
        return plan;
    }

    let end = frame.visible_end.min(row_count.saturating_sub(1));
    let mut planned = 0u64;
    for row in frame.visible_start..=end {
        let Some((cached, _)) = st.row_scene_cache.get(&row) else {
            continue;
        };
        if cached.syntax_replay_key.is_none() {
            continue;
        }
        st.cache_stats.row_scene_fast_get_calls =
            st.cache_stats.row_scene_fast_get_calls.saturating_add(1);

        let content = cached.content.clone();
        if content.preedit_range.is_some() {
            continue;
        }

        let line_idx = st.display_map.display_row_line(row);
        let syntax_spans = match lookup_row_syntax_spans(st, line_idx, max_entries) {
            SyntaxRowCacheLookup::Hit(spans) => spans,
            SyntaxRowCacheLookup::Miss { tick } => {
                populate_row_syntax_spans_after_miss(st, line_idx, max_entries, tick)
            }
        };
        if syntax_spans.is_empty() {
            continue;
        }

        let Some((cached, last_used)) = st.row_scene_cache.get_mut(&row) else {
            continue;
        };
        let matches = row_scene_cached_entry_matches_syntax(
            cached,
            &content.range,
            &content.text,
            &content.row_spans,
            &syntax_spans,
            text_style,
            constraints,
            font_stack_key,
            scale_factor,
            theme_revision,
            code_font_feature_policy_rev,
            fg,
        );
        if !matches {
            st.cache_stats.row_scene_fast_misses =
                st.cache_stats.row_scene_fast_misses.saturating_add(1);
            continue;
        }

        let scene_origin = cached.origin;
        let geom = cached.geom.clone();
        let is_rich = cached.is_rich;
        let ops = Arc::clone(&cached.ops);
        let hosted_resources = cached.hosted_resources.clone();
        st.row_scene_cache_tick = st.row_scene_cache_tick.saturating_add(1);
        let tick = st.row_scene_cache_tick;
        *last_used = tick;
        let _ = cached;
        st.row_scene_cache_queue.push_back((row, tick));

        let Some(rect) = frame.row_rect(content_bounds, row) else {
            continue;
        };
        plan.entries
            .push_back(fret_ui::canvas::CanvasSceneFragment::new(
                RowSceneFragmentPayload {
                    row,
                    content,
                    geom,
                    is_rich,
                },
                ops,
                hosted_resources,
                rect,
                scene_origin,
            ));
        planned = planned.saturating_add(1);
        st.cache_stats.row_scene_fast_hits = st.cache_stats.row_scene_fast_hits.saturating_add(1);
        st.cache_stats.row_scene_get_calls = st.cache_stats.row_scene_get_calls.saturating_add(1);
        st.cache_stats.row_scene_hits = st.cache_stats.row_scene_hits.saturating_add(1);
    }

    compact_row_lru_queue_if_needed(
        &st.row_scene_cache,
        &mut st.row_scene_cache_queue,
        max_entries,
    );

    if st.paint_perf_enabled {
        st.paint_perf_frame.rows_scene_prepaint_planned = st
            .paint_perf_frame
            .rows_scene_prepaint_planned
            .saturating_add(planned);
    }
    if let Some(started) = started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_scene_prepaint_plan,
            &mut st.paint_perf_frame.ns_row_scene_prepaint_plan,
            started,
        );
    }

    plan
}

pub(super) fn replay_row_scene_plan_entry(
    painter: &mut fret_ui::canvas::CanvasPainter<'_>,
    st: &mut CodeEditorState,
    entry: &RowSceneReplayPlanEntry,
    origin: Point,
) {
    let replay_delta = row_scene_replay_delta(entry.scene_origin, origin);
    let touch_started = st.paint_perf_enabled.then(Instant::now);
    painter.touch_hosted_resources(&entry.hosted_resources);
    if let Some(started) = touch_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_scene_replay_touch,
            &mut st.paint_perf_frame.ns_row_scene_replay_touch,
            started,
        );
    }
    let replay_started = st.paint_perf_enabled.then(Instant::now);
    painter.scene().replay_ops_translated_with_text_blob_ids(
        entry.ops.as_ref(),
        replay_delta,
        entry.hosted_resources.text_blob_ids(),
    );
    if let Some(started) = replay_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_scene_replay_ops,
            &mut st.paint_perf_frame.ns_row_scene_replay_ops,
            started,
        );
    }
    if st.paint_perf_enabled {
        st.paint_perf_frame.rows_scene_replayed =
            st.paint_perf_frame.rows_scene_replayed.saturating_add(1);
        st.paint_perf_frame.rows_scene_prepaint_plan_used = st
            .paint_perf_frame
            .rows_scene_prepaint_plan_used
            .saturating_add(1);
    }
}

#[cfg(feature = "syntax")]
pub(super) fn refresh_row_scene_syntax_replay_key(
    st: &mut CodeEditorState,
    row: usize,
    key: Option<&RowSceneSyntaxReplayKey>,
) {
    let Some(key) = key else {
        return;
    };
    if let Some((cached, _)) = st.row_scene_cache.get_mut(&row) {
        cached.syntax_replay_key = Some(key.clone());
    }
}

#[cfg(feature = "syntax")]
#[allow(clippy::too_many_arguments)]
pub(super) fn try_replay_row_scene_cache_fast_syntax(
    painter: &mut fret_ui::canvas::CanvasPainter<'_>,
    st: &mut CodeEditorState,
    row: usize,
    row_range: &Range<usize>,
    line: &Arc<str>,
    row_spans: &Arc<[fret_code_editor_view::DisplayRowSpan]>,
    syntax_spans: &Arc<[SyntaxSpan]>,
    text_style: &TextStyle,
    constraints: CanvasTextConstraints,
    font_stack_key: fret_runtime::TextFontStackKey,
    scale_factor: f32,
    theme_revision: u64,
    code_font_feature_policy_rev: u64,
    fg: Color,
    origin: fret_core::Point,
    max_entries: usize,
) -> Option<(RowGeom, bool)> {
    ensure_row_scene_cache_fresh(st);
    st.cache_stats.row_scene_fast_get_calls =
        st.cache_stats.row_scene_fast_get_calls.saturating_add(1);
    st.row_scene_cache_tick = st.row_scene_cache_tick.saturating_add(1);
    let tick = st.row_scene_cache_tick;
    let path_started = st.paint_perf_enabled.then(Instant::now);
    let mut probe_started = st.paint_perf_enabled.then(Instant::now);

    let replayed = {
        match st.row_scene_cache.get_mut(&row) {
            Some((cached, last_used)) => {
                let key_compare_started = st.paint_perf_enabled.then(Instant::now);
                let matches = row_scene_cached_entry_matches_syntax(
                    cached,
                    row_range,
                    line,
                    row_spans,
                    syntax_spans,
                    text_style,
                    constraints,
                    font_stack_key,
                    scale_factor,
                    theme_revision,
                    code_font_feature_policy_rev,
                    fg,
                );
                if let Some(started) = key_compare_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_fast_key_compare,
                        &mut st.paint_perf_frame.ns_row_scene_fast_key_compare,
                        started,
                    );
                }
                if matches {
                    *last_used = tick;
                    if let Some(started) = probe_started.take() {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_fast_probe,
                            &mut st.paint_perf_frame.ns_row_scene_fast_probe,
                            started,
                        );
                    }
                    let replay_delta = row_scene_replay_delta(cached.origin, origin);
                    let touch_started = st.paint_perf_enabled.then(Instant::now);
                    painter.touch_hosted_resources(&cached.hosted_resources);
                    if let Some(started) = touch_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_replay_touch,
                            &mut st.paint_perf_frame.ns_row_scene_replay_touch,
                            started,
                        );
                    }
                    let replay_started = st.paint_perf_enabled.then(Instant::now);
                    painter.scene().replay_ops_translated_with_text_blob_ids(
                        cached.ops.as_ref(),
                        replay_delta,
                        cached.hosted_resources.text_blob_ids(),
                    );
                    if let Some(started) = replay_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_replay_ops,
                            &mut st.paint_perf_frame.ns_row_scene_replay_ops,
                            started,
                        );
                    }
                    if st.paint_perf_enabled {
                        st.paint_perf_frame.rows_scene_replayed =
                            st.paint_perf_frame.rows_scene_replayed.saturating_add(1);
                    }
                    Some((cached.geom.clone(), cached.is_rich))
                } else {
                    if let Some(started) = probe_started.take() {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_fast_probe,
                            &mut st.paint_perf_frame.ns_row_scene_fast_probe,
                            started,
                        );
                    }
                    None
                }
            }
            None => {
                if let Some(started) = probe_started.take() {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_fast_probe,
                        &mut st.paint_perf_frame.ns_row_scene_fast_probe,
                        started,
                    );
                }
                None
            }
        }
    };

    let out = if let Some(out) = replayed {
        st.row_scene_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.row_scene_cache,
            &mut st.row_scene_cache_queue,
            max_entries,
        );
        st.cache_stats.row_scene_fast_hits = st.cache_stats.row_scene_fast_hits.saturating_add(1);
        st.cache_stats.row_scene_get_calls = st.cache_stats.row_scene_get_calls.saturating_add(1);
        st.cache_stats.row_scene_hits = st.cache_stats.row_scene_hits.saturating_add(1);
        Some(out)
    } else {
        st.cache_stats.row_scene_fast_misses =
            st.cache_stats.row_scene_fast_misses.saturating_add(1);
        None
    };
    if let Some(started) = path_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_scene_fast_path,
            &mut st.paint_perf_frame.ns_row_scene_fast_path,
            started,
        );
    }
    out
}

#[allow(clippy::too_many_arguments)]
pub(super) fn try_replay_row_scene_cache(
    painter: &mut fret_ui::canvas::CanvasPainter<'_>,
    st: &mut CodeEditorState,
    row: usize,
    key: &RowSceneKey,
    origin: fret_core::Point,
    max_entries: usize,
) -> Option<(RowGeom, bool)> {
    ensure_row_scene_cache_fresh(st);
    st.cache_stats.row_scene_get_calls = st.cache_stats.row_scene_get_calls.saturating_add(1);
    st.row_scene_cache_tick = st.row_scene_cache_tick.saturating_add(1);
    let tick = st.row_scene_cache_tick;
    let path_started = st.paint_perf_enabled.then(Instant::now);
    let mut probe_started = st.paint_perf_enabled.then(Instant::now);

    let replayed = {
        match st.row_scene_cache.get_mut(&row) {
            Some((cached, last_used)) => {
                let key_compare_started = st.paint_perf_enabled.then(Instant::now);
                let matches = cached.key == *key;
                if let Some(started) = key_compare_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_full_key_compare,
                        &mut st.paint_perf_frame.ns_row_scene_full_key_compare,
                        started,
                    );
                }
                if matches {
                    *last_used = tick;
                    if let Some(started) = probe_started.take() {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_full_probe,
                            &mut st.paint_perf_frame.ns_row_scene_full_probe,
                            started,
                        );
                    }
                    let replay_delta = row_scene_replay_delta(cached.origin, origin);
                    let touch_started = st.paint_perf_enabled.then(Instant::now);
                    painter.touch_hosted_resources(&cached.hosted_resources);
                    if let Some(started) = touch_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_replay_touch,
                            &mut st.paint_perf_frame.ns_row_scene_replay_touch,
                            started,
                        );
                    }
                    let replay_started = st.paint_perf_enabled.then(Instant::now);
                    painter.scene().replay_ops_translated_with_text_blob_ids(
                        cached.ops.as_ref(),
                        replay_delta,
                        cached.hosted_resources.text_blob_ids(),
                    );
                    if let Some(started) = replay_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_replay_ops,
                            &mut st.paint_perf_frame.ns_row_scene_replay_ops,
                            started,
                        );
                    }
                    if st.paint_perf_enabled {
                        st.paint_perf_frame.rows_scene_replayed =
                            st.paint_perf_frame.rows_scene_replayed.saturating_add(1);
                    }
                    Some((cached.geom.clone(), cached.is_rich))
                } else {
                    if let Some(started) = probe_started.take() {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_full_probe,
                            &mut st.paint_perf_frame.ns_row_scene_full_probe,
                            started,
                        );
                    }
                    None
                }
            }
            None => {
                if let Some(started) = probe_started.take() {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_full_probe,
                        &mut st.paint_perf_frame.ns_row_scene_full_probe,
                        started,
                    );
                }
                None
            }
        }
    };

    let out = if let Some(out) = replayed {
        st.row_scene_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.row_scene_cache,
            &mut st.row_scene_cache_queue,
            max_entries,
        );
        st.cache_stats.row_scene_hits = st.cache_stats.row_scene_hits.saturating_add(1);
        Some(out)
    } else {
        st.cache_stats.row_scene_misses = st.cache_stats.row_scene_misses.saturating_add(1);
        None
    };
    if let Some(started) = path_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_scene_full_path,
            &mut st.paint_perf_frame.ns_row_scene_full_path,
            started,
        );
    }
    out
}

#[cfg(feature = "syntax")]
pub(super) fn store_row_scene_cache(
    st: &mut CodeEditorState,
    row: usize,
    key: RowSceneKey,
    content: Arc<RowContentSnapshot>,
    origin: fret_core::Point,
    geom: RowGeom,
    is_rich: bool,
    ops: Vec<SceneOp>,
    syntax_replay_key: Option<RowSceneSyntaxReplayKey>,
    max_entries: usize,
) {
    if max_entries == 0 || ops.is_empty() {
        return;
    }

    ensure_row_scene_cache_fresh(st);
    let store_started = st.paint_perf_enabled.then(Instant::now);
    st.row_scene_cache_tick = st.row_scene_cache_tick.saturating_add(1);
    let tick = st.row_scene_cache_tick;
    let ops_len = ops.len() as u64;
    let ops: Arc<[SceneOp]> = Arc::from(ops);
    let hosted_resources = fret_ui::canvas::CanvasHostedResources::from_scene_ops(ops.as_ref());

    if let Some((old, _)) = st.row_scene_cache.insert(
        row,
        (
            RowSceneCacheEntry {
                key,
                content,
                origin,
                geom,
                is_rich,
                ops,
                hosted_resources,
                syntax_replay_key,
            },
            tick,
        ),
    ) {
        st.row_scene_cache_scene_ops_len_total = st
            .row_scene_cache_scene_ops_len_total
            .saturating_sub(old.ops.len() as u64);
    }
    st.row_scene_cache_scene_ops_len_total = st
        .row_scene_cache_scene_ops_len_total
        .saturating_add(ops_len);
    st.row_scene_cache_queue.push_back((row, tick));
    compact_row_lru_queue_if_needed(
        &st.row_scene_cache,
        &mut st.row_scene_cache_queue,
        max_entries,
    );

    while st.row_scene_cache.len() > max_entries {
        let Some((victim, victim_tick)) = st.row_scene_cache_queue.pop_front() else {
            break;
        };
        let remove = st
            .row_scene_cache
            .get(&victim)
            .is_some_and(|(_, last_used)| *last_used == victim_tick);
        if remove {
            if let Some((old, _)) = st.row_scene_cache.remove(&victim) {
                st.row_scene_cache_scene_ops_len_total = st
                    .row_scene_cache_scene_ops_len_total
                    .saturating_sub(old.ops.len() as u64);
            }
            st.cache_stats.row_scene_evictions =
                st.cache_stats.row_scene_evictions.saturating_add(1);
        }
    }
    if let Some(started) = store_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_scene_store,
            &mut st.paint_perf_frame.ns_row_scene_store,
            started,
        );
    }
    if st.paint_perf_enabled {
        st.paint_perf_frame.rows_scene_stored =
            st.paint_perf_frame.rows_scene_stored.saturating_add(1);
        st.paint_perf_frame.row_scene_ops_stored = st
            .paint_perf_frame
            .row_scene_ops_stored
            .saturating_add(ops_len);
    }
}

#[cfg(not(feature = "syntax"))]
pub(super) fn store_row_scene_cache(
    st: &mut CodeEditorState,
    row: usize,
    key: RowSceneKey,
    content: Arc<RowContentSnapshot>,
    origin: fret_core::Point,
    geom: RowGeom,
    is_rich: bool,
    ops: Vec<SceneOp>,
    max_entries: usize,
) {
    if max_entries == 0 || ops.is_empty() {
        return;
    }

    ensure_row_scene_cache_fresh(st);
    let store_started = st.paint_perf_enabled.then(Instant::now);
    st.row_scene_cache_tick = st.row_scene_cache_tick.saturating_add(1);
    let tick = st.row_scene_cache_tick;
    let ops_len = ops.len() as u64;
    let ops: Arc<[SceneOp]> = Arc::from(ops);
    let hosted_resources = fret_ui::canvas::CanvasHostedResources::from_scene_ops(ops.as_ref());

    if let Some((old, _)) = st.row_scene_cache.insert(
        row,
        (
            RowSceneCacheEntry {
                key,
                content,
                origin,
                geom,
                is_rich,
                ops,
                hosted_resources,
            },
            tick,
        ),
    ) {
        st.row_scene_cache_scene_ops_len_total = st
            .row_scene_cache_scene_ops_len_total
            .saturating_sub(old.ops.len() as u64);
    }
    st.row_scene_cache_scene_ops_len_total = st
        .row_scene_cache_scene_ops_len_total
        .saturating_add(ops_len);
    st.row_scene_cache_queue.push_back((row, tick));
    compact_row_lru_queue_if_needed(
        &st.row_scene_cache,
        &mut st.row_scene_cache_queue,
        max_entries,
    );

    while st.row_scene_cache.len() > max_entries {
        let Some((victim, victim_tick)) = st.row_scene_cache_queue.pop_front() else {
            break;
        };
        let remove = st
            .row_scene_cache
            .get(&victim)
            .is_some_and(|(_, last_used)| *last_used == victim_tick);
        if remove {
            if let Some((old, _)) = st.row_scene_cache.remove(&victim) {
                st.row_scene_cache_scene_ops_len_total = st
                    .row_scene_cache_scene_ops_len_total
                    .saturating_sub(old.ops.len() as u64);
            }
            st.cache_stats.row_scene_evictions =
                st.cache_stats.row_scene_evictions.saturating_add(1);
        }
    }
    if let Some(started) = store_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_scene_store,
            &mut st.paint_perf_frame.ns_row_scene_store,
            started,
        );
    }
    if st.paint_perf_enabled {
        st.paint_perf_frame.rows_scene_stored =
            st.paint_perf_frame.rows_scene_stored.saturating_add(1);
        st.paint_perf_frame.row_scene_ops_stored = st
            .paint_perf_frame
            .row_scene_ops_stored
            .saturating_add(ops_len);
    }
}
