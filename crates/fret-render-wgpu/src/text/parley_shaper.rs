use fret_core::{FontId, TextInputRef, TextShapingStyle, TextSlant, TextSpan, TextStyle};
use parley::FontContext;
use parley::FontData;
use parley::Layout;
use parley::LayoutContext;
use parley::fontique::{FamilyId, GenericFamily};
use parley::style::{
    FontSettings, FontStyle, FontVariation, FontWeight as ParleyFontWeight, StyleProperty,
    TextStyle as ParleyTextStyle,
};
use read_fonts::{FontRef, TableProvider as _};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::sync::Arc;

use super::FontCatalogEntryMetadata;

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
    pub normalized_coords: Arc<[i16]>,
    pub synthesis: parley::fontique::Synthesis,
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

pub(crate) struct ParleyShaper {
    fcx: FontContext,
    lcx: LayoutContext<[u8; 4]>,
    layout: Layout<[u8; 4]>,
    default_locale: Option<String>,
    common_fallback_stack_suffix: String,
    system_fonts_enabled: bool,
    registered_font_blobs: VecDeque<RegisteredFontBlob>,
    registered_font_blobs_total_bytes: usize,
    family_id_cache_lower: HashMap<String, FamilyId>,
    all_font_names_cache: Option<Vec<String>>,
    all_font_catalog_entries_cache: Option<Vec<FontCatalogEntryMetadata>>,
}

#[derive(Debug, Clone)]
struct RegisteredFontBlob {
    hash: u64,
    len: usize,
    blob: parley::fontique::Blob<u8>,
}

fn registered_font_blobs_max_count() -> usize {
    // Keep enough space for apps that load multiple font families (UI, mono, icons, etc) while
    // preventing unbounded growth when hot-reloading or repeatedly injecting fonts.
    std::env::var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(256)
        .min(4096)
}

fn registered_font_blobs_max_bytes() -> usize {
    // Safety valve for memory-backed font injection. This is a soft cap: we evict the oldest
    // entries until we are within budget.
    std::env::var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(256 * 1024 * 1024)
        .min(2 * 1024 * 1024 * 1024)
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    use std::hash::Hasher as _;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(bytes);
    hasher.finish()
}

impl Default for ParleyShaper {
    fn default() -> Self {
        Self {
            fcx: FontContext::default(),
            lcx: LayoutContext::default(),
            layout: Layout::default(),
            default_locale: None,
            common_fallback_stack_suffix: String::new(),
            system_fonts_enabled: true,
            registered_font_blobs: VecDeque::new(),
            registered_font_blobs_total_bytes: 0,
            family_id_cache_lower: HashMap::new(),
            all_font_names_cache: None,
            all_font_catalog_entries_cache: None,
        }
    }
}

impl ParleyShaper {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn record_registered_font_blob_bytes_for_tests(&mut self, bytes: Vec<u8>) {
        let blob = parley::fontique::Blob::<u8>::from(bytes);
        self.record_registered_font_blob(blob);
    }

    #[cfg(test)]
    fn registered_font_blob_lengths_for_tests(&self) -> Vec<usize> {
        self.registered_font_blobs.iter().map(|b| b.len).collect()
    }

    #[cfg(test)]
    fn registered_font_blob_total_bytes_for_tests(&self) -> usize {
        self.registered_font_blobs_total_bytes
    }

    fn invalidate_catalog_caches(&mut self) {
        self.family_id_cache_lower.clear();
        self.all_font_names_cache = None;
        self.all_font_catalog_entries_cache = None;
    }

