# WGPU Composite Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Composite Test Harness Migration

Status: Done on 2026-05-18

Exit criteria:

- `tests/support::render_scene_rgba8_with_format` lets integration tests choose an output
  `wgpu::TextureFormat`.
- `tests/support::render_scene_rgba8` remains the default `Rgba8Unorm` wrapper for existing tests.
- `composite_group_conformance.rs` uses shared support for final readback and pixel sampling.
- The composite test's `Rgba8UnormSrgb` output format remains explicit at the call site.

## M1 — Verification And Closeout

Status: Done on 2026-05-18

Exit criteria:

- Composite-group conformance tests pass.
- Backend test compile gate passes.
- Layering, workstream catalog, and diff whitespace gates pass.
- Lane docs are updated and closed.
