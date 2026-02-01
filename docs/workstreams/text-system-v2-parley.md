# Text System v2 (Parley) — Workstream Roadmap & TODO Tracker

Status: Active (design proposed in ADR 0157; implementation in progress)

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

- Landed on `main` (follow-up work continues via the work packages below).

### Current Focus (recommended order)

This tracker spans multiple layers and can easily become too coarse-grained. The recommended
sequence below keeps milestones small enough to ship and validate incrementally.

P0 (cache correctness + font config):

- B1: font stack revision key + family overrides (cache invalidation contract).
- C1: make UI build `TextInput` deterministically (no "shaping leaks").

P1 (geometry correctness across spans):

- B2: cluster mapping sufficient for caret stops + hit-testing + span-aware paint. (done)
- E: unit tests for span boundaries + ellipsis mapping.

P2 (quality & ecosystem):

- B5: gamma/contrast correction + quality knobs in cache keys.
- D: migrate markdown/code-view to spans (no fallback hacks).

### Work Packages (granular, shippable)

The boxes below are the suggested "bite-sized" milestones for the remaining `[ ]` items.

#### WP1 — Font stack revision + family overrides (B1)

Exit criteria:

- A stable `font_stack_key` (or revision) invalidates both layout and raster caches when:
  - any font bytes are added/removed
  - family mapping changes (defaults or overrides)
- `TextFontFamilyConfig` overrides participate in the effective font stack deterministically.
- Evidence: `TextSystem` cache keys include `font_stack_key` (layout + raster).

#### WP2 — Cluster mapping for geometry queries (B2)

Exit criteria:

- Shaping outputs expose enough mapping to support:
  - caret stops (monotonic x mapping)
  - `hit_test_point` and `hit_test_x`
  - span-aware paint assignment (no "color drift" across span boundaries)
- Add focused tests for:
  - caret/hit-test around span boundary
  - wrap boundary across span boundary

#### WP3 — Quality knobs are cache-keyed (B5)

Exit criteria:

- Gamma/contrast correction uniforms are plumbed end-to-end and gated by a settings struct.
- The chosen quality knobs participate in:
  - glyph raster cache keys
  - atlas residency keys where applicable

#### WP4 — `fret-ui` integration (C)

Exit criteria:

- `TextProps` / `StyledTextProps` / `SelectableTextProps` either:
  - carry `TextInput`, or
  - build it deterministically without observing theme revision for shaping.
- Theme-only (paint) changes do not trigger reshaping/re-wrapping (validated by a focused test).

#### WP5 — Ecosystem migrations + smoke demo (D/E)

Exit criteria:

- `ecosystem/fret-markdown`: strikethrough + inline code are represented as spans.
- `ecosystem/fret-code-view`: syntax highlighting is represented as spans and supports wrapping.
- Add an integration demo (or deterministic snapshot) covering:
  - mixed scripts (LTR + RTL)
  - emoji sequences (ZWJ/VS16/keycaps)
  - IME preedit (Windows-first)

#### WP7 — `text_v2` graduation (remove legacy + rename/flatten modules)

Goal:

- Retire the temporary `text_v2` module namespace once the Parley path is the only shaping backend and the
  platform baseline (Windows + macOS) is validated, then rename/flatten modules to reduce churn.

Exit criteria:

- No legacy shaping backend remains (Parley-only; no runtime/feature gates).
- The `text_v2` module namespace is removed:
  - `crates/fret-render/src/text/*` is the canonical module surface for Parley shaping + wrapping.
  - all imports, tests, and call sites stop referencing `text_v2`.
- All text conformance and UI integration tests pass:
  - `cargo nextest run -p fret-render`
  - `cargo nextest run -p fret-ui`
  - `cargo nextest run --workspace` (preferred before landing)
- Documentation is updated to match the new module surface:
  - workstreams referencing `text_v2` are updated
  - ADR cross-references remain valid (update anchors if necessary)

Evidence checklist (when completed):

