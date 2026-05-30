# Material 3 Segmented Button Diagnostics Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

- [x] M3SBPDIAG-010 [owner=codex] [scope=tools/diag-scripts/ui-gallery/material3]
  Goal: Promote the existing Material3 SegmentedButton roving-semantics gallery script into a
  dedicated diagnostics suite.
  Review: DONE. The suite manifest now promotes the screenshot script through the diagnostics
  registry.

- [x] M3SBPDIAG-020 [owner=codex] [deps=M3SBPDIAG-010]
  Goal: Run the roving-semantics diagnostics and focused segmented-button Rust gates.
  Review: DONE. The diagnostics run passed, and the focused semantics/headless gates passed.

- [x] M3SBPDIAG-030 [owner=codex] [deps=M3SBPDIAG-020] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Close SegmentedButtonSet in the component matrix.
  Review: DONE. The row now records diagnostics alignment and keeps the existing recipe/foundation
  boundary.
