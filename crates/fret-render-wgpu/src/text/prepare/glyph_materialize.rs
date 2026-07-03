use super::super::{GlyphInstance, TextGlyphClusterBuilder, TextLine, TextSystem};
use fret_core::geometry::Px;
use fret_render_text::FontFaceKey;
use fret_render_text::{
    ParleyGlyph, PreparedLine, ResolvedSpan, TextLineCluster, paint_span_for_text_range,
};
use std::collections::HashMap;
use std::ops::Range;

impl TextSystem {
    pub(in super::super) fn materialize_prepared_line(
        &mut self,
        prepared_line: PreparedLine,
        resolved_spans: Option<&[ResolvedSpan]>,
        scale: f32,
        text_baseline: Px,
        glyphs: &mut Vec<GlyphInstance>,
        clusters: &mut Vec<TextGlyphClusterBuilder>,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
        lines: &mut Vec<TextLine>,
    ) {
        let (layout, prepared_glyphs) = prepared_line.into_parts();
        let line_index = lines.len().min(u32::MAX as usize) as u32;
        let cluster_base = clusters.len();
        self.materialize_prepared_line_glyphs(
            prepared_glyphs,
            &layout,
            resolved_spans,
            scale,
            text_baseline,
            glyphs,
            clusters,
            cluster_base,
            line_index,
            face_usage,
        );
        lines.push(layout);
    }

    fn materialize_prepared_line_glyphs(
        &mut self,
        prepared_glyphs: Vec<ParleyGlyph>,
        layout: &TextLine,
        resolved_spans: Option<&[ResolvedSpan]>,
        scale: f32,
        text_baseline: Px,
        glyphs: &mut Vec<GlyphInstance>,
        clusters: &mut Vec<TextGlyphClusterBuilder>,
        cluster_base: usize,
        line_index: u32,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
    ) {
        for glyph in prepared_glyphs {
            let cluster_index = cluster_index_for_prepared_glyph(
                &glyph,
                layout,
                clusters,
                cluster_base,
                line_index,
                scale,
                text_baseline,
            );
            let Some(instance) = self.materialize_prepared_line_glyph(
                &glyph,
                resolved_spans,
                scale,
                cluster_index.min(u32::MAX as usize) as u32,
                face_usage,
            ) else {
                continue;
            };
            let glyph_index = glyphs.len();
            if let Some(cluster) = clusters.get_mut(cluster_index) {
                cluster.record_glyph(glyph_index, &instance);
            }
            glyphs.push(instance);
        }
    }

    fn materialize_prepared_line_glyph(
        &mut self,
        glyph: &ParleyGlyph,
        resolved_spans: Option<&[ResolvedSpan]>,
        scale: f32,
        cluster_index: u32,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
    ) -> Option<GlyphInstance> {
        let context = self.prepare_prepared_glyph_context(glyph, face_usage)?;
        let (x, x_bin, y, y_bin) = prepared_glyph_origin_bins(glyph);
        let paint_span = prepared_glyph_paint_span(resolved_spans, glyph);
        let (glyph_key, x0_px, y0_px, w_px, h_px) = self.resolve_prepared_glyph_bounds(
            glyph,
            context.glyph_id,
            context.face_key,
            context.size_bits,
            x_bin,
            y_bin,
            x,
            y,
        )?;
        Some(prepared_glyph_instance(
            glyph_key,
            x0_px,
            y0_px,
            w_px,
            h_px,
            paint_span,
            cluster_index,
            scale,
        ))
    }
}

fn prepared_glyph_paint_span(
    resolved_spans: Option<&[ResolvedSpan]>,
    glyph: &ParleyGlyph,
) -> Option<u16> {
    resolved_spans
        .and_then(|spans| paint_span_for_text_range(spans, &glyph.text_range(), glyph.is_rtl()))
}

fn prepared_glyph_instance(
    glyph_key: super::super::atlas::GlyphKey,
    x0_px: f32,
    y0_px: f32,
    w_px: f32,
    h_px: f32,
    paint_span: Option<u16>,
    cluster_index: u32,
    scale: f32,
) -> GlyphInstance {
    GlyphInstance::new(
        [x0_px / scale, y0_px / scale, w_px / scale, h_px / scale],
        paint_span,
        cluster_index,
        glyph_key,
    )
}

