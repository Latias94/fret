---
title: Shadcn Parity Discovery Fifth Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, dialog, app-demo composition harness
---

# Fifth Sweep Audit

This audit records the Dialog docs-demo open slice from the v2 sweep. The slice promoted Dialog
into the fixture-driven suite and closed a UI Gallery docs-demo composition drift.

## Objective Criteria

The slice required:

1. Capture the Dialog docs demo open state with layout sidecar, bundle, and screenshot evidence.
2. Compare stable shadcn `new-york-v4` geometry facts against the upstream DOM snapshot.
3. Classify every non-passing result by layer.
4. Fix the highest-confidence confirmed issue.
5. Promote the slice into the v2 suite as a reusable regression gate.

## Findings

### FieldSet/Field composition drift

- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/dialog_demo_open_mismatch_report_v2_pre_fix.json`
  recorded three critical app-demo findings.
- Symptom:
  Fret DialogContent measured `344.667px` tall while the upstream open DOM snapshot measured
  `323.333px`.
- Secondary symptoms:
  the name input was `8px` too low, the username input was `13.333px` too far below the name input,
  and the footer action row was `21.333px` too low.
- Owner and layer:
  `gallery_composition` / `app_demo`.
- Root cause:
  the UI Gallery Dialog demo used `FieldSet` / `Field`, which brings field policy spacing. The
  upstream shadcn docs demo uses plain `Label` / `Input` stacks: outer `grid gap-4` and per-field
  `grid gap-3`.
- Fix:
  `apps/fret-ui-gallery/src/ui/snippets/dialog/demo.rs` now uses plain `Label` / `Input` stacks
  with `Space::N4` outer gap and `Space::N3` per-field gap. The demo also exposes stable
  `ui-gallery-dialog-demo-body` and `ui-gallery-dialog-demo-footer` test ids.

### Native text-metric tolerance

- After the composition fix, body height, username-input delta, and button geometry matched the
  upstream docs-demo structure.
- Residual vertical deltas were `2.667px`, caused by native text metrics and 1.5 scale-factor
  rounding in the header/title stack.
- The fixture tolerates this with a `3px` vertical whole-surface tolerance while the pre-fix
  `8-21px` app-demo drift still fails.

## Evidence

- Diag script:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-docs-demo-open-screenshot.json`
- Pre-fix sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-dialog-docs-demo-open-layout/sessions/1778482156610-166220/1778482166183-ui-gallery-dialog-docs-demo-open-desktop.layout/layout.taffy.v1.json`
- Post-fix sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-dialog-docs-demo-open-layout-post-fix/sessions/1778483112449-183236/1778483119711-ui-gallery-dialog-docs-demo-open-desktop.layout/layout.taffy.v1.json`
- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/dialog_demo_open_mismatch_report_v2_pre_fix.json`
- Post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/dialog_demo_open_mismatch_report_v2.json`
- Fixture:
  `tools/parity-discovery/fixtures/dialog_demo_open_parts_v1.json`
- Suite:
  `tools/parity-discovery/suites/shadcn_parity_discovery_v2.json`

## Gate Result

- Dialog pre-fix report:
  `3 mismatch`, all classified as `app_demo`.
- Dialog post-fix report:
  `3 pass_known`, `0 mismatch`, `0 blocked`, `0 top findings`.
- V2 suite:
  `6 reports`, `13 parts`, `13 pass_known`, `0 top findings`.

## Residual Follow-Ups

- Add a separate Dialog focus/escape/close behavior slice if the next sweep wants modal policy
  coverage. This slice intentionally gates docs-demo geometry, not focus-trap behavior.
- If future native/web text metric work narrows the header delta, the fixture tolerance can be
  tightened back below `3px`.
