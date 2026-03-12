# Workstream: Text Shaping Surface v1 (OpenType Features + Cache Semantics)

Status: M0 implemented (contracts + Parley plumbing); M1 implemented (ecosystem policy adoption in `fret-code-editor` + `fret-code-view`).

This document is **non-normative**. Normative contracts live in ADRs (notably the v2 Parley text
system ADRs) and in the `fret-core` public API.

Related workstreams / references:

- Text system v2 tracker: `docs/workstreams/standalone/text-system-v2-parley.md`
- Font system audit/roadmap: `docs/workstreams/standalone/font-system-v1.md`
- Integration hazards (historical): `docs/workstreams/text-layout-integration-v1/text-layout-integration-v1.md`

## Problem Statement

Fret’s Parley-based text stack already supports:

- semantic `FontId` (generic families + explicit family),
- per-span **variable font axis** overrides,
- per-span weight/slant/letter-spacing overrides,
- strict shaping vs paint separation at the span level.

However, for editor-grade text, we still need a first-class shaping surface for:

1) **OpenType feature toggles** (e.g. `calt`, `liga`, `ss01`…),
2) a stable and auditable mapping from `fret-core` shaping attributes to Parley style properties,
3) explicit cache-keying rules so that feature changes never reuse stale shaping / glyph raster
   outputs.

Historically, we did not expose OpenType features in `fret-core::TextShapingStyle`, even though
Parley supports `StyleProperty::FontFeatures`. This gap is now closed for M0 (contracts + plumbing);
remaining work is ecosystem policy adoption and productization.

## Goals

1) Add a portable, serializable, best-effort OpenType features representation to `fret-core`.
2) Plumb the representation into Parley shaping (`StyleProperty::FontFeatures`) deterministically.
3) Ensure feature settings:
   - participate in shaping/layout cache keys,
   - participate in any downstream raster cache keys **only indirectly** via the shaped output
     identity (glyph ids, positions, and per-glyph font identity).
4) Provide focused tests that prevent regressions in:
   - feature application,
   - cache key correctness,
   - span boundary behavior.

## Non-goals (v1)

- A full end-user settings UI for feature toggles (component/ecosystem concern).
- A “CSS font-feature-settings” parser exposed as a stable public surface (can be staged later).
- Per-span font size and per-span line-height (out of scope for this workstream; tracked elsewhere).

## Current State (Evidence)

- Shaping attributes in `fret-core`:
  - `crates/fret-core/src/text/mod.rs` (`TextShapingStyle`, `TextFontAxisSetting`)
- Parley plumbing:
  - `crates/fret-render-text/src/parley_shaper.rs` (`shaping_properties_for_span`)
  - sets: `FontStack`, `FontVariations`, `FontWeight`, `FontStyle`, `LetterSpacing`, `FontFeatures`
- Wrapper + cache keys:
  - `crates/fret-render-wgpu/src/text/mod.rs` (`TextBlobKey`, `TextShapeKey`, measure caches)
  - spans participate in shaping key via a “shaping-only fingerprint” (do not regress this).
  - features participate in the shaping fingerprint and blob keys:
    - `crates/fret-render-wgpu/src/text/mod.rs` (`features_shaping_fingerprint`)

## Proposed API Surface (fret-core)

### New type: `TextFontFeatureSetting`

Add a best-effort, portable representation of OpenType features:

- Tag: 4-byte OpenType feature tag (e.g. `"calt"`, `"liga"`, `"ss01"`).
- Value: `u32` (OpenType feature “value” a.k.a. setting). Conventionally:
  - `0` = off,
  - `1` = on,
  - but higher values are allowed for features like `cvXX` / alternates in some fonts.

Suggested struct (shape, naming subject to repo conventions):

- `TextFontFeatureSetting { tag: String, value: u32 }`

Validation rules (best-effort):

