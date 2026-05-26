# IMUI Table Header Owner Split v1 - Handoff

Status: Closed
Last updated: 2026-05-25

Current slice: closed on 2026-05-25.

THO-010 status: complete. This lane is a private `fret-ui-kit::imui` table owner split, not a public
table API lane.

THO-020 status: complete. `table_controls/header.rs` now owns sortable/plain header cells,
visible-label parsing, trigger response assembly, sort indicator visuals, and resize handle
behavior.

THO-030 status: complete. Focused `fret-ui-kit` table smoke tests, focused `fret-imui` table
interaction tests, source-policy, catalog, JSON, format, and whitespace gates all pass.

Closeout:

1. This lane is closed with a closeout audit.
2. Continue table follow-ons from fresh proof pressure rather than expanding this lane.
