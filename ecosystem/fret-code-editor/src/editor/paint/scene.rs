#[cfg(feature = "syntax")]
use super::syntax::SyntaxSpan;
#[cfg(feature = "syntax")]
use crate::editor::syntax::ensure_syntax_row_cache_fresh;

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(super) enum RowSceneStoreSource {
    Paint,
    PrepaintEdge,
}

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

pub(in crate::editor) fn shift_row_scene_cache_for_single_line_edit(
    st: &mut CodeEditorState,
    before_line_rows: Range<usize>,
    after_line_rows: Range<usize>,
    edit_old_end: usize,
    edit_byte_delta: isize,
) {
    st.sync_row_scene_cache_epoch();
    #[cfg(feature = "syntax")]
    {
        st.row_scene_replay_plan_cache = None;
    }

    if st.row_scene_cache.is_empty() {
        return;
    }

    let row_delta = after_line_rows.len() as isize - before_line_rows.len() as isize;
    let old_cache = std::mem::take(&mut st.row_scene_cache);
    let mut new_cache = HashMap::with_capacity(old_cache.len());

    st.row_scene_cache_scene_ops_len_total = 0;

    for (row, (entry, tick)) in old_cache {
        if before_line_rows.contains(&row) {
            continue;
        }

        let Some(entry) =
            shift_row_scene_cache_entry_for_single_line_edit(entry, edit_old_end, edit_byte_delta)
        else {
            continue;
        };

        let new_row = if row >= before_line_rows.end {
            shift_usize(row, row_delta)
        } else {
            row
        };
        st.row_scene_cache_scene_ops_len_total = st
            .row_scene_cache_scene_ops_len_total
            .saturating_add(entry.retained.chunk.ops_len() as u64);
        new_cache.insert(new_row, (entry, tick));
    }

    st.row_scene_cache = new_cache;
    rebuild_row_scene_cache_queue(st);
}

fn shift_row_scene_cache_entry_for_single_line_edit(
    mut entry: RowSceneCacheEntry,
    edit_old_end: usize,
    edit_byte_delta: isize,
) -> Option<RowSceneCacheEntry> {
    if !matches!(entry.key.paint_key, RowScenePaintKey::Plain { .. }) {
        return None;
    }
    #[cfg(feature = "syntax")]
    if entry.syntax_replay_key.is_some() {
        return None;
    }

    entry.retained = shift_retained_row_scene_fragment_for_single_line_edit(
        entry.retained,
        edit_old_end,
        edit_byte_delta,
    )?;
    Some(entry)
}

fn shift_retained_row_scene_fragment_for_single_line_edit(
    retained: Arc<RowSceneRetainedFragment>,
    edit_old_end: usize,
    edit_byte_delta: isize,
) -> Option<Arc<RowSceneRetainedFragment>> {
    if retained.is_rich
        || retained.geom.fold_map.is_some()
        || retained.geom.has_preedit
        || retained.geom.preedit.is_some()
        || retained.content.range != retained.geom.row_range
    {
        return None;
    }

    let content = shift_row_content_snapshot_for_single_line_edit(
        Arc::clone(&retained.content),
        edit_old_end,
        edit_byte_delta,
    )?;
    let mut geom = retained.geom.clone();
    geom.row_range =
        shift_range_for_single_line_edit(geom.row_range, edit_old_end, edit_byte_delta);

    let mut shifted = (*retained).clone();
    shifted.content = content;
    shifted.geom = geom;
    Some(Arc::new(shifted))
}

fn shift_row_content_snapshot_for_single_line_edit(
    snapshot: Arc<RowContentSnapshot>,
    edit_old_end: usize,
    edit_byte_delta: isize,
) -> Option<Arc<RowContentSnapshot>> {
    if snapshot.preedit_range.is_some()
        || snapshot.fold_map.is_some()
        || !snapshot.row_spans.is_empty()
    {
        return None;
    }

    let range =
        shift_range_for_single_line_edit(snapshot.range.clone(), edit_old_end, edit_byte_delta);
    Some(Arc::new(RowContentSnapshot {
        text: Arc::clone(&snapshot.text),
        range,
        fold_map: None,
        preedit_range: None,
        row_spans: Arc::from([]),
    }))
}

