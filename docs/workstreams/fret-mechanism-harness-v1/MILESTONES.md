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
- A synthetic prepaint virtual-list window-update fixture now covers scroll-offset, viewport-resize,
  items-revision, and scroll-to-item reason/detail attribution. Its first run exposed a confirmed
  mechanism defect: prepaint debug telemetry kept the specific viewport/items-revision reason, but
  the actual dirty cache-root reason regressed to generic prefetch/window-update detail. The
  prepaint path now uses one classifier for both debug telemetry and cache-root dirty attribution.
- The same fixture now covers a length-shrink input-change case. It exposed that stale rendered
  window counts were not classified as `InputsChange` before offset deltas. Prepaint now emits a
  dedicated `scroll_handle_inputs_change_window_update` invalidation detail for this case.

## M29: DataTable Retained Filter-Shrink Consumer Gate

Status: complete

- The synthetic length-shrink `InputsChange` proof now has a real component-consumer companion:
  `ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json` runs against the DataTable
  torture page with `FRET_UI_GALLERY_DATA_TABLE_RETAINED=1`.
- The first runtime pass exposed a confirmed component-layer defect. Retained DataTable row entries
  were rebuilt from raw data and local sorting instead of the filtered `FlatRowOrderCache`, so a
  global filter could leave the retained virtualized path behaving as if it still had the original
  50,000 rows.
- The same gate exposed a confirmed mechanism-layer defect. Layout-time virtual-list telemetry used
  local classification and did not reliably classify render-window or previous-window
  `count`/`overscan` mismatches as `InputsChange`.
- Layout and prepaint virtual-list paths now share `classify_virtual_list_window_shift`, and the
  DataTable retained path now derives row entries from the shared filtered row order.
- Diagnostics virtual-list matching predicates now support `invalidation_detail` filtering so
  non-retained DataTable filter-shrink companions can later assert
  `scroll_handle_inputs_change_window_update` directly.
- The real runtime gate passes and records a layout-sourced retained virtual-list
  `reason=inputs_change`, `apply_mode=retained_reconcile` proof after applying
  `GlobalFilter: Process 123`.

## M30: DataTable View-Cache Filter-Shrink Consumer Gate

Status: complete

- A second DataTable torture companion now covers the non-retained, view-cache-enabled path. It runs
  with `FRET_UI_GALLERY_VIEW_CACHE=1` and asserts layout-sourced
  `reason=inputs_change`, `apply_mode=non_retained_rerender`, and
  `invalidation_detail=scroll_handle_inputs_change_window_update`.
- The first non-retained run exposed a harness precondition issue rather than a mechanism defect:
  default Gallery runs have `view_cache_active=false`, so the layout record correctly had
  `reason=inputs_change` but no view-cache invalidation detail.
- The script and suite names now carry the view-cache precondition, and the passing runtime evidence
  proves the filtered row count shrinks from 50,000 to 111 through the real DataTable consumer.

## M31: DataTable RTL Idle-Stability Consumer Gate

Status: complete

- The DataTable page now has a promoted RTL idle-stability runtime gate in addition to Checkbox,
  ScrollArea, and plain Table.
- The gate scrolls the public DataTable docs page to `ui-gallery-data-table-rtl-root`, waits for
  `ui-gallery-data-table-rtl-root` and `ui-gallery-data-table-rtl-footer` bounds to settle, and then
  samples `ui-gallery-content-viewport` for 60 no-input frames.
- The gate passed and did not reproduce a scroll-jitter mechanism defect on the DataTable RTL
  surface.

## M32: Combobox RTL Long-Text Gate Audit

Status: complete

- The Combobox RTL long-text companion already existed, was registered in the combobox, RTL smoke,
  and shadcn conformance suites, and passed a fresh audit rerun.
- The gate proves the physical-left RTL chevron stays before the trigger label, the long label
  stays in its width budget, the popup content shell records a top collision flip with
  `sideOffset=6`, and the selected option label stays before the physical-right RTL checkmark.
- No mechanism or recipe defect was reproduced in this slice. The defect was workstream state
  drift: TODO and coverage docs still listed RTL long-text as an uncovered gap even though a
  promoted runtime gate existed and was green.

## M33: Pointer Occlusion Runtime Gate Strengthening

Status: complete

- The context-menu pointer occlusion wheel gate now has a structural oracle instead of only
  screenshots and a final bundle.
- The gate verifies the overlay page content viewport starts at `scroll.y=0`, has a non-zero scroll
  range, receives a wheel through `BlockMouseExceptScroll`, ends at non-zero `scroll.y`, and keeps
  the context menu mounted.
- No pointer occlusion mechanism defect was reproduced. The defect found in this slice was a
  harness weakness: the previous gate could pass without proving that underlay wheel pass-through
  actually moved a scroll container.

## M34: Pointer-Capture Runtime Lifecycle Gate

Status: complete

- Diagnostics protocol now exposes `input_pointer_capture_active_is`, backed by
  `debug.input_arbitration.pointer_capture_active`, so runtime scripts can assert pointer-capture
  lifecycle directly instead of inferring it from pixels or widget-specific semantics.
- The ScrollArea scrollbar drag baseline gate now asserts capture is active immediately after
  scrollbar `pointer_down`, remains active during drag, and is released after `pointer_up`.
- The real UI Gallery gate passed while preserving the existing `semantics_scroll_approx_eq y=20`
  scroll-progress oracle. No pointer-capture mechanism defect was reproduced. The defect fixed in
  this slice was a harness observability gap: runtime diagnostics could not previously lock
  capture active/release state.
- Remaining follow-up: add owner/test-id attribution, underlay blocking probes, and
  release-on-cancel coverage once a stable public demo exposes those surfaces.

## M35: Pointer-Capture Owner and Cancel Runtime Gate

Status: complete

- The pointer-capture runtime gate now has owner-level assertions through the new `captured_is`
  predicate, so scripts can prove which semantics node owns capture rather than only proving that
  some capture is active.
- The first owner-level runtime run exposed a confirmed mechanism defect: semantics snapshots could
  stay stale when live pointer-capture owner state changed without a layout or semantics dirty bit.
  `UiTree::request_semantics_snapshot_if_dirty()` now refreshes when snapshot focus or captured
  owner differs from the current tree state.
- The ScrollArea drag baseline gate now asserts owner true during drag and owner false after
  `pointer_up`, while the new pointer-cancel companion asserts both active capture and owner state
  clear after `pointer_cancel`.
- Focused gates cover `UiPredicateV1::CapturedIs` serialization, synthetic oracle behavior,
  runtime predicate evaluation, and the `fret-ui` semantics dirty gate.
- Remaining follow-up: add captured-pointer underlay blocking, multi-pointer/cross-window capture,
  and nested overlay branch/corridor runtime probes.

## M36: Multi-Pointer Captured-Underlay Runtime Gate

Status: complete

- Schema-v2 pointer-session steps now accept optional `pointer_id`, defaulting to `0` for existing
  mouse-pointer scripts.
- The diagnostics runner now tracks active pointer sessions by `PointerId` instead of a single
  global session, and pointer-move continuation state is keyed by the script pointer id.
- Non-default touch/pen pointer sessions no longer write global mouse cursor/button override files,
  so a synthetic underlay touch cannot move or release pointer 0's captured mouse drag.
- UI Gallery ScrollArea diagnostics now expose a separate `viewport-probe` target without changing
  the original scrollbar drag baseline's scroll/group semantics.
- The new runtime gate `ui-gallery-scrollbar-drag-multipointer-underlay-touch.json` proves pointer 0
  keeps scrollbar capture while pointer 1 touch-down/up targets the viewport probe, then proves
  pointer 0 cancel releases active capture and owner state.
- Remaining follow-up: extend the same multi-pointer capability to cross-window docking/tear-out and
  nested overlay branch/corridor probes.

## M37: Cross-Window Docking Multi-Pointer Runtime Gate

Status: complete

- Diagnostics protocol now exposes `dock_viewport_capture_active_is`, backed by the docking
  `viewport_capture` diagnostics snapshot, so runtime scripts can assert the forbidden competing
  capture state directly.
- The new docking arbitration gate
  `docking-arbitration-demo-multiwindow-dock-drag-suppresses-viewport-touch.json` tears off a dock
  tab into a second OS window, starts a dock drag in the overlapping moving window, then probes the
  main-window viewport with pointer 1 as touch input.
- The gate proves the dock drag remains active and the main-window viewport does not start capture
  while the secondary pointer is down/up.
- No core docking defect was reproduced. The defect fixed in this slice was a harness observability
  gap: runtime diagnostics could not previously assert viewport-capture absence directly.
- Remaining follow-up: add nested overlay branch/corridor runtime probes that combine modal
  barrier, submenu safe corridor, hover intent, and outside-pointer routing.

## M38: ContextMenu Submenu Branch/Corridor Runtime Gate

Status: complete

- ContextMenu submenu coverage now goes beyond placement traces: the runtime gate proves Radix-style
  branch behavior for moving into submenu content, back to the parent trigger, away from the
  trigger, and onto another parent-menu item.
- The new gate `ui-gallery-context-menu-submenu-branch-corridor-routing.json` passed against UI
  Gallery and is promoted into both the mechanism overlay/focus suite and the shadcn conformance
  suite.
- No core hit-test or ContextMenu recipe defect was reproduced. The defect fixed in this slice was
  harness coverage weakness: branch/corridor behavior was only indirectly covered by placement and
  existence assertions.
- Remaining follow-up: add a stable semantics/accessibility runtime gate for filtered default
  selectors versus diagnostic visibility.

## M39: Hidden Semantics Runtime Selector Parity Gate

Status: complete

- Runtime diagnostics default selectors now match the synthetic hidden-subtree oracle: nodes with
  `SemanticsFlags.hidden` or a hidden ancestor are filtered out of default selector predicates.
- Diagnostics now expose `raw_semantics_hidden_is`, a raw/effective hidden predicate for scripts
  that need to prove hidden/decorative nodes remain inspectable without weakening default
  selector semantics.
- The new UI Gallery gate
  `ui-gallery-separator-decorative-hidden-semantics.json` passed against shadcn Separator and is
  promoted into the shadcn conformance suite.
- The slice found and fixed a real diagnostics/runtime mechanism mismatch; no Separator recipe
  defect was reproduced after the runtime selector fix.
- Remaining follow-up: add a dynamic semantics mutation gate for live/expanded/selected or
  active-descendant state changes on a stable UI Gallery recipe.

## M40: Accordion Expanded Semantics Runtime Mutation Gate

Status: complete

- Diagnostics protocol now exposes `expanded_is`, backed by the semantics expanded flag.
- The synthetic mechanism oracle now evaluates `UiPredicateV1::ExpandedIs`, so controlled fixtures
  and runtime scripts share the same predicate vocabulary.
- The Accordion Usage runtime gate now asserts the trigger starts expanded, becomes collapsed after
  click, and becomes expanded again after reopening while the panel mount/unmount behavior remains
  covered.
- The gate passed against UI Gallery and is promoted into both `ui-gallery-shadcn-runtime-evidence`
  and `ui-gallery-shadcn-conformance`.
- No Accordion recipe defect was reproduced. The defect fixed in this slice was harness
  observability: runtime scripts could not previously assert expanded-state mutation directly.
- Remaining follow-up: add active-descendant mutation coverage for Combobox/Command/Listbox
  composite widgets where focus remains on a container while the active item changes.

## M41: Composite Active-Descendant Runtime Gate Promotion

Status: complete

- Existing Combobox and Command runtime scripts now form a durable conformance gate for
  active-descendant mutation: Combobox covers both disabled auto-highlight and first-match
  auto-highlight, while Command covers controlled selection value and ArrowDown mutation.
- The Command controlled-selection scripts are promoted into `ui-gallery-command`, and all four
  active-descendant scripts are promoted into `ui-gallery-shadcn-conformance`.
- Script roundtrip coverage now locks the four promoted scripts, and the registry now indexes the
  Command controlled-selection scripts with `active_descendant` tags.
- No Combobox or Command recipe defect was reproduced. The defect fixed in this slice was harness
  promotion: active-descendant probes existed but were not all reachable from durable suites.
- Remaining follow-up: add dynamic live-region/status mutation coverage, or selected-state mutation
  gates for listbox-style recipes once a deterministic page exposes stable selected semantics.

## M42: Sonner Live-Region Runtime Mutation Gate

Status: complete

- Diagnostics protocol now exposes `semantics_live_is` and `semantics_live_atomic_is`.
- The synthetic mechanism oracle and bootstrap runtime predicate evaluator now share the same
  live-region predicate vocabulary.
- The new Sonner runtime gate proves the `Notifications` toast viewport is absent before a toast is
  shown, exposes `live=polite` and `live_atomic=false` while the toast is mounted, and disappears
  after swipe dismissal.
- The gate passed against UI Gallery and is promoted into `ui-gallery-sonner-docs`,
  `ui-gallery-shadcn-runtime-evidence`, and `ui-gallery-shadcn-conformance`.
- No Sonner recipe defect was reproduced. The defect fixed in this slice was harness observability:
  runtime scripts could not previously assert live-region metadata directly.
- Remaining follow-up: add selected-state mutation runtime gates for listbox-style recipes.

## M43: Select Selected-State Runtime Mutation Gate

Status: complete

- The primary Select commit-and-label UI Gallery script now asserts selected semantics after a
  value commit: after selecting Banana and reopening the popup, Banana reports
  `selected_is=true` while Apple reports `selected_is=false`.
- The first promoted run exposed a diagnostics script stability defect rather than a Select recipe
  defect: the script clicked the item before overlay placement/bounds stabilized, so the synthetic
  click was sent to `y=0` and the label stayed on the previous value.
- The script now waits for the overlay placement trace and visible viewport bounds before item
  clicks and before selected-state assertions.
- The gate passed after the harness fix and is promoted into `ui-gallery-shadcn-runtime-evidence`
  and `ui-gallery-shadcn-conformance`.
- Remaining follow-up: add a companion selected-state mutation gate for Tabs or DataTable faceted
  filters if their gallery pages expose stable selected semantics without state coupling.

## M44: Tabs Selected-State Runtime Mutation Companion

Status: complete

- Added `ui-gallery-tabs-selected-state-mutation.json`, a non-screenshot runtime gate that proves
  the Tabs demo starts with Account selected, Password unselected, then flips those selected flags
  after clicking the Password tab.
- The gate uses stable UI Gallery trigger test IDs and the shared `selected_is` predicate, so it
  checks the same runtime semantics surface as the Select gate without depending on overlay
  placement.
- The gate passed on the first runtime run and is promoted into `ui-gallery-shadcn-runtime-evidence`
  and `ui-gallery-shadcn-conformance`.
- No Tabs mechanism or recipe defect was reproduced. The value of this slice is coverage breadth:
  selected-state semantics are now gated on both overlay-backed Select and inline Tabs.
- Remaining follow-up: move to collection/positional semantics mutation, or add DataTable faceted
  selected-state coverage only if a stable selected item surface is available.

## M45: Command Collection Metadata Runtime Mutation Gate

Status: complete

- Added `ui-gallery-command-scrollable-collection-metadata-mutation.json`, a shadcn Command runtime
  gate that asserts collection position metadata before and after filtering.
- The gate proves the scrollable Command item `Code Editor` starts with `pos_in_set=23` and
  `set_size=23`, then after filtering to `code editor` updates to `pos_in_set=1` and `set_size=1`.
- Diagnostics protocol now has builder helpers for `pos_in_set_is` and `set_size_is`, plus focused
  protocol serialization and bootstrap runtime predicate tests for collection metadata.
- The gate passed against UI Gallery and is promoted into `ui-gallery-command`,
  `ui-gallery-shadcn-runtime-evidence`, and `ui-gallery-shadcn-conformance`.
- No Command recipe or semantics mechanism defect was reproduced. The fixed defect was a harness
  coverage gap: collection metadata predicates existed, but no durable shadcn runtime mutation gate
  exercised filtered collection rebuilding.
- Remaining follow-up: cover collection metadata mutation across pagination or virtual-list window
  changes to test retained/windowed node reuse rather than only filtered list rebuilding.

## M46: DataTable Pagination Collection Metadata Runtime Gate

Status: complete

- Added diagnostics-only row anchors to the UI Gallery default DataTable via `TableDebugIds`, using
  the stable prefix `ui-gallery-data-table-default-row-`.
- Added `ui-gallery-data-table-default-pagination-collection-metadata.json`, a runtime gate that
  checks row collection metadata across page changes: page 1 rows 1/2 and 2/2, page 2 rows 1/2 and
  2/2, and final page row 1/1.
- The first runtime run exposed a diagnostics script stability defect: `Next` was partially outside
  the window, so the script click produced `no_hit` and never advanced pagination.
- The script now scrolls `Next` into view, gates `bounds_within_window`, and uses `click_stable`
  before asserting the next page's row metadata.
- The gate passed after the script fix and is promoted into `ui-gallery-data-table`,
  `ui-gallery-shadcn-runtime-evidence`, and `ui-gallery-shadcn-conformance`.
- No DataTable pagination or core semantics defect was reproduced. The remaining higher-risk gap is
  retained/windowed collection metadata mutation, where reused row nodes can retain stale
  `pos_in_set` or `set_size`.

## M47: Retained Virtual-List Collection Metadata Runtime Gate

Status: complete

- `SemanticsDecoration` and `SemanticsProps` now expose collection metadata, so non-pressable
  semantics nodes can publish `pos_in_set` and `set_size` without becoming pressables or adding
  layout wrappers.
- A focused `fret-ui` gate proves both layout-transparent decorations and semantics wrappers can
  stamp collection metadata into the snapshot.
- The Virtual List Torture page now publishes row-root semantics IDs
  `ui-gallery-virtual-list-row-N` with `ListItem` role, label, `pos_in_set`, and `set_size`, while
  preserving the existing row label IDs used by older scripts.
- Added `ui-gallery-virtual-list-retained-collection-metadata-bounce.json`, a runtime gate that
  proves row 0 starts as item 1/10000, row 25 becomes item 26/10000 after a retained window-boundary
  scroll, row 0 is detached at that offset, and row 0 returns as item 1/10000 after bouncing back.
- The first two runtime drafts found harness authoring defects rather than retained-list defects:
  the script duplicated existing scroll-handle and keep-alive reuse telemetry assertions. The final
  gate scopes itself to collection metadata plus retained attach/detach, and the existing boundary
  script continues to own scroll/reuse telemetry.
- The promoted `ui-gallery-vlist-window-boundary-retained` suite now runs both the old boundary
  gate and the new collection metadata bounce gate.
- No stale retained virtual-list collection metadata defect was reproduced after the mechanism
  observability fix.

## M48: Retained Tree Hierarchy Semantics Mutation Gate

Status: complete

- Added `level_is` to the diagnostics protocol, bootstrap predicate evaluator, and mechanism
  harness oracle so hierarchy metadata can be asserted outside screenshots.
- Exported semantics `level` through diagnostics bundles and added synthetic `SemanticsProps`
  fixture coverage for tree-item hierarchy metadata.
- Tree and FileTree rows now publish `TreeItem` role, hierarchy `level`, and parent-row
  `expanded` metadata; Tree toggle buttons also expose `expanded`.
- Added `ui-gallery-tree-retained-hierarchy-semantics-toggle.json`, a retained Tree runtime gate
  that proves root/folder/leaf levels, root/folder expanded state, collapse child detachment, and
  expansion metadata after retained row reuse.
- The first runtime attempts found harness precondition/stability issues: Tree Torture is a
  `gallery-dev` page, and the old `click + type_text` nav prelude was less deterministic than the
  existing `type_text_into(clear_before_type=true)` pattern.
- The first valid retained Tree run found a real `fret-ui-kit` Tree policy defect: toggle buttons
  dispatched `tree.toggle.<id>` but no owning layer handled that command, so `TreeState.expanded`
  never changed and row-level `expanded` semantics stayed stale.
- Tree now mutates its owned `TreeState` directly for selection and expansion, with focused policy
  tests plus the retained Tree diagnostics suite locking the behavior.
- Remaining follow-up: move to retained non-list action-state mutation, for example disabled or
  invokable row action surfaces on DataTable/FileTree if a stable runtime page exposes them.

## M49: DataTable Pagination Disabled/Invoke Action-State Runtime Gate

Status: complete

- Added `disabled_is` and generic `semantics_action_is` to the diagnostics protocol, builder,
  bootstrap runtime predicate evaluator, and mechanism harness UI predicate oracle.
- Diagnostics bundle semantics action export now includes `decrement` and `increment`, so the
  runtime action surface matches the full core `SemanticsActions` contract rather than only the
  action flags that earlier scripts happened to need.
- The synthetic semantics relation fixture now covers a disabled pressable option and asserts both
  `disabled_is=true` and `semantics_action_is(invoke)=false` through the same UI predicate path
  used by runtime scripts.
- `ui-gallery-data-table-default-pagination-collection-metadata.json` now also checks DataTable
  Prev/Next action-state mutation: first page has Prev disabled/non-invokable and Next
  enabled/invokable; final page flips those states.
- The first runtime retry with the new predicate failed because the old `target/dev-fast`
  `fretboard-dev.exe` binary had not been rebuilt and did not know `semantics_action_is`. After
  rebuilding, the runtime gate passed.
- No DataTable pagination component defect was reproduced. The fixed defect was a harness and
  diagnostics observability gap: runtime scripts could not assert disabled state or arbitrary
  semantics actions, and bundles omitted two core action flags.
- Remaining follow-up: add a retained/windowed action-state mutation surface where reused rows or
  tree/file rows can change `disabled`, `selected`, or `invoke` state without being rebuilt.

## M50: Retained Tree Selected/Invoke Action-State Runtime Gate

Status: complete

- Extended `ui-gallery-tree-retained-hierarchy-semantics-toggle.json` instead of adding a parallel
  script, so hierarchy semantics, collapse/expand detach/reattach, selected-state mutation, and
  row invoke availability are checked in one retained Tree lifecycle.
- The gate now proves retained Tree rows start unselected and invokable, selecting row
  `1000000` sets `selected_is=true`, collapsing root detaches that row, re-expanding reattaches it
  with `selected_is=true`, and selecting leaf row `2000000` moves selected state away from the
  previous row.
- Added script metadata tags and hints so the diagnostics registry classifies this as both
  hierarchy and action-state coverage.
- Focused script roundtrip passed, the single runtime gate passed, and the full
  `ui-gallery-tree-retained` suite passed with the extended script.
- No retained Tree mechanism or component policy defect was reproduced. This closes the first
  retained/windowed action-state mutation gate for selected/invoke state.
- Remaining follow-up: add a surface where retained row disabled state or invoke availability
  changes dynamically, because selected-state mutation does not prove stale disabled action
  suppression.

## M51: Retained Tree Dynamic Disabled/Invoke Suppression Runtime Gate

Status: complete

- Added a diagnostics-only Tree Torture control that flips retained row `2000000` between enabled
  and disabled through the owned `TreeItem` model while retaining the same large Tree surface.
- Extended `ui-gallery-tree-retained-hierarchy-semantics-toggle.json` so the retained Tree gate now
  proves dynamic `disabled_is`, `semantics_action_is(focus)`, and
  `semantics_action_is(invoke)` mutation, disabled-row pointer clicks do not change selection, and
  re-enabled rows become selectable again.
- The first compile found and fixed a diagnostics-control wiring issue: the new control needed the
  existing action extension trait import and the status label needed the existing `text_color`
  styling path instead of a non-existent `.text_muted()` helper.
- Keyboard-route probes showed this Tree surface is not a valid disabled-but-focusable activation
  target: disabled rows correctly lose focus/invoke action and cannot keep focus after being
  disabled.
- Focused script roundtrip passed, the single runtime gate passed, and the full
  `ui-gallery-tree-retained` suite passed after the dynamic disabled assertions were added.
