use cosmic_text::{Family, FontSystem};
use fret_core::scene::{Scene, SceneOp};
use fret_core::{
    AttributedText, CaretAffinity, HitTestResult, Point, Rect, Size, TextBlobId, TextConstraints,
    TextInputRef, TextMetrics, TextOverflow, TextSlant, TextSpan, TextStyle, TextWrap,
    geometry::Px,
};
use slotmap::SlotMap;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
    sync::{Arc, OnceLock},
};

use parley::fontique::GenericFamily as ParleyGenericFamily;

pub(crate) mod parley_shaper;
pub(crate) mod wrapper;

fn released_blob_cache_entries() -> usize {
    static ENTRIES: OnceLock<usize> = OnceLock::new();
    *ENTRIES.get_or_init(|| {
        // Default: off. Opt in to retain recently released text blobs to reduce `Text::prepare`
        // thrash when wrap widths oscillate (e.g. interactive resize jitter).
        std::env::var("FRET_TEXT_RELEASED_BLOB_CACHE_ENTRIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0)
            .min(2048)
    })
}

fn unwrapped_layout_cache_entries() -> usize {
    static ENTRIES: OnceLock<usize> = OnceLock::new();
    *ENTRIES.get_or_init(|| {
        // Default: on for native builds (bounded). Retain width-independent “unwrapped” shaping
        // results and reuse them across wrap-width changes (reduces `Text::prepare` churn under
        // resize jitter; large win for editor resize probes).
        std::env::var("FRET_TEXT_UNWRAPPED_LAYOUT_CACHE_ENTRIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            // Allow disabling via env var (`0`).
            .unwrap_or(default_unwrapped_layout_cache_entries())
            .min(8192)
    })
}

fn default_unwrapped_layout_cache_entries() -> usize {
    // Keep wasm builds conservative; native builds get a bounded default to improve interactive
    // resize and editor-class text workloads.
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        1024
    }
}

fn unwrapped_layout_cache_max_text_len_bytes() -> usize {
    static MAX_BYTES: OnceLock<usize> = OnceLock::new();
    *MAX_BYTES.get_or_init(|| {
        // Do not cache huge single-line paragraphs by default. The wrap-from-unwrapped fast path
        // targets UI chrome and short editor labels, not large documents.
        std::env::var("FRET_TEXT_UNWRAPPED_LAYOUT_CACHE_MAX_TEXT_LEN_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(4096)
            .max(64)
            .min(1_048_576)
    })
}

fn measure_shaping_cache_entries() -> usize {
    static ENTRIES: OnceLock<usize> = OnceLock::new();
    *ENTRIES.get_or_init(|| {
        // Default: 4096 entries. This cache is the main defense against `TextService::measure`
        // reshaping thrash when a layout pass touches many unique strings (common in editor
        // surfaces) and when wrap widths churn during interactive resize.
        std::env::var("FRET_TEXT_MEASURE_SHAPING_CACHE_ENTRIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(4096)
            .max(64)
            .min(65_536)
    })
}

fn measure_shaping_cache_min_text_len_bytes() -> usize {
    static MIN_BYTES: OnceLock<usize> = OnceLock::new();
    *MIN_BYTES.get_or_init(|| {
        // Default: cache only "meaningfully expensive" paragraphs (e.g. long editor lines).
        //
        // Short UI labels (menus/tabs/buttons) are typically cheap to shape, and caching every
        // distinct label can bloat the cache and degrade cache locality across long-lived
        // reuse-launch perf suites.
        std::env::var("FRET_TEXT_MEASURE_SHAPING_CACHE_MIN_TEXT_LEN_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(128)
            .min(1_048_576)
    })
}

struct FretFallback;

impl cosmic_text::Fallback for FretFallback {
    fn common_fallback(&self) -> &[&'static str] {
        // For Web/WASM, there are no system fonts. If the app bundles fonts (e.g. via `fret-fonts`
        // feature flags), include those families in the fallback chain so mixed-script text works
        // without explicit per-span font selection.
        #[cfg(target_arch = "wasm32")]
        {
            &[
                // UI (bundled in `fret-fonts` bootstrap)
                "Inter",
                // CJK (bundled via `fret-fonts/cjk-lite`)
                "Noto Sans CJK SC",
                // Emoji (bundled via `fret-fonts/emoji`)
                "Noto Color Emoji",
            ]
        }
        #[cfg(target_os = "windows")]
        {
            &[
                // UI
                "Segoe UI",
                "Tahoma",
                // CJK
                "Microsoft YaHei UI",
                "Microsoft YaHei",
                "Yu Gothic UI",
                "Meiryo UI",
                "Meiryo",
                "Nirmala UI",
                // Bundled/portable fallbacks (if available)
                "Noto Sans CJK SC",
                // Emoji
                "Segoe UI Emoji",
                "Segoe UI Symbol",
                "Noto Color Emoji",
            ]
        }
        #[cfg(target_os = "macos")]
        {
            &[
                // UI (attempt a couple of common names; fontdb will skip missing families)
                "SF Pro Text",
                ".SF NS Text",
                "Helvetica Neue",
                // CJK
                "PingFang SC",
                "PingFang TC",
                "Hiragino Sans",
                // Emoji
                "Apple Color Emoji",
                // Bundled/portable fallbacks (if available)
                "Noto Sans CJK SC",
                "Noto Color Emoji",
            ]
        }
        #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
        {
            &[
                // UI
                "Noto Sans",
                "DejaVu Sans",
                "Liberation Sans",
                // CJK
                "Noto Sans CJK SC",
                "Noto Sans CJK JP",
                "Noto Sans CJK TC",
                // Emoji
                "Noto Color Emoji",
            ]
        }
        #[cfg(not(any(
            target_arch = "wasm32",
            target_os = "windows",
            target_os = "macos",
            all(unix, not(any(target_os = "macos", target_os = "android")))
        )))]
        {
            &[]
        }
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        <cosmic_text::PlatformFallback as cosmic_text::Fallback>::forbidden_fallback(
            &cosmic_text::PlatformFallback,
        )
    }

    fn script_fallback(&self, script: unicode_script::Script, locale: &str) -> &[&'static str] {
        <cosmic_text::PlatformFallback as cosmic_text::Fallback>::script_fallback(
            &cosmic_text::PlatformFallback,
            script,
            locale,
        )
    }
}

fn build_installed_family_set(db: &cosmic_text::fontdb::Database) -> HashSet<String> {
    let mut set = HashSet::new();
    for face in db.faces() {
        for (family, _lang) in &face.families {
            set.insert(family.to_ascii_lowercase());
        }
    }
    set
}

fn first_installed_family<'a>(
    installed: &HashSet<String>,
    candidates: &'a [&'a str],
) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|name| installed.contains(&name.to_ascii_lowercase()))
}

