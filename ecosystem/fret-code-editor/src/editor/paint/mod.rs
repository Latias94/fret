//! Painting, caching, and text shaping helpers for the code editor surface.

use fret_core::time::Instant;
use std::collections::{HashMap, VecDeque};
use std::ops::Range;

use super::*;
use fret_core::TextMetrics;

const ROW_CACHE_QUEUE_COMPACT_FACTOR: usize = 4;
const ROW_CACHE_QUEUE_COMPACT_MIN_LEN: usize = 1024;

fn compact_row_lru_queue_if_needed<T>(
    cache: &HashMap<usize, (T, u64)>,
    queue: &mut VecDeque<(usize, u64)>,
    max_entries: usize,
) -> bool {
    let budget_base = cache.len().max(max_entries).max(1);
    let compact_threshold = budget_base
        .saturating_mul(ROW_CACHE_QUEUE_COMPACT_FACTOR)
        .max(ROW_CACHE_QUEUE_COMPACT_MIN_LEN);
    if queue.len() <= compact_threshold {
        return false;
    }

    let mut entries = cache
        .iter()
        .map(|(row, (_, tick))| (*row, *tick))
        .collect::<Vec<_>>();
    entries.sort_by_key(|(_, tick)| *tick);
    queue.clear();
    queue.extend(entries);
    true
}

fn add_paint_perf_elapsed(us: &mut u64, ns: &mut u64, started: Instant) {
    let elapsed = started.elapsed();
    *us = us.saturating_add(elapsed.as_micros() as u64);
    let nanos = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
    *ns = ns.saturating_add(nanos);
}

#[cfg(feature = "syntax")]
fn normalize_syntax_spans_for_text(text: &str, spans: &mut Vec<SyntaxSpan>) {
    let max = text.len();

    for span in spans.iter_mut() {
        let mut start = span.range.start.min(max);
        let mut end = span.range.end.min(max).max(start);

        // Keep span boundaries grapheme-safe so we never split ZWJ/VS16 clusters.
        start = fret_code_editor_view::clamp_to_grapheme_boundary_down(text, start).min(max);
        end = fret_code_editor_view::clamp_to_grapheme_boundary_up(text, end)
            .min(max)
            .max(start);

        span.range = start..end;
    }

    spans.retain(|s| s.range.start < s.range.end);
    spans.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(a.range.end.cmp(&b.range.end))
            .then(a.highlight.cmp(&b.highlight))
    });
    spans.dedup_by(|a, b| a.range == b.range && a.highlight == b.highlight);

    // Ensure a stable, non-overlapping sequence even if inputs are stale after edits.
    let mut out: Vec<SyntaxSpan> = Vec::with_capacity(spans.len());
    let mut cursor = 0usize;
    for span in spans.drain(..) {
        let start = span.range.start.max(cursor);
        let end = span.range.end.max(start);
        if start >= end {
            continue;
        }

        if let Some(last) = out.last_mut()
            && last.highlight == span.highlight
            && last.range.end == start
        {
            last.range.end = end;
            cursor = last.range.end;
            continue;
        }

        cursor = end;
        out.push(SyntaxSpan {
            range: start..end,
            highlight: span.highlight,
        });
    }
    *spans = out;
}

#[cfg(feature = "syntax")]
fn mapped_row_syntax_spans_for_rich_text(
    line: &str,
    seg_start_in_line: usize,
    base_len: usize,
    row_spans: &[fret_code_editor_view::DisplayRowSpan],
    spans: &[SyntaxSpan],
) -> Option<Vec<SyntaxSpan>> {
    let seg_end_in_line = seg_start_in_line.saturating_add(base_len);
    let mut clipped: Vec<SyntaxSpan> = Vec::new();
    for span in spans {
        let start = span.range.start.max(seg_start_in_line);
        let end = span.range.end.min(seg_end_in_line);
        if start >= end {
            continue;
        }
        clipped.push(SyntaxSpan {
            range: (start - seg_start_in_line)..(end - seg_start_in_line),
            highlight: span.highlight,
        });
    }

    if clipped.is_empty() {
        return None;
    }

    clipped.sort_by_key(|s| s.range.start);
    clipped.dedup_by(|a, b| a.range == b.range && a.highlight == b.highlight);

    let mut merged: Vec<SyntaxSpan> = Vec::new();
    for span in clipped {
        if let Some(last) = merged.last_mut()
            && last.highlight == span.highlight
            && last.range.end == span.range.start
        {
            last.range.end = span.range.end;
            continue;
        }
        merged.push(span);
    }

    let mut mapped: Vec<SyntaxSpan> = Vec::new();
    if row_spans.is_empty() {
        mapped = merged;
    } else {
        for span in merged {
            let ranges = fret_code_editor_view::row_spans::map_buffer_range_to_display_ranges(
                row_spans,
                span.range.clone(),
                base_len,
                line.len(),
            );
            for r in ranges {
                let start = r.start.min(line.len());
                let end = r.end.min(line.len()).max(start);
                if start >= end {
                    continue;
                }
                mapped.push(SyntaxSpan {
                    range: start..end,
                    highlight: span.highlight,
                });
            }
        }
    }

    normalize_syntax_spans_for_text(line, &mut mapped);
    Some(mapped)
}