fn rebuild_row_scene_cache_queue(st: &mut CodeEditorState) {
    let mut entries = st
        .row_scene_cache
        .iter()
        .map(|(row, (_, tick))| (*row, *tick))
        .collect::<Vec<_>>();
    entries.sort_by_key(|(_, tick)| *tick);
    st.row_scene_cache_queue = entries.into();
}

fn shift_range_for_single_line_edit(
    range: Range<usize>,
    edit_old_end: usize,
    delta: isize,
) -> Range<usize> {
    if range.end <= edit_old_end || delta == 0 {
        return range;
    }
    let start = shift_usize(range.start, delta);
    let end = shift_usize(range.end, delta);
    start..end.max(start)
}

fn shift_usize(value: usize, delta: isize) -> usize {
    if delta >= 0 {
        value.saturating_add(delta as usize)
    } else {
        value.saturating_sub((-delta) as usize)
    }
}

#[cfg(feature = "syntax")]
fn px_bits(value: Px) -> u32 {
    value.0.to_bits()
}

#[cfg(feature = "syntax")]
#[allow(clippy::too_many_arguments)]
fn replay_plan_cache_key_for_frame(
    st: &CodeEditorState,
    frame: WindowedRowsPaintFrame,
    content_bounds: Rect,
    max_entries: usize,
    text_style: &TextStyle,
    fg: Color,
    theme_revision: u64,
    constraints: CanvasTextConstraints,
    scale_factor: f32,
    row_count: usize,
    end: usize,
) -> Option<RowSceneReplayPlanCacheKey> {
    if frame.visible_start > end {
        return None;
    }

    Some(RowSceneReplayPlanCacheKey {
        buffer_revision: st.buffer.revision(),
        display_wrap_cols: st.display_wrap_cols,
        folds_epoch: st.folds_epoch,
        inlays_epoch: st.inlays_epoch,
        display_map_epoch: st.display_map_epoch,
        feature_payload_epoch: st.feature_payloads.epoch(),
        max_entries,
        row_count,
        row_height_bits: px_bits(frame.row_height),
        row_stride_bits: px_bits(frame.row_stride),
        gap_bits: px_bits(frame.gap),
        scroll_margin_bits: px_bits(frame.scroll_margin),
        content_origin_x_bits: px_bits(content_bounds.origin.x),
        content_width_bits: px_bits(content_bounds.size.width),
        content_height_bits: px_bits(content_bounds.size.height),
        text_style: RowSceneTextStyleKey::from_style(text_style),
        constraints: RowSceneTextConstraintsKey::from_constraints(constraints),
        font_stack_key: st.font_stack_key.0,
        scale_bits: scale_factor.max(1.0).to_bits(),
        theme_revision,
        code_font_feature_policy_rev: st.code_font_feature_policy_rev,
        fg: fg.into(),
    })
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
#[allow(clippy::too_many_arguments)]
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
        hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
        hosted_resources_touched: false,
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
        hosted_resources: fret_ui::canvas::CanvasHostedResources::default(),
        hosted_resources_touched: false,
    };

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
    if frame.visible_start > end {
        return plan;
    }
    let cache_key = replay_plan_cache_key_for_frame(
        st,
        frame,
        content_bounds,
        max_entries,
        text_style,
        fg,
        theme_revision,
        constraints,
        scale_factor,
        row_count,
        end,
    );
    let mut cached_entries_by_row = HashMap::<usize, Arc<RowSceneRetainedFragment>>::new();
    if let Some(cache_key) = cache_key.as_ref()
        && let Some(cached) = st.row_scene_replay_plan_cache.as_ref()
        && cached.key == *cache_key
    {
        cached_entries_by_row.reserve(cached.entries.len());
        cached_entries_by_row.extend(
            cached
                .entries
                .iter()
                .map(|entry| (entry.row, Arc::clone(&entry.retained))),
        );
    }
    enum ReplayCandidateProbe {
        NoCache,
        Unsupported,
        Preedit,
        KeyMismatch,
        Hit {
            retained: Arc<RowSceneRetainedFragment>,
            tick: u64,
        },
    }

    let plain_paint_key = RowScenePaintKey::Plain { fg: fg.into() };
    let mut planned = 0u64;
    for (row, rect) in frame.row_rects(content_bounds) {
        if row > end {
            break;
        }

        let row_requires_preedit = row_requires_paint_time_preedit(st, row);

        if !row_requires_preedit
            && let Some(cached_retained) = cached_entries_by_row.get(&row)
            && st
                .row_scene_cache
                .get(&row)
                .is_some_and(|(current, _)| Arc::ptr_eq(&current.retained, cached_retained))
        {
            plan.entries.push_back(RowSceneReplayPlanEntry {
                row,
                retained: Arc::clone(cached_retained),
                local_bounds: rect,
            });
            plan.hosted_resources
                .extend_resources(&cached_retained.hosted_resources);
            planned = planned.saturating_add(1);
            if st.paint_perf_enabled {
                st.paint_perf_frame.rows_scene_prepaint_plan_cache_hits = st
                    .paint_perf_frame
                    .rows_scene_prepaint_plan_cache_hits
                    .saturating_add(1);
            }
            continue;
        }

        if st.paint_perf_enabled && cached_entries_by_row.contains_key(&row) {
            st.paint_perf_frame.rows_scene_prepaint_plan_cache_rejects = st
                .paint_perf_frame
                .rows_scene_prepaint_plan_cache_rejects
                .saturating_add(1);
        }

        if st.paint_perf_enabled {
            st.paint_perf_frame.rows_scene_prepaint_candidates = st
                .paint_perf_frame
                .rows_scene_prepaint_candidates
                .saturating_add(1);
        }
        if row_requires_preedit {
            if st.paint_perf_enabled {
                st.paint_perf_frame.rows_scene_prepaint_skip_preedit = st
                    .paint_perf_frame
                    .rows_scene_prepaint_skip_preedit
                    .saturating_add(1);
            }
            continue;
        }
        let mut probe_started = st.paint_perf_enabled.then(Instant::now);
        let probe = match st.row_scene_cache.get_mut(&row) {
            Some((cached, last_used)) => {
                let content = &cached.retained.content;
                if cached.syntax_replay_key.is_none()
                    && !matches!(cached.key.paint_key, RowScenePaintKey::Plain { .. })
                {
                    ReplayCandidateProbe::Unsupported
                } else if content.preedit_range.is_some() {
                    ReplayCandidateProbe::Preedit
                } else {
                    let key_compare_started = st.paint_perf_enabled.then(Instant::now);
                    let matches = if let Some(key) = cached.syntax_replay_key.as_ref() {
                        key.matches_cached_replay_context(
                            content.as_ref(),
                            text_style,
                            constraints,
                            font_stack_key,
                            scale_factor,
                            theme_revision,
                            code_font_feature_policy_rev,
                            fg,
                        )
                    } else {
                        cached.key.paint_key == plain_paint_key
                    };
                    if let Some(started) = key_compare_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_prepaint_key_compare,
                            &mut st.paint_perf_frame.ns_row_scene_prepaint_key_compare,
                            started,
                        );
                    }

                    if matches {
                        if let Some(started) = probe_started.take() {
                            add_paint_perf_elapsed(
                                &mut st.paint_perf_frame.us_row_scene_prepaint_probe,
                                &mut st.paint_perf_frame.ns_row_scene_prepaint_probe,
                                started,
                            );
                        }
                        st.row_scene_cache_tick = st.row_scene_cache_tick.saturating_add(1);
                        let tick = st.row_scene_cache_tick;
                        *last_used = tick;
                        ReplayCandidateProbe::Hit {
                            retained: Arc::clone(&cached.retained),
                            tick,
                        }
                    } else {
                        ReplayCandidateProbe::KeyMismatch
                    }
                }
            }
            None => ReplayCandidateProbe::NoCache,
        };
        if let Some(started) = probe_started {
            add_paint_perf_elapsed(
                &mut st.paint_perf_frame.us_row_scene_prepaint_probe,
                &mut st.paint_perf_frame.ns_row_scene_prepaint_probe,
                started,
            );
        }

        let (retained, tick) = match probe {
            ReplayCandidateProbe::Hit {
                retained: candidate,
                tick,
            } => {
                st.cache_stats.row_scene_fast_get_calls =
                    st.cache_stats.row_scene_fast_get_calls.saturating_add(1);
                (candidate, tick)
            }
            ReplayCandidateProbe::NoCache => {
                if st.paint_perf_enabled {
                    st.paint_perf_frame.rows_scene_prepaint_skip_no_cache = st
                        .paint_perf_frame
                        .rows_scene_prepaint_skip_no_cache
                        .saturating_add(1);
                }
                continue;
            }
            ReplayCandidateProbe::Unsupported => {
                if st.paint_perf_enabled {
                    st.paint_perf_frame.rows_scene_prepaint_skip_unsupported_key = st
                        .paint_perf_frame
                        .rows_scene_prepaint_skip_unsupported_key
                        .saturating_add(1);
                }
                continue;
            }
            ReplayCandidateProbe::Preedit => {
                st.cache_stats.row_scene_fast_get_calls =
                    st.cache_stats.row_scene_fast_get_calls.saturating_add(1);
                if st.paint_perf_enabled {
                    st.paint_perf_frame.rows_scene_prepaint_skip_preedit = st
                        .paint_perf_frame
                        .rows_scene_prepaint_skip_preedit
                        .saturating_add(1);
                }
                continue;
            }
            ReplayCandidateProbe::KeyMismatch => {
                st.cache_stats.row_scene_fast_get_calls =
                    st.cache_stats.row_scene_fast_get_calls.saturating_add(1);
                if st.paint_perf_enabled {
                    st.paint_perf_frame.rows_scene_prepaint_skip_key_mismatch = st
                        .paint_perf_frame
                        .rows_scene_prepaint_skip_key_mismatch
                        .saturating_add(1);
                }
                st.cache_stats.row_scene_fast_misses =
                    st.cache_stats.row_scene_fast_misses.saturating_add(1);
                continue;
            }
        };

        st.row_scene_cache_queue.push_back((row, tick));

        plan.entries.push_back(RowSceneReplayPlanEntry {
            row,
            retained: Arc::clone(&retained),
            local_bounds: rect,
        });
        plan.hosted_resources
            .extend_resources(&retained.hosted_resources);
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
    st.row_scene_replay_plan_cache = if planned > 0 {
        cache_key.map(|key| RowSceneReplayPlanCache {
            key,
            entries: plan
                .entries
                .iter()
                .map(|entry| RowSceneReplayPlanCacheEntry {
                    row: entry.row,
                    retained: Arc::clone(&entry.retained),
                })
                .collect(),
        })
    } else {
        None
    };
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
    plan_hosted_resources: Option<&fret_ui::canvas::CanvasHostedResources>,
    entry: &RowSceneReplayPlanEntry,
    origin: Point,
) {
    let replay_delta = row_scene_replay_delta(entry.retained.origin, origin);
    if let Some(resources) = plan_hosted_resources
        && !resources.is_empty()
    {
        let touch_started = st.paint_perf_enabled.then(Instant::now);
        painter.touch_hosted_resources(resources);
        if let Some(started) = touch_started {
            add_paint_perf_elapsed(
                &mut st.paint_perf_frame.us_row_scene_replay_touch,
                &mut st.paint_perf_frame.ns_row_scene_replay_touch,
                started,
            );
        }
    } else {
        let touch_started = st.paint_perf_enabled.then(Instant::now);
        painter.touch_hosted_resources(&entry.retained.hosted_resources);
        if let Some(started) = touch_started {
            add_paint_perf_elapsed(
                &mut st.paint_perf_frame.us_row_scene_replay_touch,
                &mut st.paint_perf_frame.ns_row_scene_replay_touch,
                started,
            );
        }
    }
    let replay_started = st.paint_perf_enabled.then(Instant::now);
    entry
        .retained
        .chunk
        .replay_translated_into(painter.scene(), replay_delta);
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
        st.row_scene_replay_plan_cache = None;
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
                    let replay_delta = row_scene_replay_delta(cached.retained.origin, origin);
                    let touch_started = st.paint_perf_enabled.then(Instant::now);
                    painter.touch_hosted_resources(&cached.retained.hosted_resources);
                    if let Some(started) = touch_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_replay_touch,
                            &mut st.paint_perf_frame.ns_row_scene_replay_touch,
                            started,
                        );
                    }
                    let replay_started = st.paint_perf_enabled.then(Instant::now);
                    cached
                        .retained
                        .chunk
                        .replay_translated_into(painter.scene(), replay_delta);
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
                    Some((cached.retained.geom.clone(), cached.retained.is_rich))
                } else {
                    if st.paint_perf_enabled {
                        st.paint_perf_frame.rows_scene_fast_miss_key_mismatch = st
                            .paint_perf_frame
                            .rows_scene_fast_miss_key_mismatch
                            .saturating_add(1);
                    }
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
                if st.paint_perf_enabled {
                    st.paint_perf_frame.rows_scene_fast_miss_no_entry = st
                        .paint_perf_frame
                        .rows_scene_fast_miss_no_entry
                        .saturating_add(1);
                }
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
                    let replay_delta = row_scene_replay_delta(cached.retained.origin, origin);
                    let touch_started = st.paint_perf_enabled.then(Instant::now);
                    painter.touch_hosted_resources(&cached.retained.hosted_resources);
                    if let Some(started) = touch_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_replay_touch,
                            &mut st.paint_perf_frame.ns_row_scene_replay_touch,
                            started,
                        );
                    }
                    let replay_started = st.paint_perf_enabled.then(Instant::now);
                    cached
                        .retained
                        .chunk
                        .replay_translated_into(painter.scene(), replay_delta);
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
                    Some((cached.retained.geom.clone(), cached.retained.is_rich))
                } else {
                    if st.paint_perf_enabled {
                        st.paint_perf_frame.rows_scene_full_miss_key_mismatch = st
                            .paint_perf_frame
                            .rows_scene_full_miss_key_mismatch
                            .saturating_add(1);
                    }
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
                if st.paint_perf_enabled {
                    st.paint_perf_frame.rows_scene_full_miss_no_entry = st
                        .paint_perf_frame
                        .rows_scene_full_miss_no_entry
                        .saturating_add(1);
                }
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
    local_bounds: Rect,
    origin: fret_core::Point,
    geom: RowGeom,
    is_rich: bool,
    ops: Vec<SceneOp>,
    syntax_replay_key: Option<RowSceneSyntaxReplayKey>,
    max_entries: usize,
    source: RowSceneStoreSource,
) -> Option<Arc<RowSceneRetainedFragment>> {
    if max_entries == 0 || ops.is_empty() {
        return None;
    }

    ensure_row_scene_cache_fresh(st);
    let store_started = st.paint_perf_enabled.then(Instant::now);
    st.row_scene_cache_tick = st.row_scene_cache_tick.saturating_add(1);
    let tick = st.row_scene_cache_tick;
    let ops_len = ops.len() as u64;
    let chunk = fret_core::SceneChunk::from_ops(Arc::from(ops));
    let hosted_resources = fret_ui::canvas::CanvasHostedResources::from_scene_ops(chunk.ops());
    let retained = Arc::new(RowSceneRetainedFragment {
        content,
        local_bounds,
        origin,
        geom,
        is_rich,
        chunk,
        hosted_resources,
    });

    let replaced = st.row_scene_cache.insert(
        row,
        (
            RowSceneCacheEntry {
                key,
                retained: Arc::clone(&retained),
                syntax_replay_key,
            },
            tick,
        ),
    );
    if let Some((old, _)) = replaced {
        st.row_scene_cache_scene_ops_len_total = st
            .row_scene_cache_scene_ops_len_total
            .saturating_sub(old.retained.chunk.ops_len() as u64);
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
                    .saturating_sub(old.retained.chunk.ops_len() as u64);
            }
            st.cache_stats.row_scene_evictions =
                st.cache_stats.row_scene_evictions.saturating_add(1);
        }
    }
    match source {
        RowSceneStoreSource::Paint => {
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
                if row == st.paint_perf_frame.visible_start as usize {
                    st.paint_perf_frame.rows_scene_stored_at_visible_start = st
                        .paint_perf_frame
                        .rows_scene_stored_at_visible_start
                        .saturating_add(1);
                }
                if row == st.paint_perf_frame.visible_end as usize {
                    st.paint_perf_frame.rows_scene_stored_at_visible_end = st
                        .paint_perf_frame
                        .rows_scene_stored_at_visible_end
                        .saturating_add(1);
                }
                st.paint_perf_frame.row_scene_ops_stored = st
                    .paint_perf_frame
                    .row_scene_ops_stored
                    .saturating_add(ops_len);
            }
        }
        RowSceneStoreSource::PrepaintEdge => {
            if let Some(started) = store_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_row_scene_prepaint_edge_store,
                    &mut st.paint_perf_frame.ns_row_scene_prepaint_edge_store,
                    started,
                );
            }
            if st.paint_perf_enabled {
                st.paint_perf_frame.rows_scene_prepaint_edge_stored = st
                    .paint_perf_frame
                    .rows_scene_prepaint_edge_stored
                    .saturating_add(1);
                st.paint_perf_frame.row_scene_prepaint_edge_ops_stored = st
                    .paint_perf_frame
                    .row_scene_prepaint_edge_ops_stored
                    .saturating_add(ops_len);
            }
        }
    }

    Some(retained)
}

