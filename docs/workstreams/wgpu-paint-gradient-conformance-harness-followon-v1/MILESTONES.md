# WGPU Paint Gradient Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Paint Gradient Test Harness Migration

Status: Done on 2026-05-18

Exit criteria:

- `paint_gradient_conformance.rs` uses `support::{pixel_rgba, render_scene_rgba8}` for final
  readback.
- Local duplicated final-render readback helpers are deleted from the file.
- `composite_group_conformance.rs` is untouched unless a format-aware support helper is explicitly
  split into a future lane.

## M1 — Verification And Closeout

Status: Done on 2026-05-18

Exit criteria:

- Paint-gradient conformance tests pass.
- Backend test compile gate passes.
- Layering, workstream catalog, and diff whitespace gates pass.
- Lane docs are updated and closed.
