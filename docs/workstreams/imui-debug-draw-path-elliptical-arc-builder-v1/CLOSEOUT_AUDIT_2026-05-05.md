# ImUi Debug Draw Path Elliptical Arc Builder v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the scoped Dear ImGui `PathEllipticalArcTo` ergonomics follow-on above the
canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `ImUiDebugDrawPath::elliptical_arc_to`.
- Appended sampled rotated elliptical arc points to the temporary path builder.
- Used a stable default segment count for `segments == 0`.
- Treated invalid radii, rotation, and angle inputs as no-op.
- Kept the implementation in `fret-ui-kit::imui` without widening `fret-imui`, runtime, renderer, or
  retained path contracts.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for channel
splitting, hit-test-aware debug interaction, reusable draw-list command metadata, image loading
recipes, or concave path fill parity.
