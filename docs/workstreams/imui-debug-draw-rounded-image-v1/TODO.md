# ImUi Debug Draw Rounded Image v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add `ImUiDebugDrawList::add_image_rounded`.
- [x] Add `ImUiDebugDrawList::add_image_rounded_with_options`.
- [x] Add `ImUiDebugDrawList::add_image_region_rounded`.
- [x] Add dedicated rounded image command variants.
- [x] Lower rounded image commands through `PushClipRRect` plus existing image scene ops.
- [x] Share the Dear ImGui `PathRect`-style rounding clamp with path rectangle sampling.

## Verification

- [x] Add source-level unit coverage for command recording and corner clamp behavior.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
