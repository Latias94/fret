//! Rich text materialization, syntax row-rich cache, and prefetch ownership.

use std::ops::Range;
use std::sync::Arc;

#[cfg(feature = "syntax")]
use crate::editor::syntax::{
    SYNTAX_PREFETCH_AHEAD_ROWS, SyntaxSpan, ensure_syntax_row_cache_fresh,
};

use super::*;

#[cfg(feature = "syntax")]
pub(super) fn normalize_syntax_spans_for_text(text: &str, spans: &mut Vec<SyntaxSpan>) {
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
pub(super) fn mapped_row_syntax_spans_for_rich_text(
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
pub(super) fn store_row_rich_cache_entry(
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

#[cfg(feature = "syntax")]
const ROW_RICH_PREFETCH_EDGE_ROWS: usize = 8;

#[cfg(feature = "syntax")]
fn push_unique_row(rows: &mut Vec<usize>, row: usize) {
    if !rows.contains(&row) {
        rows.push(row);
    }
}

#[cfg(feature = "syntax")]
pub(super) fn arc_str_ptr_or_content_eq(a: &Arc<str>, b: &Arc<str>) -> bool {
    Arc::ptr_eq(a, b) || a.as_ref() == b.as_ref()
}

#[cfg(feature = "syntax")]
pub(super) fn arc_slice_ptr_or_content_eq<T: PartialEq>(a: &Arc<[T]>, b: &Arc<[T]>) -> bool {
    Arc::ptr_eq(a, b) || a.as_ref() == b.as_ref()
}

#[cfg(feature = "syntax")]
pub(super) fn row_rich_prefetch_candidate_rows(
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

pub(in crate::editor) fn materialize_preedit_rich_text(
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

pub(in crate::editor) fn materialize_preedit_rich_text_for_range(
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
pub(in crate::editor) fn schedule_row_rich_prefetch_for_frame(
    st: &mut CodeEditorState,
    frame: WindowedRowsPaintFrame,
    max_entries: usize,
    window: fret_core::AppWindowId,
    theme: fret_ui::Theme,
) {
    let max_entries = frame_cache_max_entries(st, max_entries);
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
pub(super) fn materialize_row_rich_text_with_fg(
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