- `rg -n "text_v2" crates/ ecosystem/` returns no hits (or only in historical docs).
- A focused PR/commit contains only mechanical renames + import updates + test fixes (no behavior changes).
- This tracker no longer lists "`text_v2` naming" as an open question (Risks section).

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

Current:

- Manual emoji conformance harness exists: `apps/fret-examples/src/emoji_conformance_demo.rs`.
  - Web runner supports `?demo=emoji_conformance_demo` and optional bundled emoji fonts via
    `apps/fret-demo-web` feature `emoji-fonts`.
- Automated conformance (unit): emoji sequences (VS16/ZWJ/flags/keycaps) produce
  `GlyphQuadKind::Color` and populate the `color_atlas` when a bundled color emoji font is available
  (`cargo nextest run -p fret-render`; `crates/fret-render/src/text.rs`).
- Manual CJK conformance harness exists: `apps/fret-examples/src/cjk_conformance_demo.rs`.
  - Web runner supports `?demo=cjk_conformance_demo` and optional bundled CJK fonts via
    `apps/fret-demo-web` feature `cjk-lite-fonts`.
- Automated conformance (unit): CJK glyphs populate `mask_atlas`/`subpixel_atlas` when a bundled CJK
  font is available (`cargo nextest run -p fret-render`; `crates/fret-render/src/text.rs`).

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
  - `prepare(&TextInput, TextConstraints) -> (TextBlobId, TextMetrics)`
  - `measure(&TextInput, TextConstraints) -> TextMetrics`
  - keep a borrowed helper `TextInputRef<'_>` for renderer shaping paths.
  - [x] Update ADR 0006 references if signatures change (keep the boundary semantics stable).

### B) `fret-render` text system v2 implementation

**B0 — Legacy backend hardening (pre-Parley)**
- [x] Split shaping vs paint cache keys (geometry reuse across theme-only changes).
- [x] Keep per-span paint as a palette (no paint baked into glyph quads / shaping outputs).

**B1 — Font + fallback + stable keys**
- [x] Define a “font stack key” / revision model that invalidates caches on font DB changes.
  - Evidence: `crates/fret-render/src/text.rs` (`font_stack_key`, `font_db_revision`, `add_fonts`,
    `set_font_families`), `crates/fret-runtime/src/font_catalog.rs` (`TextFontStackKey`),
    `crates/fret-ui/src/declarative/host_widget/paint.rs` (observes `TextFontStackKey`).
- [x] Implement family overrides (`TextFontFamilyConfig`) in the new system.
  - Evidence: `crates/fret-render/src/text.rs` (`TextSystem::set_font_families`),
    `crates/fret-launch/src/runner/desktop/mod.rs` + `crates/fret-launch/src/runner/web.rs`
    (applies config + publishes `TextFontStackKey`).

**B2 — Shaper (Parley)**
- [x] Integrate Parley shaping for a single line/chunk with attributed spans.
- [x] Produce glyph/cluster mapping sufficient for:
  - caret stops
  - hit-testing
  - span-aware paint assignment
  - Evidence: `crates/fret-render/src/text/parley_shaper.rs` (`ShapedCluster`, `ParleyGlyph::is_rtl`),
    `crates/fret-render/src/text/wrapper.rs` (`hit_test_x`), `crates/fret-render/src/text.rs`
    (`caret_stops_for_slice`, `paint_span_for_text_range`).
  - Tests: `crates/fret-render/src/text.rs` (`paint_span_for_text_range_is_directional_across_span_boundary`),
    `crates/fret-render/src/text/wrapper.rs` (`word_wrap_produces_multiple_lines_and_full_coverage`).

**B3 — Wrapper (wrap + truncation)**
- [x] Implement `wrap=None` using a wrapper layer that drives shaping on slices.
- [x] Implement `TextOverflow::Ellipsis` per ADR 0059 (single-line, wired for `TextWrap::None` + `Ellipsis`):
  - stable ellipsis glyph sequence
  - caret/selection mapping rules at truncation boundary
- [x] Implement `wrap=Word` using the wrapper layer (multi-line slicing + caret mapping).

