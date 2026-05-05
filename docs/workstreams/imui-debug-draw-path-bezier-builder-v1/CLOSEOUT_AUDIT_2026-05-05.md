# ImUi Debug Draw Path Bezier Builder v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the scoped Dear ImGui `PathBezier*CurveTo` ergonomics follow-on above the
canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `ImUiDebugDrawPath::bezier_quadratic_curve_to`.
- Added `ImUiDebugDrawPath::bezier_cubic_curve_to`.
- Appended sampled points from the current path point, matching Dear ImGui's path-builder shape.
- Used a stable default segment count for `segments == 0`.
- Treated missing start points as no-op instead of panicking.
- Kept the implementation in `fret-ui-kit::imui` without widening `fret-imui`, runtime, renderer, or
  retained path contracts.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for elliptical
path arcs, rounded `PathRect` parity, channel splitting, hit-test-aware debug interaction, reusable
draw-list command metadata, or image loading recipes.
