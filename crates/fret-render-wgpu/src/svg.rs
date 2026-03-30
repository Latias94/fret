use std::{
    borrow::Cow,
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use fret_core::SvgFit;
use resvg::tiny_skia::{Pixmap, Transform};
use thiserror::Error;

use crate::upload_counters::record_svg_upload;

pub(crate) const SMOOTH_SVG_SCALE_FACTOR: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct SvgAlphaMask {
    pub size_px: (u32, u32),
    pub alpha: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SvgRgbaImage {
    pub size_px: (u32, u32),
    /// Unpremultiplied RGBA8 pixels (matches `SceneOp::Image` expectations).
    pub rgba: Vec<u8>,
}

pub struct UploadedAlphaMask {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size_px: (u32, u32),
}

pub struct UploadedRgbaImage {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size_px: (u32, u32),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SvgTextBridgeDiagnostics {
    pub selection_misses: Vec<SvgTextFontSelectionMiss>,
    pub fallback_records: Vec<SvgTextFontFallbackRecord>,
    pub missing_glyphs: Vec<SvgTextMissingGlyphRecord>,
}

impl SvgTextBridgeDiagnostics {
    pub(crate) fn is_clean(&self) -> bool {
        self.selection_misses.is_empty() && self.missing_glyphs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SvgTextFontSelectionMiss {
    pub requested_families: Vec<String>,
    pub weight: u16,
    pub style: &'static str,
    pub stretch: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SvgTextFontFallbackRecord {
    pub text: String,
    pub from_family: String,
    pub to_family: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SvgTextMissingGlyphRecord {
    pub text: String,
    pub resolved_family: String,
}

struct SvgBridgeParseOutcome {
    tree: usvg::Tree,
    diagnostics: SvgTextBridgeDiagnostics,
}

#[derive(Debug, Error)]
pub(crate) enum SvgRenderError {
    #[error(
        "text-bearing SVG assets are not supported by the first-party SVG raster pipeline; convert text to outlines"
    )]
    TextNodesUnsupported,
    #[error(transparent)]
    Parse(#[from] usvg::Error),
}

#[derive(Clone, Copy)]
pub(crate) struct SvgRenderer;

#[derive(Default)]
struct SvgTextBridgeDiagnosticsCollector {
    selection_misses: BTreeSet<SvgTextFontSelectionMiss>,
    fallback_records: BTreeSet<SvgTextFontFallbackRecord>,
}

fn render_scale(
    svg_size: usvg::Size,
    target_box_px: (u32, u32),
    smooth_scale_factor: f32,
    fit: SvgFit,
) -> Result<(f32, f32, u32, u32), usvg::Error> {
    let (w, h) = target_box_px;
    if w == 0 || h == 0 {
        return Err(usvg::Error::InvalidSize);
    }

    let box_w = w as f32 * smooth_scale_factor;
    let box_h = h as f32 * smooth_scale_factor;
    let scale_x = box_w / svg_size.width();
    let scale_y = box_h / svg_size.height();
    if !scale_x.is_finite() || !scale_y.is_finite() || scale_x <= 0.0 || scale_y <= 0.0 {
        return Err(usvg::Error::InvalidSize);
    }

    let (sx, sy, out_w, out_h) = match fit {
        SvgFit::Contain => {
            let scale = scale_x.min(scale_y);
            let out_w = (svg_size.width() * scale).ceil() as u32;
            let out_h = (svg_size.height() * scale).ceil() as u32;
            (scale, scale, out_w, out_h)
        }
        SvgFit::Width => {
            let out_w = (svg_size.width() * scale_x).ceil() as u32;
            let out_h = (svg_size.height() * scale_x).ceil() as u32;
            (scale_x, scale_x, out_w, out_h)
        }
        SvgFit::Stretch => {
            let out_w = box_w.ceil() as u32;
            let out_h = box_h.ceil() as u32;
            (scale_x, scale_y, out_w, out_h)
        }
    };

    if out_w == 0 || out_h == 0 {
        return Err(usvg::Error::InvalidSize);
    }

    Ok((sx, sy, out_w, out_h))
}

impl SvgRenderer {
    pub(crate) fn new() -> Self {
        Self
    }

    fn parse_supported_tree(&self, bytes: &[u8]) -> Result<usvg::Tree, SvgRenderError> {
        self.parse_tree(bytes, None)
    }

    fn parse_tree(
        &self,
        bytes: &[u8],
        bridge_font_db: Option<&usvg::fontdb::Database>,
    ) -> Result<usvg::Tree, SvgRenderError> {
        if let Some(fontdb) = bridge_font_db {
            let outcome = self.parse_tree_with_bridge_font_db(bytes, fontdb)?;
            debug_assert!(
                outcome.diagnostics.is_clean(),
                "bridge-backed shipped svg parse path must stay diagnostics-clean until text support is promoted"
            );
            return Ok(outcome.tree);
        }

        ensure_svg_text_free(bytes)?;

        let options = usvg::Options::default();
        let tree = usvg::Tree::from_data(bytes, &options)?;
        Ok(tree)
    }

    fn parse_tree_with_bridge_font_db(
        &self,
        bytes: &[u8],
        bridge_font_db: &usvg::fontdb::Database,
    ) -> Result<SvgBridgeParseOutcome, SvgRenderError> {
        let collector = Arc::new(Mutex::new(SvgTextBridgeDiagnosticsCollector::default()));
        let mut options = usvg::Options::default();
        *options.fontdb_mut() = bridge_font_db.clone();
        options.font_resolver = build_bridge_font_resolver(Arc::clone(&collector));

        let tree = usvg::Tree::from_data(bytes, &options)?;
        let missing_glyphs = collect_missing_glyphs_from_tree(&tree);
        let collector = collector
            .lock()
            .expect("svg bridge diagnostics collector lock poisoned");

        Ok(SvgBridgeParseOutcome {
            tree,
            diagnostics: SvgTextBridgeDiagnostics {
                selection_misses: collector.selection_misses.iter().cloned().collect(),
                fallback_records: collector.fallback_records.iter().cloned().collect(),
                missing_glyphs,
            },
        })
    }

    fn render_alpha_mask_for_tree(
        &self,
        tree: &usvg::Tree,
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
        fit: SvgFit,
    ) -> Result<SvgAlphaMask, SvgRenderError> {
        let svg_size = tree.size();
        let (sx, sy, out_w, out_h) =
            render_scale(svg_size, target_box_px, smooth_scale_factor, fit)?;

        let mut pixmap = Pixmap::new(out_w, out_h).ok_or(usvg::Error::InvalidSize)?;
        let transform = Transform::from_scale(sx, sy);
        resvg::render(tree, transform, &mut pixmap.as_mut());

        let alpha = pixmap
            .pixels()
            .iter()
            .map(|p| p.alpha())
            .collect::<Vec<_>>();
        Ok(SvgAlphaMask {
            size_px: (pixmap.width(), pixmap.height()),
            alpha,
        })
    }

    fn render_rgba_for_tree(
        &self,
        tree: &usvg::Tree,
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
        fit: SvgFit,
    ) -> Result<SvgRgbaImage, SvgRenderError> {
        let svg_size = tree.size();
        let (sx, sy, out_w, out_h) =
            render_scale(svg_size, target_box_px, smooth_scale_factor, fit)?;

        let mut pixmap = Pixmap::new(out_w, out_h).ok_or(usvg::Error::InvalidSize)?;
        let transform = Transform::from_scale(sx, sy);
        resvg::render(tree, transform, &mut pixmap.as_mut());

        // tiny-skia pixmap stores RGBA premultiplied alpha. Our image pipeline expects
        // unpremultiplied RGBA (it premultiplies in the shader).
        let mut rgba = pixmap.take();
        debug_assert_eq!(rgba.len(), (out_w as usize) * (out_h as usize) * 4);

        for px in rgba.chunks_exact_mut(4) {
            let a = px[3] as u32;
            if a == 0 || a == 255 {
                continue;
            }
            // Unpremultiply with rounding; clamp to 0..255.
            let r = (px[0] as u32 * 255 + a / 2) / a;
            let g = (px[1] as u32 * 255 + a / 2) / a;
            let b = (px[2] as u32 * 255 + a / 2) / a;
            px[0] = r.min(255) as u8;
            px[1] = g.min(255) as u8;
            px[2] = b.min(255) as u8;
        }

        Ok(SvgRgbaImage {
            size_px: (out_w, out_h),
            rgba,
        })
    }

    pub(crate) fn render_alpha_mask_fit_mode(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
        fit: SvgFit,
    ) -> Result<SvgAlphaMask, SvgRenderError> {
        let tree = self.parse_supported_tree(bytes)?;
        self.render_alpha_mask_for_tree(&tree, target_box_px, smooth_scale_factor, fit)
    }

    pub(crate) fn render_rgba_fit_mode(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
        fit: SvgFit,
    ) -> Result<SvgRgbaImage, SvgRenderError> {
        let tree = self.parse_supported_tree(bytes)?;
        self.render_rgba_for_tree(&tree, target_box_px, smooth_scale_factor, fit)
    }

    pub(crate) fn render_alpha_mask_fit_mode_with_bridge_font_db(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
        fit: SvgFit,
        fontdb: &usvg::fontdb::Database,
    ) -> Result<(SvgAlphaMask, SvgTextBridgeDiagnostics), SvgRenderError> {
        let outcome = self.parse_tree_with_bridge_font_db(bytes, fontdb)?;
        let mask = self.render_alpha_mask_for_tree(
            &outcome.tree,
            target_box_px,
            smooth_scale_factor,
            fit,
        )?;
        Ok((mask, outcome.diagnostics))
    }

    pub(crate) fn render_rgba_fit_mode_with_bridge_font_db(
        &self,
        bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
        fit: SvgFit,
        fontdb: &usvg::fontdb::Database,
    ) -> Result<(SvgRgbaImage, SvgTextBridgeDiagnostics), SvgRenderError> {
        let outcome = self.parse_tree_with_bridge_font_db(bytes, fontdb)?;
        let image =
            self.render_rgba_for_tree(&outcome.tree, target_box_px, smooth_scale_factor, fit)?;
        Ok((image, outcome.diagnostics))
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn ensure_svg_text_free(bytes: &[u8]) -> Result<(), SvgRenderError> {
    if svg_contains_text_nodes(bytes)? {
        return Err(SvgRenderError::TextNodesUnsupported);
    }
    Ok(())
}

pub(crate) fn svg_contains_text_nodes(bytes: &[u8]) -> Result<bool, SvgRenderError> {
    let xml = decode_svg_xml(bytes)?;
    let options = usvg::roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let document = usvg::roxmltree::Document::parse_with_options(xml.as_ref(), options)
        .map_err(usvg::Error::ParsingFailed)?;
    Ok(document.descendants().any(is_text_element))
}

fn decode_svg_xml(bytes: &[u8]) -> Result<Cow<'_, str>, SvgRenderError> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        let decoded = usvg::decompress_svgz(bytes)?;
        let text = std::str::from_utf8(&decoded).map_err(|_| usvg::Error::NotAnUtf8Str)?;
        Ok(Cow::Owned(text.to_owned()))
    } else {
        let text = std::str::from_utf8(bytes).map_err(|_| usvg::Error::NotAnUtf8Str)?;
        Ok(Cow::Borrowed(text))
    }
}

fn is_text_element(node: usvg::roxmltree::Node<'_, '_>) -> bool {
    if !node.is_element() {
        return false;
    }

    let tag_name = node.tag_name();
    if tag_name.namespace() != Some("http://www.w3.org/2000/svg") {
        return false;
    }

    matches!(tag_name.name(), "text" | "tspan" | "textPath" | "tref")
}

fn build_bridge_font_resolver(
    collector: Arc<Mutex<SvgTextBridgeDiagnosticsCollector>>,
) -> usvg::FontResolver<'static> {
    let selection_collector = Arc::clone(&collector);
    let fallback_collector = Arc::clone(&collector);

    usvg::FontResolver {
        select_font: Box::new(move |font, fontdb| {
            let requested_query_families = font
                .families()
                .iter()
                .map(svg_font_family_to_fontdb_family)
                .collect::<Vec<_>>();
            let requested_query = usvg::fontdb::Query {
                families: &requested_query_families,
                weight: usvg::fontdb::Weight(font.weight()),
                stretch: svg_font_stretch_to_fontdb(font.stretch()),
                style: svg_font_style_to_fontdb(font.style()),
            };
            if fontdb.query(&requested_query).is_none() {
                let record = SvgTextFontSelectionMiss {
                    requested_families: font.families().iter().map(svg_font_family_label).collect(),
                    weight: font.weight(),
                    style: svg_font_style_label(font.style()),
                    stretch: svg_font_stretch_label(font.stretch()),
                };
                selection_collector
                    .lock()
                    .expect("svg bridge diagnostics collector lock poisoned")
                    .selection_misses
                    .insert(record);
            }

            let mut query_families = requested_query_families;
            query_families.push(usvg::fontdb::Family::Serif);
            let query = usvg::fontdb::Query {
                families: &query_families,
                weight: usvg::fontdb::Weight(font.weight()),
                stretch: svg_font_stretch_to_fontdb(font.stretch()),
                style: svg_font_style_to_fontdb(font.style()),
            };
            fontdb.query(&query)
        }),
        select_fallback: Box::new(move |character, exclude_fonts, fontdb| {
            let &base_font_id = exclude_fonts.first()?;
            let base_face = fontdb.face(base_font_id)?;

            for face in fontdb.faces() {
                if exclude_fonts.contains(&face.id) {
                    continue;
                }

                if base_face.style != face.style
                    && base_face.weight != face.weight
                    && base_face.stretch != face.stretch
                {
                    continue;
                }

                if !face_supports_character(fontdb.as_ref(), face.id, character) {
                    continue;
                }

                fallback_collector
                    .lock()
                    .expect("svg bridge diagnostics collector lock poisoned")
                    .fallback_records
                    .insert(SvgTextFontFallbackRecord {
                        text: character.to_string(),
                        from_family: preferred_face_family_name(base_face),
                        to_family: preferred_face_family_name(face),
                    });
                return Some(face.id);
            }

            None
        }),
    }
}

fn svg_font_family_to_fontdb_family(family: &usvg::FontFamily) -> usvg::fontdb::Family<'_> {
    match family {
        usvg::FontFamily::Serif => usvg::fontdb::Family::Serif,
        usvg::FontFamily::SansSerif => usvg::fontdb::Family::SansSerif,
        usvg::FontFamily::Cursive => usvg::fontdb::Family::Cursive,
        usvg::FontFamily::Fantasy => usvg::fontdb::Family::Fantasy,
        usvg::FontFamily::Monospace => usvg::fontdb::Family::Monospace,
        usvg::FontFamily::Named(name) => usvg::fontdb::Family::Name(name),
    }
}

fn svg_font_family_label(family: &usvg::FontFamily) -> String {
    match family {
        usvg::FontFamily::Serif => "serif".to_string(),
        usvg::FontFamily::SansSerif => "sans-serif".to_string(),
        usvg::FontFamily::Cursive => "cursive".to_string(),
        usvg::FontFamily::Fantasy => "fantasy".to_string(),
        usvg::FontFamily::Monospace => "monospace".to_string(),
        usvg::FontFamily::Named(name) => name.to_string(),
    }
}

fn svg_font_style_to_fontdb(style: usvg::FontStyle) -> usvg::fontdb::Style {
    match style {
        usvg::FontStyle::Normal => usvg::fontdb::Style::Normal,
        usvg::FontStyle::Italic => usvg::fontdb::Style::Italic,
        usvg::FontStyle::Oblique => usvg::fontdb::Style::Oblique,
    }
}

fn svg_font_style_label(style: usvg::FontStyle) -> &'static str {
    match style {
        usvg::FontStyle::Normal => "normal",
        usvg::FontStyle::Italic => "italic",
        usvg::FontStyle::Oblique => "oblique",
    }
}

fn svg_font_stretch_to_fontdb(stretch: usvg::FontStretch) -> usvg::fontdb::Stretch {
    match stretch {
        usvg::FontStretch::UltraCondensed => usvg::fontdb::Stretch::UltraCondensed,
        usvg::FontStretch::ExtraCondensed => usvg::fontdb::Stretch::ExtraCondensed,
        usvg::FontStretch::Condensed => usvg::fontdb::Stretch::Condensed,
        usvg::FontStretch::SemiCondensed => usvg::fontdb::Stretch::SemiCondensed,
        usvg::FontStretch::Normal => usvg::fontdb::Stretch::Normal,
        usvg::FontStretch::SemiExpanded => usvg::fontdb::Stretch::SemiExpanded,
        usvg::FontStretch::Expanded => usvg::fontdb::Stretch::Expanded,
        usvg::FontStretch::ExtraExpanded => usvg::fontdb::Stretch::ExtraExpanded,
        usvg::FontStretch::UltraExpanded => usvg::fontdb::Stretch::UltraExpanded,
    }
}

fn svg_font_stretch_label(stretch: usvg::FontStretch) -> &'static str {
    match stretch {
        usvg::FontStretch::UltraCondensed => "ultra-condensed",
        usvg::FontStretch::ExtraCondensed => "extra-condensed",
        usvg::FontStretch::Condensed => "condensed",
        usvg::FontStretch::SemiCondensed => "semi-condensed",
        usvg::FontStretch::Normal => "normal",
        usvg::FontStretch::SemiExpanded => "semi-expanded",
        usvg::FontStretch::Expanded => "expanded",
        usvg::FontStretch::ExtraExpanded => "extra-expanded",
        usvg::FontStretch::UltraExpanded => "ultra-expanded",
    }
}

fn preferred_face_family_name(face: &usvg::fontdb::FaceInfo) -> String {
    face.families
        .iter()
        .find(|(_, language)| *language == usvg::fontdb::Language::English_UnitedStates)
        .or_else(|| face.families.first())
        .map(|(family, _)| family.clone())
        .unwrap_or_else(|| face.post_script_name.clone())
}

fn face_supports_character(
    fontdb: &usvg::fontdb::Database,
    face_id: usvg::fontdb::ID,
    character: char,
) -> bool {
    fontdb
        .with_face_data(face_id, |font_data, face_index| {
            ttf_parser::Face::parse(font_data, face_index)
                .ok()
                .and_then(|face| face.glyph_index(character))
                .filter(|glyph_id| glyph_id.0 != 0)
                .is_some()
        })
        .unwrap_or(false)
}

fn collect_missing_glyphs_from_tree(tree: &usvg::Tree) -> Vec<SvgTextMissingGlyphRecord> {
    let mut records = BTreeSet::new();
    collect_missing_glyphs_from_group(tree.root(), tree.fontdb().as_ref(), &mut records);
    records.into_iter().collect()
}

fn collect_missing_glyphs_from_group(
    group: &usvg::Group,
    fontdb: &usvg::fontdb::Database,
    records: &mut BTreeSet<SvgTextMissingGlyphRecord>,
) {
    for node in group.children() {
        match node {
            usvg::Node::Group(child_group) => {
                collect_missing_glyphs_from_group(child_group, fontdb, records);
            }
            usvg::Node::Path(_) => {}
            usvg::Node::Image(image) => {
                if let usvg::ImageKind::SVG(tree) = image.kind() {
                    collect_missing_glyphs_from_group(tree.root(), tree.fontdb().as_ref(), records);
                }
            }
            usvg::Node::Text(text) => collect_missing_glyphs_from_text(text, fontdb, records),
        }
    }

    if let Some(clip_path) = group.clip_path() {
        collect_missing_glyphs_from_group(clip_path.root(), fontdb, records);
        if let Some(sub_clip_path) = clip_path.clip_path() {
            collect_missing_glyphs_from_group(sub_clip_path.root(), fontdb, records);
        }
    }

    if let Some(mask) = group.mask() {
        collect_missing_glyphs_from_group(mask.root(), fontdb, records);
        if let Some(sub_mask) = mask.mask() {
            collect_missing_glyphs_from_group(sub_mask.root(), fontdb, records);
        }
    }

    for filter in group.filters() {
        for primitive in filter.primitives() {
            if let usvg::filter::Kind::Image(image) = primitive.kind() {
                collect_missing_glyphs_from_group(image.root(), fontdb, records);
            }
        }
    }
}

fn collect_missing_glyphs_from_text(
    text: &usvg::Text,
    fontdb: &usvg::fontdb::Database,
    records: &mut BTreeSet<SvgTextMissingGlyphRecord>,
) {
    for span in text.layouted() {
        for glyph in &span.positioned_glyphs {
            if glyph.id.0 != 0 {
                continue;
            }

            let resolved_family = fontdb
                .face(glyph.font)
                .map(preferred_face_family_name)
                .unwrap_or_else(|| "<unknown>".to_string());
            records.insert(SvgTextMissingGlyphRecord {
                text: glyph.text.clone(),
                resolved_family,
            });
        }
    }
}

pub fn upload_alpha_mask(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mask: &SvgAlphaMask,
) -> UploadedAlphaMask {
    let (w, h) = mask.size_px;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fret svg alpha mask"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bytes_per_row = w;
    let aligned_bytes_per_row = bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);
    let data = if aligned_bytes_per_row == bytes_per_row {
        mask.alpha.clone()
    } else {
        let mut padded = vec![0u8; (aligned_bytes_per_row * h) as usize];
        for row in 0..h as usize {
            let src0 = row * w as usize;
            let src1 = src0 + w as usize;
            let dst0 = row * aligned_bytes_per_row as usize;
            let dst1 = dst0 + w as usize;
            padded[dst0..dst1].copy_from_slice(&mask.alpha[src0..src1]);
        }
        padded
    };

    if w > 0 && h > 0 {
        record_svg_upload(data.len());
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(aligned_bytes_per_row),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    UploadedAlphaMask {
        texture,
        view,
        size_px: (w, h),
    }
}

pub fn upload_rgba_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &SvgRgbaImage,
) -> UploadedRgbaImage {
    let (w, h) = image.size_px;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fret svg rgba"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bytes_per_row = w * 4;
    let aligned_bytes_per_row = bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);
    let data = if aligned_bytes_per_row == bytes_per_row {
        image.rgba.clone()
    } else {
        let mut padded = vec![0u8; (aligned_bytes_per_row * h) as usize];
        for row in 0..h as usize {
            let src0 = row * (w as usize) * 4;
            let src1 = src0 + (w as usize) * 4;
            let dst0 = row * aligned_bytes_per_row as usize;
            let dst1 = dst0 + (w as usize) * 4;
            padded[dst0..dst1].copy_from_slice(&image.rgba[src0..src1]);
        }
        padded
    };

    if w > 0 && h > 0 {
        record_svg_upload(data.len());
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(aligned_bytes_per_row),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    UploadedRgbaImage {
        texture,
        view,
        size_px: (w, h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_svg_bridge_font_db() -> usvg::fontdb::Database {
        // Keep SVG bridge diagnostics/tests on the bundled-only lane so host system-font drift
        // cannot change fallback outcomes.
        unsafe {
            std::env::set_var("FRET_TEXT_SYSTEM_FONTS", "0");
        }

        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut renderer = crate::Renderer::new(&ctx.adapter, &ctx.device);

        let fonts: Vec<Vec<u8>> =
            fret_fonts::test_support::face_blobs(fret_fonts::default_profile().faces.iter())
                .collect();
        let added = renderer.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load for svg bridge");

        let _ = renderer.set_text_font_families(&fret_core::TextFontFamilyConfig {
            common_fallback_injection: fret_core::TextCommonFallbackInjection::CommonFallback,
            ui_sans: vec!["Inter".to_string()],
            ui_mono: vec!["JetBrains Mono".to_string()],
            ..Default::default()
        });

        renderer.build_svg_text_font_db_for_bridge()
    }

    #[test]
    fn svg_alpha_mask_has_coverage() {
        let svg = r#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
  <circle cx="8" cy="8" r="6" fill="black" />
</svg>
"#;
        let renderer = SvgRenderer::new();
        let mask = renderer
            .render_alpha_mask_fit_mode(
                svg.as_bytes(),
                (32, 32),
                SMOOTH_SVG_SCALE_FACTOR,
                SvgFit::Contain,
            )
            .expect("render alpha mask");

        let (w, h) = mask.size_px;
        assert!(w > 0 && h > 0);
        assert_eq!(mask.alpha.len(), (w as usize) * (h as usize));
        assert!(mask.alpha.iter().any(|&a| a > 0));
        assert!(mask.alpha.iter().any(|&a| a < 255));
    }

    #[test]
    fn svg_rgba_renders_pixels() {
        let svg = r##"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
  <rect x="0" y="0" width="16" height="16" fill="#ff0000" />
  <circle cx="8" cy="8" r="6" fill="#00ff00" fill-opacity="0.5" />
</svg>
"##;
        let renderer = SvgRenderer::new();
        let img = renderer
            .render_rgba_fit_mode(
                svg.as_bytes(),
                (32, 32),
                SMOOTH_SVG_SCALE_FACTOR,
                SvgFit::Contain,
            )
            .expect("render rgba");

        let (w, h) = img.size_px;
        assert!(w > 0 && h > 0);
        assert_eq!(img.rgba.len(), (w as usize) * (h as usize) * 4);
        assert!(img.rgba.chunks_exact(4).any(|px| px[3] > 0));
        // The output is composited on an opaque background (the red rect), so alpha will likely
        // be fully opaque; assert color blending instead.
        assert!(img.rgba.chunks_exact(4).any(|px| px[1] > 0 && px[0] < 255));
    }

    #[test]
    fn svg_text_nodes_are_rejected_for_alpha_and_rgba_rasterization() {
        let svg = r#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 16">
  <text x="4" y="12" font-size="12">Fret</text>
</svg>
"#;
        let renderer = SvgRenderer::new();

        let alpha_err = renderer
            .render_alpha_mask_fit_mode(
                svg.as_bytes(),
                (64, 32),
                SMOOTH_SVG_SCALE_FACTOR,
                SvgFit::Contain,
            )
            .expect_err("text-bearing SVG alpha mask should be rejected");
        assert!(matches!(alpha_err, SvgRenderError::TextNodesUnsupported));

        let rgba_err = renderer
            .render_rgba_fit_mode(
                svg.as_bytes(),
                (64, 32),
                SMOOTH_SVG_SCALE_FACTOR,
                SvgFit::Contain,
            )
            .expect_err("text-bearing SVG rgba raster should be rejected");
        assert!(matches!(rgba_err, SvgRenderError::TextNodesUnsupported));
    }

    #[test]
    fn svg_text_can_render_with_bridge_font_db_seed() {
        let svg = r##"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 24">
  <text x="4" y="18" font-family="Inter" font-size="16" fill="#000000">Fret</text>
</svg>
"##;
        let renderer = SvgRenderer::new();
        let fontdb = build_svg_bridge_font_db();

        let image = renderer
            .render_rgba_fit_mode_with_bridge_font_db(
                svg.as_bytes(),
                (128, 48),
                SMOOTH_SVG_SCALE_FACTOR,
                SvgFit::Contain,
                &fontdb,
            )
            .expect("svg text should render when fed from the renderer bridge font db");
        let (image, diagnostics) = image;

        let (w, h) = image.size_px;
        assert!(w > 0 && h > 0);
        assert_eq!(image.rgba.len(), (w as usize) * (h as usize) * 4);
        assert!(diagnostics.is_clean());
        assert!(
            image.rgba.chunks_exact(4).any(|px| px[3] > 0),
            "expected bridge-backed svg text rasterization to produce covered pixels"
        );
    }

    #[test]
    fn svg_text_bridge_diagnostics_record_font_selection_misses() {
        let svg = r#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 24">
  <text x="4" y="18" font-family="Definitely Missing" font-size="16">Fret</text>
</svg>
"#;
        let renderer = SvgRenderer::new();
        let fontdb = build_svg_bridge_font_db();

        let outcome = renderer
            .parse_tree_with_bridge_font_db(svg.as_bytes(), &fontdb)
            .expect("bridge parse should succeed even when the requested font family is absent");

        assert_eq!(
            outcome.diagnostics.selection_misses,
            vec![SvgTextFontSelectionMiss {
                requested_families: vec!["Definitely Missing".to_string()],
                weight: 400,
                style: "normal",
                stretch: "normal",
            }]
        );
        assert!(outcome.diagnostics.fallback_records.is_empty());
        assert!(outcome.diagnostics.missing_glyphs.is_empty());
        assert!(!outcome.diagnostics.is_clean());
    }

    #[test]
    fn svg_text_bridge_diagnostics_record_fallbacks() {
        let svg = r#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 24">
  <text x="4" y="18" font-family="Inter" font-size="16">A中</text>
</svg>
"#;
        let renderer = SvgRenderer::new();
        let fontdb = build_svg_bridge_font_db();

        let outcome = renderer
            .parse_tree_with_bridge_font_db(svg.as_bytes(), &fontdb)
            .expect("bridge parse should succeed when fallback fonts cover the text");

        assert!(outcome.diagnostics.selection_misses.is_empty());
        assert!(
            outcome
                .diagnostics
                .fallback_records
                .contains(&SvgTextFontFallbackRecord {
                    text: "中".to_string(),
                    from_family: "Inter".to_string(),
                    to_family: "Noto Sans CJK SC".to_string(),
                }),
            "expected fallback diagnostics to record the renderer-approved CJK fallback face, actual diagnostics: {:?}",
            outcome.diagnostics
        );
        assert!(outcome.diagnostics.missing_glyphs.is_empty());
        assert!(outcome.diagnostics.is_clean());
    }

    #[test]
    fn svg_text_bridge_diagnostics_record_missing_glyphs() {
        let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 64 24\">\
  <text x=\"4\" y=\"18\" font-family=\"Inter\" font-size=\"16\">&#x0378;</text>\
</svg>";
        let renderer = SvgRenderer::new();
        let fontdb = build_svg_bridge_font_db();

        let outcome = renderer
            .parse_tree_with_bridge_font_db(svg.as_bytes(), &fontdb)
            .expect("bridge parse should succeed even when a glyph cannot be resolved");

        assert!(outcome.diagnostics.selection_misses.is_empty());
        assert!(
            outcome.diagnostics.fallback_records.is_empty(),
            "expected no successful fallback records for an unsupported scalar, actual diagnostics: {:?}",
            outcome.diagnostics
        );
        assert_eq!(
            outcome.diagnostics.missing_glyphs,
            vec![SvgTextMissingGlyphRecord {
                text: "\u{378}".to_string(),
                resolved_family: "Inter".to_string(),
            }]
        );
        assert!(!outcome.diagnostics.is_clean());
    }
}
