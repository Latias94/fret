use super::TextSystem;
use fret_core::{
    CaretAffinity, HitTestResult, Point, Rect, TextBlobId, TextInkMetrics, TextLineMetrics,
    geometry::Px,
};

impl TextSystem {
    pub fn caret_x(&self, blob: TextBlobId, index: usize) -> Option<Px> {
        let blob_id = blob;
        let blob = self.blobs.get(blob_id)?;
        if blob.shape.lines.len() > 1 {
            return Some(
                self.caret_rect(blob_id, index, CaretAffinity::Downstream)?
                    .origin
                    .x,
            );
        }
        let stops = blob.shape.caret_stops.as_ref();
        Some(fret_render_text::geometry::caret_x_from_stops(stops, index))
    }

    pub fn hit_test_x(&self, blob: TextBlobId, x: Px) -> Option<usize> {
        let blob_id = blob;
        let blob = self.blobs.get(blob_id)?;
        if blob.shape.lines.len() > 1 {
            return Some(self.hit_test_point(blob_id, Point::new(x, Px(0.0)))?.index);
        }
        let stops = blob.shape.caret_stops.as_ref();
        Some(fret_render_text::geometry::hit_test_x_from_stops(stops, x))
    }

    pub fn caret_stops(&self, blob: TextBlobId) -> Option<&[(usize, Px)]> {
        Some(self.blobs.get(blob)?.shape.caret_stops.as_ref())
    }

    pub fn caret_rect(
        &self,
        blob: TextBlobId,
        index: usize,
        affinity: CaretAffinity,
    ) -> Option<Rect> {
        let blob = self.blobs.get(blob)?;
        fret_render_text::geometry::caret_rect_from_lines(
            blob.shape.lines.as_ref(),
            index,
            affinity,
        )
    }

    pub fn hit_test_point(&self, blob: TextBlobId, point: Point) -> Option<HitTestResult> {
        let blob = self.blobs.get(blob)?;
        fret_render_text::geometry::hit_test_point_from_lines(blob.shape.lines.as_ref(), point)
    }

    pub fn selection_rects(
        &self,
        blob: TextBlobId,
        range: (usize, usize),
        out: &mut Vec<Rect>,
    ) -> Option<()> {
        let blob = self.blobs.get(blob)?;
        fret_render_text::geometry::selection_rects_from_lines(
            blob.shape.lines.as_ref(),
            range,
            out,
        );
        Some(())
    }

    pub fn selection_rects_clipped(
        &self,
        blob: TextBlobId,
        range: (usize, usize),
        clip: Rect,
        out: &mut Vec<Rect>,
    ) -> Option<()> {
        let blob = self.blobs.get(blob)?;
        fret_render_text::geometry::selection_rects_from_lines_clipped(
            blob.shape.lines.as_ref(),
            range,
            clip,
            out,
        );
        Some(())
    }

    pub fn first_line_metrics(&self, blob: TextBlobId) -> Option<TextLineMetrics> {
        let blob = self.blobs.get(blob)?;
        let line = blob.shape.lines.first()?;
        Some(TextLineMetrics {
            ascent: line.ascent,
            descent: line.descent,
            line_height: line.height,
        })
    }

    pub fn first_line_ink_metrics(&self, blob: TextBlobId) -> Option<TextInkMetrics> {
        let blob = self.blobs.get(blob)?;
        let line = blob.shape.lines.first()?;
        Some(TextInkMetrics {
            ascent: line.ink_ascent,
            descent: line.ink_descent,
        })
    }

    pub fn last_line_metrics(&self, blob: TextBlobId) -> Option<TextLineMetrics> {
        let blob = self.blobs.get(blob)?;
        let line = blob.shape.lines.last()?;
        Some(TextLineMetrics {
            ascent: line.ascent,
            descent: line.descent,
            line_height: line.height,
        })
    }

    pub fn last_line_ink_metrics(&self, blob: TextBlobId) -> Option<TextInkMetrics> {
        let blob = self.blobs.get(blob)?;
        let line = blob.shape.lines.last()?;
        Some(TextInkMetrics {
            ascent: line.ink_ascent,
            descent: line.ink_descent,
        })
    }
}
