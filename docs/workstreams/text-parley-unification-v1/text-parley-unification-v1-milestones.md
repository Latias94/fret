# Text Parley Unification v1 — Milestones

This file defines small, landable milestones for the Parley unification refactor.
For day-to-day tasks, use:

- `docs/workstreams/text-parley-unification-v1/text-parley-unification-v1-todo.md`

## M0 — Baseline + edge-case gates

Deliverables:

- A renderer-side unit test gate for empty-string metrics + caret rect height.
- A UI-side unit test gate for empty input caret/selection visibility (TextInput + TextArea).

Exit criteria:

- `cargo nextest run -p fret-render-wgpu empty_string_produces_nonzero_line_metrics_and_caret_rect` passes.
- A UI-level focused test exists and passes in `cargo nextest run -p fret-ui` (or the smallest package that hosts it).

## M1 — Coordinate mapping consolidation (UI)

Deliverables:

- One shared “content space → widget box space” mapping helper is used by:
  - TextInput selection/caret drawing
  - TextArea selection/preedit drawing
- Any remaining widget-local vertical placement math is either deleted or intentionally documented.

Exit criteria:

- A targeted test (or diag script) proves selection/caret vertical placement does not regress when:
  - widget height is larger than one line
  - scale factor changes (e.g. 1.0 vs 1.5)

## M2 — Parley-only shaping path (renderer)

Deliverables:

- Parley is the default shaping path for the primary renderer backend(s).
- Wrapping and ellipsis behavior is deterministic and locked with focused tests.

Exit criteria:

- `cargo nextest run -p fret-render-wgpu` passes.
- A small conformance suite exists for line-breaking edge cases (fixture-driven where possible).

## M3 — IME + editor-grade geometry closure

Deliverables:

- Scripted IME repro(s) exist for preedit ranges + caret placement.
- Geometry correctness around span boundaries and mixed direction text is gated.

Exit criteria:

- A `fretboard diag` suite (or equivalent scripted repro) demonstrates:
  - stable caret height/position during IME composition
  - stable selection rect segmentation under wrapping

