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
  - truncation does not produce out-of-bounds indices.
  - caret/hit-test around the truncation boundary is representable and deterministic.
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

- [ ] Replace `RichText`/`TextRun` with `TextInput`/`TextSpan` (migrate call sites).
- [ ] Define `TextShapingStyle` vs `TextPaintStyle` types and persistence strategy (settings).
- [ ] Enforce span invariants:
  - sum(len) == text.len()
  - boundaries are UTF-8 char boundaries
- [ ] Update `TextService` to:
  - `prepare(TextInput<'_>, TextConstraints) -> (TextBlobId, TextMetrics)`
  - `measure(TextInput<'_>, TextConstraints) -> TextMetrics`

### B) `fret-render` text system v2 implementation

- [ ] Integrate Parley shaping for a single line/chunk with attributed spans.
- [ ] Implement wrapper layer:
  - `wrap=None` + `TextOverflow::Ellipsis` (single-line)
  - `wrap=Word` (multi-line)
- [ ] Produce glyph/cluster mapping sufficient for:
  - caret stops
  - hit-testing
  - span-aware paint assignment
- [ ] Replace append-only atlas packing with multi-page free-rect packing (e.g. `etagere`).
- [ ] Split atlas by texture kind:
  - monochrome (coverage)
  - polychrome (emoji / color glyphs)
- [ ] Implement quality baseline:
  - `SUBPIXEL_VARIANTS_X = 4`, `Y = 1/4` (platform policy)
  - shader gamma/contrast correction uniforms (GPUI-aligned)

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

## Notes

- Zed/GPUI is a useful reference decomposition:
  - `repo-ref/zed/crates/gpui/src/text_system/line_wrapper.rs`
  - `repo-ref/zed/crates/gpui/src/text_system/line_layout.rs`

