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
