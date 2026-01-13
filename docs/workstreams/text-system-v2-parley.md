# Text System v2 (Parley) — Workstream Roadmap & TODO Tracker

Status: Active (design locked by ADR 0157; implementation in progress on `feat/text-system-v2-parley`)

This document is a living, implementation-focused tracker. It is intentionally non-authoritative;
the normative contract is `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`.

## Scope (v2)

- Adopt **Parley** as the shaping engine (line/chunk shaping as the primitive).
- Introduce **attributed spans** with a strict separation between:
  - shaping-affecting attributes (participate in shaping/layout cache keys)
  - paint-only attributes (do not invalidate shaping/layout caches)
- Implement a renderer-owned **wrapper layer** (wrap + ellipsis/truncation) driving the shaper.
- Upgrade renderer text atlases to be **multi-page, budgeted, evictable** (no unbounded append-only growth).
- Lock a **quality baseline**: subpixel variants + shader gamma/contrast correction.
- Keep the stable UI boundary: `TextBlobId` + `TextMetrics` + geometry queries (ADR 0006/0045/0046).

Non-goals (v2):

- Per-span font size / per-span line height (kept uniform per layout for now).
- Full-document code-editor virtualization (separate higher-level layer).
- Rich clipboard formats (tracked by ADR 0110/0152).

## Milestones

### M0 — Contract landed (core API shape)

Exit criteria:

- `fret-core` exposes `TextInput` + `TextSpan` (shaping vs paint split) and clamps invariants.
- `TextService` has a single entry point for `TextInput`.
- `fret-ui` compiles with the new API (adaptation layer allowed, but no shaping leaks).

Status:

- Done in `feat/text-system-v2-parley` (`3bb0fc8`).

### M1 — Renderer text system v2 (Parley + wrapper + atlas)

Exit criteria:

- Renderer can render plain and attributed text end-to-end using Parley.
- Wrapping + `TextOverflow::Ellipsis` behaves deterministically (including caret mapping).
- Geometry queries match ADR 0045/0046 (caret rect, hit-test point, selection rects).
- Atlas is budgeted + evictable and does not grow forever in a long-running demo.

### M2 — Ecosystem migration (Markdown + code view)

Exit criteria:

- `ecosystem/fret-markdown` no longer falls back for `strikethrough`/`inline code` due to missing text primitives.
- `ecosystem/fret-code-view` uses spans for syntax highlighting under wrapping.
- Theme color changes do not trigger reshaping/re-wrapping (paint-only update path works).

### M3 — Quality baseline + conformance

Exit criteria:

- Subpixel strategy is implemented and included in raster/cache keys.
- Shader-side gamma/contrast correction is present and tuneable via settings.
- Add at least one conformance harness/demo snapshot for mixed-script + emoji + IME preedit.

## Acceptance Checklist (what “done” means)

### Correctness

- Geometry queries:
  - `caret_rect` and `hit_test_point` behave consistently across multiline wraps (affinity rules).
  - `selection_rects` are stable and do not “drift” across span boundaries.
- Ellipsis:
  - Truncation does not produce out-of-bounds indices.
  - Caret/hit-test around the truncation boundary is representable and deterministic.
- Span invariants:
  - invalid spans are handled deterministically (debug assert + release clamp strategy).

### Performance

- Theme-only changes (colors/decoration colors) do not trigger reshaping/re-wrapping.
- Text atlas remains within a configured budget under churn (no unbounded growth).
- Shaping is cached by stable keys and avoids repeated work across frames.

### Portability

- Font family overrides work (system UI defaults + configured candidates).
- Emoji / color glyphs follow the polychrome path and render correctly (including fallback).
  - If a platform cannot supply color glyph rasters, behavior degrades predictably.

### Observability

- Add a lightweight debug snapshot surface (counts for: blobs, atlas tiles/pages, evictions, uploads).

## TODO (checkboxes)

Legend:

- [ ] pending
- [x] done
- [~] in progress
- [!] blocked / needs decision

### A) `fret-core` contract changes

- [x] Replace `RichText`/`TextRun` with `TextInput`/`TextSpan` (migrate call sites).
- [x] Define `TextShapingStyle` vs `TextPaintStyle` types and serialization strategy (settings persistence).
- [x] Enforce span invariants:
  - sum(len) == text.len()
  - boundaries are UTF-8 char boundaries
