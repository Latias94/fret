# IMUI Table Body Owner Split v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Split Chosen

Exit criteria:

- The closed/complete header owner split is not widened.
- Body/pinning/scroll rendering is identified as the next private owner seam.
- Existing table behavior lanes remain closed.

## M1 - Body Owner Extracted

Exit criteria:

- [x] `table_controls/body.rs` owns body/header row wrapping, pinned groups, horizontal scroll wrapping,
  and cell wrapper rendering.
- [x] `table_controls.rs` delegates body row and header row group assembly to the private body owner.
- [x] No public IMUI table names or response shapes change.

## M2 - Behavior Proved Unchanged

Exit criteria:

- [x] Focused `fret-ui-kit` and `fret-imui` table gates pass.
- [x] Source-policy gate freezes the private body owner split.
- [x] Catalog, JSON, formatting, and whitespace checks pass.