fn default_sans_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["Segoe UI", "Tahoma", "Arial"]
    }
    #[cfg(target_os = "macos")]
    {
        &["SF Pro Text", ".SF NS Text", "Helvetica Neue", "Helvetica"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["Noto Sans", "DejaVu Sans", "Liberation Sans"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        &[]
    }
}

fn default_monospace_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["Cascadia Mono", "Consolas", "Courier New"]
    }
    #[cfg(target_os = "macos")]
    {
        &["SF Mono", "Menlo", "Monaco"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["Noto Sans Mono", "DejaVu Sans Mono", "Liberation Mono"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        &[]
    }
}

fn default_serif_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["Times New Roman", "Georgia"]
    }
    #[cfg(target_os = "macos")]
    {
        &["New York", "Times New Roman", "Times"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["DejaVu Serif", "Noto Serif", "Liberation Serif"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        &[]
    }
}

fn font_stack_cache_key(
    locale: &str,
    db: &cosmic_text::fontdb::Database,
    db_revision: u64,
    common_fallback_config: &[String],
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    locale.hash(&mut hasher);

    db.family_name(&Family::SansSerif).hash(&mut hasher);
    db.family_name(&Family::Serif).hash(&mut hasher);
    db.family_name(&Family::Monospace).hash(&mut hasher);

    // Include the framework-level fallback policy so changing it can't reuse stale blobs.
    <FretFallback as cosmic_text::Fallback>::common_fallback(&FretFallback).hash(&mut hasher);
    common_fallback_config.hash(&mut hasher);
    <cosmic_text::PlatformFallback as cosmic_text::Fallback>::forbidden_fallback(
        &cosmic_text::PlatformFallback,
    )
    .hash(&mut hasher);

    // Ensure font-db mutations (user font loading, web font injection, etc.) participate in the
    // cache key even when generic family names are unchanged.
    db_revision.hash(&mut hasher);

    hasher.finish()
}

#[derive(Debug, Clone)]
pub struct GlyphInstance {
    /// Logical-space rect relative to the text baseline origin.
    pub rect: [f32; 4],
    pub paint_span: Option<u16>,
    key: GlyphKey,
}

impl GlyphInstance {
    pub fn kind(&self) -> GlyphQuadKind {
        self.key.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphQuadKind {
    Mask,
    Color,
    Subpixel,
}

#[derive(Debug, Clone)]
pub struct TextBlob {
    pub shape: Arc<TextShape>,
    pub paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
    pub decorations: Arc<[TextDecoration]>,
    ref_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecorationKind {
    Underline,
    Strikethrough,
}

#[derive(Debug, Clone)]
pub struct TextDecoration {
    pub kind: TextDecorationKind,
    /// Rect in the same coordinate space as `TextService::selection_rects` (y=0 at top of text box).
    pub rect: Rect,
    /// When present, uses `TextBlob.paint_palette[paint_span]` as the base color if no explicit override exists.
    pub paint_span: Option<u16>,
    /// Optional explicit decoration color override.
    pub color: Option<fret_core::Color>,
}

#[derive(Debug, Clone)]
pub struct TextShape {
    pub glyphs: Arc<[GlyphInstance]>,
    pub metrics: TextMetrics,
    pub lines: Arc<[TextLine]>,
    pub caret_stops: Arc<[(usize, Px)]>,
}

#[derive(Debug, Clone)]
pub struct TextLine {
    pub start: usize,
    pub end: usize,
    #[allow(dead_code)]
    pub width: Px,
    pub y_top: Px,
    /// Baseline Y for this line (y=0 at top of text box).
    pub y_baseline: Px,
    pub height: Px,
    pub caret_stops: Vec<(usize, Px)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextBlobKey {
    text: Arc<str>,
    spans_shaping_key: u64,
    spans_paint_key: u64,
    backend: u8,
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_bits: Option<u32>,
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    overflow: TextOverflow,
    scale_bits: u32,
}

impl TextBlobKey {
    fn new(
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        font_stack_key: u64,
    ) -> Self {
        let max_width_bits = match constraints.wrap {
            // `TextWrap::None` does not change shaping results based on width unless we need to
            // materialize an overflow policy (ellipsis). Callers clip at higher levels.
            TextWrap::None if constraints.overflow != TextOverflow::Ellipsis => None,
            _ => constraints.max_width.map(|w| w.0.to_bits()),
        };
        Self {
            text: Arc::<str>::from(text),
            spans_shaping_key: 0,
            spans_paint_key: 0,
            backend: 0,
            font: style.font.clone(),
            font_stack_key,
            size_bits: style.size.0.to_bits(),
            weight: style.weight.0,
            slant: match style.slant {
                TextSlant::Normal => 0,
                TextSlant::Italic => 1,
                TextSlant::Oblique => 2,
            },
            line_height_bits: style.line_height.map(|px| px.0.to_bits()),
            letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
            max_width_bits,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            scale_bits: constraints.scale_factor.to_bits(),
        }
    }

    fn new_attributed(
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
        font_stack_key: u64,
    ) -> Self {
        let mut out = Self::new(rich.text.as_ref(), base_style, constraints, font_stack_key);
        out.spans_shaping_key = spans_shaping_fingerprint(&rich.spans);
        out.spans_paint_key = spans_paint_fingerprint(&rich.spans);
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextShapeKey {
    text: Arc<str>,
    spans_shaping_key: u64,
    backend: u8,
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_bits: Option<u32>,
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    overflow: TextOverflow,
    scale_bits: u32,
}

impl TextShapeKey {
    fn from_blob_key(key: &TextBlobKey) -> Self {
        Self {
            text: key.text.clone(),
            spans_shaping_key: key.spans_shaping_key,
            backend: key.backend,
            font: key.font.clone(),
            font_stack_key: key.font_stack_key,
            size_bits: key.size_bits,
            weight: key.weight,
            slant: key.slant,
            line_height_bits: key.line_height_bits,
            letter_spacing_bits: key.letter_spacing_bits,
            max_width_bits: key.max_width_bits,
            wrap: key.wrap,
            overflow: key.overflow,
            scale_bits: key.scale_bits,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FontFaceKey {
    blob_id: u64,
    face_index: u32,
    variation_key: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    font: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    kind: GlyphQuadKind,
}

fn stable_font_blob_id(bytes: &[u8]) -> u64 {
    // Stable, dependency-free fingerprint for font bytes.
    // This intentionally avoids `DefaultHasher` to stay deterministic across Rust versions.
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    fn fnv1a64_update(mut hash: u64, input: &[u8]) -> u64 {
        for b in input {
            hash ^= u64::from(*b);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv1a64_update(hash, b"fret.text.font_blob_id.v1\0");
    hash = fnv1a64_update(hash, &(bytes.len() as u64).to_le_bytes());
    fnv1a64_update(hash, bytes)
}

fn subpixel_bin_q4(pos: f32) -> (i32, u8) {
    // Keep behavior aligned with cosmic-text's `SubpixelBin::new`.
    let trunc = pos as i32;
    let fract = pos - trunc as f32;

    if pos.is_sign_negative() {
        if fract > -0.125 {
            (trunc, 0)
        } else if fract > -0.375 {
            (trunc - 1, 3)
        } else if fract > -0.625 {
            (trunc - 1, 2)
        } else if fract > -0.875 {
            (trunc - 1, 1)
        } else {
            (trunc - 1, 0)
        }
    } else {
        #[allow(clippy::collapsible_else_if)]
        if fract < 0.125 {
            (trunc, 0)
        } else if fract < 0.375 {
            (trunc, 1)
        } else if fract < 0.625 {
            (trunc, 2)
        } else if fract < 0.875 {
            (trunc, 3)
        } else {
            (trunc + 1, 0)
        }
    }
}

fn subpixel_bin_as_float(bin: u8) -> f32 {
    match bin {
        0 => 0.0,
        1 => 0.25,
        2 => 0.5,
        3 => 0.75,
        _ => 0.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextQualitySettings {
    /// Gamma parameter for shader-side alpha correction.
    ///
    /// This is used to derive the `gamma_ratios` uniform (GPUI-aligned). Values are clamped to
    /// `[1.0, 2.2]`.
    pub gamma: f32,
    /// Enhanced contrast factor for grayscale (mask) glyph sampling.
    pub grayscale_enhanced_contrast: f32,
    /// Enhanced contrast factor for subpixel (RGB coverage) glyph sampling.
    pub subpixel_enhanced_contrast: f32,
}

impl Default for TextQualitySettings {
    fn default() -> Self {
        // Windows-first defaults, aligned with the Zed/GPUI baseline (see ADR 0109/0157).
        Self {
            gamma: 1.8,
            grayscale_enhanced_contrast: 1.0,
            subpixel_enhanced_contrast: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextQualityState {
    gamma: f32,
    gamma_ratios: [f32; 4],
    grayscale_enhanced_contrast: f32,
    subpixel_enhanced_contrast: f32,
    key: u64,
}

impl TextQualityState {
    fn new(settings: TextQualitySettings) -> Self {
        let gamma = settings.gamma.clamp(1.0, 2.2);
        let grayscale_enhanced_contrast = settings.grayscale_enhanced_contrast.max(0.0);
        let subpixel_enhanced_contrast = settings.subpixel_enhanced_contrast.max(0.0);

        let gamma_ratios = gamma_correction_ratios(gamma);

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        gamma.to_bits().hash(&mut hasher);
        grayscale_enhanced_contrast.to_bits().hash(&mut hasher);
        subpixel_enhanced_contrast.to_bits().hash(&mut hasher);
        let key = hasher.finish();

        Self {
            gamma,
            gamma_ratios,
            grayscale_enhanced_contrast,
            subpixel_enhanced_contrast,
            key,
        }
    }
}

// Adapted from the Microsoft Terminal alpha correction tables (via Zed/GPUI).
// See ADR 0029 / ADR 0109 for the rationale and references.
fn gamma_correction_ratios(gamma: f32) -> [f32; 4] {
    const GAMMA_INCORRECT_TARGET_RATIOS: [[f32; 4]; 13] = [
        [0.0000 / 4.0, 0.0000 / 4.0, 0.0000 / 4.0, 0.0000 / 4.0], // gamma = 1.0
        [0.0166 / 4.0, -0.0807 / 4.0, 0.2227 / 4.0, -0.0751 / 4.0], // gamma = 1.1
        [0.0350 / 4.0, -0.1760 / 4.0, 0.4325 / 4.0, -0.1370 / 4.0], // gamma = 1.2
        [0.0543 / 4.0, -0.2821 / 4.0, 0.6302 / 4.0, -0.1876 / 4.0], // gamma = 1.3
        [0.0739 / 4.0, -0.3963 / 4.0, 0.8167 / 4.0, -0.2287 / 4.0], // gamma = 1.4
        [0.0933 / 4.0, -0.5161 / 4.0, 0.9926 / 4.0, -0.2616 / 4.0], // gamma = 1.5
        [0.1121 / 4.0, -0.6395 / 4.0, 1.1588 / 4.0, -0.2877 / 4.0], // gamma = 1.6
        [0.1300 / 4.0, -0.7649 / 4.0, 1.3159 / 4.0, -0.3080 / 4.0], // gamma = 1.7
        [0.1469 / 4.0, -0.8911 / 4.0, 1.4644 / 4.0, -0.3234 / 4.0], // gamma = 1.8
        [0.1627 / 4.0, -1.0170 / 4.0, 1.6051 / 4.0, -0.3347 / 4.0], // gamma = 1.9
        [0.1773 / 4.0, -1.1420 / 4.0, 1.7385 / 4.0, -0.3426 / 4.0], // gamma = 2.0
        [0.1908 / 4.0, -1.2652 / 4.0, 1.8650 / 4.0, -0.3476 / 4.0], // gamma = 2.1
        [0.2031 / 4.0, -1.3864 / 4.0, 1.9851 / 4.0, -0.3501 / 4.0], // gamma = 2.2
    ];

    const NORM13: f32 = ((0x10000 as f64) / (255.0 * 255.0) * 4.0) as f32;
    const NORM24: f32 = ((0x100 as f64) / (255.0) * 4.0) as f32;

    let index = ((gamma * 10.0).round() as usize).clamp(10, 22) - 10;
    let ratios = GAMMA_INCORRECT_TARGET_RATIOS[index];
    [
        ratios[0] * NORM13,
        ratios[1] * NORM24,
        ratios[2] * NORM13,
        ratios[3] * NORM24,
    ]
}

const SUBPIXEL_VARIANTS_X: u8 = 4;
const SUBPIXEL_VARIANTS_Y: u8 = if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
    1
} else {
    SUBPIXEL_VARIANTS_X
};

fn subpixel_bin_y(pos: f32) -> (i32, u8) {
    let (y, bin) = subpixel_bin_q4(pos);
    if SUBPIXEL_VARIANTS_Y <= 1 {
        (y, 0)
    } else {
        (y, bin)
    }
}

const TEXT_ATLAS_MAX_PAGES: usize = 2;

#[derive(Debug, Default, Clone, Copy)]
struct GlyphAtlasFramePerf {
    hits: u64,
    misses: u64,
    inserts: u64,
    evict_glyphs: u64,
    evict_pages: u64,
    out_of_space: u64,
    too_large: u64,
    pending_uploads: u64,
    pending_upload_bytes: u64,
    upload_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
struct GlyphAtlasEntry {
    page: u16,
    alloc_id: etagere::AllocId,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    placement_left: i32,
    placement_top: i32,
    live_refs: u32,
    last_used_epoch: u64,
}

#[derive(Debug)]
struct PendingUpload {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
}

#[derive(Debug, Default, Clone, Copy)]
struct GlyphAtlasPerfSnapshot {
    uploads: u64,
    upload_bytes: u64,
    evicted_glyphs: u64,
    evicted_pages: u64,
    evicted_page_glyphs: u64,
    resets: u64,
}

#[derive(Debug, Default)]
struct GlyphAtlasPerfStats {
    uploads: u64,
    upload_bytes: u64,
    evicted_glyphs: u64,
    evicted_pages: u64,
    evicted_page_glyphs: u64,
    resets: u64,
}

struct GlyphAtlasPage {
    allocator: etagere::BucketedAtlasAllocator,
    pending: Vec<PendingUpload>,
    live_glyph_refs: u32,
    last_used_epoch: u64,
    bind_group: wgpu::BindGroup,
    _texture: wgpu::Texture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlyphAtlasInsertError {
    OutOfSpace,
    TooLarge,
}

struct GlyphAtlas {
    width: u32,
    height: u32,
    padding_px: u32,
    pages: Vec<GlyphAtlasPage>,
    glyphs: HashMap<GlyphKey, GlyphAtlasEntry>,
    revision: u64,
    used_px: u64,
    perf_frame: GlyphAtlasFramePerf,
    perf: GlyphAtlasPerfStats,
}

impl GlyphAtlas {
    fn new(
        device: &wgpu::Device,
        atlas_bind_group_layout: &wgpu::BindGroupLayout,
        atlas_sampler: &wgpu::Sampler,
        label_prefix: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        page_count: usize,
    ) -> Self {
        let padding_px = 1;
        let mut pages: Vec<GlyphAtlasPage> = Vec::with_capacity(page_count.max(1));

        for i in 0..page_count.max(1) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("{label_prefix} page {i}")),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{label_prefix} bind group page {i}")),
                layout: atlas_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(atlas_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
            });

            pages.push(GlyphAtlasPage {
                allocator: etagere::BucketedAtlasAllocator::new(etagere::Size::new(
                    width as i32,
                    height as i32,
                )),
                pending: Vec::new(),
                live_glyph_refs: 0,
                last_used_epoch: 0,
                bind_group,
                _texture: texture,
            });
        }

        Self {
            width,
            height,
            padding_px,
            pages,
            glyphs: HashMap::new(),
            revision: 0,
            used_px: 0,
            perf_frame: GlyphAtlasFramePerf::default(),
            perf: GlyphAtlasPerfStats::default(),
        }
    }

    fn begin_frame_diagnostics(&mut self) {
        self.perf_frame = GlyphAtlasFramePerf::default();
    }

    fn diagnostics_snapshot(&self) -> fret_core::RendererGlyphAtlasPerfSnapshot {
        let pages = self.pages.len() as u32;
        let capacity_px = u64::from(self.width)
            .saturating_mul(u64::from(self.height))
            .saturating_mul(u64::from(pages.max(1)));
        fret_core::RendererGlyphAtlasPerfSnapshot {
            width: self.width,
            height: self.height,
            pages,
            entries: self.glyphs.len() as u64,
            used_px: self.used_px,
            capacity_px,
            frame_hits: self.perf_frame.hits,
            frame_misses: self.perf_frame.misses,
            frame_inserts: self.perf_frame.inserts,
            frame_evict_glyphs: self.perf_frame.evict_glyphs,
            frame_evict_pages: self.perf_frame.evict_pages,
            frame_out_of_space: self.perf_frame.out_of_space,
            frame_too_large: self.perf_frame.too_large,
            frame_pending_uploads: self.perf_frame.pending_uploads,
            frame_pending_upload_bytes: self.perf_frame.pending_upload_bytes,
            frame_upload_bytes: self.perf_frame.upload_bytes,
        }
    }

    fn take_perf_snapshot(&mut self) -> GlyphAtlasPerfSnapshot {
        let snap = GlyphAtlasPerfSnapshot {
            uploads: self.perf.uploads,
            upload_bytes: self.perf.upload_bytes,
            evicted_glyphs: self.perf.evicted_glyphs,
            evicted_pages: self.perf.evicted_pages,
            evicted_page_glyphs: self.perf.evicted_page_glyphs,
            resets: self.perf.resets,
        };
        self.perf = GlyphAtlasPerfStats::default();
        snap
    }

    fn reset(&mut self) {
        self.perf.resets = self.perf.resets.saturating_add(1);
        self.revision = self.revision.saturating_add(1);
        self.glyphs.clear();
        self.used_px = 0;
        for page in &mut self.pages {
            page.allocator = etagere::BucketedAtlasAllocator::new(etagere::Size::new(
                self.width as i32,
                self.height as i32,
            ));
            page.pending.clear();
            page.live_glyph_refs = 0;
            page.last_used_epoch = 0;
        }
    }

    fn bind_group(&self, page: u16) -> &wgpu::BindGroup {
        let idx = (page as usize).min(self.pages.len().saturating_sub(1));
        &self.pages[idx].bind_group
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn entry(&self, key: GlyphKey) -> Option<GlyphAtlasEntry> {
        self.glyphs.get(&key).copied()
    }

    fn get(&mut self, key: GlyphKey, epoch: u64) -> Option<GlyphAtlasEntry> {
        let Some(hit) = self.glyphs.get_mut(&key) else {
            self.perf_frame.misses = self.perf_frame.misses.saturating_add(1);
            return None;
        };
        self.perf_frame.hits = self.perf_frame.hits.saturating_add(1);
        hit.last_used_epoch = epoch;
        let idx = (hit.page as usize).min(self.pages.len().saturating_sub(1));
        self.pages[idx].last_used_epoch = epoch;
        Some(*hit)
    }

    fn inc_live_ref(&mut self, key: GlyphKey) {
        let Some(entry) = self.glyphs.get_mut(&key) else {
            return;
        };
        if entry.live_refs == 0 {
            let idx = (entry.page as usize).min(self.pages.len().saturating_sub(1));
            self.pages[idx].live_glyph_refs = self.pages[idx].live_glyph_refs.saturating_add(1);
        }
        entry.live_refs = entry.live_refs.saturating_add(1);
    }

    fn dec_live_ref(&mut self, key: GlyphKey) {
        let Some(entry) = self.glyphs.get_mut(&key) else {
            return;
        };
        if entry.live_refs == 0 {
            return;
        }
        entry.live_refs -= 1;
        if entry.live_refs == 0 {
            let idx = (entry.page as usize).min(self.pages.len().saturating_sub(1));
            if self.pages[idx].live_glyph_refs > 0 {
                self.pages[idx].live_glyph_refs -= 1;
            }
        }
    }

    fn inc_live_refs(&mut self, keys: &[GlyphKey]) {
        for &k in keys {
            self.inc_live_ref(k);
        }
    }

    fn dec_live_refs(&mut self, keys: &[GlyphKey]) {
        for &k in keys {
            self.dec_live_ref(k);
        }
    }

    fn evict_lru_unreferenced_glyph(&mut self) -> bool {
        let mut victim: Option<(GlyphKey, GlyphAtlasEntry)> = None;
        for (&k, &e) in &self.glyphs {
            if e.live_refs > 0 {
                continue;
            }
            let pick = match victim {
                None => true,
                Some((_, prev)) => e.last_used_epoch < prev.last_used_epoch,
            };
            if pick {
                victim = Some((k, e));
            }
        }

        let Some((victim_key, victim_entry)) = victim else {
            return false;
        };

        let pad = self.padding_px;
        let w_pad = victim_entry.w.saturating_add(pad.saturating_mul(2));
        let h_pad = victim_entry.h.saturating_add(pad.saturating_mul(2));
        self.used_px = self
            .used_px
            .saturating_sub(u64::from(w_pad).saturating_mul(u64::from(h_pad)));

        let page_idx = (victim_entry.page as usize).min(self.pages.len().saturating_sub(1));
        self.pages[page_idx]
            .allocator
            .deallocate(victim_entry.alloc_id);
        self.glyphs.remove(&victim_key);
        self.perf.evicted_glyphs = self.perf.evicted_glyphs.saturating_add(1);
        self.revision = self.revision.saturating_add(1);
        self.perf_frame.evict_glyphs = self.perf_frame.evict_glyphs.saturating_add(1);
        true
    }

    fn evict_lru_unreferenced_page(&mut self) -> bool {
        let mut victim: Option<usize> = None;
        for (idx, page) in self.pages.iter().enumerate() {
            if page.live_glyph_refs > 0 {
                continue;
            }
            let pick = match victim {
                None => true,
                Some(prev) => page.last_used_epoch < self.pages[prev].last_used_epoch,
            };
            if pick {
                victim = Some(idx);
            }
        }

        let Some(victim) = victim else {
            return false;
        };

        self.pages[victim].allocator = etagere::BucketedAtlasAllocator::new(etagere::Size::new(
            self.width as i32,
            self.height as i32,
        ));
        self.pages[victim].pending.clear();
        self.pages[victim].last_used_epoch = 0;
        self.pages[victim].live_glyph_refs = 0;

        let victim_page = victim as u16;
        let keys_to_remove: Vec<GlyphKey> = self
            .glyphs
            .iter()
            .filter_map(|(k, e)| (e.page == victim_page).then_some(*k))
            .collect();
        let pad = self.padding_px;
        self.perf.evicted_pages = self.perf.evicted_pages.saturating_add(1);
        self.perf.evicted_page_glyphs = self
            .perf
            .evicted_page_glyphs
            .saturating_add(keys_to_remove.len() as u64);
        for k in keys_to_remove {
            if let Some(entry) = self.glyphs.remove(&k) {
                let w_pad = entry.w.saturating_add(pad.saturating_mul(2));
                let h_pad = entry.h.saturating_add(pad.saturating_mul(2));
                self.used_px = self
                    .used_px
                    .saturating_sub(u64::from(w_pad).saturating_mul(u64::from(h_pad)));
            }
        }

        self.revision = self.revision.saturating_add(1);
        self.perf_frame.evict_pages = self.perf_frame.evict_pages.saturating_add(1);
        true
    }

    fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        for page in &mut self.pages {
            for upload in std::mem::take(&mut page.pending) {
                if upload.w == 0 || upload.h == 0 {
                    continue;
                }

                let bytes_per_row = upload.w.saturating_mul(upload.bytes_per_pixel);
                if bytes_per_row == 0 {
                    continue;
                }

                let expected_len = (bytes_per_row as usize).saturating_mul(upload.h as usize);
                debug_assert_eq!(upload.data.len(), expected_len);
                if upload.data.len() != expected_len {
                    continue;
                }

                let aligned_bytes_per_row = bytes_per_row
                    .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                    * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
                let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);

                let mut owned: Vec<u8> = Vec::new();
                let bytes: &[u8] = if aligned_bytes_per_row == bytes_per_row {
                    &upload.data
                } else {
                    owned.resize(
                        (aligned_bytes_per_row as usize).saturating_mul(upload.h as usize),
                        0,
                    );
                    for row in 0..upload.h as usize {
                        let src0 = row.saturating_mul(bytes_per_row as usize);
                        let src1 = src0.saturating_add(bytes_per_row as usize);
                        let dst0 = row.saturating_mul(aligned_bytes_per_row as usize);
                        let dst1 = dst0.saturating_add(bytes_per_row as usize);
                        owned[dst0..dst1].copy_from_slice(&upload.data[src0..src1]);
                    }
                    &owned
                };

                self.perf.uploads = self.perf.uploads.saturating_add(1);
                self.perf.upload_bytes = self.perf.upload_bytes.saturating_add(bytes.len() as u64);
                self.perf_frame.upload_bytes = self
                    .perf_frame
                    .upload_bytes
                    .saturating_add(bytes.len() as u64);

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &page._texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: upload.x,
                            y: upload.y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    bytes,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(aligned_bytes_per_row),
                        rows_per_image: Some(upload.h),
                    },
                    wgpu::Extent3d {
                        width: upload.w,
                        height: upload.h,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    fn get_or_insert(
        &mut self,
        key: GlyphKey,
        w: u32,
        h: u32,
        placement_left: i32,
        placement_top: i32,
        bytes_per_pixel: u32,
        data: Vec<u8>,
        epoch: u64,
    ) -> Result<GlyphAtlasEntry, GlyphAtlasInsertError> {
        if let Some(hit) = self.glyphs.get_mut(&key) {
            self.perf_frame.hits = self.perf_frame.hits.saturating_add(1);
            hit.last_used_epoch = epoch;
            let idx = (hit.page as usize).min(self.pages.len().saturating_sub(1));
            self.pages[idx].last_used_epoch = epoch;
            return Ok(*hit);
        }
        self.perf_frame.misses = self.perf_frame.misses.saturating_add(1);

        let pad = self.padding_px;
        let w_pad = w.saturating_add(pad.saturating_mul(2));
        let h_pad = h.saturating_add(pad.saturating_mul(2));
        if w == 0 || h == 0 || w_pad == 0 || h_pad == 0 || w_pad > self.width || h_pad > self.height
        {
            self.perf_frame.too_large = self.perf_frame.too_large.saturating_add(1);
            return Err(GlyphAtlasInsertError::TooLarge);
        }

        let size = etagere::Size::new(w_pad as i32, h_pad as i32);

        for _ in 0..=self.pages.len() {
            for (page_index, page) in self.pages.iter_mut().enumerate() {
                let Some(allocation) = page.allocator.allocate(size) else {
                    continue;
                };

                let Ok(base_x) = u32::try_from(allocation.rectangle.min.x) else {
                    page.allocator.deallocate(allocation.id);
                    continue;
                };
                let Ok(base_y) = u32::try_from(allocation.rectangle.min.y) else {
                    page.allocator.deallocate(allocation.id);
                    continue;
                };

                let x = base_x.saturating_add(pad);
                let y = base_y.saturating_add(pad);

                page.pending.push(PendingUpload {
                    x,
                    y,
                    w,
                    h,
                    bytes_per_pixel,
                    data,
                });
                page.last_used_epoch = epoch;

                self.perf_frame.inserts = self.perf_frame.inserts.saturating_add(1);
                self.perf_frame.pending_uploads = self.perf_frame.pending_uploads.saturating_add(1);
                self.perf_frame.pending_upload_bytes =
                    self.perf_frame.pending_upload_bytes.saturating_add(
                        u64::from(w)
                            .saturating_mul(u64::from(h))
                            .saturating_mul(u64::from(bytes_per_pixel)),
                    );
                self.used_px = self
                    .used_px
                    .saturating_add(u64::from(w_pad).saturating_mul(u64::from(h_pad)));

                let entry = GlyphAtlasEntry {
                    page: page_index as u16,
                    alloc_id: allocation.id,
                    x,
                    y,
                    w,
                    h,
                    placement_left,
                    placement_top,
                    live_refs: 0,
                    last_used_epoch: epoch,
                };
                self.glyphs.insert(key, entry);
                self.revision = self.revision.saturating_add(1);
                return Ok(entry);
            }

            if self.evict_lru_unreferenced_glyph() {
                continue;
            }
            if self.evict_lru_unreferenced_page() {
                continue;
            }
            self.perf_frame.out_of_space = self.perf_frame.out_of_space.saturating_add(1);
            return Err(GlyphAtlasInsertError::OutOfSpace);
        }

        self.perf_frame.out_of_space = self.perf_frame.out_of_space.saturating_add(1);
        Err(GlyphAtlasInsertError::OutOfSpace)
    }
}

#[allow(dead_code)]
fn subpixel_mask_to_alpha(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 4);
    for rgba in data.chunks_exact(4) {
        out.push(rgba[0].max(rgba[1]).max(rgba[2]));
    }
    out
}

fn collect_font_names(db: &cosmic_text::fontdb::Database) -> Vec<String> {
    let mut by_lower: HashMap<String, String> = HashMap::new();

    for face in db.faces() {
        for (family, _lang) in &face.families {
            let key = family.to_ascii_lowercase();
            by_lower.entry(key).or_insert_with(|| family.clone());
        }
    }

    for family in [
        db.family_name(&Family::SansSerif),
        db.family_name(&Family::Serif),
        db.family_name(&Family::Monospace),
    ] {
        let key = family.to_ascii_lowercase();
        by_lower.entry(key).or_insert_with(|| family.to_string());
    }

    let mut names: Vec<String> = by_lower.into_values().collect();
    names.sort_unstable_by(|a, b| {
        a.to_ascii_lowercase()
            .cmp(&b.to_ascii_lowercase())
            .then(a.cmp(b))
    });
    names
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextMeasureKey {
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_bits: Option<u32>,
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    overflow: TextOverflow,
    scale_bits: u32,
}

impl TextMeasureKey {
    fn new(style: &TextStyle, constraints: TextConstraints, font_stack_key: u64) -> Self {
        let max_width_bits = match constraints.wrap {
            // `TextWrap::None` does not change shaping results based on width; callers clamp or
            // apply overflow policy at higher levels. Normalize away width so repeated measurements
            // (e.g. layout engine intrinsic probes) can reuse cached metrics.
            TextWrap::None => None,
            TextWrap::Word | TextWrap::Grapheme => constraints.max_width.map(|w| w.0.to_bits()),
        };
        Self {
            font: style.font.clone(),
            font_stack_key,
            size_bits: style.size.0.to_bits(),
            weight: style.weight.0,
            slant: match style.slant {
                TextSlant::Normal => 0,
                TextSlant::Italic => 1,
                TextSlant::Oblique => 2,
            },
            line_height_bits: style.line_height.map(|px| px.0.to_bits()),
            letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
            max_width_bits,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            scale_bits: constraints.scale_factor.to_bits(),
        }
    }
}

#[derive(Debug, Clone)]
struct TextMeasureEntry {
    text_hash: u64,
    spans_hash: u64,
    text: Arc<str>,
    spans: Option<Arc<[TextSpan]>>,
    metrics: TextMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextMeasureShapingKey {
    text_hash: u64,
    text_len: usize,
    spans_shaping_key: u64,
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_bits: Option<u32>,
    scale_bits: u32,
}

#[derive(Debug, Clone)]
struct TextMeasureShapingEntry {
    text: Arc<str>,
    spans: Option<Arc<[TextSpan]>>,
    width_px: f32,
    baseline_px: f32,
    line_height_px: f32,
    clusters: Arc<[parley_shaper::ShapedCluster]>,
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn spans_shaping_fingerprint(spans: &[TextSpan]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "fret.text.spans.shaping.v0".hash(&mut hasher);
    for s in spans {
        s.len.hash(&mut hasher);
        s.shaping.font.hash(&mut hasher);
        s.shaping.weight.hash(&mut hasher);
        s.shaping.slant.hash(&mut hasher);
        s.shaping
            .letter_spacing_em
            .map(|v| v.to_bits())
            .hash(&mut hasher);
    }
    hasher.finish()
}

fn spans_paint_fingerprint(spans: &[TextSpan]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "fret.text.spans.paint.v0".hash(&mut hasher);
    for s in spans {
        s.len.hash(&mut hasher);
        paint_fingerprint_color(&mut hasher, s.paint.fg);
        paint_fingerprint_color(&mut hasher, s.paint.bg);

        match s.paint.underline.as_ref() {
            None => 0u8.hash(&mut hasher),
            Some(u) => {
                1u8.hash(&mut hasher);
                paint_fingerprint_color(&mut hasher, u.color);
                std::mem::discriminant(&u.style).hash(&mut hasher);
            }
        }

        match s.paint.strikethrough.as_ref() {
            None => 0u8.hash(&mut hasher),
            Some(st) => {
                1u8.hash(&mut hasher);
                paint_fingerprint_color(&mut hasher, st.color);
                std::mem::discriminant(&st.style).hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

fn paint_fingerprint_color(
    hasher: &mut std::collections::hash_map::DefaultHasher,
    color: Option<fret_core::Color>,
) {
    match color {
        None => 0u8.hash(hasher),
        Some(c) => {
            1u8.hash(hasher);
            c.r.to_bits().hash(hasher);
            c.g.to_bits().hash(hasher);
            c.b.to_bits().hash(hasher);
            c.a.to_bits().hash(hasher);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextUnwrappedKey {
    text: Arc<str>,
    spans_shaping_key: u64,
    backend: u8,
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_bits: Option<u32>,
    scale_bits: u32,
}

impl TextUnwrappedKey {
    fn from_blob_key(key: &TextBlobKey) -> Self {
        Self {
            text: key.text.clone(),
            spans_shaping_key: key.spans_shaping_key,
            backend: key.backend,
            font: key.font.clone(),
            font_stack_key: key.font_stack_key,
            size_bits: key.size_bits,
            weight: key.weight,
            slant: key.slant,
            line_height_bits: key.line_height_bits,
            letter_spacing_bits: key.letter_spacing_bits,
            scale_bits: key.scale_bits,
        }
    }
}

pub struct TextSystem {
    font_system: FontSystem,
    parley_shaper: crate::text::parley_shaper::ParleyShaper,
    parley_scale: parley::swash::scale::ScaleContext,
    font_stack_key: u64,
    font_db_revision: u64,
    quality: TextQualityState,
    common_fallback_config: Vec<String>,

    blobs: SlotMap<TextBlobId, TextBlob>,
    blob_cache: HashMap<TextBlobKey, TextBlobId>,
    blob_key_by_id: HashMap<TextBlobId, TextBlobKey>,
    released_blob_lru: VecDeque<TextBlobId>,
    released_blob_set: HashSet<TextBlobId>,
    unwrapped_layout_cache:
        HashMap<TextUnwrappedKey, Arc<crate::text::parley_shaper::ShapedLineLayout>>,
    unwrapped_layout_lru: VecDeque<TextUnwrappedKey>,
    unwrapped_layout_set: HashSet<TextUnwrappedKey>,
    shape_cache: HashMap<TextShapeKey, Arc<TextShape>>,
    measure_cache: HashMap<TextMeasureKey, VecDeque<TextMeasureEntry>>,
    measure_shaping_cache: HashMap<TextMeasureShapingKey, TextMeasureShapingEntry>,
    measure_shaping_fifo: VecDeque<TextMeasureShapingKey>,

    mask_atlas: GlyphAtlas,
    color_atlas: GlyphAtlas,
    subpixel_atlas: GlyphAtlas,
    atlas_bind_group_layout: wgpu::BindGroupLayout,

    text_pin_mask: Vec<Vec<GlyphKey>>,
    text_pin_color: Vec<Vec<GlyphKey>>,
    text_pin_subpixel: Vec<Vec<GlyphKey>>,
    font_bytes_by_blob_id: HashMap<u64, Arc<[u8]>>,
    font_face_key_by_fontique: HashMap<(u64, u32), FontFaceKey>,

    perf_frame_cache_resets: u64,
    perf_frame_blob_cache_hits: u64,
    perf_frame_blob_cache_misses: u64,
    perf_frame_blobs_created: u64,
    perf_frame_shape_cache_hits: u64,
    perf_frame_shape_cache_misses: u64,
    perf_frame_shapes_created: u64,
    perf_frame_unwrapped_layout_cache_hits: u64,
    perf_frame_unwrapped_layout_cache_misses: u64,
    perf_frame_unwrapped_layouts_created: u64,

    glyph_atlas_epoch: u64,
}

enum WrappedForPrepare {
    Owned(crate::text::wrapper::WrappedLayout),
    UnwrappedWordLtr {
        kept_end: usize,
        unwrapped: Arc<crate::text::parley_shaper::ShapedLineLayout>,
        lines: Vec<crate::text::wrapper::WrappedLineSliceFromUnwrappedLtr>,
    },
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TextAtlasPerfSnapshot {
    pub(crate) uploads: u64,
    pub(crate) upload_bytes: u64,
    pub(crate) evicted_glyphs: u64,
    pub(crate) evicted_pages: u64,
    pub(crate) evicted_page_glyphs: u64,
    pub(crate) resets: u64,
}

pub type TextFontFamilyConfig = fret_core::TextFontFamilyConfig;

fn metrics_from_wrapped_lines(
    lines: &[crate::text::parley_shaper::ShapedLineLayout],
    scale: f32,
) -> TextMetrics {
    let snap_vertical = scale.is_finite() && scale.fract().abs() > 1e-4 && scale >= 1.0;

    let mut first_baseline_px = lines.first().map(|l| l.baseline.max(0.0)).unwrap_or(0.0);
    if snap_vertical && let Some(first) = lines.first() {
        let top_px = 0.0_f32;
        let bottom_px = (top_px + first.line_height.max(0.0)).round().max(top_px);
        let height_px = (bottom_px - top_px).max(0.0);
        first_baseline_px = (top_px + first.baseline.max(0.0))
            .round()
            .clamp(top_px, top_px + height_px);
    }

    let mut max_w_px = 0.0_f32;
    let mut total_h_px = 0.0_f32;
    if snap_vertical {
        let mut top_px = 0.0_f32;
        for line in lines {
            max_w_px = max_w_px.max(line.width.max(0.0));
            let bottom_px = (top_px + line.line_height.max(0.0)).round().max(top_px);
            top_px = bottom_px;
        }
        total_h_px = top_px;
    } else {
        for line in lines {
            max_w_px = max_w_px.max(line.width.max(0.0));
            total_h_px += line.line_height.max(0.0);
        }
    }

    TextMetrics {
        size: Size::new(
            Px((max_w_px / scale).max(0.0)),
            Px((total_h_px / scale).max(0.0)),
        ),
        baseline: Px((first_baseline_px / scale).max(0.0)),
    }
}

fn metrics_for_uniform_lines(
    max_w_px: f32,
    line_count: usize,
    baseline_px: f32,
    line_height_px: f32,
    scale: f32,
) -> TextMetrics {
    let snap_vertical = scale.is_finite() && scale.fract().abs() > 1e-4 && scale >= 1.0;

    let mut first_baseline_px = baseline_px.max(0.0);
    if snap_vertical {
        let top_px = 0.0_f32;
        let bottom_px = (top_px + line_height_px.max(0.0)).round().max(top_px);
        let height_px = (bottom_px - top_px).max(0.0);
        first_baseline_px = (top_px + baseline_px.max(0.0))
            .round()
            .clamp(top_px, top_px + height_px);
    }

    let total_h_px = if snap_vertical {
        let mut top_px = 0.0_f32;
        for _ in 0..line_count.max(1) {
            let bottom_px = (top_px + line_height_px.max(0.0)).round().max(top_px);
            top_px = bottom_px;
        }
        top_px
    } else {
        line_height_px.max(0.0) * (line_count.max(1) as f32)
    };

    TextMetrics {
        size: Size::new(
            Px((max_w_px / scale).max(0.0)),
            Px((total_h_px / scale).max(0.0)),
        ),
        baseline: Px((first_baseline_px / scale).max(0.0)),
    }
}

fn is_word_char_for_wrap(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(c, '\u{00C0}'..='\u{00FF}')
        || matches!(c, '\u{0100}'..='\u{017F}')
        || matches!(c, '\u{0180}'..='\u{024F}')
        || matches!(c, '\u{0400}'..='\u{04FF}')
        || matches!(c, '\u{1E00}'..='\u{1EFF}')
        || matches!(c, '\u{0300}'..='\u{036F}')
        || matches!(
            c,
            '-' | '_' | '.' | '\'' | '$' | '%' | '@' | '#' | '^' | '~' | ',' | '=' | ':' | '?'
        )
}

fn word_wrap_line_stats(
    text: &str,
    clusters: &[parley_shaper::ShapedCluster],
    max_width_px: f32,
) -> (usize, f32) {
    let end = text.len();
    if end == 0 || clusters.is_empty() {
        return (1, 0.0);
    }

    let mut line_count: usize = 0;
    let mut max_w_px: f32 = 0.0;

    let mut line_start_byte: usize = 0;
    let mut cluster_idx: usize = 0;

    while line_start_byte < end && cluster_idx < clusters.len() {
        let line_start_x = clusters[cluster_idx].x0;

        let mut last_fit_cluster_idx: Option<usize> = None;
        let mut last_fit_end_byte: usize = line_start_byte;

        let mut last_candidate_cluster_idx: Option<usize> = None;
        let mut last_candidate_byte: usize = line_start_byte;

        let mut first_non_whitespace: Option<usize> = None;
        let mut prev_ch: char = '\0';

        for (i, c) in clusters.iter().enumerate().skip(cluster_idx) {
            if c.text_range.start >= end {
                break;
            }
            if c.text_range.start < line_start_byte {
                continue;
            }

            let rel_x1 = c.x1 - line_start_x;
            if rel_x1 > max_width_px + 0.5 {
                break;
            }

            last_fit_cluster_idx = Some(i);
            last_fit_end_byte = c.text_range.end.min(end);

            let Some(ch) = text[c.text_range.start..].chars().next() else {
                continue;
            };

            if ch != ' ' && first_non_whitespace.is_none() {
                first_non_whitespace = Some(c.text_range.start);
            }

            if first_non_whitespace.is_some() {
                if is_word_char_for_wrap(ch) {
                    if prev_ch == ' ' && ch != ' ' {
                        last_candidate_cluster_idx = Some(i);
                        last_candidate_byte = c.text_range.start;
                    }
                } else if ch != ' ' {
                    last_candidate_cluster_idx = Some(i);
                    last_candidate_byte = c.text_range.start;
                }
            }

            prev_ch = ch;
        }

        let (cut_byte, next_cluster_idx, line_w_px) = if last_fit_end_byte >= end {
            let fit_idx = last_fit_cluster_idx.unwrap_or(cluster_idx);
            let end_x = clusters[fit_idx].x1;
            (end, clusters.len(), (end_x - line_start_x).max(0.0))
        } else if let Some(candidate_idx) = last_candidate_cluster_idx
            && last_candidate_byte > line_start_byte
        {
            let end_cluster_idx = candidate_idx.saturating_sub(1).max(cluster_idx);
            let end_x = clusters[end_cluster_idx].x1;
            (
                last_candidate_byte,
                candidate_idx,
                (end_x - line_start_x).max(0.0),
            )
        } else if let Some(fit_idx) = last_fit_cluster_idx {
            let end_x = clusters[fit_idx].x1;
            (
                last_fit_end_byte,
                fit_idx.saturating_add(1),
                (end_x - line_start_x).max(0.0),
            )
        } else {
            let c = &clusters[cluster_idx];
            let cut = c
                .text_range
                .end
                .min(end)
                .max(line_start_byte.saturating_add(1));
            let end_x = c.x1;
            (
                cut,
                cluster_idx.saturating_add(1),
                (end_x - line_start_x).max(0.0),
            )
        };

        max_w_px = max_w_px.max(line_w_px);
        line_count = line_count.saturating_add(1);

        if cut_byte <= line_start_byte {
            break;
        }
        line_start_byte = cut_byte;
        cluster_idx = next_cluster_idx;
    }

    (line_count.max(1), max_w_px)
}

fn grapheme_wrap_line_stats(
    text: &str,
    clusters: &[parley_shaper::ShapedCluster],
    max_width_px: f32,
) -> (usize, f32) {
    let end = text.len();
    if end == 0 || clusters.is_empty() {
        return (1, 0.0);
    }

    let mut line_count: usize = 0;
    let mut max_w_px: f32 = 0.0;

    let mut line_start_byte: usize = 0;
    let mut cluster_idx: usize = 0;

    while line_start_byte < end && cluster_idx < clusters.len() {
        let line_start_x = clusters[cluster_idx].x0;

        let mut last_fit_cluster_idx: Option<usize> = None;
        let mut last_fit_end_byte: usize = line_start_byte;

        for (i, c) in clusters.iter().enumerate().skip(cluster_idx) {
            if c.text_range.start >= end {
                break;
            }
            if c.text_range.start < line_start_byte {
                continue;
            }

            let rel_x1 = c.x1 - line_start_x;
            if rel_x1 > max_width_px + 0.5 {
                break;
            }

            last_fit_cluster_idx = Some(i);
            last_fit_end_byte = c.text_range.end.min(end);
        }

        let (cut_byte, next_cluster_idx, line_w_px) = if let Some(fit_idx) = last_fit_cluster_idx {
            let end_x = clusters[fit_idx].x1;
            (
                last_fit_end_byte,
                fit_idx.saturating_add(1),
                (end_x - line_start_x).max(0.0),
            )
        } else {
            let c = &clusters[cluster_idx];
            let cut = c
                .text_range
                .end
                .min(end)
                .max(line_start_byte.saturating_add(1));
            let end_x = c.x1;
            (
                cut,
                cluster_idx.saturating_add(1),
                (end_x - line_start_x).max(0.0),
            )
        };

        max_w_px = max_w_px.max(line_w_px);
        line_count = line_count.saturating_add(1);

        if cut_byte <= line_start_byte {
            break;
        }
        line_start_byte = cut_byte;
        cluster_idx = next_cluster_idx;
    }

    (line_count.max(1), max_w_px)
}

impl TextSystem {
    /// Returns a sorted list of available font family names.
    ///
    /// This is intended for settings/UI pickers. The result is best-effort and platform-dependent.
    pub fn all_font_names(&self) -> Vec<String> {
        collect_font_names(self.font_system.db())
    }

    pub fn font_stack_key(&self) -> u64 {
        self.font_stack_key
    }

    pub fn text_quality_key(&self) -> u64 {
        self.quality.key
    }

    pub fn text_quality_uniforms(&self) -> ([f32; 4], f32, f32) {
        (
            self.quality.gamma_ratios,
            self.quality.grayscale_enhanced_contrast,
            self.quality.subpixel_enhanced_contrast,
        )
    }

    pub fn begin_frame_diagnostics(&mut self) {
        self.perf_frame_cache_resets = 0;
        self.perf_frame_blob_cache_hits = 0;
        self.perf_frame_blob_cache_misses = 0;
        self.perf_frame_blobs_created = 0;
        self.perf_frame_shape_cache_hits = 0;
        self.perf_frame_shape_cache_misses = 0;
        self.perf_frame_shapes_created = 0;
        self.perf_frame_unwrapped_layout_cache_hits = 0;
        self.perf_frame_unwrapped_layout_cache_misses = 0;
        self.perf_frame_unwrapped_layouts_created = 0;
        self.mask_atlas.begin_frame_diagnostics();
        self.color_atlas.begin_frame_diagnostics();
        self.subpixel_atlas.begin_frame_diagnostics();
    }

    pub fn diagnostics_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextPerfSnapshot {
        fret_core::RendererTextPerfSnapshot {
            frame_id,
            font_stack_key: self.font_stack_key,
            font_db_revision: self.font_db_revision,
            blobs_live: self.blobs.len() as u64,
            blob_cache_entries: self.blob_cache.len() as u64,
            shape_cache_entries: self.shape_cache.len() as u64,
            measure_cache_buckets: self.measure_cache.len() as u64,
            unwrapped_layout_cache_entries: self.unwrapped_layout_cache.len() as u64,
            frame_unwrapped_layout_cache_hits: self.perf_frame_unwrapped_layout_cache_hits,
            frame_unwrapped_layout_cache_misses: self.perf_frame_unwrapped_layout_cache_misses,
            frame_unwrapped_layouts_created: self.perf_frame_unwrapped_layouts_created,
            frame_cache_resets: self.perf_frame_cache_resets,
            frame_blob_cache_hits: self.perf_frame_blob_cache_hits,
            frame_blob_cache_misses: self.perf_frame_blob_cache_misses,
            frame_blobs_created: self.perf_frame_blobs_created,
            frame_shape_cache_hits: self.perf_frame_shape_cache_hits,
            frame_shape_cache_misses: self.perf_frame_shape_cache_misses,
            frame_shapes_created: self.perf_frame_shapes_created,
            mask_atlas: self.mask_atlas.diagnostics_snapshot(),
            color_atlas: self.color_atlas.diagnostics_snapshot(),
            subpixel_atlas: self.subpixel_atlas.diagnostics_snapshot(),
        }
    }

    pub fn set_text_quality_settings(&mut self, settings: TextQualitySettings) -> bool {
        let next = TextQualityState::new(settings);
        if next == self.quality {
            return false;
        }
        self.quality = next;
        true
    }

    /// Adds font bytes (TTF/OTF/TTC) to the font database.
    ///
    /// Returns the number of newly loaded faces. When this returns non-zero, all cached text blobs
    /// and atlas entries are cleared to avoid reusing stale shaping/rasterization results.
    pub fn add_fonts(&mut self, fonts: impl IntoIterator<Item = Vec<u8>>) -> usize {
        let fonts: Vec<Vec<u8>> = fonts.into_iter().collect();

        let before_faces = self.font_system.db().faces().count();
        for data in fonts.iter().cloned() {
            self.font_system.db_mut().load_font_data(data);
        }
        let after_faces = self.font_system.db().faces().count();
        let added = after_faces.saturating_sub(before_faces);
        let parley_added = self.parley_shaper.add_fonts(fonts);

        if added > 0 || parley_added > 0 {
            self.font_db_revision = self.font_db_revision.saturating_add(1);
            self.font_stack_key = font_stack_cache_key(
                self.font_system.locale(),
                self.font_system.db(),
                self.font_db_revision,
                &self.common_fallback_config,
            );
            self.perf_frame_cache_resets = self.perf_frame_cache_resets.saturating_add(1);
            self.blobs.clear();
            self.blob_cache.clear();
            self.blob_key_by_id.clear();
            self.clear_released_blob_cache();
            self.clear_unwrapped_layout_cache();
            self.shape_cache.clear();
            self.measure_cache.clear();
            self.measure_shaping_cache.clear();
            self.measure_shaping_fifo.clear();
            self.mask_atlas.reset();
            self.color_atlas.reset();
            self.subpixel_atlas.reset();
            self.text_pin_mask.iter_mut().for_each(|v| v.clear());
            self.text_pin_color.iter_mut().for_each(|v| v.clear());
            self.text_pin_subpixel.iter_mut().for_each(|v| v.clear());
            self.font_bytes_by_blob_id.clear();
            self.font_face_key_by_fontique.clear();
        }

        added
    }

    pub fn new(device: &wgpu::Device) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret glyph atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret glyph atlas bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
            });

        let mask_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph mask atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::R8Unorm,
            TEXT_ATLAS_MAX_PAGES,
        );
        let color_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph color atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            TEXT_ATLAS_MAX_PAGES,
        );
        let subpixel_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph subpixel atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::Rgba8Unorm,
            TEXT_ATLAS_MAX_PAGES,
        );

        let (locale, mut db) = FontSystem::new().into_locale_and_db();
        let installed = build_installed_family_set(&db);

        let sans = first_installed_family(&installed, default_sans_candidates());
        if let Some(sans) = sans {
            db.set_sans_serif_family(sans);
        }
        let serif = first_installed_family(&installed, default_serif_candidates());
        if let Some(serif) = serif {
            db.set_serif_family(serif);
        }
        let mono = first_installed_family(&installed, default_monospace_candidates());
        if let Some(mono) = mono {
            db.set_monospace_family(mono);
        }

        let font_db_revision = 0u64;
        let common_fallback_config = Vec::new();
        let font_stack_key =
            font_stack_cache_key(&locale, &db, font_db_revision, &common_fallback_config);
        let font_system = FontSystem::new_with_locale_and_db_and_fallback(locale, db, FretFallback);

        let mut parley_shaper = crate::text::parley_shaper::ParleyShaper::new();
        if let Some(sans) = sans {
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::SansSerif, sans);
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::SystemUi, sans);
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::UiSansSerif, sans);
        }
        if let Some(serif) = serif {
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::Serif, serif);
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::UiSerif, serif);
        }
        if let Some(mono) = mono {
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::Monospace, mono);
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::UiMonospace, mono);
        }

        // Align Parley generic fallback ordering with the framework fallback chain so that Web/WASM
        // (no system fonts) can resolve mixed-script text without per-span font selection.
        let generics = [
            ParleyGenericFamily::SansSerif,
            ParleyGenericFamily::Serif,
            ParleyGenericFamily::Monospace,
            ParleyGenericFamily::SystemUi,
            ParleyGenericFamily::UiSansSerif,
            ParleyGenericFamily::UiSerif,
            ParleyGenericFamily::UiMonospace,
        ];
        for &family in <FretFallback as cosmic_text::Fallback>::common_fallback(&FretFallback) {
            for &generic in &generics {
                let _ = parley_shaper.append_generic_family_name(generic, family);
            }
        }

        let measure_shaping_entries = measure_shaping_cache_entries();

        Self {
            font_system,
            parley_shaper,
            parley_scale: parley::swash::scale::ScaleContext::new(),
            font_stack_key,
            font_db_revision,
            quality: TextQualityState::new(TextQualitySettings::default()),
            common_fallback_config,

            blobs: SlotMap::with_key(),
            blob_cache: HashMap::new(),
            blob_key_by_id: HashMap::new(),
            released_blob_lru: VecDeque::new(),
            released_blob_set: HashSet::new(),
            unwrapped_layout_cache: HashMap::new(),
            unwrapped_layout_lru: VecDeque::new(),
            unwrapped_layout_set: HashSet::new(),
            shape_cache: HashMap::new(),
            measure_cache: HashMap::new(),
            // Pre-reserve to avoid HashMap rehash spikes on editor pages that touch thousands of
            // unique text strings during a single resize/layout sequence.
            measure_shaping_cache: HashMap::with_capacity(measure_shaping_entries.min(65_536)),
            measure_shaping_fifo: VecDeque::with_capacity(measure_shaping_entries.min(65_536)),

            mask_atlas,
            color_atlas,
            subpixel_atlas,
            atlas_bind_group_layout,

            text_pin_mask: vec![Vec::new(); 3],
            text_pin_color: vec![Vec::new(); 3],
            text_pin_subpixel: vec![Vec::new(); 3],
            font_bytes_by_blob_id: HashMap::new(),
            font_face_key_by_fontique: HashMap::new(),

            perf_frame_cache_resets: 0,
            perf_frame_blob_cache_hits: 0,
            perf_frame_blob_cache_misses: 0,
            perf_frame_blobs_created: 0,
            perf_frame_shape_cache_hits: 0,
            perf_frame_shape_cache_misses: 0,
            perf_frame_shapes_created: 0,
            perf_frame_unwrapped_layout_cache_hits: 0,
            perf_frame_unwrapped_layout_cache_misses: 0,
            perf_frame_unwrapped_layouts_created: 0,

            glyph_atlas_epoch: 1,
        }
    }

    pub fn set_font_families(&mut self, config: &TextFontFamilyConfig) -> bool {
        let installed = build_installed_family_set(self.font_system.db());
        let old_key = self.font_stack_key;
        let mut parley_changed = false;
        let common_fallback =
            <FretFallback as cosmic_text::Fallback>::common_fallback(&FretFallback);
        let config_fallback_changed = self.common_fallback_config != config.common_fallback;
        self.common_fallback_config
            .clone_from(&config.common_fallback);

        let pick =
            |overrides: &[String], defaults: &'static [&'static str]| -> Option<Cow<'_, str>> {
                for candidate in overrides {
                    if installed.contains(&candidate.to_ascii_lowercase()) {
                        return Some(Cow::Owned(candidate.clone()));
                    }
                }
                for &candidate in defaults {
                    if installed.contains(&candidate.to_ascii_lowercase()) {
                        return Some(Cow::Borrowed(candidate));
                    }
                }
                None
            };

        {
            let db = self.font_system.db_mut();

            if let Some(sans) = pick(&config.ui_sans, default_sans_candidates()) {
                db.set_sans_serif_family(sans.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::SansSerif, sans.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::SystemUi, sans.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::UiSansSerif, sans.as_ref());
            }
            if let Some(serif) = pick(&config.ui_serif, default_serif_candidates()) {
                db.set_serif_family(serif.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::Serif, serif.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::UiSerif, serif.as_ref());
            }
            if let Some(mono) = pick(&config.ui_mono, default_monospace_candidates()) {
                db.set_monospace_family(mono.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::Monospace, mono.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::UiMonospace, mono.as_ref());
            }
        }

        let generics = [
            ParleyGenericFamily::SansSerif,
            ParleyGenericFamily::Serif,
            ParleyGenericFamily::Monospace,
            ParleyGenericFamily::SystemUi,
            ParleyGenericFamily::UiSansSerif,
            ParleyGenericFamily::UiSerif,
            ParleyGenericFamily::UiMonospace,
        ];
        for family in &self.common_fallback_config {
            for &generic in &generics {
                parley_changed |= self
                    .parley_shaper
                    .append_generic_family_name(generic, family);
            }
        }
        for &family in common_fallback {
            for &generic in &generics {
                parley_changed |= self
                    .parley_shaper
                    .append_generic_family_name(generic, family);
            }
        }

        let mut new_key = font_stack_cache_key(
            self.font_system.locale(),
            self.font_system.db(),
            self.font_db_revision,
            &self.common_fallback_config,
        );
        if new_key == old_key && (parley_changed || config_fallback_changed) {
            // Fontique generic family changes do not participate in the cosmic-text key, so we
            // bump the revision to ensure caches cannot reuse stale Parley shaping results.
            self.font_db_revision = self.font_db_revision.saturating_add(1);
            new_key = font_stack_cache_key(
                self.font_system.locale(),
                self.font_system.db(),
                self.font_db_revision,
                &self.common_fallback_config,
            );
        }
        if new_key == old_key {
            return false;
        }

        self.font_stack_key = new_key;
        self.perf_frame_cache_resets = self.perf_frame_cache_resets.saturating_add(1);
        self.blobs.clear();
        self.blob_cache.clear();
        self.blob_key_by_id.clear();
        self.clear_released_blob_cache();
        self.clear_unwrapped_layout_cache();
        self.shape_cache.clear();
        self.measure_cache.clear();
        self.measure_shaping_cache.clear();
        self.measure_shaping_fifo.clear();
        self.mask_atlas.reset();
        self.color_atlas.reset();
        self.subpixel_atlas.reset();
        self.text_pin_mask.iter_mut().for_each(|v| v.clear());
        self.text_pin_color.iter_mut().for_each(|v| v.clear());
        self.text_pin_subpixel.iter_mut().for_each(|v| v.clear());
        self.font_bytes_by_blob_id.clear();
        self.font_face_key_by_fontique.clear();
        true
    }

    pub fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.atlas_bind_group_layout
    }

    pub fn mask_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.mask_atlas.bind_group(page)
    }

    pub fn color_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.color_atlas.bind_group(page)
    }

    pub fn subpixel_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.subpixel_atlas.bind_group(page)
    }

    pub fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        self.mask_atlas.flush_uploads(queue);
        self.color_atlas.flush_uploads(queue);
        self.subpixel_atlas.flush_uploads(queue);
    }

    pub(crate) fn take_atlas_perf_snapshot(&mut self) -> TextAtlasPerfSnapshot {
        let mask = self.mask_atlas.take_perf_snapshot();
        let color = self.color_atlas.take_perf_snapshot();
        let subpixel = self.subpixel_atlas.take_perf_snapshot();

        TextAtlasPerfSnapshot {
            uploads: mask.uploads + color.uploads + subpixel.uploads,
            upload_bytes: mask.upload_bytes + color.upload_bytes + subpixel.upload_bytes,
            evicted_glyphs: mask.evicted_glyphs + color.evicted_glyphs + subpixel.evicted_glyphs,
            evicted_pages: mask.evicted_pages + color.evicted_pages + subpixel.evicted_pages,
            evicted_page_glyphs: mask.evicted_page_glyphs
                + color.evicted_page_glyphs
                + subpixel.evicted_page_glyphs,
            resets: mask.resets + color.resets + subpixel.resets,
        }
    }

    pub(crate) fn atlas_revision(&self) -> u64 {
        self.mask_atlas
            .revision()
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ self.color_atlas.revision().rotate_left(1)
            ^ self.subpixel_atlas.revision().rotate_left(2)
    }

    pub(crate) fn glyph_uv_for_instance(&self, glyph: &GlyphInstance) -> Option<(u16, [f32; 4])> {
        let (atlas, w, h) = match glyph.kind() {
            GlyphQuadKind::Mask => (
                &self.mask_atlas,
                self.mask_atlas.width as f32,
                self.mask_atlas.height as f32,
            ),
            GlyphQuadKind::Color => (
                &self.color_atlas,
                self.color_atlas.width as f32,
                self.color_atlas.height as f32,
            ),
            GlyphQuadKind::Subpixel => (
                &self.subpixel_atlas,
                self.subpixel_atlas.width as f32,
                self.subpixel_atlas.height as f32,
            ),
        };

        let entry = atlas.entry(glyph.key)?;
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let u0 = entry.x as f32 / w;
        let v0 = entry.y as f32 / h;
        let u1 = (entry.x.saturating_add(entry.w) as f32) / w;
        let v1 = (entry.y.saturating_add(entry.h) as f32) / h;
        Some((entry.page, [u0, v0, u1, v1]))
    }

    pub fn prepare_for_scene(&mut self, scene: &Scene, frame_index: u64) {
        let ring_len = self
            .text_pin_mask
            .len()
            .min(self.text_pin_color.len())
            .min(self.text_pin_subpixel.len());
        if ring_len == 0 {
            return;
        }
        let bucket = (frame_index as usize) % ring_len;

        let old_mask = std::mem::take(&mut self.text_pin_mask[bucket]);
        let old_color = std::mem::take(&mut self.text_pin_color[bucket]);
        let old_subpixel = std::mem::take(&mut self.text_pin_subpixel[bucket]);
        self.mask_atlas.dec_live_refs(&old_mask);
        self.color_atlas.dec_live_refs(&old_color);
        self.subpixel_atlas.dec_live_refs(&old_subpixel);

        let mut mask_keys: HashSet<GlyphKey> = HashSet::new();
        let mut color_keys: HashSet<GlyphKey> = HashSet::new();
        let mut subpixel_keys: HashSet<GlyphKey> = HashSet::new();

        for op in scene.ops() {
            let SceneOp::Text { text, .. } = *op else {
                continue;
            };
            let Some(blob) = self.blobs.get(text) else {
                continue;
            };
            for glyph in blob.shape.glyphs.as_ref() {
                match glyph.kind() {
                    GlyphQuadKind::Mask => {
                        mask_keys.insert(glyph.key);
                    }
                    GlyphQuadKind::Color => {
                        color_keys.insert(glyph.key);
                    }
                    GlyphQuadKind::Subpixel => {
                        subpixel_keys.insert(glyph.key);
                    }
                }
            }
        }

        let epoch = frame_index;
        let mut new_mask: Vec<GlyphKey> = mask_keys.into_iter().collect();
        let mut new_color: Vec<GlyphKey> = color_keys.into_iter().collect();
        let mut new_subpixel: Vec<GlyphKey> = subpixel_keys.into_iter().collect();

        for &key in &new_mask {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_color {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_subpixel {
            self.ensure_glyph_in_atlas(key, epoch);
        }

        self.mask_atlas.inc_live_refs(&new_mask);
        self.color_atlas.inc_live_refs(&new_color);
        self.subpixel_atlas.inc_live_refs(&new_subpixel);

        self.text_pin_mask[bucket].append(&mut new_mask);
        self.text_pin_color[bucket].append(&mut new_color);
        self.text_pin_subpixel[bucket].append(&mut new_subpixel);
    }

    fn ensure_glyph_in_atlas(&mut self, key: GlyphKey, epoch: u64) {
        let already_present = match key.kind {
            GlyphQuadKind::Mask => self.mask_atlas.get(key, epoch).is_some(),
            GlyphQuadKind::Color => self.color_atlas.get(key, epoch).is_some(),
            GlyphQuadKind::Subpixel => self.subpixel_atlas.get(key, epoch).is_some(),
        };
        if already_present {
            return;
        }

        self.ensure_parley_glyph(key, epoch);
    }

    fn ensure_parley_glyph(&mut self, key: GlyphKey, epoch: u64) {
        let Some(font_bytes) = self.font_bytes_by_blob_id.get(&key.font.blob_id) else {
            return;
        };

        let Some(font_ref) =
            parley::swash::FontRef::from_index(font_bytes.as_ref(), key.font.face_index as usize)
        else {
            return;
        };
        let Ok(glyph_id) = u16::try_from(key.glyph_id) else {
            return;
        };

        let font_size = f32::from_bits(key.size_bits).max(1.0);
        let mut scaler = self
            .parley_scale
            .builder(font_ref)
            .size(font_size)
            .hint(false)
            .build();

        let offset_px = parley::swash::zeno::Vector::new(
            subpixel_bin_as_float(key.x_bin),
            subpixel_bin_as_float(key.y_bin),
        );
        let Some(image) = parley::swash::scale::Render::new(&[
            parley::swash::scale::Source::ColorOutline(0),
            parley::swash::scale::Source::ColorBitmap(parley::swash::scale::StrikeWith::BestFit),
            parley::swash::scale::Source::Outline,
        ])
        .offset(offset_px)
        .render(&mut scaler, glyph_id) else {
            return;
        };
        if image.placement.width == 0 || image.placement.height == 0 {
            return;
        }

        let (image_kind, bytes_per_pixel) = match image.content {
            parley::swash::scale::image::Content::Mask => (GlyphQuadKind::Mask, 1),
            parley::swash::scale::image::Content::Color => (GlyphQuadKind::Color, 4),
            parley::swash::scale::image::Content::SubpixelMask => (GlyphQuadKind::Subpixel, 4),
        };
        if image_kind != key.kind {
            return;
        }

        let data = image.data;

        match key.kind {
            GlyphQuadKind::Mask => {
                let _ = self.mask_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
            GlyphQuadKind::Color => {
                let _ = self.color_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
            GlyphQuadKind::Subpixel => {
                let _ = self.subpixel_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
        }
    }

    pub fn blob(&self, id: TextBlobId) -> Option<&TextBlob> {
        self.blobs.get(id)
    }

    #[allow(dead_code)]
    pub fn prepare_input(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        match input {
            TextInputRef::Plain { text, style } => self.prepare(text, style, constraints),
            TextInputRef::Attributed { text, base, spans } => {
                let spans = sanitize_spans_for_text(text, spans);
                if spans.is_none() {
                    return self.prepare(text, base, constraints);
                }
                let rich = AttributedText {
                    text: Arc::<str>::from(text),
                    spans: spans.expect("non-empty spans"),
                };
                self.prepare_attributed(&rich, base, constraints)
            }
        }
    }

    pub fn prepare(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let key = TextBlobKey::new(text, style, constraints, self.font_stack_key);
        self.prepare_with_key(key, style, None, constraints)
    }

    pub fn prepare_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let spans = sanitize_spans_for_text(rich.text.as_ref(), rich.spans.as_ref());
        if spans.is_none() {
            return self.prepare(rich.text.as_ref(), base_style, constraints);
        }
        let rich = AttributedText {
            text: rich.text.clone(),
            spans: spans.expect("non-empty spans"),
        };
        let key = TextBlobKey::new_attributed(&rich, base_style, constraints, self.font_stack_key);
        self.prepare_with_key(key, base_style, Some(rich.spans.as_ref()), constraints)
    }

    fn prepare_with_key(
        &mut self,
        mut key: TextBlobKey,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let text = key.text.clone();
        key.backend = 1;

        let scale = constraints.scale_factor.max(1.0);
        let snap_vertical = scale.is_finite() && scale.fract().abs() > 1e-4 && scale >= 1.0;

        if let Some(id) = self.blob_cache.get(&key).copied() {
            let hit = match self.blobs.get_mut(id) {
                Some(blob) => {
                    self.perf_frame_blob_cache_hits =
                        self.perf_frame_blob_cache_hits.saturating_add(1);
                    let was_released = blob.ref_count == 0;
                    blob.ref_count = blob.ref_count.saturating_add(1);
                    Some((blob.shape.metrics, was_released))
                }
                None => None,
            };

            if let Some((metrics, was_released)) = hit {
                if was_released {
                    self.remove_released_blob(id);
                }
                return (id, metrics);
            }

            // Stale cache entry (shouldn't happen, but keep it robust).
            self.blob_cache.remove(&key);
            self.blob_key_by_id.remove(&id);
        }
        self.perf_frame_blob_cache_misses = self.perf_frame_blob_cache_misses.saturating_add(1);

        let resolved_spans = spans.and_then(|spans| resolve_spans_for_text(text.as_ref(), spans));
        let paint_palette = resolved_spans.as_ref().map(|spans| {
            let mut palette: Vec<Option<fret_core::Color>> = Vec::with_capacity(spans.len());
            palette.extend(spans.iter().map(|s| s.fg));
            Arc::<[Option<fret_core::Color>]>::from(palette)
        });

        let shape_key = TextShapeKey::from_blob_key(&key);
        let shape = if let Some(shape) = self.shape_cache.get(&shape_key) {
            self.perf_frame_shape_cache_hits = self.perf_frame_shape_cache_hits.saturating_add(1);
            shape.clone()
        } else {
            self.perf_frame_shape_cache_misses =
                self.perf_frame_shape_cache_misses.saturating_add(1);
            let shape = {
                let input = match spans {
                    Some(spans) => TextInputRef::Attributed {
                        text: text.as_ref(),
                        base: style,
                        spans,
                    },
                    None => TextInputRef::Plain {
                        text: text.as_ref(),
                        style,
                    },
                };
                let wrapped = self.wrap_for_prepare(input, &key, constraints);
                let epoch = {
                    let e = self.glyph_atlas_epoch;
                    self.glyph_atlas_epoch = self.glyph_atlas_epoch.saturating_add(1);
                    e
                };

                let mut glyphs: Vec<GlyphInstance> = Vec::new();
                let mut lines: Vec<TextLine> = Vec::new();
                let mut first_line_caret_stops: Vec<(usize, Px)> = Vec::new();
                let mut line_top_px = 0.0_f32;

                let metrics = match wrapped {
                    WrappedForPrepare::Owned(wrapped) => {
                        let kept_end = wrapped.kept_end;

                        let first_baseline_px = wrapped
                            .lines
                            .first()
                            .map(|l| l.baseline.max(0.0))
                            .unwrap_or(0.0);
                        let first_baseline_px =
                            if snap_vertical && let Some(first) = wrapped.lines.first() {
                                let top_px = 0.0_f32;
                                let bottom_px =
                                    (top_px + first.line_height.max(0.0)).round().max(top_px);
                                let height_px = (bottom_px - top_px).max(0.0);
                                (top_px + first.baseline.max(0.0))
                                    .round()
                                    .clamp(top_px, top_px + height_px)
                            } else {
                                first_baseline_px
                            };

                        let metrics = metrics_from_wrapped_lines(&wrapped.lines, scale);
                        lines.reserve(wrapped.lines.len().max(1));

                        for (i, (range, line)) in wrapped
                            .line_ranges
                            .iter()
                            .cloned()
                            .zip(wrapped.lines.into_iter())
                            .enumerate()
                        {
                            if snap_vertical {
                                line_top_px = line_top_px.round();
                            }

                            let line_height_px_raw = line.line_height.max(0.0);
                            let line_baseline_px_raw = line.baseline.max(0.0);

                            let (line_height_px, baseline_pos_px) = if snap_vertical {
                                let bottom_px =
                                    (line_top_px + line_height_px_raw).round().max(line_top_px);
                                let height_px = (bottom_px - line_top_px).max(0.0);
                                let baseline_pos_px = (line_top_px + line_baseline_px_raw)
                                    .round()
                                    .clamp(line_top_px, line_top_px + height_px);
                                (height_px, baseline_pos_px)
                            } else {
                                (line_height_px_raw, line_top_px + line_baseline_px_raw)
                            };

                            let line_offset_px = baseline_pos_px - first_baseline_px;

                            let slice = &text[range.clone()];
                            let caret_stops = caret_stops_for_slice(
                                slice,
                                range.start,
                                &line.clusters,
                                line.width.max(0.0),
                                scale,
                                kept_end,
                            );
                            if i == 0 {
                                first_line_caret_stops = caret_stops.clone();
                            }

                            lines.push(TextLine {
                                start: range.start,
                                end: range.end.min(kept_end),
                                width: Px((line.width / scale).max(0.0)),
                                y_top: Px((line_top_px / scale).max(0.0)),
                                y_baseline: Px((baseline_pos_px / scale).max(0.0)),
                                height: Px((line_height_px / scale).max(0.0)),
                                caret_stops,
                            });

                            for g in line.glyphs {
                                let Ok(glyph_id) = u16::try_from(g.id) else {
                                    continue;
                                };
                                let fontique_id = g.font.data.id();
                                let face_index = g.font.index;
                                let face_key = if let Some(hit) = self
                                    .font_face_key_by_fontique
                                    .get(&(fontique_id, face_index))
                                    .copied()
                                {
                                    hit
                                } else {
                                    let bytes = g.font.data.data();
                                    let blob_id = stable_font_blob_id(bytes);
                                    self.font_bytes_by_blob_id
                                        .entry(blob_id)
                                        .or_insert_with(|| Arc::from(bytes.to_vec()));
                                    let key = FontFaceKey {
                                        blob_id,
                                        face_index,
                                        variation_key: 0,
                                    };
                                    self.font_face_key_by_fontique
                                        .insert((fontique_id, face_index), key);
                                    key
                                };

                                let pos_y = g.y + line_offset_px;
                                let (x, x_bin) = subpixel_bin_q4(g.x);
                                let (y, y_bin) = subpixel_bin_y(pos_y);

                                let text_range = (range.start + g.text_range.start)
                                    ..(range.start + g.text_range.end);
                                let paint_span = resolved_spans.as_deref().and_then(|spans| {
                                    paint_span_for_text_range(spans, &text_range, g.is_rtl)
                                });

                                let size_bits = g.font_size.to_bits();
                                let mut atlas_hit: Option<(GlyphKey, GlyphAtlasEntry)> = None;
                                let color_key = GlyphKey {
                                    font: face_key,
                                    glyph_id: g.id,
                                    size_bits,
                                    x_bin,
                                    y_bin,
                                    kind: GlyphQuadKind::Color,
                                };
                                if let Some(entry) = self.color_atlas.get(color_key, epoch) {
                                    atlas_hit = Some((color_key, entry));
                                } else {
                                    let subpixel_key = GlyphKey {
                                        font: face_key,
                                        glyph_id: g.id,
                                        size_bits,
                                        x_bin,
                                        y_bin,
                                        kind: GlyphQuadKind::Subpixel,
                                    };
                                    if let Some(entry) =
                                        self.subpixel_atlas.get(subpixel_key, epoch)
                                    {
                                        atlas_hit = Some((subpixel_key, entry));
                                    } else {
                                        let mask_key = GlyphKey {
                                            font: face_key,
                                            glyph_id: g.id,
                                            size_bits,
                                            x_bin,
                                            y_bin,
                                            kind: GlyphQuadKind::Mask,
                                        };
                                        if let Some(entry) = self.mask_atlas.get(mask_key, epoch) {
                                            atlas_hit = Some((mask_key, entry));
                                        }
                                    }
                                }

                                let (glyph_key, x0_px, y0_px, w_px, h_px) =
                                    if let Some((glyph_key, entry)) = atlas_hit {
                                        (
                                            glyph_key,
                                            x as f32 + entry.placement_left as f32,
                                            y as f32 - entry.placement_top as f32,
                                            entry.w as f32,
                                            entry.h as f32,
                                        )
                                    } else {
                                        let Some(font_ref) = parley::swash::FontRef::from_index(
                                            g.font.data.data(),
                                            g.font.index as usize,
                                        ) else {
                                            continue;
                                        };

                                        let mut scaler = self
                                            .parley_scale
                                            .builder(font_ref)
                                            .size(g.font_size.max(1.0))
                                            .hint(false)
                                            .build();

                                        let offset_px = parley::swash::zeno::Vector::new(
                                            subpixel_bin_as_float(x_bin),
                                            subpixel_bin_as_float(y_bin),
                                        );

                                        let Some(image) = parley::swash::scale::Render::new(&[
                                            parley::swash::scale::Source::ColorOutline(0),
                                            parley::swash::scale::Source::ColorBitmap(
                                                parley::swash::scale::StrikeWith::BestFit,
                                            ),
                                            parley::swash::scale::Source::Outline,
                                        ])
                                        .offset(offset_px)
                                        .render(&mut scaler, glyph_id) else {
                                            continue;
                                        };

                                        if image.placement.width == 0 || image.placement.height == 0
                                        {
                                            continue;
                                        }

                                        let placement = image.placement;
                                        let (kind, bytes_per_pixel) = match image.content {
                                            parley::swash::scale::image::Content::Mask => {
                                                (GlyphQuadKind::Mask, 1)
                                            }
                                            parley::swash::scale::image::Content::Color => {
                                                (GlyphQuadKind::Color, 4)
                                            }
                                            parley::swash::scale::image::Content::SubpixelMask => {
                                                (GlyphQuadKind::Subpixel, 4)
                                            }
                                        };

                                        let glyph_key = GlyphKey {
                                            font: face_key,
                                            glyph_id: g.id,
                                            size_bits,
                                            x_bin,
                                            y_bin,
                                            kind,
                                        };

                                        let data = image.data;
                                        match kind {
                                            GlyphQuadKind::Mask => {
                                                let _ = self.mask_atlas.get_or_insert(
                                                    glyph_key,
                                                    placement.width,
                                                    placement.height,
                                                    placement.left,
                                                    placement.top,
                                                    bytes_per_pixel,
                                                    data,
                                                    epoch,
                                                );
                                            }
                                            GlyphQuadKind::Color => {
                                                let _ = self.color_atlas.get_or_insert(
                                                    glyph_key,
                                                    placement.width,
                                                    placement.height,
                                                    placement.left,
                                                    placement.top,
                                                    bytes_per_pixel,
                                                    data,
                                                    epoch,
                                                );
                                            }
                                            GlyphQuadKind::Subpixel => {
                                                let _ = self.subpixel_atlas.get_or_insert(
                                                    glyph_key,
                                                    placement.width,
                                                    placement.height,
                                                    placement.left,
                                                    placement.top,
                                                    bytes_per_pixel,
                                                    data,
                                                    epoch,
                                                );
                                            }
                                        }

                                        (
                                            glyph_key,
                                            x as f32 + placement.left as f32,
                                            y as f32 - placement.top as f32,
                                            placement.width as f32,
                                            placement.height as f32,
                                        )
                                    };

                                glyphs.push(GlyphInstance {
                                    rect: [
                                        x0_px / scale,
                                        y0_px / scale,
                                        w_px / scale,
                                        h_px / scale,
                                    ],
                                    paint_span,
                                    key: glyph_key,
                                });
                            }

                            line_top_px += line_height_px;
                        }

                        metrics
                    }
                    WrappedForPrepare::UnwrappedWordLtr {
                        kept_end,
                        unwrapped,
                        lines: slices,
                        ..
                    } => {
                        let first_baseline_px = unwrapped.baseline.max(0.0);
                        let first_baseline_px = if snap_vertical {
                            let top_px = 0.0_f32;
                            let bottom_px = (top_px + unwrapped.line_height.max(0.0))
                                .round()
                                .max(top_px);
                            let height_px = (bottom_px - top_px).max(0.0);
                            (top_px + unwrapped.baseline.max(0.0))
                                .round()
                                .clamp(top_px, top_px + height_px)
                        } else {
                            first_baseline_px
                        };

                        let mut max_w_px = 0.0_f32;
                        for s in &slices {
                            max_w_px = max_w_px.max(s.width_px.max(0.0));
                        }
                        let metrics = metrics_for_uniform_lines(
                            max_w_px,
                            slices.len().max(1),
                            unwrapped.baseline.max(0.0),
                            unwrapped.line_height.max(0.0),
                            scale,
                        );

                        lines.reserve(slices.len().max(1));

                        for (i, s) in slices.into_iter().enumerate() {
                            if snap_vertical {
                                line_top_px = line_top_px.round();
                            }

                            let line_height_px_raw = unwrapped.line_height.max(0.0);
                            let line_baseline_px_raw = unwrapped.baseline.max(0.0);

                            let (line_height_px, baseline_pos_px) = if snap_vertical {
                                let bottom_px =
                                    (line_top_px + line_height_px_raw).round().max(line_top_px);
                                let height_px = (bottom_px - line_top_px).max(0.0);
                                let baseline_pos_px = (line_top_px + line_baseline_px_raw)
                                    .round()
                                    .clamp(line_top_px, line_top_px + height_px);
                                (height_px, baseline_pos_px)
                            } else {
                                (line_height_px_raw, line_top_px + line_baseline_px_raw)
                            };

                            let line_offset_px = baseline_pos_px - first_baseline_px;

                            let slice = &text[s.range.clone()];
                            let caret_stops = caret_stops_for_slice_from_unwrapped_ltr(
                                slice,
                                s.range.start,
                                &unwrapped.clusters,
                                s.cluster_range.clone(),
                                s.line_start_x,
                                s.width_px.max(0.0),
                                scale,
                                kept_end,
                            );
                            if i == 0 {
                                first_line_caret_stops = caret_stops.clone();
                            }

                            lines.push(TextLine {
                                start: s.range.start,
                                end: s.range.end.min(kept_end),
                                width: Px((s.width_px / scale).max(0.0)),
                                y_top: Px((line_top_px / scale).max(0.0)),
                                y_baseline: Px((baseline_pos_px / scale).max(0.0)),
                                height: Px((line_height_px / scale).max(0.0)),
                                caret_stops,
                            });

                            for g in unwrapped.glyphs[s.glyph_range.clone()].iter() {
                                let Ok(glyph_id) = u16::try_from(g.id) else {
                                    continue;
                                };
                                let fontique_id = g.font.data.id();
                                let face_index = g.font.index;
                                let face_key = if let Some(hit) = self
                                    .font_face_key_by_fontique
                                    .get(&(fontique_id, face_index))
                                    .copied()
                                {
                                    hit
                                } else {
                                    let bytes = g.font.data.data();
                                    let blob_id = stable_font_blob_id(bytes);
                                    self.font_bytes_by_blob_id
                                        .entry(blob_id)
                                        .or_insert_with(|| Arc::from(bytes.to_vec()));
                                    let key = FontFaceKey {
                                        blob_id,
                                        face_index,
                                        variation_key: 0,
                                    };
                                    self.font_face_key_by_fontique
                                        .insert((fontique_id, face_index), key);
                                    key
                                };

                                let pos_y = g.y + line_offset_px;
                                let x = g.x - s.line_start_x;
                                let (x, x_bin) = subpixel_bin_q4(x);
                                let (y, y_bin) = subpixel_bin_y(pos_y);

                                let text_range = g.text_range.clone();
                                let paint_span = resolved_spans.as_deref().and_then(|spans| {
                                    paint_span_for_text_range(spans, &text_range, g.is_rtl)
                                });

                                let size_bits = g.font_size.to_bits();
                                let mut atlas_hit: Option<(GlyphKey, GlyphAtlasEntry)> = None;
                                let color_key = GlyphKey {
                                    font: face_key,
                                    glyph_id: g.id,
                                    size_bits,
                                    x_bin,
                                    y_bin,
                                    kind: GlyphQuadKind::Color,
                                };
                                if let Some(entry) = self.color_atlas.get(color_key, epoch) {
                                    atlas_hit = Some((color_key, entry));
                                } else {
                                    let subpixel_key = GlyphKey {
                                        font: face_key,
                                        glyph_id: g.id,
                                        size_bits,
                                        x_bin,
                                        y_bin,
                                        kind: GlyphQuadKind::Subpixel,
                                    };
                                    if let Some(entry) =
                                        self.subpixel_atlas.get(subpixel_key, epoch)
                                    {
                                        atlas_hit = Some((subpixel_key, entry));
                                    } else {
                                        let mask_key = GlyphKey {
                                            font: face_key,
                                            glyph_id: g.id,
                                            size_bits,
                                            x_bin,
                                            y_bin,
                                            kind: GlyphQuadKind::Mask,
                                        };
                                        if let Some(entry) = self.mask_atlas.get(mask_key, epoch) {
                                            atlas_hit = Some((mask_key, entry));
                                        }
                                    }
                                }

                                let (glyph_key, x0_px, y0_px, w_px, h_px) =
                                    if let Some((glyph_key, entry)) = atlas_hit {
                                        (
                                            glyph_key,
                                            x as f32 + entry.placement_left as f32,
                                            y as f32 - entry.placement_top as f32,
                                            entry.w as f32,
                                            entry.h as f32,
                                        )
                                    } else {
                                        let Some(font_ref) = parley::swash::FontRef::from_index(
                                            g.font.data.data(),
                                            g.font.index as usize,
                                        ) else {
                                            continue;
                                        };

                                        let mut scaler = self
                                            .parley_scale
                                            .builder(font_ref)
                                            .size(g.font_size.max(1.0))
                                            .hint(false)
                                            .build();

                                        let offset_px = parley::swash::zeno::Vector::new(
                                            subpixel_bin_as_float(x_bin),
                                            subpixel_bin_as_float(y_bin),
                                        );

                                        let Some(image) = parley::swash::scale::Render::new(&[
                                            parley::swash::scale::Source::ColorOutline(0),
                                            parley::swash::scale::Source::ColorBitmap(
                                                parley::swash::scale::StrikeWith::BestFit,
                                            ),
                                            parley::swash::scale::Source::Outline,
                                        ])
                                        .offset(offset_px)
                                        .render(&mut scaler, glyph_id) else {
                                            continue;
                                        };

                                        if image.placement.width == 0 || image.placement.height == 0
                                        {
                                            continue;
                                        }

                                        let placement = image.placement;
                                        let (kind, bytes_per_pixel) = match image.content {
                                            parley::swash::scale::image::Content::Mask => {
                                                (GlyphQuadKind::Mask, 1)
                                            }
                                            parley::swash::scale::image::Content::Color => {
                                                (GlyphQuadKind::Color, 4)
                                            }
                                            parley::swash::scale::image::Content::SubpixelMask => {
                                                (GlyphQuadKind::Subpixel, 4)
                                            }
                                        };

                                        let glyph_key = GlyphKey {
                                            font: face_key,
                                            glyph_id: g.id,
                                            size_bits,
                                            x_bin,
                                            y_bin,
                                            kind,
                                        };

                                        let data = image.data;
                                        match kind {
                                            GlyphQuadKind::Mask => {
                                                let _ = self.mask_atlas.get_or_insert(
                                                    glyph_key,
                                                    placement.width,
                                                    placement.height,
                                                    placement.left,
                                                    placement.top,
                                                    bytes_per_pixel,
                                                    data,
                                                    epoch,
                                                );
                                            }
                                            GlyphQuadKind::Color => {
                                                let _ = self.color_atlas.get_or_insert(
                                                    glyph_key,
                                                    placement.width,
                                                    placement.height,
                                                    placement.left,
                                                    placement.top,
                                                    bytes_per_pixel,
                                                    data,
                                                    epoch,
                                                );
                                            }
                                            GlyphQuadKind::Subpixel => {
                                                let _ = self.subpixel_atlas.get_or_insert(
                                                    glyph_key,
                                                    placement.width,
                                                    placement.height,
                                                    placement.left,
                                                    placement.top,
                                                    bytes_per_pixel,
                                                    data,
                                                    epoch,
                                                );
                                            }
                                        }

                                        (
                                            glyph_key,
                                            x as f32 + placement.left as f32,
                                            y as f32 - placement.top as f32,
                                            placement.width as f32,
                                            placement.height as f32,
                                        )
                                    };

                                glyphs.push(GlyphInstance {
                                    rect: [
                                        x0_px / scale,
                                        y0_px / scale,
                                        w_px / scale,
                                        h_px / scale,
                                    ],
                                    paint_span,
                                    key: glyph_key,
                                });
                            }

                            line_top_px += line_height_px;
                        }

                        metrics
                    }
                };

                Arc::new(TextShape {
                    glyphs: Arc::from(glyphs),
                    metrics,
                    lines: Arc::from(lines),
                    caret_stops: Arc::from(first_line_caret_stops),
                })
            };
            self.perf_frame_shapes_created = self.perf_frame_shapes_created.saturating_add(1);
            self.shape_cache.insert(shape_key.clone(), shape.clone());
            shape
        };

        let decorations: Vec<TextDecoration> = resolved_spans
            .as_deref()
            .map(|spans| decorations_for_lines(shape.lines.as_ref(), spans, scale, snap_vertical))
            .unwrap_or_default();

        let metrics = shape.metrics;
        let id = self.blobs.insert(TextBlob {
            shape,
            paint_palette,
            decorations: Arc::from(decorations),
            ref_count: 1,
        });
        self.perf_frame_blobs_created = self.perf_frame_blobs_created.saturating_add(1);
        self.blob_cache.insert(key.clone(), id);
        self.blob_key_by_id.insert(id, key);
        (id, metrics)
    }

    pub fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        const MEASURE_CACHE_PER_BUCKET_LIMIT: usize = 256;

        let mut normalized_constraints = constraints;
        if normalized_constraints.wrap == TextWrap::None {
            normalized_constraints.max_width = None;
        }

        let key = TextMeasureKey::new(style, normalized_constraints, self.font_stack_key);
        let text_hash = hash_text(text);
        if let Some(bucket) = self.measure_cache.get_mut(&key)
            && let Some(hit) = bucket
                .iter()
                .find(|e| e.text_hash == text_hash && e.spans_hash == 0 && e.text.as_ref() == text)
        {
            let mut metrics = hit.metrics;
            if constraints.wrap == TextWrap::None
                && constraints.overflow == TextOverflow::Ellipsis
                && let Some(max_width) = constraints.max_width
            {
                metrics.size.width = max_width;
            }
            return metrics;
        }

        let scale = constraints.scale_factor.max(1.0);
        let max_width_for_fast = match constraints {
            TextConstraints {
                max_width: Some(max_width),
                wrap: TextWrap::Word | TextWrap::Grapheme,
                overflow: TextOverflow::Clip,
                ..
            } if !text.contains('\n') => Some(max_width),
            _ => None,
        };

        let metrics = if let Some(max_width) = max_width_for_fast {
            let allow_shaping_cache = text.len() >= measure_shaping_cache_min_text_len_bytes();

            let shaping_key = TextMeasureShapingKey {
                text_hash,
                text_len: text.len(),
                spans_shaping_key: 0,
                font: style.font.clone(),
                font_stack_key: self.font_stack_key,
                size_bits: style.size.0.to_bits(),
                weight: style.weight.0,
                slant: match style.slant {
                    TextSlant::Normal => 0,
                    TextSlant::Italic => 1,
                    TextSlant::Oblique => 2,
                },
                line_height_bits: style.line_height.map(|px| px.0.to_bits()),
                letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
                scale_bits: constraints.scale_factor.to_bits(),
            };

            let max_width_px = max_width.0 * scale;

            if allow_shaping_cache {
                let (width_px, baseline_px, line_height_px, clusters) = if let Some(hit) =
                    self.measure_shaping_cache.get(&shaping_key)
                    && hit.text.as_ref() == text
                    && hit.spans.is_none()
                {
                    (
                        hit.width_px,
                        hit.baseline_px,
                        hit.line_height_px,
                        hit.clusters.clone(),
                    )
                } else {
                    let line = self
                        .parley_shaper
                        .shape_single_line_metrics(TextInputRef::plain(text, style), scale);
                    let clusters: Arc<[parley_shaper::ShapedCluster]> = Arc::from(line.clusters);

                    let existed = self
                        .measure_shaping_cache
                        .insert(
                            shaping_key.clone(),
                            TextMeasureShapingEntry {
                                text: Arc::<str>::from(text),
                                spans: None,
                                width_px: line.width,
                                baseline_px: line.baseline,
                                line_height_px: line.line_height,
                                clusters: clusters.clone(),
                            },
                        )
                        .is_some();
                    if !existed {
                        self.measure_shaping_fifo.push_back(shaping_key.clone());
                        let limit = measure_shaping_cache_entries();
                        while self.measure_shaping_fifo.len() > limit {
                            let Some(evict) = self.measure_shaping_fifo.pop_front() else {
                                break;
                            };
                            self.measure_shaping_cache.remove(&evict);
                        }
                    }

                    (line.width, line.baseline, line.line_height, clusters)
                };

                let (line_count, max_w_px) = if width_px <= max_width_px + 0.5 {
                    (1, width_px.max(0.0))
                } else {
                    match constraints.wrap {
                        TextWrap::Word => {
                            word_wrap_line_stats(text, clusters.as_ref(), max_width_px)
                        }
                        TextWrap::Grapheme => {
                            grapheme_wrap_line_stats(text, clusters.as_ref(), max_width_px)
                        }
                        TextWrap::None => unreachable!(),
                    }
                };
                metrics_for_uniform_lines(max_w_px, line_count, baseline_px, line_height_px, scale)
            } else {
                let line = self
                    .parley_shaper
                    .shape_single_line_metrics(TextInputRef::plain(text, style), scale);
                let width_px = line.width;
                let baseline_px = line.baseline;
                let line_height_px = line.line_height;
                let clusters = line.clusters;

                let (line_count, max_w_px) = if width_px <= max_width_px + 0.5 {
                    (1, width_px.max(0.0))
                } else {
                    match constraints.wrap {
                        TextWrap::Word => {
                            word_wrap_line_stats(text, clusters.as_slice(), max_width_px)
                        }
                        TextWrap::Grapheme => {
                            grapheme_wrap_line_stats(text, clusters.as_slice(), max_width_px)
                        }
                        TextWrap::None => unreachable!(),
                    }
                };
                metrics_for_uniform_lines(max_w_px, line_count, baseline_px, line_height_px, scale)
            }
        } else {
            let wrapped = crate::text::wrapper::wrap_with_constraints_measure_only(
                &mut self.parley_shaper,
                TextInputRef::plain(text, style),
                normalized_constraints,
            );
            metrics_from_wrapped_lines(&wrapped.lines, scale)
        };

        let bucket = self.measure_cache.entry(key).or_default();
        bucket.push_back(TextMeasureEntry {
            text_hash,
            spans_hash: 0,
            text: Arc::<str>::from(text),
            spans: None,
            metrics,
        });
        while bucket.len() > MEASURE_CACHE_PER_BUCKET_LIMIT {
            bucket.pop_front();
        }

        let mut metrics = metrics;
        if constraints.wrap == TextWrap::None
            && constraints.overflow == TextOverflow::Ellipsis
            && let Some(max_width) = constraints.max_width
        {
            metrics.size.width = max_width;
        }
        metrics
    }

    pub fn measure_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        const MEASURE_CACHE_PER_BUCKET_LIMIT: usize = 256;

        let mut normalized_constraints = constraints;
        if normalized_constraints.wrap == TextWrap::None {
            normalized_constraints.max_width = None;
        }

        let key = TextMeasureKey::new(base_style, normalized_constraints, self.font_stack_key);
        let text_hash = hash_text(rich.text.as_ref());
        let spans_hash = spans_shaping_fingerprint(rich.spans.as_ref());

        if let Some(bucket) = self.measure_cache.get_mut(&key)
            && let Some(hit) = bucket.iter().find(|e| {
                e.text_hash == text_hash
                    && e.spans_hash == spans_hash
                    && e.text.as_ref() == rich.text.as_ref()
                    && e.spans.as_ref().is_some_and(|s| {
                        Arc::ptr_eq(s, &rich.spans) || s.as_ref() == rich.spans.as_ref()
                    })
            })
        {
            let mut metrics = hit.metrics;
            if constraints.wrap == TextWrap::None
                && constraints.overflow == TextOverflow::Ellipsis
                && let Some(max_width) = constraints.max_width
            {
                metrics.size.width = max_width;
            }
            return metrics;
        }

        let scale = constraints.scale_factor.max(1.0);
        let max_width_for_fast = match constraints {
            TextConstraints {
                max_width: Some(max_width),
                wrap: TextWrap::Word | TextWrap::Grapheme,
                overflow: TextOverflow::Clip,
                ..
            } if !rich.text.as_ref().contains('\n') => Some(max_width),
            _ => None,
        };

        let metrics = if let Some(max_width) = max_width_for_fast {
            let allow_shaping_cache = rich.text.len() >= measure_shaping_cache_min_text_len_bytes();

            let shaping_key = TextMeasureShapingKey {
                text_hash,
                text_len: rich.text.len(),
                spans_shaping_key: spans_hash,
                font: base_style.font.clone(),
                font_stack_key: self.font_stack_key,
                size_bits: base_style.size.0.to_bits(),
                weight: base_style.weight.0,
                slant: match base_style.slant {
                    TextSlant::Normal => 0,
                    TextSlant::Italic => 1,
                    TextSlant::Oblique => 2,
                },
                line_height_bits: base_style.line_height.map(|px| px.0.to_bits()),
                letter_spacing_bits: base_style.letter_spacing_em.map(|v| v.to_bits()),
                scale_bits: constraints.scale_factor.to_bits(),
            };

            let max_width_px = max_width.0 * scale;
            let text = rich.text.as_ref();

            if allow_shaping_cache {
                let (width_px, baseline_px, line_height_px, clusters) = if let Some(hit) =
                    self.measure_shaping_cache.get(&shaping_key)
                    && hit.text.as_ref() == rich.text.as_ref()
                    && hit.spans.as_ref().is_some_and(|s| {
                        Arc::ptr_eq(s, &rich.spans) || s.as_ref() == rich.spans.as_ref()
                    }) {
                    (
                        hit.width_px,
                        hit.baseline_px,
                        hit.line_height_px,
                        hit.clusters.clone(),
                    )
                } else {
                    let line = self.parley_shaper.shape_single_line_metrics(
                        TextInputRef::Attributed {
                            text: rich.text.as_ref(),
                            base: base_style,
                            spans: rich.spans.as_ref(),
                        },
                        scale,
                    );
                    let clusters: Arc<[parley_shaper::ShapedCluster]> = Arc::from(line.clusters);

                    let existed = self
                        .measure_shaping_cache
                        .insert(
                            shaping_key.clone(),
                            TextMeasureShapingEntry {
                                text: rich.text.clone(),
                                spans: Some(rich.spans.clone()),
                                width_px: line.width,
                                baseline_px: line.baseline,
                                line_height_px: line.line_height,
                                clusters: clusters.clone(),
                            },
                        )
                        .is_some();
                    if !existed {
                        self.measure_shaping_fifo.push_back(shaping_key.clone());
                        let limit = measure_shaping_cache_entries();
                        while self.measure_shaping_fifo.len() > limit {
                            let Some(evict) = self.measure_shaping_fifo.pop_front() else {
                                break;
                            };
                            self.measure_shaping_cache.remove(&evict);
                        }
                    }

                    (line.width, line.baseline, line.line_height, clusters)
                };

                let (line_count, max_w_px) = if width_px <= max_width_px + 0.5 {
                    (1, width_px.max(0.0))
                } else {
                    match constraints.wrap {
                        TextWrap::Word => {
                            word_wrap_line_stats(text, clusters.as_ref(), max_width_px)
                        }
                        TextWrap::Grapheme => {
                            grapheme_wrap_line_stats(text, clusters.as_ref(), max_width_px)
                        }
                        TextWrap::None => unreachable!(),
                    }
                };
                metrics_for_uniform_lines(max_w_px, line_count, baseline_px, line_height_px, scale)
            } else {
                let line = self.parley_shaper.shape_single_line_metrics(
                    TextInputRef::Attributed {
                        text: rich.text.as_ref(),
                        base: base_style,
                        spans: rich.spans.as_ref(),
                    },
                    scale,
                );
                let width_px = line.width;
                let baseline_px = line.baseline;
                let line_height_px = line.line_height;
                let clusters = line.clusters;

                let (line_count, max_w_px) = if width_px <= max_width_px + 0.5 {
                    (1, width_px.max(0.0))
                } else {
                    match constraints.wrap {
                        TextWrap::Word => {
                            word_wrap_line_stats(text, clusters.as_slice(), max_width_px)
                        }
                        TextWrap::Grapheme => {
                            grapheme_wrap_line_stats(text, clusters.as_slice(), max_width_px)
                        }
                        TextWrap::None => unreachable!(),
                    }
                };
                metrics_for_uniform_lines(max_w_px, line_count, baseline_px, line_height_px, scale)
            }
        } else {
            let wrapped = crate::text::wrapper::wrap_with_constraints_measure_only(
                &mut self.parley_shaper,
                TextInputRef::Attributed {
                    text: rich.text.as_ref(),
                    base: base_style,
                    spans: rich.spans.as_ref(),
                },
                normalized_constraints,
            );
            metrics_from_wrapped_lines(&wrapped.lines, scale)
        };

        let bucket = self.measure_cache.entry(key).or_default();
        bucket.push_back(TextMeasureEntry {
            text_hash,
            spans_hash,
            text: rich.text.clone(),
            spans: Some(rich.spans.clone()),
            metrics,
        });
        while bucket.len() > MEASURE_CACHE_PER_BUCKET_LIMIT {
            bucket.pop_front();
        }

        let mut metrics = metrics;
        if constraints.wrap == TextWrap::None
            && constraints.overflow == TextOverflow::Ellipsis
            && let Some(max_width) = constraints.max_width
        {
            metrics.size.width = max_width;
        }
        metrics
    }

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
        if stops.is_empty() {
            return Some(Px(0.0));
        }
        if let Some((_, x)) = stops.iter().find(|(i, _)| *i == index) {
            return Some(*x);
        }
        let mut last = Px(0.0);
        for (i, x) in stops {
            if *i > index {
                break;
            }
            last = *x;
        }
        Some(last)
    }

    pub fn hit_test_x(&self, blob: TextBlobId, x: Px) -> Option<usize> {
        let blob_id = blob;
        let blob = self.blobs.get(blob_id)?;
        if blob.shape.lines.len() > 1 {
            return Some(self.hit_test_point(blob_id, Point::new(x, Px(0.0)))?.index);
        }
        let stops = blob.shape.caret_stops.as_ref();
        if stops.is_empty() {
            return Some(0);
        }
        let mut best = stops[0].0;
        let mut best_dist = (stops[0].1.0 - x.0).abs();
        for (idx, px) in stops {
            let dist = (px.0 - x.0).abs();
            if dist < best_dist {
                best = *idx;
                best_dist = dist;
            }
        }
        Some(best)
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
        caret_rect_from_lines(blob.shape.lines.as_ref(), index, affinity)
    }

    pub fn hit_test_point(&self, blob: TextBlobId, point: Point) -> Option<HitTestResult> {
        let blob = self.blobs.get(blob)?;
        hit_test_point_from_lines(blob.shape.lines.as_ref(), point)
    }

    pub fn selection_rects(
        &self,
        blob: TextBlobId,
        range: (usize, usize),
        out: &mut Vec<Rect>,
    ) -> Option<()> {
        let blob = self.blobs.get(blob)?;
        selection_rects_from_lines(blob.shape.lines.as_ref(), range, out);
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
        selection_rects_from_lines_clipped(blob.shape.lines.as_ref(), range, clip, out);
        Some(())
    }

    pub fn release(&mut self, blob: TextBlobId) {
        let entries = released_blob_cache_entries();

        let Some(b) = self.blobs.get_mut(blob) else {
            return;
        };

        if b.ref_count > 1 {
            b.ref_count = b.ref_count.saturating_sub(1);
            return;
        }

        if b.ref_count == 0 {
            return;
        }

        if entries > 0 {
            b.ref_count = 0;
            self.insert_released_blob(blob, entries);
            return;
        }

        self.evict_blob(blob);
    }

    fn remove_released_blob(&mut self, id: TextBlobId) {
        if !self.released_blob_set.remove(&id) {
            return;
        }
        if let Some(pos) = self.released_blob_lru.iter().position(|v| *v == id) {
            self.released_blob_lru.remove(pos);
        }
    }

    fn insert_released_blob(&mut self, id: TextBlobId, entries: usize) {
        if entries == 0 {
            return;
        }

        if !self.released_blob_set.insert(id) {
            if let Some(pos) = self.released_blob_lru.iter().position(|v| *v == id) {
                self.released_blob_lru.remove(pos);
            }
        }
        self.released_blob_lru.push_back(id);

        while self.released_blob_lru.len() > entries {
            let Some(evict) = self.released_blob_lru.pop_front() else {
                break;
            };
            self.released_blob_set.remove(&evict);
            if self.blobs.get(evict).is_some_and(|b| b.ref_count > 0) {
                continue;
            }
            self.evict_blob(evict);
        }
    }

    fn clear_released_blob_cache(&mut self) {
        self.released_blob_lru.clear();
        self.released_blob_set.clear();
    }

    fn evict_blob(&mut self, blob: TextBlobId) {
        self.remove_released_blob(blob);

        let remove_shape = self
            .blobs
            .get(blob)
            .is_some_and(|b| Arc::strong_count(&b.shape) == 2);

        if let Some(key) = self.blob_key_by_id.remove(&blob) {
            self.blob_cache.remove(&key);
            if remove_shape {
                let shape_key = TextShapeKey::from_blob_key(&key);
                self.shape_cache.remove(&shape_key);
            }
        }
        let _ = self.blobs.remove(blob);
    }

    fn wrap_for_prepare(
        &mut self,
        input: TextInputRef<'_>,
        blob_key: &TextBlobKey,
        constraints: TextConstraints,
    ) -> WrappedForPrepare {
        let scale = constraints.scale_factor.max(1.0);
        let max_width = match constraints {
            TextConstraints {
                max_width: Some(max_width),
                wrap: TextWrap::Word,
                ..
            } => max_width,
            _ => {
                return WrappedForPrepare::Owned(crate::text::wrapper::wrap_with_constraints(
                    &mut self.parley_shaper,
                    input,
                    constraints,
                ));
            }
        };

        let text = match input {
            TextInputRef::Plain { text, .. } => text,
            TextInputRef::Attributed { text, .. } => text,
        };
        if text.contains('\n') {
            return WrappedForPrepare::Owned(crate::text::wrapper::wrap_with_constraints(
                &mut self.parley_shaper,
                input,
                constraints,
            ));
        }

        let entries = unwrapped_layout_cache_entries();
        if entries == 0 || text.len() > unwrapped_layout_cache_max_text_len_bytes() {
            return WrappedForPrepare::Owned(crate::text::wrapper::wrap_with_constraints(
                &mut self.parley_shaper,
                input,
                constraints,
            ));
        }

        let unwrapped = self.get_or_shape_unwrapped_layout(input, blob_key, scale, entries);
        let max_width_px = max_width.0 * scale;

        if let Some(lines) = crate::text::wrapper::wrap_word_slices_from_unwrapped_ltr(
            text,
            unwrapped.as_ref(),
            max_width_px,
        ) {
            return WrappedForPrepare::UnwrappedWordLtr {
                kept_end: text.len(),
                unwrapped,
                lines,
            };
        }

        WrappedForPrepare::Owned(crate::text::wrapper::wrap_with_constraints(
            &mut self.parley_shaper,
            input,
            constraints,
        ))
    }

    fn clear_unwrapped_layout_cache(&mut self) {
        self.unwrapped_layout_cache.clear();
        self.unwrapped_layout_lru.clear();
        self.unwrapped_layout_set.clear();
    }

    fn get_or_shape_unwrapped_layout(
        &mut self,
        input: TextInputRef<'_>,
        blob_key: &TextBlobKey,
        scale: f32,
        max_entries: usize,
    ) -> Arc<crate::text::parley_shaper::ShapedLineLayout> {
        let key = TextUnwrappedKey::from_blob_key(blob_key);

        if let Some(hit) = self.unwrapped_layout_cache.get(&key).cloned() {
            self.perf_frame_unwrapped_layout_cache_hits = self
                .perf_frame_unwrapped_layout_cache_hits
                .saturating_add(1);
            self.touch_unwrapped_lru(&key, max_entries);
            return hit;
        }
        self.perf_frame_unwrapped_layout_cache_misses = self
            .perf_frame_unwrapped_layout_cache_misses
            .saturating_add(1);

        let scale = if scale.is_finite() {
            scale.max(1.0)
        } else {
            1.0
        };
        let shaped = self.parley_shaper.shape_single_line(input, scale);
        let shaped = Arc::new(shaped);
        self.perf_frame_unwrapped_layouts_created =
            self.perf_frame_unwrapped_layouts_created.saturating_add(1);
        self.unwrapped_layout_cache
            .insert(key.clone(), shaped.clone());
        self.touch_unwrapped_lru(&key, max_entries);
        shaped
    }

    fn touch_unwrapped_lru(&mut self, key: &TextUnwrappedKey, max_entries: usize) {
        if max_entries == 0 {
            return;
        }
        if !self.unwrapped_layout_set.insert(key.clone()) {
            if let Some(pos) = self.unwrapped_layout_lru.iter().position(|k| k == key) {
                self.unwrapped_layout_lru.remove(pos);
            }
        }
        self.unwrapped_layout_lru.push_back(key.clone());

        while self.unwrapped_layout_lru.len() > max_entries {
            let Some(evict) = self.unwrapped_layout_lru.pop_front() else {
                break;
            };
            self.unwrapped_layout_set.remove(&evict);
            self.unwrapped_layout_cache.remove(&evict);
        }
    }
}

#[cfg(any())]
#[derive(Debug, Clone)]
struct PreparedLayout {
    metrics: TextMetrics,
    lines: Vec<cosmic_text::LayoutLine>,
    line_tops_px: Vec<f32>,
    local_starts: Vec<usize>,
    local_ends: Vec<usize>,
    paragraph_ends: Vec<usize>,
}

#[derive(Clone, Debug)]
struct ResolvedSpan {
    start: usize,
    end: usize,
    slot: u16,
    fg: Option<fret_core::Color>,
    underline: Option<ResolvedDecoration>,
    strikethrough: Option<ResolvedDecoration>,
}

#[derive(Clone, Debug)]
struct ResolvedDecoration {
    color: Option<fret_core::Color>,
}

fn resolve_spans_for_text(text: &str, spans: &[TextSpan]) -> Option<Vec<ResolvedSpan>> {
    if spans.is_empty() {
        return None;
    }

    let mut out: Vec<ResolvedSpan> = Vec::with_capacity(spans.len());
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
            let slot = u16::try_from(out.len()).ok()?;
            out.push(ResolvedSpan {
                start: offset,
                end,
                slot,
                fg: span.paint.fg,
                underline: span
                    .paint
                    .underline
                    .as_ref()
                    .map(|u| ResolvedDecoration { color: u.color }),
                strikethrough: span
                    .paint
                    .strikethrough
                    .as_ref()
                    .map(|s| ResolvedDecoration { color: s.color }),
            });
        }
        offset = end;
    }
    if offset != text.len() {
        return None;
    }

    Some(out)
}

fn span_has_any_overrides(span: &TextSpan) -> bool {
    span.shaping.font.is_some()
        || span.shaping.weight.is_some()
        || span.shaping.slant.is_some()
        || span.shaping.letter_spacing_em.is_some()
        || span.paint.fg.is_some()
        || span.paint.bg.is_some()
        || span.paint.underline.is_some()
        || span.paint.strikethrough.is_some()
}

fn clamp_down_to_char_boundary(text: &str, idx: usize) -> usize {
    let mut i = idx.min(text.len());
    while i > 0 && !text.is_char_boundary(i) {
        i = i.saturating_sub(1);
    }
    i
}

fn next_char_boundary(text: &str, idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    let idx = clamp_down_to_char_boundary(text, idx);
    if idx >= text.len() {
        return text.len();
    }
    let ch = text[idx..].chars().next().unwrap();
    idx + ch.len_utf8()
}

fn clamp_span_end_to_char_boundary(text: &str, start: usize, desired_end: usize) -> usize {
    let raw_end = desired_end.min(text.len());
    if text.is_char_boundary(raw_end) {
        return raw_end;
    }

    let down = clamp_down_to_char_boundary(text, raw_end);
    if down > start {
        return down;
    }

    let up = next_char_boundary(text, raw_end);
    up.max(start).min(text.len())
}

fn sanitize_spans_for_text(text: &str, spans: &[TextSpan]) -> Option<Arc<[TextSpan]>> {
    if spans.is_empty() || text.is_empty() {
        return None;
    }

    let text_len = text.len();
    let mut out: Vec<TextSpan> = Vec::with_capacity(spans.len().saturating_add(1));

    let mut offset: usize = 0;
    for span in spans {
        if offset >= text_len {
            break;
        }

        let desired_end = offset.saturating_add(span.len);
        let mut end = clamp_span_end_to_char_boundary(text, offset, desired_end);

        if end == offset && desired_end > offset {
            end = next_char_boundary(text, offset);
        }

        let mut s = span.clone();
        s.len = end.saturating_sub(offset);
        out.push(s);
        offset = end;
    }

    if offset < text_len {
        out.push(TextSpan::new(text_len - offset));
    }

    // Avoid forcing "attributed" shaping when spans carry no effective overrides.
    if out.len() == 1 && out[0].len == text_len && !span_has_any_overrides(&out[0]) {
        return None;
    }

    Some(Arc::<[TextSpan]>::from(out))
}

#[cfg(any())]
fn paint_span_for_glyph(
    spans: &[ResolvedSpan],
    base_offset: usize,
    g: &cosmic_text::LayoutGlyph,
) -> Option<u16> {
    let mut global = base_offset.saturating_add(g.start);
    if g.start == g.end && global > 0 {
        global = global.saturating_sub(1);
    }
    spans
        .iter()
        .find(|s| global >= s.start && global < s.end)
        .map(|s| s.slot)
}

fn paint_span_for_text_range(
    spans: &[ResolvedSpan],
    range: &std::ops::Range<usize>,
    is_rtl: bool,
) -> Option<u16> {
    let idx = if range.start == range.end {
        range.start.saturating_sub(1)
    } else if is_rtl {
        range.end.saturating_sub(1)
    } else {
        range.start
    };
    spans
        .iter()
        .find(|s| idx >= s.start && idx < s.end)
        .map(|s| s.slot)
}

#[cfg(any())]
fn layout_text(
    font_system: &mut FontSystem,
    scratch: &mut ShapeBuffer,
    text: &str,
    attrs: &Attrs,
    spans: Option<&[TextSpan]>,
    font_size_px: f32,
    constraints: TextConstraints,
    scale: f32,
) -> (PreparedLayout, Vec<usize>) {
    let max_width_px = constraints.max_width.map(|w| w.0 * scale);
    let wrap = match constraints.wrap {
        TextWrap::None => cosmic_text::Wrap::None,
        TextWrap::Word => cosmic_text::Wrap::Word,
    };

    let want_ellipsis = matches!(constraints.overflow, TextOverflow::Ellipsis)
        && matches!(constraints.wrap, TextWrap::None)
        && max_width_px.is_some();

    let mut all_lines: Vec<cosmic_text::LayoutLine> = Vec::new();
    let mut line_tops_px: Vec<f32> = Vec::new();
    let mut local_starts: Vec<usize> = Vec::new();
    let mut local_ends: Vec<usize> = Vec::new();
    let mut paragraph_ends: Vec<usize> = Vec::new();
    let mut line_starts_global: Vec<usize> = Vec::new();

    let mut max_w_px = 0.0_f32;
    let mut total_h_px = 0.0_f32;
    let mut first_ascent_px: Option<f32> = None;

    let resolved_spans: Option<Vec<ResolvedSpan>> =
        spans.and_then(|spans| resolve_spans_for_text(text, spans));

    let mut push_slice = |base_offset: usize, slice: &str, paragraph_end: usize| {
        let mut attrs_list = AttrsList::new(attrs);
        attrs_list.add_span(0..slice.len(), attrs);

        if let Some(spans) = resolved_spans.as_ref() {
            for span in spans {
                if span.end <= base_offset || span.start >= paragraph_end {
                    continue;
                }

                let start = span.start.max(base_offset) - base_offset;
                let end = span.end.min(paragraph_end) - base_offset;
                if start >= end || end > slice.len() {
                    continue;
                }

                let mut span_attrs = attrs.clone();
                if let Some(font) = span.font.as_ref() {
                    span_attrs = span_attrs.family(family_for_font_id(font));
                }
                if let Some(weight) = span.weight {
                    span_attrs = span_attrs.weight(Weight(weight.0));
                }
                if let Some(slant) = span.slant {
                    span_attrs = match slant {
                        TextSlant::Normal => span_attrs.style(CosmicStyle::Normal),
                        TextSlant::Italic => span_attrs.style(CosmicStyle::Italic),
                        TextSlant::Oblique => span_attrs.style(CosmicStyle::Oblique),
                    };
                }
                if let Some(letter_spacing_em) = span.letter_spacing_em
                    && letter_spacing_em != 0.0
                    && letter_spacing_em.is_finite()
                {
                    span_attrs = span_attrs.letter_spacing(letter_spacing_em);
                }
                attrs_list.add_span(start..end, &span_attrs);
            }
        }

        let shape_line = ShapeLine::new(font_system, slice, &attrs_list, Shaping::Advanced, 4);
        let mut layout_lines: Vec<cosmic_text::LayoutLine> = Vec::new();
        shape_line.layout_to_buffer(
            scratch,
            font_size_px,
            max_width_px,
            wrap,
            None,
            &mut layout_lines,
            None,
            Hinting::Disabled,
        );

        let mut ellipsis_local_end: Option<usize> = None;
        if want_ellipsis
            && layout_lines.len() == 1
            && let Some(max_w) = max_width_px
            && let Some(line) = layout_lines.get_mut(0)
            // Avoid spurious ellipses caused by subpixel layout rounding (especially visible in
            // list rows where the remaining gap makes the truncation look "wrong").
            && line.w > max_w + 0.5
        {
            let ellipsis_text = "…";
            let (ellipsis_w, ellipsis_glyphs) = {
                let mut ellipsis_attrs_list = AttrsList::new(attrs);
                ellipsis_attrs_list.add_span(0..ellipsis_text.len(), attrs);
                let ellipsis_shape = ShapeLine::new(
                    font_system,
                    ellipsis_text,
                    &ellipsis_attrs_list,
                    Shaping::Advanced,
                    4,
                );
                let mut ellipsis_lines: Vec<cosmic_text::LayoutLine> = Vec::new();
                ellipsis_shape.layout_to_buffer(
                    scratch,
                    font_size_px,
                    None,
                    cosmic_text::Wrap::None,
                    None,
                    &mut ellipsis_lines,
                    None,
                    Hinting::Disabled,
                );
                let w = ellipsis_lines.first().map(|l| l.w).unwrap_or(0.0);
                let glyphs = ellipsis_lines
                    .first()
                    .map(|l| l.glyphs.clone())
                    .unwrap_or_default();
                (w, glyphs)
            };

            let available_w = (max_w - ellipsis_w).max(0.0);
            let mut cut_end = 0usize;
            for g in &line.glyphs {
                let right = (g.x + g.w).max(0.0);
                if right <= available_w + 0.5 {
                    cut_end = cut_end.max(g.end.min(slice.len()));
                }
            }
            while cut_end > 0
                && slice
                    .as_bytes()
                    .get(cut_end.saturating_sub(1))
                    .is_some_and(|b| b.is_ascii_whitespace())
            {
                cut_end = cut_end.saturating_sub(1);
            }

            let mut kept: Vec<cosmic_text::LayoutGlyph> = line
                .glyphs
                .iter()
                .filter(|&g| g.end <= cut_end)
                .cloned()
                .collect();

            let ellipsis_start_x = (max_w - ellipsis_w).max(0.0);
            for mut g in ellipsis_glyphs {
                g.start = cut_end;
                g.end = cut_end;
                g.x = (g.x + ellipsis_start_x).max(0.0);
                kept.push(g);
            }
            line.glyphs = kept;
            line.w = max_w;
            ellipsis_local_end = Some(cut_end);
        }

        if layout_lines.is_empty() {
            layout_lines.push(cosmic_text::LayoutLine {
                w: 0.0,
                max_ascent: 0.0,
                max_descent: 0.0,
                line_height_opt: None,
                glyphs: Vec::new(),
            });
        }

        let layout_count = layout_lines.len();
        let mut expected_start_local: usize = 0;

        for (idx, ll) in layout_lines.into_iter().enumerate() {
            let mut local_end = ll
                .glyphs
                .iter()
                .map(|g| g.end)
                .max()
                .unwrap_or(expected_start_local);
            if idx + 1 == layout_count {
                local_end = slice.len();
            }
            if idx + 1 == layout_count
                && let Some(end) = ellipsis_local_end
            {
                local_end = end.min(slice.len());
            }

            let local_start = expected_start_local;
            expected_start_local = local_end;

            let ascent_px = ll.max_ascent.max(0.0);
            let descent_px = ll.max_descent.max(0.0);
            let min_height_px = (ascent_px + descent_px).max(0.0);
            let height_px = ll
                .line_height_opt
                .unwrap_or(min_height_px)
                .max(min_height_px)
                .max(0.0);

            // Center the baseline within the line box when line-height exceeds the font's
            // ascent+descent. This avoids visible "text floats up" artifacts when swapping fonts
            // (e.g. Nerd Fonts with unusual metrics) while keeping behavior unchanged when the
            // line box is tight.
            let padding_top_px = ((height_px - ascent_px - descent_px) * 0.5).max(0.0);
            let baseline_offset_px = padding_top_px + ascent_px;
            first_ascent_px.get_or_insert(baseline_offset_px);
            max_w_px = max_w_px.max(ll.w);

            line_tops_px.push(total_h_px);
            local_starts.push(local_start);
            local_ends.push(local_end);
            paragraph_ends.push(paragraph_end);
            line_starts_global.push(base_offset);

            total_h_px += height_px;
            all_lines.push(ll);
        }
    };

    let mut slice_start = 0usize;
    for (i, ch) in text.char_indices() {
        if ch != '\n' {
            continue;
        }
        push_slice(slice_start, &text[slice_start..i], i);
        slice_start = i + 1;
    }
    push_slice(slice_start, &text[slice_start..text.len()], text.len());

    let first_ascent_px = first_ascent_px.unwrap_or(0.0);
    let metrics = TextMetrics {
        size: Size::new(Px(max_w_px / scale), Px(total_h_px / scale)),
        baseline: Px(first_ascent_px / scale),
    };

    (
        PreparedLayout {
            metrics,
            lines: all_lines,
            line_tops_px,
            local_starts,
            local_ends,
            paragraph_ends,
        },
        line_starts_global,
    )
}

fn utf8_char_boundaries(text: &str) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(text.chars().count().saturating_add(2));
    out.push(0);
    for (i, _) in text.char_indices() {
        out.push(i);
    }
    out.push(text.len());
    out.sort_unstable();
    out.dedup();
    out
}

fn caret_stops_for_slice(
    slice: &str,
    base_offset: usize,
    clusters: &[crate::text::parley_shaper::ShapedCluster],
    line_width_px: f32,
    scale: f32,
    kept_end: usize,
) -> Vec<(usize, Px)> {
    let mut out: Vec<(usize, Px)> = Vec::new();
    let boundaries = utf8_char_boundaries(slice);

    if boundaries.is_empty() {
        return vec![(base_offset, Px(0.0))];
    }

    if clusters.is_empty() {
        for &b in &boundaries {
            let idx = base_offset + b;
            if idx > kept_end {
                continue;
            }
            let x = if b >= slice.len() {
                (line_width_px / scale).max(0.0)
            } else {
                0.0
            };
            out.push((idx, Px(x)));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
        out.dedup_by(|a, b| a.0 == b.0);
        return out;
    }

    let last_cluster_end = clusters
        .iter()
        .map(|c| c.text_range.end)
        .max()
        .unwrap_or(0)
        .min(slice.len());
    let effective_line_width_px = clusters
        .iter()
        .flat_map(|c| [c.x0, c.x1])
        .fold(line_width_px, |acc, x| acc.max(x.max(0.0)));

    let mut cluster_i = 0usize;
    for &b in &boundaries {
        let idx = base_offset + b;
        if idx > kept_end {
            continue;
        }

        while cluster_i + 1 < clusters.len() && clusters[cluster_i].text_range.end < b {
            cluster_i = cluster_i.saturating_add(1);
        }

        let x = if b <= clusters[0].text_range.start {
            let first = &clusters[0];
            if first.is_rtl {
                first.x1.max(0.0)
            } else {
                first.x0.max(0.0)
            }
        } else if b > last_cluster_end {
            let last = clusters.last().unwrap_or(&clusters[0]);
            if last.is_rtl {
                0.0
            } else {
                effective_line_width_px
            }
        } else if cluster_i >= clusters.len() {
            let last = clusters.last().unwrap_or(&clusters[0]);
            if last.is_rtl {
                0.0
            } else {
                line_width_px.max(0.0)
            }
        } else {
            let c = &clusters[cluster_i];
            let start = c.text_range.start.min(slice.len());
            let end = c.text_range.end.min(slice.len());

            if start == end {
                c.x0.max(0.0)
            } else if b <= start {
                if c.is_rtl {
                    c.x1.max(0.0)
                } else {
                    c.x0.max(0.0)
                }
            } else if b >= end {
                if c.is_rtl {
                    c.x0.max(0.0)
                } else {
                    c.x1.max(0.0)
                }
            } else {
                let denom = (end - start) as f32;
                let mut t = ((b - start) as f32 / denom).clamp(0.0, 1.0);
                if c.is_rtl {
                    t = 1.0 - t;
                }
                let w = (c.x1 - c.x0).max(0.0);
                (c.x0 + w * t).max(0.0)
            }
        };

        out.push((idx, Px((x / scale).max(0.0))));
    }

    out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

fn caret_stops_for_slice_from_unwrapped_ltr(
    slice: &str,
    base_offset: usize,
    clusters: &[crate::text::parley_shaper::ShapedCluster],
    cluster_range: std::ops::Range<usize>,
    line_start_x: f32,
    line_width_px: f32,
    scale: f32,
    kept_end: usize,
) -> Vec<(usize, Px)> {
    let mut out: Vec<(usize, Px)> = Vec::new();
    let boundaries = utf8_char_boundaries(slice);

    if boundaries.is_empty() {
        return vec![(base_offset, Px(0.0))];
    }

    let clusters = clusters
        .get(cluster_range)
        .unwrap_or(&[] as &[crate::text::parley_shaper::ShapedCluster]);

    if clusters.is_empty() {
        for &b in &boundaries {
            let idx = base_offset + b;
            if idx > kept_end {
                continue;
            }
            let x = if b >= slice.len() {
                (line_width_px / scale).max(0.0)
            } else {
                0.0
            };
            out.push((idx, Px(x)));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
        out.dedup_by(|a, b| a.0 == b.0);
        return out;
    }

    let mut last_cluster_end = 0usize;
    let mut effective_line_width_px = line_width_px.max(0.0);
    for c in clusters {
        last_cluster_end = last_cluster_end.max(c.text_range.end.saturating_sub(base_offset));
        effective_line_width_px = effective_line_width_px
            .max((c.x0 - line_start_x).max(0.0))
            .max((c.x1 - line_start_x).max(0.0));
    }
    last_cluster_end = last_cluster_end.min(slice.len());

    let first_local_start = clusters[0].text_range.start.saturating_sub(base_offset);

    let mut cluster_i = 0usize;
    for &b in &boundaries {
        let idx = base_offset + b;
        if idx > kept_end {
            continue;
        }

        while cluster_i + 1 < clusters.len()
            && clusters[cluster_i]
                .text_range
                .end
                .saturating_sub(base_offset)
                < b
        {
            cluster_i = cluster_i.saturating_add(1);
        }

        let x_px = if b <= first_local_start {
            let first = &clusters[0];
            if first.is_rtl {
                (first.x1 - line_start_x).max(0.0)
            } else {
                (first.x0 - line_start_x).max(0.0)
            }
        } else if b > last_cluster_end {
            let last = clusters.last().unwrap_or(&clusters[0]);
            if last.is_rtl {
                0.0
            } else {
                effective_line_width_px
            }
        } else if cluster_i >= clusters.len() {
            let last = clusters.last().unwrap_or(&clusters[0]);
            if last.is_rtl {
                0.0
            } else {
                line_width_px.max(0.0)
            }
        } else {
            let c = &clusters[cluster_i];
            let start = c
                .text_range
                .start
                .saturating_sub(base_offset)
                .min(slice.len());
            let end = c
                .text_range
                .end
                .saturating_sub(base_offset)
                .min(slice.len());

            let x0 = (c.x0 - line_start_x).max(0.0);
            let x1 = (c.x1 - line_start_x).max(0.0);

            if start == end {
                x0
            } else if b <= start {
                if c.is_rtl { x1 } else { x0 }
            } else if b >= end {
                if c.is_rtl { x0 } else { x1 }
            } else {
                let denom = (end - start) as f32;
                let mut t = ((b - start) as f32 / denom).clamp(0.0, 1.0);
                if c.is_rtl {
                    t = 1.0 - t;
                }
                let w = (x1 - x0).max(0.0);
                (x0 + w * t).max(0.0)
            }
        };

        out.push((idx, Px((x_px / scale).max(0.0))));
    }

    out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

#[cfg(any())]
fn build_line_caret_stops(
    base_offset: usize,
    boundaries_local: &[usize],
    glyphs: &[cosmic_text::LayoutGlyph],
    local_start: usize,
    local_end: usize,
    line_w_px: f32,
    scale: f32,
) -> Vec<(usize, Px)> {
    let mut out: Vec<(usize, Px)> = Vec::with_capacity(boundaries_local.len());
    for &idx_local in boundaries_local {
        let idx_global = base_offset + idx_local;
        if idx_local <= local_start {
            out.push((idx_global, Px(0.0)));
            continue;
        }
        if idx_local >= local_end {
            out.push((idx_global, Px(line_w_px / scale)));
            continue;
        }

        let mut x_end = 0.0_f32;
        for g in glyphs {
            if g.end <= idx_local {
                x_end = x_end.max(g.x + g.w);
            }
        }
        out.push((idx_global, Px(x_end / scale)));
    }
    out.sort_by_key(|(idx, _)| *idx);
    out.dedup_by_key(|(idx, _)| *idx);
    out
}

fn caret_x_from_stops(stops: &[(usize, Px)], index: usize) -> Px {
    if stops.is_empty() {
        return Px(0.0);
    }
    if let Ok(pos) = stops.binary_search_by_key(&index, |(idx, _)| *idx) {
        return stops[pos].1;
    }
    match stops.partition_point(|(idx, _)| *idx <= index) {
        0 => stops[0].1,
        n => stops[n.saturating_sub(1)].1,
    }
}

fn hit_test_x_from_stops(stops: &[(usize, Px)], x: Px) -> usize {
    if stops.is_empty() {
        return 0;
    }
    let mut best = stops[0].0;
    let mut best_dist = (stops[0].1.0 - x.0).abs();
    for (idx, px) in stops {
        let dist = (px.0 - x.0).abs();
        if dist < best_dist {
            best = *idx;
            best_dist = dist;
        }
    }
    best
}

fn caret_rect_from_lines(
    lines: &[TextLine],
    index: usize,
    affinity: CaretAffinity,
) -> Option<Rect> {
    if lines.is_empty() {
        return None;
    }

    let mut candidates: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if index >= line.start && index <= line.end {
            candidates.push(i);
        }
    }

    let line_idx = match candidates.as_slice() {
        [] => {
            if index <= lines[0].start {
                0
            } else {
                lines.len().saturating_sub(1)
            }
        }
        [only] => *only,
        many => match affinity {
            CaretAffinity::Upstream => many[0],
            CaretAffinity::Downstream => many[many.len().saturating_sub(1)],
        },
    };

    let line = &lines[line_idx];
    let x = caret_x_from_stops(&line.caret_stops, index);
    Some(Rect::new(
        Point::new(x, line.y_top),
        Size::new(Px(1.0), line.height),
    ))
}

fn hit_test_point_from_lines(lines: &[TextLine], point: Point) -> Option<HitTestResult> {
    if lines.is_empty() {
        return None;
    }

    let mut line_idx = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let y0 = line.y_top.0;
        let y1 = (line.y_top.0 + line.height.0).max(y0);
        if point.y.0 >= y0 && point.y.0 < y1 {
            line_idx = i;
            break;
        }
        if point.y.0 >= y1 {
            line_idx = i;
        }
    }

    let line = &lines[line_idx];
    let index = hit_test_x_from_stops(&line.caret_stops, point.x);

    let mut affinity = CaretAffinity::Downstream;
    if line_idx + 1 < lines.len() && index == line.end && lines[line_idx + 1].start == index {
        affinity = CaretAffinity::Upstream;
    }

    Some(HitTestResult { index, affinity })
}

fn selection_rects_from_lines(lines: &[TextLine], range: (usize, usize), out: &mut Vec<Rect>) {
    out.clear();
    if lines.is_empty() {
        return;
    }

    let (a, b) = (range.0.min(range.1), range.0.max(range.1));
    if a == b {
        return;
    }

    for line in lines {
        let start = a.max(line.start);
        let end = b.min(line.end);
        if start >= end {
            continue;
        }

        let x0 = caret_x_from_stops(&line.caret_stops, start);
        let x1 = caret_x_from_stops(&line.caret_stops, end);
        let left = Px(x0.0.min(x1.0));
        let right = Px(x0.0.max(x1.0));

        out.push(Rect::new(
            Point::new(left, line.y_top),
            Size::new(Px((right.0 - left.0).max(0.0)), line.height),
        ));
    }

    coalesce_selection_rects_in_place(out);
}

fn selection_rects_from_lines_clipped(
    lines: &[TextLine],
    range: (usize, usize),
    clip: Rect,
    out: &mut Vec<Rect>,
) {
    out.clear();
    if lines.is_empty() {
        return;
    }

    let clip_x0 = clip.origin.x.0;
    let clip_y0 = clip.origin.y.0;
    let clip_x1 = clip_x0 + clip.size.width.0;
    let clip_y1 = clip_y0 + clip.size.height.0;
    if clip_x1 <= clip_x0 || clip_y1 <= clip_y0 {
        return;
    }

    let (a, b) = (range.0.min(range.1), range.0.max(range.1));
    if a == b {
        return;
    }

    let start_idx = lines.partition_point(|line| {
        let y0 = line.y_top.0;
        let y1 = (line.y_top.0 + line.height.0).max(y0);
        y1 <= clip_y0
    });
    let end_idx = lines.partition_point(|line| line.y_top.0 < clip_y1);
    let start_idx = start_idx.min(end_idx);
    if start_idx >= end_idx {
        return;
    }

    for line in &lines[start_idx..end_idx] {
        let start = a.max(line.start);
        let end = b.min(line.end);
        if start >= end {
            continue;
        }

        let x0 = caret_x_from_stops(&line.caret_stops, start).0;
        let x1 = caret_x_from_stops(&line.caret_stops, end).0;
        let left = x0.min(x1);
        let right = x0.max(x1);

        let y0 = line.y_top.0;
        let y1 = (line.y_top.0 + line.height.0).max(y0);

        let ix0 = left.max(clip_x0);
        let iy0 = y0.max(clip_y0);
        let ix1 = right.min(clip_x1);
        let iy1 = y1.min(clip_y1);

        if ix1 <= ix0 || iy1 <= iy0 {
            continue;
        }

        out.push(Rect::new(
            Point::new(Px(ix0), Px(iy0)),
            Size::new(Px((ix1 - ix0).max(0.0)), Px((iy1 - iy0).max(0.0))),
        ));
    }

    coalesce_selection_rects_in_place(out);
}

fn coalesce_selection_rects_in_place(rects: &mut Vec<Rect>) {
    if rects.len() <= 1 {
        return;
    }

    let mut out: Vec<Rect> = Vec::with_capacity(rects.len());
    for r in rects.drain(..) {
        match out.last_mut() {
            Some(prev)
                if prev.origin.y == r.origin.y
                    && prev.size.height == r.size.height
                    && r.origin.x.0 <= prev.origin.x.0 + prev.size.width.0 =>
            {
                let x0 = prev.origin.x.0.min(r.origin.x.0);
                let x1 = (prev.origin.x.0 + prev.size.width.0).max(r.origin.x.0 + r.size.width.0);
                prev.origin.x = Px(x0);
                prev.size.width = Px((x1 - x0).max(0.0));
            }
            _ => out.push(r),
        }
    }
    *rects = out;
}

fn decorations_for_lines(
    lines: &[TextLine],
    spans: &[ResolvedSpan],
    scale: f32,
    snap_vertical: bool,
) -> Vec<TextDecoration> {
    let thickness_px = 1.0_f32;
    let thickness = Px(thickness_px / scale.max(1.0));

    let mut out: Vec<TextDecoration> = Vec::new();
    if lines.is_empty() || spans.is_empty() {
        return out;
    }

    for line in lines {
        let line_top_px = line.y_top.0 * scale;
        let line_bottom_px = (line.y_top.0 + line.height.0).max(line.y_top.0) * scale;
        let baseline_px = line.y_baseline.0 * scale;

        // Underline: anchor to the baseline and snap in device px under fractional scaling.
        let underline_bottom_px_raw = baseline_px + 1.0;
        let underline_bottom_px = if snap_vertical {
            underline_bottom_px_raw.round()
        } else {
            underline_bottom_px_raw
        }
        .clamp(line_top_px, line_bottom_px);
        let underline_top_px =
            (underline_bottom_px - thickness_px).clamp(line_top_px, line_bottom_px - thickness_px);
        let underline_y = Px((underline_top_px / scale).max(0.0));

        // Strikethrough: approximate as a fraction of the line height above the baseline.
        let line_height_px = (line.height.0.max(0.0) * scale).max(0.0);
        let strike_offset_px_raw = (line_height_px * 0.30).clamp(1.0, line_height_px);
        let strike_bottom_px_raw = baseline_px - strike_offset_px_raw;
        let strike_bottom_px = if snap_vertical {
            strike_bottom_px_raw.round()
        } else {
            strike_bottom_px_raw
        }
        .clamp(line_top_px, line_bottom_px);
        let strike_top_px =
            (strike_bottom_px - thickness_px).clamp(line_top_px, line_bottom_px - thickness_px);
        let strike_y = Px((strike_top_px / scale).max(0.0));

        for span in spans {
            if span.underline.is_none() && span.strikethrough.is_none() {
                continue;
            }

            let start = span.start.max(line.start);
            let end = span.end.min(line.end);
            if start >= end {
                continue;
            }

            let x0 = caret_x_from_stops(&line.caret_stops, start);
            let x1 = caret_x_from_stops(&line.caret_stops, end);
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

#[cfg(test)]
mod tests {
    use super::{
        ResolvedSpan, TextBlobKey, TextDecorationKind, TextMeasureKey, TextShapeKey,
        collect_font_names, paint_span_for_text_range, spans_paint_fingerprint,
        spans_shaping_fingerprint, subpixel_mask_to_alpha,
    };
    use cosmic_text::Family;
    use fret_core::{
        AttributedText, CaretAffinity, Color, DecorationLineStyle, FontWeight, Point, Px, Rect,
        Size, StrikethroughStyle, TextConstraints, TextInputRef, TextOverflow, TextSpan, TextStyle,
        TextWrap, UnderlineStyle,
    };
    use std::sync::Arc;

    #[test]
    fn subpixel_mask_to_alpha_uses_channel_max() {
        let data = vec![
            10u8, 3u8, 4u8, 0u8, //
            1u8, 200u8, 2u8, 0u8,
        ];
        assert_eq!(subpixel_mask_to_alpha(&data), vec![10u8, 200u8]);
    }

    #[test]
    fn paint_span_for_text_range_is_directional_across_span_boundary() {
        let spans = vec![
            ResolvedSpan {
                start: 0,
                end: 3,
                slot: 0,
                fg: None,
                underline: None,
                strikethrough: None,
            },
            ResolvedSpan {
                start: 3,
                end: 6,
                slot: 1,
                fg: None,
                underline: None,
                strikethrough: None,
            },
        ];

        // Cluster spans the boundary (2..4). We cannot split the glyph, so we pick a deterministic
        // representative index based on direction.
        assert_eq!(paint_span_for_text_range(&spans, &(2..4), false), Some(0));
        assert_eq!(paint_span_for_text_range(&spans, &(2..4), true), Some(1));

        // Synthetic 0-length ranges (e.g. ellipsis mapping) should inherit the preceding style.
        assert_eq!(paint_span_for_text_range(&spans, &(3..3), false), Some(0));
        assert_eq!(paint_span_for_text_range(&spans, &(3..3), true), Some(0));
    }

    #[test]
    fn caret_stops_for_slice_interpolates_within_cluster_ltr() {
        let clusters = vec![crate::text::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: false,
        }];

        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let x_at = |i: usize| stops.iter().find(|(idx, _)| *idx == i).unwrap().1.0;

        assert_eq!(x_at(0), 0.0);
        assert_eq!(x_at(1), 10.0);
        assert_eq!(x_at(2), 20.0);
        assert_eq!(x_at(3), 30.0);
        assert_eq!(x_at(4), 40.0);
    }

    #[test]
    fn caret_stops_for_slice_interpolates_within_cluster_rtl() {
        let clusters = vec![crate::text::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: true,
        }];

        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let x_at = |i: usize| stops.iter().find(|(idx, _)| *idx == i).unwrap().1.0;

        assert_eq!(x_at(0), 40.0);
        assert_eq!(x_at(1), 30.0);
        assert_eq!(x_at(2), 20.0);
        assert_eq!(x_at(3), 10.0);
        assert_eq!(x_at(4), 0.0);
    }

    fn is_synthetic_rtl_char(ch: char) -> bool {
        // A minimal heuristic for test inputs; the production shaper determines RTL runs via
        // Unicode properties.
        matches!(
            ch,
            '\u{0590}'..='\u{05FF}' // Hebrew
                | '\u{0600}'..='\u{06FF}' // Arabic
                | '\u{0750}'..='\u{077F}' // Arabic Supplement
                | '\u{08A0}'..='\u{08FF}' // Arabic Extended-A
        )
    }

    fn synthetic_clusters_for_text(
        text: &str,
        advance: f32,
    ) -> Vec<crate::text::parley_shaper::ShapedCluster> {
        let mut out = Vec::new();
        let mut x = 0.0_f32;
        for (start, ch) in text.char_indices() {
            let end = start + ch.len_utf8();
            out.push(crate::text::parley_shaper::ShapedCluster {
                text_range: start..end,
                x0: x,
                x1: x + advance,
                is_rtl: is_synthetic_rtl_char(ch),
            });
            x += advance;
        }
        out
    }

    #[test]
    fn word_wrap_stats_do_not_wrap_when_full_line_fits() {
        let text = "hello world";
        let clusters = synthetic_clusters_for_text(text, 10.0);
        let (lines, max_w) = super::word_wrap_line_stats(text, &clusters, 1000.0);
        assert_eq!(lines, 1);
        assert_eq!(max_w, 110.0);
    }

    #[test]
    fn word_wrap_stats_wrap_at_space_boundary() {
        let text = "hello world";
        let clusters = synthetic_clusters_for_text(text, 10.0);
        let (lines, max_w) = super::word_wrap_line_stats(text, &clusters, 60.0);
        assert_eq!(lines, 2);
        assert_eq!(max_w, 60.0);
    }

    #[test]
    fn selection_rects_for_rtl_line_has_positive_width() {
        let clusters = vec![crate::text::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: true,
        }];
        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let line = super::TextLine {
            start: 0,
            end: 4,
            width: Px(40.0),
            y_top: Px(0.0),
            y_baseline: Px(0.0),
            height: Px(10.0),
            caret_stops: stops,
        };

        let mut rects = Vec::new();
        super::selection_rects_from_lines(&[line], (0, 4), &mut rects);
        assert_eq!(rects.len(), 1);
        assert!((rects[0].origin.x.0 - 0.0).abs() < 0.001);
        assert!((rects[0].size.width.0 - 40.0).abs() < 0.001);
    }

    #[test]
    fn hit_test_point_for_rtl_line_maps_left_edge_to_logical_end() {
        let clusters = vec![crate::text::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: true,
        }];
        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let line = super::TextLine {
            start: 0,
            end: 4,
            width: Px(40.0),
            y_top: Px(0.0),
            y_baseline: Px(0.0),
            height: Px(10.0),
            caret_stops: stops,
        };

        let left = super::hit_test_point_from_lines(
            std::slice::from_ref(&line),
            fret_core::Point::new(Px(0.0), Px(5.0)),
        )
        .expect("hit test");
        assert_eq!(left.index, 4);

        let right = super::hit_test_point_from_lines(
            std::slice::from_ref(&line),
            fret_core::Point::new(Px(40.0), Px(5.0)),
        )
        .expect("hit test");
        assert_eq!(right.index, 0);
    }

    #[test]
    fn mixed_direction_selection_rects_are_nonempty() {
        // Mixed LTR + RTL + numbers + punctuation.
        let text = "abc אבג (123)";
        let clusters = synthetic_clusters_for_text(text, 10.0);
        let stops = super::caret_stops_for_slice(
            text,
            0,
            &clusters,
            10.0 * clusters.len() as f32,
            1.0,
            text.len(),
        );
        let line = super::TextLine {
            start: 0,
            end: text.len(),
            width: Px(10.0 * clusters.len() as f32),
            y_top: Px(0.0),
            y_baseline: Px(0.0),
            height: Px(10.0),
            caret_stops: stops,
        };

        let rtl_start = text.find('א').expect("hebrew start");
        let rtl_end = text.find('ג').expect("hebrew end") + 'ג'.len_utf8();

        let mut rects = Vec::new();
        super::selection_rects_from_lines(&[line], (rtl_start, rtl_end), &mut rects);
        assert_eq!(rects.len(), 1);
        assert!(
            rects[0].size.width.0 > 0.1,
            "expected a non-empty selection rect"
        );
    }

    #[test]
    fn selection_rects_clipped_culls_offscreen_lines() {
        let mut lines = Vec::new();
        for i in 0..1000usize {
            let start = i * 4;
            let end = start + 4;
            lines.push(super::TextLine {
                start,
                end,
                width: Px(100.0),
                y_top: Px((i as f32) * 10.0),
                y_baseline: Px(0.0),
                height: Px(10.0),
                caret_stops: vec![(start, Px(0.0)), (end, Px(100.0))],
            });
        }

        let clip = Rect::new(
            Point::new(Px(0.0), Px(1000.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let mut rects = Vec::new();
        super::selection_rects_from_lines_clipped(&lines, (0, 4000), clip, &mut rects);

        assert_eq!(rects.len(), 10);
        for r in &rects {
            assert!(r.origin.y.0 >= 1000.0 && r.origin.y.0 < 1100.0);
            assert!(r.size.height.0 > 0.0);
        }
    }

    #[test]
    fn selection_rects_clipped_trims_partially_visible_line() {
        let line = super::TextLine {
            start: 0,
            end: 4,
            width: Px(100.0),
            y_top: Px(0.0),
            y_baseline: Px(0.0),
            height: Px(10.0),
            caret_stops: vec![(0, Px(0.0)), (4, Px(100.0))],
        };
        let clip = Rect::new(Point::new(Px(0.0), Px(5.0)), Size::new(Px(100.0), Px(10.0)));
        let mut rects = Vec::new();
        super::selection_rects_from_lines_clipped(&[line], (0, 4), clip, &mut rects);

        assert_eq!(rects.len(), 1);
        assert!((rects[0].origin.y.0 - 5.0).abs() < 0.001);
        assert!((rects[0].size.height.0 - 5.0).abs() < 0.001);
    }

    #[test]
    fn all_font_names_is_sorted_and_deduped() {
        // This is intentionally platform-dependent; we only assert structural invariants.
        let (locale, db) = cosmic_text::FontSystem::new().into_locale_and_db();
        let _ = locale;

        let names = collect_font_names(&db);

        assert!(
            names
                .iter()
                .any(|n| n == db.family_name(&Family::SansSerif)),
            "expected sans-serif generic family to be present"
        );
        assert!(
            names.iter().any(|n| n == db.family_name(&Family::Serif)),
            "expected serif generic family to be present"
        );
        assert!(
            names
                .iter()
                .any(|n| n == db.family_name(&Family::Monospace)),
            "expected monospace generic family to be present"
        );

        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for name in &names {
            assert!(
                seen.insert(name.to_ascii_lowercase()),
                "expected case-insensitive dedupe for {name:?}"
            );
        }

        for w in names.windows(2) {
            assert!(
                w[0].to_ascii_lowercase() <= w[1].to_ascii_lowercase(),
                "expected case-insensitive sort"
            );
        }
    }

    #[test]
    fn text_blob_key_includes_typography_fields() {
        let constraints = TextConstraints {
            max_width: Some(Px(120.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 2.0,
        };

        let base = TextStyle::default();
        let k0 = TextBlobKey::new("hello", &base, constraints, 1);

        let mut style = base.clone();
        style.weight = FontWeight::BOLD;
        let k_weight = TextBlobKey::new("hello", &style, constraints, 1);
        assert_ne!(k0, k_weight);

        let mut style = base.clone();
        style.line_height = Some(Px(18.0));
        let k_line_height = TextBlobKey::new("hello", &style, constraints, 1);
        assert_ne!(k0, k_line_height);

        let mut style = base.clone();
        style.letter_spacing_em = Some(0.05);
        let k_tracking = TextBlobKey::new("hello", &style, constraints, 1);
        assert_ne!(k0, k_tracking);
    }

    #[test]
    fn text_blob_key_includes_font_fallback_policy() {
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let base = TextStyle::default();
        let k0 = TextBlobKey::new("hello", &base, constraints, 1);
        let k1 = TextBlobKey::new("hello", &base, constraints, 2);
        assert_ne!(k0, k1);
    }

    #[test]
    fn text_measure_key_ignores_width_for_wrap_none() {
        let style = TextStyle::default();

        let a = TextConstraints {
            max_width: Some(Px(120.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };
        let b = TextConstraints {
            max_width: Some(Px(320.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        assert_eq!(
            TextMeasureKey::new(&style, a, 7),
            TextMeasureKey::new(&style, b, 7)
        );
    }

    #[test]
    fn text_measure_key_includes_width_for_wrap_word() {
        let style = TextStyle::default();

        let a = TextConstraints {
            max_width: Some(Px(120.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };
        let b = TextConstraints {
            max_width: Some(Px(320.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        assert_ne!(
            TextMeasureKey::new(&style, a, 7),
            TextMeasureKey::new(&style, b, 7)
        );
    }

    #[test]
    fn sanitize_spans_extends_missing_tail() {
        let text = "hello";
        let spans = vec![TextSpan {
            len: 2,
            paint: fret_core::TextPaintStyle {
                fg: Some(fret_core::Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                ..Default::default()
            },
            ..Default::default()
        }];

        let sanitized = super::sanitize_spans_for_text(text, &spans).expect("sanitized spans");
        let rich = fret_core::AttributedText {
            text: Arc::<str>::from(text),
            spans: sanitized.clone(),
        };

        assert!(rich.is_valid());
        assert_eq!(sanitized.iter().map(|s| s.len).sum::<usize>(), text.len());
        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0].len, 2);
        assert!(sanitized[0].paint.fg.is_some());
        assert_eq!(sanitized[1].len, 3);
        assert_eq!(sanitized[1].paint.fg, None);
    }

    #[test]
    fn sanitize_spans_truncates_overflowing_last_span() {
        let text = "hello";
        let spans = vec![TextSpan {
            len: 999,
            paint: fret_core::TextPaintStyle {
                fg: Some(fret_core::Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                }),
                ..Default::default()
            },
            ..Default::default()
        }];

        let sanitized = super::sanitize_spans_for_text(text, &spans).expect("sanitized spans");
        let rich = fret_core::AttributedText {
            text: Arc::<str>::from(text),
            spans: sanitized.clone(),
        };

        assert!(rich.is_valid());
        assert_eq!(sanitized.iter().map(|s| s.len).sum::<usize>(), text.len());
        assert_eq!(sanitized.len(), 1);
        assert_eq!(sanitized[0].len, text.len());
        assert!(sanitized[0].paint.fg.is_some());
    }

    #[test]
    fn sanitize_spans_snaps_to_char_boundaries() {
        let text = "aé";
        assert_eq!(text.len(), 3);

        let spans = vec![
            TextSpan {
                len: 2,
                paint: fret_core::TextPaintStyle {
                    fg: Some(fret_core::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            TextSpan {
                len: 1,
                paint: fret_core::TextPaintStyle {
                    fg: Some(fret_core::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];

        let sanitized = super::sanitize_spans_for_text(text, &spans).expect("sanitized spans");
        let rich = fret_core::AttributedText {
            text: Arc::<str>::from(text),
            spans: sanitized.clone(),
        };

        assert!(rich.is_valid());
        assert_eq!(sanitized.iter().map(|s| s.len).sum::<usize>(), text.len());
        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0].len, 1);
        assert_eq!(sanitized[1].len, 2);
        assert_eq!(sanitized[0].paint.fg, spans[0].paint.fg);
        assert_eq!(sanitized[1].paint.fg, spans[1].paint.fg);
    }

    #[test]
    fn sanitize_spans_returns_none_for_noop_full_span() {
        let text = "hello";
        let spans = vec![TextSpan::new(text.len())];
        assert!(super::sanitize_spans_for_text(text, &spans).is_none());
    }

    #[test]
    fn multiline_metrics_are_pixel_snapped_under_non_integer_scale_factor() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut text = super::TextSystem::new(&ctx.device);

        let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let added = text.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load");

        let content = {
            let mut out = String::new();
            for _ in 0..200 {
                out.push_str("The quick brown fox jumps over the lazy dog. ");
            }
            out
        };

        let scale_factor = 1.25_f32;
        let constraints = TextConstraints {
            max_width: Some(Px(120.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor,
        };
        let style = TextStyle {
            font: fret_core::FontId::monospace(),
            size: Px(13.0),
            ..Default::default()
        };

        let (blob_id, metrics) = text.prepare(&content, &style, constraints);
        let blob = text.blob(blob_id).expect("prepared blob");
        let lines = blob.shape.lines.as_ref();
        assert!(lines.len() > 10, "expected multi-line layout");

        let is_pixel_aligned = |logical: Px| {
            let px = logical.0 * scale_factor;
            (px - px.round()).abs() < 1e-3
        };

        assert!(
            is_pixel_aligned(metrics.baseline),
            "expected baseline to align to device pixels under fractional scale"
        );

        let mut prev_y_px = -1.0_f32;
        for line in lines {
            let y_px = line.y_top.0 * scale_factor;
            let h_px = line.height.0 * scale_factor;
            let baseline_px = line.y_baseline.0 * scale_factor;
            assert!(
                (y_px - y_px.round()).abs() < 1e-3,
                "expected y_top to be pixel-aligned, got {y_px}"
            );
            assert!(
                (h_px - h_px.round()).abs() < 1e-3 && h_px > 0.0,
                "expected line height to be positive and pixel-aligned, got {h_px}"
            );
            assert!(
                y_px + 0.5 >= prev_y_px,
                "expected non-decreasing y_top across lines"
            );
            prev_y_px = y_px;
            assert!(
                (baseline_px - baseline_px.round()).abs() < 1e-3,
                "expected per-line baseline to be pixel-aligned, got {baseline_px}"
            );
        }
    }

    #[test]
    fn decorations_are_pixel_snapped_under_non_integer_scale_factor() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut text = super::TextSystem::new(&ctx.device);

        let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let added = text.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load");

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
            scale_factor,
        };
        let style = TextStyle {
            font: fret_core::FontId::monospace(),
            size: Px(13.0),
            ..Default::default()
        };

        let mut span = TextSpan::new(content.len());
        span.paint.underline = Some(UnderlineStyle {
            color: None,
            style: DecorationLineStyle::Solid,
        });
        span.paint.strikethrough = Some(StrikethroughStyle {
            color: None,
            style: DecorationLineStyle::Solid,
        });
        let rich = AttributedText::new(Arc::<str>::from(content.as_str()), Arc::from([span]));

        let (blob_id, metrics) = text.prepare_attributed(&rich, &style, constraints);
        let blob = text.blob(blob_id).expect("prepared blob");

        let underlines: Vec<_> = blob
            .decorations
            .iter()
            .filter(|d| d.kind == TextDecorationKind::Underline)
            .collect();
        let strikes: Vec<_> = blob
            .decorations
            .iter()
            .filter(|d| d.kind == TextDecorationKind::Strikethrough)
            .collect();
        assert!(
            !underlines.is_empty(),
            "expected underline decorations to be generated"
        );
        assert!(
            !strikes.is_empty(),
            "expected strikethrough decorations to be generated"
        );

        let is_pixel_aligned = |logical: Px| {
            let px = logical.0 * scale_factor;
            (px - px.round()).abs() < 1e-3
        };

        for d in underlines.iter().chain(strikes.iter()) {
            assert!(
                is_pixel_aligned(d.rect.origin.y),
                "expected decoration y to be pixel-aligned"
            );
            assert!(
                is_pixel_aligned(d.rect.size.height),
                "expected decoration height to be pixel-aligned"
            );

            let h_px = d.rect.size.height.0 * scale_factor;
            assert!(
                (h_px - 1.0).abs() < 1e-3,
                "expected a 1px hairline decoration thickness, got {h_px}"
            );

            assert!(
                d.rect.origin.y.0 >= -1e-3,
                "expected decoration to stay within the text box (top)"
            );
            assert!(
                d.rect.origin.y.0 + d.rect.size.height.0 <= metrics.size.height.0 + 1e-3,
                "expected decoration to stay within the text box (bottom)"
            );
        }
    }

    #[test]
    fn trailing_space_at_soft_wrap_is_selectable() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut text = super::TextSystem::new(&ctx.device);

        let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
            .iter()
            .map(|b| b.to_vec())
            .collect();
        let added = text.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load");

        let content = "hello world";
        let style = TextStyle {
            font: fret_core::FontId::monospace(),
            size: Px(16.0),
            ..Default::default()
        };

        let single_line_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };
        let (single_blob, _metrics) = text.prepare(content, &style, single_line_constraints);
        let x_space_end = text
            .caret_x(single_blob, 6)
            .expect("caret_x at end of space");
        let x_w_end = text.caret_x(single_blob, 7).expect("caret_x after 'w'");
        assert!(
            x_w_end.0 > x_space_end.0 + 0.1,
            "expected the 'w' to advance beyond the trailing space"
        );

        // Force a soft wrap at the boundary between the space and the next word. This keeps the
        // space as a trailing character at the visual end of the first line.
        let max_width = Px((x_space_end.0 + x_w_end.0) * 0.5);
        let wrapped_constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };
        let (blob, _metrics) = text.prepare(content, &style, wrapped_constraints);
        let blob_ref = text.blob(blob).expect("wrapped blob");
        assert!(blob_ref.shape.lines.len() >= 2, "expected the text to wrap");

        let first = &blob_ref.shape.lines[0];
        assert!(
            first.end >= 6,
            "expected the first visual line to include the trailing space (end={})",
            first.end
        );

        let caret_after_o = text
            .caret_rect(blob, 5, CaretAffinity::Downstream)
            .expect("caret rect after 'o'");
        let caret_after_space = text
            .caret_rect(blob, 6, CaretAffinity::Upstream)
            .expect("caret rect after space (upstream)");
        assert!(
            caret_after_space.origin.x.0 > caret_after_o.origin.x.0 + 0.1,
            "expected the trailing space to have positive width in caret geometry"
        );

        let mut rects = Vec::new();
        text.selection_rects(blob, (5, 6), &mut rects)
            .expect("selection rects");
        assert_eq!(rects.len(), 1);
        assert!(
            rects[0].size.width.0 > 0.1,
            "expected a non-empty selection rect for the trailing space"
        );

        text.release(single_blob);
        text.release(blob);
    }

    #[test]
    fn ellipsis_overflow_truncates_single_line_layout() {
        let text = "This is a long line that should truncate";
        let constraints = TextConstraints {
            max_width: Some(Px(80.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            scale_factor: 1.0,
        };

        let mut shaper = crate::text::parley_shaper::ParleyShaper::new();
        let base = TextStyle::default();
        let wrapped = crate::text::wrapper::wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text, &base),
            constraints,
        );

        assert_eq!(wrapped.lines.len(), 1);
        assert!(wrapped.kept_end < text.len());
        assert!(wrapped.lines[0].width <= 80.0 + 0.5);
    }

    #[test]
    fn ellipsis_truncation_hit_test_maps_ellipsis_region_to_kept_end() {
        let text = "This is a long line that should truncate";
        let constraints = TextConstraints {
            max_width: Some(Px(80.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            scale_factor: 1.0,
        };

        let mut shaper = crate::text::parley_shaper::ParleyShaper::new();
        let base = TextStyle::default();
        let wrapped = crate::text::wrapper::wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text, &base),
            constraints,
        );

        assert_eq!(wrapped.lines.len(), 1);
        let kept_end = wrapped.kept_end;
        assert!(kept_end < text.len());

        let line_layout = &wrapped.lines[0];
        assert!(
            line_layout
                .clusters
                .iter()
                .any(|c| c.text_range == (kept_end..kept_end)),
            "expected a synthetic zero-length cluster at kept_end for ellipsis mapping"
        );

        let slice = &text[..kept_end];
        let caret_stops = super::caret_stops_for_slice(
            slice,
            0,
            &line_layout.clusters,
            line_layout.width,
            1.0,
            kept_end,
        );
        let line = super::TextLine {
            start: 0,
            end: kept_end,
            width: Px(line_layout.width),
            y_top: Px(0.0),
            y_baseline: Px(0.0),
            height: Px(10.0),
            caret_stops,
        };

        let x = Px((line_layout.width - 1.0).max(0.0));
        let hit =
            super::hit_test_point_from_lines(&[line], Point::new(x, Px(5.0))).expect("hit test");
        assert_eq!(hit.index, kept_end);
    }

    #[test]
    fn emoji_sequences_use_color_quads_when_color_font_is_available() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut text = super::TextSystem::new(&ctx.device);

        let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
            .iter()
            .chain(fret_fonts::emoji_fonts().iter())
            .map(|b| b.to_vec())
            .collect();
        let added = text.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load");

        let style = TextStyle {
            font: fret_core::FontId::family("Noto Color Emoji"),
            size: Px(32.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let cases = [
            ("\u{1F600}", "single emoji"),
            ("\u{2708}\u{FE0F}", "vs16 emoji presentation"),
            ("1\u{FE0F}\u{20E3}", "keycap sequence"),
            ("\u{1F1FA}\u{1F1F8}", "flag sequence"),
            (
                "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
                "zwj family sequence",
            ),
            (
                "\u{1F3F3}\u{FE0F}\u{200D}\u{1F308}",
                "zwj rainbow flag sequence",
            ),
        ];

        for (text_str, label) in cases {
            let (blob_id, _metrics) = text.prepare(text_str, &style, constraints);
            let blob = text.blob(blob_id).expect("text blob");

            let mut color_glyphs: Vec<super::GlyphKey> = Vec::new();
            for g in blob.shape.glyphs.as_ref() {
                if matches!(g.kind(), super::GlyphQuadKind::Color) {
                    color_glyphs.push(g.key);
                }
            }

            assert!(
                !color_glyphs.is_empty(),
                "expected at least one color glyph quad for {label} when Noto Color Emoji is present"
            );

            let epoch = 1;
            for key in color_glyphs {
                text.ensure_glyph_in_atlas(key, epoch);
                assert!(
                    text.color_atlas.get(key, epoch).is_some(),
                    "expected color glyph to be present in color atlas after ensure ({label})"
                );
            }
        }
    }

    #[test]
    fn cjk_glyphs_populate_mask_or_subpixel_atlas_when_cjk_lite_font_is_available() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut text = super::TextSystem::new(&ctx.device);

        let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
            .iter()
            .chain(fret_fonts::cjk_lite_fonts().iter())
            .map(|b| b.to_vec())
            .collect();
        let added = text.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load");

        let family = "Noto Sans CJK SC";
        assert!(
            text.all_font_names()
                .iter()
                .any(|n| n.eq_ignore_ascii_case(family)),
            "expected {family} to be present after loading cjk-lite fonts"
        );

        let style = TextStyle {
            font: fret_core::FontId::family(family),
            size: Px(24.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: Some(Px(360.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let cases = [
            ("你好，世界！", "basic"),
            ("这是一段用于验证换行与标点处理的文本。", "wrapping"),
            ("数字 12345 与符号（）《》“”……", "punctuation"),
        ];

        for (text_str, label) in cases {
            let (blob_id, _metrics) = text.prepare(text_str, &style, constraints);
            let blob = text.blob(blob_id).expect("text blob");

            let glyphs = blob.shape.glyphs.as_ref();
            assert!(
                !glyphs.is_empty(),
                "expected shaped glyphs for CJK case {label}"
            );

            let mut non_color: Vec<super::GlyphKey> = Vec::new();
            for g in glyphs {
                match g.kind() {
                    super::GlyphQuadKind::Mask | super::GlyphQuadKind::Subpixel => {
                        non_color.push(g.key);
                    }
                    super::GlyphQuadKind::Color => {}
                }
            }

            assert!(
                !non_color.is_empty(),
                "expected at least one mask/subpixel glyph for CJK case {label}"
            );

            let epoch = 1;
            for key in non_color {
                text.ensure_glyph_in_atlas(key, epoch);
                match key.kind {
                    super::GlyphQuadKind::Mask => assert!(
                        text.mask_atlas.get(key, epoch).is_some(),
                        "expected mask glyph to be present in mask atlas after ensure ({label})"
                    ),
                    super::GlyphQuadKind::Subpixel => assert!(
                        text.subpixel_atlas.get(key, epoch).is_some(),
                        "expected subpixel glyph to be present in subpixel atlas after ensure ({label})"
                    ),
                    super::GlyphQuadKind::Color => {}
                }
            }
        }
    }

    #[test]
    fn cjk_fallback_uses_cjk_lite_font_without_explicit_family_when_system_fonts_are_absent() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut text = super::TextSystem::new(&ctx.device);

        // Simulate a Web/WASM-like environment: no system font discovery and only bundled fonts.
        text.font_system = cosmic_text::FontSystem::new_with_locale_and_db_and_fallback(
            "en-US".to_string(),
            cosmic_text::fontdb::Database::new(),
            super::FretFallback,
        );
        text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
        text.font_db_revision = 0;
        text.font_stack_key = super::font_stack_cache_key(
            text.font_system.locale(),
            text.font_system.db(),
            text.font_db_revision,
            &text.common_fallback_config,
        );

        let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
            .iter()
            .chain(fret_fonts::cjk_lite_fonts().iter())
            .map(|b| b.to_vec())
            .collect();
        let added = text.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load");

        let family_inter = "Inter";
        assert!(
            text.all_font_names()
                .iter()
                .any(|n| n.eq_ignore_ascii_case(family_inter)),
            "expected {family_inter} to be present after loading bootstrap fonts"
        );

        let family_cjk = "Noto Sans CJK SC";
        assert!(
            text.all_font_names()
                .iter()
                .any(|n| n.eq_ignore_ascii_case(family_cjk)),
            "expected {family_cjk} to be present after loading cjk-lite fonts"
        );

        let config = fret_core::TextFontFamilyConfig {
            ui_sans: vec![family_inter.to_string()],
            ..Default::default()
        };
        let _ = text.set_font_families(&config);

        let noto_blob_id = super::stable_font_blob_id(fret_fonts::cjk_lite_fonts()[0]);

        let style = TextStyle {
            font: fret_core::FontId::ui(),
            size: Px(24.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let (blob_id, _metrics) = text.prepare("你", &style, constraints);
        let glyph_keys: Vec<super::GlyphKey> = {
            let blob = text.blob(blob_id).expect("text blob");
            blob.shape.glyphs.iter().map(|g| g.key).collect()
        };

        assert!(!glyph_keys.is_empty(), "expected shaped glyphs for CJK");

        let used_cjk_lite = glyph_keys.iter().any(|k| k.font.blob_id == noto_blob_id);
        assert!(
            used_cjk_lite,
            "expected cjk-lite font to be selected for CJK glyphs under the UI sans stack when system fonts are absent"
        );

        let epoch = 1;
        for key in glyph_keys {
            if key.font.blob_id != noto_blob_id {
                continue;
            }

            text.ensure_glyph_in_atlas(key, epoch);
            match key.kind {
                super::GlyphQuadKind::Mask => assert!(
                    text.mask_atlas.get(key, epoch).is_some(),
                    "expected ensured CJK glyph to be present in the mask atlas"
                ),
                super::GlyphQuadKind::Subpixel => assert!(
                    text.subpixel_atlas.get(key, epoch).is_some(),
                    "expected ensured CJK glyph to be present in the subpixel atlas"
                ),
                super::GlyphQuadKind::Color => {}
            }
        }
    }

    #[test]
    fn emoji_fallback_uses_bundled_color_font_without_explicit_family_when_system_fonts_are_absent()
    {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut text = super::TextSystem::new(&ctx.device);

        // Simulate a Web/WASM-like environment: no system font discovery and only bundled fonts.
        text.font_system = cosmic_text::FontSystem::new_with_locale_and_db_and_fallback(
            "en-US".to_string(),
            cosmic_text::fontdb::Database::new(),
            super::FretFallback,
        );
        text.parley_shaper = crate::text::parley_shaper::ParleyShaper::new_without_system_fonts();
        text.font_db_revision = 0;
        text.font_stack_key = super::font_stack_cache_key(
            text.font_system.locale(),
            text.font_system.db(),
            text.font_db_revision,
            &text.common_fallback_config,
        );

        let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
            .iter()
            .chain(fret_fonts::emoji_fonts().iter())
            .map(|b| b.to_vec())
            .collect();
        let added = text.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load");

        let family_inter = "Inter";
        assert!(
            text.all_font_names()
                .iter()
                .any(|n| n.eq_ignore_ascii_case(family_inter)),
            "expected {family_inter} to be present after loading bootstrap fonts"
        );

        let family_emoji = "Noto Color Emoji";
        assert!(
            text.all_font_names()
                .iter()
                .any(|n| n.eq_ignore_ascii_case(family_emoji)),
            "expected {family_emoji} to be present after loading emoji fonts"
        );

        let config = fret_core::TextFontFamilyConfig {
            ui_sans: vec![family_inter.to_string()],
            ..Default::default()
        };
        let _ = text.set_font_families(&config);

        let emoji_blob_id = super::stable_font_blob_id(fret_fonts::emoji_fonts()[0]);

        let style = TextStyle {
            font: fret_core::FontId::ui(),
            size: Px(32.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let cases = [
            ("\u{1F600}", "single emoji"),
            ("\u{2708}\u{FE0F}", "vs16 emoji presentation"),
            ("1\u{FE0F}\u{20E3}", "keycap sequence"),
            ("\u{1F1FA}\u{1F1F8}", "flag sequence"),
            (
                "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
                "zwj family sequence",
            ),
        ];

        for (text_str, label) in cases {
            let (blob_id, _metrics) = text.prepare(text_str, &style, constraints);

            let glyph_keys: Vec<super::GlyphKey> = {
                let blob = text.blob(blob_id).expect("text blob");
                blob.shape.glyphs.iter().map(|g| g.key).collect()
            };
            assert!(
                !glyph_keys.is_empty(),
                "expected shaped glyphs for emoji case {label}"
            );

            let emoji_keys: Vec<super::GlyphKey> = glyph_keys
                .iter()
                .copied()
                .filter(|k| k.font.blob_id == emoji_blob_id)
                .collect();
            assert!(
                !emoji_keys.is_empty(),
                "expected bundled emoji font to be selected for {label} under the UI sans stack when system fonts are absent"
            );

            let color_keys: Vec<super::GlyphKey> = emoji_keys
                .iter()
                .copied()
                .filter(|k| k.kind == super::GlyphQuadKind::Color)
                .collect();
            assert!(
                !color_keys.is_empty(),
                "expected at least one color emoji glyph quad for {label}"
            );

            let epoch = 1;
            for key in color_keys {
                text.ensure_glyph_in_atlas(key, epoch);
                assert!(
                    text.color_atlas.get(key, epoch).is_some(),
                    "expected ensured emoji glyph to be present in the color atlas ({label})"
                );
            }
        }
    }

    #[test]
    fn span_fingerprints_split_shaping_and_paint() {
        let constraints = TextConstraints {
            max_width: Some(Px(200.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };
        let base = TextStyle::default();
        let text = "hello";

        let mut spans_a = vec![TextSpan {
            len: text.len(),
            shaping: Default::default(),
            paint: Default::default(),
        }];
        spans_a[0].paint.fg = Some(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });
        let mut spans_b = spans_a.clone();
        spans_b[0].paint.fg = Some(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        });

        assert_eq!(
            spans_shaping_fingerprint(&spans_a),
            spans_shaping_fingerprint(&spans_b)
        );
        assert_ne!(
            spans_paint_fingerprint(&spans_a),
            spans_paint_fingerprint(&spans_b)
        );

        let mut spans_c = spans_a.clone();
        spans_c[0].paint.underline = Some(UnderlineStyle {
            color: None,
            style: DecorationLineStyle::Solid,
        });
        assert_ne!(
            spans_paint_fingerprint(&spans_a),
            spans_paint_fingerprint(&spans_c)
        );

        let rich_a = fret_core::AttributedText::new(
            Arc::<str>::from(text),
            Arc::<[TextSpan]>::from(spans_a),
        );
        let rich_b = fret_core::AttributedText::new(
            Arc::<str>::from(text),
            Arc::<[TextSpan]>::from(spans_b),
        );

        let k_a = TextBlobKey::new_attributed(&rich_a, &base, constraints, 7);
        let k_b = TextBlobKey::new_attributed(&rich_b, &base, constraints, 7);
        assert_ne!(k_a, k_b, "paint changes should affect blob cache keys");
        assert_eq!(
            TextShapeKey::from_blob_key(&k_a),
            TextShapeKey::from_blob_key(&k_b),
            "paint changes must not affect shape cache keys"
        );
    }
}
