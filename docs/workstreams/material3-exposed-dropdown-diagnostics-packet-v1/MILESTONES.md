# Material 3 Exposed Dropdown Diagnostics Packet v1 - Milestones

Status: Closed
Last updated: 2026-05-28

## M1 - Dedicated Diagnostics Suite

Complete.

- Promoted `ui-gallery-material3-exposed-dropdown-filtering.json` into
  `tools/diag-scripts/suites/ui-gallery-material3-exposed-dropdown-filtering/suite.json`.
- Kept the existing root redirect for compatibility with older command snippets.

## M2 - Focused Verification

Complete.

- Filtering popup diagnostics passed against the Material3 UI Gallery.
- Focused Rust gates passed for blur synchronization and trailing icon popup toggling.

## M3 - Matrix Closeout

Complete.

- `exposed_dropdown` moved from `packet_done_known_follow_ons` to
  `packet_done_diagnostics_aligned`.
- The component remains recipe/foundation owned with no new kit or mechanism gap.
