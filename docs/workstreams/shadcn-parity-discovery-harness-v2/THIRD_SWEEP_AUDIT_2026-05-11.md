---
title: Shadcn Parity Discovery Third Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, sheet mobile, responsive regression gate
---

# Third Sweep Audit

This audit records the first mobile/responsive slice in the v2 sweep: the official Sheet demo
opened in a constrained `375x240` viewport.

## Objective Criteria

The slice required:

1. Capture the Sheet docs demo in the same effective viewport as the upstream web golden.
2. Compare shell, body, field, and footer geometry against upstream shadcn `new-york-v4`.
3. Classify every non-passing result by layer.
4. Fix the highest-impact confirmed issues.
5. Promote the slice into the v2 suite as a reusable regression gate.

## Findings

### Sheet shell mechanism passed

- The runner produced an effective `375x240` layout root.
- The right-side SheetContent shell matched upstream within tolerance:
  width `281.333px` vs upstream `281.5px`, height `240px`, x offset `94px`.
- Owner and layer:
  `mechanism_core` / `mechanism`.

### Footer button width mismatch

- Pre-fix symptom:
  the Save button measured `115.333px` wide while upstream measured `248.833px`.
- Owner and layer:
  `component_recipe` / `recipe`.
- Root cause:
  `SheetFooter` did not carry `w-full min-w-0` onto its outer and inner stack surfaces, so
  stretchable children could still shrink-wrap.
- Fix:
  `ecosystem/fret-ui-shadcn/src/sheet.rs` now gives `SheetFooter` full-width/min-width-zero
  constraints and a full-width inner stack.

### Field/body vertical mismatch

- Pre-fix symptom:
  the input-to-input delta was `91.333px` while upstream was `86px`.
- Owner and layer:
  `gallery_composition` / `app_demo`.
- Root cause:
  the gallery snippet used `FieldSet` / `Field`, but upstream `sheet-demo.tsx` uses plain
  `Label`/`Input` groups with `grid gap-3` inside a body with `grid flex-1 auto-rows-min gap-6 px-4`.
- Fix:
  `apps/fret-ui-gallery/src/ui/snippets/sheet/demo.rs` now uses plain `Label`/`Input` vertical
  stacks and exposes stable `ui-gallery-sheet-demo-header` and `ui-gallery-sheet-demo-body`
  selectors.

### Harness blind spot found and closed

- The first post-fix report showed footer width passing while footer vertical placement was still
  wrong because the fixture only checked button width/height/gap.
- Root cause:
  the body area could collapse under the Fret flex translation without failing the old footer
  predicates.
- Fix:
  `tools/parity-discovery/fixtures/sheet_demo_vp375x240_open_parts_v1.json` now checks body
  top/height, input offsets, save-button top, save-button width/height, and footer button gap.

### Title metrics mismatch

- Pre-fix symptom:
  Fret header height was `98px` while upstream header height was `102px`.
- Owner and layer:
  `component_recipe` / `recipe`.
- Root cause:
  upstream `SheetTitle` renders as an `h2`, inheriting a `16px / 24px` line box; Fret used the
  generic `14px / 20px` font tokens and an extra tracking-tight override.
- Fix:
  `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` now defines sheet title/description metrics, and
  `SheetTitle` no longer applies the extra letter-spacing override.

## Evidence

- Diag script:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-sheet-demo-vp375x240-open-layout.json`
- Post-fix sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-sheet-vp375x240-post-fix-2/sessions/1778480231651-151488/1778480239588-ui-gallery-sheet-demo-vp375x240-open.layout/layout.taffy.v1.json`
- Post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/sheet_demo_vp375x240_open_mismatch_report_v2.json`
- Suite report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`

## Gate Result

- Sheet report:
  `4 pass_known`, `0 mismatch`, `0 blocked`.
- V2 suite:
  `4 reports`, `7 parts`, `7 pass_known`, `0 top findings`.

## Residual Follow-Ups

- The small-viewport script uses semantic `activate` because pointer `click_stable` hit-testing can
  land on the preview wrapper at this viewport. Track this as diagnostics-surface friction rather
  than a Sheet geometry mismatch.
- Continue the v2 sweep with Tooltip or Dialog to cover additional overlay state machines.