    fn record_registered_font_blob(&mut self, blob: parley::fontique::Blob<u8>) {
        let bytes = blob.as_ref();
        let len = bytes.len();
        let hash = hash_bytes(bytes);

        if let Some(ix) = self
            .registered_font_blobs
            .iter()
            .position(|v| v.hash == hash && v.len == len && v.blob.as_ref() == bytes)
        {
            // LRU: keep the most recently injected fonts near the back.
            let entry = self.registered_font_blobs.remove(ix);
            if let Some(entry) = entry {
                self.registered_font_blobs.push_back(entry);
            }
            return;
        }

        self.registered_font_blobs_total_bytes =
            self.registered_font_blobs_total_bytes.saturating_add(len);
        self.registered_font_blobs
            .push_back(RegisteredFontBlob { hash, len, blob });

        let max_count = registered_font_blobs_max_count();
        let max_bytes = registered_font_blobs_max_bytes();
        while self.registered_font_blobs.len() > max_count
            || self.registered_font_blobs_total_bytes > max_bytes
        {
            let Some(evicted) = self.registered_font_blobs.pop_front() else {
                break;
            };
            self.registered_font_blobs_total_bytes = self
                .registered_font_blobs_total_bytes
                .saturating_sub(evicted.len);
        }
    }

    pub fn system_fonts_enabled(&self) -> bool {
        self.system_fonts_enabled
    }

    pub fn set_default_locale(&mut self, locale_bcp47: Option<String>) -> bool {
        if self.default_locale == locale_bcp47 {
            return false;
        }
        self.default_locale = locale_bcp47;
        true
    }

    pub fn set_common_fallback_stack_suffix(&mut self, suffix: String) -> bool {
        if self.common_fallback_stack_suffix == suffix {
            return false;
        }
        self.common_fallback_stack_suffix = suffix;
        true
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
        out.system_fonts_enabled = false;
        out
    }

    pub fn all_font_names(&mut self) -> Vec<String> {
        if let Some(cache) = self.all_font_names_cache.as_ref() {
            return cache.clone();
        }

        let mut by_lower: HashMap<String, String> = HashMap::new();
        for name in self.fcx.collection.family_names() {
            let key = name.to_ascii_lowercase();
            by_lower.entry(key).or_insert_with(|| name.to_string());
        }

        let mut names: Vec<String> = by_lower.into_values().collect();
        names.sort_unstable_by(|a, b| {
            a.to_ascii_lowercase()
                .cmp(&b.to_ascii_lowercase())
                .then(a.cmp(b))
        });

        self.all_font_names_cache = Some(names.clone());
        names
    }

    pub fn all_font_catalog_entries(&mut self) -> Vec<FontCatalogEntryMetadata> {
        if let Some(cache) = self.all_font_catalog_entries_cache.as_ref() {
            return cache.clone();
        }

        let mut by_lower: HashMap<String, String> = HashMap::new();
        for name in self.fcx.collection.family_names() {
            let key = name.to_ascii_lowercase();
            by_lower.entry(key).or_insert_with(|| name.to_string());
        }

        let mut names: Vec<String> = by_lower.into_values().collect();
        names.sort_unstable_by(|a, b| {
            a.to_ascii_lowercase()
                .cmp(&b.to_ascii_lowercase())
                .then(a.cmp(b))
        });

        let mut out: Vec<FontCatalogEntryMetadata> = Vec::with_capacity(names.len());
        for family in names {
            let Some(id) = self.fcx.collection.family_id(&family) else {
                continue;
            };
            let Some(info) = self.fcx.collection.family(id) else {
                continue;
            };

            let mut has_variable_axes = false;
            let mut has_wght = false;
            let mut has_wdth = false;
            let mut has_slnt = false;
            let mut has_ital = false;
            let mut has_opsz = false;

            for font in info.fonts() {
                has_variable_axes |= !font.axes().is_empty();
                has_wght |= font.has_weight_axis();
                has_wdth |= font.has_width_axis();
                has_slnt |= font.has_slant_axis();
                has_ital |= font.has_italic_axis();
                has_opsz |= font.has_optical_size_axis();
            }

            let mut known_variable_axes: Vec<String> = Vec::new();
            if has_wght {
                known_variable_axes.push("wght".to_string());
            }
            if has_wdth {
                known_variable_axes.push("wdth".to_string());
            }
            if has_slnt {
                known_variable_axes.push("slnt".to_string());
            }
            if has_ital {
                known_variable_axes.push("ital".to_string());
            }
            if has_opsz {
                known_variable_axes.push("opsz".to_string());
            }

            let is_monospace_candidate = info
                .default_font()
                .and_then(|font| {
                    let blob = font.load(Some(&mut self.fcx.source_cache))?;
                    let face = FontRef::from_index(blob.as_ref(), font.index()).ok()?;
                    let post = face.post().ok()?;
                    Some(post.is_fixed_pitch() != 0)
                })
                .unwrap_or(false);

            out.push(FontCatalogEntryMetadata {
                family,
                has_variable_axes,
                known_variable_axes,
                is_monospace_candidate,
            });
        }

        self.all_font_catalog_entries_cache = Some(out.clone());
        out
    }

