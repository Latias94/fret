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

## M13: Recipe Outside-Press Focus Policy Slice

Status: complete for the first matrix

- The shadcn focus-restore recipe fixture now covers policy-different outside outcomes for
  combobox, select, dropdown-menu, and context-menu in addition to dialog/popover.
- The matrix distinguishes restore-to-trigger, click-through underlay focus/activation, modal
  underlay blocking, and focus-clear outcomes.
- The first draft exposed a confirmed select pointer-open defect: a cached suppress decision from
  the opening pointer-up was reused for a different pointer id on the outside click. The guard now
  keys cached decisions by pointer id as well as tick id.
- Focused gates for context-menu click-through, dropdown-menu non-modal outside focus clear, select
  modal underlay blocking, context-menu Escape focus clear, and combobox close-auto-focus reason
  policy passed.

## M14: Recipe Prevent-Default Outside-Press Policy Slice

Status: complete

- The shadcn focus-restore recipe fixture now covers prevent-default outside-press outcomes for
  popover, select, dropdown-menu, and context-menu.
- The matrix distinguishes modal keep-open + underlay-blocking behavior from non-modal keep-open +
  click-through underlay focus/activation behavior.
- The first run exposed a confirmed select modal barrier defect: a barrier pointer-up dismiss
  handler accumulated across open frames and invoked the dismiss handler once per rendered frame.
  The select primitive now installs a single owned pointer-up handler for that barrier behavior.
- Focused gates for popover outside-press interception, select modal prevent-default, dropdown-menu
  modal prevent-default, and context-menu click-through prevent-default passed.

## M15: Recipe Focus-Outside and Submenu Restore Slice

Status: complete

- The shadcn focus-restore recipe fixture now covers pure focus-outside outcomes for popover,
  non-modal dropdown-menu, and non-modal context-menu.
- The matrix distinguishes default focus-outside close that preserves the externally focused
  underlay from `preventDefault` keep-open behavior, and asserts `dismiss.calls` so repeated
  handler invocation would fail the shared harness.
- A focused context-menu submenu gate now matches the existing dropdown-menu gate: keyboard
  ArrowRight opens the submenu, focus transfers to the first submenu item, and ArrowLeft restores
  focus to the submenu trigger.
- No runtime defect was found in this slice; the gap was missing fixture/focused coverage for
  already-correct policy paths.

## M16: Roving Typeahead Interaction Slice

Status: complete

- The roving focus interaction fixture now covers printable-key typeahead dispatch in addition to
  arrow navigation.
- The matrix asserts target focus/selection updates, no-match focus preservation, pointer-region
  wrapper preservation, and `roving.typeahead.calls` so duplicate handler invocation becomes visible.
- No runtime defect was found in this slice; the gap was missing fixture-level coverage for
  already-correct typeahead behavior.

## M17: Combobox Active-Descendant Interaction Slice

Status: complete

- A dedicated combobox active-descendant fixture suite now covers query-driven active descendant
  updates under a real declarative text-input path.
- The runner asserts the active descendant semantics relation, the filtered visible option count,
  the query length metric, and the selected active item id.
- No runtime defect was found in this slice; the gap was missing fixture-level coverage for
  already-correct active-descendant interaction behavior.

## M18: Nested Focus Scope Interaction Slice

Status: complete

- A dedicated nested focus scope fixture suite now covers inner and outer trapped scope interaction.
- The matrix asserts inner-scope next/previous traversal, outer-sibling traversal into the inner
  scope, and outside pointer activation that preserves focus inside the trapped inner scope.
- No runtime defect was found in this slice; the gap was missing fixture-level coverage for
  already-correct nested focus scope behavior.

## M19: Stale Parent Focus Scope Interaction Slice

Status: complete

- A dedicated stale-parent focus scope fixture suite now covers the retained-tree parent-pointer
  robustness path using the real `UiTree` tree internals.
- The matrix asserts child reachability under stale parent pointers, pointer activation outside the
  scope, and focus remaining inside the trapped scope after the click path.
- No runtime defect was found in this slice; the gap was missing fixture-level coverage for
  already-correct stale-parent focus scope behavior.

## M20: Recipe Submenu Restore Fixture Slice

Status: complete

- The shadcn focus-restore recipe fixture now covers dropdown-menu, context-menu, and menubar
  submenu keyboard open / ArrowLeft restore flows with stable submenu test ids.
- The harness records submenu-opened and submenu-closed metrics so the open/close sequence stays
  observable even though the final snapshot only shows the closed tree.
- No runtime defect was found in this slice; the gap was missing fixture-level coverage for
  already-correct submenu policy paths.

## M21: Menubar Submenu Restore Fixture Slice

Status: complete

- The submenu matrix now includes a menubar path in addition to dropdown-menu and context-menu.
- The menubar harness case proves ArrowRight opens the submenu, focus transfers into it, and
  ArrowLeft restores focus to the submenu trigger while the submenu contents unmount.
- No runtime defect was found in this slice; the gap was missing fixture-level coverage for
  already-correct menubar submenu policy paths.

## M22: Recipe Typeahead Fixture Slice

Status: complete

- A dedicated shadcn recipe typeahead fixture suite now covers select trigger typeahead selection
  without opening, dropdown-menu open-menu typeahead focus movement, and menubar open-menu
  typeahead focus movement.
