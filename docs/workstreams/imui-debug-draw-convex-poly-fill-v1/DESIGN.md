# ImUi Debug Draw Convex Poly Fill v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane adds a Dear ImGui `AddConvexPolyFilled`-style helper to the canvas-backed IMUI debug-draw
surface. The API treats convexity as a caller contract, matching the Dear ImGui naming, and lowers
the point list to a closed fill path.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw command-list API.
- `crates/fret-core` owns the vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui` remains unchanged.

## Must-Be-True Outcomes

- Callers can emit a filled convex polygon from an ordered point list.
- Fewer than three points do not emit paint.
- Filled polygons reuse the existing Canvas path fill path.
- The helper does not add a triangulation, tessellation, or hit-testing engine.

## Non-Goals

- No generic concave polygon filling contract.
- No automatic convexity validation.
- No retained path builder API.
- No draw-list channel splitting.
- No per-command hit-testing.
