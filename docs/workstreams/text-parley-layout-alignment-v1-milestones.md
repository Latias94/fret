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

## M2 — Mixed-direction selection rectangles are segment-accurate

Goal:

- Avoid overpainting and incorrect bounds for bidi selections by returning multi-segment rects per
  line when the visual selection is discontinuous.

Exit criteria:

- `selection_rects(_clipped)` can emit multiple rects per line when required.
- Add at least one focused unit test covering a mixed LTR/RTL selection range that produces
  multiple rects.

Status: Not started.

## M3 — IME/platform query robustness (bounded)

Goal:

- Improve compatibility with platform IME replacement/marked-range behaviors without changing the
  contract shape.

Exit criteria:

- Reduce “return false” failure modes in `platform_text_input_replace_and_mark_text_in_range_utf16`
  while keeping invariants documented and tested.
- Add tests for at least one non-trivial IME-marked scenario.

Status: Not started.
