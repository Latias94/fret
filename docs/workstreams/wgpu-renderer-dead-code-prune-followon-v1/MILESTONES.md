# WGPU Renderer Dead Code Prune Follow-on v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Production Suppressions Removed

Exit criteria:

- Production `#[allow(dead_code)]` suppressions are gone from `crates/fret-render-wgpu/src`.
- Removed code has no runtime caller/reader.
- Stale suppressions are not replaced with new wrappers.

Status: Complete on 2026-05-18.

## M1 - Edited Areas Verified

Exit criteria:

- Backend test targets compile.
- Representative text and render-plan tests pass.
- Residual `dead_code` scan only reports test-only allowances.

Status: Complete on 2026-05-18.

## M2 - Workstream Closed

Exit criteria:

- Workstream catalog and JSON checks pass.
- Closeout note records the remaining non-production allowances.

Status: Complete on 2026-05-18.
