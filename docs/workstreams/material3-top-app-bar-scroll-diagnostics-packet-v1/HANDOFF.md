# Material 3 TopAppBar Scroll Diagnostics Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

TopAppBar scroll diagnostics are closed for the current Material3 gallery scenarios.

Return to the broader Material3 component matrix for the next concrete residual. Do not reopen this
packet unless the promoted scroll script starts failing or a new app/design-system consumer proves
that TopAppBar scroll behavior must move into shared kit policy.

## Current Source Of Truth

- `ecosystem/fret-ui-material3/src/top_app_bar.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-top-app-bar-scroll-screenshots.json`
- `docs/workstreams/material3-top-app-bar-scroll-diagnostics-packet-v1/artifacts/top_app_bar_scroll_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Known Follow-Ons

- Nested-scroll consumption and fling velocity remain out of scope.
- Shared scroll policy belongs in `fret-ui-kit` only if another design system needs the same
  behavior.
