# Material3 FAB Token Defaults v1 - TODO

Status legend: `[ ]` open, `[~]` in progress, `[x]` done.

## M3FTD1-010 - Baseline

- [x] Read the latest Material3 token inventory.
- [x] Identify `fab` as the highest magic visual constant module.
- [x] Confirm this is a token policy/default matrix cleanup, not a core mechanism change.

## M3FTD1-020 - Default Helper

- [x] Add a FAB token default helper module.
- [x] Move icon FAB size/icon/shape defaults into the helper.
- [x] Move extended FAB size/icon/shape/spacing/text-style defaults into the helper.
- [x] Move disabled opacity and state-layer fallback defaults into the helper.
- [x] Preserve existing `fab_tokens::*` function names.

## M3FTD1-030 - Inventory And Tests

- [x] Teach the inventory script that the helper is token policy helper code.
- [x] Generate a v1 inventory artifact for this lane.
- [x] Add focused helper tests for the default matrices.

## M3FTD1-040 - Gates And Closeout

- [x] Run targeted Rust formatting, tests, check, and clippy.
- [x] Run workstream catalog and layering checks.
- [x] Write closeout audit.
- [x] Commit only this lane's changes.
