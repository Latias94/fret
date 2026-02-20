# Text Layout Alignment v1 (Parley-first) — TODO

Status: Active (2026-02-20)

This is a living task list. Keep items small enough to land with a tight regression gate.

## P0 — Coordinate space alignment (UI correctness)

- [x] Centralize “text-local <-> element-local” mapping (includes vertical placement offset).
      (Shared helpers used by paint + selectable-text pointer hit-testing.)
- [x] Ensure span background quads are positioned using the same vertical placement offset as the
      text draw op for:
      - `SelectableText`
      - `StyledText`
- [x] Ensure interactive-span bounds stored for `SelectableText` use the same mapping (so pointer
      hit regions match the rendered text).
- [x] Ensure selection highlight quads use the same mapping (fixed-height / centered text should
      not drift).

## P1 — Rich spans parity

- [x] `StyledText`: paint `TextPaintStyle.bg` spans as quads behind the text.
- [x] Add a unit test gate for `StyledText` span backgrounds (ordering + placement).

## P2 — Mixed-direction (bidi) geometry fidelity

- [x] Allow `selection_rects(_clipped)` to return multiple rectangles per line for bidi ranges
      when needed (avoid “single rect per line” overpainting).
- [x] Add fixtures/tests covering:
      - RTL-only selection (already present)
      - mixed LTR/RTL selection with non-contiguous visual segments (unit test gate)

## P3 — Platform text input (IME) robustness

- [x] Re-evaluate v1 “caret-anchored marked range” strictness for
      `platform_text_input_replace_and_mark_text_in_range_utf16` (avoid false negatives for some
      IME behaviors).
- [x] Add targeted tests for UTF-16 composed-view mapping around preedit.
