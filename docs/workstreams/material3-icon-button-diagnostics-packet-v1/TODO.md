# Material 3 IconButton Diagnostics Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

- [x] M3IBDIAG-010 [owner=codex] [scope=tools/diag-scripts/ui-gallery/material3]
  Goal: Repair stale IconButton centered-chrome diagnostics navigation.
  Review: DONE. The script now searches for `icon button`, opens
  `ui-gallery-nav-material3-icon-button`, and waits for `ui-gallery-page-material3-icon-button`.

- [x] M3IBDIAG-020 [owner=codex] [deps=M3IBDIAG-010]
  Goal: Run centered-chrome diagnostics and focused Rust gates.
  Review: DONE. Diagnostics run `1779937783108` passed; automation-surface and pressed-scene gates
  passed.

- [x] M3IBDIAG-030 [owner=codex] [deps=M3IBDIAG-020] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Close IconButton in the component matrix.
  Review: DONE. IconButton now records centered-chrome diagnostics alignment and no kit-policy or
  mechanism gap.
