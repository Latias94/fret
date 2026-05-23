//! Painting, caching, and text shaping helpers for the code editor surface.

use fret_core::time::Instant;
use std::collections::{HashMap, VecDeque};

#[cfg(all(test, feature = "syntax"))]
use super::syntax::SYNTAX_PREFETCH_AHEAD_ROWS;
#[cfg(feature = "syntax")]
use super::syntax::{
    SyntaxRowCacheLookup, SyntaxSpan, lookup_row_syntax_spans, populate_row_syntax_spans_after_miss,
};
use super::*;
mod geom_cache;
mod rich;
mod scene;
mod text;

#[cfg(feature = "syntax")]
pub(super) use self::rich::schedule_row_rich_prefetch_for_frame;
#[cfg(feature = "syntax")]
use self::rich::{
    arc_slice_ptr_or_content_eq, arc_str_ptr_or_content_eq, mapped_row_syntax_spans_for_rich_text,
    materialize_row_rich_text, store_row_rich_cache_entry,
};
pub(super) use self::rich::{
    materialize_preedit_rich_text, materialize_preedit_rich_text_for_range,
};
#[cfg(all(test, feature = "syntax"))]
use self::rich::{
    materialize_row_rich_text_with_fg, normalize_syntax_spans_for_text,
    row_rich_prefetch_candidate_rows,
};
#[cfg(test)]
pub(super) use self::text::cached_row_text;
pub(super) use self::text::{cached_row_content_snapshot, cached_row_text_with_range};
use fret_core::TextMetrics;

const ROW_CACHE_QUEUE_COMPACT_FACTOR: usize = 4;
const ROW_CACHE_QUEUE_COMPACT_MIN_LEN: usize = 1024;

