---
title: Fret Mechanism Harness v1 Milestones
status: active
date: 2026-05-12
---

# Milestones

## M0: Lane Shape

Status: complete

- Dedicated workstream exists.
- Coverage map names current coverage and gaps.
- First-open gates are listed in `WORKSTREAM.json`.

## M1: Layout and Invalidation Slice

Status: complete

- Layout primitive fixture remains the baseline geometry suite.
- Layout dirty invalidation gains fixture-driven scalar metric coverage.
- Suppressed-boundary child dirty deltas and subtree removal are case-id-addressable.

## M2: Runtime Gate Connection

Status: complete

- UI Gallery checkbox underflow script is the first real runtime proof.
- The script remains part of `diag-hardening-smoke`.
- Synthetic dirty invalidation cases explain the same class of mechanism risk.

## M3: Findings Loop

Status: complete for the first slice

- Run the gate set.
- Classify defects by layer.
- Fix confirmed defects.
- Leave evidence, regression coverage, and the next-slice recommendation.

## M4: View-Cache and Root-Boundary Invalidation Slice

Status: complete

- The layout dirty invalidation fixture now covers retained contained relayout, dirty frontier
  coverage at a wrapper/direct-child boundary, detached dirty-cache-root pruning, and view-cache
  layout-dirty expansion attribution.
- The harness runner can capture intermediate metrics so frontier state can be asserted before
  layout consumes it.
- Focused gates for `view_cache`, scroll-contained frontier behavior, and layout request build-root
  attribution passed.

## M5: Scroll-Handle Window-Update Slice

Status: complete

- A dedicated scroll-handle invalidation fixture suite now covers scroll registry change
  classification outcomes that affect cache-root reuse.
- The fixture matrix covers generic windowed scroll paint, virtual-list window escape,
  revision-only baseline handling after internal offset updates, and detached stale binding
  filtering.
- Focused gates for `view_cache_scroll`, retained virtual-list host reconcile, and scroll registry
  classification passed.

## M6: Environment View-Cache Invalidation Slice

Status: complete

- A dedicated environment view-cache fixture suite now drives the real `WindowMetricsService`
  entry point instead of direct `ElementRuntime` mutation.
- The first run exposed a confirmed mechanism defect: several platform environment keys were not
  committed into `ElementRuntime` before declarative rendering.
- `render_root` now synchronizes the missing environment keys, and the fixture plus focused
  environment gates pass.

## M7: Pointer Occlusion and Capture Routing Slice

Status: complete

- A dedicated pointer occlusion routing fixture suite now covers event-routing outcomes that are not
  visible in plain hit-test geometry: underlay dispatch suppression, wheel exceptions, modal barrier
  scoping, outside-press observers, pointer-move observers, and captured-pointer routing.
- The first run exposed a fixture schema constraint rather than a runtime defect: `domains` is a
  fixed mechanism enum and cannot be used as a free-form subdomain tag list.
- Focused gates for pointer occlusion, pointer-move observer/capture routing, and the existing
  hit-test routing fixture passed.

## M8: Focus Barrier Routing Slice

Status: complete

- A dedicated focus barrier routing fixture suite now covers tree-level overlay/focus outcomes:
  hit-test-inert focus barriers, explicit focus barrier activation, underlay focus rejection,
  focus traversal inside a barrier, modal barrier reporting, and timer dispatch stability.
- The first run exposed a runner type-boundary issue rather than a runtime defect: fixture commands
  must map to a closed enum before constructing static `CommandId` values.
- Focused gates for focus barrier transition and focus scope behavior passed.

## M9: Semantics Relation Slice

Status: complete

- `ObservedTree::from_semantics_snapshot` now preserves accessibility relation and state facts that
  were previously dropped by the shared harness observation layer.
- `MechanismPredicate` now supports selector-addressable semantics relation and boolean flag
  oracles.
- A dedicated declarative semantics relation fixture suite covers text-input combobox controls,
  active-descendant, `attach_semantics`, and `SemanticsProps` relation/state outcomes.
- Focused gates for text input semantics relations and attach-semantics relation/state behavior
  passed.

## M10: Roving Focus Interaction Slice

Status: complete

- A dedicated roving focus fixture suite now covers disabled item skipping, wrapping navigation,
  non-wrapping edge preservation, and pointer-region wrapped item collection.
- The runner asserts both the resulting focused semantics node and the active-selection model via a
  mechanism metric.
- Focused gates for existing roving flex behavior passed.

## M11: Focus Scope Interaction Slice

Status: complete

- A dedicated focus scope fixture suite now covers trapped traversal, wrap behavior, non-trapped
  traversal, and outside pointer activation without focus escape.
- The runner asserts exact focus for command traversal and mechanism metrics for pointer activation
  plus focus containment.
- Focused gates for existing focus scope and layered focus scope behavior passed.

## M12: Shadcn Focus Restore Recipe Slice

Status: complete

- A dedicated shadcn focus-restore recipe fixture suite now covers dialog, popover, combobox,
  select, and dropdown-menu Escape dismissal restoring focus to the invoking trigger and closing
  the open model.
- The same fixture suite now also covers outside-press policy differences: dialog overlay click
  restores trigger focus, while popover click-through outside press focuses and activates the
  underlay target.
- The first run exposed a real harness oracle defect: `FocusIs` used barrier-filtered selector
  lookup and could not observe restored focus outside a still-present pointer/modal barrier.
- The oracle now uses unfiltered selector lookup for focus predicates, with a focused
  `fret-mechanism-harness` regression test.
- Focused recipe gates for dialog, popover, combobox, select, and dropdown-menu Escape focus
  restore passed.
