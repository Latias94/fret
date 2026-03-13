#[cfg(test)]
use fret_render_text::cache_keys::TextBlobKey;
#[cfg(test)]
use fret_render_text::cache_keys::TextShapeKey;

#[cfg(test)]
use fret_render_text::cache_keys::spans_paint_fingerprint;
pub use fret_render_text::decorations::{TextDecoration, TextDecorationKind};
pub(crate) use fret_render_text::effective_text_scale_factor;
use fret_render_text::fallback_policy::TextFallbackPolicyV1;
#[cfg(test)]
pub(crate) use fret_render_text::font_instance_key::FontFaceKey;
use fret_render_text::font_stack::GenericFamilyInjectionState;
use fret_render_text::font_trace::FontTraceState;
pub use fret_render_text::{
    FontCatalogEntryMetadata, SystemFontRescanResult, SystemFontRescanSeed,
};

mod atlas;
mod atlas_epoch;
mod atlas_runtime_state;
mod blob_state;
mod blobs;
mod bootstrap;
mod diagnostics;
mod face_cache;
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
use self::frame_perf::TextFramePerfState;
use self::layout_cache_state::TextLayoutCacheState;
use self::pin_state::TextPinState;
pub use self::quality::TextQualitySettings;
use self::quality::TextQualityState;
#[cfg(test)]
pub(crate) use self::types::subpixel_mask_to_alpha;
pub(crate) use self::types::{DebugGlyphAtlasLookup, TextAtlasPerfSnapshot, TextFontFaceUsage};
pub use self::types::{
    GlyphInstance, GlyphQuadKind, TextBlob, TextFontFamilyConfig, TextLine, TextShape,
};

pub(crate) mod parley_shaper {
    pub use fret_render_text::parley_shaper::*;
}

pub(crate) mod wrapper {
    pub use fret_render_text::wrapper::*;
}

pub struct TextSystem {
    parley_shaper: crate::text::parley_shaper::ParleyShaper,
    parley_scale: parley::swash::scale::ScaleContext,
    font_stack_key: u64,
    font_db_revision: u64,
    fallback_policy: TextFallbackPolicyV1,
    quality: TextQualityState,
    generic_injections: GenericFamilyInjectionState,

    blob_state: TextBlobState,
    layout_cache: TextLayoutCacheState,

    atlas_runtime: TextAtlasRuntimeState,

    pin_state: TextPinState,
    face_cache: TextFaceCacheState,

    frame_perf: TextFramePerfState,

    atlas_epoch: TextAtlasEpochState,

    font_trace: FontTraceState,
}
#[cfg(test)]
mod tests;
