# Material3 Test Harness Support Ownership v1 Design

Status: Closed
Last updated: 2026-05-31

## Problem

Material3 test binaries repeatedly declared `mod interaction_harness;` at the test crate root even
when they only needed shared support helpers. The root cause was that `support/goldens.rs` imported
signature helpers through `crate::interaction_harness`, making the helper a top-level module
contract instead of support-owned infrastructure.

That pattern became more visible after interaction tests were split into family-owned binaries.

## Decision

Move the signature helper module from `tests/interaction_harness.rs` to
`tests/support/interaction_harness.rs`.

Expose it through `support::interaction_harness` and update direct imports to use that support-owned
path. Delete repeated top-level `mod interaction_harness;` declarations from Material3 test
binaries.

## Boundaries

- This lane changes test harness ownership only.
- No Material3 production API, component behavior, token, or runtime contract changes are
  introduced.
- Shared test support remains local to `fret-ui-material3`; it is not promoted to a public crate.

## Non-Goals

- Do not rewrite golden serialization.
- Do not consolidate unrelated `FakeUiServices` or layout helpers.
- Do not change the family test split created by previous lanes.

## Follow-On Shape

Future harness cleanup should only happen when repeated support modules create compile or review
friction. Do not introduce a broader test prelude until there are multiple support seams that need a
single owner.
