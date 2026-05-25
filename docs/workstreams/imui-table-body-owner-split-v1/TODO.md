# IMUI Table Body Owner Split v1 - TODO

Status: Closed
Last updated: 2026-05-25

## TBO-010 - Boundary Decision

- [x] Create the active follow-on workstream docs.
- [x] Record that this is a private body/pinning/scroll owner split, not a public table API lane.
- [x] Keep table engines, sorting state, sizing persistence, virtualization, and runtime semantics
      out of scope.

## TBO-020 - Private Body Owner Split

- [x] Add `ecosystem/fret-ui-kit/src/imui/table_controls/body.rs`.
- [x] Move `PreparedTableCell`, pinned groups, row wrapping, center-scroll wrapping, and body cell
      wrapping into the private body owner.
- [x] Keep `table_controls.rs` as public table authoring and top-level render orchestration.
- [x] Add source-policy markers that prevent body/pinning/scroll implementation from drifting back
      into the parent table file.

## TBO-030 - Verification

- [x] Run focused `fret-ui-kit` table smoke gates.
- [x] Run focused `fret-imui` table interaction gates.
- [x] Run IMUI source-policy, catalog, JSON, format, and whitespace gates.
- [x] Record fresh evidence in `EVIDENCE_AND_GATES.md`.