**B4 — Raster + atlas**
- [x] Replace append-only “pen” atlas packing with multi-page free-rect packing (e.g. `etagere`).
- [x] Split atlas by texture kind:
  - monochrome (coverage)
  - polychrome (emoji / color glyphs)
  - optional: subpixel (if multi-channel is adopted)
- [x] Add eviction policy + explicit rebuild knob for debugging.

**B5 — Quality baseline**
- [x] Implement `SUBPIXEL_VARIANTS_X = 4`, `Y = 1/4` (platform policy).
  - Evidence: `crates/fret-render/src/text.rs` (`SUBPIXEL_VARIANTS_X`, `SUBPIXEL_VARIANTS_Y`).
- [x] Add shader gamma/contrast correction uniforms (GPUI-aligned).
  - Evidence: `crates/fret-render/src/text.rs` (`TextQualitySettings`),
    `crates/fret-render/src/renderer/shaders.rs` (`TEXT_SHADER`, `TEXT_SUBPIXEL_SHADER`).
- [x] Ensure quality knobs participate in cache keys (layout/raster).
  - Evidence: subpixel binning participates in `GlyphKey` (`x_bin`, `y_bin`) and atlas residency.
  - Evidence: gamma/contrast participates in scene encoding cache keys:
    `crates/fret-render/src/renderer/render_scene/render.rs` (`SceneEncodingCacheKey.text_quality_key`),
    test `crates/fret-render/src/renderer/tests.rs` (`scene_encoding_cache_is_busted_by_text_quality_changes`).

**B6 — Cache boundary refactor (ADR 0158)**
- [x] Split “layout cache” from “glyph residency cache” (no UVs embedded in `TextShape`).
- [x] Add a frame-driven `TextSystem::prepare_for_scene(...)` that runs even when scene encoding is cached.
- [x] Move atlas allocation/uploads out of `prepare(...)` and into the scene-driven ensure step.
- [x] Define eviction semantics based on “last used frame” and document in-flight safety constraints.

**B7 — Unified glyph key + subpixel policy (ADR 0160)**
- [x] Introduce renderer-owned `GlyphKey` and remove backend-specific glyph keys.
- [x] Introduce a stable `FontFaceKey` registry (decouple from Parley fontique IDs; reserve variable font support).
- [x] Add `GlyphKind::Subpixel` (atlas + shader) and lock `SUBPIXEL_VARIANTS_X/Y` as a platform policy.
- [x] Converge shaping/layout to Parley-only and remove the legacy shaping backend gate.

### C) `fret-ui` integration surface

- [x] Update `TextProps` / `StyledTextProps` / `SelectableTextProps` to carry `TextInput` (or to build it deterministically).
  - Evidence: deterministic `TextInput` building lives on the props and is shared across measure/paint:
    `crates/fret-ui/src/element.rs` (`build_text_input*`) and its use in
    `crates/fret-ui/src/declarative/host_widget/{measure,paint}.rs`.
- [x] Ensure UI caches do not depend on theme revision for shaping (paint-only updates).
  - Evidence: make theme access paint-invalidating (not layout-invalidating) and validate that paint-only theme changes
    do not trigger text reprepare: `crates/fret-ui/src/widget.rs` (`PaintCx::theme`),
    `crates/fret-ui/src/declarative/tests/text_cache.rs` (`theme_color_change_does_not_reprepare_text_in_paint`).
  - Evidence: theme color changes do not change shaping input fingerprints:
    `crates/fret-ui/src/declarative/tests/text_cache.rs` (`theme_color_change_does_not_change_text_input_fingerprints`).