- `tag.trim()` must be exactly 4 ASCII bytes.
- invalid tags are ignored by shaping backends (do not panic).
- keep the list order stable, but canonicalize for keying by sorting by tag (and then value).

### Extend `TextShapingStyle`

Add:

- `features: Vec<TextFontFeatureSetting>`

This mirrors the existing `axes: Vec<TextFontAxisSetting>` design:

- “advanced surface”
- “best-effort”
- ignored if the resolved face does not support a tag.

## Parley Mapping (fret-render-wgpu)

In `shaping_properties_for_span`:

- Convert `TextFontFeatureSetting` into Parley `FontFeature` (internally a swash setting).
- Emit:
  - `StyleProperty::FontFeatures(FontSettings::List(...))`

Canonicalization:

- Filter invalid tags.
- Coalesce duplicates by tag, keeping the **last** occurrence (stable “last writer wins” semantics),
  then produce a sorted list for deterministic hashing.

Cache key requirements:

- The span shaping fingerprint must include the canonicalized feature list.
- Any change to the canonicalized feature list must bump:
  - shape cache keys,
  - measurement cache keys that depend on shaping,
  - text blob cache keys (since shaping changes).

## Diagnostics (Explainability)

To keep feature-related regressions debuggable:

- Add (or extend) a renderer text diagnostics snapshot to include:
  - effective per-span feature list (canonicalized),
  - and/or a compact hash that is already in the shaping key.

Do not require consumers to decode Parley internals.

## Testing Strategy

### Unit tests (minimum set)

1) Feature list canonicalization:
   - invalid tags ignored,
   - duplicates coalesced,
   - stable ordering for hashing.
2) Cache key correctness:
   - toggling `liga` changes the shaping key (and thus the prepared output identity).
3) Behavior smoke:
   - pick a font that demonstrably changes shaping with a feature toggle.
   - if bundled fonts do not provide an obvious case, add a test-only font fixture under the
     existing font test patterns (do not bloat WASM payload).

### Where tests should live

- `crates/fret-render-text/src/parley_shaper.rs` (mapping + keying tests)
- `crates/fret-render-wgpu/src/text/mod.rs` (end-to-end `prepare_*` cache behavior tests)

## Risks / Pitfalls

- Feature tags are often font-specific; tests must be robust across platforms and font sets.
- Feature toggles can change glyph selection, which changes raster outputs (expected) — ensure keys
  invalidate correctly and do not reuse cached glyphs incorrectly.
- Features may interact with the “ligature breaking” hack used on some platforms; document any
  necessary constraints.

## Milestones (High-Level)

- M0: Contracts + plumbing
  - Add `TextFontFeatureSetting` and `TextShapingStyle.features`.
  - Plumb to Parley as `StyleProperty::FontFeatures`.
  - Ensure cache keys include features deterministically.
  - Add unit tests for canonicalization + keying.
- M1: Editor-grade adoption
  - `ecosystem/fret-code-view` / `ecosystem/fret-code-editor` uses features for ligature policy
    (e.g. disable `liga`/`calt` in code, or provide a toggle).
  - Add a small conformance page in UI gallery (optional) to visualize feature toggles.
- M2: Settings surface (optional, ecosystem)
  - Add a component-layer policy for “code font features” vs “UI font features”.

For a detailed, checklist-driven plan, see:

- `docs/workstreams/text-shaping-surface-v1/text-shaping-surface-v1-milestones.md`
- `docs/workstreams/text-shaping-surface-v1/text-shaping-surface-v1-todo.md`

## Implementation notes (M0)

Evidence anchors:

- `TextFontFeatureSetting` + `TextShapingStyle.features`:
  - `crates/fret-core/src/text/mod.rs`
- Parley mapping and canonicalization:
  - `crates/fret-render-text/src/parley_shaper.rs` (`font_features_for_settings`)
- Shaping key participation:
  - `crates/fret-render-wgpu/src/text/mod.rs` (`features_shaping_fingerprint`)
