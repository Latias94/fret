# Text Parley Unification v1 — TODO Tracker

Status: Active (workstream tracker)

This document tracks TODOs for:

- `docs/workstreams/text-parley-unification-v1.md`
- `docs/workstreams/text-parley-unification-v1-milestones.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `TPU-{area}-{nnn}`
- Prefer 1–3 evidence anchors (file paths + key functions/tests and/or diag scripts).

## M0 — Baseline gates (edge cases)

- [x] TPU-render-001 Ensure empty strings produce non-zero `TextMetrics` and caret rect height.
  - Evidence:
    - `crates/fret-render-wgpu/src/text/parley_shaper.rs`
    - `crates/fret-render-wgpu/src/text/mod.rs` (`empty_string_produces_nonzero_line_metrics_and_caret_rect`)

- [x] TPU-ui-001 Add a UI-side gate for empty input selection/caret visibility (TextInput + TextArea).
  - Evidence:
    - `crates/fret-ui/src/text/input/tests.rs`
      (`text_input_caret_is_visible_even_when_backend_reports_zero_height_metrics_for_empty_text`)
    - `crates/fret-ui/src/text/area/tests.rs`
      (`caret_is_visible_even_when_backend_reports_zero_height_caret_rect`)

- [x] TPU-render-002 Decide and document the renderer contract for selection/preedit rect height:
  - Option A: renderer guarantees `height > 0` for all returned rects,
  - Option B: UI inflates by line metrics (keep behavior gated either way).
  - Decision: Option A (renderer guarantees non-degenerate geometry rects).
  - Evidence:
    - `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
      (section “Geometry rectangles must be non-degenerate”)
    - `crates/fret-render-wgpu/src/text/mod.rs`
      (`selection_and_caret_rects_are_nonzero_even_with_zero_line_height_override`)

## M1 — Coordinate mapping unification

- [ ] TPU-ui-010 Inventory all “baseline/vertical placement” math in `fret-ui` and collapse onto one helper.
  - Evidence start:
    - `crates/fret-ui/src/text/coords.rs`

- [ ] TPU-ui-011 Verify hit-testing and selection mapping use the same content→box transform across widgets.
  - Target widgets:
    - TextInput, TextArea, SelectableText

## M2 — Parley-only shaping path (remove drift)

- [ ] TPU-render-020 Audit for any non-Parley shaping paths still used by default builds and decide migration steps.
  - Goal: one shaping engine per backend, with Parley as the default direction.

- [ ] TPU-render-021 Ensure wrapping/ellipsis policies are deterministic and tested for “hard” strings:
  - mixed scripts (LTR/RTL)
  - emoji sequences (ZWJ/VS16/keycaps)
  - identifiers + CJK punctuation

## M3 — IME + editor-grade polish

- [ ] TPU-ui-030 Add a scripted diag repro for IME preedit caret/selection drift and baseline placement.
  - Target first: Windows IME (then macOS).

- [ ] TPU-ui-031 Align behavior with `repo-ref/zed` for:
  - caret affinity near line breaks
  - selection rect segmentation rules
  - hit-test around grapheme clusters
