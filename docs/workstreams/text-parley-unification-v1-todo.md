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
    - `crates/fret-render-text/src/parley_shaper.rs`
    - `crates/fret-render-wgpu/src/text/mod.rs` (`empty_string_produces_nonzero_line_metrics_and_caret_rect`)

- [x] TPU-ui-001 Add a UI-side gate for empty input selection/caret visibility (TextInput + TextArea).
  - Evidence:
    - `crates/fret-ui/src/text/input/tests.rs`
      (`text_input_draws_caret_when_focused_and_empty`)
    - `crates/fret-ui/src/text/area/tests.rs`
      (`caret_is_visible_when_text_area_is_focused_and_empty`)

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

- [x] TPU-ui-010 Inventory all “baseline/vertical placement” math in `fret-ui` and collapse onto one helper.
  - Evidence:
    - `crates/fret-ui/src/text/coords.rs` (`compute_text_vertical_offset_and_baseline`,
      `compute_first_line_box_top_and_height`)
    - `crates/fret-ui/src/text/input/input.rs` (caret placement uses the helper)
    - `crates/fret-ui/src/text/input/widget.rs` (selection box uses the helper)

- [x] TPU-ui-011 Verify hit-testing and selection mapping use the same content→box transform across widgets.
  - Target widgets:
    - TextInput, TextArea, SelectableText
  - Evidence:
    - `crates/fret-ui/src/text/coords.rs` (`TextBoxMapping`,
      `compute_text_box_mapping_for_vertical_placement`)
    - `crates/fret-ui/src/text/area/widget.rs` (hit-testing + caret/selection mapping via
      `TextBoxMapping`)
    - `crates/fret-ui/src/declarative/host_widget/paint.rs` (SelectableText hit-testing + selection
      rect mapping via `TextBoxMapping`)
    - `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs` (SelectableText event
      hit-testing uses the same mapping helper as paint)
    - `crates/fret-ui/src/declarative/tests/selection_indices.rs`
      (`selectable_text_pointer_hit_test_uses_text_local_coordinates`)

## M2 — Parley-only shaping path (remove drift)

- [x] TPU-render-020 Audit for any non-Parley shaping paths still used by default builds and decide migration steps.
  - Goal: one shaping engine per backend, with Parley as the default direction.
  - Evidence:
    - `crates/fret-render-text/src/parley_shaper.rs` (single shaping engine used by default)
    - `crates/fret-render-wgpu/src/text/mod.rs` (`pub(crate) mod parley_shaper` re-export)

- [x] TPU-render-021 Ensure wrapping/ellipsis policies are deterministic and tested for “hard” strings:
  - mixed scripts (LTR/RTL)
  - emoji sequences (ZWJ/VS16/keycaps)
  - identifiers + CJK punctuation
  - Evidence start:
    - `crates/fret-render-text/src/geometry.rs`
      (`caret_rects_are_non_degenerate_at_grapheme_boundaries_for_zwj_emoji`,
      `caret_rects_are_non_degenerate_at_grapheme_boundaries_for_keycap_emoji`,
      `caret_rects_are_non_degenerate_at_grapheme_boundaries_for_regional_indicator_flag`,
      `caret_rects_are_non_degenerate_at_grapheme_boundaries_for_vs16_emoji`)
    - `crates/fret-render-text/src/wrapper.rs`
      (`none_ellipsis_does_not_split_zwj_emoji_grapheme_cluster`)
      (`none_ellipsis_does_not_split_keycap_grapheme_cluster`,
      `none_ellipsis_does_not_split_regional_indicator_flag_grapheme_cluster`)
      (`grapheme_wrap_does_not_split_zwj_clusters`)
    - `crates/fret-render-text/src/text/tests/fixtures/text_wrap_conformance_v1.json`
      (`emoji_keycap_word_break_wrap`, `emoji_regional_indicator_flag_word_break_wrap`,
      `emoji_zwj_sequence_word_break_wrap`, `emoji_vs16_word_break_wrap`)

## M3 — IME + editor-grade polish

- [ ] TPU-ui-030 Add a scripted diag repro for IME preedit caret/selection drift and baseline placement.
  - Target first: Windows IME (then macOS).

- [ ] TPU-ui-031 Align behavior with `repo-ref/zed` for:
  - caret affinity near line breaks
  - selection rect segmentation rules
  - hit-test around grapheme clusters
