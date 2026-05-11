---
title: Shadcn Parity Discovery Fourth Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, tooltip, cross-root geometry harness
---

# Fourth Sweep Audit

This audit records the Tooltip docs-demo open slice from the v2 sweep. The slice promoted Tooltip
into the fixture-driven suite and closed a harness attribution bug in cross-overlay/root geometry
comparison.

## Objective Criteria

The slice required:

1. Capture the Tooltip docs demo open state with layout sidecar, bundle, and screenshot evidence.
2. Compare stable shadcn `new-york-v4` geometry facts against the upstream DOM snapshot.
3. Classify every non-passing result by layer.
4. Fix the highest-impact confirmed issue.
5. Promote the slice into the v2 suite as a reusable regression gate.

## Findings

### Tooltip chrome and trigger geometry passed

- Trigger size matched the upstream docs demo within tolerance:
  `71.333px x 36px` in Fret vs `71.99px x 36px` upstream.
- Tooltip content chrome matched the upstream short-label size within tolerance:
  `96.667px x 28px` in Fret vs `97.49px x 28px` upstream.
- Owner and layer:
  `component_recipe` / `recipe`.

### Cross-root arrow delta false finding

- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/tooltip_demo_open_mismatch_report_v2_pre_fix.json`
  recorded a critical mechanism finding for arrow-to-trigger `center_x` delta.
- Symptom:
  the report measured the Fret arrow/trigger center delta as `252px`, while bundle semantics showed
  the same nodes aligned at delta `0px`.
- Owner and layer:
  diagnostics harness attribution, not a Tooltip recipe bug.
- Root cause:
  the parity generator always preferred layout sidecar nodes when present. For this Tooltip slice,
  the sidecar could expose the trigger through a scroll/local taffy root and the arrow through an
  overlay root, so a cross-root delta mixed incompatible coordinate spaces. The sibling
  `bundle.schema2.json` evidence had the correct global window coordinates for both nodes.
- Fix:
  `tools/parity-discovery/shadcn_parity_discovery.py` now supports predicate-level
  `evidence_source`, and the Tooltip arrow delta requests
  `bundle_schema2_semantics`.

### Tooltip diagnostics script was not geometry-capable

- Pre-fix symptom:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-demo-open-arrow.json` only captured a
  bundle, so it could prove the open state but not serve as a geometry parity repro.
- Fix:
  the script now captures `capture_layout_sidecar` and a screenshot after the tooltip panel is
  stable, then captures the bundle.

### Upstream side placement is context-sensitive

- The upstream DOM golden places the trigger at the top edge of the target root, so Radix collision
  handling flips the default top-side Tooltip below the trigger.
- The UI Gallery docs page places the trigger in the page body, where the default top-side
  placement can remain above the trigger.
- The fixture intentionally avoids absolute side/origin assertions for this slice and locks
  context-stable facts: trigger size, panel chrome size, arrow size, and horizontal arrow alignment.

## Evidence

- Diag script:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-demo-open-arrow.json`
- Post-fix sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-tooltip-open-arrow-layout/sessions/1778480927008-4656/1778480934747-ui-gallery-tooltip-demo-open-arrow.layout/layout.taffy.v1.json`
- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/tooltip_demo_open_mismatch_report_v2_pre_fix.json`
- Post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/tooltip_demo_open_mismatch_report_v2.json`
- Fixture:
  `tools/parity-discovery/fixtures/tooltip_demo_open_parts_v1.json`
- Suite:
  `tools/parity-discovery/suites/shadcn_parity_discovery_v2.json`

## Gate Result

- Tooltip report:
  `3 pass_known`, `0 mismatch`, `0 blocked`.
- V2 suite:
  `5 reports`, `10 parts`, `10 pass_known`, `0 top findings`.

## Residual Follow-Ups

- Add a dedicated edge-anchor overlay mechanism harness if we want to compare collision-flipped
  side/origin placement against upstream Tooltip goldens. The UI Gallery docs page is not an
  equivalent absolute-position context for that assertion.
- Continue the v2 sweep with Dialog, because it exercises modal focus, escape, close button, and
  shell placement rather than hover/arrow geometry.