- [x] Keep selection state stable and based on byte indices (ADR 0044/0045/0046).
  - Evidence: clamp persisted `selection_anchor`/`caret` to valid boundaries on both the event and paint paths:
    `crates/fret-ui/src/text_edit.rs` (`clamp_selection_to_grapheme_boundaries`),
    `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs`,
    `crates/fret-ui/src/declarative/host_widget/paint.rs`,
    test `crates/fret-ui/src/declarative/tests/selection_indices.rs`.
  - Evidence: clipboard extraction clamps selection indices defensively:
    `crates/fret-ui/src/text_edit.rs` (`commands::apply_clipboard`),
    tests `crates/fret-ui/src/text_input/tests.rs` (`text_input_copy_clamps_out_of_range_selection_indices`) and
    `crates/fret-ui/src/text_area/tests.rs` (`text_area_copy_clamps_out_of_range_selection_indices`).
  - Evidence: a11y selection publishing clamps selection indices:
    `crates/fret-ui/src/text_input/widget.rs` (`semantics`) and `crates/fret-ui/src/text_area/widget.rs` (`semantics`).

### D) Ecosystem migrations

- [x] `ecosystem/fret-markdown`: express `strikethrough`, `inline code` as spans (no fallback hacks).
  - Exit criteria: inline styles are represented as `TextSpan` overrides end-to-end (parse → model → `TextInput`),
    with no special-casing in the renderer for markdown widgets.
  - Exit criteria: add at least one focused test that asserts span boundaries remain stable across wrapping.
  - Evidence: Markdown rich inline path builds span overrides:
    `ecosystem/fret-markdown/src/lib.rs` (`build_rich_attributed_text`) + test
    `ecosystem/fret-markdown/src/tests.rs` (`rich_inline_builds_spans_for_inline_code_and_strikethrough`).
  - Evidence: `TextSpan.paint.bg` is painted as quads behind the text for `SelectableText`:
    `crates/fret-ui/src/declarative/host_widget/paint.rs` (selectable text paint path) + test
    `crates/fret-ui/src/declarative/tests/selection_indices.rs` (`selectable_text_paints_span_background_quads`).
- [x] `ecosystem/fret-code-view`: represent highlighting as spans and support wrapping.
  - Exit criteria: syntax highlighting maps to span overrides (color/font/style) and can render with `TextWrap::Word`
    or `TextWrap::Grapheme` (no "one-line blob" fallback).
  - Exit criteria: add a deterministic smoke test (snapshot or structural assertions) for:
     mixed-script + emoji + long-token wrapping + selection.
  - Evidence: code blocks build `AttributedText` with per-segment `TextSpan` colors and render as a single selectable
     text surface (wrap configurable):
     `ecosystem/fret-code-view/src/code_block.rs` (`build_code_block_rich`, `render_code_block_text`).
  - Evidence: add `CodeBlockWrap::Grapheme` and map it to `TextWrap::Grapheme`:
     `ecosystem/fret-code-view/src/code_block.rs` (`text_wrap_for_code_block_wrap`) + test
     `ecosystem/fret-code-view/src/code_block.rs` (`code_block_wrap_maps_to_text_wrap`).
  - Evidence: deterministic wrap+selection smoke test:
    `ecosystem/fret-code-view/tests/wrap_and_selection_smoke.rs` (`code_block_wrap_grapheme_and_selection_smoke`).

### E) Tests & conformance

- [x] Unit tests: span invariant validation and clamping behavior.
  - Evidence: `crates/fret-render/src/text.rs` (`sanitize_spans_for_text` + tests `sanitize_spans_*`).
- [x] Unit tests: ellipsis truncation caret/hit-test mapping.
  - Evidence: `crates/fret-render/src/text.rs` (`ellipsis_truncation_hit_test_maps_ellipsis_region_to_kept_end`),
    `crates/fret-render/src/text/wrapper.rs` (`none_ellipsis_adds_zero_len_cluster_at_cut_end`).
- [x] Unit tests: wrap boundaries across span boundaries.
  - Evidence: `crates/fret-render/src/text/wrapper.rs` (`word_wrap_produces_multiple_lines_and_full_coverage`).
- [x] Unit conformance: color emoji glyphs populate `color_atlas` when a bundled color emoji font is present.
- [x] Integration demo: mixed-script + emoji + IME preedit smoke (deterministic snapshot).
  - Evidence: `crates/fret-ui/src/tree/tests/window_text_input_snapshot.rs`
    (`snapshot_reports_composed_utf16_ranges_for_mixed_script_text_during_ime_preedit`).
