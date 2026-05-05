# ImUi Debug Draw Concave Poly Fill v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane adds Dear ImGui `AddConcavePolyFilled` / `PathFillConcave`-style semantics to the
canvas-backed IMUI debug-draw helper. It records a dedicated concave-fill command so callers do not
have to mislabel simple concave polygons as convex, while lowering through the existing closed Canvas
fill path.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw facade command and scoped path finisher.
- `crates/fret-core` owns the fill rule and vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui`, runtime, and renderer crates remain unchanged.

## Must-Be-True Outcomes

- Callers can directly record a simple concave polygon fill.
- Callers can finish a scoped debug-draw path with concave fill semantics.
- Less than three points clears the scoped path and records no command.
- Lowering reuses `PathStyle::Fill(FillStyle::NonZero)` over a closed path.
- The lane does not add an IMUI triangulator, new renderer tessellation policy, or hit-test contract.

## Non-Goals

- No holes or self-intersection support contract.
- No Dear ImGui anti-aliased fringe reproduction in `fret-ui-kit`.
- No draw-list channel splitting.
- No per-command hit-testing.
- No renderer tessellation contract changes.
