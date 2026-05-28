# Material 3 Checkbox Gallery Diagnostics Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

- [x] M3CBGD-010 [owner=codex] [scope=tools/diag-scripts/ui-gallery/material3]
  Goal: Reproduce and classify the Checkbox centered-chrome diagnostics failure.
  Review: DONE. The script was stale because it navigated to the aggregate Material3 gallery page,
  which no longer exposes `ui-gallery-material3-checkbox`; the dedicated Checkbox page does.
  Evidence: failing run `1779934611432`.

- [x] M3CBGD-020 [owner=codex] [deps=M3CBGD-010] [scope=tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json]
  Goal: Repair the diagnostics script without changing component behavior.
  Review: DONE. The script now searches for and opens `ui-gallery-nav-material3-checkbox`, then
  waits for `ui-gallery-page-material3-checkbox`.
  Evidence: fixed run `1779935007417`.

- [x] M3CBGD-030 [owner=codex] [deps=M3CBGD-020] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Close the Checkbox matrix residual with focused gallery and unit evidence.
  Review: DONE. Centered chrome and tri-state diagnostics passed; matrix now records no kit-policy
  or mechanism gap.
  Evidence: fixed run `1779935007417`, tri-state run `1779935349931`, focused checkbox tests.
