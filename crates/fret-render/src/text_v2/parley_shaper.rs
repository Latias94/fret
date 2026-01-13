use fret_core::{FontId, TextInput, TextShapingStyle, TextSlant, TextSpan, TextStyle};
use parley::FontContext;
use parley::Layout;
use parley::LayoutContext;
use parley::layout::PositionedLayoutItem;
use parley::style::{
    FontStyle, FontWeight as ParleyFontWeight, StyleProperty, TextStyle as ParleyTextStyle,
};
use std::borrow::Cow;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParleyGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
    pub style_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ShapedCluster {
    pub text_range: Range<usize>,
    pub x0: f32,
    pub x1: f32,
    pub is_rtl: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ShapedLineLayout {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub baseline: f32,
    pub glyphs: Vec<ParleyGlyph>,
    pub clusters: Vec<ShapedCluster>,
}

#[derive(Default)]
pub(crate) struct ParleyShaper {
    fcx: FontContext,
    lcx: LayoutContext<[u8; 4]>,
    layout: Layout<[u8; 4]>,
}

impl ParleyShaper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shape_single_line(&mut self, input: TextInput<'_>, scale: f32) -> ShapedLineLayout {
        let (text, base_style, spans) = match input {
            TextInput::Plain { text, style } => (text, style, &[][..]),
            TextInput::Attributed { text, base, spans } => (text, base, spans),
        };

        let root_style = ParleyTextStyle::default();
        let mut builder = self
            .lcx
            .tree_builder(&mut self.fcx, scale, true, &root_style);

        builder.push_style_span(base_parley_style(base_style));

        if let Some(span_ranges) = resolve_span_ranges(text, spans) {
            for (range, span) in span_ranges {
                let chunk = &text[range.clone()];
                if let Some(props) = shaping_properties_for_span(base_style, span) {
                    builder.push_style_modification_span(props.iter());
                    builder.push_text(chunk);
                    builder.pop_style_span();
                } else {
                    builder.push_text(chunk);
                }
            }
        } else {
            builder.push_text(text);
        }

        builder.pop_style_span();
        let _built_text = builder.build_into(&mut self.layout);
        self.layout.break_all_lines(None);

        let Some(line) = self.layout.lines().next() else {
            return ShapedLineLayout {
                width: 0.0,
                ascent: 0.0,
                descent: 0.0,
                baseline: 0.0,
                glyphs: Vec::new(),
                clusters: Vec::new(),
            };
        };

        let metrics = *line.metrics();

        let mut glyphs: Vec<ParleyGlyph> = Vec::new();
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            for g in run.positioned_glyphs() {
                glyphs.push(ParleyGlyph {
                    id: g.id,
                    x: g.x,
                    y: g.y,
                    advance: g.advance,
                    style_index: g.style_index(),
                });
            }
        }

        // Note: This ignores inline boxes; our current text surface doesn't emit them.
        let mut clusters: Vec<ShapedCluster> = Vec::new();
        let mut x = metrics.offset;
        for run in line.runs() {
            for cluster in run.visual_clusters() {
                let x0 = x;
                x += cluster.advance();
                clusters.push(ShapedCluster {
                    text_range: cluster.text_range(),
                    x0,
                    x1: x,
                    is_rtl: cluster.is_rtl(),
                });
            }
        }

        ShapedLineLayout {
            width: metrics.advance,
            ascent: metrics.ascent,
            descent: metrics.descent,
            baseline: metrics.baseline,
            glyphs,
            clusters,
        }
    }
}

fn resolve_span_ranges<'a>(
    text: &'a str,
    spans: &'a [TextSpan],
) -> Option<Vec<(Range<usize>, &'a TextSpan)>> {
    if spans.is_empty() {
        return None;
    }

    let mut out: Vec<(Range<usize>, &'a TextSpan)> = Vec::with_capacity(spans.len());
    let mut offset: usize = 0;

    for span in spans {
        let end = offset.saturating_add(span.len);
        if end > text.len() {
            return None;
        }
        if !text.is_char_boundary(offset) || !text.is_char_boundary(end) {
            return None;
        }
        if span.len != 0 {
            out.push((offset..end, span));
        }
        offset = end;
    }

    if offset != text.len() {
        return None;
    }

    Some(out)
}

fn base_parley_style(style: &TextStyle) -> ParleyTextStyle<'_, [u8; 4]> {
    let mut out = ParleyTextStyle::default();
    out.font_size = style.size.0;
    out.font_weight = ParleyFontWeight::new(style.weight.0 as f32);
    out.font_style = font_style_for_slant(style.slant);
    out.letter_spacing = style.letter_spacing_em.unwrap_or(0.0).clamp(-4.0, 4.0) * style.size.0;

    let stack = font_stack_for_font_id(&style.font);
    out.font_stack = parley::style::FontStack::Source(Cow::Owned(stack));

    out
}

fn font_stack_for_font_id(font: &FontId) -> String {
    match font {
        FontId::Ui => "sans-serif".to_string(),
        FontId::Serif => "serif".to_string(),
        FontId::Monospace => "monospace".to_string(),
        FontId::Family(name) => name.clone(),
    }
}

fn font_style_for_slant(slant: TextSlant) -> FontStyle {
    match slant {
        TextSlant::Normal => FontStyle::Normal,
        TextSlant::Italic => FontStyle::Italic,
        TextSlant::Oblique => FontStyle::Oblique(None),
    }
}

fn shaping_properties_for_span(
    base: &TextStyle,
    span: &TextSpan,
) -> Option<Vec<StyleProperty<'static, [u8; 4]>>> {
    let TextShapingStyle {
        font,
        weight,
        slant,
        letter_spacing_em,
    } = &span.shaping;

    let mut out: Vec<StyleProperty<'static, [u8; 4]>> = Vec::new();

    if let Some(font) = font {
        let stack = font_stack_for_font_id(font);
        out.push(StyleProperty::FontStack(parley::style::FontStack::Source(
            Cow::Owned(stack),
        )));
    }
    if let Some(weight) = weight {
        out.push(StyleProperty::FontWeight(ParleyFontWeight::new(
            weight.0 as f32,
        )));
    }
    if let Some(slant) = slant {
        out.push(StyleProperty::FontStyle(font_style_for_slant(*slant)));
    }
    if let Some(letter_spacing_em) = letter_spacing_em {
        out.push(StyleProperty::LetterSpacing(
            letter_spacing_em.clamp(-4.0, 4.0) * base.size.0,
        ));
    }

    (!out.is_empty()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Px;

    #[test]
    fn shapes_basic_single_line() {
        let mut shaper = ParleyShaper::new();
        let style = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };
        let input = TextInput::plain("hello", &style);

        let layout = shaper.shape_single_line(input, 1.0);
        assert!(layout.width >= 0.0);
        assert!(!layout.glyphs.is_empty());
        assert!(!layout.clusters.is_empty());
    }
}
