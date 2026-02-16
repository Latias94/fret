# Text Line Breaking v1 — Milestones

This is a milestone checklist for:

- `docs/workstreams/text-line-breaking-v1.md`

## M0 — Conformance suite + invariants (no behavior changes)

Exit criteria:

- A focused wrap conformance test module exists (unit tests):
  - CJK punctuation cases,
  - identifiers (snake/camel),
  - paths/URLs,
  - emoji ZWJ/VS16,
  - long tokens.
- Existing invariant remains gated:
  - trailing whitespace at soft wrap is selectable:
    - `crates/fret-render-wgpu/src/text/mod.rs:6294` (do not regress).
- Tests are deterministic on bundled-font-only environments.

Evidence checklist:

- `cargo nextest run -p fret-render-wgpu`
- `cargo nextest run -p fret-render`

Evidence anchors (initial baseline):

- Wrap conformance fixtures:
  - `crates/fret-render-wgpu/src/text/tests/fixtures/text_wrap_conformance_v1.json`
  - `crates/fret-render-wgpu/src/text/wrapper.rs` (`text_wrap_conformance_v1_fixtures`)

## M1 — Wrapper heuristic upgrade (Unicode break opportunities)

Exit criteria:

- Word wrap candidate selection no longer relies on `is_word_char` heuristics alone.
- CJK punctuation handling improves for the conformance set (document known gaps).
- Performance:
  - no O(n²) regressions on long paragraphs,
  - resize jitter remains bounded (guard with a dedicated long-paragraph probe if this regresses).

Evidence checklist:

- Conformance tests updated and passing.
- A perf probe exists (even a unit-level “worst case” micro test) to catch obvious regressions.

## M2 — Parley-driven line breaking (migration)

Exit criteria:

- A Parley-backed “shape paragraph with wrap width” path exists and is integrated into the renderer
  wrapper for `TextWrap::Word` (replacing the legacy wrapper implementation).
- Correctness gates:
  - caret/selection mapping correctness across soft breaks:
    - `hit_test_point` / `hit_test_x`
    - `caret_rect` (affinity rules)
    - `selection_rects(_clipped)`
  - measurement/paint agree on wrapping inputs (no layout height drift).
  - The editor-grade invariant holds:
    - trailing whitespace at a soft wrap boundary remains selectable.
- Conformance gates:
  - the fixture-driven wrap conformance suite passes under the Parley path (or any gaps are
    enumerated as explicit TODOs with concrete examples).
- Performance:
  - no O(n²) regressions on long paragraphs,
  - resize jitter remains bounded under width oscillations.
- Cleanup:
  - the legacy wrapper implementation is deleted (no compatibility path retained).

Evidence checklist:

- `cargo nextest run -p fret-render-wgpu`
- `cargo nextest run -p fret-ui` (sanity)

Evidence anchors (expected):

- Paragraph line breaking entry point:
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs` (new paragraph shaping helper)
- Wrapper integration:
  - `crates/fret-render-wgpu/src/text/wrapper.rs`
- Invariants and geometry query tests:
  - `crates/fret-render-wgpu/src/text/mod.rs` (caret/hit-test/selection tests)
  - `crates/fret-render-wgpu/src/text/mod.rs:6294` (trailing-whitespace selectable gate; do not regress)
- Resize jitter perf/diag guard (catastrophic regression):
  - `tools/diag-scripts/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json`
  - `tools/perf/diag_text_wrap_resize_jitter_smoke_gate.py`

## M3 — RTL + mixed-script staging

Exit criteria:

- RTL/mixed-direction paragraphs are either:
  - fully supported, or
  - explicitly documented as limited (with deterministic tests covering the limitation surface).
- Additional mapping (if required) is implemented with tests.

Evidence anchors (expected):

- Wrapped RTL hit testing and geometry:
  - `crates/fret-render-wgpu/src/text/mod.rs`
    (`rtl_word_wrap_hit_test_maps_line_edges_to_logical_ends`)
- Mixed-direction wrapped selection geometry:
  - `crates/fret-render-wgpu/src/text/mod.rs`
    (`mixed_direction_word_wrap_selection_rects_for_rtl_range_are_nonempty`)
