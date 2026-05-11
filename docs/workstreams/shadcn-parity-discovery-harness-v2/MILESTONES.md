---
title: Shadcn Parity Discovery Harness v2 Milestones
status: active
date: 2026-05-11
---

# Milestones

## M0: Manifest and Lane Bootstrap

Status: complete

- Coverage manifest exists and orders the first v2 sweep targets.
- Coverage manifest explicitly carries all v1 suite rows forward as `covered_v1` regression locks
  before the new v2 sweep targets.
- Workstream state is explicit.
- Baseline v1 suite remains the regression anchor.
- The first v2 suite replay path exists for the initial context-menu discovery slice.

## M1: First Uncovered Overlay Sweep

Status: complete

- Add the second uncovered overlay surface as a fixture-driven report.
- The first uncovered overlay surface already produced a fixture-driven report and a pre-fix mismatch artifact.
- Navigation Menu produced a second pre-fix mismatch artifact and the fix closed it back to pass_known.

## M2: Fix and Gate

Status: complete

- The context-menu open-width issue was fixed in `apps/fret-ui-gallery/src/ui/snippets/context_menu/demo.rs`.
- The navigation-menu components-open column-width issue was fixed in `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs`.
- The confirmed finding is now promoted into a reusable component fixture and suite gate.
- Hover Card exposed a harness blind spot: the layout sidecar missed a stable node, so the discovery
  tool now falls back to sibling `bundle.schema2.json` semantics before classifying the slice.
- The hover-card open slice is now promoted into the v2 suite as a regression anchor.
- Evidence and layer classification are recorded in the lane docs and audit notes.

## M3: Broader Responsive Sweep

Status: active

- Sheet mobile `vp375x240` is the first v2 mobile/responsive slice promoted into the suite.
- The sheet slice found and closed recipe, app-demo, and fixture blind spots while keeping the
  sheet shell mechanism green.
- Tooltip docs-demo open is now promoted into the suite as the first v2 slice that locks a
  cross-overlay/root geometry predicate with explicit `bundle_schema2_semantics` evidence.
- The tooltip slice found and closed a harness attribution bug: sidecar-first delta evaluation can
  compare local taffy-root coordinates across scroll and overlay roots unless the predicate can
  request global bundle semantics coordinates.
- Dialog docs-demo open is now promoted into the suite as the first v2 modal slice that found and
  closed an app-demo composition drift without relying on a user-reported screenshot.
- The dialog slice found that the UI Gallery demo used FieldSet/Field policy spacing while upstream
  shadcn uses plain Label/Input stacks; the post-fix fixture keeps the content/body/footer geometry
  green while tolerating the remaining native text-metric delta.
- Menubar docs-demo open is now promoted into the suite as the first v2 menubar slice that found
  and closed recipe chrome drift without relying on a user-reported screenshot.
- The menubar slice found that the root shell was auto-height `38px` instead of upstream `h-9`
  `36px`, then exposed a scale-factor-sensitive border-box lane where a fixed `1px` logical border
  pushed the trigger down. The post-fix fixture keeps root height, trigger vertical lane, and File
  menu rows green.
- Input OTP docs-demo static geometry is now promoted into the suite as the first v2 input-heavy
  slice after Menubar. It found no new recipe mismatch, but it locked 36px slots, contiguous group
  offsets, the 76px separator cross-group offset, and hidden input height/top alignment while
  recording that this slice currently needs `bundle_schema2_semantics` evidence because the taffy
  layout sidecar omits the stable OTP test ids.
- Base Table docs-demo static geometry is now promoted into the suite as the first v2 table-heavy
  slice. It found no new recipe mismatch, but it locked total demo height, body row height, and
  accumulated row cadence before moving to policy-heavy DataTable coverage.
- DataTable docs-path policy coverage is now promoted as a diagnostics suite gate. It found no new
  DataTable policy mismatch across column visibility, row-actions overlays, checkbox-only
  selection, list-like shift/meta pointer selection, and smoke coverage, but it did find and close
  two scroll-before-click script gaps plus a diagnostics runner stale `last_bundle_dir` bug for
  final `capture_bundle` steps under reuse-launch suites.
- Keep the regression-lock rows green while expanding the uncovered set.
