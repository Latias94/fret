# ImUi Debug Draw Ngon Primitives v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane adds Dear ImGui `AddNgon`- and `AddNgonFilled`-style helpers to the canvas-backed IMUI
debug-draw surface. Both helpers accept a center point, radius, and explicit segment count, then
lower valid inputs to a closed regular-polygon Canvas path.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw command-list API.
- `crates/fret-core` owns the vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui` remains unchanged.

## Must-Be-True Outcomes

- Callers can emit stroked regular polygons with an explicit segment count.
- Callers can emit filled regular polygons with an explicit segment count.
- Fewer than three segments and non-positive radii do not emit paint.
- Ngon helpers reuse the existing Canvas path stroke/fill path.
- The helper does not add a tessellation or hit-testing engine.

## Non-Goals

- No ellipse support in this lane.
- No automatic segment-count policy for circles.
- No retained path builder API.
- No draw-list channel splitting.
- No per-command hit-testing.
