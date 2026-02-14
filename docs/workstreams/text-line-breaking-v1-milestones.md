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

## M1 — Wrapper heuristic upgrade (Unicode break opportunities)

Exit criteria:

- Word wrap candidate selection no longer relies on `is_word_char` heuristics alone.
- CJK punctuation handling improves for the conformance set (document known gaps).
- Performance:
  - no O(n²) regressions on long paragraphs,
  - resize jitter remains bounded (shape-once path still works).

Evidence checklist:

- Conformance tests updated and passing.
- A perf probe exists (even a unit-level “worst case” micro test) to catch obvious regressions.

## M2 — Parley-driven line breaking (migration)

Exit criteria:

- A Parley-backed “shape paragraph with wrap width” path exists and is used for LTR-only cases first.
- Caret/selection mapping correctness:
  - hit test,
  - caret rect,
  - selection rects,
  - affinity at soft breaks.
- The “selectable trailing whitespace” invariant holds under the new path.

Evidence checklist:

- `cargo nextest run -p fret-render-wgpu`
- `cargo nextest run -p fret-ui` (sanity)

## M3 — RTL + mixed-script staging

Exit criteria:

- RTL/mixed-direction paragraphs are either:
  - fully supported, or
  - explicitly routed to the compatibility wrapper path with documented limitations.
- Additional mapping (if required) is implemented with tests.

