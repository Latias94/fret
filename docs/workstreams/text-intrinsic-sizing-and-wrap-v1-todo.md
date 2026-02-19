# Text Intrinsic Sizing + Wrap Semantics v1 — TODO

Status: Active
Start: 2026-02-19

This is the task-level checklist for `docs/workstreams/text-intrinsic-sizing-and-wrap-v1.md`.

## Repros (UI Gallery)

- [ ] Create a minimal page/harness that demonstrates shrink-wrap + `TextWrap::Word` intrinsic sizing.
- [ ] Add a second repro for long-token prose (`URL-like-token-without-spaces`) where `WordBreak` is expected.
- [ ] Capture baseline diag bundles (native) for both cases.

## Intrinsic sizing implementation

- [ ] Decide v1 tokenization for “longest token”:
  - [ ] whitespace-only split, or
  - [ ] GPUI-like `is_word_char` candidate set, or
  - [ ] hybrid (whitespace split with extra token characters).
- [ ] Add renderer measurement API surface for intrinsic widths (internal-first):
  - [ ] `max_content_width`
  - [ ] `min_content_width` (wrap-dependent)
- [x] Wire intrinsic widths into the layout engine integration (Taffy min/max-content probes).
- [ ] Ensure measurement and paint use the same resolved wrap width for definite layouts.

## Tests (fast, deterministic)

- [x] `fret-render-wgpu`: unit tests for intrinsic width behavior per wrap mode.
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