- No retained Tree stale disabled/focus/invoke mechanism or component policy defect was reproduced.
- Remaining follow-up: find a focusable-disabled recipe/primitive surface, such as menu, listbox,
  or command-style items, for true Enter/Space activation suppression coverage.

## M52: Accordion Focusable-Disabled Keyboard Suppression Runtime Gate

Status: complete

- Added a UI Gallery Accordion focusable-disabled snippet that starts an uncontrolled single
  Accordion open and non-collapsible, matching the Radix outcome where the open trigger is
  aria-disabled but remains focusable.
- Added `ui-gallery-accordion-focusable-disabled-keyboard-suppression.json` and promoted it into
  the shadcn runtime evidence and conformance suites.
- The first runtime gate found a real shadcn Accordion recipe defect: the focusable disabled
  trigger exported the right semantics but had zero-width bounds, causing focus repair to clear the
  route before `focus_is` could pass.
- Accordion item wrapper columns now fill width with `min_width=0`, so the recipe keeps
  shrink-safe layout without collapsing trigger/content width under the UI Gallery docs shell.
- Focused Accordion tests now prove focus remains enabled, invoke remains suppressed, bounds are
  non-empty, and focus repair preserves the trigger.
- The post-fix runtime gate passed and proved Enter/Space remain suppressed while the open
  non-collapsible item stays expanded.
- Remaining follow-up: add a synthetic focusable-disabled keyboard activation fixture and a second
  recipe-family runtime gate so the invariant is not only covered through Accordion policy.

## M53: Pressable Key Activation Fixture for Focusable-Disabled Suppression

Status: complete

- Added `PressableKeyActivation::None` so `crates/fret-ui` can model a focusable node that rejects
  all keyboard activation keys without turning off focus routing.
- Added `pressable_key_activation_v1.json` and a thin `fret-ui` harness covering Enter+Space,
  Enter-only, no-keyboard-activation, focusable-disabled semantics, and fully disabled Pressable
  semantics/action outcomes.
- Updated `fret-ui-kit` Accordion's aria-disabled helper so the open non-collapsible trigger now
  uses both semantics suppression (`disabled=true`, `invokable=false`) and the new keyboard
  suppression mechanism.
- The first fixture run clarified a harness boundary: direct `UiTree::set_focus` is an internal
  force-set route, so disabled reachability should be judged through traversal, semantics actions,
  and input activation rather than direct focus mutation alone.
- Focused `fret-ui`, `fret-ui-kit`, and shadcn Accordion gates passed, and the UI Gallery Accordion
  runtime gate passed again after the new mechanism was wired into the recipe path.
- Remaining follow-up: add a second recipe-family runtime gate for disabled-but-focusable items in
  menu/listbox/command-style surfaces.

## M54: Button Group Family Suite and Accessible-Name Lint Cleanup

Status: complete

- Promoted Button Group from scattered conformance entries into a dedicated
  `ui-gallery-button-group` diagnostics suite covering docs, demo, icon, size, ButtonGroupText,
  Input Group, long-text, RTL addon, input fill, separator, accessibility, and Select scenarios.
- The first family-suite run found two actionable accessibility issues: the Button Group Select
  currency trigger and UI Gallery shell Theme/Motion preset triggers had no explicit accessible
  names, and `fret-diag` lint treated `labelled_by` relations as missing labels.
- Fixed the teaching surface by adding `a11y_label(...)` where the Selects are unlabeled by visible
  FieldLabel content, and fixed the diagnostics harness so `labelled_by` counts as an accessible
  name source.
- Focused lint regression passed, the Button Group Select rerun produced zero lint findings, and
  the full `ui-gallery-button-group` suite passed with 13 scripts and zero lint warnings.
- Remaining follow-up: add renderer-level glyph/ellipsis evidence for Button Group long text, or
  move back to the second recipe-family disabled-but-focusable gate if focus/action suppression is
  the higher-risk next slice.

## M55: DropdownMenu Disabled-But-Focusable Runtime Gate

Status: complete

- Added `DropdownMenuItem::focusable_when_disabled(true)` as an explicit Base UI-style opt-in while
  preserving Radix/shadcn's default `disabled(true)` outcome as non-focusable and non-invokable.
- The recipe now separates three facts for regular items: focusability, disabled semantics, and
  activation. Opt-in disabled items remain roving candidates and focusable, but they stamp
  `disabled=true`, suppress `invoke`, and use `PressableKeyActivation::None` to reject Enter/Space.
- The implementation audit found a duplicated regular-item render path and a root-menu focus
  candidate path that still used raw `disabled`; the fix now routes focus-candidate ownership
  through `item_focusable` while keeping submenu trigger activation disabled.
- Added stable UI Gallery `Support` and `API` anchors and promoted
  `ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json` into
  `fret-mechanism-harness-overlay-focus`.
- The first diagnostics drafts exposed harness-quality issues rather than component defects:
  direct `focus` inside an overlay was not the right proof of roving focus, scripts are not indexed
  until promoted into a suite manifest, and the status oracle needed the current
  `last action: ready` baseline.
- Focused shadcn lib tests, the `fretboard-dev`/UI Gallery build, the promoted registry check, and
  the runtime diagnostics gate all passed.
- Remaining follow-up: add a command/listbox-style disabled-but-focusable runtime gate so the next
  proof exercises active-descendant/list semantics rather than only menu roving focus.

## M56: Overlay/Focus Selector Uniqueness Hardening

Status: complete

- The full `fret-mechanism-harness-overlay-focus` suite proactively found a promoted-script
  selector defect: `ui-gallery-context-menu-submenu-content` named both the Gallery page/snippet
  content node and the mounted ContextMenu overlay content node.
- The failure came from the suite lint layer (`semantics.duplicate_test_id`), not from the
  ContextMenu branch/corridor interaction assertions. This proved the suite can catch automation
  ambiguity before it hides or misattributes an overlay routing defect.
- Fixed the owning Gallery diagnostics surface by renaming the mounted overlay selector to
  `ui-gallery-context-menu-submenu-overlay-content` and updating the ContextMenu submenu routing
  and safe-corridor scripts.
- The focused ContextMenu script rerun passed, bounded `diag query test-id` checks proved both the
  page-content and overlay-content selectors are unique, and the full overlay/focus suite passed
  with 8 scripts and zero lint findings.
- Remaining follow-up: document/use the selector convention for future promoted overlay fixtures:
  page/snippet containers own `*-content`; mounted overlay panels use `*-overlay-content` or an
  equivalent panel-specific suffix.

## M57: Command Disabled-But-Focusable Active-Descendant Runtime Gate

Status: complete

- Added Command behavior rows for a default disabled item (`Legacy Export`) and an opt-in
  disabled-but-focusable item (`Deploy API`), plus a stable last-action label so the runtime gate
  can prove Enter does not dispatch a disabled active row.
- Added focused Command tests proving default disabled rows are skipped by active-descendant
  navigation, while opt-in disabled-focusable rows can become active descendants but expose
  `disabled=true`, suppress `invoke`, and reject Enter activation.
- Added and promoted
  `ui-gallery-command-palette-disabled-focusable-keyboard-suppression.json` into both
  `ui-gallery-command` and `ui-gallery-shadcn-conformance`.
- The first full Command suite runs found harness-quality defects rather than a Command recipe
  defect: `Ctrl+P` in the old keybindings script collided with the app-level command palette
  shortcut, and several Command scripts captured bundles after only the input/control was visible
  while the selected active row was outside the window.
- Hardened the affected Command scripts so keybinding proof avoids the global shortcut collision
  and every active-descendant capture first scrolls the selected row into view.
- Focused tests, the single runtime gate, and the full `ui-gallery-command` suite passed with 17
  scripts and zero lint findings.
- Remaining follow-up: add retained/windowed active-descendant action-state mutation coverage where
  a disabled or invokable row detaches, reattaches, or changes availability under filtering or
  virtualization.

## M58: Retained Active-Descendant Relation Normalization Gate

Status: complete

- Added a fixture-driven retained virtual-list active-descendant action-state case to
  `combobox_active_descendant_interaction_v1.json`.
- The first synthetic run found a real `fret-ui` mechanism defect: when the active retained row
  scrolled out of the current semantics traversal, the input still published an `active_descendant`
  edge to the detached row's old node.
- Fixed `UiTree::refresh_semantics_snapshot` so relation edges are snapshot-local before reverse
  relation normalization: detached `active_descendant` edges are cleared, and `labelled_by`,
  `described_by`, and `controls` drop targets absent from the current snapshot.
- Added the Command page runtime demo and promoted
  `ui-gallery-command-retained-active-descendant-action-state.json` into the Command family,
  shadcn conformance, and runtime-evidence suites.
- The runtime gate found one harness-quality issue after suite promotion: the demo combobox input
  had no accessible label. The demo and synthetic fixture now stamp `a11y_label`, and rerun bundles
  lint with zero warnings.
- Focused synthetic test, UI Gallery build, single runtime gate, lint checks, registry check, and
  workstream catalog check passed. The full `ui-gallery-command` suite passed with 18 scripts.
- Remaining follow-up: add synthetic/runtime relation-edge mutation coverage for `labelled_by`,
  `described_by`, and `controls`, especially when the target detaches or crosses overlay/root
  boundaries.

## M59: Semantics Relation-Edge Detach/Reattach Harness Gate

Status: complete

- Added `relation-targets-detach-reattach-clear-stale-edges` to the semantics relation fixture
  suite, with a multi-frame observer for `labelled_by`, `described_by`, and `controls`.
- The fixture proved the F98 snapshot-local relation filtering already clears those edges while
  targets are detached and resolves them again after reattach; no additional `fret-ui` mechanism
  defect was reproduced.
- Added diagnostics protocol predicates `semantics_relation_includes` and
  `semantics_relation_is_empty` so runtime scripts can directly prove raw relation edges and
  empty-state invariants.
- Added typed diagnostics builder helpers for the same predicates.
- Added bootstrap predicate evaluator support and focused tests covering `active_descendant`,
  `labelled_by`, `described_by`, and `controls`.
- Focused nextest gates passed for `fret-ui`, `fret-diag-protocol`, and `fret-bootstrap`.
- Remaining follow-up: connect these new predicates to a UI Gallery runtime relation-edge gate for
  cross overlay/root-boundary source-target ownership.

## M60: Cross-root Select Relation Runtime Gate

Status: complete

- Extended the promoted Select commit/reopen runtime script with raw relation-edge predicates:
  trigger `controls` listbox after open, listbox `labelled_by` trigger after open, trigger
  `controls` empty after commit/close, and trigger `controls` restored after reopen.
- Added `SelectContent::test_id(...)` so the UI Gallery gate can name the mounted listbox panel
  without reusing `test_id_prefix` and renaming the long-lived `select-scroll-viewport` selector.
- The first runtime gate found a real diagnostics harness defect: relation predicate endpoint
  resolution reused ordinary modal-barrier-scoped selectors, so the underlay Select trigger could
  not be selected while the popup barrier was active even though the semantics edge was present.
- Added relation-endpoint selector resolution for diagnostics predicates. Ordinary selectors still
  respect modal barrier scoping, while relation predicates can inspect visible, non-hidden source
  and target endpoints across the barrier/root boundary.
- Focused bootstrap tests, the Select runtime gate, the semantics relation fixture gate, protocol
  predicate gates, and Select test-id focused gates passed.
- Remaining follow-up: use the same runtime path to stress overlay/listbox placement ownership
  under scroll-container clipping, RTL, and viewport resize rather than only relation correctness.

## M61: Cross-root Anchored Coordinate and Root-Boundary Policy Gate

Status: complete for synthetic and focused gates

- Added `anchored_cross_root_coordinate_v1.json` and a thin `fret-ui` harness covering secondary
  viewport anchors, non-zero overlay/root boundaries, preferred-side flip, and cross-axis clamping
  against the owning root instead of the OS window origin.
- The synthetic fixture proved core `AnchoredProps` resolves cross-root anchors correctly; no core
  placement mechanism defect was reproduced in this slice.
- The slice found a policy-layer root-boundary gap: `fret-ui-kit` anchored placement helpers and
  shadcn anchored overlay recipes were using the environment viewport as their collision/clamp
  outer. That is wrong for embedded or secondary render roots.
- Added root-boundary placement helpers in `fret-ui-kit` and migrated shadcn Popover, Select,
  Tooltip, HoverCard, DropdownMenu, ContextMenu, and Menubar placement paths to them.
- Focused gates passed for the synthetic fixture, `fret-ui-kit` root-boundary helpers, shadcn
  Popover compile/placement smoke, and representative HoverCard, Tooltip, DropdownMenu,
  ContextMenu, and Menubar overlay tests.
- Remaining follow-up: add a runtime multi-viewport ownership diagnostics gate with placement trace,
  relation edges, hit-tested selection, screenshot, and layout sidecar evidence.

## M62: Runtime Multi-Viewport Combobox Root-Boundary Gate

Status: complete for the Resizable Combobox runtime gate and owning mechanism fix

- Promoted a UI Gallery Resizable fixture where a Combobox trigger lives near the bottom of a
  Resizable panel viewport root while the OS window still has room below it.
- The first runtime run found a real mechanism-layer root-boundary cache defect: the overlay
  placement trace kept choosing `bottom` and used `outer_collision=900x1000@0,0`, meaning the
  source element boundary was still the window/owner root instead of the panel viewport root.
- The fix keeps `NodeEntry.root` as the declarative owner-root contract and adds a separate
  per-element effective root-boundary cache rebuilt after final layout from live element-node
  mappings and nearest registered `viewport_root` bounds.
- `ElementContext::root_bounds_for_element` and the free `elements::root_bounds_for_element` query
  now prefer that effective boundary before falling back to owner root bounds.
- Focused mechanism and UI kit gates passed, and the runtime gate now passes with
  `chosen_side=top`, `preferred_fits_without_main_clamp=false`, and
  `outer_collision=336x378@514.67,468.67`.
- Added focused nested viewport-root precedence, same-element viewport movement, and
  view-cache-hit retained-render movement coverage. The remaining follow-up is a runtime UI Gallery
  companion only if a real surface can move cached overlay sources across viewport roots.

## M63: Non-Modal Overlay Underlay Activation Oracle

Status: complete for Popover and DropdownMenu runtime gates

- Strengthened existing non-modal overlay outside-press gates by adding a real underlay activation
  oracle instead of relying on focus/dismiss proxy signals.
- `ui-gallery-overlay-underlay-activated` now proves the underlay button's activation handler ran.
- Popover click-through and DropdownMenu non-modal outside-press runtime gates both pass with the
  new activation-status assertion.
- No new overlay mechanism defect was reproduced; the fix is a harness-quality improvement that
  makes future outside-press consumption regressions visible.
- Remaining follow-up: move to semantics/accessibility runtime gates unless fresh Radix parity
  evidence demands additional click-through families.

## M64: Read-Only Switch Semantics Action-State Gate

Status: complete for the Switch read-only runtime gate and owning recipe fix

- Added diagnostics protocol support for `read_only_is`, including typed builder support,
  bootstrap predicate evaluation, wait trace selector recording, serialization tests, and predicate
  evaluator tests.
- Strengthened the existing focused `Switch::read_only(true)` test with focus/invoke action-state
  assertions. The first run found a real recipe defect: read-only Switch blocked pointer mutation
  but still exposed `actions.invoke=true`.
- Fixed `ecosystem/fret-ui-shadcn` Switch so read-only attaches `read_only=true` and
  `invokable=false` semantics while preserving focusability.
- Added a UI Gallery read-only Switch teaching surface and promoted
  `ui-gallery-switch-read-only-action-state.json` into `ui-gallery-shadcn-conformance`.
- Focused recipe, diagnostics protocol, bootstrap predicate, registry, build, and runtime gates all
  pass. Runtime evidence is packed at
  `.fret/diag/runs/ui-gallery-switch-read-only-action-state-f112/share/1778909364811.zip`.
- Remaining follow-up: add dynamic non-list action-state mutation coverage where read-only or
  command-gated availability changes across frames without remounting the control.

## M65: Dynamic Read-Only Switch Action-State Gate

Status: complete for the UI Gallery dynamic read-only companion

- Extended the read-only Switch UI Gallery snippet with a policy toggle that flips read-only state
  in place.
- Added focused recipe coverage for `read_only=true -> false -> true` semantics mutation. The first
  draft exposed a harness modeling issue: component props do not change unless the declarative root
  is rerendered after the model update. The corrected focused gate rerenders and passes.
- Added and promoted `ui-gallery-switch-read-only-dynamic-action-state.json`.
- The runtime gate proves the same control transitions from `read_only=true/invoke=false` to
  `read_only=false/invoke=true`, allows one checked-state mutation while editable, then returns to
  `read_only=true/invoke=false` and suppresses further mutation.
- No new runtime mechanism or recipe defect was reproduced after the F112 Switch fix.
- Follow-up completed in M66: command-gated non-list action-state mutation, where command
  availability snapshots can change independently from the widget-local model.

## M66: Command-Gated Switch Action-State Gate

Status: complete for the UI Gallery command-gated companion

- Extended the Switch UI Gallery teaching surface with a command-gated control whose enabled
  state is driven externally through `WindowCommandEnabledService`.
- The first runtime pass did not find a recipe defect. It found a harness observability gap: the
  Gallery driver handled the command after `UiTree` had already recorded an unhandled dispatch, but
  it did not emit a second `handled_by_driver=true` command-dispatch decision. The runtime script
  therefore could not distinguish "bubble-through only" from "driver handled".
- Added driver-handled dispatch recording for the owned Switch command-gate toggle path in
  `apps/fret-ui-gallery/src/driver/runtime_driver.rs`.
- The strict runtime gate now passes and proves enabled `disabled=false/invoke=true`, disabled
  `disabled=true/invoke=false`, suppressed checked-state mutation while disabled, and re-enabled
  mutation after the command-gated service is cleared again.
- The slice closes the command-gated non-list action-state companion started by M65.

## M67: Shell Theme/Motion Runtime Token Gate

Status: complete for shell-level UI Gallery theme/motion preference changes

- Extended the UI Gallery diagnostics app snapshot with `theme_preset`, `motion_preset`, open
  states, Theme revision/color scheme, and effective motion token values.
- Added `ui-gallery-motion-preset-runtime-token-mutation.json` and promoted it into
  `ui-gallery-motion-pilot`.
- The gate drives the always-visible Theme/Motion preset selectors, proving model state,
  select-close state, color scheme, reduced motion zero-duration/easing tokens, and snappy duration
  tokens through `app_snapshot_field_equals`.
- The first runtime draft exposed a diagnostics oracle weakness, not a component defect: raw `f32`
  easing/bounce values are unsuitable for strict JSON equality. The snapshot now exposes rounded
  readable values plus milli-scaled integer fields, and the script asserts the integer fields.
- Follow-up completed by M68: runner/platform-injected environment preference changes now have a
  separate runtime gate. This slice covers the Gallery shell selectors; M68 covers the platform
  event path.

## M68: Platform Preference Runtime Environment Gate

Status: complete for runner-injected platform preference changes

- Added `set_window_preferences` to diagnostics script v2 and mapped it through runtime effects,
  desktop/web runner handling, and the same `WindowMetricsService` path used by real platform
  environment changes.
- Added a Motion Presets `environment_probe` that reads color scheme, reduced motion, and text
  scale through `ElementContext` environment queries, plus a matching UI Gallery app snapshot under
  `/shell/window_metrics_preferences`.
- Added `ui-gallery-platform-preferences-runtime-environment-mutation.json` and promoted it into
  `ui-gallery-motion-pilot`.
- The first real runtime run exposed a harness script defect, not a mechanism defect: the script
  waited for the Motion Presets page probe while still on the default page. It now navigates to the
  page explicitly before waiting for the probe.
- The passing gate proves the full chain:
  `diag script -> runtime Effect -> runner -> WindowMetricsService -> global change notification -> ElementRuntime environment query -> UI Gallery snapshot/probe`.

## M69: UI Gallery Page-Entry Authoring Lint

Status: complete for promoted Motion Presets scripts

- Added a scoped registry lint for `ui-gallery-motion-pilot`: page-local
  `ui-gallery-motion-presets-*` selectors require a prior page-root proof for
  `ui-gallery-page-motion-presets`.
- The first strict pass found and fixed an existing script authoring issue:
  `ui-gallery-motion-presets-fluid-tabs-pixels-changed-fixed-frame-delta.json` navigated to Motion
  Presets but did not assert the page root before waiting for a page-local trigger.
- Added `tools/test_check_diag_scripts_registry.py` so the lint proves three cases directly: a bad
  page-local selector before page entry fails, a selector after page entry passes, and the global
  shell motion preset trigger remains allowed.
- The lint turns the F116 false-timeout lesson into a reusable harness guard instead of relying on
  individual agents to remember the page-entry convention.
- Follow-up audit extended the same rule to promoted Select scripts, where the current audit
  showed zero page-entry violations. Combobox became strict after the lint learned to treat
  explicit `FRET_UI_GALLERY_START_PAGE=combobox` defaults as valid entry evidence. DataTable is not
  yet a strict page-entry family; it needs cleanup before the same rule can be enabled there without
  introducing historical noise.

## M70: Motion-Pilot Alert/Drawer Defect Discovery

Status: complete for AlertAction slot metadata, Drawer snap-point gates, and Sidebar
tooling-timeout/visibility follow-up.

- Reran `ui-gallery-motion-pilot` after the page-entry lint work and confirmed the suite continued
  to find real issues rather than only validating new harness code.
- The first blocker was a recipe/diagnostics boundary defect: `AlertAction` used a fixed internal
  diagnostics `test_id` as a slot marker, producing duplicate exported test ids in UI Gallery.
- Added layout-transparent `AnyElement::component_slot(...)` and migrated `AlertAction` to use it,
  keeping internal recipe classification out of semantics, layout, hit testing, and a11y.
- The next blocker was Drawer snap-point clickability: the trigger existed but was far below the
  visible window, proving that UI Gallery long-page scripts must assert visibility before
  `click_stable`.
- `click_stable` timeout diagnostics now report target/window geometry and classify fully offscreen
  targets as `click_stable.target_outside_window`.
- All promoted Drawer snap-point scripts now scroll the trigger into view and assert window bounds
  before clicking.
- The spring-retarget Drawer script now encodes the correct Vaul-style policy: a sufficiently large
  downward drag may dismiss the drawer, and the gate verifies content removal plus focus restore.
- Rerunning the motion-pilot suite advanced past Drawer, Motion Presets, and Overlay Dialog before
  exposing the Sidebar harness gap: `click_stable` could run until external tooling timeout without
  producing a forced triage bundle.
- Follow-up evidence now closes that gap: `fret-diag` preserves forced-bundle evidence for
  long-running intent-step tooling timeouts, and the Sidebar fixed-frame-delta script scrolls the
  long-page trigger into view and gates window bounds before `click_stable`.

## M71: HoverRegion Absolute-Child Envelope Defect Discovery

Status: complete for px, fractional, and right/bottom absolute-child HoverRegion envelopes.

- Extended `layout_primitives_v1.json` with HoverRegion absolute-child cases that assert final
  layout bounds, measured max-content size, child placement, and hit-test samples.
- The first case exposed a mechanism defect where HoverRegion final layout needed to include
  absolute children for hover/hit-test coverage, but intrinsic measurement still used the generic
  passthrough path and collapsed to `0 x 0`.
- The follow-on fractional-inset case exposed a second mechanism defect: fractional left/top insets
  were ignored during shrink-wrap envelope sizing, so final placement could push the child outside
  the wrapper's hover/hit-test bounds.
- A real-surface right/bottom companion was added after confirming shadcn ScrollArea and code-block
  scrollbar chrome use HoverRegion-wrapped absolute overlays with end insets. It passed with the
  shared helper, so no third HoverRegion mechanism defect was reproduced.
