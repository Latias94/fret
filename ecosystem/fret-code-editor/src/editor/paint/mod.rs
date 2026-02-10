//! Painting, caching, and text shaping helpers for the code editor surface.

use std::ops::Range;
use std::time::Instant;

use super::*;
use fret_core::TextMetrics;

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

    let (row_range, line, row_folds, row_preedit_range) = if perf_enabled {
        let started = Instant::now();
        let out = cached_row_text_with_range(st, row, text_cache_max_entries);
        st.paint_perf_frame.us_row_text = st
            .paint_perf_frame
            .us_row_text
            .saturating_add(started.elapsed().as_micros() as u64);
        out
    } else {
        cached_row_text_with_range(st, row, text_cache_max_entries)
    };
    painter.scene().push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: fret_core::Paint::TRANSPARENT,

        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT,

        corner_radii: Corners::all(Px(0.0)),
    });
    if perf_enabled {
        st.paint_perf_frame.quads_background =
            st.paint_perf_frame.quads_background.saturating_add(1);
    }

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
            scale_factor,
        };
        let started = perf_enabled.then(Instant::now);
        let metrics = services
            .text()
            .measure_str(" ", text_style, measure_constraints);
        if let Some(started) = started {
            st.paint_perf_frame.us_baseline_measure = st
                .paint_perf_frame
                .us_baseline_measure
                .saturating_add(started.elapsed().as_micros() as u64);
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

    if let Some(preedit) = &st.preedit {
        if st.compose_inline_preedit {
            if let Some(range) = row_preedit_range.clone() {
                let rich = materialize_preedit_rich_text_for_range(
                    Arc::clone(&line),
                    range,
                    preedit,
                    fg,
                    selection_bg,
                );
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
                    preedit,
                    fg,
                    selection_bg,
                );
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
                    st.paint_perf_frame.us_text_draw = st
                        .paint_perf_frame
                        .us_text_draw
                        .saturating_add(started.elapsed().as_micros() as u64);
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
    let row_has_preedit = st.preedit.is_some()
        && if st.compose_inline_preedit {
            row_preedit_range.is_some()
        } else {
            row_preedit.is_some()
        };
    #[cfg(feature = "syntax")]
    {
        if !drew_rich {
            let line_idx = st.display_map.display_row_line(row);
            let spans = if perf_enabled {
                let started = Instant::now();
                let spans = cached_row_syntax_spans(st, line_idx, text_cache_max_entries);
                st.paint_perf_frame.us_syntax_spans = st
                    .paint_perf_frame
                    .us_syntax_spans
                    .saturating_add(started.elapsed().as_micros() as u64);
                spans
            } else {
                cached_row_syntax_spans(st, line_idx, text_cache_max_entries)
            };
            if !spans.is_empty() {
                let rich_cache_max_entries = text_cache_max_entries.min(2048);
                st.cache_stats.row_rich_get_calls =
                    st.cache_stats.row_rich_get_calls.saturating_add(1);

                let seg_start_in_line = row_range
                    .start
                    .saturating_sub(st.buffer.line_start(line_idx).unwrap_or(row_range.start));
                let seg_end_in_line = seg_start_in_line.saturating_add(line.len());

                let theme_revision = {
                    let theme = painter.theme();
                    theme.revision()
                };

                st.row_rich_cache_tick = st.row_rich_cache_tick.saturating_add(1);
                let tick = st.row_rich_cache_tick;

                if let Some((cached, last_used)) = st.row_rich_cache.get_mut(&row) {
                    let hit = cached.theme_revision == theme_revision
                        && cached.row_range == row_range
                        && Arc::ptr_eq(&cached.line, &line)
                        && Arc::ptr_eq(&cached.syntax_spans, &spans);
                    if hit {
                        *last_used = tick;
                        st.row_rich_cache_queue.push_back((row, tick));
                        st.cache_stats.row_rich_hits =
                            st.cache_stats.row_rich_hits.saturating_add(1);

                        let started = perf_enabled.then(Instant::now);
                        let (blob, metrics) = painter.rich_text_with_blob(
                            key,
                            DrawOrder(2),
                            origin,
                            cached.rich.clone(),
                            text_style.clone(),
                            fg,
                            constraints,
                            scale_factor,
                        );
                        if let Some(started) = started {
                            st.paint_perf_frame.us_text_draw = st
                                .paint_perf_frame
                                .us_text_draw
                                .saturating_add(started.elapsed().as_micros() as u64);
                        }
                        row_blob = Some(blob);
                        row_blob_metrics = Some(metrics);
                        drew_rich = true;
                    }
                }

                if !drew_rich {
                    st.cache_stats.row_rich_misses =
                        st.cache_stats.row_rich_misses.saturating_add(1);

                    let mut clipped: Vec<SyntaxSpan> = Vec::new();
                    for span in spans.as_ref() {
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

                    if !clipped.is_empty() {
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

                        let started = perf_enabled.then(Instant::now);
                        let rich = {
                            let theme = painter.theme();
                            materialize_row_rich_text(theme, Arc::clone(&line), merged.as_ref())
                        };
                        if let Some(started) = started {
                            st.paint_perf_frame.us_rich_materialize = st
                                .paint_perf_frame
                                .us_rich_materialize
                                .saturating_add(started.elapsed().as_micros() as u64);
                        }
                        st.row_rich_cache.insert(
                            row,
                            (
                                RowRichCacheEntry {
                                    row_range: row_range.clone(),
                                    line: Arc::clone(&line),
                                    syntax_spans: Arc::clone(&spans),
                                    theme_revision,
                                    rich: rich.clone(),
                                },
                                tick,
                            ),
                        );
                        st.row_rich_cache_queue.push_back((row, tick));

                        while st.row_rich_cache.len() > rich_cache_max_entries {
                            let Some((victim, victim_tick)) = st.row_rich_cache_queue.pop_front()
                            else {
                                break;
                            };
                            let remove = st
                                .row_rich_cache
                                .get(&victim)
                                .is_some_and(|(_, last_used)| *last_used == victim_tick);
                            if remove {
                                st.row_rich_cache.remove(&victim);
                                st.cache_stats.row_rich_evictions =
                                    st.cache_stats.row_rich_evictions.saturating_add(1);
                            }
                        }

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
                            st.paint_perf_frame.us_text_draw = st
                                .paint_perf_frame
                                .us_text_draw
                                .saturating_add(started.elapsed().as_micros() as u64);
                        }
                        row_blob = Some(blob);
                        row_blob_metrics = Some(metrics);
                        drew_rich = true;
                    }
                }
            }
        }
    }

    if !drew_rich {
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
            st.paint_perf_frame.us_text_draw = st
                .paint_perf_frame
                .us_text_draw
                .saturating_add(started.elapsed().as_micros() as u64);
        }
        row_blob = Some(blob);
        row_blob_metrics = Some(metrics);
    }

    let mut fresh_geom = None::<RowGeom>;
    let mut caret_stops = &[][..];
    let mut caret_rect_top = None::<Px>;
    let mut caret_rect_height = None::<Px>;
    if let (Some(blob), Some(blob_metrics)) = (row_blob, row_blob_metrics.as_ref()) {
        let cached = st.row_geom_cache.get(&row).is_some_and(|(geom, _)| {
            geom.blob == blob
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
                st.paint_perf_frame.us_caret_stops = st
                    .paint_perf_frame
                    .us_caret_stops
                    .saturating_add(started.elapsed().as_micros() as u64);
            }
            let caret_rect_started = perf_enabled.then(Instant::now);
            let caret_rect = services
                .text()
                .caret_rect(blob, 0, CaretAffinity::Downstream);
            if let Some(started) = caret_rect_started {
                st.paint_perf_frame.us_caret_rect = st
                    .paint_perf_frame
                    .us_caret_rect
                    .saturating_add(started.elapsed().as_micros() as u64);
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
                blob,
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
                    st.paint_perf_frame.us_selection_rects = st
                        .paint_perf_frame
                        .us_selection_rects
                        .saturating_add(started.elapsed().as_micros() as u64);
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
                        background: fret_core::Paint::Solid(selection_bg),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

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
                        background: fret_core::Paint::Solid(selection_bg),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

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
                    background: fret_core::Paint::Solid(caret_color),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

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
                        background: fret_core::Paint::Solid(selection_bg),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

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
                        st.paint_perf_frame.us_caret_x = st
                            .paint_perf_frame
                            .us_caret_x
                            .saturating_add(started.elapsed().as_micros() as u64);
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
                    background: fret_core::Paint::Solid(caret_color),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

                    corner_radii: Corners::all(Px(0.0)),
                });
                if perf_enabled {
                    st.paint_perf_frame.quads_caret =
                        st.paint_perf_frame.quads_caret.saturating_add(1);
                }
            }
        }
    }

    // Cache row geometry for pointer hit-testing / IME cursor-area anchoring in event handlers.
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    let folds_epoch = st.folds_epoch;
    let inlays_epoch = st.inlays_epoch;
    if st.row_geom_cache_rev != rev
        || st.row_geom_cache_wrap_cols != wrap_cols
        || st.row_geom_cache_folds_epoch != folds_epoch
        || st.row_geom_cache_inlays_epoch != inlays_epoch
    {
        st.row_geom_cache_rev = rev;
        st.row_geom_cache_wrap_cols = wrap_cols;
        st.row_geom_cache_folds_epoch = folds_epoch;
        st.row_geom_cache_inlays_epoch = inlays_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
    }

    st.row_geom_cache_tick = st.row_geom_cache_tick.saturating_add(1);
    let tick = st.row_geom_cache_tick;
    let has_row_geom = fresh_geom.is_some() || st.row_geom_cache.contains_key(&row);
    if has_row_geom {
        if let Some(geom) = fresh_geom {
            st.row_geom_cache.insert(row, (geom, tick));
        } else if let Some((_, last_used)) = st.row_geom_cache.get_mut(&row) {
            *last_used = tick;
        }

        st.row_geom_cache_queue.push_back((row, tick));
        while st.row_geom_cache.len() > text_cache_max_entries {
            let Some((victim, victim_tick)) = st.row_geom_cache_queue.pop_front() else {
                break;
            };
            let remove = st
                .row_geom_cache
                .get(&victim)
                .is_some_and(|(_, last_used)| *last_used == victim_tick);
            if remove {
                st.row_geom_cache.remove(&victim);
            }
        }
    }

    if perf_enabled {
        st.paint_perf_frame.rows_drew_rich = st
            .paint_perf_frame
            .rows_drew_rich
            .saturating_add(drew_rich as u64);
        if let Some(row_started) = row_started {
            st.paint_perf_frame.us_total = st
                .paint_perf_frame
                .us_total
                .saturating_add(row_started.elapsed().as_micros() as u64);
        }
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
) {
    st.cache_stats.row_text_get_calls = st.cache_stats.row_text_get_calls.saturating_add(1);
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    let folds_epoch = st.folds_epoch;
    let inlays_epoch = st.inlays_epoch;
    if st.row_text_cache_rev != rev
        || st.row_text_cache_wrap_cols != wrap_cols
        || st.row_text_cache_folds_epoch != folds_epoch
        || st.row_text_cache_inlays_epoch != inlays_epoch
    {
        st.row_text_cache_rev = rev;
        st.row_text_cache_wrap_cols = wrap_cols;
        st.row_text_cache_folds_epoch = folds_epoch;
        st.row_text_cache_inlays_epoch = inlays_epoch;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.cache_stats.row_text_resets = st.cache_stats.row_text_resets.saturating_add(1);
        #[cfg(feature = "syntax")]
        {
            st.row_rich_cache_tick = 0;
            st.row_rich_cache.clear();
            st.row_rich_cache_queue.clear();
            st.cache_stats.row_rich_resets = st.cache_stats.row_rich_resets.saturating_add(1);
        }
    }

    st.row_text_cache_tick = st.row_text_cache_tick.saturating_add(1);
    let tick = st.row_text_cache_tick;

    if let Some((text, last_used)) = st.row_text_cache.get_mut(&row) {
        *last_used = tick;
        st.row_text_cache_queue.push_back((row, tick));
        st.cache_stats.row_text_hits = st.cache_stats.row_text_hits.saturating_add(1);
        return (
            text.range.clone(),
            Arc::clone(&text.text),
            text.fold_map.clone(),
            text.preedit_range.clone(),
        );
    }
    st.cache_stats.row_text_misses = st.cache_stats.row_text_misses.saturating_add(1);

    let materialized = st.display_map.materialize_display_row_text(&st.buffer, row);
    let range = materialized.row_range.clone();
    let range_for_return = range.clone();
    let preedit_range = materialized.preedit_range.clone();

    let spans: Vec<super::geom::RowFoldSpan> = materialized
        .spans
        .into_iter()
        .map(|span| super::geom::RowFoldSpan {
            buffer_range: span.buffer_range,
            display_range: span.display_range,
        })
        .collect();
    let fold_map = (!spans.is_empty()).then_some(super::geom::RowFoldMap::new(spans));
    let text = materialized.text;

    st.row_text_cache.insert(
        row,
        (
            RowTextCacheEntry {
                text: Arc::clone(&text),
                range,
                fold_map: fold_map.clone(),
                preedit_range: preedit_range.clone(),
            },
            tick,
        ),
    );
    st.row_text_cache_queue.push_back((row, tick));

    while st.row_text_cache.len() > max_entries {
        let Some((victim, victim_tick)) = st.row_text_cache_queue.pop_front() else {
            break;
        };
        let remove = st
            .row_text_cache
            .get(&victim)
            .is_some_and(|(_, last_used)| *last_used == victim_tick);
        if remove {
            st.row_text_cache.remove(&victim);
            st.cache_stats.row_text_evictions = st.cache_stats.row_text_evictions.saturating_add(1);
        }
    }

    (range_for_return, text, fold_map, preedit_range)
}

