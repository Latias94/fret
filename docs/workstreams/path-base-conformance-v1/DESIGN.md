# Path Base Conformance v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

ADR 0080 now owns the base prepared vector path contract, while ADR 0277 and ADR 0278 own additive
stroke-style and path-paint extensions. The extension lanes are closed, but ADR 0080 still records a
base conformance gap:

- fill-rule behavior for intersecting same-winding overlap regions,
- conservative `PathMetrics.bounds`,
- and transformed path rendering under clipping.

This lane is intentionally not a new path API. It exists to turn the already accepted base contract
into runnable renderer gates so future renderer work cannot regress path semantics while refactoring
the WGPU backend.

## Assumptions First

- Confident: `fret-core` owns the portable path vocabulary and `fret-render-wgpu` owns default
  tessellation. Evidence: `crates/fret-core/src/vector_path.rs`,
  `crates/fret-render-wgpu/src/renderer/path.rs`. If wrong, these tests would need to live in a
  different backend-independent harness.
- Confident: ADR 0277 and ADR 0278 are additive and should remain closed. Evidence:
  `docs/adr/0277-path-stroke-style-v2.md`, `docs/adr/0278-path-paint-surface-v1.md`. If wrong, this
  lane would blur base path conformance with stroke/paint extension scope.
- Likely: the first executable proof should be a WGPU renderer conformance test plus a path module
  unit test for tessellation bounds. Evidence: existing gates in
  `crates/fret-render-wgpu/tests/path_stroke_style_v2_conformance.rs` and
  `crates/fret-render-wgpu/tests/path_paint_conformance.rs`. If wrong, a shared render-test helper
  may be needed first.
- Likely: rotated transform plus clip can be locked for `SceneOp::Path` directly because ADR 0078
  and existing affine clip tests already cover the stack semantics for quads/clips. Evidence:
  `crates/fret-render-wgpu/tests/affine_clip_conformance.rs`,
  `crates/fret-render-wgpu/tests/clip_path_conformance.rs`. If wrong, the slice should split a
  separate transform/clip gate follow-on.

## Target State

- `crates/fret-render-wgpu/tests/path_base_conformance.rs` locks visible path behavior for:
  - `FillRule::NonZero` versus `FillRule::EvenOdd` on intersecting same-winding overlap regions,
  - `SceneOp::Path` transformed under an active clip,
  - command robustness where practical for base path rendering.
- `crates/fret-render-wgpu/src/renderer/path.rs` has unit coverage proving reported
  `PathMetrics.bounds` contain tessellated vertices for representative fill/stroke styles.
- `docs/adr/0080-vector-path-contract.md` and `docs/adr/IMPLEMENTATION_ALIGNMENT.md` no longer
  claim the base fill-rule/bounds gap is only code-reviewed.
- Any remaining ADR 0080 gap is explicitly narrower than this lane.

## Out Of Scope

- New public path API.
- Reopening ADR 0277 stroke-style v2 or ADR 0278 path-paint lanes.
- Full SVG/path parity beyond the accepted ADR 0080 command vocabulary.
- Non-scaling strokes or transform-aware retessellation.
- Renderer architecture splits unrelated to path contract conformance.

## First Slice

`PBC-010`: add the base path conformance tests and update the workstream evidence. Only change
renderer behavior if the tests reveal a real contract violation.

## Closure

Closed on 2026-05-18 after the base WGPU conformance gates and ADR/alignment updates landed. Future
path work should start as narrower additive ADR/workstream slices rather than reopening this lane.
