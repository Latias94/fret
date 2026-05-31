# Material3 Time Family Token Fallback v1 - TODO

Status legend: `[ ]` open, `[~]` in progress, `[x]` done.

## M3TTF1-010 - Baseline

- [x] Read the latest Material3 token inventory.
- [x] Identify time-family period selector fallback duplication.
- [x] Confirm no core mechanism changes are needed.

## M3TTF1-020 - Shared Helper

- [x] Add a time-family period selector token helper.
- [x] Migrate TimePicker period selector token access to the helper.
- [x] Migrate TimeInput period selector token access to the helper.
- [x] Preserve existing token module function names used by recipes and visual fixtures.

## M3TTF1-030 - Inventory And Tests

- [x] Teach the inventory script that the helper is shared token policy.
- [x] Generate a v1 inventory artifact for this lane.
- [x] Add focused tests for the shared period selector helper.

## M3TTF1-040 - Gates And Closeout

- [x] Run targeted Rust formatting, tests, check, and clippy.
- [x] Run workstream catalog and layering checks.
- [x] Write closeout audit.
- [x] Commit only this lane's changes.
