# WGPU Stroke Dash Shadow Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Stroke, Dash, And Shadow Test Harness Migration

Status: Done on 2026-05-18

Exit criteria:

- `dashed_border_conformance.rs` uses shared support for final readback and pixel sampling.
- `dash_semantics_rrect_vs_path_conformance.rs` uses shared support for final readback and pixel
  sampling.
- `stroke_paint_conformance.rs` uses shared support for final readback and pixel sampling.
- `shadow_rrect_conformance.rs` uses shared support for final readback and pixel sampling.
- Local duplicated final-render helpers are deleted from the named files.

## M1 — Verification And Closeout

Status: Done on 2026-05-18

Exit criteria:

- Grouped stroke/dash/shadow conformance tests pass.
- Backend test compile gate passes.
- Layering, workstream catalog, and diff whitespace gates pass.
- Lane docs are updated and closed.