#[cfg(feature = "syntax")]
#[allow(clippy::too_many_arguments)]
fn store_row_rich_cache_entry(
    st: &mut CodeEditorState,
    row: usize,
    row_range: Range<usize>,
    line: Arc<str>,
    syntax_spans: Arc<[SyntaxSpan]>,
    row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
    theme_revision: u64,
    code_font_feature_policy_rev: u64,
    rich: AttributedText,
    max_entries: usize,
    tick: u64,
) {
    let entry_line_bytes = line.len() as u64;
    let entry_row_spans_len = row_spans.len() as u64;
    let entry_syntax_spans_len = syntax_spans.len() as u64;
    let entry_rich_spans_len = rich.spans.len() as u64;

    if let Some((old, _)) = st.row_rich_cache.insert(
        row,
        (
            RowRichCacheEntry {
                row_range,
                line,
                syntax_spans,
                row_spans,
                theme_revision,
                code_font_feature_policy_rev,
                rich,
            },
            tick,
        ),
    ) {
        st.row_rich_cache_line_bytes_estimate_total = st
            .row_rich_cache_line_bytes_estimate_total
            .saturating_sub(old.line.len() as u64);
        st.row_rich_cache_row_spans_len_total = st
            .row_rich_cache_row_spans_len_total
            .saturating_sub(old.row_spans.len() as u64);
        st.row_rich_cache_syntax_spans_len_total = st
            .row_rich_cache_syntax_spans_len_total
            .saturating_sub(old.syntax_spans.len() as u64);
        st.row_rich_cache_rich_spans_len_total = st
            .row_rich_cache_rich_spans_len_total
            .saturating_sub(old.rich.spans.len() as u64);
    }
    st.row_rich_cache_line_bytes_estimate_total = st
        .row_rich_cache_line_bytes_estimate_total
        .saturating_add(entry_line_bytes);
    st.row_rich_cache_row_spans_len_total = st
        .row_rich_cache_row_spans_len_total
        .saturating_add(entry_row_spans_len);
    st.row_rich_cache_syntax_spans_len_total = st
        .row_rich_cache_syntax_spans_len_total
        .saturating_add(entry_syntax_spans_len);
    st.row_rich_cache_rich_spans_len_total = st
        .row_rich_cache_rich_spans_len_total
        .saturating_add(entry_rich_spans_len);
    st.row_rich_cache_queue.push_back((row, tick));
    compact_row_lru_queue_if_needed(
        &st.row_rich_cache,
        &mut st.row_rich_cache_queue,
        max_entries,
    );

    while st.row_rich_cache.len() > max_entries {
        let Some((victim, victim_tick)) = st.row_rich_cache_queue.pop_front() else {
            break;
        };
        let remove = st
            .row_rich_cache
            .get(&victim)
            .is_some_and(|(_, last_used)| *last_used == victim_tick);
        if remove {
            if let Some((old, _)) = st.row_rich_cache.remove(&victim) {
                st.row_rich_cache_line_bytes_estimate_total = st
                    .row_rich_cache_line_bytes_estimate_total
                    .saturating_sub(old.line.len() as u64);
                st.row_rich_cache_row_spans_len_total = st
                    .row_rich_cache_row_spans_len_total
                    .saturating_sub(old.row_spans.len() as u64);
                st.row_rich_cache_syntax_spans_len_total = st
                    .row_rich_cache_syntax_spans_len_total
                    .saturating_sub(old.syntax_spans.len() as u64);
                st.row_rich_cache_rich_spans_len_total = st
                    .row_rich_cache_rich_spans_len_total
                    .saturating_sub(old.rich.spans.len() as u64);
            }
            st.cache_stats.row_rich_evictions = st.cache_stats.row_rich_evictions.saturating_add(1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_row(
    painter: &mut fret_ui::canvas::CanvasPainter<'_>,
    st: &mut CodeEditorState,
    row: usize,
    rect: Rect,
    row_h: Px,
    cell_w: Px,
    text_cache_max_entries: usize,
    text_style: &TextStyle,
    fg: Color,
    selection_bg: Color,
    caret_color: Color,
) {
    st.last_bounds = Some(painter.bounds());

    let perf_enabled = st.paint_perf_enabled;
    let row_started = perf_enabled.then(Instant::now);

    if perf_enabled {
        st.paint_perf_frame.rows_painted = st.paint_perf_frame.rows_painted.saturating_add(1);
    }

    let (row_range, line, row_folds, row_preedit_range, row_spans) = if perf_enabled {
        let started = Instant::now();
        let out = cached_row_text_with_range(st, row, text_cache_max_entries);
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_text,
            &mut st.paint_perf_frame.ns_row_text,
            started,
        );
        out
    } else {
        cached_row_text_with_range(st, row, text_cache_max_entries)
    };
    #[cfg(not(feature = "syntax"))]
    let _ = &row_spans;
    // Rows do not emit an inert transparent background quad here.
    // Hit-testing already lives in the surrounding PointerRegion.
    //
    // Align the text baseline within the row rect.
    //
    // `SceneOp::Text` expects a baseline origin. However, our editor rows are expressed as
    // top-left anchored rects (`rect.origin.y` is the row top), and `row_h` can exceed the
    // font's actual line height. Measure a representative line to compute a stable baseline and
    // vertically center the glyph box within the row.
    let scale_factor = painter.scale_factor();
    // Keep a stable (generous) max width for shaping so window resize drag doesn't force every
    // visible row to re-prepare text blobs on each pixel delta.
    //
    // We still rely on viewport scissoring for correctness; the max width is an upper bound to
    // avoid shaping arbitrarily long unwrapped lines.
    let stable_max_width = if cell_w.0 > 0.01 {
        // ~512 monospace columns is enough for typical editor viewports and keeps the cache key
        // stable across small/medium resizes.
        Px((cell_w.0 * 512.0).max(rect.size.width.0))
    } else {
        rect.size.width
    };
    let scale_bits = scale_factor.to_bits();
    let cached = st.baseline_measure_cache.as_ref().is_some_and(|cache| {
        cache.max_width == stable_max_width
            && cache.row_h == row_h
            && cache.scale_bits == scale_bits
            && &cache.text_style == text_style
    });
    let (metrics, measured_h) = if cached {
        let cache = st
            .baseline_measure_cache
            .as_ref()
            .expect("checked cache presence");
        (cache.metrics, cache.measured_h)
    } else {
        let (services, _) = painter.services_and_scene();
        let measure_constraints = fret_core::TextConstraints {
            max_width: Some(stable_max_width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor,
        };
        let started = perf_enabled.then(Instant::now);
        let metrics = services
            .text()
            .measure_str(" ", text_style, measure_constraints);
        if let Some(started) = started {
            add_paint_perf_elapsed(
                &mut st.paint_perf_frame.us_baseline_measure,
                &mut st.paint_perf_frame.ns_baseline_measure,
                started,
            );
        }
        let measured_h = if metrics.size.height.0 > 0.01 {
            metrics.size.height
        } else {
            // Defensive fallback: keep a stable non-zero box even if the text backend returns an
            // empty metrics set (should be rare for a single space).
            Px(row_h.0.max(16.0))
        };
        st.baseline_measure_cache = Some(BaselineMeasureCache {
            max_width: stable_max_width,
            row_h,
            scale_bits,
            text_style: text_style.clone(),
            metrics,
            measured_h,
        });
        (metrics, measured_h)
    };
    let text_y_pad = Px(((row_h.0 - measured_h.0).max(0.0)) / 2.0);
    let origin = fret_core::Point::new(
        rect.origin.x,
        Px(rect.origin.y.0 + text_y_pad.0 + metrics.baseline.0),
    );
    let scope = painter.key_scope(&"fret-code-editor-row-text");
    let key: u64 = painter.child_key(scope, &(row, 0u8)).into();
    let constraints = CanvasTextConstraints {
        max_width: Some(stable_max_width),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let mut drew_rich = false;
    let mut row_preedit = None::<RowPreeditMapping>;
    let mut row_blob = None::<fret_core::TextBlobId>;
    let mut row_blob_metrics = None::<TextMetrics>;
    let mut row_geom_key = None::<geom::RowGeomKey>;
    let mut row_scene_key = None::<RowSceneKey>;
    let mut row_scene_is_rich = false;
    let mut row_scene_replayed = false;
    #[cfg(feature = "syntax")]
    let mut row_scene_syntax_replay_key = None::<RowSceneSyntaxReplayKey>;
    let mut fresh_geom = None::<RowGeom>;
    let row_scene_ops_start = painter.scene().ops_len();
    let compose_inline_preedit = st.compose_inline_preedit
        || st
            .preedit_replace_range
            .as_ref()
            .is_some_and(|r| !r.is_empty());

    if let Some(preedit) = st.preedit.clone() {
        if compose_inline_preedit {
            if let Some(range) = row_preedit_range.clone() {
                let rich = materialize_preedit_rich_text_for_range(
                    Arc::clone(&line),
                    range,
                    &st.code_font_shaping_style,
                    &preedit,
                    fg,
                    selection_bg,
                );
                row_geom_key = Some(geom::RowGeomKey::for_attributed(
                    &rich,
                    text_style,
                    (
                        constraints.max_width,
                        constraints.wrap,
                        constraints.overflow,
                        fret_core::TextAlign::Start,
                        scale_factor,
                    ),
                    st.font_stack_key,
                ));
                row_scene_key = row_geom_key
                    .clone()
                    .map(|key| RowSceneKey::preedit(key, fg, selection_bg));
                row_scene_is_rich = true;
                if let Some(scene_key) = row_scene_key.clone()
                    && let Some((geom, is_rich)) = try_replay_row_scene_cache(
                        painter,
                        st,
                        row,
                        &scene_key,
                        origin,
                        text_cache_max_entries,
                    )
                {
                    fresh_geom = Some(geom);
                    row_preedit = fresh_geom.as_ref().and_then(|g| g.preedit);
                    row_scene_replayed = true;
                    row_scene_is_rich = is_rich;
                    drew_rich = is_rich;
                }
                if !row_scene_replayed {
                    let key: u64 = painter.child_key(scope, &(row, 2u8)).into();
                    let (blob, metrics) = painter.rich_text_with_blob(
                        key,
                        DrawOrder(2),
                        origin,
                        rich,
                        text_style.clone(),
                        fg,
                        constraints,
                        scale_factor,
                    );
                    row_blob = Some(blob);
                    row_blob_metrics = Some(metrics);
                    drew_rich = true;
                }
            }
        } else {
            let caret = st.selection.caret().min(st.buffer.len_bytes());
            let caret_pt = st.display_map.byte_to_display_point(&st.buffer, caret);
            if caret_pt.row == row {
                let caret_local = caret.saturating_sub(row_range.start);
                let mut caret_in_line = caret_local.min(line.len());
                if let Some(folds) = row_folds.as_ref() {
                    caret_in_line = folds
                        .buffer_local_to_display_local(caret_local)
                        .min(line.len());
                }
                caret_in_line =
                    fret_code_editor_view::clamp_to_char_boundary(line.as_ref(), caret_in_line);

                let rich = materialize_preedit_rich_text(
                    Arc::clone(&line),
                    caret_in_line,
                    &st.code_font_shaping_style,
                    &preedit,
                    fg,
                    selection_bg,
                );
                row_geom_key = Some(geom::RowGeomKey::for_attributed(
                    &rich,
                    text_style,
                    (
                        constraints.max_width,
                        constraints.wrap,
                        constraints.overflow,
                        fret_core::TextAlign::Start,
                        scale_factor,
                    ),
                    st.font_stack_key,
                ));
                row_scene_key = row_geom_key
                    .clone()
                    .map(|key| RowSceneKey::preedit(key, fg, selection_bg));
                row_scene_is_rich = true;
                if let Some(scene_key) = row_scene_key.clone()
                    && let Some((geom, is_rich)) = try_replay_row_scene_cache(
                        painter,
                        st,
                        row,
                        &scene_key,
                        origin,
                        text_cache_max_entries,
                    )
                {
                    fresh_geom = Some(geom);
                    row_preedit = fresh_geom.as_ref().and_then(|g| g.preedit);
                    row_scene_replayed = true;
                    drew_rich = is_rich;
                }
                if !row_scene_replayed {
                    let key: u64 = painter.child_key(scope, &(row, 2u8)).into();
                    let started = perf_enabled.then(Instant::now);
                    let (blob, metrics) = painter.rich_text_with_blob(
                        key,
                        DrawOrder(2),
                        origin,
                        rich,
                        text_style.clone(),
                        fg,
                        constraints,
                        scale_factor,
                    );
                    if let Some(started) = started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_text_draw,
                            &mut st.paint_perf_frame.ns_text_draw,
                            started,
                        );
                    }
                    row_preedit = Some(RowPreeditMapping {
                        insert_at: caret_in_line,
                        preedit_len: preedit.text.len(),
                    });
                    row_blob = Some(blob);
                    row_blob_metrics = Some(metrics);
                    drew_rich = true;
                }
            }
        }
    }
    let row_has_preedit = st.preedit.is_some()
        && if compose_inline_preedit {
            row_preedit_range.is_some()
        } else {
            row_preedit.is_some()
        };
    #[cfg(feature = "syntax")]
    let line_idx = st.display_map.display_row_line(row);
    #[cfg(feature = "syntax")]
    let theme_revision = {
        let theme = painter.theme();
        theme.revision()
    };
    #[cfg(feature = "syntax")]
    #[allow(unused_assignments)]
    let mut syntax_spans = None::<Arc<[SyntaxSpan]>>;
    #[cfg(feature = "syntax")]
    let mut syntax_lookup_miss_tick = None::<u64>;
    #[cfg(feature = "syntax")]
    {
        let lookup_started = perf_enabled.then(Instant::now);
        match lookup_row_syntax_spans(st, line_idx, text_cache_max_entries) {
            SyntaxRowCacheLookup::Hit(spans) => {
                syntax_spans = Some(Arc::clone(&spans));
                if let Some(started) = lookup_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_syntax_spans,
                        &mut st.paint_perf_frame.ns_syntax_spans,
                        started,
                    );
                }

                if !row_scene_replayed
                    && !drew_rich
                    && let Some((geom, is_rich)) = try_replay_row_scene_cache_fast_syntax(
                        painter,
                        st,
                        row,
                        &row_range,
                        &line,
                        &row_spans,
                        &spans,
                        text_style,
                        constraints,
                        st.font_stack_key,
                        scale_factor,
                        theme_revision,
                        st.code_font_feature_policy_rev,
                        fg,
                        origin,
                        text_cache_max_entries,
                    )
                {
                    fresh_geom = Some(geom);
                    row_preedit = fresh_geom.as_ref().and_then(|g| g.preedit);
                    row_scene_replayed = true;
                    drew_rich = is_rich;
                }
            }
            SyntaxRowCacheLookup::Miss { tick } => {
                syntax_lookup_miss_tick = Some(tick);
                if let Some(started) = lookup_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_syntax_spans,
                        &mut st.paint_perf_frame.ns_syntax_spans,
                        started,
                    );
                }
            }
        }
    }
    #[cfg(feature = "syntax")]
    {
        if !row_scene_replayed && !drew_rich {
            let spans = if let Some(spans) = syntax_spans.as_ref() {
                Arc::clone(spans)
            } else {
                let started = perf_enabled.then(Instant::now);
                let tick = syntax_lookup_miss_tick
                    .take()
                    .expect("syntax lookup miss tick must exist after a miss");
                let spans = populate_row_syntax_spans_after_miss(
                    st,
                    line_idx,
                    text_cache_max_entries,
                    tick,
                );
                if let Some(started) = started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_syntax_spans,
                        &mut st.paint_perf_frame.ns_syntax_spans,
                        started,
                    );
                }
                spans
            };
            if !spans.is_empty() {
                let rich_cache_max_entries = text_cache_max_entries.min(2048);
                st.cache_stats.row_rich_get_calls =
                    st.cache_stats.row_rich_get_calls.saturating_add(1);

                let seg_start_in_line = row_range
                    .start
                    .saturating_sub(st.buffer.line_start(line_idx).unwrap_or(row_range.start));
                let base_len = row_range.end.saturating_sub(row_range.start);

                st.row_rich_cache_tick = st.row_rich_cache_tick.saturating_add(1);
                let tick = st.row_rich_cache_tick;

                let mut cached_rich_hit = None::<AttributedText>;
                if let Some((cached, last_used)) = st.row_rich_cache.get_mut(&row) {
                    let hit = cached.theme_revision == theme_revision
                        && cached.row_range == row_range
                        && cached.code_font_feature_policy_rev == st.code_font_feature_policy_rev
                        && arc_str_ptr_or_content_eq(&cached.line, &line)
                        && arc_slice_ptr_or_content_eq(&cached.syntax_spans, &spans)
                        && arc_slice_ptr_or_content_eq(&cached.row_spans, &row_spans);
                    if hit {
                        *last_used = tick;
                        cached_rich_hit = Some(cached.rich.clone());
                    }
                }
                if cached_rich_hit.is_some() {
                    st.row_rich_cache_queue.push_back((row, tick));
                    compact_row_lru_queue_if_needed(
                        &st.row_rich_cache,
                        &mut st.row_rich_cache_queue,
                        rich_cache_max_entries,
                    );
                    st.cache_stats.row_rich_hits = st.cache_stats.row_rich_hits.saturating_add(1);
                }

                if let Some(rich) = cached_rich_hit {
                    row_geom_key = Some(geom::RowGeomKey::for_attributed(
                        &rich,
                        text_style,
                        (
                            constraints.max_width,
                            constraints.wrap,
                            constraints.overflow,
                            fret_core::TextAlign::Start,
                            scale_factor,
                        ),
                        st.font_stack_key,
                    ));
                    row_scene_key = row_geom_key
                        .clone()
                        .map(|key| RowSceneKey::syntax(key, fg, theme_revision));
                    #[cfg(feature = "syntax")]
                    {
                        row_scene_syntax_replay_key = Some(RowSceneSyntaxReplayKey::new(
                            row_range.clone(),
                            Arc::clone(&line),
                            Arc::clone(&row_spans),
                            Arc::clone(&spans),
                            text_style,
                            constraints,
                            st.font_stack_key,
                            scale_factor,
                            theme_revision,
                            st.code_font_feature_policy_rev,
                            fg,
                        ));
                    }
                    row_scene_is_rich = true;
                    if let Some(scene_key) = row_scene_key.clone()
                        && let Some((geom, is_rich)) = try_replay_row_scene_cache(
                            painter,
                            st,
                            row,
                            &scene_key,
                            origin,
                            text_cache_max_entries,
                        )
                    {
                        fresh_geom = Some(geom);
                        row_preedit = fresh_geom.as_ref().and_then(|g| g.preedit);
                        row_scene_replayed = true;
                        drew_rich = is_rich;
                    }

                    if !row_scene_replayed {
                        let started = perf_enabled.then(Instant::now);
                        let (blob, metrics) = painter.rich_text_with_blob(
                            key,
                            DrawOrder(2),
                            origin,
                            rich,
                            text_style.clone(),
                            fg,
                            constraints,
                            scale_factor,
                        );
                        if let Some(started) = started {
                            add_paint_perf_elapsed(
                                &mut st.paint_perf_frame.us_text_draw,
                                &mut st.paint_perf_frame.ns_text_draw,
                                started,
                            );
                        }
                        row_blob = Some(blob);
                        row_blob_metrics = Some(metrics);
                        drew_rich = true;
                    }
                }

                if !drew_rich {
                    st.cache_stats.row_rich_misses =
                        st.cache_stats.row_rich_misses.saturating_add(1);

                    if let Some(mapped) = mapped_row_syntax_spans_for_rich_text(
                        line.as_ref(),
                        seg_start_in_line,
                        base_len,
                        row_spans.as_ref(),
                        spans.as_ref(),
                    ) {
                        let started = perf_enabled.then(Instant::now);
                        let rich = {
                            let theme = painter.theme();
                            materialize_row_rich_text(
                                theme,
                                Arc::clone(&line),
                                mapped.as_ref(),
                                &st.code_font_shaping_style,
                            )
                        };
                        row_geom_key = Some(geom::RowGeomKey::for_attributed(
                            &rich,
                            text_style,
                            (
                                constraints.max_width,
                                constraints.wrap,
                                constraints.overflow,
                                fret_core::TextAlign::Start,
                                scale_factor,
                            ),
                            st.font_stack_key,
                        ));
                        row_scene_key = row_geom_key
                            .clone()
                            .map(|key| RowSceneKey::syntax(key, fg, theme_revision));
                        #[cfg(feature = "syntax")]
                        {
                            row_scene_syntax_replay_key = Some(RowSceneSyntaxReplayKey::new(
                                row_range.clone(),
                                Arc::clone(&line),
                                Arc::clone(&row_spans),
                                Arc::clone(&spans),
                                text_style,
                                constraints,
                                st.font_stack_key,
                                scale_factor,
                                theme_revision,
                                st.code_font_feature_policy_rev,
                                fg,
                            ));
                        }
                        row_scene_is_rich = true;
                        if let Some(started) = started {
                            add_paint_perf_elapsed(
                                &mut st.paint_perf_frame.us_rich_materialize,
                                &mut st.paint_perf_frame.ns_rich_materialize,
                                started,
                            );
                        }
                        store_row_rich_cache_entry(
                            st,
                            row,
                            row_range.clone(),
                            Arc::clone(&line),
                            Arc::clone(&spans),
                            Arc::clone(&row_spans),
                            theme_revision,
                            st.code_font_feature_policy_rev,
                            rich.clone(),
                            rich_cache_max_entries,
                            tick,
                        );

                        if let Some(scene_key) = row_scene_key.clone()
                            && let Some((geom, is_rich)) = try_replay_row_scene_cache(
                                painter,
                                st,
                                row,
                                &scene_key,
                                origin,
                                text_cache_max_entries,
                            )
                        {
                            fresh_geom = Some(geom);
                            row_preedit = fresh_geom.as_ref().and_then(|g| g.preedit);
                            row_scene_replayed = true;
                            drew_rich = is_rich;
                        }

                        if !row_scene_replayed {
                            let started = perf_enabled.then(Instant::now);
                            let (blob, metrics) = painter.rich_text_with_blob(
                                key,
                                DrawOrder(2),
                                origin,
                                rich,
                                text_style.clone(),
                                fg,
                                constraints,
                                scale_factor,
                            );
                            if let Some(started) = started {
                                add_paint_perf_elapsed(
                                    &mut st.paint_perf_frame.us_text_draw,
                                    &mut st.paint_perf_frame.ns_text_draw,
                                    started,
                                );
                            }
                            row_blob = Some(blob);
                            row_blob_metrics = Some(metrics);
                            drew_rich = true;
                        }
                    }
                }
            }
        }
    }

    if !row_scene_replayed && !drew_rich {
        if !st.code_font_shaping_style.features.is_empty() {
            let rich = AttributedText::new(
                Arc::clone(&line),
                vec![TextSpan {
                    len: line.len(),
                    shaping: st.code_font_shaping_style.clone(),
                    paint: Default::default(),
                }],
            );
            row_geom_key = Some(geom::RowGeomKey::for_attributed(
                &rich,
                text_style,
                (
                    constraints.max_width,
                    constraints.wrap,
                    constraints.overflow,
                    fret_core::TextAlign::Start,
                    scale_factor,
                ),
                st.font_stack_key,
            ));
            row_scene_key = row_geom_key.clone().map(|key| RowSceneKey::plain(key, fg));
            row_scene_is_rich = true;
            if let Some(scene_key) = row_scene_key.clone()
                && let Some((geom, is_rich)) = try_replay_row_scene_cache(
                    painter,
                    st,
                    row,
                    &scene_key,
                    origin,
                    text_cache_max_entries,
                )
            {
                fresh_geom = Some(geom);
                row_preedit = fresh_geom.as_ref().and_then(|g| g.preedit);
                row_scene_replayed = true;
                drew_rich = is_rich;
            }
            if !row_scene_replayed {
                let started = perf_enabled.then(Instant::now);
                let (blob, metrics) = painter.rich_text_with_blob(
                    key,
                    DrawOrder(2),
                    origin,
                    rich,
                    text_style.clone(),
                    fg,
                    constraints,
                    scale_factor,
                );
                if let Some(started) = started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_text_draw,
                        &mut st.paint_perf_frame.ns_text_draw,
                        started,
                    );
                }
                row_blob = Some(blob);
                row_blob_metrics = Some(metrics);
                drew_rich = true;
            }
        } else {
            row_geom_key = Some(geom::RowGeomKey::for_plain(
                &line,
                text_style,
                (
                    constraints.max_width,
                    constraints.wrap,
                    constraints.overflow,
                    fret_core::TextAlign::Start,
                    scale_factor,
                ),
                st.font_stack_key,
            ));
            row_scene_key = row_geom_key.clone().map(|key| RowSceneKey::plain(key, fg));
            if let Some(scene_key) = row_scene_key.clone()
                && let Some((geom, is_rich)) = try_replay_row_scene_cache(
                    painter,
                    st,
                    row,
                    &scene_key,
                    origin,
                    text_cache_max_entries,
                )
            {
                fresh_geom = Some(geom);
                row_preedit = fresh_geom.as_ref().and_then(|g| g.preedit);
                row_scene_replayed = true;
                drew_rich = is_rich;
            }
            if !row_scene_replayed {
                let started = perf_enabled.then(Instant::now);
                let (blob, metrics) = painter.text_with_blob(
                    key,
                    DrawOrder(2),
                    origin,
                    Arc::clone(&line),
                    text_style.clone(),
                    fg,
                    constraints,
                    scale_factor,
                );
                if let Some(started) = started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_text_draw,
                        &mut st.paint_perf_frame.ns_text_draw,
                        started,
                    );
                }
                row_blob = Some(blob);
                row_blob_metrics = Some(metrics);
            }
        }
    }

    let mut caret_stops = &[][..];
    let mut caret_rect_top = None::<Px>;
    let mut caret_rect_height = None::<Px>;
    if let Some(geom) = fresh_geom.as_ref() {
        caret_stops = geom.caret_stops.as_slice();
        caret_rect_top = geom.caret_rect_top;
        caret_rect_height = geom.caret_rect_height;
    }
    if let (Some(blob), Some(blob_metrics), Some(row_geom_key)) =
        (row_blob, row_blob_metrics.as_ref(), row_geom_key)
    {
        let cached = st.row_geom_cache.get(&row).is_some_and(|(geom, _)| {
            geom.key == row_geom_key
                && geom.row_range == row_range
                && geom.has_preedit == row_has_preedit
                && geom.preedit == row_preedit
        });
        if cached {
            let geom = &st
                .row_geom_cache
                .get(&row)
                .expect("checked cache presence")
                .0;
            caret_stops = geom.caret_stops.as_slice();
            caret_rect_top = geom.caret_rect_top;
            caret_rect_height = geom.caret_rect_height;
        } else {
            let mut stops: Vec<(usize, Px)> = Vec::new();
            let (services, _) = painter.services_and_scene();
            let caret_stops_started = perf_enabled.then(Instant::now);
            services.text().caret_stops(blob, &mut stops);
            if let Some(started) = caret_stops_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_caret_stops,
                    &mut st.paint_perf_frame.ns_caret_stops,
                    started,
                );
            }
            let caret_rect_started = perf_enabled.then(Instant::now);
            let caret_rect = services
                .text()
                .caret_rect(blob, 0, CaretAffinity::Downstream);
            if let Some(started) = caret_rect_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_caret_rect,
                    &mut st.paint_perf_frame.ns_caret_rect,
                    started,
                );
            }

            // `caret_rect` is relative to the text box top (y=0 at the top of the blob box).
            // Convert it into row-local coordinates by anchoring the box using the *actual* blob
            // baseline, not the placeholder measurement baseline.
            let text_box_top_in_row = Px(origin.y.0 - blob_metrics.baseline.0 - rect.origin.y.0);
            if caret_rect.size.height.0 > 0.0 {
                caret_rect_top = Some(Px(text_box_top_in_row.0 + caret_rect.origin.y.0));
                caret_rect_height = Some(caret_rect.size.height);
            } else if blob_metrics.size.height.0 > 0.0 {
                // Some backends may not provide a caret rect yet. Fall back to the blob's box so
                // the caret doesn't appear "floating" at the row top.
                caret_rect_top = Some(text_box_top_in_row);
                caret_rect_height = Some(blob_metrics.size.height);
            }

            fresh_geom = Some(RowGeom {
                row_range: row_range.clone(),
                key: row_geom_key,
                caret_stops: stops,
                fold_map: row_folds.clone(),
                caret_rect_top,
                caret_rect_height,
                has_preedit: row_has_preedit,
                preedit: row_preedit,
            });
            caret_stops = fresh_geom
                .as_ref()
                .expect("fresh geom present")
                .caret_stops
                .as_slice();
        }
    }

    let pending_row_scene_store = if !row_scene_replayed {
        row_scene_key.clone().and_then(|row_scene_key| {
            let row_scene_ops_end = painter.scene().ops_len();
            if row_scene_ops_end <= row_scene_ops_start {
                return None;
            }
            let geom = fresh_geom
                .clone()
                .or_else(|| st.row_geom_cache.get(&row).map(|(geom, _)| geom.clone()))?;
            let capture_started = perf_enabled.then(Instant::now);
            let ops = {
                let scene = painter.scene();
                scene.ops()[row_scene_ops_start..row_scene_ops_end].to_vec()
            };
            if let Some(started) = capture_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_row_scene_capture_ops,
                    &mut st.paint_perf_frame.ns_row_scene_capture_ops,
                    started,
                );
            }
            #[cfg(feature = "syntax")]
            {
                Some((
                    row_scene_key,
                    geom,
                    row_scene_is_rich,
                    ops,
                    row_scene_syntax_replay_key.clone(),
                ))
            }
            #[cfg(not(feature = "syntax"))]
            {
                Some((row_scene_key, geom, row_scene_is_rich, ops))
            }
        })
    } else {
        None
    };

    let sel = st.selection.normalized();
    let mut drew_selection = false;
    if !sel.is_empty() {
        let global_start = sel.start.max(row_range.start).min(row_range.end);
        let global_end = sel.end.max(row_range.start).min(row_range.end);
        if global_start < global_end
            && let Some(blob) = row_blob
        {
            let mut local_start = global_start.saturating_sub(row_range.start);
            let mut local_end = global_end.saturating_sub(row_range.start);
            if let Some(folds) = &row_folds {
                local_start = folds.buffer_local_to_display_local(local_start);
                local_end = folds.buffer_local_to_display_local(local_end);
            }
            local_start = local_start.min(line.len());
            local_end = local_end.min(line.len());
            if local_start < local_end {
                let (services, _) = painter.services_and_scene();
                st.selection_rect_scratch.clear();
                let started = perf_enabled.then(Instant::now);
                services.text().selection_rects(
                    blob,
                    (local_start, local_end),
                    &mut st.selection_rect_scratch,
                );
                if let Some(started) = started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_selection_rects,
                        &mut st.paint_perf_frame.ns_selection_rects,
                        started,
                    );
                }

                for local_rect in st.selection_rect_scratch.iter().copied() {
                    let x0 = local_rect.origin.x.0;
                    let x1 = x0 + local_rect.size.width.0;
                    let x0 = x0.clamp(0.0, rect.size.width.0);
                    let x1 = x1.clamp(0.0, rect.size.width.0);
                    let w = (x1 - x0).max(0.0);
                    if w <= 0.0 {
                        continue;
                    }
                    let sel_rect = Rect::new(
                        fret_core::Point::new(Px(rect.origin.x.0 + x0), rect.origin.y),
                        Size::new(Px(w), row_h),
                    );
                    painter.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: sel_rect,
                        background: fret_core::Paint::Solid(selection_bg).into(),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),

                        corner_radii: Corners::all(Px(0.0)),
                    });
                    if perf_enabled {
                        st.paint_perf_frame.quads_selection =
                            st.paint_perf_frame.quads_selection.saturating_add(1);
                    }
                    drew_selection = true;
                }
            }
        }
    }

    if !caret_stops.is_empty() {
        // Draw selection using caret stops so that selection geometry matches hit-testing.
        if !drew_selection && !sel.is_empty() {
            let global_start = sel.start.max(row_range.start).min(row_range.end);
            let global_end = sel.end.max(row_range.start).min(row_range.end);
            if global_start < global_end {
                let mut local_start = global_start.saturating_sub(row_range.start);
                let mut local_end = global_end.saturating_sub(row_range.start);
                if let Some(folds) = &row_folds {
                    local_start = folds.buffer_local_to_display_local(local_start);
                    local_end = folds.buffer_local_to_display_local(local_end);
                }
                let mut ranges: Vec<(usize, usize)> = Vec::new();
                if let Some(preedit) = row_preedit {
                    // Paint-time preedit injection: selection indices are expressed in the base
                    // (pre-injection) row string, but caret stops are measured against the injected
                    // blob. Split and shift the selection range to keep the injected preedit gap
                    // unselected.
                    if local_end <= preedit.insert_at {
                        ranges.push((local_start, local_end));
                    } else if local_start >= preedit.insert_at {
                        ranges.push((
                            local_start.saturating_add(preedit.preedit_len),
                            local_end.saturating_add(preedit.preedit_len),
                        ));
                    } else {
                        ranges.push((local_start, preedit.insert_at));
                        ranges.push((
                            preedit.insert_at.saturating_add(preedit.preedit_len),
                            local_end.saturating_add(preedit.preedit_len),
                        ));
                    }
                } else {
                    // View-composed preedit: selection indices are already in the composed row
                    // string coordinate space. Remove the composed preedit range so we don't select
                    // uncommitted text.
                    ranges.push((local_start, local_end));
                    if let Some(gap) = row_preedit_range.as_ref() {
                        let gap_start = gap.start;
                        let gap_end = gap.end;
                        let mut clipped: Vec<(usize, usize)> = Vec::new();
                        for (a, b) in ranges.drain(..) {
                            if b <= gap_start || a >= gap_end {
                                clipped.push((a, b));
                                continue;
                            }
                            if a < gap_start {
                                clipped.push((a, gap_start));
                            }
                            if b > gap_end {
                                clipped.push((gap_end, b));
                            }
                        }
                        ranges = clipped;
                    }
                }

                for (a, b) in ranges {
                    if a >= b {
                        continue;
                    }
                    let x0 = caret_x_for_index(caret_stops, a);
                    let x1 = caret_x_for_index(caret_stops, b);
                    if x0.0 == x1.0 {
                        continue;
                    }
                    let x = Px(rect.origin.x.0 + x0.0.min(x1.0));
                    let w = Px((x1.0 - x0.0).abs());
                    let sel_rect =
                        Rect::new(fret_core::Point::new(x, rect.origin.y), Size::new(w, row_h));
                    painter.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: sel_rect,
                        background: fret_core::Paint::Solid(selection_bg).into(),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),

                        corner_radii: Corners::all(Px(0.0)),
                    });
                    if perf_enabled {
                        st.paint_perf_frame.quads_selection =
                            st.paint_perf_frame.quads_selection.saturating_add(1);
                    }
                }
            }
        }

        // Draw caret using caret stops so that caret geometry matches hit-testing and IME anchoring.
        if st.selection.is_caret() {
            let caret = st.selection.caret().min(st.buffer.len_bytes());
            let caret_pt = st.display_map.byte_to_display_point(&st.buffer, caret);
            if caret_pt.row == row {
                let mut local = caret.saturating_sub(row_range.start);
                if let Some(folds) = &row_folds {
                    local = folds.buffer_local_to_display_local(local);
                }
                if let Some(preedit) = &st.preedit
                    && (row_preedit.is_some() || row_preedit_range.is_some())
                {
                    local = local.saturating_add(preedit_cursor_offset_bytes(preedit));
                }
                let x0 = caret_x_for_index(caret_stops, local);
                let (caret_top, caret_h) = if let (Some(top), Some(h)) =
                    (caret_rect_top, caret_rect_height)
                    && h.0 > 0.0
                {
                    (top, Px(h.0.min(row_h.0)))
                } else {
                    (Px(0.0), row_h)
                };
                let caret_rect = Rect::new(
                    fret_core::Point::new(
                        Px(rect.origin.x.0 + x0.0),
                        Px(rect.origin.y.0 + caret_top.0),
                    ),
                    Size::new(Px(1.0), caret_h),
                );
                painter.scene().push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: caret_rect,
                    background: fret_core::Paint::Solid(caret_color).into(),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),

                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }
    } else {
        // Fallback to the MVP monospace heuristic if caret stops are unavailable.
        if !drew_selection && !sel.is_empty() {
            let start_pt = st.display_map.byte_to_display_point(&st.buffer, sel.start);
            let end_pt = st.display_map.byte_to_display_point(&st.buffer, sel.end);
            if row >= start_pt.row && row <= end_pt.row {
                let line_cols = line.chars().count();
                let start_col = if row == start_pt.row { start_pt.col } else { 0 };
                let end_col = if row == end_pt.row {
                    end_pt.col
                } else {
                    line_cols
                };
                if start_col != end_col {
                    let x0 = Px(rect.origin.x.0 + start_col as f32 * cell_w.0);
                    let x1 = Px(rect.origin.x.0 + end_col as f32 * cell_w.0);
                    let x = Px(x0.0.min(x1.0));
                    let w = Px((x1.0 - x0.0).abs());
                    let sel_rect =
                        Rect::new(fret_core::Point::new(x, rect.origin.y), Size::new(w, row_h));
                    painter.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: sel_rect,
                        background: fret_core::Paint::Solid(selection_bg).into(),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),

                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }
        }

        if st.selection.is_caret() {
            let caret = st.selection.caret().min(st.buffer.len_bytes());
            let caret_pt = st.display_map.byte_to_display_point(&st.buffer, caret);
            if caret_pt.row == row {
                let caret_rect = if let Some(blob) = row_blob {
                    let mut local = caret.saturating_sub(row_range.start);
                    if let Some(folds) = &row_folds {
                        local = folds.buffer_local_to_display_local(local);
                    }
                    local = local.min(line.len());
                    if let Some(preedit) = &st.preedit
                        && (row_preedit.is_some() || row_preedit_range.is_some())
                    {
                        local = local.saturating_add(preedit_cursor_offset_bytes(preedit));
                    }
                    let max_len = if let Some(preedit) = &st.preedit
                        && (row_preedit.is_some() || row_preedit_range.is_some())
                    {
                        if row_preedit.is_some() {
                            line.len().saturating_add(preedit.text.len())
                        } else {
                            line.len()
                        }
                    } else {
                        line.len()
                    };
                    local = local.min(max_len);

                    let (services, _) = painter.services_and_scene();
                    let started = perf_enabled.then(Instant::now);
                    let x0 = services.text().caret_x(blob, local);
                    if let Some(started) = started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_caret_x,
                            &mut st.paint_perf_frame.ns_caret_x,
                            started,
                        );
                    }

                    let (caret_top, caret_h) = if let (Some(top), Some(h)) =
                        (caret_rect_top, caret_rect_height)
                        && h.0 > 0.0
                    {
                        (top, Px(h.0.min(row_h.0)))
                    } else {
                        (Px(0.0), row_h)
                    };
                    Rect::new(
                        fret_core::Point::new(
                            Px(rect.origin.x.0 + x0.0),
                            Px(rect.origin.y.0 + caret_top.0),
                        ),
                        Size::new(Px(1.0), caret_h),
                    )
                } else {
                    let mut col = caret_pt.col;
                    if let Some(preedit) = &st.preedit {
                        col = col.saturating_add(preedit_cursor_offset_cols(preedit));
                    }
                    let x = Px(rect.origin.x.0 + col as f32 * cell_w.0);
                    Rect::new(
                        fret_core::Point::new(x, rect.origin.y),
                        Size::new(Px(1.0), row_h),
                    )
                };
                painter.scene().push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: caret_rect,
                    background: fret_core::Paint::Solid(caret_color).into(),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),

                    corner_radii: Corners::all(Px(0.0)),
                });
                if perf_enabled {
                    st.paint_perf_frame.quads_caret =
                        st.paint_perf_frame.quads_caret.saturating_add(1);
                }
            }
        }
    }

    #[cfg(feature = "syntax")]
    if let Some((row_scene_key, geom, is_rich, ops, syntax_replay_key)) = pending_row_scene_store {
        store_row_scene_cache(
            st,
            row,
            row_scene_key,
            origin,
            geom,
            is_rich,
            ops,
            syntax_replay_key,
            text_cache_max_entries,
        );
    }
    #[cfg(not(feature = "syntax"))]
    if let Some((row_scene_key, geom, is_rich, ops)) = pending_row_scene_store {
        store_row_scene_cache(
            st,
            row,
            row_scene_key,
            origin,
            geom,
            is_rich,
            ops,
            text_cache_max_entries,
        );
    }

    let row_geom_cache_started = perf_enabled.then(Instant::now);
    {
        // Cache row geometry for pointer hit-testing / IME cursor-area anchoring in event handlers.
        let rev = st.buffer.revision();
        let wrap_cols = st.display_wrap_cols;
        let folds_epoch = st.folds_epoch;
        let inlays_epoch = st.inlays_epoch;
        let display_map_epoch = st.display_map_epoch;
        if st.row_geom_cache_rev != rev
            || st.row_geom_cache_wrap_cols != wrap_cols
            || st.row_geom_cache_folds_epoch != folds_epoch
            || st.row_geom_cache_inlays_epoch != inlays_epoch
            || st.row_geom_cache_display_map_epoch != display_map_epoch
        {
            st.row_geom_cache_rev = rev;
            st.row_geom_cache_wrap_cols = wrap_cols;
            st.row_geom_cache_folds_epoch = folds_epoch;
            st.row_geom_cache_inlays_epoch = inlays_epoch;
            st.row_geom_cache_display_map_epoch = display_map_epoch;
            st.row_geom_cache_tick = 0;
            st.row_geom_cache.clear();
            st.row_geom_cache_queue.clear();
            st.row_geom_cache_caret_stops_len_total = 0;
        }

        st.row_geom_cache_tick = st.row_geom_cache_tick.saturating_add(1);
        let tick = st.row_geom_cache_tick;
        let has_row_geom = fresh_geom.is_some() || st.row_geom_cache.contains_key(&row);
        if has_row_geom {
            if let Some(geom) = fresh_geom {
                let caret_stops_len = geom.caret_stops.len() as u64;
                if let Some((old, _)) = st.row_geom_cache.insert(row, (geom, tick)) {
                    st.row_geom_cache_caret_stops_len_total = st
                        .row_geom_cache_caret_stops_len_total
                        .saturating_sub(old.caret_stops.len() as u64);
                }
                st.row_geom_cache_caret_stops_len_total = st
                    .row_geom_cache_caret_stops_len_total
                    .saturating_add(caret_stops_len);
            } else if let Some((_, last_used)) = st.row_geom_cache.get_mut(&row) {
                *last_used = tick;
            }

            st.row_geom_cache_queue.push_back((row, tick));
            compact_row_lru_queue_if_needed(
                &st.row_geom_cache,
                &mut st.row_geom_cache_queue,
                text_cache_max_entries,
            );
            while st.row_geom_cache.len() > text_cache_max_entries {
                let Some((victim, victim_tick)) = st.row_geom_cache_queue.pop_front() else {
                    break;
                };
                let remove = st
                    .row_geom_cache
                    .get(&victim)
                    .is_some_and(|(_, last_used)| *last_used == victim_tick);
                if remove {
                    if let Some((old, _)) = st.row_geom_cache.remove(&victim) {
                        st.row_geom_cache_caret_stops_len_total = st
                            .row_geom_cache_caret_stops_len_total
                            .saturating_sub(old.caret_stops.len() as u64);
                    }
                }
            }
        }
    }
    if let Some(started) = row_geom_cache_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_geom_cache,
            &mut st.paint_perf_frame.ns_row_geom_cache,
            started,
        );
    }

    if perf_enabled {
        st.paint_perf_frame.rows_drew_rich = st
            .paint_perf_frame
            .rows_drew_rich
            .saturating_add(drew_rich as u64);
        if let Some(row_started) = row_started {
            add_paint_perf_elapsed(
                &mut st.paint_perf_frame.us_total,
                &mut st.paint_perf_frame.ns_total,
                row_started,
            );
        }
    }
}