pub(super) fn compact_row_lru_queue_if_needed<T>(
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

pub(super) fn add_paint_perf_elapsed(us: &mut u64, ns: &mut u64, started: Instant) {
    let elapsed = started.elapsed();
    *us = us.saturating_add(elapsed.as_micros() as u64);
    let nanos = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
    *ns = ns.saturating_add(nanos);
}

pub(super) fn row_requires_paint_time_preedit(st: &CodeEditorState, row: usize) -> bool {
    st.preedit.is_some()
        && st
            .paint_frame_overlay
            .caret
            .is_some_and(|caret| caret.row == row)
}

pub(super) fn frame_cache_max_entries(st: &CodeEditorState, max_entries: usize) -> usize {
    if max_entries == 0 {
        return 0;
    }

    max_entries
        .max(st.paint_frame_cache_min_entries)
        .min(CODE_EDITOR_ROW_CACHE_MAX_ENTRIES)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepaint_row_scene_replay_plan_for_frame(
    st: &mut CodeEditorState,
    frame: WindowedRowsPaintFrame,
    content_bounds: Rect,
    cell_w: Px,
    text_cache_max_entries: usize,
    text_style: &TextStyle,
    fg: Color,
    theme_revision: u64,
    scale_factor: f32,
) -> RowSceneReplayPlan {
    let width = Px(content_bounds.size.width.0.max(0.0));
    let stable_max_width = if cell_w.0 > 0.01 {
        Px((cell_w.0 * 512.0).max(width.0))
    } else {
        width
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(stable_max_width),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    scene::replay_row_scene_plan_candidates_for_frame(
        st,
        frame,
        content_bounds,
        text_cache_max_entries,
        text_style,
        fg,
        theme_revision,
        constraints,
        scale_factor,
    )
}

#[cfg(feature = "syntax")]
#[allow(clippy::too_many_arguments)]
pub(super) fn prepaint_row_scene_replay_plan_for_frame_with_edge_prebuild(
    cx: &mut fret_ui::canvas::CanvasPrepaintCx<'_>,
    st: &mut CodeEditorState,
    frame: WindowedRowsPaintFrame,
    content_bounds: Rect,
    cell_w: Px,
    text_cache_max_entries: usize,
    text_style: &TextStyle,
    fg: Color,
    theme_revision: u64,
    scale_factor: f32,
) -> RowSceneReplayPlan {
    let row_count = st.display_map.row_count();
    if row_count > 0 {
        let end = frame.visible_end.min(row_count.saturating_sub(1));
        let start = frame.visible_start.min(end);
        let mut last_prebuilt = None::<usize>;
        for edge in [start, end] {
            if last_prebuilt == Some(edge) || st.row_scene_cache.contains_key(&edge) {
                continue;
            }
            let Some(rect) = frame.row_rect(content_bounds, edge) else {
                continue;
            };
            let _ = cx.with_scene_painter(|painter| {
                prebuild_edge_row_scene_fragment_for_frame(
                    painter,
                    st,
                    edge,
                    rect,
                    frame.row_height,
                    cell_w,
                    text_cache_max_entries,
                    text_style,
                    fg,
                    theme_revision,
                )
            });
            last_prebuilt = Some(edge);
        }
    }

    prepaint_row_scene_replay_plan_for_frame(
        st,
        frame,
        content_bounds,
        cell_w,
        text_cache_max_entries,
        text_style,
        fg,
        theme_revision,
        scale_factor,
    )
}

pub(super) fn take_row_scene_replay_plan_entry(
    plan: Option<&mut RowSceneReplayPlan>,
    frame_seq: u64,
    row: usize,
) -> (Option<RowSceneReplayPlanEntry>, usize, Option<&'static str>) {
    let Some(plan) = plan else {
        return (None, 0, None);
    };
    if plan.frame_seq != frame_seq {
        let rejected = plan.entries.len();
        plan.entries.clear();
        plan.hosted_resources_touched = false;
        return (None, rejected, Some("frame_seq_mismatch"));
    }

    let mut rejected = 0usize;
    while plan.entries.front().is_some_and(|entry| entry.row < row) {
        let _ = plan.entries.pop_front();
        rejected = rejected.saturating_add(1);
    }

    if plan.entries.front().is_some_and(|entry| entry.row == row) {
        return (plan.entries.pop_front(), rejected, None);
    }

    (
        None,
        rejected,
        (rejected > 0).then_some("row_advanced_past_entry"),
    )
}

pub(super) fn take_row_scene_replay_plan_hosted_resources_once(
    plan: Option<&mut RowSceneReplayPlan>,
) -> Option<fret_ui::canvas::CanvasHostedResources> {
    let plan = plan?;
    if plan.hosted_resources_touched || plan.hosted_resources.is_empty() {
        return None;
    }
    plan.hosted_resources_touched = true;
    Some(plan.hosted_resources.clone())
}

#[allow(clippy::too_many_arguments)]
fn row_text_origin_and_constraints(
    st: &mut CodeEditorState,
    row_h: Px,
    cell_w: Px,
    text_style: &TextStyle,
    rect: Rect,
    scale_factor: f32,
    perf_enabled: bool,
) -> (fret_core::Point, CanvasTextConstraints, TextMetrics) {
    let stable_max_width = if cell_w.0 > 0.01 {
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
        let measured_h = Px(row_h.0.max(text_style.size.0).max(16.0));
        let metrics = TextMetrics {
            size: Size::new(Px(0.0), measured_h),
            baseline: Px((measured_h.0 * 0.5).max(0.0)),
        };
        st.baseline_measure_cache = Some(BaselineMeasureCache {
            max_width: stable_max_width,
            row_h,
            scale_bits,
            text_style: text_style.clone(),
            metrics,
            measured_h,
        });
        if perf_enabled {
            st.paint_perf_frame.us_baseline_measure =
                st.paint_perf_frame.us_baseline_measure.saturating_add(0);
        }
        (metrics, measured_h)
    };
    let text_y_pad = Px(((row_h.0 - measured_h.0).max(0.0)) / 2.0);
    let origin = fret_core::Point::new(
        rect.origin.x,
        Px(rect.origin.y.0 + text_y_pad.0 + metrics.baseline.0),
    );
    let constraints = CanvasTextConstraints {
        max_width: Some(stable_max_width),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    (origin, constraints, metrics)
}

#[cfg(feature = "syntax")]
#[allow(clippy::too_many_arguments)]
pub(super) fn prebuild_edge_row_scene_fragment_for_frame(
    painter: &mut fret_ui::canvas::CanvasPrepaintPainter<'_>,
    st: &mut CodeEditorState,
    row: usize,
    rect: Rect,
    row_h: Px,
    cell_w: Px,
    text_cache_max_entries: usize,
    text_style: &TextStyle,
    fg: Color,
    theme_revision: u64,
) -> Option<RowSceneReplayPlanEntry> {
    if row_requires_paint_time_preedit(st, row) {
        return None;
    }
    if st.row_scene_cache.contains_key(&row) {
        return None;
    }

    let perf_enabled = st.paint_perf_enabled;
    let text_cache_max_entries = frame_cache_max_entries(st, text_cache_max_entries);
    let scale_factor = painter.scale_factor();
    let (origin, constraints, _baseline_metrics) = row_text_origin_and_constraints(
        st,
        row_h,
        cell_w,
        text_style,
        rect,
        scale_factor,
        perf_enabled,
    );

    let row_content = cached_row_content_snapshot(st, row, text_cache_max_entries);
    if row_content.preedit_range.is_some() {
        return None;
    }

    let row_range = row_content.range.clone();
    let line = Arc::clone(&row_content.text);
    let row_spans = Arc::clone(&row_content.row_spans);
    let line_idx = st.display_map.display_row_line(row);
    let syntax_spans = match lookup_row_syntax_spans(st, line_idx, text_cache_max_entries) {
        SyntaxRowCacheLookup::Hit(spans) => spans,
        SyntaxRowCacheLookup::Miss { tick } => {
            populate_row_syntax_spans_after_miss(st, line_idx, text_cache_max_entries, tick)
        }
    };

    let scope = painter.key_scope(&"fret-code-editor-row-text");
    let key: u64 = painter.child_key(scope, &(row, 0u8)).into();
    let mut row_scene_syntax_replay_key = None::<RowSceneSyntaxReplayKey>;
    let mut row_scene_is_rich = false;
    let row_scene_key;
    let (blob, blob_metrics) = if !syntax_spans.is_empty()
        && let Some(mapped) = mapped_row_syntax_spans_for_rich_text(
            line.as_ref(),
            row_range
                .start
                .saturating_sub(st.buffer.line_start(line_idx).unwrap_or(row_range.start)),
            row_range.end.saturating_sub(row_range.start),
            row_spans.as_ref(),
            syntax_spans.as_ref(),
        ) {
        let rich = {
            let theme = painter.theme();
            materialize_row_rich_text(
                theme,
                Arc::clone(&line),
                mapped.as_ref(),
                &st.code_font_shaping_style,
            )
        };
        let row_geom_key = geom::RowGeomKey::for_attributed(
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
        );
        row_scene_key = RowSceneKey::syntax(row_geom_key.clone(), fg, theme_revision);
        row_scene_syntax_replay_key = Some(RowSceneSyntaxReplayKey::new(
            row_range.clone(),
            Arc::clone(&line),
            Arc::clone(&row_spans),
            Arc::clone(&syntax_spans),
            text_style,
            constraints,
            st.font_stack_key,
            scale_factor,
            theme_revision,
            st.code_font_feature_policy_rev,
            fg,
        ));
        row_scene_is_rich = true;
        st.row_rich_cache_tick = st.row_rich_cache_tick.saturating_add(1);
        let tick = st.row_rich_cache_tick;
        store_row_rich_cache_entry(
            st,
            row,
            row_range.clone(),
            Arc::clone(&line),
            Arc::clone(&syntax_spans),
            Arc::clone(&row_spans),
            theme_revision,
            st.code_font_feature_policy_rev,
            rich.clone(),
            text_cache_max_entries.min(2048),
            tick,
        );
        painter.rich_text_with_blob(
            key,
            DrawOrder(2),
            origin,
            rich,
            text_style.clone(),
            fg,
            constraints,
            scale_factor,
        )
    } else if !st.code_font_shaping_style.features.is_empty() {
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
            text_style,
            (
                constraints.max_width,
                constraints.wrap,
                constraints.overflow,
                fret_core::TextAlign::Start,
                scale_factor,
            ),
            st.font_stack_key,
        );
        row_scene_key = RowSceneKey::plain(row_geom_key, fg);
        row_scene_is_rich = true;
        painter.rich_text_with_blob(
            key,
            DrawOrder(2),
            origin,
            rich,
            text_style.clone(),
            fg,
            constraints,
            scale_factor,
        )
    } else {
        let row_geom_key = geom::RowGeomKey::for_plain(
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
        );
        row_scene_key = RowSceneKey::plain(row_geom_key, fg);
        painter.text_with_blob(
            key,
            DrawOrder(2),
            origin,
            Arc::clone(&line),
            text_style.clone(),
            fg,
            constraints,
            scale_factor,
        )
    };

    let mut stops: Vec<(usize, Px)> = Vec::new();
    let caret_rect = {
        let (services, _) = painter.services_and_scene();
        services.text().caret_stops(blob, &mut stops);
        services
            .text()
            .caret_rect(blob, 0, CaretAffinity::Downstream)
    };
    let text_box_top_in_row = Px(origin.y.0 - blob_metrics.baseline.0 - rect.origin.y.0);
    let (caret_rect_top, caret_rect_height) = if caret_rect.size.height.0 > 0.0 {
        (
            Some(Px(text_box_top_in_row.0 + caret_rect.origin.y.0)),
            Some(caret_rect.size.height),
        )
    } else if blob_metrics.size.height.0 > 0.0 {
        (Some(text_box_top_in_row), Some(blob_metrics.size.height))
    } else {
        (None, None)
    };
    let geom = RowGeom {
        row_range: row_range.clone(),
        key: row_scene_key.row_geom_key.clone(),
        caret_stops: stops,
        fold_map: row_content.fold_map.clone(),
        caret_rect_top,
        caret_rect_height,
        has_preedit: false,
        preedit: None,
    };

    let fragment = painter.scene_fragment((), rect, origin);

    #[cfg(feature = "syntax")]
    let retained = scene::store_row_scene_cache(
        st,
        row,
        row_scene_key,
        Arc::clone(&row_content),
        origin,
        geom.clone(),
        row_scene_is_rich,
        fragment.ops.as_ref().to_vec(),
        row_scene_syntax_replay_key,
        text_cache_max_entries,
        scene::RowSceneStoreSource::PrepaintEdge,
    );
    geom_cache::store_row_geom_cache(st, row, Some(geom), text_cache_max_entries, perf_enabled);

    retained.map(|retained| RowSceneReplayPlanEntry {
        row,
        retained,
        local_bounds: rect,
    })
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
    let cache_base_entries = text_cache_max_entries;
    let text_cache_max_entries = frame_cache_max_entries(st, text_cache_max_entries);
    let row_started = perf_enabled.then(Instant::now);

    if perf_enabled {
        st.paint_perf_frame.cache_base_entries = cache_base_entries as u64;
        st.paint_perf_frame.cache_effective_entries = text_cache_max_entries as u64;
        st.paint_perf_frame.rows_painted = st.paint_perf_frame.rows_painted.saturating_add(1);
    }

    let replay_setup_started = perf_enabled.then(Instant::now);
    let (replay_plan_entry, rejected_entries, reject_reason) = take_row_scene_replay_plan_entry(
        painter.scene_fragment_mut::<RowSceneReplayPlan>(),
        st.paint_perf_frame.frame_seq,
        row,
    );
    if rejected_entries > 0 {
        painter.record_scene_fragment_rejected_entries(
            rejected_entries,
            reject_reason.unwrap_or("row_scene_plan_rejected"),
        );
    }
    let replay_plan_entry_matches_rect = replay_plan_entry
        .as_ref()
        .is_some_and(|entry| entry.local_bounds == rect);
    if replay_plan_entry_matches_rect {
        painter.record_scene_fragment_used_entries(1);
    }

    let row_content = if let Some(entry) = replay_plan_entry
        .as_ref()
        .filter(|_| replay_plan_entry_matches_rect)
    {
        Arc::clone(&entry.retained.content)
    } else if perf_enabled {
        let started = Instant::now();
        let out = cached_row_content_snapshot(st, row, text_cache_max_entries);
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_text,
            &mut st.paint_perf_frame.ns_row_text,
            started,
        );
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_content_resolve,
            &mut st.paint_perf_frame.ns_row_content_resolve,
            started,
        );
        out
    } else {
        cached_row_content_snapshot(st, row, text_cache_max_entries)
    };
    let row_range = row_content.range.clone();
    if replay_plan_entry.is_some() && !replay_plan_entry_matches_rect {
        painter.record_scene_fragment_rejected_entries(1, "rect_mismatch");
    }
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
    let overlay = st.paint_frame_overlay;
    let can_return_after_planned_replay =
        replay_plan_entry_matches_rect && !overlay.touches_row(row, &row_range);
    let compose_inline_preedit = st.compose_inline_preedit
        || st
            .preedit_replace_range
            .as_ref()
            .is_some_and(|r| !r.is_empty());

    if let Some(entry) = replay_plan_entry.as_ref() {
        if entry.local_bounds == rect {
            row_scene_key = None;
            row_scene_is_rich = entry.retained.is_rich;
            row_scene_replayed = true;
            drew_rich = entry.retained.is_rich;
            row_preedit = entry.retained.geom.preedit;
            let replay_plan_hosted_resources = take_row_scene_replay_plan_hosted_resources_once(
                painter.scene_fragment_mut::<RowSceneReplayPlan>(),
            );
            if let Some(started) = replay_setup_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_row_scene_replay_setup,
                    &mut st.paint_perf_frame.ns_row_scene_replay_setup,
                    started,
                );
            }
            scene::replay_row_scene_plan_entry(
                painter,
                st,
                replay_plan_hosted_resources.as_ref(),
                entry,
                origin,
            );
        }
    }

    if can_return_after_planned_replay {
        let geom_for_cache = replay_plan_entry.as_ref().and_then(|entry| {
            (!st.row_geom_cache.contains_key(&row)).then(|| entry.retained.geom.clone())
        });
        geom_cache::store_row_geom_cache(
            st,
            row,
            geom_for_cache,
            text_cache_max_entries,
            perf_enabled,
        );
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
        return;
    }
    if replay_plan_entry_matches_rect {
        fresh_geom = replay_plan_entry
            .as_ref()
            .map(|entry| entry.retained.geom.clone());
    }

    let line = Arc::clone(&row_content.text);
    let row_folds = row_content.fold_map.clone();
    let row_preedit_range = row_content.preedit_range.clone();
    let row_spans = Arc::clone(&row_content.row_spans);
    #[cfg(not(feature = "syntax"))]
    let _ = &row_spans;

    if !row_scene_replayed && let Some(preedit) = st.preedit.clone() {
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
                let row_geom_key_started = perf_enabled.then(Instant::now);
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
                if let Some(started) = row_geom_key_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_geom_key,
                        &mut st.paint_perf_frame.ns_row_geom_key,
                        started,
                    );
                }
                let row_scene_key_started = perf_enabled.then(Instant::now);
                row_scene_key = row_geom_key
                    .clone()
                    .map(|key| RowSceneKey::preedit(key, fg, selection_bg));
                if let Some(started) = row_scene_key_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_key,
                        &mut st.paint_perf_frame.ns_row_scene_key,
                        started,
                    );
                }
                row_scene_is_rich = true;
                if let Some(scene_key) = row_scene_key.clone()
                    && let Some((geom, is_rich)) = scene::try_replay_row_scene_cache(
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
            if let Some(caret_overlay) = overlay.caret
                && caret_overlay.row == row
            {
                let caret_local = caret_overlay.byte.saturating_sub(row_range.start);
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
                let row_geom_key_started = perf_enabled.then(Instant::now);
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
                if let Some(started) = row_geom_key_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_geom_key,
                        &mut st.paint_perf_frame.ns_row_geom_key,
                        started,
                    );
                }
                let row_scene_key_started = perf_enabled.then(Instant::now);
                row_scene_key = row_geom_key
                    .clone()
                    .map(|key| RowSceneKey::preedit(key, fg, selection_bg));
                if let Some(started) = row_scene_key_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_scene_key,
                        &mut st.paint_perf_frame.ns_row_scene_key,
                        started,
                    );
                }
                row_scene_is_rich = true;
                if let Some(scene_key) = row_scene_key.clone()
                    && let Some((geom, is_rich)) = scene::try_replay_row_scene_cache(
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
    if !row_scene_replayed && !drew_rich {
        let line_idx = st.display_map.display_row_line(row);
        let theme_revision = {
            let theme = painter.theme();
            theme.revision()
        };
        #[allow(unused_assignments)]
        let mut syntax_spans = None::<Arc<[SyntaxSpan]>>;
        let mut syntax_lookup_miss_tick = None::<u64>;
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
                    && let Some((geom, is_rich)) = scene::try_replay_row_scene_cache_fast_syntax(
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

                let row_rich_cache_compare_started = perf_enabled.then(Instant::now);
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
                if let Some(started) = row_rich_cache_compare_started {
                    add_paint_perf_elapsed(
                        &mut st.paint_perf_frame.us_row_rich_cache_compare,
                        &mut st.paint_perf_frame.ns_row_rich_cache_compare,
                        started,
                    );
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
                    let row_geom_key_started = perf_enabled.then(Instant::now);
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
                    if let Some(started) = row_geom_key_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_geom_key,
                            &mut st.paint_perf_frame.ns_row_geom_key,
                            started,
                        );
                    }
                    let row_scene_key_started = perf_enabled.then(Instant::now);
                    row_scene_key = row_geom_key
                        .clone()
                        .map(|key| RowSceneKey::syntax(key, fg, theme_revision));
                    if let Some(started) = row_scene_key_started {
                        add_paint_perf_elapsed(
                            &mut st.paint_perf_frame.us_row_scene_key,
                            &mut st.paint_perf_frame.ns_row_scene_key,
                            started,
                        );
                    }
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
                        && let Some((geom, is_rich)) = scene::try_replay_row_scene_cache(
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
                        scene::refresh_row_scene_syntax_replay_key(
                            st,
                            row,
                            row_scene_syntax_replay_key.as_ref(),
                        );
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
                        let row_geom_key_started = perf_enabled.then(Instant::now);
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
                        if let Some(started) = row_geom_key_started {
                            add_paint_perf_elapsed(
                                &mut st.paint_perf_frame.us_row_geom_key,
                                &mut st.paint_perf_frame.ns_row_geom_key,
                                started,
                            );
                        }
                        let row_scene_key_started = perf_enabled.then(Instant::now);
                        row_scene_key = row_geom_key
                            .clone()
                            .map(|key| RowSceneKey::syntax(key, fg, theme_revision));
                        if let Some(started) = row_scene_key_started {
                            add_paint_perf_elapsed(
                                &mut st.paint_perf_frame.us_row_scene_key,
                                &mut st.paint_perf_frame.ns_row_scene_key,
                                started,
                            );
                        }
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
                            && let Some((geom, is_rich)) = scene::try_replay_row_scene_cache(
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
                            scene::refresh_row_scene_syntax_replay_key(
                                st,
                                row,
                                row_scene_syntax_replay_key.as_ref(),
                            );
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
            let row_geom_key_started = perf_enabled.then(Instant::now);
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
            if let Some(started) = row_geom_key_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_row_geom_key,
                    &mut st.paint_perf_frame.ns_row_geom_key,
                    started,
                );
            }
            let row_scene_key_started = perf_enabled.then(Instant::now);
            row_scene_key = row_geom_key.clone().map(|key| RowSceneKey::plain(key, fg));
            if let Some(started) = row_scene_key_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_row_scene_key,
                    &mut st.paint_perf_frame.ns_row_scene_key,
                    started,
                );
            }
            row_scene_is_rich = true;
            if let Some(scene_key) = row_scene_key.clone()
                && let Some((geom, is_rich)) = scene::try_replay_row_scene_cache(
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
            let row_geom_key_started = perf_enabled.then(Instant::now);
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
            if let Some(started) = row_geom_key_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_row_geom_key,
                    &mut st.paint_perf_frame.ns_row_geom_key,
                    started,
                );
            }
            let row_scene_key_started = perf_enabled.then(Instant::now);
            row_scene_key = row_geom_key.clone().map(|key| RowSceneKey::plain(key, fg));
            if let Some(started) = row_scene_key_started {
                add_paint_perf_elapsed(
                    &mut st.paint_perf_frame.us_row_scene_key,
                    &mut st.paint_perf_frame.ns_row_scene_key,
                    started,
                );
            }
            if let Some(scene_key) = row_scene_key.clone()
                && let Some((geom, is_rich)) = scene::try_replay_row_scene_cache(
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
    let row_geom_resolve_started = perf_enabled.then(Instant::now);
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
    if let Some(started) = row_geom_resolve_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_geom_resolve,
            &mut st.paint_perf_frame.ns_row_geom_resolve,
            started,
        );
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

    let row_overlay_started = perf_enabled.then(Instant::now);
    let sel = overlay.selection_range();
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
        if let Some(caret_overlay) = overlay.caret
            && caret_overlay.row == row
        {
            let mut local = caret_overlay.byte.saturating_sub(row_range.start);
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
    } else {
        // Fallback to the MVP monospace heuristic if caret stops are unavailable.
        if !drew_selection && !sel.is_empty() {
            let start_pt = overlay.selection_start_point;
            let end_pt = overlay.selection_end_point;
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
                    if perf_enabled {
                        st.paint_perf_frame.quads_selection =
                            st.paint_perf_frame.quads_selection.saturating_add(1);
                    }
                }
            }
        }

        if let Some(caret_overlay) = overlay.caret
            && caret_overlay.row == row
        {
            let caret_rect = if let Some(blob) = row_blob {
                let mut local = caret_overlay.byte.saturating_sub(row_range.start);
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
                let mut col = caret_overlay.col;
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
                st.paint_perf_frame.quads_caret = st.paint_perf_frame.quads_caret.saturating_add(1);
            }
        }
    }
    if let Some(started) = row_overlay_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_overlay,
            &mut st.paint_perf_frame.ns_row_overlay,
            started,
        );
    }

    #[cfg(feature = "syntax")]
    if let Some((row_scene_key, geom, is_rich, ops, syntax_replay_key)) = pending_row_scene_store {
        scene::store_row_scene_cache(
            st,
            row,
            row_scene_key,
            row_content.clone(),
            origin,
            geom,
            is_rich,
            ops,
            syntax_replay_key,
            text_cache_max_entries,
            scene::RowSceneStoreSource::Paint,
        );
    }
    #[cfg(not(feature = "syntax"))]
    if let Some((row_scene_key, geom, is_rich, ops)) = pending_row_scene_store {
        scene::store_row_scene_cache(
            st,
            row,
            row_scene_key,
            row_content.clone(),
            origin,
            geom,
            is_rich,
            ops,
            text_cache_max_entries,
        );
    }

    geom_cache::store_row_geom_cache(st, row, fresh_geom, text_cache_max_entries, perf_enabled);

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
    fn syntax_replay_key_matches_equivalent_current_inputs() {
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

        let other_line: Arc<str> = Arc::<str>::from("abc");
        let other_row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]> = Arc::from(Vec::new());
        let other_syntax_spans: Arc<[SyntaxSpan]> = Arc::from(vec![SyntaxSpan {
            range: 0..3,
            highlight: "keyword",
        }]);
        assert!(key.matches_current(
            &(0..3),
            &other_line,
            &other_row_spans,
            &other_syntax_spans,
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

        let different_syntax_spans: Arc<[SyntaxSpan]> = Arc::from(vec![SyntaxSpan {
            range: 0..3,
            highlight: "string",
        }]);
        assert!(!key.matches_current(
            &(0..3),
            &other_line,
            &other_row_spans,
            &different_syntax_spans,
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
        let rows = super::syntax::syntax_rows_from_highlight_spans(
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
