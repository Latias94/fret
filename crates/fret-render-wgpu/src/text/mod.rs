use fret_core::TextBlobId;
use slotmap::SlotMap;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use fret_render_text::cache_keys::{TextBlobKey, TextShapeKey};

#[cfg(test)]
use fret_render_text::cache_keys::spans_paint_fingerprint;
pub use fret_render_text::decorations::{TextDecoration, TextDecorationKind};
pub(crate) use fret_render_text::effective_text_scale_factor;
use fret_render_text::fallback_policy::TextFallbackPolicyV1;
use fret_render_text::font_instance_key::FontFaceKey;
use fret_render_text::font_stack::GenericFamilyInjectionState;
use fret_render_text::font_trace::FontTraceState;
use fret_render_text::measure::TextMeasureCaches;
pub use fret_render_text::{
    FontCatalogEntryMetadata, SystemFontRescanResult, SystemFontRescanSeed,
};

mod atlas;
mod blobs;
mod bootstrap;
mod diagnostics;
mod fonts;
mod frame_perf;
mod measure;
mod prepare;
mod quality;
mod queries;
mod types;

use self::atlas::{GlyphAtlas, GlyphKey};
use self::frame_perf::TextFramePerfState;
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

    blobs: SlotMap<TextBlobId, TextBlob>,
    blob_cache: HashMap<TextBlobKey, TextBlobId>,
    blob_key_by_id: HashMap<TextBlobId, TextBlobKey>,
    released_blob_lru: VecDeque<TextBlobId>,
    released_blob_set: HashSet<TextBlobId>,
    shape_cache: HashMap<TextShapeKey, Arc<TextShape>>,
    measure: TextMeasureCaches,

    mask_atlas: GlyphAtlas,
    color_atlas: GlyphAtlas,
    subpixel_atlas: GlyphAtlas,
    atlas_bind_group_layout: wgpu::BindGroupLayout,

    text_pin_mask: Vec<Vec<GlyphKey>>,
    text_pin_color: Vec<Vec<GlyphKey>>,
    text_pin_subpixel: Vec<Vec<GlyphKey>>,
    font_data_by_face: HashMap<(u64, u32), parley::FontData>,
    font_instance_coords_by_face: HashMap<FontFaceKey, Arc<[i16]>>,
    font_face_family_name_cache: HashMap<(u64, u32), String>,

    frame_perf: TextFramePerfState,

    glyph_atlas_epoch: u64,

    font_trace: FontTraceState,
}
#[cfg(test)]
mod tests;