fn ensure_row_scene_cache_fresh(st: &mut CodeEditorState) {
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    let folds_epoch = st.folds_epoch;
    let inlays_epoch = st.inlays_epoch;
    let display_map_epoch = st.display_map_epoch;
    if st.row_scene_cache_rev != rev
        || st.row_scene_cache_wrap_cols != wrap_cols
        || st.row_scene_cache_folds_epoch != folds_epoch
        || st.row_scene_cache_inlays_epoch != inlays_epoch
        || st.row_scene_cache_display_map_epoch != display_map_epoch
    {
        st.invalidate_row_scene_cache();
    }
}

#[cfg(feature = "syntax")]
enum SyntaxRowCacheLookup {
    Hit(Arc<[SyntaxSpan]>),
    Miss { tick: u64 },
}

#[cfg(feature = "syntax")]
const SYNTAX_PREFETCH_CHUNK_ROWS: usize =
    SYNTAX_CACHE_LOOKBACK_ROWS + SYNTAX_CACHE_LOOKAHEAD_ROWS + 1;

#[cfg(feature = "syntax")]
const SYNTAX_PREFETCH_AHEAD_ROWS: usize = SYNTAX_PREFETCH_CHUNK_ROWS / 2;

#[cfg(feature = "syntax")]
const ROW_RICH_PREFETCH_EDGE_ROWS: usize = 8;

