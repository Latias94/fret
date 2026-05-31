# Material3 TextInput Mechanism Test Ownership v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M1: Mechanism Ownership

Exit criteria:

- Editable TextInput `Event::TextInput` model-update behavior is covered under `fret-ui`.
- The Material3 residual interaction-regression binary is deleted.
- No Material3 tests remain in an ownerless residual file.

Status: Complete.

## M2: Boundary Hygiene

Exit criteria:

- `fret-ui-material3` no longer hosts raw `TextInputProps` mechanism coverage.
- `fret-ui` owns the primitive TextInput event contract.
- No shared harness or public API is widened.

Status: Complete.

## M3: Closeout Evidence

Exit criteria:

- Focused `fret-ui` nextest gate passes.
- `fret-ui` and `fret-ui-material3` check/clippy gates pass.
- Workstream catalog, layering, JSON, and diff hygiene gates pass.

Status: Complete.
