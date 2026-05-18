# WGPU Mask Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Mask Test Harness Migration

Status: Done on 2026-05-18

Exit criteria:

- `mask_gradient_conformance.rs` uses `support::{pixel_rgba, render_scene_rgba8}` for final
  readback.
- `mask_image_conformance.rs` uses `support::{pixel_rgba, render_scene_rgba8}` for final readback.
- Local duplicated final-render readback helpers are deleted from both files.

## M1 — Verification And Closeout

Status: Done on 2026-05-18

Exit criteria:

- Mask conformance tests pass.
- Backend test compile gate passes.
- Layering, workstream catalog, and diff whitespace gates pass.
- Lane docs are updated and closed.
