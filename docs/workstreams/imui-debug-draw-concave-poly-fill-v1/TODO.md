# ImUi Debug Draw Concave Poly Fill v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add `ImUiDebugDrawList::add_concave_poly_filled`.
- [x] Add `ImUiDebugDrawPath::fill_concave`.
- [x] Add a dedicated `ConcavePolyFilled` command.
- [x] Lower concave fills through the existing closed Canvas fill path.

## Verification

- [x] Add source-level unit coverage for direct commands, path finisher behavior, path clearing for
  invalid point counts, and closed path lowering.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
