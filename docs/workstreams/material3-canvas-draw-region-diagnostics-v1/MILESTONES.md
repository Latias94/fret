# Material 3 Canvas Draw Region Diagnostics v1 - Milestones

Status: Closed
Last updated: 2026-05-28

## M0 - Classification

Exit criteria:

- ProgressIndicator and Slider draw surfaces are mapped.
- Existing diagnostics mechanisms are checked before code edits.
- Owner classification distinguishes mechanism, foundation, recipe, and script work.

## M1 - Foundation Helper

Exit criteria:

- A shared hidden diagnostic-anchor helper exists, or the lane records why per-component anchors are
  safer.
- The helper has a focused selector/bounds proof.
- It does not introduce visible paint, focus stops, or user-facing accessibility names.

## M2 - ProgressIndicator

Exit criteria:

- Linear progress rectangular regions are anchored or explicitly rejected.
- Circular/indeterminate regions have scene/golden-only evidence when exact anchors would be false.
- Progress headless goldens pass.

## M3 - Slider

Exit criteria:

- Slider and range slider expose stable anchors for deterministic rectangular painted regions.
- Existing slider semantics and keyboard behavior still pass.
- Optional UI Gallery diagnostics script uses stable selectors rather than coordinates.

## M4 - Closeout

Exit criteria:

- Targeted Rust gates are fresh and recorded.
- Workstream JSON/catalog checks pass.
- Any exact named scene-op diagnostics work is split into a mechanism follow-on.
- The lane status is closed or clearly handed off.

Closeout note: completed on 2026-05-28. The lane closed with recipe-level rectangular anchors and
left exact named `SceneOp` diagnostics as a future mechanism follow-on, not Material3 policy.
