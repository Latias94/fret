# ImUi Debug Draw Clip Metadata v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui exposes `ImDrawCmd::ClipRect` so renderers and tooling can inspect the effective clip
state for a draw command. Fret's IMUI debug draw surface already has explicit `push_clip_rect` /
`pop_clip_rect` commands, but the public command summaries did not include source-level effective
clip metadata. This lane adds that metadata without changing painting behavior or the scene
contract.

## Ownership

- `fret-ui-kit::imui` owns source-level debug draw command summaries and clip stack simulation.
- `crates/fret-core` and render backends stay unchanged.
- Hit-testing, backend scissor attribution, and renderer callbacks stay out of this lane.

## Must-Be-True Outcomes

- Each command summary reports the effective source-level clip rect, if any.
- Each command summary reports the clip stack depth after applying that command.
- List summary reports maximum clip depth and final clip depth.
- Active channel split summaries still use eventual merge order before simulating clip state.
- Public smoke coverage proves the new fields are reachable through the IMUI facade.

## Non-Goals

- No hit-test-aware clip policy.
- No renderer scissor/batch attribution.
- No changes to `CanvasPainter` clip behavior.
- No `ImDrawCallback` / user renderer callback surface.
