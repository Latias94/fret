# IMUI Table Header Owner Split v1 - TODO

Status: Closed
Last updated: 2026-05-25

## THO-010 - Boundary Decision

- [x] Create the active workstream docs.
- [x] Record that this is a private owner split, not a public table API lane.
- [x] Keep table engines, sorting state, sizing persistence, and runtime semantics out of scope.

## THO-020 - Private Header Owner Split

- [x] Add `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs`.
- [x] Move visible-label parsing, sortable/plain header cells, sort indicator visuals, trigger
      response assembly, and resize handle behavior into the private header owner.
- [x] Keep `table_controls.rs` as the public table authoring and body assembly owner.
- [x] Add source-policy markers that prevent the header implementation from drifting back into the
      parent table file.

## THO-030 - Verification

- [x] Run focused `fret-ui-kit` table smoke gates.
- [x] Run focused `fret-imui` table interaction gates.
- [x] Run IMUI source-policy, catalog, JSON, format, and whitespace gates.
- [x] Record fresh evidence in `EVIDENCE_AND_GATES.md`.
