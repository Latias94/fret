# ADR 0059: Text Overflow (Ellipsis) and Truncation

- Status: Accepted
- Date: 2025-12-25

## Context

Tailwind/shadcn-style component recipes frequently rely on single-line truncation:

- `truncate` (single-line, `…` when content exceeds available width)
- `line-clamp-*` (multi-line, future)

Fret already supports measuring and shaping text under `TextConstraints { max_width, wrap }`, but
with `wrap = None` there was no standardized way to express "clip vs ellipsis" overflow behavior.
This made it hard to produce consistent UX for lists/command palettes/menus without custom widget
logic or ad-hoc string truncation in components.

## Decision

1) Add `TextOverflow` to `fret-core` and extend `TextConstraints` with an `overflow` field:

- `TextOverflow::Clip` (default)
- `TextOverflow::Ellipsis`

2) Implement `TextOverflow::Ellipsis` in the text backend (Parley-based renderer text system) for the
single-line case (`wrap = None` and `max_width = Some(_)`):

- When the shaped line width exceeds `max_width`, replace the tail of the line with an ellipsis
  glyph sequence (best-effort "…").
- The ellipsis glyphs are assigned `start=end=cut_end` so the produced caret/selection metadata
  remains representable as "end of visible content".

3) Expose the overflow semantics at the declarative authoring layer via `TextProps.overflow`.

## Non-Goals (for now)

- Multi-line truncation / `line-clamp-*` (requires explicit max-line semantics and more complex
  shaping/selection mapping).
- Per-span overflow behavior (overflow is a constraint-level decision).

## Consequences

- Component-layer recipes can express truncation via composition:
  `TextProps { wrap: None, overflow: Ellipsis, .. }` under a constrained width (e.g. flex item with
  `min_width=0`).
- Theme/token naming is unaffected; truncation is a layout/text constraint semantic rather than a
  theme concern.
- The ellipsis implementation is backend-driven and must be kept consistent across future text
  backends (e.g. wasm).