#[cfg(feature = "syntax")]
fn syntax_prefetch_chunk_for_row(row: usize, line_count: usize) -> Option<(usize, usize)> {
    if line_count == 0 {
        return None;
    }

    let row = row.min(line_count.saturating_sub(1));
    let chunk_start = (row / SYNTAX_PREFETCH_CHUNK_ROWS) * SYNTAX_PREFETCH_CHUNK_ROWS;
    let chunk_end = chunk_start
        .saturating_add(SYNTAX_PREFETCH_CHUNK_ROWS.saturating_sub(1))
        .min(line_count.saturating_sub(1));
    Some((chunk_start, chunk_end))
}

#[cfg(feature = "syntax")]
fn syntax_row_cache_chunk_is_ready(
    st: &CodeEditorState,
    chunk_start: usize,
    chunk_end: usize,
) -> bool {
    (chunk_start..=chunk_end).all(|row| st.syntax_row_cache.contains_key(&row))
}

#[cfg(feature = "syntax")]
fn push_unique_row(rows: &mut Vec<usize>, row: usize) {
    if !rows.contains(&row) {
        rows.push(row);
    }
}

#[cfg(feature = "syntax")]
fn arc_str_ptr_or_content_eq(a: &Arc<str>, b: &Arc<str>) -> bool {
    Arc::ptr_eq(a, b) || a.as_ref() == b.as_ref()
}

#[cfg(feature = "syntax")]
fn arc_slice_ptr_or_content_eq<T: PartialEq>(a: &Arc<[T]>, b: &Arc<[T]>) -> bool {
    Arc::ptr_eq(a, b) || a.as_ref() == b.as_ref()
}

#[cfg(feature = "syntax")]
fn row_rich_prefetch_candidate_rows(
    visible_start: usize,
    visible_end: usize,
    row_count: usize,
    direction: i8,
) -> Vec<usize> {
    if row_count == 0 {
        return Vec::new();
    }

    let last = row_count.saturating_sub(1);
    let visible_start = visible_start.min(last);
    let visible_end = visible_end.min(last);
    let mut rows = Vec::with_capacity(ROW_RICH_PREFETCH_EDGE_ROWS + 4);

    if direction < 0 {
        for delta in 1..=ROW_RICH_PREFETCH_EDGE_ROWS {
            let row = visible_start.saturating_sub(delta);
            push_unique_row(&mut rows, row);
            if row == 0 {
                break;
            }
        }
        push_unique_row(
            &mut rows,
            visible_start.saturating_sub(SYNTAX_PREFETCH_AHEAD_ROWS),
        );
        push_unique_row(&mut rows, visible_end.saturating_add(1).min(last));
    } else {
        for delta in 1..=ROW_RICH_PREFETCH_EDGE_ROWS {
            let row = visible_end.saturating_add(delta).min(last);
            push_unique_row(&mut rows, row);
            if row == last {
                break;
            }
        }
        push_unique_row(
            &mut rows,
            visible_end
                .saturating_add(SYNTAX_PREFETCH_AHEAD_ROWS)
                .min(last),
        );
        push_unique_row(&mut rows, visible_start.saturating_sub(1));
    }

    rows
}

