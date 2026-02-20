use crate::geometry::{TextLineDecorationGeometry, caret_x_from_stops};
use crate::spans::ResolvedSpan;
use fret_core::{Color, Point, Rect, Size, geometry::Px};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecorationKind {
    Underline,
    Strikethrough,
}

#[derive(Debug, Clone)]
pub struct TextDecoration {
    pub kind: TextDecorationKind,
    /// Rect in the same coordinate space as selection rects (y=0 at the top of the text box).
    pub rect: Rect,
    /// When present, uses `paint_palette[paint_span]` as the base color if no explicit override exists.
    pub paint_span: Option<u16>,
    /// Optional explicit decoration color override.
    pub color: Option<Color>,
}

#[derive(Debug, Clone, Copy)]
pub struct TextDecorationMetricsPx {
    pub underline_offset_px: f32,
    pub strikeout_offset_px: f32,
    pub stroke_size_px: f32,
}

pub fn decorations_for_lines<L: TextLineDecorationGeometry>(
    lines: &[L],
    spans: &[ResolvedSpan],
    metrics_px: Option<TextDecorationMetricsPx>,
    scale: f32,
    snap_vertical: bool,
) -> Vec<TextDecoration> {
    let mut out: Vec<TextDecoration> = Vec::new();
    if lines.is_empty() || spans.is_empty() {
        return out;
    }
    if !scale.is_finite() || scale <= 0.0 {
        return out;
    }

    for line in lines {
        let y_top = line.y_top().0;
        let height = line.height().0.max(0.0);
        let baseline = line.y_baseline().0;

        let line_top_px = y_top * scale;
        let line_bottom_px = (y_top + height).max(y_top) * scale;
        let baseline_px = baseline * scale;

        let line_height_px = (height * scale).max(0.0);
        let max_thickness_px = line_height_px.max(1.0);

        let (thickness_px, underline_y, strike_y) = if let Some(m) = metrics_px {
            let raw = m.stroke_size_px.abs().max(1.0).min(max_thickness_px);
            let thickness_px = if snap_vertical {
                raw.round().max(1.0)
            } else {
                raw
            };

            // Swash metrics are expressed in the typical typographic coordinate system where
            // positive Y points upward. Convert to our Y-down coordinate space by subtracting
            // from the baseline.
            let underline_top_px_raw = baseline_px - m.underline_offset_px;
            let underline_bottom_px_raw = underline_top_px_raw + thickness_px;
            let underline_bottom_px = if snap_vertical {
                underline_bottom_px_raw.round()
            } else {
                underline_bottom_px_raw
            }
            .clamp(line_top_px, line_bottom_px);
            let max_top_px = (line_bottom_px - thickness_px).max(line_top_px);
            let underline_top_px =
                (underline_bottom_px - thickness_px).clamp(line_top_px, max_top_px);

            let strike_top_px_raw = baseline_px - m.strikeout_offset_px;
            let strike_bottom_px_raw = strike_top_px_raw + thickness_px;
            let strike_bottom_px = if snap_vertical {
                strike_bottom_px_raw.round()
            } else {
                strike_bottom_px_raw
            }
            .clamp(line_top_px, line_bottom_px);
            let strike_top_px = (strike_bottom_px - thickness_px).clamp(line_top_px, max_top_px);

            (
                thickness_px,
                Px((underline_top_px / scale).max(0.0)),
                Px((strike_top_px / scale).max(0.0)),
            )
        } else {
            let thickness_px = 1.0_f32;

            // Underline: anchor to the baseline and snap in device px under fractional scaling.
            let underline_bottom_px_raw = baseline_px + 1.0;
            let underline_bottom_px = if snap_vertical {
                underline_bottom_px_raw.round()
            } else {
                underline_bottom_px_raw
            }
            .clamp(line_top_px, line_bottom_px);
            let max_top_px = (line_bottom_px - thickness_px).max(line_top_px);
            let underline_top_px =
                (underline_bottom_px - thickness_px).clamp(line_top_px, max_top_px);
            let underline_y = Px((underline_top_px / scale).max(0.0));

            // Strikethrough: approximate as a fraction of the line height above the baseline.
            let strike_offset_px_raw = (line_height_px * 0.30).clamp(1.0, line_height_px);
            let strike_bottom_px_raw = baseline_px - strike_offset_px_raw;
            let strike_bottom_px = if snap_vertical {
                strike_bottom_px_raw.round()
            } else {
                strike_bottom_px_raw
            }
            .clamp(line_top_px, line_bottom_px);
            let strike_top_px = (strike_bottom_px - thickness_px).clamp(line_top_px, max_top_px);
            let strike_y = Px((strike_top_px / scale).max(0.0));

            (thickness_px, underline_y, strike_y)
        };

        let thickness = Px((thickness_px / scale).max(0.0));

        for span in spans {
            if span.underline.is_none() && span.strikethrough.is_none() {
                continue;
            }

            let start = span.start.max(line.start());
            let end = span.end.min(line.end());
            if start >= end {
                continue;
            }

            let x0 = caret_x_from_stops(line.caret_stops(), start);
            let x1 = caret_x_from_stops(line.caret_stops(), end);
            let left = Px(x0.0.min(x1.0));
            let right = Px(x0.0.max(x1.0));
            let width = Px((right.0 - left.0).max(thickness.0));

            if let Some(underline) = span.underline.as_ref() {
                out.push(TextDecoration {
                    kind: TextDecorationKind::Underline,
                    rect: Rect::new(Point::new(left, underline_y), Size::new(width, thickness)),
                    paint_span: Some(span.slot),
                    color: underline.color,
                });
            }

            if let Some(strikethrough) = span.strikethrough.as_ref() {
                out.push(TextDecoration {
                    kind: TextDecorationKind::Strikethrough,
                    rect: Rect::new(Point::new(left, strike_y), Size::new(width, thickness)),
                    paint_span: Some(span.slot),
                    color: strikethrough.color,
                });
            }
        }
    }

    out
}
