# IMUI Table Header Owner Split v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Split Chosen

Exit criteria:

- EWG-070 identifies table header behavior as the next high-payoff private owner split.
- Existing closed table lanes remain closed; this lane owns only private structure.

## M1 - Header Owner Extracted

Exit criteria:

- `table_controls/header.rs` owns header trigger, sort visual, visible-label, and resize handle
  implementation details.
- `table_controls.rs` delegates to the private header owner and no longer defines the moved
  implementation functions.

## M2 - Behavior Proved Unchanged

Exit criteria:

- Focused `fret-ui-kit` and `fret-imui` table gates pass.
- Source-policy gate freezes the private owner split.
- Catalog, JSON, formatting, and whitespace checks pass.