#[cfg(feature = "syntax")]
fn syntax_row_cache_store_rows<I>(
    st: &mut CodeEditorState,
    rows: I,
    max_entries: usize,
    tick: u64,
) -> usize
where
    I: IntoIterator<Item = (usize, Arc<[SyntaxSpan]>)>,
{
    let mut stored_rows = 0usize;
    for (row, spans) in rows {
        let spans_len = spans.len() as u64;
        if let Some((old, _)) = st.syntax_row_cache.insert(row, (Arc::clone(&spans), tick)) {
            st.syntax_row_cache_spans_len_total = st
                .syntax_row_cache_spans_len_total
                .saturating_sub(old.len() as u64);
        }
        st.syntax_row_cache_spans_len_total = st
            .syntax_row_cache_spans_len_total
            .saturating_add(spans_len);
        st.syntax_row_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.syntax_row_cache,
            &mut st.syntax_row_cache_queue,
            max_entries,
        );
        stored_rows = stored_rows.saturating_add(1);

        while st.syntax_row_cache.len() > max_entries {
            let Some((victim, victim_tick)) = st.syntax_row_cache_queue.pop_front() else {
                break;
            };
            let remove = st
                .syntax_row_cache
                .get(&victim)
                .is_some_and(|(_, last_used)| *last_used == victim_tick);
            if remove {
                if let Some((old, _)) = st.syntax_row_cache.remove(&victim) {
                    st.syntax_row_cache_spans_len_total = st
                        .syntax_row_cache_spans_len_total
                        .saturating_sub(old.len() as u64);
                }
                st.cache_stats.syntax_evictions = st.cache_stats.syntax_evictions.saturating_add(1);
            }
        }
    }

    stored_rows
}

#[cfg(feature = "syntax")]
fn syntax_rows_from_highlight_spans(
    start_byte: usize,
    row_start: usize,
    row_ranges: &[Range<usize>],
    spans: Vec<fret_syntax::HighlightSpan>,
) -> Arc<[(usize, Arc<[SyntaxSpan]>)]> {
    let mut per_row = vec![Vec::<SyntaxSpan>::new(); row_ranges.len()];

    for span in spans {
        let Some(highlight) = span.highlight else {
            continue;
        };

        let global_start = start_byte.saturating_add(span.range.start);
        let global_end = start_byte.saturating_add(span.range.end);
        if global_start >= global_end {
            continue;
        }

        let mut row_idx = row_ranges
            .iter()
            .position(|row_range| row_range.end > global_start)
            .unwrap_or(row_ranges.len());
        while row_idx < row_ranges.len() {
            let row_range = &row_ranges[row_idx];
            if row_range.start >= global_end {
                break;
            }

            let inter_start = global_start.max(row_range.start);
            let inter_end = global_end.min(row_range.end);
            if inter_start < inter_end {
                per_row[row_idx].push(SyntaxSpan {
                    range: (inter_start - row_range.start)..(inter_end - row_range.start),
                    highlight,
                });
            }

            row_idx = row_idx.saturating_add(1);
        }
    }

    let mut rows = Vec::with_capacity(row_ranges.len());
    for (idx, mut spans) in per_row.into_iter().enumerate() {
        spans.sort_by(|a, b| {
            a.range
                .start
                .cmp(&b.range.start)
                .then(a.range.end.cmp(&b.range.end))
                .then(a.highlight.cmp(&b.highlight))
        });
        spans.dedup_by(|a, b| a.range == b.range && a.highlight == b.highlight);

        let mut merged: Vec<SyntaxSpan> = Vec::new();
        for span in spans {
            if let Some(last) = merged.last_mut()
                && last.highlight == span.highlight
                && last.range.end == span.range.start
            {
                last.range.end = span.range.end;
                continue;
            }
            merged.push(span);
        }

        rows.push((row_start + idx, Arc::from(merged)));
    }

    Arc::from(rows)
}

#[cfg(feature = "syntax")]
fn ensure_syntax_row_cache_fresh(st: &mut CodeEditorState) {
    let rev = st.buffer.revision();
    if st.syntax_row_cache_rev != rev || st.syntax_row_cache_language != st.language {
        st.syntax_row_cache_rev = rev;
        st.syntax_row_cache_language = st.language.clone();
        st.syntax_row_cache_tick = 0;
        st.syntax_row_cache.clear();
        st.syntax_row_cache_queue.clear();
        st.syntax_row_cache_spans_len_total = 0;
        st.cache_stats.syntax_resets = st.cache_stats.syntax_resets.saturating_add(1);
        st.row_rich_cache_tick = 0;
        st.row_rich_cache.clear();
        st.row_rich_cache_queue.clear();
        st.row_rich_cache_line_bytes_estimate_total = 0;
        st.row_rich_cache_row_spans_len_total = 0;
        st.row_rich_cache_syntax_spans_len_total = 0;
        st.row_rich_cache_rich_spans_len_total = 0;
        st.invalidate_row_scene_cache();
        st.cache_stats.row_rich_resets = st.cache_stats.row_rich_resets.saturating_add(1);
        st.clear_row_rich_prefetch_runtime();
        if let Some(runtime) = st.syntax_prefetch_runtime.as_ref() {
            runtime.clear();
        }
    }
}

#[cfg(feature = "syntax")]
fn lookup_row_syntax_spans(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> SyntaxRowCacheLookup {
    st.cache_stats.syntax_get_calls = st.cache_stats.syntax_get_calls.saturating_add(1);
    ensure_syntax_row_cache_fresh(st);

    st.syntax_row_cache_tick = st.syntax_row_cache_tick.saturating_add(1);
    let tick = st.syntax_row_cache_tick;

    if let Some((spans, last_used)) = st.syntax_row_cache.get_mut(&row) {
        *last_used = tick;
        let spans = Arc::clone(spans);
        st.syntax_row_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.syntax_row_cache,
            &mut st.syntax_row_cache_queue,
            max_entries,
        );
        st.cache_stats.syntax_hits = st.cache_stats.syntax_hits.saturating_add(1);
        return SyntaxRowCacheLookup::Hit(spans);
    }

    st.cache_stats.syntax_misses = st.cache_stats.syntax_misses.saturating_add(1);
    SyntaxRowCacheLookup::Miss { tick }
}

#[cfg(feature = "syntax")]
fn populate_row_syntax_spans_after_miss(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
    tick: u64,
) -> Arc<[SyntaxSpan]> {
    let language = st.language.clone();
    let Some(language) = language.as_deref() else {
        return Arc::<[SyntaxSpan]>::from([]);
    };

    let line_count = st.buffer.line_count();
    if line_count == 0 {
        return Arc::<[SyntaxSpan]>::from([]);
    }

    if st.syntax_prefetch_runtime.is_some() {
        let _ = tick;
        let _ = language;
        return Arc::<[SyntaxSpan]>::from([]);
    }

    let chunk_start = row.saturating_sub(SYNTAX_CACHE_LOOKBACK_ROWS);
    let chunk_end = row
        .saturating_add(SYNTAX_CACHE_LOOKAHEAD_ROWS)
        .min(line_count.saturating_sub(1));
    populate_syntax_row_cache_for_chunk(st, chunk_start, chunk_end, language, max_entries, tick);

    st.syntax_row_cache
        .get(&row)
        .map(|(spans, _)| Arc::clone(spans))
        .unwrap_or_else(|| Arc::<[SyntaxSpan]>::from([]))
}

#[cfg(feature = "syntax")]
#[allow(clippy::too_many_arguments)]
fn try_replay_row_scene_cache_fast_syntax(
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
    let mut probe_started = st.paint_perf_enabled.then(Instant::now);

    let replayed = {
        match st.row_scene_cache.get_mut(&row) {
            Some((cached, last_used))
                if cached.syntax_replay_key.as_ref().is_some_and(|key| {
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
                }) =>
            {
                *last_used = tick;
                if let Some(started) = probe_started.take() {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_fast_probe,
                        &mut st.paint_perf_frame.ns_row_scene_fast_probe,
                        started,
                    );
                }
                let replay_delta = fret_core::Point::new(
                    Px(origin.x.0 - cached.origin.x.0),
                    Px(origin.y.0 - cached.origin.y.0),
                );
                let touch_started = st.paint_perf_enabled.then(Instant::now);
                painter.touch_hosted_resources_in_scene_ops(cached.ops.as_slice());
                if let Some(started) = touch_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_replay_touch,
                        &mut st.paint_perf_frame.ns_row_scene_replay_touch,
                        started,
                    );
                }
                let replay_started = st.paint_perf_enabled.then(Instant::now);
                painter
                    .scene()
                    .replay_ops_translated(cached.ops.as_slice(), replay_delta);
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
            }
            Some(_) | None => {
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

    if let Some(out) = replayed {
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
    }
}

fn try_replay_row_scene_cache(
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
    let mut probe_started = st.paint_perf_enabled.then(Instant::now);

    let replayed = {
        match st.row_scene_cache.get_mut(&row) {
            Some((cached, last_used)) if cached.key == *key => {
                *last_used = tick;
                if let Some(started) = probe_started.take() {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_full_probe,
                        &mut st.paint_perf_frame.ns_row_scene_full_probe,
                        started,
                    );
                }
                let replay_delta = fret_core::Point::new(
                    Px(origin.x.0 - cached.origin.x.0),
                    Px(origin.y.0 - cached.origin.y.0),
                );
                let touch_started = st.paint_perf_enabled.then(Instant::now);
                painter.touch_hosted_resources_in_scene_ops(cached.ops.as_slice());
                if let Some(started) = touch_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_replay_touch,
                        &mut st.paint_perf_frame.ns_row_scene_replay_touch,
                        started,
                    );
                }
                let replay_started = st.paint_perf_enabled.then(Instant::now);
                painter
                    .scene()
                    .replay_ops_translated(cached.ops.as_slice(), replay_delta);
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
            }
            Some(_) | None => {
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

    if let Some(out) = replayed {
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
    }
}