- [x] Update `TextService` to:
  - `prepare(TextInput<'_>, TextConstraints) -> (TextBlobId, TextMetrics)`
  - `measure(TextInput<'_>, TextConstraints) -> TextMetrics`
- [x] Update ADR 0006 references if signatures change (keep the boundary semantics stable).

### B) `fret-render` text system v2 implementation

**B0 — Legacy backend hardening (pre-Parley)**
- [x] Split shaping vs paint cache keys (geometry reuse across theme-only changes).
- [x] Keep per-span paint as a palette (no paint baked into glyph quads / shaping outputs).

**B1 — Font + fallback + stable keys**
- [ ] Define a “font stack key” / revision model that invalidates caches on font DB changes.
- [ ] Implement family overrides (`TextFontFamilyConfig`) in the new system.

**B2 — Shaper (Parley)**
- [~] Integrate Parley shaping for a single line/chunk with attributed spans.
- [ ] Produce glyph/cluster mapping sufficient for:
  - caret stops
  - hit-testing
  - span-aware paint assignment

**B3 — Wrapper (wrap + truncation)**
- [~] Implement `wrap=None` using a wrapper layer that drives shaping on slices.
- [~] Implement `TextOverflow::Ellipsis` per ADR 0059 (single-line, wired for `TextWrap::None` + `Ellipsis`):
  - stable ellipsis glyph sequence
  - caret/selection mapping rules at truncation boundary
- [ ] Implement `wrap=Word` using the wrapper layer (multi-line slicing + caret mapping).

**B4 — Raster + atlas**
- [ ] Replace append-only “pen” atlas packing with multi-page free-rect packing (e.g. `etagere`).
- [ ] Split atlas by texture kind:
  - monochrome (coverage)
  - polychrome (emoji / color glyphs)
  - optional: subpixel (if multi-channel is adopted)
- [ ] Add eviction policy + explicit rebuild knob for debugging.

**B5 — Quality baseline**
- [ ] Implement `SUBPIXEL_VARIANTS_X = 4`, `Y = 1/4` (platform policy).
- [ ] Add shader gamma/contrast correction uniforms (GPUI-aligned).
- [ ] Ensure quality knobs participate in cache keys (layout/raster).

### C) `fret-ui` integration surface

- [ ] Update `TextProps` / `StyledTextProps` / `SelectableTextProps` to carry `TextInput` (or to build it deterministically).
- [ ] Ensure UI caches do not depend on theme revision for shaping (paint-only updates).
- [ ] Keep selection state stable and based on byte indices (ADR 0044/0045/0046).

### D) Ecosystem migrations

- [ ] `ecosystem/fret-markdown`: express `strikethrough`, `inline code` as spans (no fallback hacks).
- [ ] `ecosystem/fret-code-view`: represent highlighting as spans and support wrapping.

### E) Tests & conformance

- [ ] Unit tests: span invariant validation and clamping behavior.
- [ ] Unit tests: ellipsis truncation caret/hit-test mapping.
- [ ] Unit tests: wrap boundaries across span boundaries.
- [ ] Integration demo: mixed-script + emoji + IME preedit smoke (deterministic snapshot).

## Risks / Open Questions

- Ellipsis glyph choice: keep current `"…"` vs legacy placeholder; ensure fallback is stable across platforms.
- Parley cluster/index semantics: ensure we can map to UTF-8 byte offsets with correct clamping.
- Atlas eviction determinism: ensure eviction does not cause flicker without explicit rebuild strategy.

## Progress Log (append-only)

- 2026-01-13: ADR 0157 added (design locked), worktree created.
- 2026-01-13: M0 contract landed (commit `3bb0fc8`).
- 2026-01-13: M1 started: add Parley dependency + single-line shaper prototype in `crates/fret-render/src/text_v2/mod.rs`.
- 2026-01-13: M1.1: split shaping/paint caches in the current text backend (`TextShapeKey` + per-span palette; theme-only changes no longer force reshaping).
- 2026-01-13: M1.2: add `text_v2` wrapper prototype for `wrap=None + Ellipsis` with cluster-based hit-test mapping (unit tests only; not integrated yet).
- 2026-01-13: M1.3: wire Parley `wrap=None + Ellipsis` through `TextSystem::prepare_*` (renders via swash into the existing atlases; still missing fractional positioning + font config integration).
- 2026-01-13: M1.4: align Parley rasterization with cosmic-text subpixel binning + wire `add_fonts` and `set_font_families` into Parley fontique generics (reduces drift across backends).
