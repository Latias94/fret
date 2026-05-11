---
title: Shadcn Parity Discovery First Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, context-menu, gallery-composition regression gate
---

# First Sweep Audit

This audit closes the first v2 overlay slice. It records the prompt-to-artifact closure for the
context-menu docs-demo open state, the supporting script change that enabled layout-sidecar
capture, and the resulting fix/gate.

## Objective Criteria

The slice required:

1. Capture an open-state context-menu layout sidecar from the UI Gallery docs demo.
2. Compare the open panel against the upstream shadcn open DOM snapshot.
3. Classify and fix any confirmed mismatch.
4. Promote the confirmed finding into a reusable fixture and suite gate.

## Findings

### Context Menu open width

- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/context_menu_demo_open_mismatch_report_v2_pre_fix.json`
  recorded 1 mismatch and 1 critical top finding.
- Owner and layer:
  `gallery_composition` / `app_demo`.
- Symptom:
  the open content width was `192.667px` in Fret while the upstream golden is `208px`.
- Root cause:
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/demo.rs` used `min_width(Px(192.0))`
  instead of the upstream `w-52` lane.
- Fix:
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/demo.rs` now uses `min_width(Px(208.0))`.
- Post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/context_menu_demo_open_mismatch_report_v2.json`
  records `1 pass_known`, `0 mismatch`, `0 top findings`.
- Suite report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`
  records `1 report`, `1 part`, `0 top findings`.

### Harness gap closed

- `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-docs-smoke.json` now captures
  the open layout sidecar, screenshot, and bundle before dismissing the menu.
- The reusable fixture lives in
  `tools/parity-discovery/fixtures/context_menu_demo_open_parts_v1.json`.
- The one-command suite lives in
  `tools/parity-discovery/suites/shadcn_parity_discovery_v2.json`.

## Residual Follow-Ups

- Add the next uncovered overlay surfaces, starting with hover-card, tooltip, dialog, and sheet.
- Add the first new mobile or responsive v2 target after the overlay lane stays stable.
