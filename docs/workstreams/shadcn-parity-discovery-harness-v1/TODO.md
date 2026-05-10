---
title: Shadcn Parity Discovery Harness v1 TODO
status: active
date: 2026-05-09
---

# TODO

## M0-M1: Button Group Prototype

- [x] Create a dedicated discovery workstream.
- [x] Keep the first implementation in `tools/parity-discovery/`; do not add a crate yet.
- [x] Define part mapping schema v1.
- [x] Define mismatch report schema v1.
- [x] Encode Button Group upstream source facts from `new-york-v4`.
- [x] Encode Fret facts from existing UI Gallery snippets, render-flow assertions, and diagnostics
  evidence.
- [x] Generate a deterministic first report artifact for Button Group.
- [x] Document promotion rules for diag scripts, component fixtures, and mechanism harness cases.

## M2: Fret Live Measurement

- [x] Add a Fret-side extractor that can read layout sidecars by stable `test_id`.
- [x] Add structured geometry predicates to the Button Group fixture.
- [x] Regenerate the Button Group report from measured Fret sidecar evidence.
- [x] Add a Fret-side effective viewport predicate for layout sidecar root bounds.
- [x] Add upstream viewport and theme dimensions to the mapping schema.

## M3-M3b: Upstream Web Measurement

- [x] Add a web-side extractor for upstream shadcn docs-path examples or goldens.
- [x] Compare upstream DOM measurements and Fret sidecar measurements through the same predicate
  vocabulary.
- [x] Add Dropdown Menu fixture-driven discovery checks from Fret sidecar evidence.
- [x] Add Input fixture-driven discovery checks from Fret sidecar evidence.
- [x] Add owner classification to generated reports.
- [x] Regenerate per-component reports for Dropdown Menu and Input.
- [x] Prove the harness can discover a measured shadcn mismatch without a user screenshot report.
- [x] Promote the Dropdown Menu `w-56` finding into either a recipe fix or a mechanism-sidecar unit
  contract follow-up.
- [x] Freeze the layout sidecar raw/logical unit contract in a mechanism-level follow-up.
- [x] Make remaining source-only rows runnable instead of prose-only.

## M4: Component Sweep

- [x] Add at least two more components with different failure modes.
- [x] Cover at least one overlay or focus-driven component.
- [x] Cover at least one responsive/container-sensitive component.
- [x] Split responsive combobox shell / wrapper / command / listbox into separate measured parts.
- [x] Classify shell-only drift as `mechanism_core` with `mechanism_harness` promotion.
- [x] Land the first shell-sizing fix slice for Popover size hints and Drawer top/bottom max-height.
- [x] Promote the Drawer 80vh shell rule into the lightweight shadcn mechanism harness fixture.
- [x] Promote or verify the Popover `HoverRegion`/`Stack` size-hint fix with a lightweight overlay harness
  case instead of relying only on full UI Gallery diag.
- [x] Capture fresh post-fix desktop/mobile responsive combobox sidecars.
- [x] Regenerate responsive combobox reports from post-fix sidecars and confirm the shell mismatches
  are gone.
- [x] Record the second proactive sweep audit with at least two non-user-reported findings, root
  cause ownership, and promotion targets.
- [x] Turn the native `set_window_inner_size` requested/effective height offset into a focused
  diagnostics-runner follow-up instead of keeping it only as a combobox-script workaround.
- [x] Revisit whether the tool should become a crate after three components and stable report
  semantics.

## Backlog

- [ ] Add severity scoring once mismatches can be measured live.
- [ ] Add report-to-issue helpers only after the mismatch taxonomy stabilizes.
- [ ] Add material-style source adapters after shadcn parity proves the generic schema.
