# ImUi Debug Draw Rounded Image v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Reference Decision

- Dear ImGui reference: `AddImageRounded`.
- Fret decision: expose rounded image clipping semantics above existing image scene ops, while
  leaving texture tint and arbitrary image quads as separate scene/renderer contract gaps.

## M1 - Implementation Slice

- Added full-image and image-region rounded command helpers.
- Added dedicated command variants to preserve command intent.
- Reused `SceneOp::PushClipRRect`, `SceneOp::Image`, and `SceneOp::ImageRegion`.
- Shared the `PathRect`-compatible rounding clamp with the path rectangle builder.

## M2 - Verification Slice

- Added unit coverage for command recording and corner clamp outcomes.
- Added public smoke compile coverage.
- Ran focused and full `fret-ui-kit --features imui` gates.

## M3 - Closeout

- Updated repo workstream indexes and the IMUI gap audit.
- Closed this lane as a narrow follow-on.