    pub fn resolve_family_id(&mut self, name: &str) -> Option<FamilyId> {
        let name = name.trim();
        if name.is_empty() {
            return None;
        }

        if let Some(id) = self.fcx.collection.family_id(name) {
            return Some(id);
        }

        let target = name.to_ascii_lowercase();
        if let Some(id) = self.family_id_cache_lower.get(&target).copied() {
            return Some(id);
        }

        let mut resolved_name: Option<String> = None;
        for candidate in self.fcx.collection.family_names() {
            if candidate.to_ascii_lowercase() != target {
                continue;
            }
            resolved_name = Some(candidate.to_string());
            break;
        }

        let resolved = resolved_name
            .as_deref()
            .and_then(|name| self.fcx.collection.family_id(name));

        if let Some(id) = resolved {
            self.family_id_cache_lower.insert(target, id);
        }
        resolved
    }

    pub fn generic_family_ids(&mut self, generic: GenericFamily) -> Vec<FamilyId> {
        self.fcx.collection.generic_families(generic).collect()
    }

    pub fn set_generic_family_ids(&mut self, generic: GenericFamily, ids: &[FamilyId]) -> bool {
        let before = self.generic_family_ids(generic);
        if before == ids {
            return false;
        }
        self.fcx
            .collection
            .set_generic_families(generic, ids.iter().copied());
        true
    }

    pub fn add_fonts(&mut self, fonts: impl IntoIterator<Item = Vec<u8>>) -> usize {
        let mut added = 0usize;
        for data in fonts {
            let blob = parley::fontique::Blob::<u8>::from(data);
            self.record_registered_font_blob(blob.clone());
            let families = self.fcx.collection.register_fonts(blob, None);
            added = added.saturating_add(families.iter().map(|(_, fonts)| fonts.len()).sum());
        }
        if added > 0 {
            self.invalidate_catalog_caches();
        }
        added
    }

    pub fn system_font_rescan_seed(&self) -> Option<super::SystemFontRescanSeed> {
        if !self.system_fonts_enabled {
            return None;
        }

        Some(super::SystemFontRescanSeed {
            registered_font_blobs: self
                .registered_font_blobs
                .iter()
                .map(|b| b.blob.clone())
                .collect(),
        })
    }