- [x] Integration demo: emoji sequences (ZWJ/VS16/keycaps) + IME preedit smoke (deterministic snapshot).
  - Evidence: `crates/fret-ui/src/tree/tests/window_text_input_snapshot.rs`
    (`snapshot_reports_composed_utf16_ranges_for_emoji_sequences_during_ime_preedit`).

## Risks / Open Questions

- Ellipsis glyph choice: keep current `"…"` vs legacy placeholder; ensure fallback is stable across platforms.
- Parley cluster/index semantics: ensure we can map to UTF-8 byte offsets with correct clamping.
- Atlas eviction determinism: ensure eviction does not cause flicker without explicit rebuild strategy.
- `text_v2` naming: tracked as WP7 (graduation / rename / flattening) once the platform baseline is validated.
- Platform defaults: decide `TextQualitySettings` defaults per platform (Windows-first; validate macOS after contract stabilization).

## Progress Log (append-only)

- 2026-01-13: ADR 0157 added (design locked), worktree created.
- 2026-01-13: ADR 0158 added (layout cache boundary + glyph residency direction).
- 2026-01-13: M0 contract landed (commit `3bb0fc8`).
- 2026-01-13: M1 started: add Parley dependency + single-line shaper prototype in `crates/fret-render/src/text_v2/mod.rs`.
- 2026-01-13: M1.1: split shaping/paint caches in the current text backend (`TextShapeKey` + per-span palette; theme-only changes no longer force reshaping).
- 2026-01-13: M1.2: add `text_v2` wrapper prototype for `wrap=None + Ellipsis` with cluster-based hit-test mapping (unit tests only; not integrated yet).
- 2026-01-13: M1.3: wire Parley `wrap=None + Ellipsis` through `TextSystem::prepare_*` (renders via swash into the existing atlases; still missing fractional positioning + font config integration).
- 2026-01-13: M1.4: align Parley rasterization with cosmic-text subpixel binning + wire `add_fonts` and `set_font_families` into Parley fontique generics (reduces drift across backends).
- 2026-01-13: M1.5: add Parley word wrap + multiline layout (commit `12e0aa2`), then extend wrapping across newlines (commit `63a00be`).
- 2026-01-13: M1.6: add multipage glyph atlas budget + plumb atlas page through draws (commit `c29a866`).
- 2026-01-13: M1.7: evict unreferenced glyph atlas pages (commit `eac5619`).
- 2026-01-13: M1.8: evict unused glyphs from the atlas via per-glyph live refs (commit `2983e98`).
- 2026-01-13: B6: decouple text layout from glyph atlas residency + add `prepare_for_scene` and atlas revision cache key (commit `4885937`).
- 2026-01-13: ADR 0160 added: unify glyph identity (`GlyphKey`) + subpixel rendering policy (commit `d56a8be`).
- 2026-01-13: B7.1: unify glyph key and switch `prepare` to Parley-only (commit `797fe93`).
- 2026-01-13: B7.2: add subpixel atlas + shader/pipeline and platform subpixel policy (commit `8282cf1`).
- 2026-01-13: B7.3: add stable `FontFaceKey` registry (commit `9a7f81a`).
- 2026-01-13: Align `TextService` to `prepare(&TextInput, ...)` and introduce `TextInputRef<'_>` for shaping-only codepaths.
- 2026-01-14: Add automated color emoji atlas conformance test in `fret-render` (commit `26ddc6d`).
- 2026-01-14: Expose `fret-fonts` bootstrap/emoji bundles (commit `84b8a65`).
- 2026-01-14: Expand emoji sequence conformance (VS16/ZWJ/flags/keycaps) in `fret-render` (commit `dbc5a89`).
- 2026-01-14: Add cjk-lite bundle + CJK conformance demo and atlas test (commit `8c0700b`).
- 2026-01-14: Add wasm fallback candidates for bundled fonts (commit `56b6e92`).
- 2026-01-31: Add deterministic window snapshot smoke test for mixed-script + emoji + IME preedit.
