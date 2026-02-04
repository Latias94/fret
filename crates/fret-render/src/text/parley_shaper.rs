use fret_core::{FontId, TextInputRef, TextShapingStyle, TextSlant, TextSpan, TextStyle};
use parley::FontContext;
use parley::FontData;
use parley::Layout;
use parley::LayoutContext;
use parley::style::{
    FontStyle, FontWeight as ParleyFontWeight, StyleProperty, TextStyle as ParleyTextStyle,
};
use std::borrow::Cow;
use std::ops::Range;

fn min_line_height_for_metrics(ascent: f32, descent: f32) -> f32 {
    let ascent = ascent.max(0.0);
    let descent_mag = if descent.is_sign_negative() {
        (-descent).max(0.0)
    } else {
        descent.max(0.0)
    };
    ascent + descent_mag
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParleyGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
    pub font: FontData,
    pub font_size: f32,
    pub text_range: Range<usize>,
    pub is_rtl: bool,
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
    pub line_height: f32,
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

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_without_system_fonts() -> Self {
        let mut out = Self::default();
        out.fcx.collection =
            parley::fontique::Collection::new(parley::fontique::CollectionOptions {
                shared: false,
                system_fonts: false,
            });
        out.fcx.source_cache = parley::fontique::SourceCache::default();
        out
    }

    pub fn add_fonts(&mut self, fonts: impl IntoIterator<Item = Vec<u8>>) -> usize {
        let mut added = 0usize;
        for data in fonts {
            let blob = parley::fontique::Blob::<u8>::from(data);
            let families = self.fcx.collection.register_fonts(blob, None);
            added = added.saturating_add(families.iter().map(|(_, fonts)| fonts.len()).sum());
        }
        added
    }

    pub fn set_generic_family_name(
        &mut self,
        generic: parley::fontique::GenericFamily,
        family_name: &str,
    ) -> bool {
        let Some(id) = self.fcx.collection.family_id(family_name) else {
            return false;
        };

        let before = self.fcx.collection.generic_families(generic).next();
        if before == Some(id) {
            return false;
        }

        self.fcx
            .collection
            .set_generic_families(generic, std::iter::once(id));
        true
    }

    pub fn append_generic_family_name(
        &mut self,
        generic: parley::fontique::GenericFamily,
        family_name: &str,
    ) -> bool {
        let Some(id) = self.fcx.collection.family_id(family_name) else {
            return false;
        };

        if self
            .fcx
            .collection
            .generic_families(generic)
            .any(|existing| existing == id)
        {
            return false;
        }

        self.fcx
            .collection
            .append_generic_families(generic, std::iter::once(id));
        true
    }

    pub fn shape_single_line(&mut self, input: TextInputRef<'_>, scale: f32) -> ShapedLineLayout {
        let (text, base_style, spans) = match input {
            TextInputRef::Plain { text, style } => (text, style, &[][..]),
            TextInputRef::Attributed { text, base, spans } => (text, base, spans),
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
            if text.is_empty() {
                let fallback = self.shape_single_line(TextInputRef::plain(" ", base_style), scale);
                return ShapedLineLayout {
                    width: 0.0,
                    ascent: fallback.ascent,
                    descent: fallback.descent,
                    baseline: fallback.baseline,
                    line_height: fallback.line_height,
                    glyphs: Vec::new(),
                    clusters: Vec::new(),
                };
            }
            return ShapedLineLayout {
                width: 0.0,
                ascent: 0.0,
                descent: 0.0,
                baseline: 0.0,
                line_height: 0.0,
                glyphs: Vec::new(),
                clusters: Vec::new(),
            };
        };

        let metrics = *line.metrics();
        let mut line_height = metrics.line_height.max(0.0);
        line_height = line_height.max(min_line_height_for_metrics(metrics.ascent, metrics.descent));
        if let Some(requested) = base_style.line_height {
            line_height = line_height.max((requested.0 * scale).max(0.0));
        }

        let mut glyphs: Vec<ParleyGlyph> = Vec::new();
        let mut clusters: Vec<ShapedCluster> = Vec::new();

        // Note: This ignores inline boxes; our current text surface doesn't emit them.
        let mut run_x = metrics.offset;
        for run in line.runs() {
            let font = run.font();
            let font_data = font.clone();
            let font_size = run.font_size();

            for cluster in run.visual_clusters() {
                let cluster_range = cluster.text_range();
                let cluster_x0 = run_x;

                let mut glyph_x = cluster_x0;
                for mut g in cluster.glyphs() {
                    g.x += glyph_x;
                    glyph_x += g.advance;

                    glyphs.push(ParleyGlyph {
                        id: g.id,
                        x: g.x,
                        y: g.y,
                        advance: g.advance,
                        font: font_data.clone(),
                        font_size,
                        text_range: cluster_range.clone(),
                        is_rtl: cluster.is_rtl(),
                    });
                }

                run_x = cluster_x0 + cluster.advance();
                clusters.push(ShapedCluster {
                    text_range: cluster_range,
                    x0: cluster_x0,
                    x1: run_x,
                    is_rtl: cluster.is_rtl(),
                });
            }
        }

        ShapedLineLayout {
            width: metrics.advance,
            ascent: metrics.ascent,
            descent: metrics.descent,
            baseline: metrics.baseline,
            line_height,
            glyphs,
            clusters,
        }
    }

    pub fn shape_single_line_metrics(
        &mut self,
        input: TextInputRef<'_>,
        scale: f32,
    ) -> ShapedLineLayout {
        let (text, base_style, spans) = match input {
            TextInputRef::Plain { text, style } => (text, style, &[][..]),
            TextInputRef::Attributed { text, base, spans } => (text, base, spans),
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
            if text.is_empty() {
                let fallback =
                    self.shape_single_line_metrics(TextInputRef::plain(" ", base_style), scale);
                return ShapedLineLayout {
                    width: 0.0,
                    ascent: fallback.ascent,
                    descent: fallback.descent,
                    baseline: fallback.baseline,
                    line_height: fallback.line_height,
                    glyphs: Vec::new(),
                    clusters: Vec::new(),
                };
            }
            return ShapedLineLayout {
                width: 0.0,
                ascent: 0.0,
                descent: 0.0,
                baseline: 0.0,
                line_height: 0.0,
                glyphs: Vec::new(),
                clusters: Vec::new(),
            };
        };

        let metrics = *line.metrics();
        let mut line_height = metrics.line_height.max(0.0);
        line_height = line_height.max(min_line_height_for_metrics(metrics.ascent, metrics.descent));
        if let Some(requested) = base_style.line_height {
            line_height = line_height.max((requested.0 * scale).max(0.0));
        }

        let mut clusters: Vec<ShapedCluster> = Vec::new();

        let mut run_x = metrics.offset;
        for run in line.runs() {
            for cluster in run.visual_clusters() {
                let cluster_range = cluster.text_range();
                let cluster_x0 = run_x;
                run_x = cluster_x0 + cluster.advance();
                clusters.push(ShapedCluster {
                    text_range: cluster_range,
                    x0: cluster_x0,
                    x1: run_x,
                    is_rtl: cluster.is_rtl(),
                });
            }
        }

        ShapedLineLayout {
            width: metrics.advance,
            ascent: metrics.ascent,
            descent: metrics.descent,
            baseline: metrics.baseline,
            line_height,
            glyphs: Vec::new(),
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
        let input = TextInputRef::plain("hello", &style);

        let layout = shaper.shape_single_line(input, 1.0);
        assert!(layout.width >= 0.0);
        assert!(!layout.glyphs.is_empty());
        assert!(!layout.clusters.is_empty());
    }

    #[test]
    fn clamps_line_height_to_font_extents() {
        let mut shaper = ParleyShaper::new_without_system_fonts();
        shaper.add_fonts(fret_fonts::default_fonts().iter().map(|b| b.to_vec()));

        let style = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            line_height: Some(Px(1.0)),
            ..Default::default()
        };
        let input = TextInputRef::plain("Hello", &style);

        let layout = shaper.shape_single_line(input, 1.0);
        let min = min_line_height_for_metrics(layout.ascent, layout.descent);
        assert!(
            layout.line_height + 0.001 >= min,
            "line_height={} ascent={} descent={} min={}",
            layout.line_height,
            layout.ascent,
            layout.descent,
            min
        );
    }

    #[test]
    fn respects_explicit_line_height_override() {
        let mut shaper = ParleyShaper::new_without_system_fonts();
        shaper.add_fonts(fret_fonts::default_fonts().iter().map(|b| b.to_vec()));

        let style = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            line_height: Some(Px(40.0)),
            ..Default::default()
        };
        let input = TextInputRef::plain("Hello", &style);

        let layout = shaper.shape_single_line(input, 1.0);
        assert!(layout.line_height + 0.001 >= 40.0);
    }
}