    pub fn apply_system_font_rescan_result(
        &mut self,
        result: super::SystemFontRescanResult,
    ) -> bool {
        if !self.system_fonts_enabled {
            return false;
        }

        self.fcx.collection = result.collection;
        self.invalidate_catalog_caches();
        self.all_font_names_cache = Some(result.all_font_names);
        self.all_font_catalog_entries_cache = Some(result.all_font_catalog_entries);
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

        builder.push_style_span(base_parley_style(
            base_style,
            self.default_locale.as_deref(),
            &self.common_fallback_stack_suffix,
        ));

        if let Some(span_ranges) = resolve_span_ranges(text, spans) {
            for (range, span) in span_ranges {
                let chunk = &text[range.clone()];
                if let Some(props) = shaping_properties_for_span(
                    base_style,
                    span,
                    &self.common_fallback_stack_suffix,
                ) {
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
            let normalized_coords: Arc<[i16]> = Arc::from(run.normalized_coords());
            let synthesis = run.synthesis();

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
                        normalized_coords: normalized_coords.clone(),
                        synthesis,
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

        builder.push_style_span(base_parley_style(
            base_style,
            self.default_locale.as_deref(),
            &self.common_fallback_stack_suffix,
        ));

        if let Some(span_ranges) = resolve_span_ranges(text, spans) {
            for (range, span) in span_ranges {
                let chunk = &text[range.clone()];
                if let Some(props) = shaping_properties_for_span(
                    base_style,
                    span,
                    &self.common_fallback_stack_suffix,
                ) {
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

fn base_parley_style<'a>(
    style: &TextStyle,
    locale: Option<&'a str>,
    common_fallback_stack_suffix: &str,
) -> ParleyTextStyle<'a, [u8; 4]> {
    let stack = font_stack_for_font_id(&style.font, common_fallback_stack_suffix);
    ParleyTextStyle {
        font_size: style.size.0,
        font_weight: ParleyFontWeight::new(style.weight.0 as f32),
        font_style: font_style_for_slant(style.slant),
        letter_spacing: style.letter_spacing_em.unwrap_or(0.0).clamp(-4.0, 4.0) * style.size.0,
        locale,
        font_stack: parley::style::FontStack::Source(Cow::Owned(stack)),
        ..Default::default()
    }
}

fn font_stack_for_font_id(font: &FontId, common_fallback_stack_suffix: &str) -> String {
    match font {
        FontId::Ui => "sans-serif".to_string(),
        FontId::Serif => "serif".to_string(),
        FontId::Monospace => "monospace".to_string(),
        FontId::Family(name) => {
            if common_fallback_stack_suffix.is_empty() {
                return name.clone();
            }
            format!("{name}, {common_fallback_stack_suffix}")
        }
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
    common_fallback_stack_suffix: &str,
) -> Option<Vec<StyleProperty<'static, [u8; 4]>>> {
    let TextShapingStyle {
        font,
        weight,
        slant,
        letter_spacing_em,
        axes,
    } = &span.shaping;

    let mut out: Vec<StyleProperty<'static, [u8; 4]>> = Vec::new();

    if let Some(font) = font {
        let stack = font_stack_for_font_id(font, common_fallback_stack_suffix);
        out.push(StyleProperty::FontStack(parley::style::FontStack::Source(
            Cow::Owned(stack),
        )));
    }

    if !axes.is_empty() {
        let variations = font_variations_for_axes(axes);
        if !variations.is_empty() {
            out.push(StyleProperty::FontVariations(FontSettings::List(
                Cow::Owned(variations.into()),
            )));
        }
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

fn font_variations_for_axes(axes: &[fret_core::TextFontAxisSetting]) -> Vec<FontVariation> {
    use std::collections::BTreeMap;

    let mut by_tag: BTreeMap<u32, FontVariation> = BTreeMap::new();
    for axis in axes {
        let tag = axis.tag.trim();
        if tag.is_empty() {
            continue;
        }
        let bytes = tag.as_bytes();
        if bytes.len() != 4 {
            continue;
        }
        if !axis.value.is_finite() {
            continue;
        }

        let mut tag_bytes = [0u8; 4];
        tag_bytes.copy_from_slice(bytes);
        let tuple = (tag_bytes, axis.value);
        let setting = FontVariation::from(&tuple);
        by_tag.insert(setting.tag, setting);
    }

    by_tag.into_values().collect::<Vec<_>>()
}

pub(super) fn run_system_font_rescan(
    seed: super::SystemFontRescanSeed,
) -> super::SystemFontRescanResult {
    let mut shaper = ParleyShaper::new();
    shaper.fcx.collection =
        parley::fontique::Collection::new(parley::fontique::CollectionOptions {
            shared: false,
            system_fonts: true,
        });
    shaper.fcx.source_cache = parley::fontique::SourceCache::default();

    for blob in seed.registered_font_blobs.into_iter() {
        let _ = shaper.fcx.collection.register_fonts(blob, None);
    }

    let all_font_names = shaper.all_font_names();
    let all_font_catalog_entries = shaper.all_font_catalog_entries();
    super::SystemFontRescanResult {
        collection: shaper.fcx.collection,
        all_font_names,
        all_font_catalog_entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Px;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn font_catalog_caches_invalidate_after_add_fonts() {
        let mut shaper = ParleyShaper::new_without_system_fonts();

        let names0 = shaper.all_font_names();
        assert!(
            !names0.iter().any(|n| n.eq_ignore_ascii_case("Inter")),
            "expected Inter to be absent before adding bundled fonts"
        );

        let entries0 = shaper.all_font_catalog_entries();
        assert!(
            !entries0
                .iter()
                .any(|e| e.family.eq_ignore_ascii_case("Inter")),
            "expected catalog entries to be empty of Inter before adding bundled fonts"
        );

        let added = shaper.add_fonts(fret_fonts::bootstrap_fonts().iter().map(|b| b.to_vec()));
        assert!(added > 0, "expected bundled fonts to load");

        let names1 = shaper.all_font_names();
        assert!(
            names1.iter().any(|n| n.eq_ignore_ascii_case("Inter")),
            "expected Inter to be present after adding bundled fonts"
        );
        assert_eq!(
            names1,
            shaper.all_font_names(),
            "expected repeated catalog reads to be stable"
        );

        let entries1 = shaper.all_font_catalog_entries();
        assert!(
            entries1
                .iter()
                .any(|e| e.family.eq_ignore_ascii_case("Inter")),
            "expected catalog entries to include Inter after adding bundled fonts"
        );
        assert_eq!(
            entries1,
            shaper.all_font_catalog_entries(),
            "expected repeated catalog reads to be stable"
        );
    }

    #[test]
    fn registered_font_blobs_dedup_and_lru_eviction_by_count() {
        let _guard = env_lock().lock().unwrap();
        let prev_max_count = std::env::var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT").ok();
        let prev_max_bytes = std::env::var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES").ok();
        unsafe {
            std::env::set_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT", "2");
            std::env::set_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES", "1048576");
        }

        let mut shaper = ParleyShaper::new_without_system_fonts();
        shaper.record_registered_font_blob_bytes_for_tests(vec![1u8; 1]); // A
        shaper.record_registered_font_blob_bytes_for_tests(vec![2u8; 2]); // B
        shaper.record_registered_font_blob_bytes_for_tests(vec![1u8; 1]); // A again (touch)
        shaper.record_registered_font_blob_bytes_for_tests(vec![3u8; 3]); // C -> evict B

        assert_eq!(shaper.registered_font_blob_lengths_for_tests(), vec![1, 3]);
        assert_eq!(shaper.registered_font_blob_total_bytes_for_tests(), 4);

        unsafe {
            match prev_max_count {
                Some(v) => std::env::set_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT", v),
                None => std::env::remove_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT"),
            }
            match prev_max_bytes {
                Some(v) => std::env::set_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES", v),
                None => std::env::remove_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES"),
            }
        }
    }

    #[test]
    fn registered_font_blobs_eviction_by_bytes_budget() {
        let _guard = env_lock().lock().unwrap();
        let prev_max_count = std::env::var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT").ok();
        let prev_max_bytes = std::env::var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES").ok();
        unsafe {
            std::env::set_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT", "4096");
            std::env::set_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES", "3");
        }

        let mut shaper = ParleyShaper::new_without_system_fonts();
        shaper.record_registered_font_blob_bytes_for_tests(vec![1u8; 2]);
        shaper.record_registered_font_blob_bytes_for_tests(vec![2u8; 2]);

        assert_eq!(shaper.registered_font_blob_lengths_for_tests(), vec![2]);
        assert_eq!(shaper.registered_font_blob_total_bytes_for_tests(), 2);

        unsafe {
            match prev_max_count {
                Some(v) => std::env::set_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT", v),
                None => std::env::remove_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_COUNT"),
            }
            match prev_max_bytes {
                Some(v) => std::env::set_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES", v),
                None => std::env::remove_var("FRET_TEXT_REGISTERED_FONT_BLOBS_MAX_BYTES"),
            }
        }
    }

    #[test]
    fn rescan_is_noop_when_system_fonts_disabled() {
        let shaper = ParleyShaper::new_without_system_fonts();
        assert!(shaper.system_font_rescan_seed().is_none());
    }

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
