---
title: Shadcn Parity Discovery Sixth Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, menubar, recipe chrome harness
---

# Sixth Sweep Audit

This audit records the Menubar docs-demo open slice from the v2 sweep. The slice promoted Menubar
into the fixture-driven suite and closed a recipe chrome drift found without a user-reported
screenshot.

## Objective Criteria

The slice required:

1. Capture the Menubar docs demo open state with layout sidecar, bundle, and screenshot evidence.
2. Compare stable shadcn `new-york-v4` root/trigger/menu-row geometry facts against the upstream
   DOM snapshot.
3. Classify every non-passing result by layer.
4. Fix the highest-confidence confirmed issue.
5. Promote the slice into the v2 suite as a reusable regression gate.

## Findings

### Root shell auto-height drift

- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/menubar_demo_open_mismatch_report_v2_pre_fix.json`
  recorded two critical recipe findings.
- Symptom:
  Fret Menubar root measured `38px` tall while the upstream open DOM snapshot measured `36px`.
- Secondary symptom:
  the File trigger started `5.333px` below the root top instead of the upstream `4px` lane.
- Owner and layer:
  `component_recipe` / `recipe`.
- Root cause:
  the Fret recipe let the Menubar root auto-size from padding, border, and trigger content. Upstream
  shadcn owns `h-9` on the Menubar root, so the root is fixed to a `36px` border-box height.
- Fix:
  `ecosystem/fret-ui-shadcn/src/menubar.rs` now defaults the Menubar root layout height to `36px`
  before applying caller `refine_layout` overrides.

### Scale-factor-sensitive trigger lane

- After fixing the root height, the trigger height was explicitly restored to `28px`, but the
  trigger still started too low because the root's fixed `1px` logical border snapped to a wider
  layout border at scale factor `1.5`.
- Root cause:
  the upstream web golden behaves like a one-physical-pixel hairline in the captured coordinate
  system. Using `1px` logical border in Fret shifted the border-box content lane by about `1.333px`.
- Fix:
  the Menubar root default border width now resolves to `1 / scale_factor` unless the caller
  explicitly overrides `border_width`. This keeps shadcn border-box geometry aligned while leaving
  explicit custom chrome unchanged.

## Evidence

- Diag script:
  `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-demo-open-layout.json`
- Pre-fix sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-menubar-demo-open-layout/sessions/1778484440394-172856/1778484449246-ui-gallery-menubar-demo-open.layout/layout.taffy.v1.json`
- Post-fix sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-menubar-demo-open-layout-post-fix-3/sessions/1778487579461-183428/1778487719630-ui-gallery-menubar-demo-open.layout/layout.taffy.v1.json`
- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/menubar_demo_open_mismatch_report_v2_pre_fix.json`
- Post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/menubar_demo_open_mismatch_report_v2.json`
- Fixture:
  `tools/parity-discovery/fixtures/menubar_demo_open_parts_v1.json`
- Suite:
  `tools/parity-discovery/suites/shadcn_parity_discovery_v2.json`

## Gate Result

- Menubar pre-fix report:
  `1 mismatch` part, with two failing recipe checks.
- Menubar post-fix report:
  `2 pass_known`, `0 mismatch`, `0 blocked`, `0 top findings`.
- Focused Rust gate:
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib menubar::tests::menubar_root_shell_matches_shadcn_h9_height -- --exact --nocapture`
  passed.

## Residual Follow-Ups

- Add a separate Menubar keyboard/roving/typeahead slice if the next sweep wants policy coverage.
  This slice intentionally gates docs-demo geometry, not keyboard behavior.
- Audit whether other shadcn root shells with `border p-* h-*` need a shared hairline-border helper
  rather than component-local scale-factor handling.
