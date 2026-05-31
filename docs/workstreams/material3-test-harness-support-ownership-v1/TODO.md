# Material3 Test Harness Support Ownership v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3THS-*`.

## Tasks

- [x] M3THS-010: Audit interaction harness declarations.
  - Scope: `ecosystem/fret-ui-material3/tests/*.rs` and `tests/support`.
  - Result: repeated top-level `mod interaction_harness;` declarations were caused by support
    goldens depending on `crate::interaction_harness`.

- [x] M3THS-020: Move harness ownership into support.
  - Scope: `tests/interaction_harness.rs` to `tests/support/interaction_harness.rs`.
  - Result: signature helpers are now support-owned.

- [x] M3THS-030: Update imports and module declarations.
  - Scope: Material3 test binaries.
  - Result: direct users import `support::interaction_harness::*`; redundant top-level module
    declarations were removed.

- [x] M3THS-040: Verify and close.
  - Scope: focused nextest gates, Material3 check/clippy, catalog, layering, and diff hygiene.
  - Expected result: committed harness ownership cleanup with clean worktree.

## Notes

- `text_field_hover.rs` now declares `mod support;` because it directly uses the signature helpers
  without the broader Material3 support harness.
