# Material 3 Switch Diagnostics Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

- [x] M3SWDIAG-010 [owner=codex] [scope=docs/workstreams/material3-parity-harness-fearless-refactor-v1]
  Goal: Re-read the existing Switch adapter report before changing the matrix.
  Review: DONE. The report has five `pass_known` parts, zero mismatches, and no mechanism findings.
  Evidence: `material3_switch_adapter_report_v1.json`.

- [x] M3SWDIAG-020 [owner=codex] [deps=M3SWDIAG-010] [scope=tools/diag-scripts/ui-gallery/material3]
  Goal: Run a fresh promoted gallery diagnostic over Switch icon states.
  Review: DONE. The icons state-matrix screenshots script passed and exposed stable switch ids for
  default, disabled, icons-both, and selected-icon-only variants.
  Evidence: diag run `1779937207775`.

- [x] M3SWDIAG-030 [owner=codex] [deps=M3SWDIAG-020] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Close Switch in the matrix with explicit layer classification.
  Review: DONE. Switch remains recipe/foundation owned; no kit-policy or mechanism gap was found.
