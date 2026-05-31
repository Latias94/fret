# Material3 Test Harness Support Ownership v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M1: Support Ownership

Exit criteria:

- `interaction_harness.rs` lives under `tests/support`.
- `support/goldens.rs` imports signature helpers through support-local paths.
- Test binaries no longer declare top-level `mod interaction_harness;`.

Status: Complete.

## M2: Import Hygiene

Exit criteria:

- Direct signature-helper imports use `support::interaction_harness`.
- `cargo check -p fret-ui-material3 --features diagnostics --tests` passes.
- No production code or public API changes are introduced.

Status: Complete.

## M3: Closeout Evidence

Exit criteria:

- Focused harness-user nextest gates pass.
- Material3 package check/clippy gates pass.
- Workstream catalog, layering, JSON, and diff hygiene gates pass.

Status: Complete.
