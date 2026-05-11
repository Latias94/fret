---
title: Fret Mechanism Harness v1 TODO
status: active
date: 2026-05-12
---

# TODO

- [x] Create the dedicated workstream and coverage map.
- [x] Extend `fret-mechanism-harness` with scalar mechanism metrics for non-geometry facts.
- [x] Add fixture-driven layout dirty invalidation cases for suppressed boundaries and repair paths.
- [x] Connect the checkbox "Enable notifications" underflow path as the first UI Gallery runtime
  diagnostics gate.
- [x] Run the first gate set and record actual command results in `FINDINGS_2026-05-11.md`.
- [x] If a new confirmed defect appears, fix it in the owning layer and add regression coverage.
  - Result: no new confirmed defect appeared in this slice; the known suppressed-boundary defect was
    already fixed and is now locked by fixture coverage plus diagnostics.
- [x] Pick the next slice from the largest uncovered mechanism gap.
- [x] Extend layout dirty invalidation fixtures into view-cache/root-boundary invalidation:
  retained contained relayout, scroll/direct-child dirty frontier coverage, detached dirty-cache-root
  pruning, and view-cache layout-dirty expansion attribution.
- [x] Run focused gates for the view-cache/root-boundary slice and record the results.
- [x] Add a dedicated scroll-handle invalidation fixture suite for windowed-paint cache-root dirtying,
  virtual-list window escape, revision-only baseline handling, and detached stale binding filtering.
- [x] Run focused gates for the scroll-handle window-update slice and record the results.
- [x] Promote environment-triggered cache-root invalidation into a fixture-driven declarative
  cache-hit harness using the real `WindowMetricsService` entry point.
- [x] Fix the confirmed environment sync defect exposed by the harness.
- [x] Promote pointer occlusion and captured-pointer routing into a fixture-driven tree harness.
- [x] Run focused gates for pointer occlusion, pointer-move observer, captured-pointer, and existing
  hit-test routing coverage.
- [x] Promote focus barrier and focus traversal routing into a fixture-driven tree harness.
- [x] Run focused gates for focus barrier and focus scope coverage.
- [x] Promote semantics relations and boolean accessibility flags into shared harness observations
  and fixture predicates.
- [x] Add declarative semantics relation fixtures for text-input combobox controls,
  active-descendant, `attach_semantics`, and `SemanticsProps` relation/state outcomes.
- [x] Add declarative roving focus fixture coverage for disabled skip, wrap, no-wrap edge behavior,
  and pointer-region wrapped items.
- [x] Add declarative focus scope fixture coverage for trapped traversal, wrap, non-trap traversal,
  and pointer activation without focus escape.
- [x] Add shadcn recipe-consumer focus restore fixture coverage for dialog, popover, and combobox
  Escape dismissal.
- [x] Fix the focus oracle bug exposed by recipe focus restore coverage.
- [x] Extend shadcn recipe-consumer Escape restore fixture coverage to select and dropdown-menu.
- [x] Extend shadcn recipe focus policy fixture coverage to dialog overlay click and popover
  click-through outside press.
- [x] Add shadcn outside-press focus restore/clear matrix coverage for select, dropdown-menu,
  combobox, and context-menu policy differences.
- [x] Fix the select pointer-open outside-press guard-cache defect exposed by the outside-press
  fixture.
- [x] Add prevent-default outside-press policy matrices for select, dropdown-menu, popover, and
  context-menu.
- [x] Add focus-outside policy matrices for popover, dropdown-menu, and context-menu, plus a
  context-menu nested submenu keyboard restore focused gate.
- [x] Extend the roving focus interaction fixture with printable-key typeahead dispatch,
  no-match preservation, wrapper traversal, and call-count metrics.
- [ ] Add a real UI Gallery scroll/virtual-list diagnostics gate once the smallest stable demo page
  and selectors are identified.
- [ ] Add UI Gallery diagnostics for runtime platform preference/environment changes once a stable
  demo page exists.
- [ ] Add a UI Gallery pointer occlusion/capture diagnostics gate once a stable overlay demo exposes
  test ids for underlay, overlay, and observer/capture state.
- [x] Add active-descendant interaction fixture coverage for combobox query-driven active descendant
  selection.
- [x] Add nested focus scope fixture coverage for inner/outer trapped scope traversal and pointer
  focus containment.
- [x] Add stale-parent focus scope fixture coverage for retained-tree parent-pointer robustness.
- [x] Add dropdown-menu, context-menu, and menubar submenu fixture coverage for ArrowRight open
  and ArrowLeft restore parity.
- [x] Extend recipe-level typeahead parity beyond the current mechanism matrices.
- [ ] Add semantics fixtures for value/editing metadata, collection metadata, actions, live regions,
  and hidden-subtree policy.
- [x] Add initial UI Gallery overlay/focus diagnostics for stable default pages.
- [ ] Add modal-barrier runtime coverage on a default-compatible page, or split `gallery-dev`
  diagnostics suites so dev-only overlay scripts cannot be run with the default gallery binary.
