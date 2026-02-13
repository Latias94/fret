# ADR 0152: Polychrome Glyphs (Color Emoji) and the Text Rendering Pipeline (v1)

- Status: Proposed
- Date: 2026-01-14

## Context

Fret targets editor-grade UI and must render emoji and other polychrome glyphs correctly across:

- native (Windows/macOS/Linux),
- Web/WASM (no system font discovery; bundled fonts are the baseline),
- mixed-script content (Latin + CJK + RTL + emoji sequences).

Unlike monochrome glyphs, emoji and polychrome glyphs may be sourced from:

- COLR/CPAL color outlines,
- bitmap strikes (CBDT/CBLC),
- embedded PNG strikes (sbix),
- or other color-capable font formats.

We already have the foundational pieces:

- A text pipeline and atlas strategy (ADR 0029).
- A text system v2 direction based on Parley shaping and attributed spans (ADR 0142).
- A font bootstrap + invalidation contract with `TextFontStackKey` (ADR 0147).
- A manual emoji conformance harness (demo) to validate VS16/ZWJ/flags/keycaps and fallback behavior.

What is not locked yet is the contract for how color glyphs flow through the renderer pipeline
(cache keys, atlas storage, and draw submission), so that we do not build feature surfaces on top of
undefined behavior.

## Goals

1) Define a deterministic **glyph content classification** contract (mono vs subpixel vs color).
2) Define atlas and upload ownership for color glyphs (avoid ad-hoc “special cases” in widgets).
3) Ensure cache keys are correct: font/config changes must invalidate both layout and raster caches.
4) Provide predictable behavior on Web/WASM (bundled fonts + curated defaults).
5) Keep the UI boundary stable: UI provides `TextInput`, renderer produces `TextBlobId` + metrics.

## Non-goals

- Full OpenType color feature exposure at the UI layer (feature tags, palette selection UI).
- Emoji fallback policy that depends on OS-specific heuristics.
- Perfect parity with native OS text renderers (ClearType, platform hinting quirks).

## Decision

### 1) Classify rendered glyph quads into three content kinds

Prepared text runs in the renderer classify each glyph quad as one of:

- `Mask`: 1-channel glyph coverage mask (monochrome).
- `Subpixel`: 4-channel subpixel mask (RGB coverage + alpha semantics).
- `Color`: 4-channel RGBA bitmap (polychrome glyph).

This classification is derived from the font rasterization output (not from Unicode ranges), so it
works for any polychrome glyphs (emoji, icons, etc.).

### 2) Use dedicated atlases per content kind

The renderer owns and maintains separate atlases:

- `mask_atlas` for `Mask`,
- `subpixel_atlas` for `Subpixel`,
- `color_atlas` for `Color`.

This keeps sampling and shader paths explicit, and avoids mixing different texture formats or
channel semantics.

### 3) Rasterization strategy (priority order)

When rasterizing a glyph, the renderer attempts sources in this order:

1) Color outline (COLR/CPAL) if present,
2) Color bitmap strikes (CBDT/CBLC / sbix) if present,
3) Monochrome outline as fallback.

This ordering is applied per glyph, per font face, per size.

### 4) Cache key rules

Polychrome correctness depends on correct invalidation.

The following inputs must participate in cache keys that affect rasterization and layout reuse:

- `TextFontStackKey` (includes configured font family stacks + locale + font DB revision).
- font face identifier (stable within a `TextFontStackKey` epoch).
- glyph id + subpixel quantization parameters (if used).
- font size / scale factor.
- rasterization kind (Mask/Subpixel/Color) and the chosen source path (outline vs bitmap) if it
  changes the produced pixels.

Paint-only span attributes (foreground color, underline color, etc.) must **not** participate in
 shaping/layout keys (ADR 0142), but may participate in paint caching where applicable.

### 5) Emoji fallback policy (portable baseline)

We treat emoji fonts as a **fallback layer**.

- On Web/WASM, the curated default family lists should include a known emoji family as a last
  resort (e.g. `Noto Color Emoji`) when available.
- On native platforms, the curated defaults may include well-known OS emoji families to improve
  determinism (e.g. `Segoe UI Emoji`, `Apple Color Emoji`), but actual availability remains
  platform-dependent.

The authoritative “what families are in play” remains `TextFontFamilyConfig` + `TextFontStackKey`
(ADR 0147). No widget may bypass this by injecting hidden fallback stacks.

### 6) Conformance baseline

We keep an explicit conformance surface for polychrome correctness:

- A manual demo that exercises:
  - ZWJ sequences (family, professions),
  - VS16 behavior,
  - regional indicator flags,
  - keycaps,
  - skin tone modifiers,
  - mixed scripts.
- A follow-up automated snapshot strategy once rendering determinism is stable.

## Implementation notes (non-normative)

Current implementation already follows this direction in the renderer:

- Glyph content classification + dedicated atlases live in `crates/fret-render-wgpu/src/text/mod.rs`.
- The conformance harness is `apps/fret-examples/src/emoji_conformance_demo.rs`.

## Follow-ups

- Promote this ADR to `Accepted` once:
  - the cache-key rules are validated by a regression harness,
  - emoji conformance is verified on Windows + macOS + Web/WASM.

## References

- Text pipeline and atlas strategy: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- Text system v2 (Parley + spans): `docs/adr/0142-text-system-v2-parley-attributed-spans-and-quality-baseline.md`
- Font bootstrap + invalidation: `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
- Emoji conformance demo: `apps/fret-examples/src/emoji_conformance_demo.rs`
