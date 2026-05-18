# WGPU Test Support Dead Code Prune v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Allowance Removed

Exit criteria:

- `#[allow(dead_code)]` is removed from `tests/support/mod.rs`.
- Readback-only and explicit-format tests import narrower support modules.
- Default scene-render tests keep the existing facade.

Status: Complete on 2026-05-18.

## M1 - Test Support Entrypoints Verified

Exit criteria:

- `fret-render-wgpu` test targets compile.
- Dead-code scan returns no hits in `crates/fret-render-wgpu/src` or `crates/fret-render-wgpu/tests`.
- Representative nextest gates pass for default scene, explicit format, and readback-only helpers.

Status: Complete on 2026-05-18.

## M2 - Workstream Closed

Exit criteria:

- Workstream catalog, JSON validation, and diff whitespace checks pass.
- Closeout note records the integration-test compilation model that caused the original allowance.

Status: Complete on 2026-05-18.
