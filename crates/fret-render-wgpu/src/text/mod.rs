#[cfg(test)]
use fret_render_text::TextBlobKey;
#[cfg(test)]
use fret_render_text::TextShapeKey;

#[cfg(test)]
pub(crate) use fret_render_text::FontFaceKey;
use fret_render_text::ParleyShaper;
#[cfg(test)]
use fret_render_text::TextFallbackPolicyV1;
pub(crate) use fret_render_text::effective_text_scale_factor;
#[cfg(test)]
use fret_render_text::spans_paint_fingerprint;
pub use fret_render_text::{
    FontCatalogEntryMetadata, SystemFontRescanResult, SystemFontRescanSeed,
};
pub use fret_render_text::{TextDecoration, TextDecorationKind};

mod atlas;
mod atlas_flow;
mod atlas_runtime_state;
mod blob_state;
mod blobs;
mod bootstrap;
mod diagnostics;
#[cfg(not(target_arch = "wasm32"))]
mod diagnostics_debug;
mod face_cache;
mod font_runtime_state;
mod fonts;
mod frame_perf;
mod layout_cache_state;
mod measure;
mod pin_state;
mod prepare;
mod quality;
mod queries;
mod types;

#[cfg(test)]
use self::atlas::GlyphKey;
use self::atlas_runtime_state::TextAtlasRuntimeState;
use self::blob_state::TextBlobState;
pub(crate) use self::blobs::TextBlobRenderData;
pub(crate) use self::diagnostics::TextSceneResourceSnapshot;
use self::face_cache::TextFaceCacheState;
use self::font_runtime_state::TextFontRuntimeState;
use self::frame_perf::TextFramePerfState;
use self::layout_cache_state::TextLayoutCacheState;
pub(crate) use self::pin_state::TextFrameResidency;
use self::pin_state::TextPinState;
pub use self::quality::TextQualitySettings;
use self::quality::TextQualityState;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use self::types::DebugGlyphAtlasLookup;
pub use self::types::TextFontFamilyConfig;
pub(crate) use self::types::TextLine;
use self::types::{GlyphInstance, TextBlob, TextShape};
pub(crate) use self::types::{
    TextAtlasPerfSnapshot, TextFontFaceUsage, TextGlyphCluster, TextGlyphClusterBuilder,
    TextRenderGlyph, TextRenderGlyphKind,
};
use std::time::Duration;

pub struct TextSystem {
    parley_shaper: ParleyShaper,
    parley_scale: parley::swash::scale::ScaleContext,
    font_runtime: TextFontRuntimeState,
    quality: TextQualityState,

    blob_state: TextBlobState,
    layout_cache: TextLayoutCacheState,

    atlas_runtime: TextAtlasRuntimeState,

    pin_state: TextPinState,
    face_cache: TextFaceCacheState,

    frame_perf: TextFramePerfState,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TextPrepareScenePerf {
    pub(crate) collect_pin_keys: Duration,
    pub(crate) bucket_delta: Duration,
    pub(crate) prewarm: Duration,
    pub(crate) pin_bucket_update: Duration,
    pub(crate) flush_uploads: Duration,
    pub(crate) fast_scene_bucket_reused: bool,
    pub(crate) scene_text_blobs: u64,
    pub(crate) pinned_glyph_keys: u64,
    pub(crate) prewarm_glyph_keys: u64,
    pub(crate) retained_glyph_keys: u64,
    pub(crate) added_glyph_keys: u64,
    pub(crate) removed_glyph_keys: u64,
}
#[cfg(test)]
mod tests;
