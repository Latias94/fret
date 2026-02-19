# Text Intrinsic Sizing + Wrap Semantics v1 — TODO

Status: Active
Start: 2026-02-19

This is the task-level checklist for `docs/workstreams/text-intrinsic-sizing-and-wrap-v1.md`.

## Repros (UI Gallery)

- [ ] Create a minimal page/harness that demonstrates shrink-wrap + `TextWrap::Word` intrinsic sizing.
- [x] Add a second repro for long-token prose (`URL-like-token-without-spaces`) where `WordBreak` is expected.
  - Implemented in `Text / Measured Bounds Overlay` (`apps/fret-ui-gallery/.../text/measure_overlay.rs`).
- [ ] Capture baseline diag bundles (native) for both cases.
  - [x] `tools/diag-scripts/ui-gallery-text-measure-overlay-wrap-modes-screenshots.json`
  - [x] `tools/diag-scripts/ui-gallery-tabs-wrap-and-baseline-screenshots.json`

## Intrinsic sizing implementation

- [x] Decide v1 segmentation for `TextWrap::Word` min-content:
  - Use Parley/UAX#14 line-breaking opportunities with `OverflowWrap::Normal` (no emergency mid-token breaks).
  - Do not introduce a custom GPUI-like `is_word_char` set in v1; use `WordBreak`/`Grapheme` explicitly when needed.
- [ ] Add renderer measurement API surface for intrinsic widths (internal-first):
  - [ ] `max_content_width`
  - [ ] `min_content_width` (wrap-dependent)
- [x] Wire intrinsic widths into the layout engine integration (Taffy min/max-content probes).
- [ ] Ensure measurement and paint use the same resolved wrap width for definite layouts.

## Tests (fast, deterministic)

- [x] `fret-render-wgpu`: unit tests for intrinsic width behavior per wrap mode.
- [x] `fret-render-wgpu`: wasm conformance gate runs the same wrap fixtures (Parley/UAX#14) under `wasm-bindgen-test`.
- [ ] `fret-ui`: integration test that validates no “vertical text” under shrink-wrap container.
- [ ] Add at least one selection/hit-test test case for wrapped text (ensure indices remain stable).

## Ecosystem authoring ergonomics

- [ ] Add UI kit helpers:
  - [x] `text_prose_break_words` (or equivalent) using `TextWrap::WordBreak`
  - [x] `text_code_wrap` using `TextWrap::Grapheme`
- [ ] Update shadcn recipes where long-token body copy currently overflows horizontally.
- [ ] Add short authoring note to the docs page(s) that explain when to use each wrap mode.

## Follow-ups (separate feature)

- [ ] Explicit multiline `line-clamp` design + ADR update (must follow ADR 0221 rules).
