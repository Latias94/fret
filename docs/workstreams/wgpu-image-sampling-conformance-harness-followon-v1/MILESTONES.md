# WGPU Image Sampling Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Backlog Baseline

Exit criteria:

- The duplicated local readback and pixel helpers are identified in the image sampling conformance
  test.
- Explicit render-target setup is identified as intentionally test-owned.

Status: Complete.

## M1 — Migration

Exit criteria:

- `image_sampling_hint_conformance.rs` uses shared `read_texture_rgba8` and `pixel_rgba`.
- Local helper copies and stale imports are removed from the named test.
- Explicit render-target setup remains local to the test.

Status: Complete.

## M2 — Verification And Closeout

Exit criteria:

- Affected conformance test passes.
- `fret-render-wgpu` tests compile.
- Layering and workstream catalog checks pass.
- Closeout evidence is recorded.

Status: Complete.