fn prepared_glyph_origin_bins(glyph: &ParleyGlyph) -> (i32, u8, i32, u8) {
    let (x, x_bin) = super::super::atlas::subpixel_bin_q4(glyph.x());
    let (y, y_bin) = super::super::atlas::subpixel_bin_y(glyph.y());
    (x, x_bin, y, y_bin)
}

fn line_cluster_visual_bounds(
    line: &TextLine,
    cluster: &TextLineCluster,
    text_baseline: Px,
) -> [f32; 4] {
    let x0 = cluster.x0().0.min(cluster.x1().0);
    let x1 = cluster.x0().0.max(cluster.x1().0);
    let y0 = line.y_top().0 - text_baseline.0;
    [x0, y0, (x1 - x0).max(0.0), line.height().0.max(0.0)]
}

fn cluster_index_for_prepared_glyph(
    glyph: &ParleyGlyph,
    layout: &TextLine,
    clusters: &mut Vec<TextGlyphClusterBuilder>,
    cluster_base: usize,
    line_index: u32,
    scale: f32,
    text_baseline: Px,
) -> usize {
    let glyph_range = glyph_cluster_text_range(layout, glyph, scale);
    if let Some(index) = clusters
        .iter()
        .enumerate()
        .skip(cluster_base)
        .find_map(|(index, cluster)| (glyph_range == cluster.text_range()).then_some(index))
    {
        return index;
    }

    clusters.push(TextGlyphClusterBuilder::new(
        line_index,
        glyph_range.clone(),
        glyph_cluster_visual_bounds(layout, &glyph_range, scale, text_baseline, glyph),
        glyph.is_rtl(),
    ));
    clusters.len().saturating_sub(1)
}

fn glyph_cluster_text_range(line: &TextLine, glyph: &ParleyGlyph, scale: f32) -> Range<usize> {
    let mut range = glyph.text_range();
    let x0 = glyph.x() / scale;
    let x1 = (glyph.x() + glyph.advance().max(0.0)) / scale;
    if !x0.is_finite() || !x1.is_finite() {
        return range;
    }

    for cluster in line.clusters() {
        if cluster.is_rtl() != glyph.is_rtl() {
            continue;
        }
        let cx0 = cluster.x0().0.min(cluster.x1().0);
        let cx1 = cluster.x0().0.max(cluster.x1().0);
        let overlaps = if x1 > x0 {
            cx0 < x1 && x0 < cx1
        } else {
            x0 >= cx0 && x0 <= cx1
        };
        if overlaps {
            let cluster_range = cluster.text_range();
            range.start = range.start.min(cluster_range.start);
            range.end = range.end.max(cluster_range.end);
        }
    }
    range
}

fn glyph_cluster_visual_bounds(
    line: &TextLine,
    glyph_range: &Range<usize>,
    scale: f32,
    text_baseline: Px,
    glyph: &ParleyGlyph,
) -> [f32; 4] {
    let mut bounds: Option<[f32; 4]> = None;
    for cluster in line.clusters() {
        if !ranges_intersect_or_touch_empty(glyph_range, &cluster.text_range()) {
            continue;
        }
        let cluster_bounds = line_cluster_visual_bounds(line, cluster, text_baseline);
        bounds = Some(match bounds {
            Some(existing) => union_rects(existing, cluster_bounds),
            None => cluster_bounds,
        });
    }
    bounds.unwrap_or_else(|| {
        [
            glyph.x() / scale,
            -text_baseline.0,
            0.0,
            line.height().0.max(0.0),
        ]
    })
}

fn ranges_intersect_or_touch_empty(a: &Range<usize>, b: &Range<usize>) -> bool {
    if a.start == a.end {
        return a.start >= b.start && a.start <= b.end;
    }
    if b.start == b.end {
        return b.start >= a.start && b.start <= a.end;
    }
    a.start < b.end && b.start < a.end
}

fn union_rects(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let ax1 = a[0] + a[2];
    let ay1 = a[1] + a[3];
    let bx1 = b[0] + b[2];
    let by1 = b[1] + b[3];
    let x0 = a[0].min(b[0]);
    let y0 = a[1].min(b[1]);
    let x1 = ax1.max(bx1);
    let y1 = ay1.max(by1);
    [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)]
}
