# WGPU Conformance Harness v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Path Batch Migrated

Exit criteria:

- `tests/support/mod.rs` provides the shared WGPU readback/render helpers.
- Four path-related conformance tests use it.
- The migrated tests preserve existing pixel assertions and skip-on-no-adapter behavior.

Status: Met on 2026-05-18.

## M1 — Verified And Closed

Exit criteria:

- Path conformance batch passes.
- `cargo check -p fret-render-wgpu --locked --tests -j 1` passes.
- Workstream catalog and diff checks pass.
- The lane either closes or explicitly splits a narrower follow-on.

Status: Met on 2026-05-18.