- `fret-ui` now routes HoverRegion measurement through a dedicated path and shares a conservative
  absolute-child envelope helper between HoverRegion layout and measurement.
- The focused gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-capture`
  with latest run id `4ab0b09a-f343-4d37-89d9-646b00bf491c`.
- Adjacent layout regression filters also pass after the shared helper change:
  `declarative::tests::layout::basics` passed 38/38 with run id
  `c394dc00-75ad-472c-8ac3-303eb9745667`, and
  `declarative::tests::layout::interactivity` passed 17/17 with run id
  `e3f5be40-3ea4-4665-ba08-0d412f1d792e`.
- Viewport-root wrapper regressions also pass after the helper change:
  `declarative::tests::layout::viewport_roots` passed 37/37 with run id
  `279a732e-d9da-4033-9701-3e3ccef1e05b`.

## M72: HoverCard Strict Diagnostics Authoring Gate

Status: complete for promoted HoverCard page-entry and long-page click-visibility authoring.

- Extended `tools/check_diag_scripts_registry.py` so promoted `ui-gallery-hover-card` scripts join
  the strict page-entry and long-page click-visibility checks.
- Added focused registry self-tests proving HoverCard page-local selectors require either
  `ui-gallery-page-hover-card` evidence or `FRET_UI_GALLERY_START_PAGE=hover_card`, and that
  HoverCard content `click_stable` targets need a prior visibility guard.
- The stricter rules found zero current HoverCard violations after the earlier fixed-frame-clock
  and sides-placement repairs, so this slice locked an already-clean suite instead of changing
  runtime scripts.
- Registry gates pass:
  `python tools/test_check_diag_scripts_registry.py` ran 24 tests, and
  `python tools/check_diag_scripts_registry.py` reports the registry is up to date.
- Runtime evidence stays green:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-hover-card --dir target/fret-diag-hover-card-strict-authoring-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  produced
  `target/fret-diag-hover-card-strict-authoring-v1/sessions/1779002136522-139728/suite.summary.json`
  with `status=passed`, 6/6 rows, `scripts_with_evidence=6`, `focus_mismatch_total=0`, and zero
  lint errors/warnings.

## M73: Menubar Placement Strict Diagnostics Authoring Gate

Status: complete for promoted Menubar Placement page-entry and long-page click-visibility authoring.

- Extended `tools/check_diag_scripts_registry.py` so promoted `ui-gallery-menubar-placement`
  scripts join the strict page-entry and long-page click-visibility checks.
- Added focused registry self-tests proving Menubar page-local selectors require either
  `ui-gallery-page-menubar` evidence or `FRET_UI_GALLERY_START_PAGE=menubar`, and that Menubar
  content `click_stable` targets need a prior visibility guard.
- A dry run found zero current Menubar Placement violations because the existing three placement
  scripts already carry explicit Menubar page defaults, page-root checks, and window-contained
  scroll guards before stable content clicks.
- Registry gates pass:
  `python tools/test_check_diag_scripts_registry.py` ran 27 tests, and
  `python tools/check_diag_scripts_registry.py` reports the registry is up to date.
- Runtime evidence stays green:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-menubar-placement --dir target/fret-diag-menubar-placement-strict-authoring-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  produced
  `target/fret-diag-menubar-placement-strict-authoring-v1/sessions/1779003319016-117380/suite.summary.json`
  with `status=passed`, 3/3 rows, and zero lint errors/warnings for every row.
- Follow-up completed by M74: `ui-gallery-dropdown-menu` is now strict after adding the missing
  visibility guards for `ui-gallery-dropdown-menu-demo-trigger.chrome` and
  `ui-gallery-dropdown-menu-submenu-trigger.chrome`.

## M74: DropdownMenu Strict Diagnostics Authoring Gate

Status: complete for promoted DropdownMenu page-entry and long-page click-visibility authoring.

- Extended `tools/check_diag_scripts_registry.py` so promoted `ui-gallery-dropdown-menu` scripts
  join the strict page-entry and long-page click-visibility checks.
- Added focused registry self-tests proving DropdownMenu page-local selectors require either
  `ui-gallery-page-dropdown-menu` evidence or `FRET_UI_GALLERY_START_PAGE=dropdown_menu`, and that
  DropdownMenu content `click_stable` targets need a prior visibility guard.
- Fixed the two dry-run violations found during M73:
  `ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json` now scrolls the demo
  trigger fully into the window before `click_stable`, and
  `ui-gallery-dropdown-menu-submenu-open-smoke.json` now requires full window containment when
  scrolling the submenu trigger.
- Registry gates pass:
  `python tools/test_check_diag_scripts_registry.py` ran 30 tests, and
  `python tools/check_diag_scripts_registry.py` reports the registry is up to date.
- Runtime evidence stays green:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-dropdown-menu --dir target/fret-diag-dropdown-menu-strict-authoring-v2 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  produced
  `target/fret-diag-dropdown-menu-strict-authoring-v2/sessions/1779004799053-142764/suite.summary.json`
  with `status=passed`, 3/3 rows, `scripts_with_evidence=3`, `focus_mismatch_total=0`, and zero
  lint errors/warnings.
- First strict suite attempt:
  `target/fret-diag-dropdown-menu-strict-authoring-v1/sessions/1779004410147-126432/suite.summary.json`
  failed in the Basic typeahead script with `timeout.no_frames` after resize. The focused rerun
  passed with run id `1779004715484`, and the full v2 suite passed, so no DropdownMenu recipe or
  mechanism defect was confirmed.

## M75: ContextMenu Strict Diagnostics Authoring Gate

Status: complete for promoted ContextMenu page-entry and long-page click-visibility authoring.

- Extended `tools/check_diag_scripts_registry.py` so promoted `ui-gallery-context-menu` scripts
  join the strict page-entry and long-page click-visibility checks.
- Added focused registry self-tests proving ContextMenu page-local selectors require either
  `ui-gallery-page-context-menu` evidence or `FRET_UI_GALLERY_START_PAGE=context_menu`, and that
  ContextMenu content `click_stable` targets need a prior visibility guard.
- A dry run found zero current ContextMenu violations because the two corridor scripts already
  enter the page explicitly and guard the submenu trigger before stable right-click.
- Registry gates pass:
  `python tools/test_check_diag_scripts_registry.py` ran 33 tests, and
  `python tools/check_diag_scripts_registry.py` reports the registry is up to date.
- Runtime evidence stays green:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-context-menu --dir target/fret-diag-context-menu-strict-authoring-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  produced
  `target/fret-diag-context-menu-strict-authoring-v1/sessions/1779005731883-139360/suite.summary.json`
  with `status=passed`, 2/2 rows, `scripts_with_evidence=2`, `focus_mismatch_total=0`, overlay
  `chosen_side_counts.right=2`, and zero lint errors/warnings.
- No ContextMenu recipe, hit-test routing, or overlay placement mechanism defect was reproduced in
  this slice.

## M76: Button Group Strict Diagnostics Authoring Gate

Status: complete for promoted Button Group page-entry and long-page click-visibility authoring.

- Extended `tools/check_diag_scripts_registry.py` so promoted `ui-gallery-button-group` scripts
  join the strict page-entry and long-page click-visibility checks.
- Added focused registry self-tests proving Button Group page-local selectors require either
  `ui-gallery-page-button-group` evidence or `FRET_UI_GALLERY_START_PAGE=button_group`, and that
  Button Group content `click_stable` targets need a prior visibility guard.
- The strict dry run found three authoring violations in
  `ui-gallery-button-group-demo-screenshots.json`,
  `ui-gallery-button-group-accessibility-screenshots.json`, and
  `ui-gallery-button-group-select-screenshots.json`: each clicked a Code tab without a target-level
  window-bounds guard.
- The first strict runtime suite then exposed a second script precondition in the Select screenshot
  path. The Code tab selector existed uniquely, but its bounds were at `y=2522.6665` in a
  `720px`-tall window, so the script now scrolls
  `ui-gallery-button-group-select-content` fully into the Gallery content viewport before the
  Preview and Code captures.
- Registry gates pass:
  `python tools/test_check_diag_scripts_registry.py` ran 36 tests, and
  `python tools/check_diag_scripts_registry.py` reports the registry is up to date.
- Runtime evidence stays green:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-button-group --dir target/fret-diag-button-group-strict-authoring-v2 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  produced
  `target/fret-diag-button-group-strict-authoring-v2/sessions/1779008052527-138688/suite.summary.json`
  with `status=passed`, 13/13 rows, `scripts_with_evidence=13`, `focus_mismatch_total=0`, and
  zero lint errors/warnings.
- No new Button Group recipe, layout, or accessibility mechanism defect was reproduced in this
  slice; the confirmed issue was diagnostics authoring debt.

## M77: Moving Cached Combobox Interaction-Cache Replay Gate

Status: complete for the Resizable moving cached Combobox runtime gate and owning mechanism fix.

- Added a Resizable UI Gallery companion where a Combobox source root is inside ViewCache, starts
  in the left panel, moves to the right panel, and is opened after the move.
- The first runtime run exposed a real `fret-ui` mechanism defect: semantics and paint bounds moved
  with the cached root, but prepaint replayed interaction records at their old absolute positions,
  so hit-test routing landed on the right panel container and the Combobox input never appeared.
- `InteractionCacheEntry` now stores the cache-root origin used when the records were captured, and
  interaction-cache replay translates replayed bounds by the cache-root origin delta before
  rebuilding the current interaction cache.
- Focused guards pass for the new prepaint regression, the prepaint family filter, and the existing
  view-cache/render-transform hit-test test.
- The runtime gate now passes:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json --dir target/fret-diag-resizable-moving-cached-combobox-v6 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  with run id `1779027983606`.
- The promoted `ui-gallery-resizable` suite now passes 2/2 after adding the moving cached Combobox
  companion:
  `target/fret-diag-resizable-suite-after-moving-cached-combobox-v1/sessions/1779029073205-16452/suite.summary.json`.
- This closes the explicit cached overlay-source movement follow-on from M62 for v1.

## M78: Environment Platform-Preference Coverage Map Closeout

Status: complete for current Motion Presets platform-preference runtime evidence.

- Audited the Environment ViewCache row and found stale gap text: the coverage map still asked for
  runner/platform-injected UI Gallery diagnostics even though M68 had already added
  `ui-gallery-platform-preferences-runtime-environment-mutation.json`.
- Reran the focused runtime gate with current `dev-fast` binaries:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-platform-preferences-runtime-environment-mutation.json --dir target/fret-diag-platform-preferences-runtime-environment-mutation-v2 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe`.
- The gate passed with run id `1779029357027`, proving diagnostics-injected color scheme,
  reduced-motion, and text-scale changes reach both `/shell/window_metrics_preferences` app
  snapshots and the Motion Presets `ElementContext` environment probe.
- Updated `COVERAGE_MAP.md` and `EVIDENCE_AND_GATES.md` so the remaining Environment gap is limited
  to platform changes without a stable runtime surface, such as safe-area, forced-colors, or
  occlusion mutation.

## M79: Retained Table Runtime Suite

Status: complete for retained Table sorting, pagination, typeahead, row-pinning, and window-boundary
scroll coverage.

- Promoted `ui-gallery-table-retained` into a durable 7-row runtime suite covering keyboard
  typeahead, multi-sort shift-click, row-pinning with `keep_pinned_rows` false and true, descending
  sort, sort/select/scroll, and retained window-boundary scroll.
- The slice found a real `fret-ui-kit` retained Table defect: retained row entries did not respect
  pagination and `keep_pinned_rows`. The retained entry path now uses the shared row model and
  revision so page changes and pinned rows match policy.
- The slice also found diagnostics harness defects: no-frame keepalive ticks consumed
  `wait_frames`, and aggregate debug history predicates were blocked by latest-snapshot freshness
  before ring-history matching. Both are fixed and covered by focused `fret-bootstrap` tests.
- Several promoted scripts were hardened as script/oracle debt rather than component defects:
  `sort-desc` now targets `row-9999` after descending sort, row-pinning scripts use stable page
  status, typeahead uses `press_keys`, and touch-wheel scripts use the correct scroll direction.
- Focused `window-boundary-scroll` passes with run id `1779038066334` and AI packet at
  `target/fret-diag-table-retained-window-boundary-scroll-focused-v4/sessions/1779037981653-160576/1779038066334/ai.packet`.
- The full suite passes 7/7 with zero lint errors/warnings:
  `target/fret-diag-table-retained-suite-candidate-v4/sessions/1779038155054-118992/suite.summary.json`.

## M80: AI FileTree Auto-Height VirtualList Runtime Gate

Status: complete for AI FileTree semantics/action-state runtime coverage and the owning measured-leaf
dirtying fix.

- Promoted `ui-gallery-ai-file-tree` as the current runtime gate for FileTree hierarchy semantics,
  expanded/selected/action-state mutation, large-scroll behavior, and screenshot evidence.
- The first action-state run found a real `fret-ui` mechanism defect: expanding the FileTree updated
  rows and semantics, but the auto-height `VirtualList` measured leaf kept its old intrinsic height
  in the parent flex layout, so the following Basic Usage doc section overlapped the expanded rows.
- AI FileTree uses `cx.virtual_list_keyed_with_layout`, so this was not a retained-host stale-row
  defect. The defect was missing dirty propagation for measured leaves whose size changes with
  `len` or `items_revision`.
- VirtualList prop diffing now treats layout-affecting fields as layout dirty, and the flow builder
  marks VirtualList and ResizablePanelGroup measured Taffy nodes dirty when their UI node layout is
  invalidated.
- Focused `fret-ui` regression coverage passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui auto_height_virtual_list_len_growth_reflows_following_siblings --no-fail-fast --no-capture`.
- The VirtualList family passes 50/50:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui virtual_list --no-fail-fast`.
- The full AI FileTree suite passes 4/4:
  `target/fret-diag-ai-file-tree-semantics-action-state-after-vlist-measured-leaf-dirty-v1/sessions/1779041323318-52900/suite.summary.json`.
- Registry and formatting gates pass:
  `cargo fmt -p fret-ui --check`,
  `python tools/check_diag_scripts_registry.py`, and
  `python tools/test_check_diag_scripts_registry.py`.

## M81: AI FileTree Strict Zero-Warning Lint Gate

Status: complete for AI FileTree zero-warning diagnostics hygiene after the measured-leaf fix.

- The first green AI FileTree runtime suite still reported `layout.zero_size` warnings for
  demo-only `0 x 0` state markers used to expose selection/copy action model state.
- The markers are now hidden semantics anchors, and the scripts assert them with
  `raw_semantics_hidden_is hidden=true` rather than default visible-selector existence.
- `fret-diag` lint now ignores visible-bounds and missing-label warnings for non-focused hidden
  nodes while preserving warnings for visible zero-size test-id nodes.
- The suite now enforces `lint_policy.max_warning_issues=0`.
- Focused lint coverage passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag lint_ignores_hidden_state_anchors_for_visible_bounds_warnings --no-fail-fast --no-capture`.
- The full AI FileTree suite now passes 4/4 with zero lint errors and zero lint warnings:
  `target/fret-diag-ai-file-tree-zero-warning-hidden-markers-v2/sessions/1779043283276-169328/suite.summary.json`.

## M82: Grid Gap Layout/Measurement Fixture

Status: complete for grid column/row gap layout and intrinsic max-content measurement coverage.

- Added a focused layout primitive case for a 2x2 auto grid with independent `column_gap=8` and
  `row_gap=6`.
- The fixture locks final child placement and max-content measurement, proving the grid gap
  contract is shared between final layout and intrinsic measurement.
- The first run found an oracle mistake rather than a mechanism defect: the second row starts at
  `10 + 6 = 16`, not `18`, and the grid's max-content height is `28`.
- The corrected gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `f1be5c37-82c6-4ffe-b55a-f7f19090fd33`.

## M83: Flex Order Auto-Margin Fixture

Status: complete for flex visual-order auto-margin trailing-group post-processing.

- Added `flex-order-auto-margin-uses-visual-order`, a layout primitive case that combines
  `FlexItemStyle.order` with `margin-left: auto` in a fixed-width flex row.
- The first run exposed a real mechanism defect: the flex engine and measurement path respected
  visual order, but `layout_flex_impl_engine` still scanned source-order children for auto-margin
  tail groups and shifted the wrong siblings.
- `layout_flex_impl_engine` now routes auto-margin tail detection, tail-size computation, gap
  preservation, shift application, and final child layout iteration through
  `ordered_flex_children`.
- The corrected gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `a53d39d8-f93e-4390-b859-28b233a843a1`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M86: Flex Gap Layout/Measurement Fixture

Status: complete for flex gap final-layout and intrinsic max-content measurement consistency.

- Added `flex-gap-measure-matches-layout`, a focused layout primitive case for a two-child
  horizontal flex row with `gap=8`.
- The fixture locks both child positions and scalar metrics for final layout bounds plus
  `measure_in(MaxContent)`, proving both paths agree on the `58 x 12` envelope.
- No new mechanism defect was reproduced.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `4468bb08-582b-4be1-a09f-eb9e59149a41`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M85: Flex Margin-Top Auto Visual-Order Fixture

Status: complete for the vertical auto-margin companion used by Sheet/Drawer-style footer
placement.

- Added `flex-order-margin-top-auto-uses-visual-order`, which combines `FlexItemStyle.order` with
  `margin-top: auto` in a fixed-height vertical flex column.
- This closes the vertical auto-margin analogue for recipe surfaces that use `mt_auto()` to push
  footer/action groups to the bottom of a stack.
- No new mechanism defect was reproduced. The M83/F166 visual-order post-processing fix already
  satisfies the vertical oracle.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `93063426-0c6d-4e44-bba0-fc0bb68de234`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M84: Flex Margin-Right Auto Visual-Order Fixture

Status: complete for the right-side auto-margin companion used by RTL recipe helpers.

- Added `flex-order-margin-right-auto-uses-visual-order`, which combines `FlexItemStyle.order` with
  `margin-right: auto` in a fixed-width flex row.
- This closes the right-side auto-margin analogue for recipe surfaces that map logical auto margins
  to `mr-auto` under RTL.
- No new mechanism defect was reproduced. The current flex engine plus the M83/F166 visual-order
  post-processing fix satisfies the oracle.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `7592ea5c-a8fa-4a11-8869-8533db84bdfb`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M87: Flex Wrap Gap Layout/Measurement Fixture

Status: complete for wrapped flex gap final-layout and definite-width intrinsic max-content
measurement consistency.

- Added `flex-wrap-gap-measure-matches-layout`, a focused layout primitive case for a 68px-wide
  horizontal flex row with `wrap=true` and `gap=8`.
- The fixture locks child placement across two lines and scalar metrics for final layout bounds
  plus definite-width `measure_in(MaxContent)`, proving both paths agree on the `68 x 34` envelope.
- No new mechanism defect was reproduced.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `0353a271-aaaf-49d0-9254-f770af9ba4c1`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M88: Pressable Absolute-Only Wrapper Envelope Fixture

Status: complete for Pressable/passthrough absolute-only wrapper layout, measurement, and hit-test
envelope consistency.

- Added `pressable-fractional-absolute-child-envelope-matches-layout`, a focused layout primitive
  case for an auto/auto `Pressable` wrapper with a single absolute child using fractional left/top
  insets.
- The first run exposed a real mechanism defect: the flow engine solved the `Pressable` wrapper as
  `0 x 0` because absolute children do not contribute to ordinary flow sizing, while the widget
  path could still place and hit-test the child.
- The flow engine now models auto/auto wrappers with only absolute children as measured leaves for
  parent flow sizing, and passthrough measurement uses the shared absolute-child envelope during
  final definite probes as well as intrinsic/placeholder probes.
- The corrected gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `cedf2113-d532-4c84-b69a-728b052ae6a0`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M89: Pressable Mixed Flow/Absolute Wrapper Envelope Fixture

Status: complete for Pressable/passthrough mixed flow plus absolute child layout, measurement, and
hit-test envelope consistency.

- Added `pressable-mixed-flow-absolute-child-envelope-matches-layout`, a focused layout primitive
  case for an auto/auto `Pressable` wrapper with one normal flow child plus one fractional-inset
  absolute child that extends past the flow child.
- The first run exposed a real mechanism defect: wrapper layout and placeholder measurement used
  the flow child's `20 x 10` size while the absolute child required a `34 x 12` envelope. The
  near-edge hit-test sample missed the absolute child.
- The fix extends F171 from absolute-only wrappers to mixed wrappers: passthrough measurement
  unions auto-axis absolute envelopes with flow-child size, parent flow sizing treats auto wrappers
  with absolute children as measured leaves, and positioned-container fallback layout places flow
  children at their measured size while placing absolute children against the union envelope.
- The corrected gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `0f75010e-ebc0-4f4c-b835-aff6c0086b9d`.
- Scoped layout companion tests pass:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui declarative::tests::layout::basics declarative::tests::layout::interactivity --no-fail-fast`
  with 55/55 tests.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M90: RenderTransform Mixed Flow/Absolute Envelope Fixture

Status: complete for transformed mixed flow plus absolute child layout, measurement, visual, and
hit-test envelope consistency.

- Added `render-transform-mixed-flow-absolute-envelope-matches-visual-hit`, a focused layout
  primitive case that places the F172 mixed Pressable flow/absolute envelope under a
  `RenderTransform`.
- No new mechanism defect was reproduced. The current mechanism keeps the `34 x 12` layout and
  placeholder measurement envelope, translates visual and hit spaces together by `40px`, misses the
  layout-space near-edge sample, and hits the absolute child at the translated visual-space
  near-edge sample.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `fd0237ae-17ff-435d-a416-b34b2e8f5345`.
- JSON fixture validation passes:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\layout_primitives_v1.json`.

## M91: ViewCache Mixed Flow/Absolute Movement Fixture

Status: complete for clean-reuse ViewCache movement of a mixed flow plus absolute child wrapper.

- Added `view_cache_hit_moving_mixed_absolute_wrapper_updates_bounds_and_hit_test`, a focused
  declarative ViewCache test that reuses the F172 mixed Pressable flow/absolute envelope while a
  parent spacer moves the cached subtree from `x=0` to `x=40`.
- No new mechanism defect was reproduced. The current mechanism keeps the child render closure
  clean (`renders == 1`), moves the wrapper layout bounds to `40,0 34 x 12`, moves element visual
  bounds with layout bounds, places the absolute child at `48.5,1.2 25 x 10`, and keeps fallback
  hit-testing plus runtime routing aligned.
- The first red assertion was a harness oracle mistake: after the spacer is inserted, the old
  sample point legitimately hits the expanded outer row. The corrected oracle verifies that the old
  point no longer hits the cached absolute child and that the translated point does hit it.
- The focused gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_hit_moving_mixed_absolute_wrapper_updates_bounds_and_hit_test --no-fail-fast --no-capture`
  with Nextest run id `7e88f25a-7ea4-42c2-96d6-781f9da9482d`.
