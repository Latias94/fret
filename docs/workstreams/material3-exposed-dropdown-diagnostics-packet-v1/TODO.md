# Material 3 Exposed Dropdown Diagnostics Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

- [x] M3EXDDIAG-010 [owner=codex] [scope=tools/diag-scripts/ui-gallery/material3]
  Goal: Promote the existing Material3 ExposedDropdown filtering script into a dedicated diagnostics
  suite.
  Review: DONE. The suite manifest now makes the filtering popup script reachable from the
  diagnostics registry while preserving the existing top-level redirect.

- [x] M3EXDDIAG-020 [owner=codex] [deps=M3EXDDIAG-010]
  Goal: Run filtering diagnostics and focused ExposedDropdown Rust gates.
  Review: DONE. The diagnostics run passed and the two state-model gates passed:
  blur synchronization and trailing icon overlay toggling.

- [x] M3EXDDIAG-030 [owner=codex] [deps=M3EXDDIAG-020] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Close ExposedDropdown in the component matrix.
  Review: DONE. The row now records diagnostics alignment and keeps the existing recipe/foundation
  boundary.
