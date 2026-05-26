---
title: Shadcn Parity Discovery Harness v1 Milestones
status: active
date: 2026-05-09
---

# Milestones

## M0: Lane and Schema

Status: complete

- Dedicated workstream exists.
- Part mapping schema v1 is documented.
- Mismatch report schema v1 is documented.
- First-open state lives in `WORKSTREAM.json`.

## M1: Button Group Prototype

Status: complete

- `tools/parity-discovery/fixtures/button_group_parts_v1.json` maps upstream Button Group facts to
  Fret UI Gallery evidence.
- `tools/parity-discovery/shadcn_parity_discovery.py` validates the mapping shape and emits a
  deterministic report.
- `artifacts/button_group_mismatch_report_v1.json` records the first report.
- Promotion rules are documented in `DESIGN.md` and `EVIDENCE_AND_GATES.md`.

## M2: Fret Live Measurement

Status: complete

- Read `layout.taffy.v1.json` sidecars by stable `test_id`.
- Evaluate Button Group structured geometry predicates from measured Fret bounds.
- Regenerate `artifacts/button_group_mismatch_report_v1.json` with measured Fret evidence.

## M3: Multi-component Fret Evidence

Status: complete

- Add fixture-driven reports for Dropdown Menu and Input.
- Run dedicated UI Gallery diagnostics scripts that capture layout sidecars for both components.
- Add owner classification to generated reports.
- Capture one measured Dropdown Menu mismatch and one passing Input coverage set.
- Keep report failures case-id-addressable.

## M3b: Upstream Web Evidence Extraction

Status: complete

- Evaluate upstream shadcn docs-path examples from browser DOM evidence.
- Add viewport/theme metadata to upstream DOM predicate results.
- Compare upstream DOM measurements and Fret sidecar measurements through the same predicate
  vocabulary.
- Freeze the layout sidecar unit contract: `layout.taffy.v1.json` rects are logical px, so Dropdown
  Menu `w-56` compares as upstream DOM width `224` and Fret logical sidecar width `224`.

## M4: Multi-component Discovery

Status: complete

- Add at least three components total.
- Include layout/chrome, overlay/interaction, and responsive/container-sensitive cases.
- Add a responsive `combobox-responsive` lane that produces deterministic desktop/mobile reports
  and classifies current shell sizing drift to `mechanism_core` while keeping command/listbox
  subparts passing.
- Land the first shell-sizing fix slice:
  - Popover size hints include `HoverRegion` and `Stack` constraints, and the popover content child
    is built before the Radix dialog wrapper so desktop shell sizing can see the command/listbox
    max-height lane on the opening frame.
  - Drawer top/bottom max-height follows upstream `max-h-[80vh]` without an extra edge-gap clamp.
  - `cargo check -p fret-ui-shadcn --lib -j 1` passes under low-memory local settings.
  - `mechanism_harness_recipe_layout_cases_match_oracles` now includes and passes
    `responsive-drawer-bottom-sheet-caps-visible-lane` and
    `popover-command-shell-wraps-hover-region-max-height`.
- Fresh post-fix UI Gallery desktop/mobile diag captures and regenerated combobox reports now pass.
  The mobile fixture includes an effective-viewport guard so a native resize/request mismatch is
  classified as diagnostics-surface evidence before Drawer geometry is compared.
- A second sweep audit now records three proactive, non-user-reported findings:
  ButtonGroup SelectTrigger chrome sizing, Popover command shell sizing, and Drawer 80vh shell
  sizing. The audit maps each finding to root cause ownership and a promotion target.
- Decide whether to promote `tools/parity-discovery/` into a crate.

## M5: Calendar Upstream DOM Coverage

Status: complete

- Wire the Calendar custom-cell large viewport to upstream DOM goldens so the suite can compare
  the responsive Calendar lane against an upstream source of truth instead of sidecar-only evidence.
- Wire the Calendar custom-cell small viewport to upstream DOM goldens.
- Wire the Calendar custom-cell popover / mixed variant to upstream DOM goldens.
- Decide to use a fixture-level context map for responsive Calendar and future multi-viewport DOM
  families instead of relying on capture-name suffixes alone.

## M6: Calendar Responsive Coverage

Status: complete

- Calendar custom-cell small and large viewport discovery reports pass.
- The mixed responsive Calendar report passes after the weekday-row/day-grid spacing normalization
  in `calendar_range.rs` and `calendar_multiple.rs`.
- The suite regenerates with `31 pass_known`, `0 mismatch`, and `0 top findings`.
- The resolved calendar gap stays classified as a recipe-layer issue rather than a mechanism-layer
  issue.

## M7: Calendar Hijri Layout Gate

Status: complete

- The docs-path `calendar-hijri` example now has a dedicated web-vs-fret layout gate, and it
  passed after restoring the Hijri weekday-row/day-grid gap to `Space::N2`.

## M8: Calendar Hijri Discovery Coverage

Status: complete

- The docs-path `calendar-hijri` example is now also part of the parity-discovery suite via
  roots-dump evidence and root-relative geometry predicates.
