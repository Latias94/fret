# Material 3 SearchView State Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3SV-*`.

## M0 - Source And Current-State Packet

- [x] M3SV-010 [owner=codex] [deps=none] [scope=repo-ref/compose-multiplatform-core,ecosystem/fret-ui-material3/src/search_view.rs,docs/workstreams/material3-search-view-state-packet-v1]
  Goal: Record Compose SearchBar/SearchBarState outcomes and classify current Fret gaps by layer.
  Validation: packet artifact with source anchors, target truths, and first red/green gate plan.
  Review: DONE. Compose source establishes separate query/state, docked/full-screen expanded
  presentations, and back collapse; Fret maps this to Material recipe presentation plus existing
  kit overlay policy.
  Evidence: `artifacts/search_view_source_packet_v1.md`.
  Handoff: Do not implement predictive back in this lane.

## M1 - Full-Screen Presentation Slice

- [x] M3SV-020 [owner=codex] [deps=M3SV-010] [scope=ecosystem/fret-ui-material3/src/search_view.rs,ecosystem/fret-ui-material3/tests]
  Goal: Add an explicit full-screen SearchView presentation mode while preserving docked defaults.
  Validation: focused Rust test proves full-screen overlay opens, exposes stable ids, and closes on
  Escape through overlay policy.
  Review: DONE. Added `SearchViewPresentation::{Docked, FullScreen}` with Docked default and a
  modal full-screen path.
  Evidence: `ecosystem/fret-ui-material3/tests/search_view_behavior.rs`.
  Handoff: Keep `open` and `query` model ownership unchanged.

## M2 - Focus And Semantics

- [x] M3SV-030 [owner=codex] [deps=M3SV-020] [scope=ecosystem/fret-ui-material3/src/search_view.rs,ecosystem/fret-ui-material3/tests]
  Goal: Prove full-screen input focus routing and modal focus containment without duplicating root
  test ids.
  Validation: focused semantics/focus test plus automation-surface update if new part ids are
  public.
  Review: DONE_WITH_KNOWN_FOLLOW_ONS. Full-screen focuses the overlay-local header input and exposes
  `*.overlay.header*` ids; close autofocus is intentionally prevented to avoid immediate reopen from
  the collapsed underlay input focus-gained policy.
  Evidence: `search_view_full_screen_uses_modal_overlay_and_closes_on_escape`;
  `material3_search_view_exposes_stable_part_test_ids`.
  Handoff: Split a generic platform-back lane only if Escape is insufficient proof.

## M3 - Gallery Diagnostic And Golden Guard

- [x] M3SV-040 [owner=codex] [deps=M3SV-020,M3SV-030] [scope=tools/diag-scripts/ui-gallery/material3,apps/fret-ui-gallery,ecosystem/fret-ui-material3/tests/radio_alignment.rs]
  Goal: Add or update the smallest gallery diagnostic/golden coverage for docked and full-screen
  SearchView presentation.
  Validation: focused diag or headless golden gate, with stable `test_id` selectors.
  Review: DONE. The SearchView headless suite now includes `full_screen_open`; no separate gallery
  diag was needed for the first slice.
  Evidence: `material3_headless_search_view_suite_goldens_v1`;
  `goldens/material3-headless/v1/material3-search-view.*.json`.
  Handoff: Do not broaden into all SearchBar top-app-bar behavior.

## M4 - Closeout

- [x] M3SV-050 [owner=codex] [deps=M3SV-030,M3SV-040] [scope=docs/workstreams/material3-search-view-state-packet-v1]
  Goal: Close the lane with fresh gates and split platform/predictive-back follow-ons.
  Validation: JSON/catalog, targeted Rust gates, diag/golden gates, and diff audit.
  Review: DONE. Focused Rust, headless golden, check, clippy, JSON, and catalog gates pass.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to the broader Material3 goal.