pub(super) fn materialize_preedit_rich_text(
    line: Arc<str>,
    caret_in_line: usize,
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

    AttributedText::new(display, spans)
}

pub(super) fn materialize_preedit_rich_text_for_range(
    line: Arc<str>,
    preedit_range: Range<usize>,
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
pub(super) fn cached_row_syntax_spans(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> Arc<[SyntaxSpan]> {
    st.cache_stats.syntax_get_calls = st.cache_stats.syntax_get_calls.saturating_add(1);
    let rev = st.buffer.revision();
    if st.syntax_row_cache_rev != rev || st.syntax_row_cache_language != st.language {
        st.syntax_row_cache_rev = rev;
        st.syntax_row_cache_language = st.language.clone();
        st.syntax_row_cache_tick = 0;
        st.syntax_row_cache.clear();
        st.syntax_row_cache_queue.clear();
        st.cache_stats.syntax_resets = st.cache_stats.syntax_resets.saturating_add(1);
        st.row_rich_cache_tick = 0;
        st.row_rich_cache.clear();
        st.row_rich_cache_queue.clear();
        st.cache_stats.row_rich_resets = st.cache_stats.row_rich_resets.saturating_add(1);
    }

    st.syntax_row_cache_tick = st.syntax_row_cache_tick.saturating_add(1);
    let tick = st.syntax_row_cache_tick;

    if let Some((spans, last_used)) = st.syntax_row_cache.get_mut(&row) {
        *last_used = tick;
        st.syntax_row_cache_queue.push_back((row, tick));
        st.cache_stats.syntax_hits = st.cache_stats.syntax_hits.saturating_add(1);
        return Arc::clone(spans);
    }
    st.cache_stats.syntax_misses = st.cache_stats.syntax_misses.saturating_add(1);

    let language = st.language.clone();
    let Some(language) = language.as_deref() else {
        return Arc::<[SyntaxSpan]>::from([]);
    };

    let line_count = st.buffer.line_count();
    if line_count == 0 {
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

    let Some(slice) = st.buffer.slice_to_string(start_byte..end_byte) else {
        return;
    };

    let Ok(spans) = fret_syntax::highlight(slice.as_str(), language) else {
        return;
    };

    let mut row_ranges = Vec::with_capacity(chunk_end - chunk_start + 1);
    for row in chunk_start..=chunk_end {
        row_ranges.push(st.buffer.line_byte_range(row).unwrap_or(0..0));
    }

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

        let global_end_for_row = global_end.saturating_sub(1);
        let start_row = st.buffer.line_index_at_byte(global_start);
        let end_row = st.buffer.line_index_at_byte(global_end_for_row);

        for row in start_row..=end_row {
            if row < chunk_start || row > chunk_end {
                continue;
            }
            let row_idx = row - chunk_start;
            let row_range = &row_ranges[row_idx];
            let inter_start = global_start.max(row_range.start);
            let inter_end = global_end.min(row_range.end);
            if inter_start >= inter_end {
                continue;
            }
            per_row[row_idx].push(SyntaxSpan {
                range: (inter_start - row_range.start)..(inter_end - row_range.start),
                highlight,
            });
        }
    }

    for (i, spans) in per_row.into_iter().enumerate() {
        let row = chunk_start + i;

        let mut spans = spans;
        spans.sort_by_key(|s| s.range.start);
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

        let spans: Arc<[SyntaxSpan]> = Arc::from(merged);
        st.syntax_row_cache.insert(row, (Arc::clone(&spans), tick));
        st.syntax_row_cache_queue.push_back((row, tick));

        while st.syntax_row_cache.len() > max_entries {
            let Some((victim, victim_tick)) = st.syntax_row_cache_queue.pop_front() else {
                break;
            };
            let remove = st
                .syntax_row_cache
                .get(&victim)
                .is_some_and(|(_, last_used)| *last_used == victim_tick);
            if remove {
                st.syntax_row_cache.remove(&victim);
                st.cache_stats.syntax_evictions = st.cache_stats.syntax_evictions.saturating_add(1);
            }
        }
    }
}

#[cfg(feature = "syntax")]
pub(super) fn syntax_color(theme: &fret_ui::Theme, highlight: &str) -> Option<Color> {
    let mut key = String::with_capacity("color.syntax.".len() + highlight.len());
    key.push_str("color.syntax.");
    key.push_str(highlight);
    if let Some(c) = theme.color_by_key(key.as_str()) {
        return Some(c);
    }

    let fallback = highlight.split('.').next().unwrap_or(highlight);
    if fallback != highlight {
        let mut key = String::with_capacity("color.syntax.".len() + fallback.len());
        key.push_str("color.syntax.");
        key.push_str(fallback);
        if let Some(c) = theme.color_by_key(key.as_str()) {
            return Some(c);
        }
    }

    match fallback {
        "comment" => Some(theme.color_required("muted-foreground")),
        "keyword" | "operator" => Some(theme.color_required("primary")),
        "property" | "variable" => Some(theme.color_required("foreground")),
        "punctuation" => Some(theme.color_required("muted-foreground")),

        "string" => Some(theme.color_required("foreground")),
        "number" | "boolean" | "constant" => Some(theme.color_required("primary")),
        "type" | "constructor" | "function" => Some(theme.color_required("foreground")),
        _ => None,
    }
}

#[cfg(feature = "syntax")]
pub(super) fn materialize_row_rich_text(
    theme: &fret_ui::Theme,
    line: Arc<str>,
    spans: &[SyntaxSpan],
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
                ..Default::default()
            });
        }

        let fg = syntax_color(theme, span.highlight);
        out.push(TextSpan {
            len: end - start,
            shaping: Default::default(),
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
            ..Default::default()
        });
    }

    AttributedText::new(line, out)
}
