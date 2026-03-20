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
mod atlas_epoch;
mod atlas_flow;
mod atlas_runtime_state;
mod blob_state;
mod blobs;
mod bootstrap;
mod diagnostics;
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
use self::atlas_epoch::TextAtlasEpochState;
use self::atlas_runtime_state::TextAtlasRuntimeState;
use self::blob_state::TextBlobState;
use self::face_cache::TextFaceCacheState;
use self::font_runtime_state::TextFontRuntimeState;
use self::frame_perf::TextFramePerfState;
use self::layout_cache_state::TextLayoutCacheState;
use self::pin_state::TextPinState;
pub use self::quality::TextQualitySettings;
use self::quality::TextQualityState;
pub use self::types::TextFontFamilyConfig;
#[cfg(test)]
pub(crate) use self::types::subpixel_mask_to_alpha;
pub(crate) use self::types::{DebugGlyphAtlasLookup, TextAtlasPerfSnapshot, TextFontFaceUsage};
pub(crate) use self::types::{GlyphInstance, GlyphQuadKind, TextLine};
use self::types::{TextBlob, TextShape};

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

    atlas_epoch: TextAtlasEpochState,
}
#[cfg(test)]
mod tests;
