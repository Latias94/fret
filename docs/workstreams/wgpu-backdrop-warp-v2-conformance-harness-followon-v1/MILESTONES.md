# WGPU Backdrop Warp V2 Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Backlog Baseline

Exit criteria:

- The duplicated local helpers are identified in the BackdropWarpV2 conformance test.
- The helper shape is proven compatible with `tests/support::render_scene_rgba8`.
- The local warp-map registration helper is explicitly retained.

Status: Complete.

## M1 — Migration

Exit criteria:

- `effect_backdrop_warp_v2_conformance.rs` uses shared `pixel_rgba` and `render_scene_rgba8`.
- Local readback helper copies and stale imports are removed from the named test.
- Warp-map image registration remains local to the test.

Status: Complete.

## M2 — Verification And Closeout

Exit criteria:

- Affected conformance test passes.
- `fret-render-wgpu` tests compile.
- Layering and workstream catalog checks pass.
- Closeout evidence is recorded.

Status: Complete.