- The ViewCache family gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache --no-fail-fast`
  with Nextest run id `a49ebb69-7e9a-4b66-bc83-7f5f52d35a0e`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M92: Renderer Font Trace Predicate for Combobox Long Text

Status: complete for script-level renderer text-preparation evidence on the LTR Combobox
long-text gate.

- Added `render_text_font_trace_entries_matching_ge`, a diagnostics predicate that filters
  renderer font trace entries by text preview, `FontId`, wrap mode, overflow mode, missing glyph
  count, and optional family usage metadata.
- Promoted `ui-gallery-combobox-long-text-geometry.json` from geometry-only long-text evidence to
  renderer-backed evidence by enabling `FRET_TEXT_FONT_TRACE_ALL=1` and asserting the selected
  long label is prepared with `font=ui`, `wrap=none`, `overflow=ellipsis`, and
  `missing_glyphs=0`.
- The first runtime drafts found a diagnostics script issue, not a Combobox or renderer defect:
  bare `wait_frames` steps can stall on this static page. The corrected script uses bounded
  predicate convergence instead.
- Protocol and evaluator gates pass with Nextest run ids
  `88d1c4cf-a5e7-4b17-91bd-998df1857420` and
  `4ec491be-913d-47fa-b1e3-d7e756594342`.
- The runtime diagnostics gate passes with run id `1779077880731`.

## M93: RTL Combobox Renderer Font Trace Companion

Status: complete for renderer text-preparation evidence on the RTL Combobox long-text gate.

- Promoted `ui-gallery-combobox-rtl-long-text-geometry.json` from geometry-only RTL long-text
  evidence to renderer-backed evidence by enabling `FRET_TEXT_FONT_TRACE_ALL=1` and asserting the
  selected RTL long label is prepared with `font=ui`, `wrap=none`, `overflow=ellipsis`, and
  `missing_glyphs=0`.
- Removed bare static-page `wait_frames` steps from the RTL script and used bounded predicate
  convergence, matching the LTR gate's failure mode hardening.
- No new Combobox recipe, RTL layout, or renderer defect was reproduced. The existing physical-left
  chevron, physical-right checkmark, top-flip placement, layout sidecar, screenshot, and bundle
  evidence remain intact.
- The RTL script roundtrip gate passes with Nextest run id
  `23514d59-c3bc-4985-8c8f-d1047d32e6aa`.
- The runtime diagnostics gate passes with run id `1779078285665`.

## M94: Command Docs Demo Long-Query Renderer Font Trace Companion

Status: complete for renderer text-preparation evidence on the embedded Command search input.

- Promoted `ui-gallery-command-docs-demo-long-query-text.json` from TextInput visual clipping
  evidence to renderer-backed evidence by enabling `FRET_TEXT_FONT_TRACE_ALL=1` and asserting the
  long query is prepared with `font=ui`, `wrap=none`, `overflow=clip`, and `missing_glyphs=0`.
- Preserved the existing `value_len_ge`, bounds, IME cursor, horizontal overflow/offset,
  visible-text viewport, text-height, layout sidecar, screenshot, and bundle captures.
- Removed remaining bare static-page waits from the script and used bounded predicate convergence,
  including the runner's requested/effective viewport contract: requested `1280x1200` and
  `760x640` produce effective diagnostic windows `1280x1220` and `760x660` on the current Windows
  native runner.
- No Command recipe or renderer defect was reproduced.
- The focused script roundtrip gate passes with Nextest run id
  `347bc280-0e4c-4f1d-beed-062fd2e4903f`.
- The full `script_json_roundtrip` gate passes with Nextest run id
  `9f94f99d-86c7-417b-9b0f-5f29e4ca5797`.
- The runtime diagnostics gate passes with run id `1779080299508`.

## M95: Input/File Long-Text Renderer Font Trace Companion

Status: complete for renderer text-preparation evidence on the plain Input and file-composed Input
long-text gate.

- Promoted `ui-gallery-input-basic-and-file-long-text.json` from TextInput visual clipping evidence
  to renderer-backed evidence by enabling `FRET_TEXT_FONT_TRACE_ALL=1` and asserting both long
  values are prepared with `font=ui`, `wrap=none`, `overflow=clip`, and `missing_glyphs=0`.
- Removed cross-page navigation and bare static waits from the script by starting directly on
  `FRET_UI_GALLERY_START_PAGE=input`, waiting for the effective `760x720` diagnostic window after
  requesting `760x700`, and using predicate convergence for the two text values.
- No Input recipe, file-composition input, or renderer defect was reproduced.
- The focused script roundtrip gate passes with Nextest run id
  `4fc33be1-73de-4abd-b44e-1372c97cbe10`.
- The full `script_json_roundtrip` gate passes with Nextest run id
  `29fe790a-4b52-4b96-9d3c-0fb7677a8401`.
- The runtime diagnostics gate passes with run id `1779081108865`.

## M96: Button Group Input Group Long-Text Renderer Font Trace Companion

Status: complete for renderer text-preparation evidence on the Button Group Input Group long-text
gate.

- Promoted `ui-gallery-button-group-input-group-long-text.json` from TextInput visual clipping and
  trailing-control geometry evidence to renderer-backed evidence by enabling
  `FRET_TEXT_FONT_TRACE_ALL=1` and asserting the long grouped-input value is prepared with
  `font=ui`, `wrap=none`, `overflow=clip`, and `missing_glyphs=0`.
- Removed the remaining bare static waits from the script by starting on the Button Group Input
  Group section, waiting for the effective `760x660` diagnostic window after requesting `760x640`,
  waiting for page/root predicates, and converging on the renderer trace after `set_text_value`.
- The first runtime draft found a diagnostics precondition issue, not a Button Group or renderer
  defect: waiting for the direct control to be within the window before `set_text_value` observed
  its pre-value semantics bounds as `0 x 0`. The final script waits on the owning group root before
  mutation and keeps the existing post-mutation control-size assertions.
- The focused script roundtrip gate passes with Nextest run id
  `d656b55e-a80c-4a53-8cb0-98a8c2307872`.
- The full `script_json_roundtrip` gate passes with Nextest run id
  `72429701-d2a0-4d9f-9742-563fc421a36f`.
- The runtime diagnostics gate passes with run id `1779082851147`.

## M97: FractionalRenderTransform Visual/Hit Fixture

Status: complete for size-derived render-transform visual and hit-space consistency.

- Added `fractional-render-transform-derives-visual-hit-from-layout-size`, a focused layout
  primitive case that wraps a `20 x 20` Pressable in `FractionalRenderTransform(2.0, 0.5)`.
- The fixture locks ADR 0082 semantics for the fractional wrapper: layout stays at `0,0 20 x 20`,
  while visual and hit spaces move by `40 x 10` from the laid-out size.
- No new mechanism defect was reproduced. The current mechanism computes the fractional transform
  during layout, records transformed visual bounds during paint, and routes hit-testing through the
  translated visual center.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `8a831141-fd89-4656-be5b-59a3d206bdef`.
- JSON fixture validation passes:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\layout_primitives_v1.json`.

## M98: MaskLayer Paint-Only Hit-Test Fixture

Status: complete for MaskLayer bounds versus hit-test clipping semantics.

- Added `mask-layer-bounds-do-not-clip-hit-testing-by-default` and
  `mask-layer-overflow-clip-suppresses-escaped-child-hit` to the hit-test routing fixture.
- The cases lock ADR 0239/0273 semantics: mask coverage is paint-only and mask bounds are not an
  implicit hit-test clip, while explicit `Overflow::Clip` on the `MaskLayer` wrapper still clips
  escaped descendants.
- No new mechanism defect was reproduced. The first red run used a width-overflow child that was
  legitimately constrained to the wrapper width, so the fixture now uses an offset escaped child to
  isolate the hit-test contract.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  with Nextest run id `a72e3112-544e-405a-957d-d4d00dfad034`.
- JSON fixture validation passes:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\hit_test_routing_v1.json`.

## M99: EffectLayer Computation-Bound Hit-Test Fixture

Status: complete for EffectLayer bounds versus hit-test clipping semantics.

- Added `effect-layer-bounds-do-not-clip-hit-testing-by-default` and
  `effect-layer-overflow-clip-suppresses-escaped-child-hit` to the hit-test routing fixture.
- The cases lock ADR 0117/0118 semantics: effect bounds are computation bounds, not an implicit
  hit-test clip, while explicit `Overflow::Clip` on the `EffectLayer` wrapper still clips escaped
  descendants.
- No new mechanism defect was reproduced. The existing `fret-ui` runtime already keeps effect
  bounds separate from hit-testing and routes clipping through the explicit overflow contract.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  with Nextest run id `c31f8473-555a-4b65-996c-0648d8f85b75`.
- JSON fixture validation passes:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\hit_test_routing_v1.json`.

## M100: CompositeGroup Computation-Bound Hit-Test Fixture

Status: complete for CompositeGroup bounds versus hit-test clipping semantics.

- Added `composite-group-bounds-do-not-clip-hit-testing-by-default` and
  `composite-group-overflow-clip-suppresses-escaped-child-hit` to the hit-test routing fixture.
- The cases lock ADR 0247 semantics: compositing group bounds are computation bounds, not an
  implicit hit-test clip, while explicit `Overflow::Clip` on the `CompositeGroup` wrapper still
  clips escaped descendants.
- No new mechanism defect was reproduced. The existing `fret-ui` runtime already keeps compositing
  group bounds separate from hit-testing and routes clipping through the explicit overflow
  contract.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  with Nextest run id `eb30efc1-a5c9-40a1-84ae-8360753f0842`.
- JSON fixture validation passes:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\hit_test_routing_v1.json`.

## M101: BackdropSourceGroup Computation-Bound Hit-Test Fixture

Status: complete for BackdropSourceGroup bounds versus hit-test clipping semantics.

- Added `backdrop-source-group-bounds-do-not-clip-hit-testing-by-default` and
  `backdrop-source-group-overflow-clip-suppresses-escaped-child-hit` to the hit-test routing
  fixture.
- The cases lock ADR 0305 semantics: backdrop source group bounds are computation bounds, not an
  implicit hit-test clip, while explicit `Overflow::Clip` on the `BackdropSourceGroup` wrapper
  still clips escaped descendants.
- No new mechanism defect was reproduced. The existing `fret-ui` runtime already keeps backdrop
  source group bounds separate from hit-testing and routes clipping through the explicit overflow
  contract.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  with Nextest run id `05c28589-9f50-4aa2-b1d5-3ab3a3700de2`.
- JSON fixture validation passes:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\hit_test_routing_v1.json`.

## M102: Relative Inset Flow-Sibling Layout Primitive Fixture

Status: complete for `PositionStyle::Relative` final-position and flow-sibling semantics.

- Added `relative-inset-offsets-final-position-without-affecting-flow-siblings` to the layout
  primitive fixture.
- The case locks ADR 0062 and `element.rs` semantics: relative inset offsets move the element's
  final layout and hit-test position, but siblings still consume the original flow slot.
- No new mechanism defect was reproduced. The current flex layout path keeps the moved Pressable at
  `0,12`, the sibling at `20,0`, and routes hit-testing to the moved final position rather than the
  original flow slot.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `6a87d598-b4f7-4b0d-83c6-c9842cdb9d25`.
- JSON fixture validation passes:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\layout_primitives_v1.json`.

## M103: Static Inset Ignore Layout Primitive Fixture

Status: complete for `PositionStyle::Static` inset-ignore semantics.

- Added `static-inset-ignored-by-default-flow-position` to the layout primitive fixture.
- The case locks the opposite side of the ADR 0062 position/inset contract: inset offsets are
  ignored for default flow-positioned nodes until the element opts into relative or absolute
  positioning.
- No new mechanism defect was reproduced. The current layout path keeps the static Pressable at
  `0,0`, the sibling at `20,0`, and routes hit-testing to the original flow slot rather than the
  hypothetical inset-offset position.
- The gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `63fb7f75-45f1-4f9b-bbfa-4f20d22d7d5c`.
- JSON fixture validation passes:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\layout_primitives_v1.json`.

## M104: ViewCache Relative Inset Clean-Reuse Movement Gate

Status: complete for clean ViewCache reuse with relative-inset final-position movement.

- Added `view_cache_hit_moving_relative_inset_wrapper_updates_bounds_and_hit_test` as a focused
  `fret-ui` test.
- The case locks the cached/replayed companion to M102: a `PositionStyle::Relative` Pressable with
  `top: 12px` must keep final-position semantics when a clean ViewCache subtree is moved by an
  outer layout change without rerendering the cached child.
- No new mechanism defect was reproduced. The current runtime updates layout bounds, current visual
  bounds, fallback hit-testing, and interaction-cache routing from `0,12` to `40,12` while rendering
  the cached subtree once.
- The focused gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_hit_moving_relative_inset_wrapper_updates_bounds_and_hit_test --no-fail-fast --no-capture`
  with Nextest run id `9db3ccd2-727f-4e22-be43-bd9f6f1f4b09`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M105: Input Disabled TextInput Action-State Runtime Gate

Status: complete for disabled leaf TextInput action-state semantics.

- Added `ui-gallery-input-disabled-control` to the disabled Input TextInput builder.
- Added `ui-gallery-input-disabled-action-state.json` and promoted it into
  `ui-gallery-shadcn-runtime-evidence`.
- The runtime gate locks the concrete disabled TextInput semantics node to `disabled=true`,
  `focus=false`, and `set_value=false`, so disabled visual styling cannot mask stale accessibility
  action metadata.
- No new Input recipe defect was reproduced. Early scroll-based drafts exposed a separate
  diagnostics authoring/tooling hazard on the long Input page, so the final gate keeps the
  action-state proof independent of deep-section scroll visibility.
- The runtime gate passes:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-disabled-action-state.json --dir target\fret-diag-input-disabled-action-state-v4 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779094906772`.
- Protocol roundtrip passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_input_disabled_action_state --no-fail-fast`
  with Nextest run id `4317d185-d642-4d7b-a042-592ef62530ce`.
- Build and formatting pass:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev` and
  `cargo fmt -p fret-ui-gallery -p fret-diag-protocol --check`.

## M106: ViewCache Relative Inset Semantics Movement Gate

Status: complete for clean ViewCache reuse with relative-inset semantics bounds movement.

- Added `view_cache_semantics_moving_relative_inset_updates_bounds_without_rerender` as a focused
  `fret-ui` test.
- The case locks the semantics companion to M104: a `PositionStyle::Relative` Pressable with
  `top: 12px` must keep one semantics node and move that node's bounds when a clean ViewCache
  subtree moves without rerendering the cached child.
- No new mechanism defect was reproduced. The current runtime moves the semantics bounds from
  `0,12` to `40,12` while rendering the cached subtree once.
- The focused gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_semantics_moving_relative_inset_updates_bounds_without_rerender --no-fail-fast --no-capture`
  with Nextest run id `c013f3b5-819d-45ba-8722-ddea5139213d`.
- Formatting passes:
  `cargo fmt -p fret-ui --check`.

## M107: Retained Table Selected Semantics Focused Gate

Status: complete for focused retained Table selected-state semantics; runtime companion assertions
are authored but blocked by an existing launch/layout convergence precondition.

- Added `table_virtualized_retained_selected_semantics_follow_windowed_row_selection` in
  `fret-ui-kit`.
- The gate locks the retained Table row-selection semantics path directly: row 0 starts
  `selected=false`, a pointer click refreshes row 0 to `selected=true`, then scrolling to row 25
  keeps row 25 `selected=false` and detaches row 0 from the current semantics snapshot.
- Added `selected_is` assertions to `ui-gallery-table-retained-sort-select-scroll.json` and
  roundtrip tests for both retained Table sort/select/scroll and window-boundary scripts.
- No retained Table selected-semantics defect was reproduced. The focused runtime rerun timed out
  before selected assertions at `bounds_within_window(ui-gallery-table-retained-header-row)`, where
  the forced bundle still showed the retained Table subtree at `0,0 0x0`.
- The focused gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_selected_semantics_follow_windowed_row_selection --no-fail-fast --no-capture`
  with Nextest run id `bfefef11-f3dc-435a-a986-1d0cc16666d2`.
- Protocol roundtrip and registry gates pass:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_table_retained_sort_select_scroll script_v2_roundtrip_ui_gallery_table_retained_window_boundary_scroll --no-fail-fast`
  with Nextest run id `c6dd9233-dda9-48fd-8fde-c510fc6d9ac1`, and
  `python tools\check_diag_scripts_registry.py`.

## M108: Retained Table Header Bounds Flex Snapshot Fix

Status: complete for the retained Table direct-start header-row bounds convergence defect.

- Fixed a `fret-ui` flex mechanism defect in `layout_flex_impl_engine`: recursive child layout could
  invalidate later sibling solved rects while the same final layout pass still needed those rects.
- The flex path now snapshots ordered child rects before recursive child layout and uses that
  snapshot for auto-margin tail computation, gap preservation, shifts, and final child layout.
- Added `table_retained_torture_direct_start_header_bounds_converge` in the Gallery driver tests to
  lock the direct-start retained Table header row at non-zero bounds.
- Removed temporary debug probes and Gallery-side layout workarounds; the fix lives in the mechanism
  layer.
- Focused gates pass:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --features gallery-dev table_retained_torture_direct_start_header_bounds_converge --no-fail-fast --no-capture`
  with Nextest run id `e84b549f-2b87-4faa-afb2-969c294ae01e`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `faa8f32d-8f3e-4831-aa95-00e1861f831b`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_selected_semantics_follow_windowed_row_selection --no-fail-fast --no-capture`
  with Nextest run id `9951e6c7-722f-4713-be3f-797dd2d01a6e`; and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_table_retained_sort_select_scroll script_v2_roundtrip_ui_gallery_table_retained_window_boundary_scroll --no-fail-fast`
  with Nextest run id `f8df7c84-6a3e-4dde-bbbe-0c0e31546407`.
- Runtime diagnostics now pass the previous header-row bounds precondition and fail later at the row
  selected-state assertion because the row click hit-tests the enclosing `scroll_bar`; that is the
  next isolated follow-up, not part of this flex fix.

## M109: Retained Table Scrollbar Hit Region Absolute-Size Fix

Status: complete for the retained Table sort/select/scroll row-click hit-region defect.

- Fixed a `fret-ui` positioned-layout mechanism defect: manual absolute-child layout paths carried
  only `InsetStyle`, so an absolute wrapper with explicit size could lose that size and let
  fill-sized descendants expand to the full probe bounds.
- The concrete runtime symptom was the retained Table row click from F191 landing on the enclosing
  ScrollArea `scroll_bar` instead of `ui-gallery-table-retained-row-0`.
- `PositionedLayoutStyle::Absolute` now carries `InsetStyle + SizeStyle`, and
  `layout_absolute_child_with_probe_bounds` resolves explicit width/height for absolute children
  whose axis is pinned on only one side.
- Added `absolute_interactivity_gate_preserves_scrollbar_track_bounds` in `fret-ui` to lock the
  ScrollArea-style gate/opacity/scrollbar chain to a `10px` right track and prove content hits do
  not share the scrollbar target.
- Updated the retained Table sort/select/scroll script's post-scroll unselected-row oracle to the
  stable visible row `ui-gallery-table-retained-row-10015`.
- Focused gates pass:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui absolute_interactivity_gate_preserves_scrollbar_track_bounds --no-fail-fast --no-capture`
  with Nextest run id `ae114762-9f8e-4ac9-9594-606305eee7ec`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  with Nextest run id `7163fc89-31dd-4f1d-a2ef-ba2e522dac41`; and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_table_retained_sort_select_scroll --no-fail-fast --no-capture`
  with Nextest run id `67a19613-9301-4ef6-98a4-e20af5bff6b4`.
- Runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\table\ui-gallery-table-retained-sort-select-scroll.json --dir target\fret-diag-table-retained-selected-sort-select-scroll-after-absolute-size-fix-current --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  with run id `1779121343180`.

## M110: Retained DataTable Selected-State Runtime Companion

Status: complete for the retained DataTable sort/select/scroll selected-state diagnostics companion.

- Extended `ui-gallery-data-table-retained-sort-select-scroll.json` with `selected_is` assertions on
  the real DataTable torture retained path.
- The runtime proof covers row 0 `selected=false` before the click, row 0 `selected=true` after the
  click, and post-scroll row `ui-gallery-data-table-row-10015` `selected=false`.
- Added `script_v2_roundtrip_ui_gallery_data_table_retained_sort_select_scroll` so the redirect
  script and new selected predicates are covered by protocol roundtrip.
- The protocol gate passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_data_table_retained_sort_select_scroll --no-fail-fast --no-capture`
  with Nextest run id `a23c2c5e-e7a7-499b-b2c3-62b73ed5ffd8`.
- Runtime diagnostics pass:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-sort-select-scroll.json --dir target\fret-diag-data-table-retained-selected-sort-select-scroll-current --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev`
  with run id `1779122449287`.

## M111: UI Gallery Node Graph Cull Runtime Gate Coverage

Status: complete for Node Graph Cull cull-window runtime evidence and protocol roundtrip coverage.

- Reused the existing Node Graph Cull diagnostics scripts and suite manifests rather than creating a
  new script surface.
- Added protocol roundtrip tests for the three redirect entry points:
  `ui-gallery-node-graph-cull-torture-pan-zoom`,
  `ui-gallery-node-graph-cull-window-shifts`, and
  `ui-gallery-node-graph-cull-window-no-shifts-small-pan`.
- The runtime suites cover complementary cull-window behavior: pan/zoom smoke, large-pan cull
  window shifts, and a small-pan zero-shift guard.
- No mechanism defect was reproduced. The earlier `ensure_visible_timeout` was a stale-binary
  false failure: `target/debug/fret-ui-gallery.exe` did not expose
  `ui-gallery-nav-node-graph-cull-torture` after nav search, while the rebuilt
  `target/dev-fast/fret-ui-gallery.exe` did.
- Protocol roundtrip passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_node_graph_cull --no-fail-fast --no-capture`
  with Nextest run id `fad3a59e-43d7-47b4-9183-81ae290e61d5`.
- Runtime diagnostics suites pass:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-node-graph-cull --dir target/fret-diag-node-graph-cull-suite-current --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  with run id `1779124205290`;
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-node-graph-cull-window-shifts --dir target/fret-diag-node-graph-cull-window-shifts-suite-current --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  with run id `1779124258887`; and
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-node-graph-cull-window-no-shifts-small-pan --dir target/fret-diag-node-graph-cull-window-no-shifts-small-pan-suite-current --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  with run id `1779124253283`.

## M112: UI Gallery Canvas Cull Runtime Gate Stabilization

Status: complete for Canvas Cull pan/zoom runtime evidence and protocol roundtrip coverage.

- Added `script_v2_roundtrip_ui_gallery_canvas_cull_torture_pan_zoom` so the Canvas Cull redirect
  script is covered by `fret-diag-protocol` roundtrip.
- Hardened `ui-gallery-canvas-cull-torture-pan-zoom.json` to enter the long Gallery nav through
  `ui-gallery-nav-search`, type `canvas cull`, `ensure_visible` the target row, and then click the
  Canvas Cull torture page.
- The first runtime suite exposed a diagnostics authoring defect: the old direct click found
  `ui-gallery-nav-canvas-cull-torture`, but its live bounds were at `y=993.3` in a `720px` window,
  so the click was clamped out of the window and hit-tested `no_hit`.
- No Canvas Cull mechanism defect was reproduced after the script entry fix.
- JSON validation passes:
  `python -m json.tool tools\diag-scripts\ui-gallery\perf\ui-gallery-canvas-cull-torture-pan-zoom.json > $null`.
- Protocol roundtrip passes:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_canvas_cull_torture_pan_zoom --no-fail-fast --no-capture`
  with Nextest run id `b32d1fd3-74f8-46c3-893f-1bee7b5d65f6`.
- Runtime diagnostics pass:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-canvas-cull --dir target/fret-diag-canvas-cull-suite-after-search-entry --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  with run id `1779125873114`; `check.pixels_changed.json` also passed for
  `ui-gallery-canvas-cull-root`.

## M113: Strict Gallery Nav Click Visibility Authoring Lint

Status: complete for the cull/torture promoted-suite nav-click authoring guard.

- Added `STRICT_NAV_CLICK_VISIBILITY_SUITES` to `tools/check_diag_scripts_registry.py` for:
  `ui-gallery-canvas-cull`, `ui-gallery-chart-torture`, `ui-gallery-node-graph-cull`,
  `ui-gallery-node-graph-cull-window-shifts`, and
  `ui-gallery-node-graph-cull-window-no-shifts-small-pan`.
