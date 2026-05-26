---
title: Shadcn Parity Discovery Second Sweep Audit
status: active
date: 2026-05-10
scope: shadcn parity discovery, responsive combobox, mechanism harness promotion
---

# Second Sweep Audit

This audit records the prompt-to-artifact closure for the post-fix responsive combobox
shell-sizing evidence and the second proactive parity discovery sweep. It is not a replacement for
the generated reports; it maps the objective to concrete artifacts so the lane can be reviewed
without relying on memory of the investigation.

## Objective Criteria

The active objective requires:

1. Close the shadcn responsive combobox shell-sizing post-fix evidence.
2. Start a second proactive parity discovery sweep over the DropdownMenu, Select, Popover,
   Command, Drawer, and ButtonGroup family.
3. Reuse the shell/child segmented oracle so shell drift is separated from child content drift.
4. Find at least two non-user-reported layout, overlay, or mechanism issues.
5. Classify root cause ownership.
6. Promote reproducible issues into a mechanism harness, component fixture, or diagnostics script.

## Prompt-to-Artifact Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| Responsive combobox desktop post-fix report is closed | `artifacts/combobox_responsive_open_mismatch_report_v1.json` reports 4 parts, 4 pass, 0 mismatch | Done |
| Responsive combobox mobile post-fix report is closed | `artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json` reports 6 parts, 6 pass, 0 mismatch | Done |
| Mobile viewport drift is gated before Drawer geometry | `mobile_effective_viewport` uses a `root_metric` predicate and passes before shell checks | Done |
| Shell/child segmentation is preserved | Desktop separates popover shell, command root, and listbox; mobile separates effective viewport, drawer shell, wrapper, command root, and listbox | Done |
| At least two proactive issues were found | ButtonGroup SelectTrigger chrome sizing, Popover shell size hints, and Drawer 80vh shell sizing were all discovered through generated reports/sidecars rather than user screenshots | Done |
| Root cause ownership is classified | ButtonGroup is `component_recipe`; Popover shell is `mechanism_core`; Drawer shell is `mechanism_core` with source-backed Drawer recipe policy | Done |
| Reproducible issues are promoted | ButtonGroup lands in component/report/diag gates; Popover and Drawer land in `mechanism_layout_recipe_cases_v1.json` and the recipe mechanism harness runner | Done |
| DropdownMenu was swept without current drift | `dropdown_menu_mismatch_report_v1.json` reports 3 parts, 3 pass, 0 mismatch and keeps overlay content selector coverage | Done |
| Select was swept through a high-risk embedded docs path | ButtonGroup Select fixture maps the embedded currency `SelectTrigger`, compares upstream DOM and Fret sidecars, and now passes after the chrome padding fix | Done |
| Command child drift was disambiguated from shell drift | Responsive combobox reports keep Command root/listbox parts passing while Popover/Drawer shell parts carried the pre-fix mismatches | Done |

## Findings

### 1. ButtonGroup SelectTrigger Chrome Sizing

- Surface: ButtonGroup docs-path select example.
- Discovery evidence:
  - Initial discovery run:
    `target/fret-diag/shadcn-parity-discovery-sweep-v1/button-group-select/sessions/1778335284436-9032`.
  - Updated report:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/button_group_mismatch_report_v1.json`.
- Pre-fix symptom: the embedded currency `SelectTrigger` was too narrow against the upstream
  ButtonGroup select example baseline.
- Root cause: the shadcn Select trigger chrome used asymmetric left/right padding that did not match
  the source-backed compact trigger lane used inside ButtonGroup.
- Owner: `component_recipe`.
- Promotion:
  - Component/report fixture: `tools/parity-discovery/fixtures/button_group_parts_v1.json`.
  - UI Gallery render-flow invariant: `apps/fret-ui-gallery/src/driver/render_flow.rs`.
  - Diagnostics script: `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json`.
- Current status: post-fix report records 7 parts, 7 pass, 0 mismatch; the currency trigger width is
  about 58 logical px against the upstream 58.219 px baseline.

### 2. Popover Command Shell Sizing

- Surface: responsive combobox desktop open state.
- Discovery evidence:
  - Pre-fix segmented desktop report recorded `desktop_popover_shell_surface` as the only failing
    part while the Command root and listbox passed.
  - Post-fix report:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_open_mismatch_report_v1.json`.
- Pre-fix symptom: `PopoverContent` stayed at the placement fallback height instead of wrapping the
  Command subtree height.
- Root cause: popover size-hint collection skipped wrapper elements that matter in self-drawn UI
  trees, especially `HoverRegion` and `Stack`; the content child also had to be built before the
  Radix dialog wrapper so the opening frame could read its size hint.
- Owner: `mechanism_core`.
- Promotion:
  - Unit/root-cause gates in `ecosystem/fret-ui-shadcn/src/popover.rs`.
  - Mechanism recipe fixture:
    `ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json`
    (`popover-command-shell-wraps-hover-region-max-height`).
  - Harness runner:
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/mechanism_harness.rs`.
- Current status: desktop report records 4 parts, 4 pass, 0 mismatch.

### 3. Drawer Bottom Sheet 80vh Sizing

- Surface: responsive combobox mobile open state.
- Discovery evidence:
  - Pre-fix segmented mobile report recorded `mobile_drawer_shell_surface` as the only failing part
    while the command wrapper, Command root, and listbox passed.
  - Post-fix report:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json`.
- Pre-fix symptom: `DrawerContent` height clamped to 164px in the effective 375x240 viewport instead
  of the upstream 192px 80vh lane.
- Root cause: Fret applied an extra `viewport - 96px` edge-gap clamp that is not part of upstream
  shadcn v4 Drawer top/bottom content sizing (`max-h-[80vh]`).
- Owner: `mechanism_core` for promotion, with the source-backed recipe policy fixed in Drawer.
- Promotion:
  - Unit/root-cause gate in `ecosystem/fret-ui-shadcn/src/drawer.rs`.
  - Mechanism recipe fixture:
    `ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json`
    (`responsive-drawer-bottom-sheet-caps-visible-lane`).
  - Harness runner:
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/mechanism_harness.rs`.
- Current status: mobile report records 6 parts, 6 pass, 0 mismatch and gates the effective
  viewport before comparing the Drawer shell.

## Non-Issue Classifications

- DropdownMenu: the current report is a pass. The sweep still improved the diagnostics surface by
  keeping a stable overlay content selector, but it is not counted as one of the two required
  layout/overlay/mechanism defects.
- Command: the responsive combobox segmentation found that the Command root and listbox were
  already correct while the outer Popover/Drawer shell was wrong. That negative finding is important
  because it prevents misclassifying shell bugs as Command recipe bugs.
- Select: the high-risk Select surface in this sweep is the ButtonGroup embedded trigger. Direct
  Select overlay placement and chrome coverage already lives in the existing shadcn web-vs-Fret
  fixture suites and remains a future report-generation candidate rather than a required new drift.

## Residual Follow-Ups

- Promote the native Windows requested/effective viewport height offset into a runner-level
  diagnostics follow-up. The current mobile report gates effective viewport size correctly, but it
  does not make the runner resize contract controllable by itself.
- Keep the parity-discovery report generator as a tools prototype until a separate decision says the
  report format should become a crate. The mechanism harness crate already owns in-process runtime
  oracles; the report generator is still a source-to-evidence triage surface.
- A later Select/Command report expansion can reuse the same schema, but the current objective's
  minimum of two proactive findings is already met by the ButtonGroup, Popover, and Drawer issues.
