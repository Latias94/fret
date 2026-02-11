# ADR 0145: Unified Glyph Key and Subpixel Rendering Policy

- Status: Proposed
- Date: 2026-01-13

## Context

ADR 0142 locks the direction for text shaping/layout (Parley + attributed spans) and explicitly states that the mainline text system must not depend on backend feature gates. ADR 0143 then locks the cache boundary between:

- layout/shaping results (stable, cacheable, no atlas UVs), and
- glyph atlas residency (budgeted, evictable, frame-driven).

The current implementation still carries a “backend-shaped” glyph identity:

- `cosmic_text::CacheKey` for the cosmic path (includes internal flags and its own subpixel binning),
- a separate `ParleyGlyphKey` for the Parley path (font blob id + index + glyph id + size + bins).

This has two long-term problems:

1) **Key fragmentation**: the renderer cannot treat glyph rasterization uniformly (pinning, eviction, atlas lookup, cache invalidation, metrics/raster bounds caching) because the key type is backend-specific.
2) **Quality drift**: subpixel mask handling is currently degraded to an alpha mask in some cases, which is not future-proof for editor-grade text under non-integer DPI scaling and small font sizes.

Zed/GPUI’s reference decomposition (in `repo-ref/zed`) uses:

- a single `RenderGlyphParams` key that includes font identity, glyph id, font size, subpixel variant, and rendering mode flags,
- platform policies for subpixel variants (e.g. `SUBPIXEL_VARIANTS_X/Y`),
- an atlas keyed by those parameters, independent of the layout object.

Fret needs the same unification to safely remove backend gates and converge the renderer on a single shaping/layout backend (Parley) while keeping glyph rasterization and atlas residency stable and cacheable.

## Goals

1) Define a single, renderer-owned `GlyphKey` type that is stable across shaping backends.
2) Lock a subpixel rendering policy that can scale to editor-grade quality (gamma/contrast correction, deterministic variants).
3) Ensure cache keys (layout cache, residency cache, scene encoding cache) include all policy knobs needed for correctness.
4) Make the text stack converge to **one** shaping/layout backend (Parley) without conditional compilation or runtime “backend gates”.
5) Keep the public UI boundary unchanged (prepare + geometry queries + `TextBlobId`).

## Decision

### 1) Introduce a unified renderer glyph key

Define a renderer-level key that identifies a rasterized glyph variant unambiguously:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font: FontFaceKey,
    pub glyph_id: u32,
    pub size_bits: u32,
    pub subpixel: SubpixelVariant,
    pub kind: GlyphKind,
    pub flags: GlyphFlags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontFaceKey {
    /// Stable ID for a specific font blob/face (owned by the renderer font registry).
    pub blob_id: u64,
    /// Face index within the font blob (TTC / font collections).
    pub face_index: u32,
    /// Optional: variation coordinates fingerprint (for variable fonts).
    pub variation_key: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubpixelVariant {
    pub x: u8,
    pub y: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlyphKind {
    Mask,
    Color,
    Subpixel,
}
```

Notes:

- `FontFaceKey` must remain valid even if the platform font database changes ordering. It is owned by a renderer “font registry” that maps to raw font bytes.
- `variation_key` is reserved for variable font support; it can be `0` until the axis model is implemented.
- `GlyphKind` is a *rendering output kind*, not a shaping attribute. It is determined by rasterization (outline, color outline, bitmap strike, etc.).

### 2) Layout outputs only `GlyphInstance { rect, key, paint_span }`

Layout/shaping outputs must not embed atlas placement. It returns per-glyph instances:

- `rect`: the logical-space quad (relative to the baseline origin),
- `key`: the unified `GlyphKey`,
- `paint_span`: optional paint-only span slot for per-glyph color.

The layout algorithm must also be responsible for **deterministic subpixel variant selection** and for snapping the glyph placement to match the rasterized variant. This keeps raster placement, atlas keying, and quad positions aligned.

### 3) Subpixel variants are a platform policy and must be part of cache keys

Define platform-dependent constants:

- `SUBPIXEL_VARIANTS_X` (default: `4`)
- `SUBPIXEL_VARIANTS_Y` (default: `1` on Windows/Linux, `4` on macOS; exact policy may evolve)

This policy must participate in:

- the shaping/layout cache key (because placement snapping depends on variants),
- the glyph residency cache key (because glyph raster content depends on variants).

### 4) Add a third atlas kind for subpixel glyphs

Extend the glyph atlas to support three texture kinds:

- `Mask` atlas: `R8Unorm`
- `Color` atlas: `Rgba8UnormSrgb`
- `Subpixel` atlas: `Rgba8Unorm` (linear sampling; shader performs the correct reconstruction)

Shader-side reconstruction and gamma/contrast correction are part of the v2 quality baseline (tracked separately; see ADR 0142 “Quality baseline”).

### 5) Converge shaping/layout on Parley (no backend gate)

With a unified `GlyphKey`, the renderer no longer needs backend-specific keying or behavior gates. The mainline text system uses Parley for shaping/layout and uses a renderer-owned font registry + rasterizer (Swash-based) to produce glyph rasters for the unified key.

Legacy shaping backends, if kept temporarily, must map their output into the unified `GlyphKey` (rather than owning the glyph identity).

## Consequences

### Benefits

- One glyph identity type for atlas storage, pinning, eviction, and scene-driven ensure.
- Cleaner migration path to Parley-only shaping/layout without feature gating.
- A locked subpixel policy surface that can support high-quality editor text.

### Costs / Risks

- Requires a renderer-owned “font registry” that can provide stable `FontFaceKey -> bytes`.
- Subpixel atlas support adds shader complexity and increases atlas memory footprint.
- Cache invalidation must be audited carefully when adding new policy knobs (gamma, hinting, variations, rendering mode).

## Alternatives Considered

1) Keep backend-specific keys (`CacheKey` vs `ParleyGlyphKey`) and unify only at the atlas layer.
   - Rejected: eviction/pinning/cache key correctness remains fragmented and hard to reason about.
2) Store UV/page back into layout outputs.
   - Rejected by ADR 0143 (prevents eviction independence and breaks scene encoding cache correctness).

## Implementation Notes / Migration Plan

1) Introduce `GlyphKey` and convert the renderer glyph atlas map to key off it.
2) Add a font registry surface that can return `(blob_id, face_index, bytes)` for any font used by shaping.
3) Make Parley shaping produce `GlyphKey` for every glyph instance, including deterministic subpixel variant selection.
4) Implement `GlyphKind::Subpixel` end-to-end (atlas + shader path) and include policy knobs in cache keys.
5) Delete the “backend gate” and remove the legacy shaping path once coverage matches.

