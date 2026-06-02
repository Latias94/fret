# Material3 TextInput Mechanism Test Ownership v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3TIM-*`.

## Tasks

- [x] M3TIM-010: Audit TextInput ownership.
  - Scope: `material3_interaction_regressions.rs` and `fret-ui` TextInput interaction tests.
  - Result: residual test is mechanism-level coverage, not Material3 component coverage.

- [x] M3TIM-020: Add mechanism-level coverage.
  - Scope: `crates/fret-ui/src/declarative/tests/interactions/text_input.rs`.
  - Result: editable TextInput `Event::TextInput` model-update behavior is covered in `fret-ui`.

- [x] M3TIM-030: Delete Material3 residual binary.
  - Scope: `ecosystem/fret-ui-material3/tests/material3_interaction_regressions.rs`.
  - Result: deleted after all Material3-owned tests were split into purpose-owned binaries.

- [x] M3TIM-040: Verify and close.
  - Scope: focused nextest gates, fret-ui/material3 checks and clippy, catalog, layering, and diff
    hygiene.
  - Expected result: committed ownership cleanup with clean worktree.

## Notes

- This is a boundary cleanup, not a TextInput behavior change.