- The new lint rejects `click` or `click_stable` on page-row `ui-gallery-nav-*` targets unless the
  same target has a prior `ensure_visible(within_window=true)` or
  `scroll_into_view(require_fully_within_window=true)` guard.
- `ui-gallery-nav-search` and `ui-gallery-nav-scroll` are exempt because they are nav controls, not
  long-list page rows.
- Registry self-tests now cover the bad Canvas Cull direct-nav-click case, the guarded case, and
  the nav-search exemption.
- Gates pass:
  `python tools/test_check_diag_scripts_registry.py`
  with 39/39 tests passed, and
  `python tools/check_diag_scripts_registry.py`.

## M114: Chart Torture Sampling-Window Runtime Gate

Status: complete for Chart Torture pan/zoom sampling-window telemetry.

- Hardened `chart_sampling_window_shifts_min` so `min_actions > 1` requires distinct nonzero
  `chart_sampling_window_key` values and writes `distinct_key_count` plus sample `node` evidence.
- Raised the `ui-gallery-chart-torture` default post-run gate from one sampling-window action to
  two distinct sampling-window keys.
- Fixed the Chart Torture page to keep a shared delinea `ChartEngine` in a stable local model and
  use dataZoom-backed X/Y axes, so retained/cached root recreation no longer resets pan/zoom state
  before the diagnostics post-run check.
- Added a delinea headless regression test proving `PanDataWindowXFromBase` and
  `ZoomDataWindowXFromBase` publish changed `output.axis_windows` when `dataZoomX` is present.
- Focused gates pass:
  `cargo nextest run --cargo-profile dev-fast -p delinea interactive_data_zoom_x_pan_and_zoom_updates_output_axis_window --no-fail-fast --no-capture`
  with Nextest run id `9424544b-9b5b-4ac8-849b-61d2fd6bd6ec`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag chart_sampling_window_shifts_min build_suite_core_default_post_run_checks_sets_chart_torture_sampling_window_gate --no-fail-fast --no-capture`
  with Nextest run id `01f08348-f52b-4423-872d-cf0c3d0f1b00`; and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  with Nextest run id `3acf2b68-8b37-41da-8c73-e25f5820e177`.
- Runtime diagnostics pass:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target/fret-diag-chart-torture-suite-shared-engine-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  with run id `1779130059382`; the sampling-window evidence records `distinct_key_count=2`.

## M115: Chart Torture DataZoom Runtime Oracle

Status: complete for Chart Torture shared-engine dataZoom state assertions.

- Added a Gallery app snapshot payload for Chart Torture:
  `app_snapshot.chart_torture.engine_present`,
  `app_snapshot.chart_torture.x_data_zoom.active`, the rounded dataZoom window, and supplemental
  `ChartCanvasOutput` model counters.
- Updated `ui-gallery-chart-torture-pan-zoom.json` so the runtime gate asserts the shared engine
  starts with `x_data_zoom.active=false` and becomes `true` after scripted drag/wheel interaction.
- The first runtime draft exposed an oracle design bug, not a chart defect: `ChartCanvasOutput` is
  paint-published, so ViewCache replay can leave the output model at revision `0` before
  interaction. The final gate reads the shared `ChartEngine` state and still records output-model
  counters when paint later publishes them.
- Gates pass:
  `cargo fmt -p fret-ui-gallery --check`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  with Nextest run id `b48b7c47-8d4a-4d7d-8923-c3451a4060fe`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-chart,gallery-dev`; and
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target/fret-diag-chart-torture-suite-output-oracle-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  with run id `1779131647234`.

## M116: Chart Torture Tooltip and Axis Output Oracle

Status: complete for Chart Torture paint-published output payload assertions.

- Promoted the `ChartCanvasOutput` counters from supplemental evidence into hard script waits after
  pan/zoom interaction.
- `ui-gallery-chart-torture-pan-zoom.json` now asserts:
  `x_axis_output_window.present=true`,
  `output_model.domain_windows_count=2`, and
  `output_model.tooltip_lines_count=2`.
- This locks a stronger chart-specific runtime contract: after shared-engine pan/zoom, the
  paint-published output model must expose both domain windows and tooltip/axis-pointer text.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  with Nextest run id `2a111ce6-47bf-4c4d-8a1c-4f18abfb29a2`;
  `python tools/check_diag_scripts_registry.py`; and
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target/fret-diag-chart-torture-suite-tooltip-output-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  with run id `1779132056758`.

## M117: Workspace Shell Tabstrip Overflow Selection Gate

Status: complete for workspace shell tabstrip overflow selection and visible-order tab command
runtime coverage.

- Added `ui-gallery-workspace-tabstrip-overflow-select-command.json` and promoted it into the
  `ui-gallery-workspace-shell` suite.
- The script starts UI Gallery with `FRET_UI_GALLERY_DIAG_PROFILE=workspace_shell`, keeps the
  selected page at Overlay, resizes the window to `900 x 720` so the tabstrip overflows without
  collapsing the top-bar center column, opens the overflow menu, selects the hidden Command tab,
  and asserts `workspace_tab_strip_active_overflow_is`, active visible state, tab selection, and
  `/selected_page == "command"`.
- The full suite exposed a UI Gallery policy drift: keyboard/runtime tab commands handled by
  `WorkspaceCommandScope` used the workspace crate's default MRU tab cycle while the Gallery
  driver fallback and existing command smoke expected visible-order cycling. UI Gallery now
  configures its single-pane workspace layout with `TabCycleMode::InOrder`, while the workspace
  crate default remains MRU for editor-style consumers.
- Added a focused UI Gallery regression test for the runtime layout-command path:
  `workspace_layout_tab_next_uses_gallery_visible_order`.
- Gates pass:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery workspace_layout_tab_next_uses_gallery_visible_order --no-fail-fast --no-capture`
  with Nextest run id `c7040c5e-c9cd-4bdc-a4f4-62fc939cffd2`;
  `cargo nextest run --cargo-profile dev-fast -p fret-workspace tabs::tests::mru_next_toggles_between_two_most_recent --no-fail-fast --no-capture`
  with Nextest run id `f11787fb-15b0-488b-951c-2fc272c9f5ee`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_workspace_tabstrip_overflow_select_command --no-fail-fast --no-capture`
  with Nextest run id `9d9ea3d1-66b3-4db3-9a57-cde718d027b0`;
  `python tools/test_check_diag_scripts_registry.py`;
  `python tools/check_diag_scripts_registry.py`; and
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-workspace-shell --dir target/fret-diag-workspace-shell-suite-after-overflow-inorder-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with run ids `1779136173632`, `1779136260333`, `1779136295210`, and `1779136342864`.

## M118: Workspace Shell Demo Tab Movement Ownership Gate

Status: complete for workspace shell demo tab drag ownership, end-drop resolution, and overflow
reorder runtime coverage.

- Added specific-tab reorder command ids:
  `workspace.tab.move_before_id.<dragged_len>:<dragged><target>` and
  `workspace.tab.move_after_id.<dragged_len>:<dragged><target>`.
- `WorkspaceTabs::move_tab_relative_to` now moves the dragged tab instead of assuming the active tab
  is still the dragged tab when the command is applied. Focused tests lock active-tab independence
  and pinned-boundary rejection.
- Tab strip end-drop now resolves to a concrete canonical after-target before dispatch, so dropping
  on the explicit `drop_end` surface records the same target semantics as dropping beside a visible
  tab.
- Local tab strip drag state and pane-tree tab drag state now use stable window/root model keys so
  local state survives tabstrip subtree rebuilds during overflow/scroll/reorder frames.
- The workspace shell demo row that hosts the tab strip now uses auto height and wrapping, keeping
  overflow controls reachable under the `420 x 720` constrained runtime gate.
- The final overflow-reorder runtime failure was a diagnostics authoring issue, not a runtime
  reorder defect: `doc-a-0` was clipped, and dragging from its stale/clipped semantic bounds did
  not hit the tab. The script now activates `doc-a-0` from the overflow menu, waits for the active
  tab to become visible, and then drags it to `drop_end`.
- The rebuilt suite also exposed an app-shell ownership defect: the demo runner applied
  `workspace.*` commands to its app-owned model, then `WorkspaceCommandScope` replayed the same
  command to that model. The demo now disables workspace-model command replay on the scope while
  keeping focus-transfer hooks active.
- Gates pass:
  `rustfmt --edition 2024 --check apps/fret-examples/src/workspace_shell_demo.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs ecosystem/fret-workspace/src/command_scope.rs ecosystem/fret-workspace/src/commands.rs ecosystem/fret-workspace/src/panes.rs ecosystem/fret-workspace/src/tab_strip/drag_state.rs ecosystem/fret-workspace/src/tab_strip/intent.rs ecosystem/fret-workspace/src/tab_strip/mod.rs ecosystem/fret-workspace/src/tabs.rs`
  passed;
  `git diff --check` passed;
  `cargo test --profile dev-fast -p fret-workspace --lib end_drop_release_resolves_to_specific_after_target -- --nocapture`
  passed;
  `cargo test --profile dev-fast -p fret-workspace --lib move_specific_tab_before_after_does_not_depend_on_active_tab -- --nocapture`
  passed;
  `cargo test --profile dev-fast -p fret-workspace --lib move_specific_tab_commands_do_not_cross_pinned_boundary -- --nocapture`
  passed;
  `cargo test --profile dev-fast -p fret-workspace --test workspace_command_scope_focus_tab_strip_from_outside_pane -- --nocapture`
  passed;
  `python tools/check_diag_scripts_registry.py` passed;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_cross_pane_move_to_end script_v2_roundtrip_workspace_shell_demo_tab_overflow_activate_hidden_smoke --no-fail-fast --no-capture`
  passed with Nextest run id `53b23aaa-eca2-43aa-8e4b-201a0ed6f152`;
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/workspace-shell-demo-tab-pin-commits-preview-smoke.json --dir target/fret-diag-workspace-shell-demo-pin-preview-after-scope-owner-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target/dev-fast/workspace_shell_demo.exe`
  passed with run id `1779147052955`; and
  `target/dev-fast/fretboard-dev.exe diag suite workspace-shell-demo --dir target/fret-diag-workspace-shell-demo-suite-after-scope-owner-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/workspace_shell_demo.exe`
  passed with suite summary
  `target/fret-diag-workspace-shell-demo-suite-after-scope-owner-v1/sessions/1779147074217-22776/suite.summary.json`
  and overflow reorder run id `1779147195864`.

## M119: Workspace Shell Demo Dirty Close Button Gate

Status: complete for dirty-close policy under widget-dispatched tab close commands.

- Added `workspace-shell-demo-tab-close-button-dirty-shows-prompt-smoke.json` and promoted it into
  the `workspace-shell-demo` suite.
- The script marks the active `doc-a-2` tab dirty, clicks the real tab close button
  `workspace-shell-pane-pane-a-tab-doc-a-2.close`, and asserts the pointer dispatch trace records
  `workspace.tab.close.doc-a-2` before the dirty-close prompt appears.
- The script cancels the first prompt and verifies the dirty tab and dirty marker remain, then
  repeats the real close-button path and discards to verify the tab is removed.
- The first runtime failure exposed an app-shell redraw defect: `handle_command` installed the
  blocked dirty-close prompt model but only requested redraw for applied outcomes or UI-driver
  fallback dispatch. The demo now redraws for `blocked_dirty_close`.
- The companion `fret-workspace` unit test proves close-by-id commands can be blocked by
  `BlockDirtyClosePolicy` without removing the dirty tab.
- Runtime authoring note: while the modal prompt is open, the diagnostics modal barrier filters
  background tab selectors. The script intentionally clicks Cancel before asserting preserved
  background tab state.
- Gates pass:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-button-dirty-shows-prompt-smoke.json > $null`;
  `cargo test --profile dev-fast -p fret-workspace --lib dirty_close_policy_can_block_close_by_id -- --nocapture`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_button_dirty_shows_prompt_smoke --no-fail-fast --no-capture`
  with Nextest run id `4c8b4510-ec3f-421a-b0dc-826a0faa27ed`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps\fret-examples\src\workspace_shell_demo.rs ecosystem\fret-workspace\src\tabs.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`; and
  `git diff --check`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace-shell-demo-tab-close-button-dirty-shows-prompt-smoke.json --dir target\fret-diag-workspace-shell-demo-dirty-close-widget-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with run id `1779148945096` and AI packet
  `target/fret-diag-workspace-shell-demo-dirty-close-widget-v3/sessions/1779148942029-109108/1779148945096/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-dirty-close-widget-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with suite summary
  `target/fret-diag-workspace-shell-demo-suite-dirty-close-widget-v1/sessions/1779148963907-13484/suite.summary.json`
  and 11/11 scripts passed.

## M120: Workspace Shell Demo Close Others Dirty Aggregation Gate

Status: complete for aggregate dirty-close policy under context-menu Close Other Tabs commands.

- Added `workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json` and promoted it into
  the `workspace-shell-demo` suite.
- The script marks `doc-a-0` and `doc-a-1` dirty, activates `doc-a-2`, invokes `Close Other Tabs`
  from the real tab context menu, and asserts the pointer command dispatch trace records
  `workspace.tab.close.others`.
- The dirty-close prompt now exposes a stable diagnostics label containing the reason, active tab,
  close-count, and dirty target list. The runtime gate asserts `reason=CloseOthers`,
  `active=doc-a-2`, `close_count=2`, and `dirty=[doc-a-0, doc-a-1]`.
- The first runtime drafts exposed diagnostics authoring defects, not runtime defects: direct tab
  clicking did not make `doc-a-0` selected in this shell state, and `arrowright` was not a valid
  key token. The final script uses the existing content-focus plus tabstrip keyboard-selection
  pattern and the valid `arrow_right` key token.
- The companion `fret-workspace` unit test proves `CloseOthers` dirty-close requests aggregate
  multiple non-pinned, non-active targets while leaving pinned and active dirty tabs out of the
  close target set.
- Gates pass:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json > $null`;
  `python -m json.tool tools\diag-scripts\workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json > $null`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo test --profile dev-fast -p fret-workspace --lib dirty_close_policy_can_block_close_others_with_multiple_targets -- --nocapture`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_others_dirty_aggregation_smoke --no-fail-fast --no-capture`
  with Nextest run id `c5d88a4e-1708-43e1-aac0-39bd3f49db41`;
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`; and
  `rustfmt --edition 2024 --check apps\fret-examples\src\workspace_shell_demo.rs ecosystem\fret-workspace\src\tabs.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json --dir target\fret-diag-workspace-shell-demo-close-others-dirty-aggregation-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with run id `1779150581545` and AI packet
  `target/fret-diag-workspace-shell-demo-close-others-dirty-aggregation-v3/sessions/1779150577000-104072/1779150581545/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-close-others-dirty-aggregation-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with suite summary
  `target/fret-diag-workspace-shell-demo-suite-close-others-dirty-aggregation-v1/sessions/1779150610325-113064/suite.summary.json`;
  12/12 scripts passed and the new aggregate script run id is `1779150627934`.

## M121: Workspace Shell Demo Cross-Pane Close Button Ownership Gate

Status: complete for real close-button ownership when the clicked tab belongs to a non-active pane.

- Added `workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json` and promoted it into
  the `workspace-shell-demo` suite.
- The script starts from the default split layout where `pane-a` is the active pane and `pane-b` is
  visible with selected `doc-b-1`.
- It clicks the real `workspace-shell-pane-pane-b-tab-doc-b-1.close` button, asserts the pointer
  path dispatches `workspace.pane.activate.pane-b`, then asserts `workspace.tab.close.doc-b-1`
  dispatches from the same close button.
- The final state proves command ownership by checking `doc-b-1` is removed, `doc-b-0` remains and
  becomes selected with set size `1`, and pane-a's active `doc-a-2` remains present.
- No runtime mechanism defect was reproduced. The existing tab close interaction already carries a
  pane-activate command in the close press state and dispatches it before the close command, so the
  app-owned `WorkspaceWindowLayout` routes the close to the correct pane.
- Gates pass:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json > $null`;
  `python -m json.tool tools\diag-scripts\workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json > $null`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_cross_pane_button_ownership_smoke --no-fail-fast --no-capture`
  with Nextest run id `dfb8718a-4d49-4fbe-aacd-05b732b5f971`;
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`; and
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json --dir target\fret-diag-workspace-shell-demo-cross-pane-close-ownership-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with run id `1779151906508` and AI packet
  `target/fret-diag-workspace-shell-demo-cross-pane-close-ownership-v1/sessions/1779151900949-103552/1779151906508/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-cross-pane-close-ownership-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with suite summary
  `target/fret-diag-workspace-shell-demo-suite-cross-pane-close-ownership-v1/sessions/1779152081871-77896/suite.summary.json`;
  13/13 scripts passed and the new cross-pane close ownership script run id is `1779152100416`.

## M122: Workspace Shell Demo Cross-Pane Close Others Context-Menu Ownership Gate

Status: complete for real context-menu `Close Other Tabs` ownership when the context-clicked tab
belongs to a non-active pane.

- Added `workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json` and
  promoted it into the `workspace-shell-demo` suite.
- The script starts from the default split layout where `pane-a` is active and `pane-b` is visible
  with selected `doc-b-1`.
- It right-clicks `workspace-shell-pane-pane-b-tab-doc-b-1`, waits for a handled
  `workspace.pane.activate.pane-b`, clicks the real menu item
  `workspace-shell-pane-pane-b-tab-doc-b-1.menu.close_others`, and asserts
  `workspace.tab.close.others` dispatches from that menu item.
- The final state proves command ownership by checking pane-b's other tab `doc-b-0` is removed,
  `doc-b-1` remains selected with set size `1`, and pane-a's `doc-a-0`, `doc-a-1`, and selected
  `doc-a-2` remain present.
- No runtime ownership defect was reproduced. The first focused runtime draft did expose a
  diagnostics source-attribution gap: the right-click-triggered pane activation is recorded as
  programmatic and driver-handled rather than pointer-sourced. The final gate asserts the handled
  activation command and keeps the pointer-source assertion on the actual aggregate close command.
  M123 closes this diagnostics follow-up by making pane activation record pointer source metadata.
- Gates pass:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json > $null`;
  `python -m json.tool tools\diag-scripts\workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_others_cross_pane_context_menu_ownership_smoke --no-fail-fast --no-capture`
  with Nextest run id `e7ce6c13-3096-4fb6-a9f1-7a5c81409066`;
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`; and
  `git diff --check`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json --dir target\fret-diag-workspace-shell-demo-cross-pane-context-close-others-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with run id `1779152893863` and AI packet
  `target/fret-diag-workspace-shell-demo-cross-pane-context-close-others-v2/sessions/1779152888206-118016/1779152893863/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-cross-pane-context-close-others-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with suite summary
  `target/fret-diag-workspace-shell-demo-suite-cross-pane-context-close-others-v1/sessions/1779153282522-114068/suite.summary.json`;
  14/14 scripts passed and the new context-menu Close Others ownership script run id is
  `1779153324733`.

## M123: Workspace Shell Demo Right-Click Pane Activation Source Attribution

Status: complete for pointer-source attribution on pane activation triggered by a right-clicked tab.

- Fixed the pane-level pointer activation hook in `fret-workspace` so it records pending command
  dispatch source before dispatching `workspace.pane.activate.<id>`.
- The source attribution uses the pointer event's hit pressable target when available, so
  right-clicking a context-menu-wrapped tab attributes pane activation to the tab pressable rather
  than the pane container.
- Strengthened
  `workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json` to assert
  `workspace.pane.activate.pane-b` is `source_kind=pointer` and
  `source_test_id=workspace-shell-pane-pane-b-tab-doc-b-1`.
- Gates pass:
  `rustfmt --edition 2024 --check ecosystem\fret-workspace\src\panes.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  `git diff --check`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_others_cross_pane_context_menu_ownership_smoke --no-fail-fast --no-capture`
  with Nextest run id `bb71150c-c340-4217-9dee-e71eaab872f9`;
  `cargo nextest run --cargo-profile dev-fast -p fret-workspace --lib --no-fail-fast`
  with Nextest run id `8e889da8-a462-49bf-9685-1bb9750deba6`; and
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json --dir target\fret-diag-workspace-shell-demo-cross-pane-context-close-others-source-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with run id `1779156237310` and AI packet
  `target/fret-diag-workspace-shell-demo-cross-pane-context-close-others-source-v1/sessions/1779156234065-53332/1779156237310/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-cross-pane-context-source-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with suite summary
  `target/fret-diag-workspace-shell-demo-suite-cross-pane-context-source-v1/sessions/1779156335684-11164/suite.summary.json`;
  14/14 scripts passed and the strengthened context-menu Close Others ownership script run id is
  `1779156370933`.

## M124: Retained DataTable Column Actions And Stale Script Gates

Status: complete for retained DataTable column-action state, scoped toolbar selectors, and the
observable window-boundary runtime gate.

- Strengthened `ui-gallery-data-table-retained-column-actions-menu.json` so the real retained
  DataTable menu path asserts pointer-sourced command dispatch for
  `fret_ui_shadcn.data_table.column_action/pin_left/mem_mb`,
  `fret_ui_shadcn.data_table.column_action/sort_asc/mem_mb`, and
  `fret_ui_shadcn.data_table.column_action/hide/mem_mb`.
- The gate now proves the retained UI/model state does not split after hiding `mem_mb`: the column
  is absent from the visible table, `Sorting: mem_mb asc` and
  `Pinning: left=[mem_mb] right=[]` remain present, and the Columns menu reports `mem_mb`
  unchecked.
- Fixed retained DataTable toolbar script drift by replacing old unscoped toolbar ids with the
  current `ui-gallery-data-table-torture-toolbar-*` ids in the faceted-filter, reset-filters, and
  dashed-border screenshot scripts.
- Reworked `ui-gallery-data-table-window-boundary-scroll-retained.json` from repeated
  `wheel + wait_frames` steps to deterministic observable row-window assertions: select row 0,
  touch-wheel to row 25, prove row 0 is detached, then wheel back and prove row 0 returns.
- Added protocol roundtrip coverage for the DataTable retained window-boundary script so schema
  mistakes in that promoted gate are caught before runtime.
- No retained DataTable mechanism or recipe stale-state defect was reproduced. The red runs exposed
  diagnostics authoring drift instead: scoped toolbar ids had changed, the old window-boundary
  script could stall on a static frame with `timeout.no_frames`, and the DataTable path did not
  produce a stable retained-reconcile counter matching the attempted oracle.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-window-boundary-scroll-retained.json > $null`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_data_table_retained_column_actions_menu script_v2_roundtrip_ui_gallery_data_table_retained_faceted_filter script_v2_roundtrip_ui_gallery_data_table_retained_reset_filters script_v2_roundtrip_ui_gallery_data_table_retained_window_boundary_scroll --no-fail-fast --no-capture`
  with Nextest run id `96dcbf8b-13fa-48da-bae6-c930fad77b04`.
- Focused runtime diagnostics pass:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-column-actions-menu.json --dir target\fret-diag-data-table-retained-column-actions-menu-state-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with run id `1779157628043` and AI packet
  `target/fret-diag-data-table-retained-column-actions-menu-state-v2/sessions/1779157546485-30336/1779157628043/ai.packet`.
- Focused window-boundary diagnostics pass:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-window-boundary-scroll-retained.json --dir target\fret-diag-data-table-window-boundary-scroll-retained-deterministic-v5 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with run id `1779160262364` and AI packet
  `target/fret-diag-data-table-window-boundary-scroll-retained-deterministic-v5/sessions/1779160251641-54688/1779160262364/ai.packet`.
