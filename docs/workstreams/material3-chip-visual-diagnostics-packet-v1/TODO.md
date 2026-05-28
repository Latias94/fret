# Material 3 Chip Visual Diagnostics Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

- [x] M3CHIPVIS-010 [owner=codex] [scope=tools/diag-scripts/ui-gallery/material3]
  Goal: Add focused chip visual chrome diagnostics for the State Matrix page.
  Review: DONE. The new script starts at `material3_state_matrix`, scrolls representative chip
  rows into view, asserts root/chrome center alignment, checks the minimum target on a representative
  chip, and proves trailing-icon selectors exist.
  Evidence: `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json`.

- [x] M3CHIPVIS-020 [owner=codex] [deps=M3CHIPVIS-010]
  Goal: Run the gallery diagnostics and focused Rust gates.
  Review: DONE. The diagnostics script passed; automation-surface and chip semantics/roving tests
  passed.
  Evidence: diag run `1779936147211`, `automation_surface`, and `radio_alignment` focused gates.

- [x] M3CHIPVIS-030 [owner=codex] [deps=M3CHIPVIS-020] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Close chip visual follow-ons in the component matrix.
  Review: DONE. Chip, FilterChip, InputChip, and SuggestionChip rows now record visual diagnostics
  alignment and no new kit-policy or mechanism gap.