- The select case records `select.selected.index` and `select.value_change.calls` so wrong targets
  or duplicate value-change dispatch become observable harness facts.
- No runtime defect was found in this slice; the gap was missing recipe-level fixture coverage above
  the already-covered roving typeahead mechanism.

## M23: UI Gallery Overlay/Focus Runtime Gate Slice

Status: complete

- A dedicated diagnostics suite now promotes default-compatible UI Gallery overlay/focus paths:
  AlertDialog focus trap/tab cycle, Dialog detached-trigger focus restore, and Popover Escape focus
  restore.
- The first attempted modal-barrier path exposed a harness precondition issue: that script targets
  the `gallery-dev` Overlay preview page, so running it against the default gallery binary fails
  before it reaches the mechanism invariant.
- No runtime mechanism defect was found in the promoted default-compatible paths; the fix was to
  separate stable runtime coverage from a dev-only overlay script precondition.

## M24: Public Dialog Modal Barrier Runtime Gate Slice

Status: complete

- The overlay/focus diagnostics suite now includes a public Dialog page modal-barrier gate that does
  not depend on the `gallery-dev` Overlay preview page.
- The gate asserts modal barrier and focus-barrier roots are installed together while the dialog is
  open, remain aligned, clear together after Escape, and restore focus to the demo trigger.
- No runtime mechanism defect was found; the gap was missing default-compatible runtime coverage for
  barrier root lifecycle.

## M25: Drawer Underlay Runtime Gate and Renderer Binding Fix

Status: complete

- The overlay/focus diagnostics suite now includes a default-compatible Drawer outside-press gate
  that asserts modal underlay activation is blocked, the Drawer closes, and focus restores to the
  trigger.
- The first run exposed a confirmed renderer defect: `TextDrawKind::Mask` bound `text_vertex_buffer`
  to a text pipeline whose slot 0 expects `TextGlyphInstance` instance data.
- The render scene recorder now binds `text_glyph_instance_buffer` for mask text draws, and the
  Drawer script plus the full five-script overlay/focus suite pass.

## M26: Public Combobox Outside-Press Runtime Gate Slice

Status: complete

- The Combobox conformance demo now exposes selected/query state probes whose `test_id` nodes carry
  the current state label, making status assertions directly observable by diagnostics predicates.
- The outside-press script now scrolls the conformance trigger into view before clicking it, closing
  the gap where `exists` could match an offscreen semantics node that was not actually clickable.
- The overlay/focus diagnostics suite now includes the public Combobox outside-press gate, and the
  six-script suite passes against the release UI Gallery binary.

## M27: Scroll and Virtual-List Runtime Gate Slice

Status: complete for first runtime gates

- The dev-only Virtual List Torture page now has a promoted diagnostics gate for small-scroll
  retained-window stability. The gate resets diagnostics on the settled page and asserts
  `virtual_list_window_shift_samples_len_le max=0` before and after small wheel deltas.
- The default Checkbox page now has a promoted RTL scroll idle-stability gate. After scrolling to
  the RTL checkbox section, `assert_semantics_scroll_idle_stable` samples the content viewport for
  45 no-input frames and fails on single-frame drift or total drift.
- The first pass exposed two harness defects, not a runtime mechanism defect:
  the Virtual List script targeted a dev-only page without recording the feature precondition and
  used the wrong scroll semantics node; the new idle-stability trace was initially dropped from
  passed script evidence when a following capture step started.
- Current runtime evidence did not reproduce the reported RTL scroll jitter: the Checkbox gate
  sampled `y=2495.999755859375` for all 45 frames with `frame_delta=0.0` and `total_delta=0.0`.

## M28: Virtual-List Boundary Owner Slice

Status: complete

- Boundary-crossing virtual-list diagnostics exposed confirmed owner defects in the mechanism layer:
  non-retained wheel handling could dirty the view-cache root before prepaint observed the stale
  rendered window, retained reconcile could reuse stale cached windows after programmatic scroll,
  final scroll-handle baseline consumption lacked a last-owner non-retained dirty fallback, and
  prepaint reason attribution lost `ScrollOffset` when state offset had already been synced.
- Workspace focus registry/scope components also polluted view-cache diagnostics with render-time
  no-op `ModelStore::update` calls; they now read first and update only on actual value changes.
- Focused gates passed for the new prepaint reason regression, retained virtual-list window update,
  revision-only window update, prepaint escape invalidation, and overscan-policy mismatch cases.
- The non-retained `ui-gallery-vlist-window-boundary` runtime suite passed after the owner fix.
  The retained `ui-gallery-vlist-window-boundary-retained` runtime suite now also passes with a
  normal `suite.summary.json`.
- The retained follow-up exposed diagnostics harness drift rather than a retained-host mechanism
  defect: the original script did not bounce back to exercise reuse, and the streaming post-run
  gate was still reading the older reconcile-record field names. The script and gate now observe
  current `retained_virtual_list_reconciles[].reused_from_keep_alive_items` telemetry.
- A synthetic retained-host reconcile fixture now covers the same attach/detach/keep-alive reuse
  invariant without launching UI Gallery. Its first draft exposed another harness issue rather than
  a mechanism defect: retained reconcile debug records are frame-scoped, so the fixture runner must
  accumulate records immediately after each frame.
