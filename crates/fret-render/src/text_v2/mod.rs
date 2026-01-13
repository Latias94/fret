use fret_core::{FontId, TextInput, TextShapingStyle, TextSlant, TextSpan, TextStyle};
use parley::FontContext;
use parley::Layout;
use parley::LayoutContext;
use parley::layout::PositionedLayoutItem;
use parley::style::{
    FontStyle, FontWeight as ParleyFontWeight, StyleProperty, TextStyle as ParleyTextStyle,
};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParleyGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
    pub style_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParleyLineLayout {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub baseline: f32,
    pub glyphs: Vec<ParleyGlyph>,
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

    pub fn shape_single_line(&mut self, input: TextInput<'_>, scale: f32) -> ParleyLineLayout {
        let (text, base_style, spans) = match input {
            TextInput::Plain { text, style } => (text, style, &[][..]),
            TextInput::Attributed { text, base, spans } => (text, base, spans),
        };

        let root_style = ParleyTextStyle::default();
        let mut builder = self
            .lcx
            .tree_builder(&mut self.fcx, scale, true, &root_style);

        builder.push_style_span(base_parley_style(base_style));

        if spans.is_empty() {
            builder.push_text(text);
        } else {
            let mut offset = 0usize;
            for span in spans {
                let end = offset.saturating_add(span.len);
                if end > text.len() {
                    break;
                }
                let Some(chunk) = text.get(offset..end) else {
                    break;
                };

                if let Some(props) = shaping_properties_for_span(base_style, span) {
                    builder.push_style_modification_span(props.iter());
                    builder.push_text(chunk);
                    builder.pop_style_span();
                } else {
                    builder.push_text(chunk);
                }

                offset = end;
            }

            if offset < text.len()
                && let Some(rest) = text.get(offset..text.len())
            {
                builder.push_text(rest);
            }
        }

        builder.pop_style_span();
        let _built_text = builder.build_into(&mut self.layout);
        self.layout.break_all_lines(None);

        let first_line = self.layout.lines().next();
        let Some(line) = first_line else {
            return ParleyLineLayout {
                width: 0.0,
                ascent: 0.0,
                descent: 0.0,
                baseline: 0.0,
                glyphs: Vec::new(),
            };
        };

        let metrics = line.metrics();
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

        ParleyLineLayout {
            width: metrics.advance,
            ascent: metrics.ascent,
            descent: metrics.descent,
            baseline: metrics.baseline,
            glyphs,
        }
    }
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
    }
}
