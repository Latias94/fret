---
title: Shadcn Parity Discovery Second Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, navigation-menu, gallery-composition regression gate
---

# Second Sweep Audit

This audit records the navigation-menu components-open slice from the v2 sweep. It closes the
second overlay discovery turn with a fixture-backed mismatch report, a gallery-composition fix, and
a reusable suite gate.

## Objective Criteria

The slice required:

1. Capture the navigation-menu components-open layout sidecar from the UI Gallery docs demo.
2. Compare the open menu items against the upstream shadcn components-open DOM snapshot.
3. Classify and fix any confirmed mismatch.
4. Promote the confirmed finding into a reusable fixture and suite gate.

## Findings

### Navigation Menu components-open column widths

- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/navigation_menu_docs_demo_components_open_mismatch_report_v2_pre_fix.json`
  recorded 6 gallery-composition/app-demo mismatches and 1 critical top finding.
- Owner and layer:
  `gallery_composition` / `app_demo`.
- Symptom:
  the Components menu items were stretched to widths such as `561.333px`, `650px`, `579.333px`,
  and `814.667px` in Fret while the upstream goldens keep each item at `296px`.
- Root cause:
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs` only sized the outer demo
  row, leaving the two inner columns unconstrained.
- Fix:
  the docs demo now gives both columns explicit widths of `296px` at `lg` and `246px` at `md`.
- Post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/navigation_menu_docs_demo_components_open_mismatch_report_v2.json`
  records `1 pass_known`, `0 mismatch`, `0 top findings`.
- Suite report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`
  now includes the navigation-menu replay alongside the context-menu anchor.

### Harness gap closed

- `tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-hover-switch-and-escape.json`
  now captures the open layout sidecar and screenshot before dismissing the menu.
- The reusable fixture lives in
  `tools/parity-discovery/fixtures/navigation_menu_docs_demo_components_open_parts_v1.json`.

## Residual Follow-Ups

- Continue the sweep with hover-card, tooltip, dialog, and sheet.
- Add the first mobile or responsive v2 target after the overlay lane stays stable.