#[cfg(feature = "syntax")]
fn store_row_scene_cache(
    st: &mut CodeEditorState,
    row: usize,
    key: RowSceneKey,
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

    if let Some((old, _)) = st.row_scene_cache.insert(
        row,
        (
            RowSceneCacheEntry {
                key,
                origin,
                geom,
                is_rich,
                ops,
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
    }
}

#[cfg(not(feature = "syntax"))]
fn store_row_scene_cache(
    st: &mut CodeEditorState,
    row: usize,
    key: RowSceneKey,
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

    if let Some((old, _)) = st.row_scene_cache.insert(
        row,
        (
            RowSceneCacheEntry {
                key,
                origin,
                geom,
                is_rich,
                ops,
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
    }
}

#[cfg(test)]
pub(super) fn cached_row_text(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> Arc<str> {
    cached_row_text_with_range(st, row, max_entries).1
}

pub(super) fn cached_row_text_with_range(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> (
    Range<usize>,
    Arc<str>,
    Option<super::geom::RowFoldMap>,
    Option<Range<usize>>,
    Arc<[fret_code_editor_view::DisplayRowSpan]>,
) {
    st.cache_stats.row_text_get_calls = st.cache_stats.row_text_get_calls.saturating_add(1);
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    let folds_epoch = st.folds_epoch;
    let inlays_epoch = st.inlays_epoch;
    let display_map_epoch = st.display_map_epoch;
    if st.row_text_cache_rev != rev
        || st.row_text_cache_wrap_cols != wrap_cols
        || st.row_text_cache_folds_epoch != folds_epoch
        || st.row_text_cache_inlays_epoch != inlays_epoch
        || st.row_text_cache_display_map_epoch != display_map_epoch
    {
        st.row_text_cache_rev = rev;
        st.row_text_cache_wrap_cols = wrap_cols;
        st.row_text_cache_folds_epoch = folds_epoch;
        st.row_text_cache_inlays_epoch = inlays_epoch;
        st.row_text_cache_display_map_epoch = display_map_epoch;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.row_text_cache_text_bytes_estimate_total = 0;
        st.row_text_cache_row_spans_len_total = 0;
        st.cache_stats.row_text_resets = st.cache_stats.row_text_resets.saturating_add(1);
        #[cfg(feature = "syntax")]
        {
            st.clear_row_rich_prefetch_runtime();
            st.row_rich_cache_tick = 0;
            st.row_rich_cache.clear();
            st.row_rich_cache_queue.clear();
            st.row_rich_cache_line_bytes_estimate_total = 0;
            st.row_rich_cache_row_spans_len_total = 0;
            st.row_rich_cache_syntax_spans_len_total = 0;
            st.row_rich_cache_rich_spans_len_total = 0;
            st.cache_stats.row_rich_resets = st.cache_stats.row_rich_resets.saturating_add(1);
        }
    }

    st.row_text_cache_tick = st.row_text_cache_tick.saturating_add(1);
    let tick = st.row_text_cache_tick;

    if let Some((text, last_used)) = st.row_text_cache.get_mut(&row) {
        *last_used = tick;
        let out = (
            text.range.clone(),
            Arc::clone(&text.text),
            text.fold_map.clone(),
            text.preedit_range.clone(),
            Arc::clone(&text.row_spans),
        );
        st.row_text_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.row_text_cache,
            &mut st.row_text_cache_queue,
            max_entries,
        );
        st.cache_stats.row_text_hits = st.cache_stats.row_text_hits.saturating_add(1);
        return out;
    }
    st.cache_stats.row_text_misses = st.cache_stats.row_text_misses.saturating_add(1);

    let materialized = st.display_map.materialize_display_row_text(&st.buffer, row);
    let range = materialized.row_range.clone();
    let range_for_return = range.clone();
    let preedit_range = materialized.preedit_range.clone();

    let row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]> = Arc::from(materialized.spans);
    let spans: Vec<super::geom::RowFoldSpan> = row_spans
        .iter()
        .map(|span| super::geom::RowFoldSpan {
            buffer_range: span.buffer_range.clone(),
            display_range: span.display_range.clone(),
        })
        .collect();
    let fold_map = (!spans.is_empty()).then_some(super::geom::RowFoldMap::new(spans));
    let text = materialized.text;

    let entry_text_bytes = text.len() as u64;
    let entry_row_spans_len = row_spans.len() as u64;

    if let Some((old, _)) = st.row_text_cache.insert(
        row,
        (
            RowTextCacheEntry {
                text: Arc::clone(&text),
                range,
                fold_map: fold_map.clone(),
                preedit_range: preedit_range.clone(),
                row_spans: Arc::clone(&row_spans),
            },
            tick,
        ),
    ) {
        st.row_text_cache_text_bytes_estimate_total = st
            .row_text_cache_text_bytes_estimate_total
            .saturating_sub(old.text.len() as u64);
        st.row_text_cache_row_spans_len_total = st
            .row_text_cache_row_spans_len_total
            .saturating_sub(old.row_spans.len() as u64);
    }
    st.row_text_cache_text_bytes_estimate_total = st
        .row_text_cache_text_bytes_estimate_total
        .saturating_add(entry_text_bytes);
    st.row_text_cache_row_spans_len_total = st
        .row_text_cache_row_spans_len_total
        .saturating_add(entry_row_spans_len);
    st.row_text_cache_queue.push_back((row, tick));
    compact_row_lru_queue_if_needed(
        &st.row_text_cache,
        &mut st.row_text_cache_queue,
        max_entries,
    );

    while st.row_text_cache.len() > max_entries {
        let Some((victim, victim_tick)) = st.row_text_cache_queue.pop_front() else {
            break;
        };
        let remove = st
            .row_text_cache
            .get(&victim)
            .is_some_and(|(_, last_used)| *last_used == victim_tick);
        if remove {
            if let Some((old, _)) = st.row_text_cache.remove(&victim) {
                st.row_text_cache_text_bytes_estimate_total = st
                    .row_text_cache_text_bytes_estimate_total
                    .saturating_sub(old.text.len() as u64);
                st.row_text_cache_row_spans_len_total = st
                    .row_text_cache_row_spans_len_total
                    .saturating_sub(old.row_spans.len() as u64);
            }
            st.cache_stats.row_text_evictions = st.cache_stats.row_text_evictions.saturating_add(1);
        }
    }

    (range_for_return, text, fold_map, preedit_range, row_spans)
}

pub(super) fn materialize_preedit_rich_text(
    line: Arc<str>,
    caret_in_line: usize,
    code_shaping: &fret_core::TextShapingStyle,
    preedit: &PreeditState,
    fg: Color,
    selection_bg: Color,
) -> AttributedText {
    let caret_in_line = caret_in_line.min(line.len());
    let before = line.get(..caret_in_line).unwrap_or("");
    let after = line.get(caret_in_line..).unwrap_or("");

    let mut display = String::with_capacity(before.len() + preedit.text.len() + after.len());
    display.push_str(before);
    display.push_str(preedit.text.as_str());
    display.push_str(after);

    let before_len = before.len();
    let preedit_len = preedit.text.len();
    let after_len = after.len();

    let underline = UnderlineStyle {
        color: Some(fg),
        style: DecorationLineStyle::Solid,
    };

    let cursor_range = preedit.cursor.and_then(|(a, b)| {
        let a = fret_code_editor_view::clamp_to_char_boundary(preedit.text.as_str(), a)
            .min(preedit.text.len());
        let b = fret_code_editor_view::clamp_to_char_boundary(preedit.text.as_str(), b)
            .min(preedit.text.len());
        if a == b {
            return None;
        }
        Some(if a <= b { a..b } else { b..a })
    });

    let mut spans: Vec<TextSpan> = Vec::new();
    if before_len > 0 {
        spans.push(TextSpan::new(before_len));
    }

    if let Some(cursor) = cursor_range {
        let pre_a = cursor.start.min(preedit_len);
        let pre_b = cursor.end.min(preedit_len);
        if pre_a > 0 {
            spans.push(TextSpan {
                len: pre_a,
                shaping: Default::default(),
                paint: TextPaintStyle {
                    underline: Some(underline.clone()),
                    ..Default::default()
                },
            });
        }
        spans.push(TextSpan {
            len: pre_b.saturating_sub(pre_a),
            shaping: Default::default(),
            paint: TextPaintStyle {
                bg: Some(selection_bg),
                underline: Some(underline.clone()),
                ..Default::default()
            },
        });
        if pre_b < preedit_len {
            spans.push(TextSpan {
                len: preedit_len - pre_b,
                shaping: Default::default(),
                paint: TextPaintStyle {
                    underline: Some(underline),
                    ..Default::default()
                },
            });
        }
    } else {
        spans.push(TextSpan {
            len: preedit_len,
            shaping: Default::default(),
            paint: TextPaintStyle {
                underline: Some(underline),
                ..Default::default()
            },
        });
    }

    if after_len > 0 {
        spans.push(TextSpan::new(after_len));
    }

    if *code_shaping != Default::default() {
        for span in &mut spans {
            span.shaping = code_shaping.clone();
        }
    }

    AttributedText::new(display, spans)
}

pub(super) fn materialize_preedit_rich_text_for_range(
    line: Arc<str>,
    preedit_range: Range<usize>,
    code_shaping: &fret_core::TextShapingStyle,
    preedit: &PreeditState,
    fg: Color,
    selection_bg: Color,
) -> AttributedText {
    let start = preedit_range.start.min(line.len());
    let end = preedit_range.end.min(line.len()).max(start);

    let display = line.as_ref().to_string();

    let before_len = start;
    let preedit_len = end.saturating_sub(start);
    let after_len = display.len().saturating_sub(end);

    let underline = UnderlineStyle {
        color: Some(fg),
        style: DecorationLineStyle::Solid,
    };

    let cursor_range = preedit.cursor.and_then(|(a, b)| {
        let a = fret_code_editor_view::clamp_to_char_boundary(preedit.text.as_str(), a)
            .min(preedit.text.len());
        let b = fret_code_editor_view::clamp_to_char_boundary(preedit.text.as_str(), b)
            .min(preedit.text.len());
        if a == b {
            return None;
        }
        Some(if a <= b { a..b } else { b..a })
    });

    let mut spans: Vec<TextSpan> = Vec::new();
    if before_len > 0 {
        spans.push(TextSpan::new(before_len));
    }

    if preedit_len > 0 {
        if let Some(cursor) = cursor_range {
            let pre_a = cursor.start.min(preedit_len);
            let pre_b = cursor.end.min(preedit_len);
            if pre_a > 0 {
                spans.push(TextSpan {
                    len: pre_a,
                    shaping: Default::default(),
                    paint: TextPaintStyle {
                        underline: Some(underline.clone()),
                        ..Default::default()
                    },
                });
            }
            spans.push(TextSpan {
                len: pre_b.saturating_sub(pre_a),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    bg: Some(selection_bg),
                    underline: Some(underline.clone()),
                    ..Default::default()
                },
            });
            if pre_b < preedit_len {
                spans.push(TextSpan {
                    len: preedit_len - pre_b,
                    shaping: Default::default(),
                    paint: TextPaintStyle {
                        underline: Some(underline),
                        ..Default::default()
                    },
                });
            }
        } else {
            spans.push(TextSpan {
                len: preedit_len,
                shaping: Default::default(),
                paint: TextPaintStyle {
                    underline: Some(underline),
                    ..Default::default()
                },
            });
        }
    }

    if after_len > 0 {
        spans.push(TextSpan::new(after_len));
    }

    if *code_shaping != Default::default() {
        for span in &mut spans {
            span.shaping = code_shaping.clone();
        }
    }

    AttributedText::new(display, spans)
}

#[cfg(feature = "syntax")]
const SYNTAX_CACHE_LOOKBACK_ROWS: usize = 64;

#[cfg(feature = "syntax")]
const SYNTAX_CACHE_LOOKAHEAD_ROWS: usize = 64;

#[cfg(feature = "syntax")]
pub(super) fn invalidate_syntax_row_cache_for_delta(
    st: &mut CodeEditorState,
    delta: fret_code_editor_buffer::BufferDelta,
) {
    // Keep the revision in sync so cached-row requests don't force a full cache clear.
    st.syntax_row_cache_rev = delta.after;
    if st.syntax_row_cache.is_empty() {
        return;
    }

    let line_count = st.buffer.line_count().max(1);
    let max_line = line_count.saturating_sub(1);

    let old_edit_start = delta.lines.start;
    let new_edit_start = delta.lines.start.min(max_line);
    let old_count = delta.lines.old_count.max(1);
    let new_count = delta.lines.new_count.max(1);
    let old_end_excl = old_edit_start.saturating_add(old_count);

    let invalidation_start = new_edit_start.saturating_sub(SYNTAX_CACHE_LOOKBACK_ROWS);
    let new_span_end = new_edit_start
        .saturating_add(new_count.saturating_sub(1))
        .min(max_line);
    let invalidation_end = new_span_end
        .saturating_add(SYNTAX_CACHE_LOOKAHEAD_ROWS)
        .min(max_line);

    let shift: isize = new_count as isize - old_count as isize;
    let shift_row = |row: usize| -> usize {
        if shift >= 0 {
            row.saturating_add(shift as usize)
        } else {
            row.saturating_sub(shift.unsigned_abs())
        }
    };

    let before_len = st.syntax_row_cache.len();
    let prev = std::mem::take(&mut st.syntax_row_cache);
    let mut next = HashMap::with_capacity(prev.len());

    for (row, (spans, tick)) in prev {
        // Always invalidate the edited line span in the old coordinate space.
        if row >= old_edit_start && row < old_end_excl {
            continue;
        }

        let mapped = if row >= old_end_excl {
            shift_row(row)
        } else {
            row
        };
        if mapped >= line_count {
            continue;
        }

        // Invalidate a bounded lookback/lookahead window in the new coordinate space.
        if mapped >= invalidation_start && mapped <= invalidation_end {
            continue;
        }

        next.insert(mapped, (spans, tick));
    }

    st.syntax_row_cache = next;
    let after_len = st.syntax_row_cache.len();
    let removed = before_len.saturating_sub(after_len);
    if removed > 0 {
        st.cache_stats.syntax_evictions = st
            .cache_stats
            .syntax_evictions
            .saturating_add(removed as u64);
    }
    rebuild_syntax_row_cache_queue(st);
}

#[cfg(feature = "syntax")]
pub(super) fn rebuild_syntax_row_cache_queue(st: &mut CodeEditorState) {
    let mut entries: Vec<(usize, u64)> = st
        .syntax_row_cache
        .iter()
        .map(|(row, (_, tick))| (*row, *tick))
        .collect();
    entries.sort_by_key(|(_, tick)| *tick);
    st.syntax_row_cache_queue = entries.into();
}

#[cfg(feature = "syntax")]
#[allow(dead_code)]
pub(super) fn cached_row_syntax_spans(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> Arc<[SyntaxSpan]> {
    match lookup_row_syntax_spans(st, row, max_entries) {
        SyntaxRowCacheLookup::Hit(spans) => spans,
        SyntaxRowCacheLookup::Miss { tick } => {
            populate_row_syntax_spans_after_miss(st, row, max_entries, tick)
        }
    }
}

#[cfg(feature = "syntax")]
pub(super) fn populate_syntax_row_cache_for_chunk(
    st: &mut CodeEditorState,
    chunk_start: usize,
    chunk_end: usize,
    language: &str,
    max_entries: usize,
    tick: u64,
) {
    let line_count = st.buffer.line_count();
    if line_count == 0 || chunk_start > chunk_end {
        return;
    }

    let start_byte = st
        .buffer
        .line_start(chunk_start)
        .unwrap_or(0)
        .min(st.buffer.len_bytes());
    let end_byte = if chunk_end.saturating_add(1) < line_count {
        st.buffer
            .line_start(chunk_end.saturating_add(1))
            .unwrap_or(st.buffer.len_bytes())
            .min(st.buffer.len_bytes())
    } else {
        st.buffer.len_bytes()
    };

    if start_byte >= end_byte {
        return;
    }

    let slice_started = st.paint_perf_enabled.then(Instant::now);
    let Some(slice) = st.buffer.slice_to_string(start_byte..end_byte) else {
        return;
    };
    if let Some(started) = slice_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_syntax_slice,
            &mut st.paint_perf_frame.ns_syntax_slice,
            started,
        );
    }

    let highlight_started = st.paint_perf_enabled.then(Instant::now);
    let spans = fret_syntax::highlight(slice.as_str(), language).unwrap_or_default();
    if let Some(started) = highlight_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_syntax_highlight,
            &mut st.paint_perf_frame.ns_syntax_highlight,
            started,
        );
    }

    let distribute_started = st.paint_perf_enabled.then(Instant::now);
    let mut row_ranges = Vec::with_capacity(chunk_end - chunk_start + 1);
    for row in chunk_start..=chunk_end {
        row_ranges.push(st.buffer.line_byte_range(row).unwrap_or(0..0));
    }

    let rows = syntax_rows_from_highlight_spans(start_byte, chunk_start, &row_ranges, spans);
    if let Some(started) = distribute_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_syntax_distribute,
            &mut st.paint_perf_frame.ns_syntax_distribute,
            started,
        );
    }

    let store_started = st.paint_perf_enabled.then(Instant::now);
    let stored_rows = syntax_row_cache_store_rows(st, rows.iter().cloned(), max_entries, tick);
    st.paint_perf_frame.syntax_rows_stored = st
        .paint_perf_frame
        .syntax_rows_stored
        .saturating_add(stored_rows as u64);
    if let Some(started) = store_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_syntax_store,
            &mut st.paint_perf_frame.ns_syntax_store,
            started,
        );
    }
}

