# Material3 Token Fallback Hardening v2 - TODO

Status legend: `[ ]` open, `[~]` in progress, `[x]` done.

## M3TFH2-010 - Inventory Baseline

- [x] Read the v1 token visual matrix inventory.
- [x] Identify the highest-return family slice that has multiple real consumers.
- [x] Choose chip token family as the first production hardening slice.

## M3TFH2-020 - Chip Shared Token Helper

- [x] Add a chip-family shared helper module.
- [x] Migrate AssistChip token access to the helper.
- [x] Migrate FilterChip token access to the helper.
- [x] Migrate InputChip token access to the helper.
- [x] Migrate SuggestionChip token access to the helper.
- [x] Preserve existing token module function names consumed by recipes and fixtures.

## M3TFH2-030 - Inventory And Evidence

- [x] Teach the inventory script that the new helper is a shared token helper.
- [x] Generate a v2 inventory artifact under this workstream.
- [x] Record before/after fallback and magic-constant counts for the chip family.

## M3TFH2-040 - Gates And Closeout

- [x] Run focused Rust formatting, tests, check, and clippy.
- [x] Run workstream catalog and layering checks.
- [x] Write closeout audit.
- [x] Commit only this lane's changes.