- Full retained DataTable suite diagnostics pass:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag suite ui-gallery-data-table-retained --dir target\fret-diag-data-table-retained-suite-column-actions-state-v3 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-data-table-retained-suite-column-actions-state-v3/sessions/1779160314350-36592/suite.summary.json`;
  12/12 scripts passed and the strengthened column-actions run id is `1779160434776`.

## M125: DataTable View-Cache Filter-Shrink Inputs-Change Gate Hardening

Status: complete for self-contained launch configuration and protocol coverage of the non-retained
view-cache DataTable filter-shrink gate.

- Strengthened
  `ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json` with
  `required_launch_features=["gallery-dev"]` and `env_defaults.FRET_UI_GALLERY_VIEW_CACHE=1`, so
  promoted suite runs do not rely on caller-provided environment setup.
- Added an app-snapshot assertion for `/view_cache/enabled=true` before the script applies the
  global filter. This proves the later `non_retained_rerender` and
  `scroll_handle_inputs_change_window_update` assertions are being exercised in the intended
  Gallery view-cache mode.
- Added direct `fret-diag-protocol` roundtrip coverage for the script.
- No new mechanism or DataTable recipe defect was reproduced. The existing non-retained view-cache
  filter-shrink invalidation-detail path still passes; this slice closes the weaker gate hygiene
  around launch feature/env proof and schema coverage.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_data_table_view_cache_filter_shrink_vlist_inputs_change --no-fail-fast --no-capture`
  with Nextest run id `19530940-8e8e-477e-9b3e-80f8f0190843`; and
  `git diff --check`.
- Focused runtime diagnostics pass without manually setting `FRET_UI_GALLERY_VIEW_CACHE`:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json --dir target\fret-diag-data-table-view-cache-filter-shrink-env-default-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with run id `1779161694881` and AI packet
  `target/fret-diag-data-table-view-cache-filter-shrink-env-default-v1/sessions/1779161683796-54152/1779161694881/ai.packet`.
- Suite diagnostics pass without manually setting `FRET_UI_GALLERY_VIEW_CACHE`:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target\fret-diag-data-table-view-cache-suite-env-default-v1 --session-auto --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-data-table-view-cache-suite-env-default-v1/sessions/1779161746388-13892/suite.summary.json`;
  1/1 scripts passed and the script run id is `1779161756820`.

## M126: UI Gallery View Cache Model-Mutation Protocol Gate

Status: complete for protocol coverage and fresh runtime evidence on the View Cache harness gate.

- Added direct `fret-diag-protocol` roundtrip coverage for
  `ui-gallery-view-cache-model-mutation-through-cache.json`.
- The script already self-configures `FRET_UI_GALLERY_START_PAGE=view_cache`,
  `FRET_UI_GALLERY_VIEW_CACHE=1`, and `FRET_UI_GALLERY_VIEW_CACHE_INNER=1`, then asserts
  `/view_cache/enabled=true` and `/view_cache/inner_enabled=true` before mutating the cached
  subtree counter and Popover state.
- No new mechanism defect was reproduced. The fresh runtime pass confirms the existing
  cached-subtree counter mutation and controlled Popover open/close state still converge through
  dedicated `/view_cache` app snapshot fields.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\view-cache\ui-gallery-view-cache-model-mutation-through-cache.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_view_cache_model_mutation_through_cache --no-fail-fast --no-capture`
  with Nextest run id `e96cc371-57d7-46ca-859b-9120a0907d6d`; and
  `git diff --check`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\view-cache\ui-gallery-view-cache-model-mutation-through-cache.json --dir target\fret-diag-view-cache-model-mutation-roundtrip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with run id `1779162384646` and AI packet
  `target/fret-diag-view-cache-model-mutation-roundtrip-v1/sessions/1779162372113-24280/1779162384646/ai.packet`.
- Suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-view-cache --dir target\fret-diag-view-cache-suite-roundtrip-v1 --session-auto --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-view-cache-suite-roundtrip-v1/sessions/1779162428017-56424/suite.summary.json`;
  1/1 scripts passed and the script run id is `1779162437682`.

## M127: Resizable Moving Cached Combobox Protocol Gate

Status: complete for protocol coverage and fresh runtime evidence on cached movement/root-boundary
placement.

- Added direct `fret-diag-protocol` roundtrip coverage for
  `ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json`.
- The script already self-configures the Resizable page, the diagnostics-only moving cached
  Combobox section, and `FRET_UI_GALLERY_VIEW_CACHE=1`. It moves the cached Combobox source from
  the left Resizable panel to the right panel before opening the overlay.
- No new mechanism defect was reproduced. The fresh runtime pass confirms the existing cached-root
  interaction-cache replay fix still preserves hit-test routing, top-side overlay placement,
  right-panel/window boundary containment, and Combobox input/listbox relation edges after movement.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\resizable\ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_resizable_view_cache_moving_combobox_root_boundary --no-fail-fast --no-capture`
  with Nextest run id `c0b75f4d-b758-48c7-9aac-db09a7f02595`; and `git diff --check`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\resizable\ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json --dir target\fret-diag-resizable-view-cache-moving-combobox-roundtrip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with run id `1779163064132` and AI packet
  `target/fret-diag-resizable-view-cache-moving-combobox-roundtrip-v1/sessions/1779163052541-37388/1779163064132/ai.packet`.
- Suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-resizable --dir target\fret-diag-resizable-suite-view-cache-roundtrip-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-resizable-suite-view-cache-roundtrip-v1/sessions/1779163144561-38700/suite.summary.json`;
  2/2 scripts passed, `scripts_with_evidence=2`, `overlay_chosen_side_counts.top=2`, and the
  moving cached Combobox script run id is `1779163184863`.

## M128: Command Retained Active-Descendant Action-State Protocol Gate

Status: complete for protocol coverage and fresh runtime evidence on retained relation/action-state
mutation.

- Added direct `fret-diag-protocol` roundtrip coverage for
  `ui-gallery-command-retained-active-descendant-action-state.json`.
- The script already gates the retained/windowed Command invariant: the active descendant clears
  when the active row detaches, and after reattach the same row must expose refreshed
  `disabled=true` and `invoke=false` semantics.
- No retained relation/action-state mechanism defect was reproduced. The focused runtime pass and
  full Command suite confirm the existing synthetic retained active-descendant fixture remains
  locked by a real UI Gallery runtime surface.
- The first full-suite rerun exposed diagnostics authoring drift in
  `ui-gallery-command-docs-demo-long-query-text.json`: the docs demo was already visible, so a
  pre-positioning `scroll_into_view` could emit a no-op wheel and stall with `timeout.no_frames`.
  The script now uses `ensure_visible(within_window=true)` for that precondition while preserving
  the input-level long-text oracle.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\command\ui-gallery-command-docs-demo-long-query-text.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery\command\ui-gallery-command-retained-active-descendant-action-state.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_command_docs_demo_long_query_text script_v2_roundtrip_ui_gallery_command_retained_active_descendant_action_state --no-fail-fast --no-capture`
  with Nextest run id `07836627-15f2-45ec-9209-2915b9d38a3e`; and `git diff --check`.
- Focused retained action-state runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\command\ui-gallery-command-retained-active-descendant-action-state.json --dir target\fret-diag-command-retained-active-descendant-action-state-roundtrip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with run id `1779164006100` and AI packet
  `target/fret-diag-command-retained-active-descendant-action-state-roundtrip-v1/sessions/1779163988388-20728/1779164006100/ai.packet`.
- Focused long-query runtime diagnostics pass after the authoring fix:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\command\ui-gallery-command-docs-demo-long-query-text.json --dir target\fret-diag-command-long-query-ensure-visible-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with run id `1779164428287`.
- Full Command suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-command --dir target\fret-diag-command-suite-retained-action-state-roundtrip-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-command-suite-retained-action-state-roundtrip-v2/sessions/1779164457116-49144/suite.summary.json`;
  18/18 scripts passed, `scripts_with_evidence=18`, the long-query script run id is
  `1779164551371`, and the retained action-state script run id is `1779165106416`.

## M129: AI FileTree Protocol Coverage And Auto-Height VirtualList Refresh

Status: complete for protocol coverage, measured-leaf dirtying refresh, screenshot-script
stabilization, and fresh AI FileTree suite evidence.

- Added direct `fret-diag-protocol` roundtrip coverage for the four promoted AI FileTree scripts:
  toggle, actions, large-scroll, and zinc-dark screenshot.
- The first fresh suite rerun reproduced the same high-risk mechanism shape as M80: expanded
  FileTree rows could be present in semantics while the parent auto-height `VirtualList` measured
  leaf kept a stale intrinsic height, so the next docs section overlapped the row's hit-test area.
- `crates/fret-ui/src/layout/engine/flow.rs` now centralizes measured-leaf setup so any measured
  Taffy leaf whose `UiTree` node is layout-invalidated is also marked dirty in the layout engine.
  This covers VirtualList and the adjacent auto-sized measured leaf paths without introducing a
  FileTree-specific workaround.
- The screenshot script now waits for the expanded `file-lib` row instead of using a fixed
  two-frame delay, then asserts the hidden selected marker through `raw_semantics_hidden_is` and
  the actual selected row through `selected_is`.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\ai\ui-gallery-ai-file-tree-demo-screenshot-zinc-dark.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-ui\src\layout\engine\flow.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui auto_height_virtual_list_len_growth_reflows_following_siblings --no-fail-fast --no-capture`
  with Nextest run id `de3f626b-824f-4d21-82af-251d51680c64`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui virtual_list --no-fail-fast --no-capture`
  with Nextest run id `a2e88f71-2c4c-431d-9b0c-8cefdced2a4b`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_ai_file_tree --no-fail-fast --no-capture`
  with Nextest run id `ea3bdd56-e255-4d34-97f4-b97599cb7369`; and
  `git diff --check`.
- Focused screenshot diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery-ai-file-tree-demo-screenshot-zinc-dark.json --dir target\fret-diag-ai-file-tree-screenshot-zinc-dark-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with run id `1779168079984` and AI packet
  `target/fret-diag-ai-file-tree-screenshot-zinc-dark-v2/sessions/1779168068402-29976/1779168079984/ai.packet`.
- Full AI FileTree suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-ai-file-tree --dir target\fret-diag-ai-file-tree-suite-v3 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-ai-file-tree-suite-v3/sessions/1779168118270-70184/suite.summary.json`;
  4/4 scripts passed, `scripts_with_evidence=4`, and the screenshot script run id is
  `1779168265307`.

## M130: Workspace Shell Demo Window-Close Dirty Aggregation Gate

Status: complete for window-level dirty-close aggregation across workspace panes.

- Added `WorkspaceCloseReason::CloseWindow`,
  `WorkspaceWindowLayout::dirty_close_request_for_window_close`, and
  `WorkspaceWindowLayout::can_close_window_with_policy` so the workspace policy layer can build a
  single dirty-close request over all pane tabs before a window is closed.
- Routed the workspace shell demo's real `window.close` command and `Event::WindowCloseRequested`
  through that policy path. When policy blocks, the app-owned dirty-close prompt now reports the
  `CloseWindow` reason and preserves the window until the user chooses Discard or Save.
- Added debug controls that mark pane-a and pane-b active tabs dirty, then dispatch the real
  `window.close` command. The shared command-button helper now records pending pointer source so
  driver-handled commands keep accurate diagnostics attribution.
- Added
  `workspace-shell-demo-window-close-dirty-aggregation-smoke.json` and promoted it into the
  `workspace-shell-demo` suite. The script marks `doc-a-2` and `doc-b-1` dirty, clicks
  `workspace-shell-debug-close-window`, asserts pointer-sourced `window.close`, verifies the prompt
  label contains `reason=CloseWindow active=doc-a-2 close_count=5` and
  `dirty=[doc-a-2, doc-b-1]`, cancels, and proves both panes plus dirty markers remain.
- The first focused drafts exposed diagnostics authoring gaps rather than the final ownership
  invariant: the `window.close` driver branch did not record a command-dispatch trace, the shared
  debug command button did not record pending pointer source, and the redirect file needed
  `kind=script_redirect`/`to` to run under the suite launcher.
- Gates pass:
  `rustfmt --edition 2024 --check ecosystem\fret-workspace\src\close_policy.rs ecosystem\fret-workspace\src\layout.rs apps\fret-examples\src\workspace_shell_demo.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-window-close-dirty-aggregation-smoke.json > $null`;
  `python -m json.tool tools\diag-scripts\workspace-shell-demo-window-close-dirty-aggregation-smoke.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-workspace window_close_dirty_policy_aggregates_tabs_across_panes --no-fail-fast --no-capture`
  with Nextest run id `8cf5ad37-5f56-4572-bab9-b3d96f5a29ae`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_window_close_dirty_aggregation_smoke --no-fail-fast --no-capture`
  with Nextest run id `a076a73a-6fe1-44c5-9757-1fd257a67a0c`; and
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-window-close-dirty-aggregation-smoke.json --dir target\fret-diag-workspace-shell-demo-window-close-dirty-aggregation-v4 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with run id `1779171091877` and AI packet
  `target/fret-diag-workspace-shell-demo-window-close-dirty-aggregation-v4/sessions/1779171088566-57484/1779171091877/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-window-close-dirty-aggregation-v2 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  with suite summary
  `target/fret-diag-workspace-shell-demo-suite-window-close-dirty-aggregation-v2/sessions/1779171327648-11792/suite.summary.json`;
  the new window-close dirty aggregation script run id is `1779171369594`.

## M131: Chart Torture Visible Domain-Window Oracle

Status: complete for Chart Torture visible X domain-window correctness after pan/zoom.

- Extended `app_snapshot.chart_torture` to schema version 2 with:
  `x_full_domain_window`,
  `output_model.x_domain_window`, and runtime oracle booleans for whether the engine X axis output
  window and paint-published output-model X domain window match the active dataZoom window and
  differ from the fixture's initial full X domain.
- Hardened `ui-gallery-chart-torture-pan-zoom.json` so it first asserts the known full-domain X
  baseline (`1735689600000..1747689540000`, span `11999940000`) before interaction, then waits for
  all visible-window oracle booleans after drag/wheel.
- No new chart mechanism defect was reproduced. The fresh suite shows the expected convergence:
  after interaction, the output model and engine axis output both publish
  `1739283224994..1757471732398`, matching dataZoom and differing from the initial full domain.
- Gates pass:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\driver\diag_snapshot.rs`;
  `python -m json.tool tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  with Nextest run id `7bbb707d-390a-4659-b782-1d38ef175e24`;
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-chart,gallery-dev`;
  and `git diff --check`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json --dir target\fret-diag-chart-torture-visible-window-oracle-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  with run id `1779173616393` and AI packet
  `target/fret-diag-chart-torture-visible-window-oracle-v1/sessions/1779173543971-68812/1779173616393/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target\fret-diag-chart-torture-suite-visible-window-oracle-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-chart-torture-suite-visible-window-oracle-v1/sessions/1779173643592-70968/suite.summary.json`;
  run id `1779173655069`.

## M132: Carousel State Suite And Focus Autoplay Stop

Status: complete for compact Carousel state diagnostics and focus-triggered autoplay
stopOnInteraction correctness.

- Added `ui-gallery-carousel-state` as a compact zero-warning diagnostics suite for Carousel state
  interactions: Events select, Events reInit, autoplay stopOnLastSnap, autoplay stopOnInteraction
  via focus, and RTL controls.
- The first full suite run found a real shadcn Carousel runtime defect. Pressing Tab moved focus
  into a nested slide button and watchFocus scrolled to that slide, but the autoplay API still
  reported `playing=true` and `stopped_by_interaction=false`.
- Fixed the owning recipe layer in `ecosystem/fret-ui-shadcn/src/carousel.rs`: focus entry into a
  slide now marks autoplay as stopped by interaction whenever stopOnInteraction is enabled. The
  code still cancels an active timer token when one exists, but no longer treats token presence as
  the condition for recognizing the focus interaction.
- Added direct `fret-diag-protocol` roundtrip coverage for the five scripts that make up the new
  Carousel state suite.
- Gates pass:
  `rustfmt --edition 2024 --check ecosystem\fret-ui-shadcn\src\carousel.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-carousel-state\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --test carousel_autoplay_api_handle carousel_autoplay_stop_on_interaction_stops_after_slide_receives_focus --no-fail-fast --no-capture`
  with Nextest run id `7fc50006-1357-4756-86f2-9452c5605aab`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_carousel_state_gates --no-fail-fast --no-capture`
  with Nextest run id `96bee069-29be-4c38-8423-f89b44f5d3fa`;
  and `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\carousel\ui-gallery-carousel-plugin-autoplay-stop-on-interaction-focus-gate.json --dir target\fret-diag-carousel-stop-on-focus-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779176381271` and AI packet
  `target/fret-diag-carousel-stop-on-focus-v2/sessions/1779176372434-68188/1779176381271/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-carousel-state --dir target\fret-diag-carousel-state-suite-v2 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-carousel-state-suite-v2/sessions/1779176492375-78748/suite.summary.json`;
  5/5 rows passed, `scripts_with_evidence=5`, `focus_mismatch_total=0`, and the focus
  stopOnInteraction script run id is `1779176831378`.

## M133: Combobox Checkmark Effective Opacity Predicate

Status: complete for structured selected/unselected checkmark opacity evidence in the Combobox
long-text geometry gates.

- Added the diagnostics predicate `element_effective_opacity_approx_eq`. It reads
  `ElementRuntime` effective opacity after declarative `Opacity` wrappers are inherited, so a
  script can prove paint-only selected/hidden affordances even when their semantics/layout nodes
  remain present.
- `WindowElementState` now records current and previous effective opacity per element during
  declarative mount. `mount_element` propagates the parent opacity through children and records the
  clamped product for every element.
- The UI diagnostics predicate evaluator maps the target semantics node back to the owning element
  and compares its effective opacity with an epsilon. The script engine now also recognizes
  predicate-bearing `assert`, `wait_until`, and `drag_pointer_until` steps that need
  `ElementRuntime`; the first LTR runtime draft failed because the new predicate did not borrow
  runtime state and therefore evaluated false before reading the target.
- Hardened the LTR and RTL Combobox long-text geometry scripts so the selected long checkmark
  asserts opacity `1.0` and the unselected short checkmark asserts opacity `0.0`.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\elements\runtime.rs crates\fret-ui\src\declarative\mount.rs crates\fret-diag-protocol\src\lib.rs ecosystem\fret-bootstrap\src\ui_diagnostics\predicates.rs ecosystem\fret-bootstrap\src\ui_diagnostics\script_steps_wait.rs ecosystem\fret-bootstrap\src\ui_diagnostics\script_engine.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-long-text-geometry.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-geometry.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_element_effective_opacity_approx_eq_serializes_and_deserializes script_v2_roundtrip_ui_gallery_combobox_long_text_geometry script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_geometry --no-fail-fast --no-capture`
  with Nextest run id `021decf3-5aae-41ac-95f6-ec738542acca`;
  `cargo test --profile dev-fast -p fret-bootstrap runtime_gate_keeps_effective_opacity_predicates --features ui-app-driver,diagnostics -- --nocapture`;
  `cargo check --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics`;
  and `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-long-text-geometry.json --dir target\fret-diag-combobox-long-text-opacity-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779180476346` and AI packet
  `target/fret-diag-combobox-long-text-opacity-v2/sessions/1779180467898-80632/1779180476346/ai.packet`.
- RTL focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-geometry.json --dir target\fret-diag-combobox-rtl-long-text-opacity-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779180503756` and AI packet
  `target/fret-diag-combobox-rtl-long-text-opacity-v1/sessions/1779180495343-65848/1779180503756/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-opacity-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-combobox-geometry-placement-opacity-v1/sessions/1779180495343-74828/suite.summary.json`;
  7/7 rows passed, `scripts_with_evidence=7`, and `overlay_chosen_side_counts` reports
  `bottom=6`, `top=5`.

## M134: Text Paint Reprepare Layout Repair And Combobox Intro Gate

Status: complete for paint-time text height convergence and the Combobox Popup docs intro
non-overlap runtime gate.

- Repaired the mechanism layer in `crates/fret-ui`: after `Text`, `StyledText`, or
  `SelectableText` performs a paint-time reprepare because width or font stack changed, an
  auto-height node now invalidates layout and requests redraw when the newly prepared text height
  exceeds the current paint bounds.
- Added focused coverage in `text_cache.rs` for a wrapped text node laid out at a wider width and
  then painted at a narrower width whose prepared metrics are taller. The test proves the node is
  marked layout-invalid and a redraw effect is emitted.
- Added a stable docs-intro test id in UI Gallery and promoted
  `ui-gallery-combobox-popup-doc-intro-non-overlap.json` into the Combobox geometry placement
  suite. The gate starts directly on the Combobox Popup section at `671x460`, captures layout,
  screenshot, and bundle evidence, and asserts the intro bottom stays at least `8px` above the
  Popup title while the description stays below the title.
- Extended the Combobox Popup trigger and bottom-room gates with selected/unselected checkmark
  effective-opacity assertions, preserving the previous placement/geometry coverage.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\paint.rs crates\fret-ui\src\declarative\tests\text_cache.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_paint_width_shrink_reinvalidates_layout_when_height_grows --no-fail-fast --no-capture`
  with Nextest run id `50e6ec15-0b4f-4340-b689-c10ae58055e2`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_non_overlap script_v2_roundtrip_ui_gallery_combobox_popup_trigger script_v2_roundtrip_ui_gallery_combobox_popup_trigger_bottom_room --no-fail-fast --no-capture`
  with Nextest run id `05c76ef5-e683-4a03-a809-d71fc53256ca`;
  `cargo check --profile dev-fast -p fret-ui`;
  and `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-popup-doc-intro-non-overlap.json --dir target\fret-diag-combobox-popup-doc-intro-overlap-671x460-repair-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779186094473` and AI packet
  `target/fret-diag-combobox-popup-doc-intro-overlap-671x460-repair-v1/sessions/1779186086330-88228/1779186094473/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-text-layout-repair-v2 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-combobox-geometry-placement-text-layout-repair-v2/sessions/1779186620899-17592/suite.summary.json`;
  the new intro non-overlap script run id is `1779186747293`.

## M135: Resizable Multi-Viewport Select Root-Boundary Gate

Status: complete for Select popper placement coverage inside a Resizable panel viewport root.

- Added `multi_viewport_select.rs` to the Resizable UI Gallery page as an opt-in diagnostics
  fixture. The Select control sits near the bottom of the right Resizable panel so the OS window has
  room below, but the panel viewport root does not.
- Promoted `ui-gallery-resizable-multi-viewport-select-placement.json` into the
  `ui-gallery-resizable` suite with direct `fret-diag-protocol` roundtrip coverage and a top-level
  redirect script.
- The runtime gate waits for a `placed_rect` overlay trace with
  `anchor_test_id=ui-gallery-resizable-multi-viewport-select-root`,
  `content_test_id=ui-gallery-resizable-multi-viewport-select-listbox`, `preferred_side=bottom`,
  `chosen_side=top`, `flipped=true`, and `side_offset=6`. The trace records the Resizable panel
  bounds as the placement `outer`, proving panel-root ownership for Select as a second overlay
  family after Combobox.
- No Select placement/root-boundary defect was reproduced. The first runtime drafts exposed script
  authoring hazards instead: the outer fixture id was not the Select control, exact top-side gap
  checks overfit the placement solver, and underlay panel selectors are hidden behind the modal
  overlay barrier once the Select listbox opens.
- Gates pass:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\resizable\multi_viewport_select.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_resizable_multi_viewport_select_placement --no-fail-fast --no-capture`
  with latest Nextest run id `7944bf63-93b6-476d-aa9a-7b6b53771d9e`;
  and `git diff --check`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\resizable\ui-gallery-resizable-multi-viewport-select-placement.json --dir target\fret-diag-resizable-multi-viewport-select-placement-v8 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779193025213` and AI packet
  `target/fret-diag-resizable-multi-viewport-select-placement-v8/sessions/1779193017299-99444/1779193025213/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-resizable --dir target\fret-diag-ui-gallery-resizable-suite-select-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-ui-gallery-resizable-suite-select-v1/sessions/1779193114796-63336/suite.summary.json`;
  3/3 scripts passed, `scripts_with_evidence=3`, and the new Select script run id is
  `1779193162410`.

## M136: Text Reprepare Repair-Frame Clip And Full Combobox Startup Gate

Status: complete for same-frame clipping during paint-time text layout repair and the full
Combobox page startup non-overlap gate.

- Extended the mechanism-layer text repair in `crates/fret-ui`: when paint-time text preparation
  discovers that an auto-height `Text`, `StyledText`, or `SelectableText` blob is taller than the
  current stale layout bounds, the node still invalidates layout and requests redraw, but the same
  frame now draws the text under a rectangular clip equal to the stale layout bounds.
- Extended the focused `wrapped_text_paint_width_shrink_reinvalidates_layout_when_height_grows`
  regression so it asserts the repair frame contains `PushClipRRect -> Text -> PopClip`, preventing
  the next regression from fixing only the follow-up layout pass while allowing one-frame visual
  spill.
- Added `ui-gallery-combobox-full-page-startup-intro-non-overlap.json`, a full-page companion to
  the existing Popup-focused intro gate. It starts on the full Combobox page at `671x460`, captures
  layout/screenshot/bundle evidence before any manual resize recovery, and asserts the long docs
  intro leaves at least `16px` before the Basic section title.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\paint.rs crates\fret-ui\src\declarative\tests\text_cache.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_paint_width_shrink_reinvalidates_layout_when_height_grows --no-fail-fast --no-capture`
  with latest Nextest run id `d8184adc-9875-470f-9828-025bc220465e`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_full_page_startup_intro_non_overlap --no-fail-fast --no-capture`
  with Nextest run id `4fa52001-9eb6-4102-9bba-033f10b3e2c0`;
  `cargo check --profile dev-fast -p fret-ui`;
  and `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-full-page-startup-intro-non-overlap.json --dir target\fret-diag-combobox-full-page-startup-intro-text-clip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779194385483` and AI packet
  `target/fret-diag-combobox-full-page-startup-intro-text-clip-v1/sessions/1779194373536-98264/1779194385483/ai.packet`.
- Focused Popup companion still passes:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-popup-doc-intro-non-overlap.json --dir target\fret-diag-combobox-popup-doc-intro-text-clip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779194199027`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-text-clip-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-combobox-geometry-placement-text-clip-v1/sessions/1779194425260-69272/suite.summary.json`;
  the new full-page startup script run id is `1779194524638`.

## M137: CommandDialog Basic Overlay Focus Gate

Status: complete for Command Basic modal overlay/focus runtime coverage.

- Added `ui-gallery-command-basic-dialog-overlay-focus.json` and a top-level redirect, then promoted
  it into the `ui-gallery-command` suite with direct `fret-diag-protocol` roundtrip coverage.
- The runtime gate opens the real Command Basic `CommandDialog`, proves dialog and close-button
  semantics, input focus, listbox containment, listbox `labelled_by` wiring, input
  `active_descendant` wiring, ArrowDown movement from Calendar to Search Emoji, Escape dismissal,
  and focus restoration to the `Open Menu` button.
- The first focused runtime draft failed only because the script asserted focus on
  `ui-gallery-command-basic-trigger.chrome`, the visual chrome child. The failure bundle showed the
  focused semantics node was the outer `role=button` with label `Open Menu`, so the final gate
  asserts the semantic focus owner instead.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\command\ui-gallery-command-basic-dialog-overlay-focus.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-command-basic-dialog-overlay-focus.json > $null`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  and `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_command_basic_dialog_overlay_focus --no-fail-fast --no-capture`
  with Nextest run id `08a923e7-9ca3-4b3c-bb2f-fe62628193ec`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\command\ui-gallery-command-basic-dialog-overlay-focus.json --dir target\fret-diag-command-basic-dialog-overlay-focus-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779196803631` and AI packet
  `target/fret-diag-command-basic-dialog-overlay-focus-v2/sessions/1779196795872-108048/1779196803631/ai.packet`.
- Full runtime suite diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-command --dir target\fret-diag-ui-gallery-command-suite-dialog-overlay-focus-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-ui-gallery-command-suite-dialog-overlay-focus-v1/sessions/1779196833923-91304/suite.summary.json`;
  the new CommandDialog script run id is `1779196993347`.