#[cfg(feature = "syntax")]
fn drain_syntax_prefetch_ready(st: &mut CodeEditorState, max_entries: usize) {
    let Some(runtime) = st.syntax_prefetch_runtime.as_ref().cloned() else {
        return;
    };

    let mut drained = runtime.drain_ready();
    if drained.is_empty() {
        return;
    }

    ensure_syntax_row_cache_fresh(st);
    let Some(language) = st.language.as_ref().cloned() else {
        return;
    };
    let doc = st.buffer.doc();
    let rev = st.buffer.revision();
    let line_count = st.buffer.line_count();

    for chunk in drained.drain(..) {
        if chunk.key.doc != doc
            || chunk.key.rev != rev
            || chunk.key.language.as_ref() != language.as_ref()
            || chunk.key.chunk_start >= line_count
            || chunk.key.chunk_end >= line_count
        {
            continue;
        }

        st.syntax_row_cache_tick = st.syntax_row_cache_tick.saturating_add(1);
        let tick = st.syntax_row_cache_tick;
        let store_started = st.paint_perf_enabled.then(Instant::now);
        let stored_rows =
            syntax_row_cache_store_rows(st, chunk.rows.iter().cloned(), max_entries, tick);
        if let Some(started) = store_started {
            add_paint_perf_elapsed(
                &mut st.paint_perf_frame.us_syntax_store,
                &mut st.paint_perf_frame.ns_syntax_store,
                started,
            );
        }
        st.paint_perf_frame.syntax_rows_stored = st
            .paint_perf_frame
            .syntax_rows_stored
            .saturating_add(stored_rows as u64);
    }
}

#[cfg(feature = "syntax")]
pub(super) fn schedule_syntax_prefetch_for_frame(
    st: &mut CodeEditorState,
    frame: WindowedRowsPaintFrame,
    max_entries: usize,
    window: fret_core::AppWindowId,
) {
    drain_syntax_prefetch_ready(st, max_entries);

    let Some(runtime) = st.syntax_prefetch_runtime.as_ref().cloned() else {
        return;
    };
    let Some(language) = st.language.as_ref().cloned() else {
        return;
    };

    ensure_syntax_row_cache_fresh(st);

    let line_count = st.buffer.line_count();
    if line_count == 0 {
        return;
    }

    let visible_start = frame.visible_start.min(line_count.saturating_sub(1));
    let visible_end = frame.visible_end.min(line_count.saturating_sub(1));
    let direction = runtime.note_visible_start(visible_start);
    let mut candidate_rows = vec![visible_start, visible_end];
    let lookahead_row = if direction < 0 {
        visible_start.saturating_sub(SYNTAX_PREFETCH_AHEAD_ROWS)
    } else {
        visible_end
            .saturating_add(SYNTAX_PREFETCH_AHEAD_ROWS)
            .min(line_count.saturating_sub(1))
    };
    candidate_rows.push(lookahead_row);

    let doc = st.buffer.doc();
    let rev = st.buffer.revision();
    let mut seen = std::collections::HashSet::<(usize, usize)>::new();

    for row in candidate_rows {
        let Some((chunk_start, chunk_end)) = syntax_prefetch_chunk_for_row(row, line_count) else {
            continue;
        };
        if !seen.insert((chunk_start, chunk_end)) {
            continue;
        }
        if syntax_row_cache_chunk_is_ready(st, chunk_start, chunk_end) {
            continue;
        }

        let key = SyntaxPrefetchKey {
            doc,
            rev,
            language: language.clone(),
            chunk_start,
            chunk_end,
        };
        if !runtime.try_mark_pending(key.clone()) {
            continue;
        }

        let start_byte = st
            .buffer
            .line_start(chunk_start)
            .unwrap_or(0)
            .min(st.buffer.len_bytes());
        let end_byte = if chunk_end.saturating_add(1) < line_count {
            st.buffer
                .line_start(chunk_end.saturating_add(1))
                .unwrap_or(st.buffer.len_bytes())
                .min(st.buffer.len_bytes())
        } else {
            st.buffer.len_bytes()
        };
        let row_ranges = (chunk_start..=chunk_end)
            .map(|row| st.buffer.line_byte_range(row).unwrap_or(0..0))
            .collect::<Vec<_>>();

        let shared = runtime.shared.clone();
        let dispatcher = runtime.dispatcher.clone();
        let wake_dispatcher = dispatcher.clone();
        let job_language = language.clone();
        let slice = if start_byte < end_byte {
            st.buffer
                .slice_to_string(start_byte..end_byte)
                .unwrap_or_default()
        } else {
            String::new()
        };

        dispatcher.dispatch_background(
            Box::new(move || {
                let spans = fret_syntax::highlight(slice.as_str(), job_language.as_ref())
                    .unwrap_or_default();
                let rows =
                    syntax_rows_from_highlight_spans(start_byte, chunk_start, &row_ranges, spans);
                let mut state = shared
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let _ = state.pending.remove(&key);
                state.ready.push_back(SyntaxPrefetchChunk { key, rows });
                while state.ready.len() > 8 {
                    let _ = state.ready.pop_front();
                }
                drop(state);
                wake_dispatcher.wake(Some(window));
            }),
            fret_runtime::DispatchPriority::Low,
        );
    }
}

#[cfg(feature = "syntax")]
fn drain_row_rich_prefetch_ready(
    st: &mut CodeEditorState,
    max_entries: usize,
    theme_revision: u64,
) {
    let Some(runtime) = st.row_rich_prefetch_runtime.as_ref().cloned() else {
        return;
    };

    let mut drained = runtime.drain_ready();
    if drained.is_empty() {
        return;
    }

    ensure_syntax_row_cache_fresh(st);
    let Some(language) = st.language.as_ref().cloned() else {
        return;
    };
    let doc = st.buffer.doc();
    let rev = st.buffer.revision();
    let rich_cache_max_entries = max_entries.min(2048);

    for chunk in drained.drain(..) {
        let key = &chunk.key;
        if key.doc != doc
            || key.rev != rev
            || key.language.as_ref() != language.as_ref()
            || key.theme_revision != theme_revision
            || key.code_font_feature_policy_rev != st.code_font_feature_policy_rev
            || key.row >= st.display_map.row_count()
        {
            continue;
        }

        let current = {
            let Some((row_text, _)) = st.row_text_cache.get(&key.row) else {
                continue;
            };
            let line_idx = st.display_map.display_row_line(key.row);
            let Some((syntax_spans, _)) = st.syntax_row_cache.get(&line_idx) else {
                continue;
            };
            (
                row_text.range.clone(),
                Arc::clone(&row_text.text),
                Arc::clone(syntax_spans),
                Arc::clone(&row_text.row_spans),
            )
        };
        let (row_range, line, syntax_spans, row_spans) = current;
        if key.row_range != row_range
            || !arc_str_ptr_or_content_eq(&key.line, &line)
            || !arc_slice_ptr_or_content_eq(&key.syntax_spans, &syntax_spans)
            || !arc_slice_ptr_or_content_eq(&key.row_spans, &row_spans)
        {
            continue;
        }

        st.row_rich_cache_tick = st.row_rich_cache_tick.saturating_add(1);
        let tick = st.row_rich_cache_tick;
        store_row_rich_cache_entry(
            st,
            key.row,
            row_range,
            line,
            syntax_spans,
            row_spans,
            key.theme_revision,
            key.code_font_feature_policy_rev,
            chunk.rich,
            rich_cache_max_entries,
            tick,
        );
    }
}

#[cfg(feature = "syntax")]
pub(super) fn schedule_row_rich_prefetch_for_frame(
    st: &mut CodeEditorState,
    frame: WindowedRowsPaintFrame,
    max_entries: usize,
    window: fret_core::AppWindowId,
    theme: fret_ui::Theme,
) {
    let theme_revision = theme.revision();
    drain_row_rich_prefetch_ready(st, max_entries, theme_revision);

    let Some(runtime) = st.row_rich_prefetch_runtime.as_ref().cloned() else {
        return;
    };
    let Some(language) = st.language.as_ref().cloned() else {
        return;
    };

    ensure_syntax_row_cache_fresh(st);

    let row_count = st.display_map.row_count();
    if row_count == 0 {
        return;
    }

    let visible_start = frame.visible_start.min(row_count.saturating_sub(1));
    let visible_end = frame.visible_end.min(row_count.saturating_sub(1));
    let direction = runtime.note_visible_start(visible_start);
    let candidate_rows =
        row_rich_prefetch_candidate_rows(visible_start, visible_end, row_count, direction);

    let doc = st.buffer.doc();
    let rev = st.buffer.revision();
    let code_font_feature_policy_rev = st.code_font_feature_policy_rev;
    let code_shaping = st.code_font_shaping_style.clone();

    for row in candidate_rows {
        let line_idx = st.display_map.display_row_line(row);
        let Some((syntax_spans, _)) = st.syntax_row_cache.get(&line_idx) else {
            continue;
        };
        if syntax_spans.is_empty() {
            continue;
        }
        let syntax_spans = Arc::clone(syntax_spans);

        let (row_range, line, _, _, row_spans) = cached_row_text_with_range(st, row, max_entries);
        let already_cached = st.row_rich_cache.get(&row).is_some_and(|(cached, _)| {
            cached.theme_revision == theme_revision
                && cached.row_range == row_range
                && cached.code_font_feature_policy_rev == code_font_feature_policy_rev
                && arc_str_ptr_or_content_eq(&cached.line, &line)
                && arc_slice_ptr_or_content_eq(&cached.syntax_spans, &syntax_spans)
                && arc_slice_ptr_or_content_eq(&cached.row_spans, &row_spans)
        });
        if already_cached {
            continue;
        }

        let key = RowRichPrefetchKey::new(
            doc,
            rev,
            Arc::clone(&language),
            row,
            row_range.clone(),
            theme_revision,
            code_font_feature_policy_rev,
            Arc::clone(&line),
            Arc::clone(&syntax_spans),
            Arc::clone(&row_spans),
        );
        if !runtime.try_mark_pending(key.clone()) {
            continue;
        }

        let line_start = st.buffer.line_start(line_idx).unwrap_or(row_range.start);
        let seg_start_in_line = row_range.start.saturating_sub(line_start);
        let base_len = row_range.end.saturating_sub(row_range.start);
        let shared = runtime.shared.clone();
        let dispatcher = runtime.dispatcher.clone();
        let wake_dispatcher = dispatcher.clone();
        let job_theme = theme.clone();
        let job_code_shaping = code_shaping.clone();
        let job_key = key;

        dispatcher.dispatch_background(
            Box::new(move || {
                let mapped = mapped_row_syntax_spans_for_rich_text(
                    job_key.line.as_ref(),
                    seg_start_in_line,
                    base_len,
                    job_key.row_spans.as_ref(),
                    job_key.syntax_spans.as_ref(),
                );
                let rich = mapped.map(|mapped| {
                    materialize_row_rich_text_with_fg(
                        Arc::clone(&job_key.line),
                        mapped.as_ref(),
                        &job_code_shaping,
                        |highlight| job_theme.syntax_color(highlight),
                    )
                });

                let mut state = shared
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                state.pending.retain(|pending| !pending.matches(&job_key));
                let should_wake = if let Some(rich) = rich {
                    state
                        .ready
                        .push_back(RowRichPrefetchChunk { key: job_key, rich });
                    while state.ready.len() > 32 {
                        let _ = state.ready.pop_front();
                    }
                    true
                } else {
                    false
                };
                drop(state);
                if should_wake {
                    wake_dispatcher.wake(Some(window));
                }
            }),
            fret_runtime::DispatchPriority::Low,
        );
    }
}

#[cfg(feature = "syntax")]
pub(super) fn syntax_color(theme: &fret_ui::Theme, highlight: &str) -> Option<Color> {
    theme.syntax_color(highlight)
}

#[cfg(feature = "syntax")]
fn materialize_row_rich_text_with_fg(
    line: Arc<str>,
    spans: &[SyntaxSpan],
    code_shaping: &fret_core::TextShapingStyle,
    mut fg_for_highlight: impl FnMut(&str) -> Option<Color>,
) -> AttributedText {
    let mut out: Vec<TextSpan> = Vec::new();
    let mut cursor = 0usize;
    let max = line.len();

    for span in spans {
        let start = span.range.start.min(max);
        let end = span.range.end.min(max);
        if start >= end || start < cursor {
            continue;
        }

        if start > cursor {
            out.push(TextSpan {
                len: start - cursor,
                shaping: code_shaping.clone(),
                ..Default::default()
            });
        }

        let fg = fg_for_highlight(span.highlight);
        out.push(TextSpan {
            len: end - start,
            shaping: code_shaping.clone(),
            paint: TextPaintStyle {
                fg,
                ..Default::default()
            },
        });
        cursor = end;
    }

    if cursor < max {
        out.push(TextSpan {
            len: max - cursor,
            shaping: code_shaping.clone(),
            ..Default::default()
        });
    }

    AttributedText::new(line, out)
}

#[cfg(feature = "syntax")]
pub(super) fn materialize_row_rich_text(
    theme: &fret_ui::Theme,
    line: Arc<str>,
    spans: &[SyntaxSpan],
    code_shaping: &fret_core::TextShapingStyle,
) -> AttributedText {
    materialize_row_rich_text_with_fg(line, spans, code_shaping, |highlight| {
        syntax_color(theme, highlight)
    })
}

