# WGPU Paint Eval Space Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Backlog Baseline

Exit criteria:

- The duplicated local helpers are identified in the paint evaluation-space conformance tests.
- The helper shape is proven compatible with `tests/support::render_scene_rgba8`.

Status: Complete.

## M1 — Migration

Exit criteria:

- `paint_eval_space_stroke_s01_conformance.rs` uses shared `pixel_rgba` and
  `render_scene_rgba8`.
- `paint_eval_space_viewport_conformance.rs` uses shared `pixel_rgba` and `render_scene_rgba8`.
- Local helper copies and stale imports are removed from the named tests.

Status: Complete.

## M2 — Verification And Closeout

Exit criteria:

- Affected conformance tests pass.
- `fret-render-wgpu` tests compile.
- Layering and workstream catalog checks pass.
- Closeout evidence is recorded.

Status: Complete.