#[cfg(not(feature = "syntax"))]
#[allow(clippy::too_many_arguments)]
pub(super) fn store_row_scene_cache(
    st: &mut CodeEditorState,
    row: usize,
    key: RowSceneKey,
    content: Arc<RowContentSnapshot>,
    local_bounds: Rect,
    origin: fret_core::Point,
    geom: RowGeom,
    is_rich: bool,
    ops: Vec<SceneOp>,
    max_entries: usize,
) -> Option<Arc<RowSceneRetainedFragment>> {
    if max_entries == 0 || ops.is_empty() {
        return None;
    }

    ensure_row_scene_cache_fresh(st);
    let store_started = st.paint_perf_enabled.then(Instant::now);
    st.row_scene_cache_tick = st.row_scene_cache_tick.saturating_add(1);
    let tick = st.row_scene_cache_tick;
    let ops_len = ops.len() as u64;
    let chunk = fret_core::SceneChunk::from_ops(Arc::from(ops));
    let hosted_resources = fret_ui::canvas::CanvasHostedResources::from_scene_ops(chunk.ops());
    let retained = Arc::new(RowSceneRetainedFragment {
        content,
        local_bounds,
        origin,
        geom,
        is_rich,
        chunk,
        hosted_resources,
    });

    let replaced = st.row_scene_cache.insert(
        row,
        (
            RowSceneCacheEntry {
                key,
                retained: Arc::clone(&retained),
            },
            tick,
        ),
    );
    if let Some((old, _)) = replaced {
        st.row_scene_cache_scene_ops_len_total = st
            .row_scene_cache_scene_ops_len_total
            .saturating_sub(old.retained.chunk.ops_len() as u64);
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
                    .saturating_sub(old.retained.chunk.ops_len() as u64);
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
        if row == st.paint_perf_frame.visible_start as usize {
            st.paint_perf_frame.rows_scene_stored_at_visible_start = st
                .paint_perf_frame
                .rows_scene_stored_at_visible_start
                .saturating_add(1);
        }
        if row == st.paint_perf_frame.visible_end as usize {
            st.paint_perf_frame.rows_scene_stored_at_visible_end = st
                .paint_perf_frame
                .rows_scene_stored_at_visible_end
                .saturating_add(1);
        }
        st.paint_perf_frame.row_scene_ops_stored = st
            .paint_perf_frame
            .row_scene_ops_stored
            .saturating_add(ops_len);
    }

    Some(retained)
}
