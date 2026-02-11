# Text System Audit: Fret vs Zed (GPUI)

This document compares Fret’s current text stack with Zed/GPUI’s text system implementation
(`repo-ref/zed/crates/gpui/src/text_system.rs`) and identifies refactor work needed for a future-proof
editor-grade text pipeline.

This is an audit / implementation guide, not a contract. Normative decisions belong in ADRs.

## Scope of this audit

- Text shaping + line/chunk layout
- Font catalog / fallback stack semantics
- Cache keys + invalidation rules
- Color emoji / polychrome glyph strategy
- Geometry queries (hit-test, caret, selection)

## Zed/GPUI snapshot (what matters)

GPUI’s text architecture separates concerns cleanly:

- `TextSystem` owns:
  - querying system font names (`all_font_names`)
  - adding fonts (user-provided bytes)
  - resolving fonts with a fallback stack (`resolve_font`)
  - caching font ids / metrics / raster bounds
- `PlatformTextSystem` is OS/backend-specific (DirectWrite/CoreText/Cosmic Text) and provides:
  - font resolution + glyph rasterization + metrics
  - (effectively) the font DB and “is emoji” classification
- Layout primitives are line-centric:
  - `LineLayout` contains shaped runs (`ShapedRun`) and glyphs (`ShapedGlyph`)
  - `LineWrapper` drives wrapping/truncation on top of shaped lines
- Color emoji is treated explicitly:
  - glyphs carry `is_emoji`
  - platform backends can use a dedicated “emoji source” path
  - Linux cosmic-text backend has a targeted workaround around variation selectors

### Practical takeaways from Zed

- A stable fallback stack is part of the “text system contract”, even if fonts differ by platform.
- Cache keys must include *all* font/fallback policy inputs, otherwise stale glyph/layout reuse is inevitable.
- Emoji/VS/ZWJ correctness needs explicit conformance coverage; “it works on my OS” is not enough.

## Fret snapshot (current state)

Fret already locks a strong cross-crate boundary (UI does not shape):

- `TextBlobId` + `TextMetrics` + geometry queries (ADR 0006 / ADR 0045 / ADR 0046)
- Renderer-owned atlases and a text pipeline strategy (ADR 0029)

Recent work also improved determinism around font bootstrapping and invalidation:

- Web/wasm bootstrap now seeds curated defaults and bumps `TextFontStackKey` on font/config mutation
  (ADR 0147, and the font bootstrap plumbing in `crates/fret-runtime` + `crates/fret-launch`).
- A dedicated emoji conformance demo exists (`apps/fret-examples/src/emoji_conformance_demo.rs`).
- A dedicated CJK conformance demo exists (`apps/fret-examples/src/cjk_conformance_demo.rs`) and can
  use an optional `cjk-lite` bundled font tier on Web/WASM.

However, the *implementation shape* is still mid-transition:

- There are multiple shaping/layout paths (legacy + v2 scaffolding).
- Rich text / attributed spans exist as a direction but are not fully “single-source-of-truth” yet.
- Color emoji is available as a wasm bundle, but the renderer pipeline is not yet locked as a v2 contract.

## Key mismatches / drift (things to fix)

### 1) “One true” shaping backend (no backend gates)

Goal: avoid a permanent split-brain text pipeline.

- Adopt Parley as the primary shaping engine for line/chunk shaping.
- Keep wrapping/truncation in a renderer-owned wrapper layer (so the UI boundary stays stable).
- Avoid feature-gated backends for the mainline (“no backend gate”).

Tracking: ADR 0142 + `docs/workstreams/text-system-v2-parley.md`.

### 2) Fallback stack and missing glyph semantics must be explicit

Zed hard-codes a fallback stack and always includes it in `all_font_names`.
Fret needs an equivalent explicit “fallback policy surface”:

- `TextFontFamilyConfig` + `TextFontStackKey` must be the only way to influence fallback stacks.
- Any change in config or font DB must bump `TextFontStackKey` and invalidate shaping/raster caches.
- Provide a deterministic, cross-platform “known good” default stack, with optional emoji layer.

Tracking: ADR 0147 + ADR 0142.

### 3) Color emoji / polychrome glyph pipeline needs a v2 contract

Zed treats emoji as special (and backends choose sources accordingly).
For Fret, the future-proof approach is:

- Define a **polychrome glyph** rendering path (RGBA atlas or separate render pass).
- Define how `TextBlob` carries “glyph color mode” (mono/subpixel vs RGBA) in prepared runs.
- Make VS16/ZWJ/flags/keycaps a first-class conformance surface (demo + future automated snapshots).

Tracking: new ADR recommended (P0).

### 4) Geometry queries must come from the same layout used for rendering

Fret’s geometry query ADRs are solid; the implementation must ensure:

- hit-test, caret, selection rectangles derive from the prepared layout that produced glyph quads
- ellipsis/truncation doesn’t create “unrepresentable” indices
- affinity rules remain consistent across wrapping and span boundaries

Tracking: ADR 0045 / ADR 0046 + v2 workstream.

## Recommended refactor plan (incremental, but “future-proof”)

P0 (contracts + harness first):

1) Confirm/adjust ADR 0142 to cover:
   - single Parley backend direction
   - cache key rules (what participates in shaping keys)
2) Add an ADR for polychrome glyphs / emoji pipeline.
3) Add/keep a conformance harness:
   - `emoji_conformance_demo` (manual)
   - later: snapshot-based regression tests once rendering determinism is stable

P0/P1 (implementation):

4) Build `TextSystem v2` around:
   - `TextInput` + `TextSpan` (shaping vs paint separation)
   - Parley shaping for a single line/chunk
   - a wrapper layer for wrap/truncate + geometry query derivation
5) Make font/fallback policy a single source of truth:
   - keep `TextFontStackKey` as the invalidation contract
   - unify font injection plumbing across native + web
6) Lock the atlas strategy:
   - budgeted, evictable pages
   - mono/subpixel and RGBA paths are explicit

## References

- Zed/GPUI text system: `repo-ref/zed/crates/gpui/src/text_system.rs`
- Zed Linux cosmic-text backend (emoji/VS quirks): `repo-ref/zed/crates/gpui/src/platform/linux/text_system.rs`
- Fret font bootstrap + invalidation: `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
- Fret text system boundary: `docs/adr/0006-text-system.md`
- Fret text pipeline strategy: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- Fret text system v2 direction: `docs/adr/0142-text-system-v2-parley-attributed-spans-and-quality-baseline.md`
