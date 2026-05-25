# IMUI Table Body Owner Split v1 - Handoff

Status: Closed
Last updated: 2026-05-25

Current slice: closed on 2026-05-25.

TBO-010 status: complete. This lane is a private `fret-ui-kit::imui` table body/pinning/scroll
owner split, not a public table API lane.

TBO-020 status: complete. `table_controls/body.rs` now owns
`PreparedTableCell`, row wrapping, pinned row grouping, center horizontal scroll wrapping, and cell
wrapper rendering. `table_controls.rs` delegates to `body::{PreparedTableCell, wrap_table_row,
wrap_table_cell}` while keeping public table authoring and top-level response orchestration.

TBO-030 status: complete. Focused `fret-ui-kit` table smoke tests, focused `fret-imui` table
interaction tests, source-policy, catalog, JSON, format, and whitespace gates passed on 2026-05-25.
`git diff --check` reported only existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

Closeout:

1. This lane is implementation-complete and closed.
2. Keep shared cell geometry helpers in the parent unless a later proof justifies `cell.rs`.
3. Continue table follow-ons from fresh proof pressure rather than expanding this lane.