## M138: Combobox Short Startup Intro Non-Overlap Gate

Status: complete for screenshot-derived short-height startup coverage on the Combobox Popup docs
intro path.

- Added `ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json` plus a top-level
  redirect, then promoted it into the `ui-gallery-combobox-geometry-placement` suite with direct
  `fret-diag-protocol` roundtrip coverage.
- The gate starts directly on the Combobox Popup section at logical `663x311`, matching the
  observed `994x466` screenshot on a 1.5x scale display. It captures layout, screenshot, and bundle
  evidence at early startup before any manual resize recovery.
- No current runtime overlap defect was reproduced after the text repair-frame clipping work. The
  focused run captured the relevant layout at frame 3: `ui-gallery-doc-page-intro.bottom =
  379.3333`, `docsec-popup-title.top = 403.3333`, so the measured gap is `24px` and satisfies the
  `>= 16px` gate.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-combobox-geometry-placement\suite.json > $null`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_short_startup_non_overlap --no-fail-fast --no-capture`
  with Nextest run id `e92bb8b8-cf66-47b5-8281-1fa91f73c6b3`.
- Build pass:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json --dir target\fret-diag-combobox-popup-doc-intro-short-startup-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779198569025`, AI packet
  `target/fret-diag-combobox-popup-doc-intro-short-startup-v1/sessions/1779198558655-90216/1779198569025/ai.packet`,
  and pack
  `target/fret-diag-combobox-popup-doc-intro-short-startup-v1/sessions/1779198558655-90216/share/1779198569025.zip`.
- Full Combobox geometry placement suite pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-short-startup-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-combobox-geometry-placement-short-startup-v1/sessions/1779198616098-23160/suite.summary.json`;
  10/10 scripts passed, `scripts_with_evidence=9`, and the new short-startup script run id is
  `1779198792251`.

## M139: AI Transcript Non-Retained Scroll Count Gate

Status: complete for AI transcript torture scroll mutation coverage and suite tail policy.

- Strengthened `ui-gallery-ai-transcript-torture-scroll.json` so it injects a deterministic
  `240`-message variable-height transcript, asserts the startup message-count semantics, appends
  `100` messages, and proves the count advances to `340` with layout, screenshot, and bundle
  evidence.
- Added a hidden semantics counter to the AI transcript torture snippet so diagnostics can assert
  the real model size without depending on rendered text snippets or scroll-window selection.
- Added direct `fret-diag-protocol` roundtrip coverage for all three
  `ui-gallery-ai-transcript-retained` suite scripts.
- Removed `ui-gallery-ai-transcript-torture-scroll.json` from the retained vlist reconcile tail
  policy. `fret-ui-ai` intentionally uses non-retained virtual lists for transcript surfaces so the
  component surface does not require `UiHost + 'static`; the old suite tail check treated that
  intentional non-retained path as a retained-window failure.
- Gates pass:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\ai\transcript_torture.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs crates\fret-diag\src\diag_policy.rs crates\fret-diag\src\tests.rs`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag ai_transcript_torture_scroll_is_not_a_retained_vlist_reconcile_gate --no-fail-fast --no-capture`
  with Nextest run id `caf72dfa-d836-47df-8dd3-0aa22a1618e5`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_ai_conversation_demo_screenshot_zinc_dark script_v2_roundtrip_ui_gallery_ai_conversation_demo_scroll_button script_v2_roundtrip_ui_gallery_ai_transcript_torture_scroll --no-fail-fast --no-capture`
  with Nextest run id `674f2827-c68d-4532-917f-583e0e81cc1b`;
  and
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\perf\ui-gallery-ai-transcript-torture-scroll.json --dir target\fret-diag-ai-transcript-torture-count-gate-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with run id `1779201476652` and AI packet
  `target/fret-diag-ai-transcript-torture-count-gate-v1/sessions/1779201370619-115172/1779201476652/ai.packet`.
- Full AI transcript suite diagnostics pass after rebuilding `fretboard-dev`:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-ai-transcript-retained --dir target\fret-diag-ai-transcript-retained-cargo-policy-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-ai-transcript-retained-cargo-policy-v2/sessions/1779203319147-101240/suite.summary.json`;
  3/3 scripts passed, the torture script run id is `1779203465969`, and no retained-tail
  non-retained-shift check file was produced.

## M140: Combobox RTL Long Text Startup Non-Overlap Gate

Status: complete for the screenshot-corrected Combobox RTL Long Text cold-start overlap coverage.

- The latest user screenshot showed the focused Combobox `RTL Long Text` section title visually
  colliding with the long docs intro, not the earlier Popup title. The previous Popup/full-page
  startup gates therefore covered the right class but the wrong section target.
- Added
  `ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json`, a focused
  startup gate with `FRET_UI_GALLERY_START_SECTION=RTL Long Text` and
  `FRET_UI_GALLERY_MAIN_WINDOW_SIZE=1083x752`, matching the observed `1624x1128` physical
  screenshot at a 1.5x scale factor.
- The gate captures layout, screenshot, and bundle evidence before manual resize recovery, then
  asserts that `ui-gallery-doc-page-intro` and
  `ui-gallery-combobox-rtl-long-text-docsec-title` do not overlap, the title starts at least
  `16px` after the intro bottom, and the section description starts at least `8px` after the title.
- Added the `994x466` Popup startup companion as a secondary scale-interpretation probe and
  promoted both scripts into `ui-gallery-combobox-geometry-placement` with direct
  `fret-diag-protocol` roundtrip coverage.
- Current `dev-fast` runtime did not reproduce the overlap after the prior text repair-frame clip
  work. The focused RTL Long Text run captured a clean frame 3 screenshot and passed with run id
  `1779207094769`; the full Combobox geometry placement suite passed with the new RTL Long Text run
  id `1779208395010`.
- Gates pass:
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-combobox-geometry-placement\suite.json > $null`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`; and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_logical994_startup_non_overlap script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_doc_intro_logical1083_startup_non_overlap --no-fail-fast --no-capture`
  with Nextest run id `6619d838-cd48-41d2-b279-ede4466fc291`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-doc-intro-logical1083-gate-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779207094769`, AI packet
  `target/fret-diag-combobox-rtl-long-text-doc-intro-logical1083-gate-v1/sessions/1779207086203-106128/1779207094769/ai.packet`,
  and pack
  `target/fret-diag-combobox-rtl-long-text-doc-intro-logical1083-gate-v1/sessions/1779207086203-106128/share/1779207094769.zip`.
- Full Combobox geometry placement suite pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-rtl-long-text-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-combobox-geometry-placement-rtl-long-text-v1/sessions/1779208245269-120048/suite.summary.json`;
  12/12 scripts passed, the new RTL Long Text startup run id is `1779208395010`, and the new Popup
  logical994 run id is `1779208377600`.

## M141: First-Paint Text Auto-Height Repair

Status: complete for the cold-start text repair path behind the Combobox RTL Long Text overlap.

- A follow-up user screenshot showed the RTL Long Text overlap was still visible manually even
  though M140's focused runtime gate passed. Re-running the promoted gate exposed a gate defect:
  it pressed `Escape` before capture, which advanced an input frame and could mask the cold-start
  repair path the script meant to test.
- Removed the `Escape` step from
  `ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json`, so the gate
  captures startup layout/screenshot evidence without keyboard or resize recovery.
- Fixed `Text`, `StyledText`, and `SelectableText` paint-time repair ownership in
  `crates/fret-ui`: when any paint-time prepare produces taller metrics than an auto-height text
  node's current bounds, the node now invalidates layout, requests redraw, and clips the repair
  frame. The previous condition only covered width/font-stack reprepare and missed first-paint
  prepare or content/style-driven taller metrics.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\paint.rs crates\fret-ui\src\declarative\tests\text_cache.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_first_paint_reinvalidates_layout_when_height_grows wrapped_text_paint_width_shrink_reinvalidates_layout_when_height_grows --no-fail-fast --no-capture`
  with Nextest run id `ee45c3ee-bd9e-4983-bf51-3a676fe8efdc`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_doc_intro_logical1083_startup_non_overlap --no-fail-fast --no-capture`
  with Nextest run id `af2bdc87-44c4-4be2-8e92-d5a6a062da39`;
  `python tools\check_diag_scripts_registry.py`; and `git diff --check`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-no-input-fixed-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with run id `1779210456769`, AI packet
  `target/fret-diag-combobox-rtl-long-text-no-input-fixed-v1/sessions/1779210358866-112204/1779210456769/ai.packet`,
  and screenshot
  `target/fret-diag-combobox-rtl-long-text-no-input-fixed-v1/sessions/1779210358866-112204/screenshots/1779210460025-ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap/window-4294967297-tick-2-frame-2.png`.
- Full Combobox geometry placement suite pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-startup-text-repair-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-combobox-geometry-placement-startup-text-repair-v1/sessions/1779210565472-66488/suite.summary.json`;
  12/12 scripts passed, and the no-input RTL Long Text startup run id is `1779210808250`.

## M142: Wrapped Text Prepared Measurement Convergence

Status: complete for the startup measurement path behind the remaining Combobox RTL Long Text
visual overlap.

- A newer manual screenshot still showed `RTL Long Text` content visibly overlapping even after
  M141. The promoted runtime gate passed because it asserted layout bounds, while the visible
  failure could be caused by text ink painted taller than the measured layout box.
- Fixed `Text`, `StyledText`, and `SelectableText` measurement in `crates/fret-ui` so wrapped text
  paths prepare text and populate the shared text cache during measurement. This makes startup
  layout reserve the same prepared metrics that paint will use instead of trusting a separate
  backend `measure` result that can underestimate height before resize/font convergence.
- Preserved the `TextWrap::None` fast measurement/fingerprint path and kept the M141 paint-time
  repair as a fallback for stale paint bounds and width changes.
- Strengthened
  `ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json` to wait for
  the live RTL Long Text content/trigger and assert description-to-content plus
  description-to-trigger spacing, not only intro/title/description bounds.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\measure.rs crates\fret-ui\src\declarative\tests\text_cache.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_measure_uses_prepare_metrics_for_startup_layout wrapped_text_first_paint_reinvalidates_layout_when_height_grows theme_color_change_does_not_change_text_input_fingerprints --no-fail-fast --no-capture`
  with Nextest run id `c70a4417-6ee8-46f1-bc4f-a485bc98a122`;
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json > $null`;
  `python tools\check_diag_scripts_registry.py`; and
  `cargo build --profile dev-fast -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-startup-prepared-measure-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779215099640`, AI packet
  `target/fret-diag-combobox-rtl-long-text-startup-prepared-measure-v3/sessions/1779215091187-112692/1779215099640/ai.packet`,
  pack
  `target/fret-diag-combobox-rtl-long-text-startup-prepared-measure-v3/sessions/1779215091187-112692/share/1779215099640.zip`,
  and screenshot
  `target/fret-diag-combobox-rtl-long-text-startup-prepared-measure-v3/sessions/1779215091187-112692/screenshots/1779215103570-ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap/window-4294967297-tick-3-frame-3.png`.

## M143: Chart Torture Multi-Series Tooltip Output Gate

Status: complete for multi-series retained chart tooltip and X domain-window output coverage.

- Changed Chart Torture from one line series to two line series (`A` and `B`) sharing the same X/Y
  axes. The fixture now exercises multi-series tooltip aggregation against the large retained chart
  path rather than only a single-row tooltip.
- Extended the UI Gallery app snapshot for Chart Torture with tooltip summary fields:
  `axis_header_count`, `series_rows_count`, `source_series_rows_count`, `missing_rows_count`,
  `series_labels`, `has_series_a`, and `has_series_b`.
- Strengthened `ui-gallery-chart-torture-pan-zoom.json` so the real drag/wheel pan/zoom path
  proves the output model tooltip has one axis header, two source-owned series rows, labels `A`
  and `B`, and zero missing rows.
- The first runtime run exposed a stale diagnostics assertion: `domain_windows_count == 2` assumed
  both X and Y axes would auto-export link keys. Under ADR 0301, the shared Y axis is ambiguous
  once two Y fields participate, so only the unique X `(dataset, field)` key is exported. The gate
  now expects `domain_windows_count == 1` and keeps the explicit X output-window/dataZoom matching
  assertions.
- Gates pass:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\driver\diag_snapshot.rs apps\fret-ui-gallery\src\ui\previews\pages\torture\chart_torture.rs`;
  `python -m json.tool tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json > $null`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  with Nextest run id `993eeccd-72d1-49f4-830f-a710b0b16250`; and
  `python tools\check_diag_scripts_registry.py`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json --dir target\fret-diag-chart-torture-multiseries-tooltip-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  with run id `1779217026347`, AI packet
  `target/fret-diag-chart-torture-multiseries-tooltip-v3/sessions/1779217007250-123724/1779217026347/ai.packet`,
  and pack
  `target/fret-diag-chart-torture-multiseries-tooltip-v3/sessions/1779217007250-123724/share/1779217026347.zip`.
- Full Chart Torture suite pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target\fret-diag-chart-torture-suite-multiseries-tooltip-v1 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-chart-torture-suite-multiseries-tooltip-v1/sessions/1779217110888-98424/suite.summary.json`
  and script run id `1779217122878`.

## M144: Cached Prepared Text Stale-Bounds Repair

Status: complete for the persistent Combobox RTL Long Text startup overlap follow-up.

- A fresh manual screenshot still showed `RTL Long Text` visually colliding with the docs intro
  even though M142's runtime gate passed. The gate's screenshot captures at frame 3, so it can miss
  a first-visible-frame paint overflow that later repair frames correct.
- Added a focused mechanism regression for the remaining gap: layout prepares and caches a wrapped
  text blob with a `40px` height, then paint receives a stale `10px` auto-height bounds. Before
  the fix, the cached prepared blob path did not invalidate layout or clip; the new test failed on
  that exact condition.
- Fixed `Text`, `StyledText`, and `SelectableText` paint in `crates/fret-ui` so the auto-height
  repair helper also runs after cached/prepared metrics are loaded, not only inside the
  `needs_prepare` branch. If cached metrics exceed current auto-height bounds, paint now schedules
  layout repair, requests redraw, and clips the stale frame.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\paint.rs crates\fret-ui\src\declarative\tests\text_cache.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui text_cache --no-fail-fast --no-capture`
  with Nextest run id `c4dc5647-ab06-4015-be4a-829f175a3359`; and
  `cargo build --profile dev-fast -p fret-ui-gallery`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-ui-gallery-combobox-rtl-intro-overlap-fixed-1624-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779219333574` and AI packet
  `target/fret-diag-ui-gallery-combobox-rtl-intro-overlap-fixed-1624-v1/sessions/1779219328227-121516/1779219333574/ai.packet`.

## M145: Combobox RTL Long Text Doc Scaffold Min-Width Clamp

Status: complete for the user-visible first-frame doc scaffold overlap follow-up.

- The manual screenshot remained valid after M144: the overlap was visible between the docs intro
  and the focused `RTL Long Text` section even though the promoted gate passed on stable frames.
- The immediate scaffold defect was that `muted_full_width` and `section_title` used fill-width
  text without explicitly opting into `min-width: 0`. Under the card/flex doc layout, that leaves
  startup text measurement vulnerable to an over-wide first pass before resize recovery.
- `apps/fret-ui-gallery/src/ui/doc_layout.rs` now applies `min_width=0` to both helpers and
  `doc_text_helpers_keep_fill_width_min_w_zero` locks the helper shape.
- Gates pass:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery doc_text_helpers_keep_fill_width_min_w_zero --no-fail-fast --no-capture`
  with Nextest run id `3c572126-8add-42ad-8fc7-9b02766e5ba3`.
- Focused runtime diagnostics pass with a clean frame-3 startup screenshot:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779230956616` and AI packet
  `target/fret-diag-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap-v2/sessions/1779230951009-124292/1779230956616/ai.packet`.

## M146: Chart Explicit Link-Axis Mapping Output Gate

Status: complete for ADR 0301 explicit Y-axis output publication coverage.

- Added a retained `fret-chart` mechanism regression proving that an explicit host
  `AxisId -> LinkAxisKey` map publishes an otherwise ambiguous Y domain window to
  `ChartCanvasOutput`. Without the map, the shared multi-series Y axis remains omitted as
  ambiguous; with the map, the output model publishes `dataset=1, field=2` and the fixture
  `[-0.25, 0.75]` window.
- Added a Chart Torture runtime mode behind
  `FRET_UI_GALLERY_CHART_TORTURE_EXPLICIT_Y_LINK_MAP=1`. The UI Gallery snapshot now exposes
  `/chart_torture/output_model/y_explicit_domain_window` and the runtime oracle
  `/chart_torture/runtime_oracles/y_output_model_domain_matches_explicit_fixture`.
- Added `ui-gallery-chart-torture-explicit-y-link-map.json` plus direct protocol roundtrip
  coverage. The gate asserts the explicit Y window is present and matches `min=-250`,
  `max=750` in milli-units.
- Split the gate into its own `ui-gallery-chart-linking-explicit-y-map` suite. A diagnostics
  suite regression covers that this suite stays generic and does not inherit the
  `ui-gallery-chart-torture` pan/zoom-only `chart_sampling_window_shifts_min` tail check.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-diag\src\diag_suite.rs apps\fret-ui-gallery\src\driver\diag_snapshot.rs apps\fret-ui-gallery\src\ui\previews\pages\torture\chart_torture.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs ecosystem\fret-chart\src\retained\canvas.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-chart explicit_link_axis_map_publishes_ambiguous_y_domain_window_to_output_model --no-fail-fast --no-capture`
  with Nextest run id `6d65c626-9933-45ca-b30b-e15ce835bd83`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_explicit_y_link_map --no-fail-fast --no-capture`
  with Nextest run id `1c0b302d-5894-4a6c-a90a-8e4505d72c2e`; and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag build_suite_core_default_post_run_checks_keeps_chart_linking_explicit_y_map_generic --no-fail-fast --no-capture`
  with Nextest run id `9225e20a-b2db-445c-aae1-ef9e369cac20`.
- Runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-chart-linking-explicit-y-map --dir target\fret-diag-chart-linking-explicit-y-map-suite-v1 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-chart-linking-explicit-y-map-suite-v1/sessions/1779226956912-131628/suite.summary.json`
  and run id `1779226972500`.
- Original Chart Torture suite recheck passes after the suite split:
  `target/fret-diag-chart-torture-suite-recheck-v1/sessions/1779226999698-96944/suite.summary.json`
  with run id `1779227011824`.

## M147: Combobox RTL Long Text Client-Height Startup Gate

Status: complete for the decorated-window/client-area companion coverage.

- A follow-up manual screenshot still showed `RTL Long Text` visually overlapping the docs intro,
  but fresh `target\dev-fast\fret-ui-gallery.exe` repro attempts did not reproduce the overlap.
  The current `dev-fast` focused gate still captures a clean frame-3 screenshot at `1083x752`.
- The screenshot included the native Windows title bar, so the existing `1083x752` logical gate was
  also complemented with a shorter `1083x721` client-area gate. This covers the same approximate
  decorated `1624x1128` physical window on a 1.5x scale display after subtracting title-bar height.
- Added `ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json`, its root
  redirect, suite membership, registry entry, and protocol roundtrip coverage.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_doc_intro_client721_startup_non_overlap --no-fail-fast --no-capture`
  with Nextest run id `6814997f-e496-4dff-82a5-2c30636c7c54`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-client721-gate-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779232803236`, AI packet
  `target/fret-diag-combobox-rtl-long-text-client721-gate-v1/sessions/1779232796961-55416/1779232803236/ai.packet`,
  and screenshot size `1625x1082` physical pixels.
