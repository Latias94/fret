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
- [x] Add real UI Gallery scroll/virtual-list diagnostics gates once the smallest stable demo pages
  and selectors are identified.
  - Result: the dev-only Virtual List Torture page now gates small-scroll no-window-shift telemetry,
    and the default Checkbox page now gates post-scroll RTL viewport idle stability.
- [x] Add boundary-crossing Virtual List runtime gates for non-retained and retained owner paths.
  - Result: the non-retained `ui-gallery-vlist-window-boundary` suite passes after the owner fix.
    The retained `ui-gallery-vlist-window-boundary-retained` suite now runs a real bounce path,
    asserts keep-alive reuse, and writes a normal passing `suite.summary.json`.
- [x] Fix or explain the retained Virtual List boundary suite wrapper clean-exit gap so retained
  boundary-crossing coverage has a normal `suite.summary.json` proof.
  - Result: the gap was a diagnostics harness issue. The suite finalizer now summarizes success-tail
    failures, the retained script now bounces back to exercise reuse, and the streaming post-run
    gate reads current `retained_virtual_list_reconciles[].reused_from_keep_alive_items` schema.
- [x] Add retained-host synthetic reconcile fixture metrics so keep-alive attach/detach/reuse can
  be checked without launching UI Gallery.
  - Result: `retained_virtual_list_reconcile_v1.json` now drives a bounce scenario through the real
    declarative retained virtual-list host and asserts retained reconcile metrics. The first draft
    exposed a harness sampling bug: debug reconcile records are frame-scoped and must be captured
    per frame before the next debug frame clears them.
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
- [x] Add semantics fixtures for value/editing metadata, collection metadata, actions, live regions,
  and hidden-subtree policy.
- [x] Add initial UI Gallery overlay/focus diagnostics for stable default pages.
- [x] Add modal-barrier root lifecycle runtime coverage on a default-compatible page.
- [x] Add default-compatible Drawer modal underlay block/focus-restore activation-status coverage.
- [x] Add default-compatible Combobox outside-press dismiss/focus-restore runtime coverage with
  observable selected/query state probes.
- [x] Repair Combobox UI Gallery screenshot and interaction gates so offscreen triggers are scrolled
  into view before click, and make the neutral-dark open screenshot script independently runnable.
- [x] Fix the Combobox trigger chevron layout regression exposed by screenshot review and lock it
  with a focused geometry test.
- [x] Harden the Combobox trigger screenshot gate so the nav result is scrolled into view before
  `click_stable`.
- [ ] Add default-compatible non-modal click-through activation-status coverage once public pages
  expose stable underlay/status probes.
- [x] Promote Combobox visual/style coverage into an explicit fixture-style matrix that tracks
  component state, theme, viewport, screenshot gate, geometry predicates, and current owner/gap.
- [x] Harden the Button Group size gate with stable icon-only `Add` anchors and geometry predicates.
- [ ] Promote the Button Group family into an explicit fixture-style matrix for text/icon
  alignment, truncation, and theme/viewport variants.
- [x] Add ButtonGroupText prefix/suffix internal label anchors and vertical-centering geometry
  predicates against the middle input.
- [x] Add Button Group Input Group trailing button icon-centering and input/trailing-control
  non-overlap coverage.
- [x] Add Button Group Input Group constrained long-value bounds, trailing-control non-overlap, and
  caret/IME-anchor-in-input-bounds coverage.
- [x] Add Input Group RTL addon-order coverage for logical leading/trailing physical-side mapping.
- [x] Add Button Group Input Group visible-text viewport, horizontal overflow, and offset-range
  coverage through TextInput paint-time diagnostics.
- [x] Add Combobox Input Group long-query TextInput visual coverage and fix the TextInput viewport
  height drift it exposed.
- [x] Add Command/CommandPalette search-input TextInput visual coverage.
- [x] Add plain Input/File control TextInput visual coverage so the oracle covers non-composite
  input recipes too.
- [x] Add an authoring-surface lint or focused test that catches shadcn control snippets using
  `.into_element(cx).test_id(...)` when the builder's `.test_id(...)` is the documented direct
  control-id path.
- [x] Add runtime placement traces to the primary Combobox screenshot gates so list placement is
  tracked by both visual evidence and structured overlay-placement data.
- [x] Harden the Combobox popup-trigger UI Gallery gate with structured collision flip and
  shadcn `sideOffset=6` assertions.
- [x] Add a companion Combobox popup-trigger bottom-room fixture so the button-trigger/content
  variant is covered for preferred-bottom placement as well as collision flip.
- [x] Harden Combobox popup-trigger top/bottom runtime gates with visible trigger/content bounds and
  stable listbox bounds.
- [x] Add Combobox popup-trigger listbox/option row-height and row-spacing geometry predicates for
  both collision-flip and bottom-room runtime gates.
- [x] Add internal `CommandItem` label/checkmark anchors and Combobox popup-trigger text/checkmark
  size, vertical-centering, and inset predicates for both collision-flip and bottom-room runtime
  gates.
- [x] Retarget Combobox popup-trigger placement gates from the internal listbox to the content shell
  and add cross-metric side-gap oracles for both top-flip and bottom-room placement.
- [x] Add Combobox long-text truncation/ellipsis coverage for constrained trigger and popup option
  text, including trigger-label and option-label geometry predicates.
- [ ] Add a Combobox RTL long-text companion so leading/trailing icon/checkmark ownership is
  covered in both directions.
- [x] Tighten the Combobox Responsive desktop placement gate so it asserts preferred/chosen bottom
  placement and shadcn `sideOffset=6`, and add it to the `ui-gallery-combobox` diagnostics suite.
- [x] Fix the Popover first-open placement-size bridge so explicit content width hints beat the
  default 288px estimate, and tighten the Combobox Responsive gate with visible content/trigger
  bounds predicates.
- [x] Add a focused Popover component regression for first-open `align=center` placement so explicit
  content width controls visible x-position, not the default 288px estimate.
- [x] Add DropdownMenu nested submenu runtime placement traces so submenu side/anchor placement is
  observable from UI Gallery diagnostics.
- [x] Add ContextMenu submenu safe-corridor runtime placement traces and repair stale/offscreen
  script selectors.
- [x] Extend submenu runtime placement traces to Menubar, reusing the shared submenu diagnostics
  bridge once a stable public gallery path is selected.
- [x] Add a companion RTL submenu tight-left collision fixture that intentionally keeps the
  trigger too close to the left edge and asserts the correct flip back to physical right.
- [x] Add a synthetic anchored-panel overlay placement fixture matrix for preferred/flip decisions,
  sized best-fit, collision padding/boundary, RTL logical alignment, skidding, sticky shift, and
  arrow positioning.
- [x] Extend the anchored-panel fixture matrix with coordinate-space cases for non-zero and negative
  outer origins, offset collision boundaries, disabled shift axes, and sized best-fit under a
  non-zero viewport origin.
- [x] Add a two-frame declarative `Anchored` layout invalidation fixture for `side_offset`, `side`,
  `align`, and `anchor_element` changes, and fix the confirmed prop-diff defect in the owning mount
  layer.
- [x] Add a focused `Anchored` transformed-anchor gate and fix `anchor_element` placement to use
  visual bounds under ancestor `render_transform`.
- [x] Add a focused `Anchored` scroll-transformed-anchor gate and fix scroll geometry invalidation so
  layout-driven anchored overlays recompute after scroll offset changes.
