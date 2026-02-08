//! Geometry helpers for caret/selection/pointer hit-testing.

use std::ops::Range;

use fret_code_editor_buffer::TextBuffer;
use fret_code_editor_view::DisplayPoint;
use fret_core::{Px, Rect, Size, TextBlobId};

use super::{CodeEditorState, PreeditState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RowPreeditMapping {
    /// Byte offset within the row slice where preedit is injected.
    pub(super) insert_at: usize,
    /// UTF-8 byte length of the injected preedit text.
    pub(super) preedit_len: usize,
}

#[derive(Debug, Clone)]
pub(super) struct RowGeom {
    /// Display-row range within the buffer (UTF-8 byte indices).
    pub(super) row_range: Range<usize>,
    /// Prepared text blob that backs `caret_stops` and caret metrics.
    pub(super) blob: TextBlobId,
    /// Caret stop table for the displayed row text (byte index -> x offset).
    pub(super) caret_stops: Vec<(usize, Px)>,
    /// Optional mapping between buffer-local and display-local indices when the row materializes
    /// fold placeholders (ADR 0200).
    pub(super) fold_map: Option<RowFoldMap>,
    /// Optional caret rectangle vertical metrics derived from the renderer text system.
    ///
    /// Coordinate space: relative to the row text origin (y=0 at the top of the row text box).
    pub(super) caret_rect_top: Option<Px>,
    pub(super) caret_rect_height: Option<Px>,
    /// Mapping needed when the displayed row includes an injected preedit string.
    pub(super) preedit: Option<RowPreeditMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RowFoldSpan {
    pub(super) buffer_range: Range<usize>,
    pub(super) display_range: Range<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RowFoldMap {
    spans: Vec<RowFoldSpan>,
}

impl RowFoldMap {
    pub(super) fn new(spans: Vec<RowFoldSpan>) -> Self {
        Self { spans }
    }

    pub(super) fn buffer_local_to_display_local(&self, buffer_local: usize) -> usize {
        let mut removed = 0i64;
        let mut added = 0i64;

        for span in &self.spans {
            let start = span.buffer_range.start;
            let end = span.buffer_range.end.max(start);
            if buffer_local <= start {
                break;
            }
            if buffer_local < end {
                return span.display_range.start;
            }
            removed += end.saturating_sub(start) as i64;
            added += span.display_range.len() as i64;
        }

        let base = buffer_local as i64 + added - removed;
        base.max(0) as usize
    }

    pub(super) fn display_local_to_buffer_local(&self, display_local: usize) -> usize {
        let mut removed = 0i64;
        let mut added = 0i64;

        for span in &self.spans {
            let start = span.display_range.start;
            let end = span.display_range.end.max(start);
            if display_local < start {
                break;
            }
            if display_local < end {
                return span.buffer_range.start;
            }
            removed += span.buffer_range.len() as i64;
            added += end.saturating_sub(start) as i64;
        }

        let base = display_local as i64 + removed - added;
        base.max(0) as usize
    }
}

pub(super) fn caret_x_for_index(stops: &[(usize, Px)], index: usize) -> Px {
    if stops.is_empty() {
        return Px(0.0);
    }
    // Prefer an exact index match, otherwise clamp to the nearest representable caret stop.
    let mut lo = 0usize;
    let mut hi = stops.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if stops[mid].0 < index {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo < stops.len() && stops[lo].0 == index {
        return stops[lo].1;
    }
    if lo == 0 {
        return stops[0].1;
    }
    stops[lo.saturating_sub(1)].1
}

pub(super) fn hit_test_index_from_caret_stops(stops: &[(usize, Px)], x: Px) -> usize {
    if stops.is_empty() {
        return 0;
    }
    if stops.len() == 1 {
        return stops[0].0;
    }
    let x = x.0;

    let mut non_decreasing = true;
    let mut non_increasing = true;
    for pair in stops.windows(2) {
        let a = pair[0].1.0;
        let b = pair[1].1.0;
        if a > b {
            non_decreasing = false;
        }
        if a < b {
            non_increasing = false;
        }
    }

    if non_decreasing {
        if x <= stops[0].1.0 {
            return stops[0].0;
        }
        if x >= stops[stops.len() - 1].1.0 {
            return stops[stops.len() - 1].0;
        }
        let mut lo = 0usize;
        let mut hi = stops.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if stops[mid].1.0 < x {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        let right = lo.min(stops.len() - 1);
        let left = right.saturating_sub(1);
        let (li, lx) = stops[left];
        let (ri, rx) = stops[right];
        let ld = (x - lx.0).abs();
        let rd = (rx.0 - x).abs();
        if ld < rd || (ld == rd && li <= ri) {
            li
        } else {
            ri
        }
    } else if non_increasing {
        // Pure RTL runs can produce caret stops that are monotonically decreasing in X.
        if x >= stops[0].1.0 {
            return stops[0].0;
        }
        if x <= stops[stops.len() - 1].1.0 {
            return stops[stops.len() - 1].0;
        }
        let mut lo = 0usize;
        let mut hi = stops.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if stops[mid].1.0 > x {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        let right = lo.min(stops.len() - 1);
        let left = right.saturating_sub(1);
        let (li, lx) = stops[left];
        let (ri, rx) = stops[right];
        let ld = (lx.0 - x).abs();
        let rd = (x - rx.0).abs();
        if ld < rd || (ld == rd && li <= ri) {
            li
        } else {
            ri
        }
    } else {
        // Mixed-direction lines can yield non-monotonic caret stops (e.g. bidi). Fall back to the
        // nearest X distance to keep pointer hit-testing stable.
        let mut best = stops[0];
        let mut best_dist = (best.1.0 - x).abs();
        for stop in &stops[1..] {
            let dist = (stop.1.0 - x).abs();
            if dist < best_dist || (dist == best_dist && stop.0 < best.0) {
                best = *stop;
                best_dist = dist;
            }
        }
        best.0
    }
}

pub(super) fn map_row_local_to_buffer_byte(
    buf: &TextBuffer,
    geom: &RowGeom,
    local: usize,
) -> usize {
    let row_start = geom.row_range.start.min(buf.len_bytes());
    let row_end = geom.row_range.end.min(buf.len_bytes()).max(row_start);
    let max_local = row_end.saturating_sub(row_start);

    let mut local = local;
    if let Some(preedit) = geom.preedit {
        let insert_at = preedit.insert_at.min(max_local);
        let preedit_len = preedit.preedit_len;
        if local <= insert_at {
            local = local.min(insert_at);
            return row_start.saturating_add(local).min(row_end);
        }
        let after_insert = insert_at.saturating_add(preedit_len);
        if local >= after_insert {
            let base_local = local.saturating_sub(preedit_len);
            return row_start
                .saturating_add(base_local.min(max_local))
                .min(row_end);
        }
        // Inside the injected preedit: snap to the injection point in the base buffer.
        return row_start.saturating_add(insert_at).min(row_end);
    }

    row_start.saturating_add(local.min(max_local)).min(row_end)
}

pub(super) fn caret_for_pointer(
    st: &mut CodeEditorState,
    row: usize,
    bounds: Rect,
    position: fret_core::Point,
    cell_w: Px,
) -> usize {
    let local_x = Px(position.x.0 - bounds.origin.x.0);
    if let Some((geom, _)) = st.row_geom_cache.get(&row)
        && !geom.caret_stops.is_empty()
        && geom.preedit.is_some() == st.preedit.is_some()
    {
        let local = hit_test_index_from_caret_stops(&geom.caret_stops, local_x);
        let local = geom
            .fold_map
            .as_ref()
            .map(|m| m.display_local_to_buffer_local(local))
            .unwrap_or(local);
        let byte = map_row_local_to_buffer_byte(&st.buffer, geom, local);
        return st
            .buffer
            .clamp_to_char_boundary_left(byte.min(st.buffer.len_bytes()));
    }

    // Fallback to the MVP monospace heuristic when geometry hasn't been cached yet.
    st.cache_stats.geom_pointer_hit_test_fallbacks = st
        .cache_stats
        .geom_pointer_hit_test_fallbacks
        .saturating_add(1);
    let col = if cell_w.0 > 0.0 {
        (local_x.0 / cell_w.0).floor().max(0.0) as usize
    } else {
        0
    };
    st.display_map
        .display_point_to_byte(&st.buffer, DisplayPoint::new(row, col))
}

pub(super) fn caret_rect_for_selection(
    st: &mut CodeEditorState,
    row_h: Px,
    cell_w: Px,
    bounds: Rect,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) -> Option<Rect> {
    if !st.selection.is_caret() {
        return None;
    }

    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(st.buffer.len_bytes()));
    let pt = st.display_map.byte_to_display_point(&st.buffer, caret);
    let offset = scroll_handle.offset();
    let row_y = Px(bounds.origin.y.0 + (pt.row as f32 * row_h.0) - offset.y.0);

    let mut caret_top = Px(0.0);
    let mut caret_h = row_h;
    let mut x = None::<Px>;
    if let Some((geom, _)) = st.row_geom_cache.get(&pt.row) {
        if let (Some(top), Some(h)) = (geom.caret_rect_top, geom.caret_rect_height)
            && h.0 > 0.0
        {
            caret_top = top;
            caret_h = h;
        }

        if !geom.caret_stops.is_empty()
            && caret >= geom.row_range.start
            && geom.preedit.is_some() == st.preedit.is_some()
        {
            let mut local = caret.saturating_sub(geom.row_range.start);
            if let Some(folds) = geom.fold_map.as_ref() {
                local = folds.buffer_local_to_display_local(local);
            }
            if let Some(preedit) = st.preedit.as_ref()
                && geom.preedit.is_some()
            {
                local = local.saturating_add(preedit_cursor_offset_bytes(preedit));
            }
            let cx = caret_x_for_index(&geom.caret_stops, local);
            x = Some(Px(bounds.origin.x.0 + cx.0));
        }
    }

    let x = x.unwrap_or_else(|| {
        st.cache_stats.geom_caret_rect_fallbacks =
            st.cache_stats.geom_caret_rect_fallbacks.saturating_add(1);
        let mut col = pt.col;
        if let Some(preedit) = st.preedit.as_ref() {
            col = col.saturating_add(preedit_cursor_offset_cols(preedit));
        }
        Px(bounds.origin.x.0 + col as f32 * cell_w.0)
    });
    let y = Px(row_y.0 + caret_top.0);

    Some(Rect::new(
        fret_core::Point::new(x, y),
        Size::new(Px(1.0), caret_h),
    ))
}

pub(super) fn preedit_cursor_offset_cols(preedit: &PreeditState) -> usize {
    let mut end = preedit
        .cursor
        .map(|(_, end)| end)
        .unwrap_or_else(|| preedit.text.len());
    end = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, end).min(preedit.text.len());
    preedit.text[..end].chars().count()
}

pub(super) fn preedit_cursor_offset_bytes(preedit: &PreeditState) -> usize {
    let mut end = preedit
        .cursor
        .map(|(_, end)| end)
        .unwrap_or_else(|| preedit.text.len());
    end = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, end).min(preedit.text.len());
    end
}

pub(super) fn caret_x_for_buffer_byte_in_row(
    st: &CodeEditorState,
    row: usize,
    caret: usize,
) -> Option<Px> {
    let (geom, _) = st.row_geom_cache.get(&row)?;
    if geom.caret_stops.is_empty()
        || caret < geom.row_range.start
        || geom.preedit.is_some() != st.preedit.is_some()
    {
        return None;
    }

    let mut local = caret.saturating_sub(geom.row_range.start);
    if let Some(folds) = geom.fold_map.as_ref() {
        local = folds.buffer_local_to_display_local(local);
    }
    if let Some(preedit) = st.preedit.as_ref()
        && geom.preedit.is_some()
    {
        local = local.saturating_add(preedit_cursor_offset_bytes(preedit));
    }
    Some(caret_x_for_index(&geom.caret_stops, local))
}