- Full Combobox geometry-placement suite passes with 13/13 rows:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-client721-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-combobox-geometry-placement-client721-v1/sessions/1779232841519-125836/suite.summary.json`
  and new client-height script run id `1779232938320`.
- A plain `cargo build -p fret-ui-gallery` attempt to refresh `target\debug\fret-ui-gallery.exe`
  timed out and was not used as evidence. Current runtime evidence is from `target\dev-fast`.

## M148: Chart Explicit Y Linked-Domain Propagation Mechanism Gate

Status: complete for the retained two-chart explicit Y propagation mechanism path.

- Added
  `explicit_y_domain_window_propagates_to_second_linked_chart_output_model` in
  `ecosystem/fret-chart/src/retained/canvas.rs`.
- The regression constructs a source retained chart and a target retained chart from the same
  ambiguous multi-axis spec, applies an explicit `AxisId -> LinkAxisKey` map for the otherwise
  ambiguous Y axis, and gives the target chart a different local initial Y window.
- The test proves the full mechanism chain: source `ChartCanvasOutput` publishes the explicit Y
  window, `LinkedChartGroup::tick` writes it into shared linked-domain state, target retained paint
  consumes that model through `sync_linked_domain_windows`, and target `ChartCanvasOutput` publishes
  the propagated window rather than its local initial window.
- Gates pass:
  `rustfmt --edition 2024 --check ecosystem\fret-chart\src\retained\canvas.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-chart explicit_link_axis_map_publishes_ambiguous_y_domain_window_to_output_model explicit_y_domain_window_propagates_to_second_linked_chart_output_model --no-fail-fast --no-capture`
  with Nextest run id `620beddb-8a62-4de0-81fd-d5f2fadb28f1`.
- Runtime note: `chart_multi_axis_demo` currently logs linked-domain state but does not expose an
  app snapshot provider for direct `fretboard-dev diag` assertions. A follow-up runtime companion
  should add that bounded diagnostics surface before promoting a demo-level gate.

## M149: Fixed-Line-Box Cold Word-Wrap Text Startup Repair

Status: complete for the Combobox RTL Long Text first-visible-frame text/layout pollution.

- The follow-up manual screenshot remained valid: promoted Combobox RTL Long Text diagnostics
  gates captured stable startup frames, but they did not explain why the first visible UI could show
  text overlap until resize. A temporary frame-1 probe against the refreshed debug gallery exposed
  the missing mechanism signal: the docs intro area rendered the internal fixed-line-height sample
  text `Hg` before later frames settled to the real long paragraph.
- Root cause: `ParleyShaper::shape_paragraph_with_wrap` built the real wrapped paragraph into the
  shared Parley layout, then computed fixed line-box ascent/descent. On a cold metrics cache that
  computation calls `shape_single_line_metrics("Hg")` on the same shaper, overwriting the current
  paragraph layout before `shape_paragraph_with_wrap` iterated its lines. The first paint could
  therefore use the internal metrics sample or an under-sized paragraph layout.
- Fixed `crates/fret-render-text/src/parley_shaper.rs` so fixed line-box metrics are computed
  before the real paragraph is built. The internal `Hg` probe can still warm the base metrics cache,
  but it no longer overwrites the layout currently being returned for paint.
- Added
  `fixed_line_box_word_wrap_preserves_paragraph_layout_on_cold_metrics_cache`, which proves a fresh
  shaper with fixed line height and word wrap produces real multi-line paragraph glyphs on the cold
  metrics path.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-render-text\src\parley_shaper.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-render-text --no-fail-fast --no-capture`
  with Nextest run id `a16c3aa8-c5dc-4b48-a26f-df17e39f442e`;
  `cargo build --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`;
  and `cargo build -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`.
- Focused runtime diagnostics pass on the rebuilt `dev-fast` gallery:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-devfast-client721-fixed-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779237680925` and AI packet
  `target/fret-diag-combobox-rtl-long-text-devfast-client721-fixed-v1/sessions/1779237666672-136820/1779237680925/ai.packet`.
- Focused runtime diagnostics also pass on the rebuilt `target\debug\fret-ui-gallery.exe`:
  `target\fret-diag-combobox-rtl-long-text-debug-client721-fixed-v1\sessions\1779238353963-52416\1779238359542\ai.packet`.
  `target\release\fret-ui-gallery.exe` remains an older 2026-05-14 binary and was not used as
  evidence.

## M150: Chart Multi-Axis Linked-Domain Runtime Snapshot Gate

Status: complete for the `chart_multi_axis_demo` linked-domain runtime companion.

- M148 left a runtime gap: the retained chart mechanism test proved source-to-target linked-domain
  propagation, but `chart_multi_axis_demo` only exposed the same behavior through logs and pixels.
  The runtime gate could show a visual change, but could not assert shared/top/bottom linked-domain
  state directly.
- Added a bounded diagnostics snapshot provider to `apps/fret-examples/src/chart_multi_axis_demo.rs`.
  It publishes `/chart_multi_axis/linked_domain_windows` and runtime oracles for the shared
  linked-domain model plus the top and bottom `ChartCanvasOutput` models.
- Added `chart-multi-axis-linked-domain-window-app-snapshot.json`, which runs against
  `chart_multi_axis_demo`, waits for the existing diagnostics-only deterministic top-chart X
  window change to `[-75, 75]`, then asserts shared/top/bottom X windows all match that fixture.
- Added the `chart-multi-axis-linking` suite so this app-specific gate can be rerun without mixing
  the launch target with UI Gallery chart suites.
- Gates pass:
  `rustfmt --edition 2024 --check apps\fret-examples\src\chart_multi_axis_demo.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo build --profile dev-fast -p fret-demo --bin chart_multi_axis_demo`;
  and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_multi_axis_linked_domain_window_app_snapshot --no-fail-fast --no-capture`
  with Nextest run id `1872c4bc-48ce-4a41-a564-ed9f74f83461`.
- Runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\charts\chart-multi-axis-linked-domain-window-app-snapshot.json --dir target\fret-diag-chart-multi-axis-linked-domain-app-snapshot-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\chart_multi_axis_demo.exe`
  with run id `1779239505892` and AI packet
  `target/fret-diag-chart-multi-axis-linked-domain-app-snapshot-v1/sessions/1779239502304-133288/1779239505892/ai.packet`.
- Suite runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite chart-multi-axis-linking --dir target\fret-diag-chart-multi-axis-linking-suite-v1 --session-auto --timeout-ms 420000 --launch -- target\dev-fast\chart_multi_axis_demo.exe`
  with suite summary
  `target/fret-diag-chart-multi-axis-linking-suite-v1/sessions/1779239623009-133816/suite.summary.json`.

## M151: Item Cold Startup Long-Docs Text Runtime Gate

Status: complete for the non-Combobox fixed-line-box wrapped docs text companion.

- Rechecked the follow-up manual Combobox RTL Long Text overlap against current binaries. Rebuilt
  `target\release\fret-ui-gallery.exe` because it was still a 2026-05-14 artifact, then verified the
  focused client-height Combobox gate and a normal OS-window startup capture against current
  `release`. Current rebuilt binaries did not reproduce the visible overlap.
- Added
  `ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap.json` as the adjacent runtime
  lock recommended by F232/F233: it opens a different fixed-line-height wrapped docs paragraph from
  a cold process, focuses the Item `Item vs Field` section, captures layout/screenshot/bundle
  evidence, and asserts the long docs intro leaves space before the section title, description, and
  content.
- Added the root redirect, `ui-gallery-shadcn-runtime-evidence` suite membership, registry entry,
  and protocol roundtrip coverage.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_item_vs_field_doc_intro_client721_startup_non_overlap --no-fail-fast --no-capture`
  with Nextest run id `ee8a6ea4-70e4-4a81-af24-980b7b1f603c`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\item\ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap.json --dir target\fret-diag-item-vs-field-client721-startup-non-overlap-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\release\fret-ui-gallery.exe`
  with run id `1779242679435` and AI packet
  `target/fret-diag-item-vs-field-client721-startup-non-overlap-v1/sessions/1779242677744-141216/1779242679435/ai.packet`.
- Suite runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-ui-gallery-shadcn-runtime-evidence-item-vs-field-v1 --session-auto --timeout-ms 900000 --launch -- target\release\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-ui-gallery-shadcn-runtime-evidence-item-vs-field-v1/sessions/1779242784682-41468/suite.summary.json`
  and new script run id `1779242852622`.

## M152: View Cache Dynamic Text Mutation Runtime Gate

Status: complete for cached-subtree dynamic wrapped-text mutation coverage.

- The existing View Cache gate proved counter mutation and controlled Popover open/close state, but
  it did not prove that visible wrapped text and following layout inside the cached subtree refresh
  through the same model mutation.
- Added a dynamic text probe to the View Cache harness page. `counter=0` exposes a short baseline;
  `counter>0` exposes a longer wrapped sentence inside the cached subtree. The UI Gallery app
  snapshot now publishes `/view_cache/dynamic_text_len` and `/view_cache/dynamic_text_wrapped`.
- Added `ui-gallery-view-cache-dynamic-text-mutation-through-cache.json`, its root redirect, suite
  membership, registry entry, and protocol roundtrip coverage. The gate asserts app-snapshot state,
  visible label text, renderer text trace coverage, layout sidecars, screenshots, bundle evidence,
  dynamic-text size, dynamic-text-to-Popover spacing, and Popover/List non-overlap.
- The first runtime draft exposed an over-constrained script oracle, not a mechanism defect:
  Popover trigger and retained list are adjacent under current CardContent semantics, so that final
  assertion now proves non-overlap instead of requiring an `8px` gap.
- Gates pass:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\previews\pages\harness\view_cache.rs apps\fret-ui-gallery\src\driver\diag_snapshot.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_view_cache_dynamic_text_mutation_through_cache --no-fail-fast --no-capture`
  with Nextest run id `bd7f6552-74b2-416f-b0f0-55bb8f82742f`; and
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\view-cache\ui-gallery-view-cache-dynamic-text-mutation-through-cache.json --dir target\fret-diag-ui-gallery-view-cache-dynamic-text-mutation-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779244734657` and AI packet
  `target/fret-diag-ui-gallery-view-cache-dynamic-text-mutation-v2/sessions/1779244725576-99248/1779244734657/ai.packet`.
- Full View Cache suite passes 2/2:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-view-cache --dir target\fret-diag-ui-gallery-view-cache-suite-dynamic-text-v1 --session-auto --timeout-ms 600000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with suite summary
  `target/fret-diag-ui-gallery-view-cache-suite-dynamic-text-v1/sessions/1779244758600-135808/suite.summary.json`.

## M153: HitTestOnly Paint-Cache Replay Runtime Gate

Status: complete for stable cached hit-test-only replay counter coverage.

- Promoted `ui-gallery-hit-test-only-paint-cache-probe-sweep.json` from a local capture script into
  a self-contained UI Gallery runtime gate. The script now declares `gallery-dev`, injects
  `FRET_UI_GALLERY_PAINT_CACHE=1`, uses nav search to select the hidden
  `hit_test_only_paint_cache_probe` page, verifies `/selected_page`, resets diagnostics, sweeps the
  pointer over a stable cached canvas, asserts the region size, and captures a bundle.
- Added protocol/runtime predicates for `paint_cache_hit_test_only_replay_allowed_ge` and
  `paint_cache_hit_test_only_replay_rejected_key_mismatch_le`. Bootstrap diagnostics now export the
  matching frame counters and can evaluate them from the recent debug snapshot ring.
- The initial focused run exposed a script-authoring hole: without page navigation the script stayed
  on the default Overlay page and timed out. A later suite run exposed a real diagnostics hygiene
  issue: the probe page duplicated `ui-gallery-hit-test-only-probe-region`; the outer panel now uses
  `ui-gallery-hit-test-only-probe-panel` while the inner hit region keeps the stable id.
- No paint-cache key-mismatch defect reproduced after promotion. The passing gate proves at least
  one hit-test-only replay was allowed and zero key-mismatch rejections were observed during the
  sweep.
- Gates pass:
  `python tools\check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_paint_cache_hit_test_only_replay_counters_serialize script_v2_roundtrip_ui_gallery_hit_test_only_paint_cache_probe_sweep --no-fail-fast --no-capture`
  with Nextest run id `5c85e308-22d9-4ab5-8d94-3ca48ccf3819`;
  `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics paint_cache_hit_test_only_replay_predicates_count_ring_snapshot_maxes --no-fail-fast --no-capture`
  with Nextest run id `34eb1b6e-b6f7-4d06-80b0-ea1b1c6e764a`; and
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\diag\ui-gallery-hit-test-only-paint-cache-probe-sweep.json --dir target\fret-diag-hit-test-only-paint-cache-probe-sweep-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with run id `1779247865248` and AI packet
  `target/fret-diag-hit-test-only-paint-cache-probe-sweep-v2/sessions/1779247851157-129852/1779247865248/ai.packet`.
- The new `ui-gallery-hit-test-only-paint-cache` suite passes with zero-warning lint policy:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-hit-test-only-paint-cache --dir target\fret-diag-hit-test-only-paint-cache-suite-v3 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with suite summary
  `target/fret-diag-hit-test-only-paint-cache-suite-v3/sessions/1779249174760-142600/suite.summary.json`.

## M154: Moved Cache-Root Hit Path Reuse Gate

Status: complete for the focused stale root-only hit-path-cache guard.

- Added `prepaint_interaction_cache_root_move_invalidates_stale_root_only_hit_path` to cover the
  gap left by the older moving cache-root regression: the older test cleared
  `ui.hit_test_path_cache`, while this one keeps a primed root-only cached path alive across a
  clean view-cache root move.
- The new test first hits the root outside the cached child, moves the clean cache root and leaf
  under the same pointer position, reuses translated interaction records through prepaint, and then
  proves hit testing returns the moved leaf rather than the stale root path.
- The guard also asserts `hit_test_path_cache_misses` increments on the second hit, proving the
  root-only cached path is rejected and full hit testing is used before accepting the moved child.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\tree\tests\prepaint.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui prepaint_interaction_cache_root_move_invalidates_stale_root_only_hit_path prepaint_interaction_cache_replay_translates_records_when_cache_root_moves --no-fail-fast --no-capture`
  with Nextest run id `84167c6c-c03b-4feb-aa11-0693f55659b2`.

## M155: Hit-Test Path Cache Higher-Z Sibling Guard

Status: complete for focused stale child-path z-order reuse coverage.

- Added `hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer` to
  complement M154. It primes a cached `root -> lower_child` path, moves a higher-z sibling under
  the same pointer, and proves cached-path reuse rejects the stale lower-child path before fallback
  hit testing accepts the moved higher sibling.
- The test asserts both sides of the cache behavior: `hit_test_path_cache_misses` increments for
  the stale child path, then `hit_test_path_cache_hits` increments after fallback refreshes the
  path to the higher-z sibling.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\tree\tests\hit_test.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer hit_test_layers_cached_reuses_path_and_respects_layer_order --no-fail-fast --no-capture`
  with Nextest run id `92315d8d-56fd-4c3e-bfc1-bbfc849e954b`.

## M156: Pointer-Move Dispatch Stale Hit-Path Guard

Status: complete for focused pointer-move dispatch stale-path coverage.

- Added `pointer_move_dispatch_rejects_stale_path_when_higher_z_sibling_moves_under_pointer` to
  lift M155 from a direct hit-test query into real window dispatch. The test sends a
  `PointerEvent::Move` through `UiTree::dispatch_event`, verifies the lower-z widget receives the
  first move, moves a higher-z sibling under the same pointer, and sends a second move with the
  path cache still live.
- The second move must not be delivered to the stale lower-z target. Instead dispatch rejects the
  cached lower-child path, increments `hit_test_path_cache_misses`, delivers the move to the moved
  higher-z sibling, and refreshes the cache so a third move records a cache hit for the higher-z
  path.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\tree\tests\hit_test.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui pointer_move_dispatch_rejects_stale_path_when_higher_z_sibling_moves_under_pointer hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer --no-fail-fast --no-capture`
  with Nextest run id `093b8a5d-e67a-4b35-ab82-e02389f63173`.


## M157: Hit-Test Path-Cache Runtime Hit Counter Gate

Status: complete for the UI Gallery hit-test path-cache runtime companion.

- Extended `ui-gallery-hit-test-only-paint-cache-probe-sweep.json` beyond paint-cache replay: it now
  disables bounds-tree queries and waits for `hit_test_path_cache_hits_ge(min=1)`, proving the
  sweep reaches the cached-path fast path rather than only fallback traversal.
- The first strict runtime run failed even though paint-cache replay was allowed. Large-ring bundles
  showed `hit_test_path_cache_hits=0` and `hit_test_path_cache_misses=2` across the sweep. This
  found an over-conservative mechanism defect in cached-path sibling validation: transformed or
  non-clipping higher-z siblings forced misses even when real hit testing showed they did not
  intercept the point.
- `try_hit_test_along_cached_path` now validates higher-z siblings with `hit_test_node` instead of
  rejecting on raw bounds/transform heuristics. Focused tests prove both sides: non-hit-testable
  overlapping siblings no longer poison reuse, while a transformed higher-z sibling that truly
  covers the pointer still forces a miss and wins through fallback hit testing.
- The slice also keeps `fret-ui-gallery --features gallery-dev` buildable by updating the AI prompt
  input cursor snippet to current text-layout and foreground APIs.
- Gates pass:
  `rustfmt --edition 2024 --check crates\fret-ui\src\tree\hit_test.rs crates\fret-ui\src\tree\tests\hit_test.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui hit_test_layers_cached_ignores_non_hit_testable_overlapping_higher_z_siblings hit_test_layers_cached_checks_transformed_higher_z_siblings_before_reuse hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer pointer_move_dispatch_rejects_stale_path_when_higher_z_sibling_moves_under_pointer --no-fail-fast --no-capture`
  with Nextest run id `3d58d069-af7b-4675-a455-9f6ace214151`;
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics hit_test_path_cache_predicates_count_ring_snapshot_maxes paint_cache_hit_test_only_replay_predicates_count_ring_snapshot_maxes --no-fail-fast --no-capture`
  with Nextest run id `affc9fa2-0c6d-47e8-b252-ae83c14e9059`; and
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_hit_test_path_cache_counters_serialize script_v2_roundtrip_ui_gallery_hit_test_only_paint_cache_probe_sweep --no-fail-fast --no-capture`
  with Nextest run id `f0505399-96a7-4a92-95d7-398b48b1fd96`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\diag\ui-gallery-hit-test-only-paint-cache-probe-sweep.json --dir target\fret-diag-hit-test-only-paint-cache-path-cache-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with run id `1779259408910` and AI packet
  `target/fret-diag-hit-test-only-paint-cache-path-cache-v2/sessions/1779259321228-145468/1779259408910/ai.packet`.
- The `ui-gallery-hit-test-only-paint-cache` suite passes with the stricter path-cache predicate:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-hit-test-only-paint-cache --dir target\fret-diag-hit-test-only-paint-cache-suite-path-cache-v2 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  with summary
  `target/fret-diag-hit-test-only-paint-cache-suite-path-cache-v2/sessions/1779259631852-148980/suite.summary.json`.

## M158: RadioGroup Checked-State Semantics Runtime Gate

Status: complete for the focused RadioGroup checked-state mutation companion.

- Added `ui-gallery-radio-group-checked-state-mutation.json`, which navigates to the public
  RadioGroup page, scrolls to the Label Association example, proves the initial Free/Pro/Enterprise
  checked-state semantics, then clicks Pro and Enterprise and asserts checked-state transfer after
  each pointer activation.
- Added the focused `ui-gallery-radio-group-semantics` suite plus broad
  `ui-gallery-shadcn-runtime-evidence` membership, registry entry, and protocol roundtrip coverage.
- The broad runtime-evidence suite rerun exposed an unrelated existing Command
  retained-active-descendant `script_stalled_no_frames` failure before it reached RadioGroup. The
  RadioGroup gate therefore uses the dedicated suite for durable evidence; the Command no-frame
  failure remains a separate diagnostics-stability follow-up.
- Gates pass:
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_radio_group_checked_state_mutation --no-fail-fast --no-capture`
  with latest Nextest run id `7ced9cb1-5ecc-43bd-b118-4fc3cd0c6681`; and
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\radio-group\ui-gallery-radio-group-checked-state-mutation.json --dir target\fret-diag-radio-group-checked-state-mutation-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779261557779` and AI packet
  `target/fret-diag-radio-group-checked-state-mutation-v1/sessions/1779261539435-153996/1779261557779/ai.packet`.
- Dedicated runtime suite passes:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-radio-group-semantics --dir target\fret-diag-radio-group-semantics-suite-v4 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with summary
  `target/fret-diag-radio-group-semantics-suite-v4/sessions/1779263168285-151724/suite.summary.json`.

## M159: Desktop Repeating-Timer Redraw Starvation Repair

Status: complete for the runner scheduling defect that blocked the broad shadcn runtime-evidence
suite.

- The previous RadioGroup slice left a follow-up: the broad `ui-gallery-shadcn-runtime-evidence`
  suite could fail before reaching RadioGroup because
  `ui-gallery-command-retained-active-descendant-action-state.json` stalled with
  `script_stalled_no_frames`.
- The root cause was desktop runner timer catch-up. Repeating timers were rearmed from the stale
  `now` captured at the start of `fire_due_timers`, so a slow diagnostics keepalive handler could
  make the same repeating timer overdue again inside the same fixed-point effect-drain turn. That
  starved the winit `RedrawRequested` the handler had just requested.
- `TimerEntry::last_fired_tick` now prevents a repeating timer from firing more than once per
  runner tick, and `finish_fired_timer` rearms repeating timers relative to handler completion
  time. Asset-reload polling timers initialize the same guard.
- Diagnostics no-frame keepalive handling now also preserves explicit redraw/RAF effects for
  already-started `wait_frames`, effect-only steps, keyboard/text/IME injection, pointer-move
  injection, script startup, and post-command-flush injected input.
- Gates pass:
  `cargo test --profile dev-fast -p fret-launch repeating_timer --lib -- --nocapture`;
  `cargo test --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics --lib no_frame_keepalive -- --nocapture`;
  and `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`.
- Focused Command runtime diagnostics pass:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\command\ui-gallery-command-retained-active-descendant-action-state.json --dir target\fret-diag-command-retained-active-descendant-action-state-runner-timer-fresh-20260521 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with run id `1779298834262` and AI packet
  `target/fret-diag-command-retained-active-descendant-action-state-runner-timer-fresh-20260521/sessions/1779298813208-173816/1779298834262/ai.packet`.
- The broad `ui-gallery-shadcn-runtime-evidence` suite now passes 13/13:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-runner-timer-fresh-20260521 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  with summary
  `target/fret-diag-shadcn-runtime-evidence-runner-timer-fresh-20260521/sessions/1779299075645-7824/suite.summary.json`.

## M160: Runner Repeating-Timer Overlap Stress Gate

Status: complete for the cheap owning-layer regression that generalizes M159.

- Added `overlapping_repeating_timers_do_not_catch_up_inside_one_runner_tick` to cover two overdue
  repeating timers in the same runner tick: a window-targeted script-keepalive-style timer and a
  windowless asset-reload-poll-style timer.
- The test forces both timers stale again after their first fire without advancing `tick_id`. The
  second `fire_due_timers` call must return `false`, proving the same-tick guard applies across
  overlapping timers and not only one isolated token.
- After `tick_id` advances, both timers are allowed to fire again and update `last_fired_tick` to
  the new tick.
- Gates pass:
  `cargo test --profile dev-fast -p fret-launch overlapping_repeating_timers --lib -- --nocapture`;
  and `cargo test --profile dev-fast -p fret-launch repeating_timer --lib -- --nocapture` with 3
  passing timer tests.