#[cfg(all(test, feature = "syntax"))]
mod tests {
    use super::*;

    #[test]
    fn materialize_row_rich_text_applies_code_shaping_to_all_spans() {
        let line: Arc<str> = Arc::<str>::from("abc");
        let spans = vec![
            SyntaxSpan {
                range: 0..1,
                highlight: "keyword",
            },
            SyntaxSpan {
                range: 1..3,
                highlight: "string",
            },
        ];
        let shaping = fret_core::TextShapingStyle::default()
            .with_feature("liga", 0)
            .with_feature("calt", 0);

        let rich = materialize_row_rich_text_with_fg(line, &spans, &shaping, |_| None);
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
    fn syntax_replay_key_matches_current_inputs_by_pointer_identity() {
        let line: Arc<str> = Arc::<str>::from("abc");
        let row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]> = Arc::from(Vec::new());
        let syntax_spans: Arc<[SyntaxSpan]> = Arc::from(vec![SyntaxSpan {
            range: 0..3,
            highlight: "keyword",
        }]);
        let style = TextStyle::default();
        let constraints = CanvasTextConstraints {
            max_width: Some(Px(120.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        };
        let font_stack_key = fret_runtime::TextFontStackKey(7);
        let key = RowSceneSyntaxReplayKey::new(
            0..3,
            Arc::clone(&line),
            Arc::clone(&row_spans),
            Arc::clone(&syntax_spans),
            &style,
            constraints,
            font_stack_key,
            1.0,
            9,
            4,
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );

        assert!(key.matches_current(
            &(0..3),
            &line,
            &row_spans,
            &syntax_spans,
            &style,
            constraints,
            font_stack_key,
            1.0,
            9,
            4,
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        ));

        let other_row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]> = Arc::from(Vec::new());
        assert!(!key.matches_current(
            &(0..3),
            &line,
            &other_row_spans,
            &syntax_spans,
            &style,
            constraints,
            font_stack_key,
            1.0,
            9,
            4,
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        ));
    }

    #[test]
    fn normalize_syntax_spans_clamps_and_is_deterministic_for_stale_inputs() {
        let text = "a😀b";
        let mut spans = vec![
            SyntaxSpan {
                range: 999..1000,
                highlight: "keyword",
            },
            SyntaxSpan {
                // Inside the emoji's UTF-8 bytes.
                range: 2..4,
                highlight: "string",
            },
            SyntaxSpan {
                // Overlaps the emoji.
                range: 1..5,
                highlight: "keyword",
            },
            SyntaxSpan {
                // Out of order; overlaps the previous highlight.
                range: 0..1,
                highlight: "comment",
            },
        ];

        normalize_syntax_spans_for_text(text, &mut spans);

        assert!(
            spans.iter().all(|s| {
                s.range.start < s.range.end
                    && s.range.end <= text.len()
                    && fret_code_editor_view::clamp_to_grapheme_boundary_down(text, s.range.start)
                        == s.range.start
                    && fret_code_editor_view::clamp_to_grapheme_boundary_up(text, s.range.start)
                        == s.range.start
                    && fret_code_editor_view::clamp_to_grapheme_boundary_down(text, s.range.end)
                        == s.range.end
                    && fret_code_editor_view::clamp_to_grapheme_boundary_up(text, s.range.end)
                        == s.range.end
            }),
            "expected normalized, in-bounds, char-boundary-aligned spans"
        );
        assert!(
            spans.windows(2).all(|w| w[0].range.end <= w[1].range.start),
            "expected non-overlapping spans"
        );

        let mut out: Vec<fret_core::TextSpan> = Vec::new();
        let mut cursor = 0usize;
        for span in spans.iter() {
            if span.range.start > cursor {
                out.push(fret_core::TextSpan::new(span.range.start - cursor));
            }
            out.push(fret_core::TextSpan::new(span.range.end - span.range.start));
            cursor = span.range.end;
        }
        if cursor < text.len() {
            out.push(fret_core::TextSpan::new(text.len() - cursor));
        }
        let rich = fret_core::AttributedText::new(Arc::<str>::from(text), out);
        assert!(
            rich.is_valid(),
            "expected AttributedText to be valid after normalization"
        );
    }

    #[test]
    fn paint_only_syntax_color_changes_do_not_affect_rich_text_shaping_eq() {
        let text: Arc<str> = Arc::<str>::from("fn main() { return 1; }");
        let spans = vec![
            SyntaxSpan {
                range: 0..2,
                highlight: "keyword",
            },
            SyntaxSpan {
                range: 3..7,
                highlight: "function",
            },
            SyntaxSpan {
                range: 10..11,
                highlight: "punctuation",
            },
        ];

        let code_shaping = fret_core::TextShapingStyle::default()
            .with_feature("liga", 0)
            .with_feature("calt", 0);

        let rich_a = materialize_row_rich_text_with_fg(
            Arc::clone(&text),
            &spans,
            &code_shaping,
            |h| match h {
                "keyword" => Some(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                _ => None,
            },
        );
        let rich_b = materialize_row_rich_text_with_fg(
            Arc::clone(&text),
            &spans,
            &code_shaping,
            |h| match h {
                "keyword" => Some(Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                }),
                _ => None,
            },
        );

        assert_ne!(
            rich_a, rich_b,
            "expected paint-only color changes to affect rich text paint"
        );
        assert!(
            rich_a.shaping_eq(&rich_b),
            "expected paint-only color changes to preserve shaping_eq"
        );
        assert!(
            rich_a
                .spans
                .iter()
                .chain(rich_b.spans.iter())
                .all(|s| s.shaping == code_shaping),
            "expected syntax highlighting to remain paint-only relative to the code shaping baseline"
        );
    }

    #[test]
    fn normalize_syntax_spans_does_not_split_zwj_or_vs16_graphemes() {
        let zwj = "👩\u{200D}💻"; // woman technologist
        let vs16 = "✌\u{FE0F}"; // victory hand with VS16
        let text = format!("a{zwj}b{vs16}c");

        let zwj_start = 1;
        let zwj_after_first_scalar = zwj_start + "👩".len();
        let zwj_end = zwj_start + zwj.len();

        let vs16_start = zwj_end + 1;
        let vs16_after_base = vs16_start + "✌".len();
        let vs16_end = vs16_start + vs16.len();

        let mut spans = vec![
            // Span boundaries inside a single grapheme cluster (char-boundary but not grapheme-boundary).
            SyntaxSpan {
                range: zwj_after_first_scalar..zwj_end,
                highlight: "keyword",
            },
            SyntaxSpan {
                range: vs16_start..vs16_after_base,
                highlight: "string",
            },
        ];

        normalize_syntax_spans_for_text(text.as_str(), &mut spans);

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].range, zwj_start..zwj_end);
        assert_eq!(spans[1].range, vs16_start..vs16_end);
    }

    #[test]
    fn syntax_prefetch_key_distinguishes_documents_with_same_revision() {
        let language: Arc<str> = Arc::<str>::from("rust");
        let key_a = SyntaxPrefetchKey {
            doc: DocId::new(),
            rev: fret_code_editor_buffer::Revision(0),
            language: Arc::clone(&language),
            chunk_start: 0,
            chunk_end: 127,
        };
        let key_b = SyntaxPrefetchKey {
            doc: DocId::new(),
            rev: fret_code_editor_buffer::Revision(0),
            language,
            chunk_start: 0,
            chunk_end: 127,
        };

        assert_ne!(key_a, key_b);
    }

    #[test]
    fn row_rich_prefetch_candidates_cover_forward_edge_before_far_lookahead() {
        let rows = row_rich_prefetch_candidate_rows(100, 120, 1_000, 1);
        let expected_head: Vec<usize> = (121..=128).collect();

        assert_eq!(&rows[..expected_head.len()], expected_head.as_slice());
        assert!(rows.contains(&(120 + SYNTAX_PREFETCH_AHEAD_ROWS)));
        assert!(rows.contains(&99));
    }

    #[test]
    fn row_rich_prefetch_candidates_cover_backward_edge_before_far_lookahead() {
        let rows = row_rich_prefetch_candidate_rows(100, 120, 1_000, -1);
        let expected_head: Vec<usize> = (92..=99).rev().collect();

        assert_eq!(&rows[..expected_head.len()], expected_head.as_slice());
        assert!(rows.contains(&(100 - SYNTAX_PREFETCH_AHEAD_ROWS)));
        assert!(rows.contains(&121));
    }

    fn row_rich_prefetch_test_key(
        doc: DocId,
        line: Arc<str>,
        syntax_spans: Arc<[SyntaxSpan]>,
        row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
    ) -> RowRichPrefetchKey {
        RowRichPrefetchKey::new(
            doc,
            fret_code_editor_buffer::Revision(1),
            Arc::<str>::from("rust"),
            4,
            10..13,
            7,
            9,
            line,
            syntax_spans,
            row_spans,
        )
    }

    #[test]
    fn row_rich_prefetch_key_distinguishes_documents_and_arc_identity() {
        let line: Arc<str> = Arc::<str>::from("abc");
        let syntax_spans: Arc<[SyntaxSpan]> = Arc::from(vec![SyntaxSpan {
            range: 0..3,
            highlight: "keyword",
        }]);
        let row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]> = Arc::from(Vec::new());
        let key_a = row_rich_prefetch_test_key(
            DocId::new(),
            Arc::clone(&line),
            Arc::clone(&syntax_spans),
            Arc::clone(&row_spans),
        );
        let key_b = row_rich_prefetch_test_key(
            DocId::new(),
            Arc::clone(&line),
            Arc::clone(&syntax_spans),
            Arc::clone(&row_spans),
        );
        let key_same_content_new_line = row_rich_prefetch_test_key(
            key_a.doc,
            Arc::<str>::from("abc"),
            Arc::clone(&syntax_spans),
            row_spans,
        );

        assert!(!key_a.matches(&key_b));
        assert!(!key_a.matches(&key_same_content_new_line));
    }

    #[derive(Default)]
    struct NoopDispatcher;

    impl fret_runtime::Dispatcher for NoopDispatcher {
        fn dispatch_on_main_thread(&self, task: fret_runtime::execution::Runnable) {
            task();
        }

        fn dispatch_background(
            &self,
            task: fret_runtime::execution::Runnable,
            _priority: fret_runtime::DispatchPriority,
        ) {
            task();
        }

        fn dispatch_after(
            &self,
            _delay: std::time::Duration,
            task: fret_runtime::execution::Runnable,
        ) {
            task();
        }

        fn wake(&self, _window: Option<fret_core::AppWindowId>) {}

        fn exec_capabilities(&self) -> fret_runtime::ExecCapabilities {
            fret_runtime::ExecCapabilities::default()
        }
    }

    #[test]
    fn row_rich_prefetch_runtime_dedupes_ready_until_drained() {
        let runtime = RowRichPrefetchRuntime::new(Arc::new(NoopDispatcher));
        let line: Arc<str> = Arc::<str>::from("abc");
        let syntax_spans: Arc<[SyntaxSpan]> = Arc::from(vec![SyntaxSpan {
            range: 0..3,
            highlight: "keyword",
        }]);
        let row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]> = Arc::from(Vec::new());
        let key = row_rich_prefetch_test_key(DocId::new(), line, syntax_spans, row_spans);

        assert!(runtime.try_mark_pending(key.clone()));
        assert!(!runtime.try_mark_pending(key.clone()));
        {
            let mut state = runtime.lock_state();
            state.pending.retain(|pending| !pending.matches(&key));
            state.ready.push_back(RowRichPrefetchChunk {
                key: key.clone(),
                rich: AttributedText::new(Arc::clone(&key.line), vec![TextSpan::new(3)]),
            });
        }

        assert!(!runtime.try_mark_pending(key.clone()));
        let drained = runtime.drain_ready();
        assert_eq!(drained.len(), 1);
        assert!(drained[0].key.matches(&key));
        assert!(runtime.try_mark_pending(key));
    }

    #[test]
    fn syntax_rows_from_highlight_spans_maps_across_rows() {
        let row_ranges = vec![0..4, 4..8, 8..12];
        let rows = syntax_rows_from_highlight_spans(
            0,
            10,
            &row_ranges,
            vec![fret_syntax::HighlightSpan {
                range: 1..10,
                highlight: Some("keyword"),
            }],
        );

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, 10);
        assert_eq!(rows[1].0, 11);
        assert_eq!(rows[2].0, 12);
        assert_eq!(
            rows[0].1.as_ref(),
            &[SyntaxSpan {
                range: 1..4,
                highlight: "keyword",
            }]
        );
        assert_eq!(
            rows[1].1.as_ref(),
            &[SyntaxSpan {
                range: 0..4,
                highlight: "keyword",
            }]
        );
        assert_eq!(
            rows[2].1.as_ref(),
            &[SyntaxSpan {
                range: 0..2,
                highlight: "keyword",
            }]
        );
    }

    #[test]
    fn row_lru_queue_compaction_drops_stale_touch_records() {
        let mut cache = HashMap::<usize, ((), u64)>::new();
        cache.insert(1, ((), 5));
        cache.insert(2, ((), 9));

        let mut queue = VecDeque::new();
        for tick in 0..1100 {
            queue.push_back((1, tick));
        }
        queue.push_back((2, 9));

        assert!(compact_row_lru_queue_if_needed(&cache, &mut queue, 2));

        assert_eq!(queue.into_iter().collect::<Vec<_>>(), vec![(1, 5), (2, 9)]);
    }
}
