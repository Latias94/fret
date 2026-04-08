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
    kind: TextDecorationKind,
    /// Rect in the same coordinate space as selection rects (y=0 at the top of the text box).
    rect: Rect,
    /// When present, uses `paint_palette[paint_span]` as the base color if no explicit override exists.
    paint_span: Option<u16>,
    /// Optional explicit decoration color override.
    color: Option<Color>,
}

impl TextDecoration {
    pub fn new(
        kind: TextDecorationKind,
        rect: Rect,
        paint_span: Option<u16>,
        color: Option<Color>,
    ) -> Self {
        Self {
            kind,
            rect,
            paint_span,
            color,
        }
    }

    pub fn kind(&self) -> TextDecorationKind {
        self.kind
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn paint_span(&self) -> Option<u16> {
        self.paint_span
    }

    pub fn color(&self) -> Option<Color> {
        self.color
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextDecorationMetricsPx {
    underline_offset_px: f32,
    strikeout_offset_px: f32,
    stroke_size_px: f32,
}

impl TextDecorationMetricsPx {
    pub fn new(underline_offset_px: f32, strikeout_offset_px: f32, stroke_size_px: f32) -> Self {
        Self {
            underline_offset_px,
            strikeout_offset_px,
            stroke_size_px,
        }
    }

    pub fn underline_offset_px(&self) -> f32 {
        self.underline_offset_px
    }

    pub fn strikeout_offset_px(&self) -> f32 {
        self.strikeout_offset_px
    }

    pub fn stroke_size_px(&self) -> f32 {
        self.stroke_size_px
    }
}

pub fn decoration_metrics_px_for_font_bytes(
    font_bytes: &[u8],
    face_index: u32,
    coords: &[i16],
    ppem: f32,
) -> Option<TextDecorationMetricsPx> {
    if !ppem.is_finite() || ppem <= 0.0 {
        return None;
    }

    let font_ref = parley::swash::FontRef::from_index(font_bytes, face_index as usize)?;
    let m = font_ref.metrics(coords).scale(ppem);
    if !m.underline_offset.is_finite()
        || !m.strikeout_offset.is_finite()
        || !m.stroke_size.is_finite()
    {
        return None;
    }

    Some(TextDecorationMetricsPx::new(
        m.underline_offset,
        m.strikeout_offset,
        m.stroke_size,
    ))
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
            let raw = m.stroke_size_px().abs().max(1.0).min(max_thickness_px);
            let thickness_px = if snap_vertical {
                raw.round().max(1.0)
            } else {
                raw
            };

            // Swash metrics are expressed in the typical typographic coordinate system where
            // positive Y points upward. Convert to our Y-down coordinate space by subtracting
            // from the baseline.
            let underline_top_px_raw = baseline_px - m.underline_offset_px();
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

            let strike_top_px_raw = baseline_px - m.strikeout_offset_px();
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
            if span.underline().is_none() && span.strikethrough().is_none() {
                continue;
            }

            let start = span.start().max(line.start());
            let end = span.end().min(line.end());
            if start >= end {
                continue;
            }

            let x0 = caret_x_from_stops(line.caret_stops(), start);
            let x1 = caret_x_from_stops(line.caret_stops(), end);
            let left = Px(x0.0.min(x1.0));
            let right = Px(x0.0.max(x1.0));
            let width = Px((right.0 - left.0).max(thickness.0));

            if let Some(underline) = span.underline() {
                out.push(TextDecoration::new(
                    TextDecorationKind::Underline,
                    Rect::new(Point::new(left, underline_y), Size::new(width, thickness)),
                    Some(span.slot()),
                    underline.color(),
                ));
            }

            if let Some(strikethrough) = span.strikethrough() {
                out.push(TextDecoration::new(
                    TextDecorationKind::Strikethrough,
                    Rect::new(Point::new(left, strike_y), Size::new(width, thickness)),
                    Some(span.slot()),
                    strikethrough.color(),
                ));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parley_shaper::ParleyShaper, prepare_layout, spans, wrapper};
    use fret_core::{
        DecorationLineStyle, FontId, Px, StrikethroughStyle, TextConstraints, TextInputRef,
        TextOverflow, TextPaintStyle, TextShapingStyle, TextSpan, TextStyle, TextWrap,
        UnderlineStyle,
    };

    fn shaper_with_bundled_fonts() -> ParleyShaper {
        let mut shaper = ParleyShaper::new_without_system_fonts();
        let added = shaper.add_fonts(fret_fonts::test_support::face_blobs(
            fret_fonts::bootstrap_profile()
                .faces
                .iter()
                .chain(fret_fonts_emoji::default_profile().faces.iter())
                .chain(fret_fonts_cjk::default_profile().faces.iter()),
        ));
        assert!(added > 0, "expected bundled fonts to load");
        shaper
    }

    #[test]
    fn decorations_are_pixel_snapped_under_non_integer_scale_factor() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = {
            let mut out = String::new();
            for _ in 0..60 {
                out.push_str("The quick brown fox jumps over the lazy dog. ");
            }
            out
        };

        let scale_factor = 1.25_f32;
        let constraints = TextConstraints {
            max_width: Some(Px(180.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor,
        };
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(13.0),
            ..Default::default()
        };

        let mut span = TextSpan {
            len: content.len(),
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle::default(),
        };
        span.paint.underline = Some(UnderlineStyle {
            color: None,
            style: DecorationLineStyle::Solid,
        });
        span.paint.strikethrough = Some(StrikethroughStyle {
            color: None,
            style: DecorationLineStyle::Solid,
        });

        let spans = [span];
        let resolved = spans::resolve_spans_for_text(content.as_str(), spans.as_slice())
            .expect("resolve spans");
        assert_eq!(resolved.len(), 1);

        let scale = crate::effective_text_scale_factor(scale_factor);
        let snap_vertical = scale.fract().abs() > 1e-4;
        assert!(
            snap_vertical,
            "expected fractional scale to enable snapping"
        );

        let wrapped = wrapper::wrap_with_constraints(
            &mut shaper,
            TextInputRef::attributed(content.as_str(), &style, spans.as_slice()),
            constraints,
        );
        let prepared = prepare_layout::prepare_layout_from_wrapped(
            content.as_str(),
            wrapped,
            constraints,
            scale,
            snap_vertical,
        );
        let lines: Vec<_> = prepared
            .lines()
            .iter()
            .map(|line| line.layout().clone())
            .collect();

        let ppem = style.size.0 * scale;
        let metrics_px = decoration_metrics_px_for_font_bytes(
            fret_fonts::bootstrap_profile()
                .faces
                .first()
                .map(|face| face.bytes)
                .expect("bootstrap font bytes"),
            0,
            &[],
            ppem,
        )
        .expect("decoration metrics");

        let decorations = decorations_for_lines(
            lines.as_slice(),
            resolved.as_slice(),
            Some(metrics_px),
            scale,
            snap_vertical,
        );

        let underlines: Vec<_> = decorations
            .iter()
            .filter(|d| d.kind() == TextDecorationKind::Underline)
            .collect();
        let strikes: Vec<_> = decorations
            .iter()
            .filter(|d| d.kind() == TextDecorationKind::Strikethrough)
            .collect();
        assert!(!underlines.is_empty(), "expected underline decorations");
        assert!(!strikes.is_empty(), "expected strikethrough decorations");

        let is_pixel_aligned = |logical: Px| {
            let px = logical.0 * scale_factor;
            (px - px.round()).abs() < 1e-3
        };

        for d in underlines.iter().chain(strikes.iter()) {
            let rect = d.rect();
            assert!(
                is_pixel_aligned(rect.origin.y),
                "expected decoration y to be pixel-aligned"
            );
            assert!(
                is_pixel_aligned(rect.size.height),
                "expected decoration height to be pixel-aligned"
            );

            let h_px = rect.size.height.0 * scale_factor;
            assert!(
                h_px >= 1.0 - 1e-3,
                "expected a visible decoration thickness (>= 1px), got {h_px}"
            );
            assert!(
                h_px <= 4.0 + 1e-3,
                "expected decoration thickness to remain bounded, got {h_px}"
            );

            assert!(
                rect.origin.y.0 >= -1e-3,
                "expected decoration to stay within the text box (top)"
            );
            assert!(
                rect.origin.y.0 + rect.size.height.0 <= prepared.metrics().size.height.0 + 1e-3,
                "expected decoration to stay within the text box (bottom)"
            );
        }
    }
}
