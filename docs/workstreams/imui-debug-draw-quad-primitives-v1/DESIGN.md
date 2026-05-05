# ImUi Debug Draw Quad Primitives v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane adds Dear ImGui `AddQuad`- and `AddQuadFilled`-style helpers to the canvas-backed IMUI
debug-draw surface. Both helpers accept four caller-ordered points and lower them to a closed Canvas
path, using stroke policy for the outline helper and fill policy for the filled helper.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw command-list API.
- `crates/fret-core` owns the vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui` remains unchanged.

## Must-Be-True Outcomes

- Callers can emit stroked quads from four ordered points.
- Callers can emit filled quads from four ordered points.
- Quad helpers reuse the existing Canvas path stroke/fill path.
- The helper does not add triangulation, tessellation, or hit-testing behavior.

## Non-Goals

- No rounded quad contract.
- No automatic convexity or winding validation.
- No retained path builder API.
- No draw-list channel splitting.
- No per-command hit-testing.
