# Text Layout Alignment v1 (Parley-first) — Milestones

Status: Active (2026-02-20)

Exit criteria for each milestone should be verifiable with unit tests and/or diag evidence.

## M0 — Workstream scaffolding

Exit criteria:

- This charter + TODO + milestones exist and describe scope and gates.

Status: Done.

## M1 — Span background parity for `StyledText` + coordinate correctness

Goal:

- Rich span backgrounds (`TextPaintStyle.bg`) should render consistently for `StyledText` and
  `SelectableText`.
- Background quads must be positioned using the same vertical placement offset as the text draw op
  (fixed-height/centered bounds must not drift).

Exit criteria:

- `StyledText` paints background quads for spans with `TextPaintStyle.bg`.
- Both `StyledText` and `SelectableText` apply vertical placement offsets consistently for:
  - span background quads
  - selection highlight quads
  - stored interactive-span bounds (SelectableText)
- Unit tests:
  - existing `selectable_text_paints_span_background_quads` is extended to validate placement
  - new `styled_text_paints_span_background_quads` validates ordering + placement

Status: Done.

## Implementation status (2026-02-20)

- M0: Done.
- M1: Done.
  - Evidence:
    - `crates/fret-ui/src/declarative/host_widget/paint.rs` (shared background-quad helper + vertical-offset mapping)
    - `crates/fret-ui/src/declarative/tests/selection_indices.rs` (fixed-height placement gates)
- M2: Done.
  - Evidence:
    - `crates/fret-render-wgpu/src/text/mod.rs` (cluster-driven multi-segment `selection_rects(_clipped)`)
    - `crates/fret-render-wgpu/src/text/mod.rs` (unit test gate: `mixed_direction_selection_rects_split_across_visual_runs`)
- M3: Done.
  - Evidence:
    - `crates/fret-ui/src/text/input/widget.rs` (tolerant `platform_text_input_replace_and_mark_text_in_range_utf16`)
    - `crates/fret-ui/src/text/area/widget.rs` (tolerant `platform_text_input_replace_and_mark_text_in_range_utf16`)
    - `crates/fret-ui/src/tree/tests/platform_text_input.rs` (unit test gate: replacement-range commit + marked mismatch)
- M4: Done.
  - Evidence:
    - `crates/fret-ui/src/text/coords.rs` (shared vertical placement mapping helper)
    - `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs` (pointer hit-testing uses text-local coordinates)
    - `crates/fret-ui/src/declarative/tests/selection_indices.rs` (unit test gate: `selectable_text_pointer_hit_test_uses_text_local_coordinates`)
- M5: Done.
  - Evidence:
    - `crates/fret-ui/src/text/coords.rs` (shared vertical placement + baseline mapping helper)
    - `crates/fret-ui/src/text/input/widget.rs` (paint + platform geometry queries reuse shared mapping)
    - `crates/fret-ui/src/text/input/input.rs` (`caret_rect` uses shared mapping + first-line metrics when available)

## M2 — Mixed-direction selection rectangles are segment-accurate

Goal:

- Avoid overpainting and incorrect bounds for bidi selections by returning multi-segment rects per
  line when the visual selection is discontinuous.

Exit criteria:

- `selection_rects(_clipped)` can emit multiple rects per line when required.
- Add at least one focused unit test covering a mixed LTR/RTL selection range that produces
  multiple rects.

Status: Done.

## M3 — IME/platform query robustness (bounded)

Goal:

- Improve compatibility with platform IME replacement/marked-range behaviors without changing the
  contract shape.

Exit criteria:

- Reduce “return false” failure modes in `platform_text_input_replace_and_mark_text_in_range_utf16`
  while keeping invariants documented and tested.
- Add tests for at least one non-trivial IME-marked scenario.

Status: Done.

## M4 — Pointer hit-testing uses the same coordinate mapping as paint

Goal:

- Ensure selectable-text pointer hit-testing passes **text-local** coordinates to `TextService`
  (matching the coordinate space used by `selection_rects` / `caret_rect` and the paint mapping).

Exit criteria:

- Shared helper for computing vertical placement offsets is used by both paint and pointer hit-testing.
- Add a unit test gate that clicks inside vertically centered text and asserts `hit_test_point`
  receives text-local coordinates.

Status: Done.

## M5 — TextInput uses shared vertical placement mapping end-to-end

Goal:

- Ensure `TextInput` uses the same vertical placement mapping as declarative text elements for:
  - paint origin (baseline + vertical offset)
  - platform text-input geometry queries (`bounds_for_range`, `character_index_for_point`)
  - caret placement (best-effort line-metrics aware)

Exit criteria:

- `TextInput` paint uses `compute_text_vertical_offset_and_baseline`.
- Platform geometry queries use the same vertical placement offset as paint.
- `TextInput::caret_rect` uses the same mapping and consults `first_line_metrics` when available.

Status: Done.
