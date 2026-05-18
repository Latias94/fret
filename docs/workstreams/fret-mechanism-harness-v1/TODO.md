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
- [x] Add prepaint virtual-list window-update fixture coverage for viewport/items-revision detail
  attribution.
  - Result: `virtual_list_window_update_v1.json` now covers scroll offset, viewport resize, items
    revision, and scroll-to-item window shifts. The fixture exposed a confirmed mechanism defect:
    debug telemetry recorded the specific viewport/items-revision reason, but actual cache-root
    dirty attribution fell back to the generic prefetch/window-update detail. Prepaint now shares
    one classifier for debug telemetry and dirty cache-root attribution.
- [x] Extend prepaint virtual-list window-update fixtures to length-shrink/input-change cases.
  - Result: a stale render-window count now classifies as `InputsChange` before offset deltas and
    uses the dedicated `scroll_handle_inputs_change_window_update` invalidation detail. The first
    run showed the length-shrink case was not exposed as an input change.
- [x] Add a DataTable retained filter-shrink runtime companion for the virtual-list
  `InputsChange` mechanism.
  - Result: `ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json` drives the real
    DataTable torture page with `FRET_UI_GALLERY_DATA_TABLE_RETAINED=1`, applies a global filter,
    and asserts a layout-sourced retained virtual-list `inputs_change` record. The first runtime
    pass exposed a component defect: retained DataTable rows were built from raw data instead of the
    filtered row order. The follow-up exposed a mechanism defect: layout-time virtual-list
    classification did not share the prepaint `InputsChange` classifier.
- [x] Add a DataTable view-cache filter-shrink runtime companion for non-retained
  `InputsChange` invalidation detail.
  - Result: `ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json` drives the
    same DataTable torture page with `FRET_UI_GALLERY_VIEW_CACHE=1`, applies a global filter, and
    asserts `reason=inputs_change`, `apply_mode=non_retained_rerender`, and
    `invalidation_detail=scroll_handle_inputs_change_window_update`. The first draft exposed a
    harness precondition issue rather than a mechanism defect: default Gallery runs have
    `view_cache_active=false`, so no invalidation detail should be expected.
- [x] Add a DataTable RTL idle-stability runtime companion.
  - Result: `ui-gallery-data-table-rtl-idle-stability.json` scrolls the public DataTable page to the
    RTL section, waits for root/footer bounds stability, samples the Gallery content viewport for 60
    no-input frames, and passes. This did not reproduce a scroll-jitter mechanism defect.
- [x] Add UI Gallery diagnostics for shell-level runtime theme/motion preference changes.
  - Result: `ui-gallery-motion-preset-runtime-token-mutation.json` now drives the always-visible
    Gallery Theme/Motion preset selectors and asserts both shell model state and effective global
    Theme runtime tokens through `app_snapshot_field_equals`. The first runtime draft exposed a
    diagnostics oracle issue: strict JSON equality on raw `f32` token values produced false
    failures, so the Gallery app snapshot now publishes rounded readable values plus milli-scaled
    integer fields for stable token assertions.
- [x] Add runner/platform-injected UI Gallery diagnostics for runtime platform
  preference/environment changes once a stable demo page exists.
  - Result: `ui-gallery-platform-preferences-runtime-environment-mutation.json` now drives
    diagnostics-only platform preference updates through the runner `WindowMetricsService` path and
    asserts both the UI Gallery app snapshot and an `ElementContext` environment-query probe see
    the same color scheme, reduced-motion, and text-scale values. The first real run exposed a
    harness script defect rather than a mechanism defect: the script waited for the Motion Presets
    page probe without first navigating to that page. The script now enters the page explicitly and
    the runtime gate passes. Fresh dev-fast evidence also passes with run id `1779029357027`, and
    `COVERAGE_MAP.md` now treats this runtime path as covered rather than a current gap.
- [x] Add a UI Gallery pointer occlusion diagnostics gate once a stable overlay demo exposes test
  ids for underlay and overlay state.
  - Result: `ui-gallery-context-menu-occlusion-wheel-pass-through.json` now asserts the content
    viewport starts at `scroll.y=0`, has a non-zero scroll range, receives a wheel through
    `BlockMouseExceptScroll`, ends with `scroll.y != 0`, and keeps the context menu mounted.
- [x] Add a UI Gallery captured-pointer lifecycle diagnostics gate.
  - Result: `ui-gallery-scrollbar-drag-baseline-content-growth.json` now asserts
    `input_pointer_capture_active_is active=true` immediately after scrollbar `pointer_down`, keeps
    it true during drag, and asserts `active=false` after `pointer_up`, while preserving the
    existing scroll progress oracle.
- [x] Add a UI Gallery captured-pointer owner/cancel companion once a stable demo exposes
  pointer-capture owner ids.
  - Result: `captured_is` now proves the scrollbar owns capture during drag and clears after
    `pointer_up`/`pointer_cancel`. The first runtime owner gate exposed a real mechanism defect:
    semantics snapshots did not refresh when live pointer-capture owner state changed without a
    layout/semantics dirty bit.
- [x] Add captured-pointer underlay blocking and multi-pointer/cross-window runtime probes once a
  stable public demo exposes underlay activation/status selectors.
  - Result: UI Gallery now covers captured-underlay ScrollArea touch probing, and docking
    arbitration now covers a cross-window dock drag where pointer 1 touches the under-moving
    main-window viewport while `dock_viewport_capture_active_is` stays false.
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
- [x] Add a UI Gallery semantics/accessibility gate for filtered default selectors versus raw
  diagnostics visibility on a stable recipe page.
  - Result: `ui-gallery-separator-decorative-hidden-semantics.json` proves a shadcn decorative
    Separator divider is absent from default `test_id` selectors while `raw_semantics_hidden_is`
    observes it as effectively hidden.
- [x] Add dynamic expanded-state semantics mutation runtime coverage on a stable UI Gallery recipe.
  - Result: `ui-gallery-accordion-usage-toggle.json` now asserts `expanded_is` true -> false ->
    true on the Accordion Usage trigger while preserving panel mount/unmount checks.
- [x] Add dynamic active-descendant or selected-state mutation runtime coverage on stable UI
  Gallery composite recipe pages.
  - Result: Combobox auto-highlight disabled/first-match scripts and Command controlled-selection
    value/ArrowDown scripts now assert active-descendant mutation and are promoted into durable
    component/conformance suites.
- [x] Add dynamic live-region update runtime coverage once a stable status/live-region page exposes
  selectors and deterministic text/state changes.
  - Result: `ui-gallery-sonner-live-region-mutation.json` asserts Sonner's `Notifications`
    viewport appears with `semantics_live_is=polite` and `semantics_live_atomic_is=false` while a
    toast is mounted, then disappears after dismissal.
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
- [x] Add default-compatible non-modal click-through activation-status coverage once public pages
  expose stable underlay/status probes.
  - Result: the Overlay preview now exposes an independent
    `ui-gallery-overlay-underlay-activated` status flag. The existing Popover click-through and
    DropdownMenu non-modal outside-press gates now assert real underlay activation in addition to
    dismissal/focus outcomes.
  - Finding: this slice found a harness weakness, not a component defect. The old gates used
    focus/dismiss as proxy signals and could miss an overlay policy regression where the underlay
    received focus but its activation handler did not run.
  - Evidence: `ui-gallery-popover-click-through-outside-press-focus-underlay.json` passed with run
    id `1778906489806`; `ui-gallery-dropdown-nonmodal-outside-press-focus-underlay.json` passed
    with run id `1778906522304`; roundtrip gate
    `script_v2_roundtrip_ui_gallery_popover_click_through_outside_press_focus_underlay
    script_v2_roundtrip_ui_gallery_dropdown_nonmodal_outside_press_focus_underlay` passed with
    Nextest run id `c563620f-80b3-4933-9da6-48d657c68a38`.
- [x] Add a non-list semantics/action-state runtime gate where visual behavior can pass while
  accessibility action metadata is stale.
  - Result: `ui-gallery-switch-read-only-action-state.json` now gates a read-only Switch through
    real UI Gallery runtime. The first focused oracle found a real shadcn recipe defect:
    `Switch::read_only(true)` blocked pointer mutation but still exposed `actions.invoke=true`.
    The fix attaches `read_only=true` and `invokable=false` semantics while preserving focusability.
    Diagnostics now has a `read_only_is` predicate and the runtime gate asserts `read_only=true`,
    `disabled=false`, `focus=true`, `invoke=false`, and no checked-state mutation after pointer,
    associated-label, Space, or Enter activation attempts.
- [x] Add a dynamic companion for non-list read-only action-state mutation.
  - Result: `ui-gallery-switch-read-only-dynamic-action-state.json` now toggles the same Switch
    from read-only to editable and back, proving `read_only` and `invoke` semantics refresh across
    frames and that checked-state mutation is allowed only while editable. No new runtime defect was
    reproduced after the F112 Switch fix. The first focused test draft caught a harness modeling
    issue: changing a model without rerendering the declarative root cannot prove component props
    changed.
- [x] Add a command-gated non-list action-state mutation companion.
  - Result: `ui-gallery-switch-command-gated-action-state.json` now drives a Switch whose invoke
    action is externally disabled and re-enabled through `WindowCommandEnabledService`. The first
    runtime pass found a diagnostics harness gap: the Gallery driver handled the command after the
    UI tree reported it unhandled, but did not record a `handled_by_driver=true` dispatch trace.
    The driver now records handled command decisions for owned gallery command paths, and the
    strict runtime gate passes with command dispatch, `disabled`, `invoke`, and checked-state
    mutation assertions.
- [x] Promote Combobox visual/style coverage into an explicit fixture-style matrix that tracks
  component state, theme, viewport, screenshot gate, geometry predicates, and current owner/gap.
- [x] Harden the Button Group size gate with stable icon-only `Add` anchors and geometry predicates.
- [x] Promote the Button Group family into an explicit fixture-style matrix for text/icon
  alignment, truncation, and theme/viewport variants.
  - Result: `ui-gallery-button-group` now runs the family matrix as a durable suite. The first run
    exposed accessibility lint noise and a real Button Group Select teaching-surface gap:
    unlabeled Select triggers were fixed in UI Gallery, and `fret-diag` lint now treats
    `labelled_by` relations as valid accessible-name sources.
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
- [x] Add a Combobox RTL long-text companion so leading/trailing icon/checkmark ownership is
  covered in both directions.
  - Result: `ui-gallery-combobox-rtl-long-text-geometry.json` is already promoted into
    `ui-gallery-combobox`, `ui-gallery-rtl-smoke`, and `ui-gallery-shadcn-conformance`. The audit
    rerun passed and proved the physical-left RTL chevron, label-after-chevron spacing,
    content-shell top collision flip, and physical-right RTL checkmark/label separation.
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
- [x] Add ContextMenu submenu branch/corridor runtime routing coverage.
  - Result: `ui-gallery-context-menu-submenu-branch-corridor-routing.json` proves moving into the
    submenu and back to the parent trigger keeps the submenu open, while moving to another parent
    item or away from the trigger closes the nested submenu without closing the root menu.
- [x] Harden the ContextMenu submenu routing gate against page-content versus overlay-content
  selector collisions.
  - Result: the overlay/focus suite caught `ui-gallery-context-menu-submenu-content` as a duplicate
    id shared by the Gallery snippet content container and the mounted overlay menu. The overlay
    panel selector is now `ui-gallery-context-menu-submenu-overlay-content`, the submenu routing and
    safe-corridor scripts target that id, and the full overlay/focus suite passes with 8 scripts and
    zero lint findings.
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
- [x] Add runtime diagnostics support for explicit pointer ids on pointer-session steps and gate a
  UI Gallery multi-pointer captured-underlay probe.
- [x] Add a cross-window docking/tear-out multi-pointer probe now that runtime scripts can hold one
  captured pointer while another pointer targets the underlay/window-under-moving surface.
  - Result:
    `docking-arbitration-demo-multiwindow-dock-drag-suppresses-viewport-touch.json` proves pointer
    0's dock drag remains active while pointer 1 touches/releases the main-window viewport, and the
    new `dock_viewport_capture_active_is` predicate proves no competing viewport capture starts.
- [x] Add a selected-state mutation runtime gate for Select commit/reopen semantics.
  - Result:
    `ui-gallery-select-commit-and-label-update.json` now waits for overlay placement/bounds,
    commits Banana, reopens the popup, and asserts Banana is selected while Apple is not. The first
    promoted run exposed an early overlay-click harness defect before passing after the stability
    fix.
- [x] Add a second selected-state mutation companion for Tabs or DataTable faceted filters if a
  stable selected semantics surface is available.
  - Result:
    `ui-gallery-tabs-selected-state-mutation.json` proves Account and Password tab triggers swap
    selected flags after activation, covering inline selected semantics in addition to Select's
    overlay-backed selected item.
- [x] Add a collection metadata mutation gate for `pos_in_set_is`/`set_size_is` across filtering,
  pagination, or virtual-list window changes.
  - Result: `ui-gallery-command-scrollable-collection-metadata-mutation.json` opens the shadcn
    Command scrollable dialog, proves `Code Editor` starts as item 23/23, filters to
    `code editor`, then proves it becomes item 1/1 while unrelated items disappear. The gate
    passed; no Command recipe or semantics mechanism defect was reproduced. The fixed gap was
    harness durability: collection metadata predicates existed, but there was no promoted shadcn
    runtime mutation gate for filtered collections.
- [x] Add collection metadata mutation coverage for pagination or virtual-list window changes so
  the next slice can test retained/windowed node reuse, not only filtered list rebuilding.
  - Result: `ui-gallery-data-table-default-pagination-collection-metadata.json` exposes
    diagnostics-only default DataTable row anchors, proves page 1 rows are 1/2 and 2/2, page 2 rows
    are 1/2 and 2/2, and the last page row is 1/1. The first runtime pass found a harness script
    defect: `Next` was partially outside the window, so the click had `no_hit`. The passing script
    now scrolls the button into view and gates bounds before clicking.
- [x] Add retained/windowed collection metadata mutation coverage for virtual-list scroll or bounce
  reuse, where stale row semantics are more likely than in paginated rebuilds.
  - Result: `ui-gallery-virtual-list-retained-collection-metadata-bounce.json` proves retained
    Virtual List Torture row metadata across top, boundary-scroll, detach, and bounce-back states.
    The slice found a mechanism observability gap: `SemanticsDecoration` and `SemanticsProps`
    could not stamp collection metadata, so non-pressable retained rows were not queryable without
    policy-layer workarounds.
- [x] Add a retained/windowed non-list semantics mutation gate, such as tree hierarchy
  `level`/expanded metadata on FileTree/Tree torture or row action-state mutation on retained
  DataTable rows.
  - Result: `level_is` is now part of the diagnostics/mechanism predicate vocabulary, Tree/FileTree
    rows publish hierarchy `level` and parent-row `expanded`, and
    `ui-gallery-tree-retained-hierarchy-semantics-toggle.json` gates retained Tree row metadata
    across collapse/expand reuse. The runtime gate found a real `fret-ui-kit` Tree policy defect:
    row toggle buttons dispatched `tree.toggle.<id>` commands that had no handler, so expanded
    state never changed. Tree now updates its owned `TreeState` model directly and the retained
    Tree diagnostics suite passes.
- [x] Add a first non-list action-state mutation runtime gate for `disabled` and semantics action
  state.
  - Result: `disabled_is` and `semantics_action_is` are now protocol/runtime/mechanism predicates,
    diagnostics bundles export the full core semantics action set, and
    `ui-gallery-data-table-default-pagination-collection-metadata.json` proves DataTable Prev/Next
    disabled plus `invoke` action-state mutation across first and final pagination states. The
    gate passed after rebuilding `fretboard-dev`; no DataTable component defect was reproduced.
- [x] Add retained/windowed action-state mutation coverage where reused DataTable/FileTree/Tree
  rows can change `selected` or `invoke` without a full node rebuild.
  - Result: `ui-gallery-tree-retained-hierarchy-semantics-toggle.json` now checks retained Tree
    row `selected_is`, `disabled_is=false`, and `semantics_action_is(invoke)=true`. It selects row
    `1000000`, collapses root so the selected row detaches, re-expands root so the row reattaches
    still selected, then selects row `2000000` and proves selected state moves away from the old
    row. The single runtime gate and full retained Tree suite passed; no retained Tree stale
    selected/invoke defect was reproduced.
- [x] Add retained/windowed disabled action-state mutation coverage where reused rows can change
  `disabled` and `invoke` availability dynamically, not just remain enabled.
  - Result: Tree Torture now exposes diagnostics-only control
    `ui-gallery-tree-toggle-target-disabled`, and
    `ui-gallery-tree-retained-hierarchy-semantics-toggle.json` proves retained row `2000000`
    flips `disabled_is` and `semantics_action_is(invoke)` in both directions. Disabled-row pointer
    clicks do not move selection; re-enabled clicks select the row again. The single runtime gate
    and full retained Tree suite passed; no retained Tree stale disabled/invoke defect was
    reproduced.
- [x] Add keyboard/action activation suppression coverage on a focusable-disabled recipe/primitive
  surface.
  - Result: `ui-gallery-accordion-focusable-disabled-keyboard-suppression.json` now covers the
    Radix-style Accordion route where the open non-collapsible trigger is disabled for assistive
    technology, remains focusable, and suppresses invoke plus Enter/Space activation. The first
    runtime run found a real shadcn Accordion layout defect: the trigger exported focusable
    disabled semantics but had zero-width bounds, so focus repair cleared the route. Accordion item
    wrapper columns now fill width with `min_width=0`, focused recipe tests lock non-empty bounds
    plus focus repair, and the runtime gate passes.
- [x] Add a synthetic mechanism-level focusable-disabled activation fixture.
  - Result: `PressableKeyActivation::None` is now a core mechanism, with
    `pressable_key_activation_v1.json` proving Enter+Space, Enter-only, no-keyboard-activation,
    focusable-disabled semantics, and fully disabled Pressable outcomes. Accordion's aria-disabled
    helper now consumes this mechanism instead of relying only on non-collapsible no-op policy.
- [x] Add a second recipe family runtime gate, such as menu/listbox/command-style
  disabled-but-focusable items, to prove the focus/invoke/key separation is not only covered through
  Accordion's non-collapsible policy.
  - Result: DropdownMenu now exposes an explicit `focusable_when_disabled(true)` opt-in for regular
    items. The UI Gallery gate
    `ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json` proves real ArrowDown
    roving focus reaches a disabled `API` item, semantics reports `disabled=true`,
    `focus=true`, `invoke=false`, and Enter/Space do not dispatch or close the menu. The first
    script drafts exposed harness gaps: direct focus inside overlays was the wrong proof path, the
    status text oracle was stale, and the new script needed suite promotion before the registry
    would index it.
- [x] Add a command/listbox-style disabled-but-focusable runtime gate so the next proof exercises
  active-descendant/list semantics instead of menu roving focus.
- [x] Add retained/windowed active-descendant action-state mutation coverage where a disabled or
  invokable row detaches, reattaches, or changes availability under filtering/virtualization.
  - Result: `combobox_active_descendant_interaction_v1.json` now includes a retained virtual-list
    case where row 2 is the active descendant, scrolls out of the rendered semantics window, then
    reattaches after its disabled/invoke state changes. The synthetic harness found a real
    `fret-ui` mechanism defect: the row disappeared from the snapshot while the input still
    resolved `active_descendant` to its old node. `UiTree::refresh_semantics_snapshot` now filters
    `active_descendant`, `labelled_by`, `described_by`, and `controls` relations to nodes present in
    the current snapshot. `ui-gallery-command-retained-active-descendant-action-state.json` gates
    the same flow through the Command page, and the rerun bundles lint cleanly after adding an
    accessible label to the demo input.
- [x] Add relation-edge normalization coverage for `labelled_by`, `described_by`, and `controls`
  targets that detach, reattach, or cross overlay/root boundaries, so the active-descendant fix is
  not the only snapshot-local relation proof.
  - Result: `semantics_relations_v1.json` now includes
    `relation-targets-detach-reattach-clear-stale-edges`, a multi-frame fixture where the source
    keeps last-known declarative element ids while label, description, and controlled targets
    detach and reattach. The fixture proves detached targets are absent, relation arrays are empty
    while detached, and all three relations resolve again after reattach. No new mechanism defect
    was reproduced because the F98 snapshot-local relation filtering already covers these edges.
    The fixed gap was diagnostics/harness capability: scripts now have
    `semantics_relation_includes` and `semantics_relation_is_empty` predicates for raw relation
    edges, plus typed builder helpers for script generation.
- [x] Add a UI Gallery runtime relation-edge gate for cross overlay/root-boundary source-target
  ownership, using the new `semantics_relation_includes`/`semantics_relation_is_empty` predicates.
  - Result: `ui-gallery-select-commit-and-label-update.json` now proves Select trigger
    `controls` the mounted listbox, the listbox is `labelled_by` the trigger, the trigger controls
    relation clears after commit/close, and the relation returns after reopen. The runtime gate
    found a diagnostics harness defect: relation predicates reused ordinary modal-barrier-scoped
    selectors, so the underlay trigger was clipped out while the popup barrier was active even
    though the semantics edge existed. Relation predicates now use endpoint-specific selector
    resolution, while ordinary selectors still respect modal barrier scoping.
- [x] Add a Select active-descendant runtime gate that exercises external active-row state through
  view-cache reuse.
  - Result: `ui-gallery-select-roving-skips-disabled-orange.json` caught a real Select
    invalidation defect: keyboard navigation updated active-row state to the next enabled item, but
    the runtime semantics snapshot kept the previous active descendant because the state lived
    outside the element tree and only requested redraw. `fret-ui-kit` and `fret-ui-shadcn` now call
    `host.notify` when active-row changes, and focused tests plus the full `ui-gallery-select`
    suite pass.
- [x] Classify Select wheel and active-descendant lint evidence so scrollable listbox state does
  not create false fatal harness failures.
  - Result: `wheel_scroll_hit_changes_test_id` now ignores wheel frames before the target test id
    exists, and `layout.active_item_out_of_window` is a warning instead of an error when the active
    descendant is inside a scrollable ancestor. This keeps the Select wheel gates focused on
    component wheel routing while preserving useful lint evidence.
- [x] Add a Select Scrollable placement baseline with layout sidecar evidence.
  - Result: `ui-gallery-select-scrollable-placement-boundary.json` now opens the Scrollable Select
    docs surface in a constrained viewport, waits for a placed-rect trace, checks window containment,
    bounds listbox size, proves trigger/listbox relations, captures a screenshot, bundle, and layout
    sidecar, and runs inside the promoted `ui-gallery-select` suite. The first drafts found harness
    authoring defects in the start-section filter and `bounds_max_size` width oracle; no runtime
    Select placement defect was confirmed.
- [x] Add an overlay/listbox placement ownership runtime slice for Select or Combobox that
  exercises scroll-container clipping, modal boundary choice, RTL direction, and viewport resize
  with placement/layout sidecar evidence.
  - Progress: the Select item-aligned resize sub-axis is now covered by
    `ui-gallery-select-demo-open-layout.json`; it proves first-open placement/relations and then
    proves resize closes the item-positioned popup and clears `controls`. This intentionally does
    not cover anchored overlays that should stay open.
  - Progress: `ui-gallery-combobox-responsive-resize-open-placement.json` now covers the anchored
    overlay stay-open resize companion: it keeps the Combobox popover mounted across resize, gates
    a fresh post-resize placement trace, allows collision flip, checks top-flip side gap,
    containment/stability, and preserves `controls`/`labelled_by` relation wiring.
  - Progress: the same component-family suite run exposed and fixed an older
    `ui-gallery-combobox-typeahead-commit-banana.json` harness defect: the script clicked an
    existing but offscreen trigger. It now scrolls the trigger into view, asserts window bounds, and
    uses `click_stable`; the full `ui-gallery-combobox` suite passes with 23 scripts.
  - Progress: `ui-gallery-select-scrollable-placement-boundary.json` now covers the Select
    Scrollable listbox baseline with placed-rect trace, window containment, listbox size bounds,
    relation wiring, screenshot, bundle, and layout sidecar evidence.
  - Progress: `ui-gallery-combobox-placement-ownership-scroll-rtl.json` now covers the explicit
    scroll-container clipping plus RTL ownership sub-axis: the trigger sits inside a clipped
    ScrollArea, the popover escapes to the overlay root, the content and option overflow the inner
    viewport, and the overflowed option remains hittable/selectable.
  - Progress: `ui-gallery-dialog-nested-combobox-modal-boundary.json` now covers the modal/root
    boundary ownership sub-axis: a Combobox opened inside a modal Dialog remains selectable while
    the modal/focus barrier is active, records placement and relation evidence, and verifies final
    barrier cleanup.
  - Progress: `anchored_cross_root_coordinate_v1.json` now covers the synthetic cross-root
    coordinate-space sub-axis. It proved core `AnchoredProps` is correct for secondary/embedded
    roots, while the owning-layer fix moved `fret-ui-kit` and shadcn anchored overlay recipes to
    root-boundary placement helpers instead of environment viewport boundaries.
  - Result: `ui-gallery-resizable-multi-viewport-combobox-placement.json` now covers the runtime
    multi-viewport ownership companion. The first run found a mechanism-layer effective
    root-boundary cache defect: Combobox placement used the OS window as the collision boundary and
    chose `bottom`. The owning fix rebuilds per-element nearest-viewport root bounds after final
    layout, and the gate now passes with a `top` flip against the Resizable panel viewport.
  - Result: `element_root_bounds_cache_uses_nearest_nested_viewport_root` now covers nested
    viewport-root precedence for the same cache.
  - Result: `element_root_bounds_cache_rebuilds_when_element_moves_between_viewport_roots` now
    covers same-element movement between viewport roots without stale effective root-boundary cache
    entries.
  - Result: `element_root_bounds_cache_rebuilds_on_view_cache_hit_after_viewport_move` now covers
    the same ownership move through retained render reuse when the cached subtree render function is
    not re-entered.
  - Result: this ownership axis is closed for v1. The runtime matrix now covers Select
    item-aligned resize-close policy, Combobox anchored resize-reposition policy, scroll-container
    clipping plus RTL ownership, modal/root-boundary ownership, synthetic cross-root coordinates,
    and the Resizable multi-viewport runtime companion. It found and fixed the mechanism-layer
    effective root-boundary cache defect where overlay placement used the OS window/owner root
    instead of the source element's nearest layout viewport root. A view-cache movement companion
    remains only as a future follow-on if a real Gallery surface can move cached overlay sources
    across viewport roots.
- [x] Add the Resizable view-cache movement companion for cached Combobox source routing.
  - Result: `ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json` now drives a
    real UI Gallery surface where a Combobox source root is cached, moved from the left Resizable
    panel to the right panel, and opened after the move. The first runtime run exposed a
    mechanism-layer prepaint interaction-cache defect: semantics and paint moved with the cache
    root, but replayed hit-test records kept their old absolute bounds, so the trigger click hit
    the panel container and the script timed out waiting for the input. `InteractionCacheEntry`
    now stores the cache-root origin and replay translates cached interaction records by the
    origin delta. Focused prepaint, prepaint-family, hit-test/view-cache transform, build, and
    runtime diagnostics gates all pass. The promoted `ui-gallery-resizable` suite also passes
    2/2 with both multi-viewport Combobox scripts producing evidence and zero lint warnings.
- [x] Add a diagnostics script lint/registry audit for long-page content clicks that use plain
  `click` without a nearby `scroll_into_view`, `bounds_within_window`, or `click_stable`
  precondition.
  - Result: promoted as a scoped registry authoring gate in `tools/check_diag_scripts_registry.py`
    for the active `ui-gallery-combobox` and `ui-gallery-select` suites. The audit found 495 unsafe
    long-page content-click patterns across the promoted registry and cleared the active Combobox
    and Select families by adding `require_fully_within_window` scrolls, `bounds_within_window`
    guards, and `click_stable` to content-target actions. `python tools/check_diag_scripts_registry.py`
    now fails future Combobox or Select scripts that regress to plain content clicks or unguarded
    `click_stable` on `ui-gallery-combobox-*` or `ui-gallery-select-*` targets. The full
    `ui-gallery-combobox` and `ui-gallery-select` suites pass after rebuilding the diagnostics
    runner.
- [x] Add a diagnostics script lint for page-local UI Gallery selectors that rely on the default
  page instead of proving or navigating to the owning page.
  - Result: `tools/check_diag_scripts_registry.py` now enforces a scoped page-entry rule for
    `ui-gallery-motion-pilot`: scripts that use `ui-gallery-motion-presets-*` page-local selectors
    must first wait for `ui-gallery-page-motion-presets`, while the always-visible shell motion
    preset trigger remains allowlisted. The first strict audit found and fixed an existing
    Motion Presets script debt: `ui-gallery-motion-presets-fluid-tabs-pixels-changed-fixed-frame-delta.json`
    entered the page but did not assert the page root before waiting for a page-local trigger.
    The follow-up candidate audit found promoted `ui-gallery-select` already at 0 page-entry
    violations, so Select is now included in the strict page-entry gate. Combobox initially showed
    36 violations under a page-root-only rule, but those scripts use explicit
    `FRET_UI_GALLERY_START_PAGE=combobox` defaults; the lint now treats those defaults as valid
    entry evidence and Combobox is strict too. DataTable still has 166 promoted-suite page-entry
    violations and remains a deliberate follow-on cleanup lane. `tools/test_check_diag_scripts_registry.py`
    locks the lint with bad-script, good-script, shell-trigger, Select page-entry, and Combobox
    start-page cases.
- [x] Continue `ui-gallery-motion-pilot` until it finds a real component/mechanism or harness
  boundary issue beyond the existing Motion Presets gates.
  - Result: the suite found and fixed `AlertAction` internal slot marker pollution by moving slot
    classification from exported diagnostics `test_id`s to `AnyElement::component_slot(...)`.
    It then found Drawer snap-point scripts that confused existence with hittability; those scripts
    now scroll long-page triggers into view and assert window bounds before `click_stable`, and the
    spring-retarget gate now verifies the actual dismiss/focus-restore contract. The next Sidebar
    gate exposed a tooling timeout evidence gap and an unguarded offscreen long-page trigger; both
    are now covered by the timeout-bundle regression and strict Sidebar visibility lint.
- [x] Fix diagnostics timeout handling so a long-running intent step leaves a forced bundle.
  - Result: `fret-diag` now preserves the prior running script result on external
    `timeout.tooling.script_result`, carries through `run_id`/`step_index`, records the last bundle
    metadata, and writes a run-id artifact alias. The focused Sidebar repro then produced enough
    evidence to classify the failure as an offscreen long-page target rather than a Sidebar recipe
    or hit-test mechanism defect.
- [x] Add Sidebar to the strict long-page click visibility authoring gate.
  - Result: `ui-gallery-sidebar-toggle-fixed-frame-delta.json` now scrolls
    `ui-gallery-sidebar-demo-toggle` into `ui-gallery-content-scroll`, asserts
    `bounds_within_window`, and only then uses `click_stable`. `tools/check_diag_scripts_registry.py`
    now treats `ui-gallery-motion-pilot` as a strict click-visibility suite for
    `ui-gallery-sidebar-*` targets, with a registry lint unit test covering the bad pattern.
- [x] Migrate non-user-facing recipe structural marker `test_id`s to `component_slot`.
  - Result: no fixed `__fret_shadcn.*` markers remain. CardAction, CardFooter, and AvatarBadge no
    longer use generated diagnostics `test_id`s for recipe-internal child classification. The same
    audit found a related mechanism-boundary issue in Item: ItemMedia and ItemDescription used
    `key_context` for internal classification, so they now use `component_slot` too. Focused tests
    prove the slots still drive layout/classification while staying out of diagnostics selectors
    and shortcut key contexts.
- [x] Add a lightweight source-hygiene gate for future recipe-internal marker misuse.
  - Result: `tools/check_shadcn_internal_slots.py` enforces that `fret-ui-shadcn.*` internal marker
    strings are declared as `*_SLOT` constants and are not passed to diagnostics `test_id`,
    `attach_test_id`, or shortcut `key_context`. `tools/test_check_shadcn_internal_slots.py`
    covers the allowed `component_slot` path plus bad constant naming, bad test-id use, and bad
    key-context use.
- [x] Continue `ui-gallery-motion-pilot` to find the next runtime-visible harness or component
  boundary issue.
  - Progress: the next Sonner gate found a real overlay policy defect. A local named Sonner
    Toaster and the shell's unnamed Toaster both rendered the same named toast, creating duplicate
    `toast-entry-1`/`toast-entry-2` semantics `test_id`s. `fret-ui-kit` now scopes unnamed toast
    layers to unnamed toasts and named layers to matching `toaster_id`s; the focused Sonner runtime
    gate passes and `diag test-ids` reports zero duplicates. The full `ui-gallery-motion-pilot`
    suite now passes 14/14 after the fix, with only non-blocking lint warnings remaining.
  - Progress: the residual Sonner `semantics.missing_label` warning was also a real policy defect,
    not lint noise. Toast action/cancel buttons rendered visible text but did not export that text
    as the button accessible name. `fret-ui-kit` now labels those pressables from `ToastAction`, a
    focused semantics snapshot test covers action/cancel labels, the Sonner focused gate lint is
    clean, and the full `ui-gallery-motion-pilot` suite still passes 14/14.
  - Progress: the three Carousel `semantics.missing_label` warnings all pointed at the same
    UI Gallery nested demo button. The demo intentionally created `Button::new("")` for a small
    inner control but forgot `.a11y_label(...)`; this was a first-party fixture/demo quality gap,
    not a Button mechanism defect. The focused Carousel gates now pass and lint clean after adding
    the accessible name.
  - Result: the remaining Motion Presets `layout.zero_size` warning exposed a shadcn Tabs recipe
    layout bug. The shared indicator's non-hit-test wrapper and absolute canvas both relied on
    CSS-like inset fill behavior without explicit Fill sizing, so the diagnostics node could be
    0px wide. Tabs now sizes the shared indicator gate and canvas explicitly, focused unit/runtime
    gates pass, and the full `ui-gallery-motion-pilot` suite is 14/14 with zero lint warnings.
- [x] Pick the next harness slice outside the now-clean `ui-gallery-motion-pilot` suite.
  - Result: moved back to the synthetic mechanism layer and expanded the Layout Primitives fixture
    suite across percent sizing, min/max constraints, percent min/max intrinsic measurement,
    wrapped text height propagation, scroll-root child layout bounds, and absolute percent inset
    placement. No new mechanism defect was reproduced; the slice closed a harness coverage gap.
- [x] Add the next layout primitive companion for text measurement/paint agreement or
  RTL/writing-mode behavior once the mechanism owner and public direction model are clear.
  - Result: added text measurement/paint agreement cases for column wrap width, max-width row wrap
    width, and overflow/scale constraints. No new mechanism defect was reproduced; the harness now
    records text measure/prepare constraint deltas and painted-height layout follow-through as
    scalar fixture metrics.
- [x] Promote declarative `ViewCache` lifecycle guarantees into a fixture-driven mechanism harness.
  - Result: `view_cache_lifecycle_v1.json` now covers clean cache-hit reuse, retained element
    state, cache-key misses, RAF invalidation, model-observation preservation across cache-hit
    frames, unrelated model scoping, inspection-mode cache bypass, and layout-query next-frame
    invalidation. No new mechanism defect was reproduced; the slice closed a harness coverage gap.
- [ ] Add RTL/writing-mode layout primitive cases once the direction/writing-mode contract is
  explicit enough to avoid encoding a recipe policy as a mechanism oracle.
- [x] Add a runtime UI Gallery companion for cached model/layout-query dependency mutation only if
  a real surface exposes a non-synthetic risk; otherwise continue with scroll/click stability or
  retained/component semantics mutation.
  - Result: closed by the later ViewCache runtime companion slice recorded below and in F148. The
    real surface was the UI Gallery ViewCache harness page; the focused `ui-gallery-view-cache`
    suite now proves cached-subtree counter mutation and Popover state through `/view_cache`
    app-snapshot payloads. The slice found a shadcn Textarea resize-grip semantics defect rather
    than a ViewCache mechanism defect.
- [x] Promote the ScrollArea content-growth runtime gate if the focused run proves it is stable.
  - Result: `ui-gallery-scroll-area-expand-at-bottom.json` passed as a focused runtime gate and is
    now part of the promoted `ui-gallery-scroll-area` suite. No new mechanism defect was
    reproduced; the slice fixed a harness promotion gap where dynamic scroll extent growth was
    checked only by a standalone script.
- [x] Turn the promoted DataTable page-entry authoring debt into a strict registry lint gate.
  - Result: the first candidate audit reported 174 violations across 21 promoted scripts under a
    single-root model. The actual harness gap was rule expressiveness: DataTable has multiple valid
    page/variant roots. `check_diag_scripts_registry.py` now supports `entry_ids`, strict
    page-entry is enabled for the promoted DataTable suites, and the registry/self-test gates pass.
- [x] Extend strict click-visibility authoring coverage to the promoted ScrollArea suite.
  - Result: five long-page content clicks were converted to guarded `click_stable` steps and
    `ui-gallery-scroll-area` is now enforced by the registry lint. Running the full ScrollArea
    suite exposed and fixed a diagnostics harness defect where current-state debug predicates could
    match stale ring snapshots, plus a multi-pointer script authoring issue where capture-state
    assertions needed `wait_until` convergence after pointer events.
- [x] Add a promoted-script lint for pointer-event steps that immediately assert current pointer
  state instead of waiting for debug-snapshot convergence.
  - Result: the candidate audit found 6 adjacent pointer/current-state patterns in promoted
    scripts. Three were already bounded `wait_until` checks; three remained immediate `assert`
    steps in ScrollArea scrollbar-drag scripts. Those scripts now use bounded `wait_until`, the
    registry lint rejects future promoted regressions, both focused ScrollArea runtime gates pass,
    and the full `ui-gallery-scroll-area` suite passes.
- [ ] If ScrollArea "Arm content growth" click intermittency recurs, add a focused diagnostics
  stability slice that proves whether the miss is click synthesis, command dispatch, or state
  publication.
- [x] Promote timer-target visibility semantics into a fixture-driven mechanism matrix.
  - Result: `timer_dispatch_v1.json` now covers visible base targets, visible but hit-test-inert
    overlay targets, hidden overlay targets, and removed overlay targets. No new mechanism defect
    was reproduced after F138; the slice turns the Select-discovered hidden-layer timer defect into
    durable, case-addressable harness coverage.
- [x] Promote the Command suite into strict diagnostics authoring coverage.
  - Result: strict page-entry and long-page click-visibility lint now cover promoted
    `ui-gallery-command` scripts. The candidate audit found 4 page-entry violations, 20 plain
    long-page content clicks, and 6 missing target-level visibility proofs. These were harness
    authoring defects, not Command component or `fret-ui` mechanism defects: the full Command suite
    passes 18/18 after replacing content clicks with guarded `click_stable`, adding missing
    visibility guards, and adding the missing Command page-root proof.
- [x] Continue the shadcn runtime evidence suite until it finds a real semantics or diagnostics
  surface defect.
  - Result: `ui-gallery-shadcn-runtime-evidence` reached the DataTable pagination gate and exposed
    real semantics lint defects after the runtime assertions passed: duplicate DataTable toolbar
    input `test_id`s across same-page examples and missing accessible names on table header sort
    button action owners. `DataTableToolbar::test_id_prefix(...)` now scopes recipe-owned child ids
    for multi-instance pages, UI Gallery DataTable/torture surfaces use scoped ids, retained and
    view-cache scripts target the scoped torture toolbar ids, and virtualized table header
    pressables now export labels in retained and non-retained paths. The suite passes 10/10 after
    rebuilding `fret-ui-gallery`.
- [x] Add a promoted-script fixed-frame-clock contract for time-sensitive diagnostics scripts.
  - Result: the HoverCard `trigger-delays` failure was a harness timing false positive. Promoted
    scripts whose names declare `fixed-frame-delta` or `trigger-delays` now carry
    `FRET_DIAG_FIXED_FRAME_DELTA_MS=16`, and the registry lint rejects future promoted scripts that
    forget that clock contract. The focused HoverCard delay gate passes under the fixed frame clock.
- [x] Repair the HoverCard `sides-placement` leave-target authoring issue and rerun the
  `ui-gallery-hover-card` suite.
  - Result: after the fixed-frame-clock repair, the HoverCard suite advances past
    `trigger-delays`; `ui-gallery-hover-card-sides-placement.json` now fails at step 20 with
    `move_pointer_no_semantics_match` because it uses `ui-gallery-status-last-action` as a leave
    target and that node is not present on the HoverCard page snapshot.
    The script now uses the stable `ui-gallery-nav-search` leave target and asserts the bottom
    case as an intentional collision flip (`preferred_side=bottom`, `chosen_side=top`,
    `flipped=true`) because the current page geometry leaves only ~43px below the trigger. The
    full `ui-gallery-hover-card` suite passes 6/6.
- [x] Promote the HoverCard suite into strict diagnostics authoring lint.
  - Result: `ui-gallery-hover-card` now participates in both strict page-entry and long-page
    click-visibility registry checks. A dry run found zero existing violations after the earlier
    fixed-frame-clock and sides-placement repairs, so this slice added the reusable lint coverage
    and focused self-tests rather than changing scripts. The full HoverCard suite still passes 6/6
    with `scripts_with_evidence=6`, `focus_mismatch_total=0`, and zero lint errors/warnings.
- [x] Promote the Menubar Placement suite into strict diagnostics authoring lint.
  - Result: `ui-gallery-menubar-placement` now participates in both strict page-entry and
    long-page click-visibility registry checks. A dry run found zero existing violations because
    the three placement scripts already set `FRET_UI_GALLERY_START_PAGE=menubar`, assert
    `ui-gallery-page-menubar`, and guard content clicks with `scroll_into_view` window
    containment. Registry self-tests now run 27 tests, and the strict runtime suite still passes
    3/3 with zero lint errors/warnings.
  - Follow-up completed below: the DropdownMenu strict promotion adds those visibility guards and
    locks the suite with registry and runtime evidence.
- [x] Promote the DropdownMenu suite into strict diagnostics authoring lint.
  - Result: `ui-gallery-dropdown-menu` now participates in both strict page-entry and long-page
    click-visibility registry checks. The strict pass fixed the two remaining authoring gaps:
    `ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json` now scrolls the demo
    trigger fully into the window before `click_stable`, and
    `ui-gallery-dropdown-menu-submenu-open-smoke.json` now requires window containment for the
    submenu trigger scroll step. Registry self-tests now run 30 tests, and the strict runtime
    suite passes 3/3 with zero lint errors/warnings.
  - Observation: the first strict suite run hit a `timeout.no_frames` stall in the Basic
    typeahead script after resize. The same script passed when rerun alone, and the full suite
    passed on rerun, so this was recorded as a harness/run stability observation rather than a
    DropdownMenu recipe or mechanism defect.
- [x] Promote the ContextMenu suite into strict diagnostics authoring lint.
  - Result: `ui-gallery-context-menu` now participates in both strict page-entry and long-page
    click-visibility registry checks. The dry run found zero current violations because both
    corridor scripts already provide an explicit `FRET_UI_GALLERY_START_PAGE=context_menu`
    default, assert the ContextMenu page root, and guard the submenu trigger before
    `click_stable`. Registry self-tests now run 33 tests, and the strict runtime suite passes 2/2
    with `scripts_with_evidence=2`, `focus_mismatch_total=0`, and zero lint errors/warnings.
- [x] Promote the Button Group suite into strict diagnostics authoring lint.
  - Result: `ui-gallery-button-group` now participates in both strict page-entry and long-page
    click-visibility registry checks. The dry run found three authoring violations in the Demo,
    Accessibility, and Select screenshot scripts: each clicked the Code tab directly after the
    Preview capture. The first strict runtime attempt then proved the Select Code tab was present
    but off-window, so the script now scrolls the Select content fully into the Gallery content
    viewport before the Preview and Code captures. Registry self-tests now run 36 tests, the
    focused Select script passes, and the strict runtime suite passes 13/13 with zero lint
    errors/warnings.
- [x] Promote Menubar submenu placement traces into a focused runtime suite.
  - Result: `ui-gallery-menubar-placement` now gates the existing LTR submenu, RTL wide submenu,
    and RTL tight-left collision scripts as a small durable placement suite. The suite passed 3/3
    with zero lint errors/warnings; no new Menubar or overlay mechanism defect was reproduced.
- [x] Promote DropdownMenu runtime evidence into a focused suite.
  - Result: `ui-gallery-dropdown-menu` now gates submenu placement, basic keyboard typeahead
    commit, and disabled-but-focusable keyboard suppression as a small durable suite. The first
    focused runs found harness defects rather than recipe defects: the typeahead script allowed a
    trigger to remain at the window edge before `click_stable`, and it used an optional status-bar
    semantics node as its result oracle. The suite now passes 3/3 with zero lint errors/warnings.
- [x] Promote ContextMenu submenu corridor evidence into a focused suite.
  - Result: `ui-gallery-context-menu` now gates both safe-corridor sweep and branch/corridor
    routing as a small independently runnable pointer-policy suite. This did not reproduce a new
    ContextMenu or hit-test mechanism defect; it closed a harness packaging gap where the scripts
    existed but were only reachable through broad conformance/overlay suites. The suite passes 2/2
    with zero lint errors/warnings.
- [x] Add a ViewCache runtime companion that proves cached-subtree model mutation through UI
  Gallery app-snapshot state instead of visible text proxies.
  - Result: `ui-gallery-view-cache` now gates counter mutation and Popover open/close state through
    the dedicated `/view_cache` snapshot payload while the page runs with nested ViewCache enabled.
    The first strict suite run found a real shadcn Textarea semantics defect: its pointer-only
    resize grip was exposed as an unlabeled visible Button. The recipe now hides that grip from the
    visible accessibility tree and removes it from Tab traversal. The suite passes with zero lint
    errors/warnings; no ViewCache mechanism defect was reproduced.
- [x] Promote the Button Group suite to strict zero-warning diagnostics lint after a clean
  candidate run.
  - Result: `ui-gallery-button-group` passed 13/13 with `focus_mismatch_total=0` and zero lint
    errors/warnings. No new Button Group component or mechanism defect was reproduced in this
    candidate run; the suite now rejects future diagnostics lint drift.
- [x] Convert the Button Group size icon-only Add row from screenshot-only evidence to geometry
  evidence.
  - Result: `ui-gallery-button-group-size-screenshots-zinc-light-dark.json` now asserts each Add
    icon's bounds and center alignment against its enclosing button for small, medium, and large
    variants. The first pass used the wrong icon selector assumption, then the real
    `*-add-icon` ids were aligned and the geometry assertions passed.
- [x] Promote the Carousel embla-engine runtime sub-suite to strict zero-warning diagnostics lint.
  - Result: the wide `ui-gallery-carousel-docs-parity` suite is too large to be a reliable single
    evidence unit under normal outer timeouts, but the focused `ui-gallery-carousel-embla-engine`
    sub-suite passed 5/5 with `focus_mismatch_total=0` and zero lint errors/warnings. No new
    Carousel mechanism or recipe defect was reproduced; the embla-engine suite now rejects future
    diagnostics lint drift for inertia, touch, resize reInit, and loop continuity/downgrade paths.
- [x] Harden the Date Picker suite after strict lint exposed scroll/script precondition gaps.
  - Result: the mobile Drawer script now uses a mobile-branch width that the Gallery content
    viewport can actually contain, the range-roving script is independently runnable with its own
    env defaults and scroll/click-stable preconditions, and `scroll_into_view` now classifies
    impossible unscrollable-axis containment instead of falling through to a generic stuck result.
    No Date Picker component or layout mechanism defect was reproduced in this slice.
- [x] Split a compact Combobox geometry/placement suite out of the broad Combobox family gate.
  - Result: the broad `ui-gallery-combobox` suite exceeded the outer command timeout after running
    many clean rows, so it is too wide to be the day-to-day geometry/placement evidence unit. The
    new `ui-gallery-combobox-geometry-placement` suite gates trigger chrome, LTR/RTL long-text
    truncation/chevron geometry, popup top/bottom placement, and responsive open/resize placement
    as a 7/7 zero-warning suite. No new Combobox recipe or overlay mechanism defect was
    reproduced.
- [x] Add flex visual-order layout/measurement consistency coverage.
  - Result: the layout primitive fixture exposed a real mechanism defect where
    `FlexItemStyle.order` affected final layout but not intrinsic flex measurement. `fret-ui` now
    shares the same visual-order child helper between layout and measurement, and the fixture locks
    both final child positions and wrap-sensitive measured width.
- [x] Add auto-container child margin measure/layout consistency coverage.
  - Result: `measure_impl` now includes child margins when it computes max-content sizing for
    auto-sized container children, so `measure` and laid-out bounds agree for auto containers with
    finite margins. The new layout primitive fixture locks both the layout bounds and the measured
    max-content size for a margin-bearing child.
- [x] Add HoverRegion absolute-child measure/layout envelope coverage.
  - Result: the layout primitive fixture exposed a real mechanism defect where HoverRegion's
    layout contract needed to include absolute-positioned children in its hover/hit-test envelope,
    but intrinsic measurement still used the generic passthrough path and collapsed to `0 x 0`.
    `measure_impl` now gives HoverRegion a dedicated absolute-child envelope path, and the fixture
    locks final bounds, measured max-content size, and center hit-testing for a left/top inset
    child.
- [x] Add HoverRegion fractional-inset absolute-child envelope coverage.
  - Result: the follow-on layout primitive fixture exposed a second HoverRegion mechanism defect:
    fractional left/top insets were treated as zero during shrink-wrap envelope sizing, so the
    wrapper stayed at the child size while final placement pushed the child beyond the wrapper's
    hover/hit-test bounds. `fret-ui` now shares a conservative absolute-child envelope helper
    between HoverRegion layout and measurement, including fractional inset solving, and the fixture
    locks final bounds, measured max-content size, child placement, and near-edge hit-testing.
- [x] Add HoverRegion right/bottom inset absolute-child envelope coverage.
  - Result: real surfaces use this path for scrollbar chrome overlays
    (`ecosystem/fret-ui-shadcn/src/scroll_area.rs` and
    `ecosystem/fret-code-view/src/code_block.rs`). The new layout primitive fixture locks a
    right/bottom inset absolute child against HoverRegion final bounds, measured max-content size,
    child placement, and near-edge hit-testing. It passed with the shared envelope helper, so no
    additional mechanism defect was reproduced.
- [x] Promote the retained Table torture surface into a durable runtime diagnostics suite.
  - Result: `ui-gallery-table-retained` now passes 7/7 across keyboard typeahead, multi-sort,
    row-pinning with `keep_pinned_rows` true/false, descending sort, sort/select/scroll, and
    window-boundary scroll. The slice found and fixed a real retained Table pagination /
    `keep_pinned_rows` defect, plus diagnostics harness defects in no-frame `wait_frames` handling
    and aggregate debug-history predicate freshness.
- [x] Promote AI FileTree semantics/action-state runtime coverage and fix auto-height VirtualList
  measured-leaf dirtying.
  - Result: `ui-gallery-ai-file-tree` now passes 4/4 across toggle, actions, large-scroll, and
    screenshot evidence. The runtime suite found a real `fret-ui` mechanism defect where
    `VirtualList` len/items-revision changes updated rows and semantics but did not dirty the Taffy
    measured leaf, so parent layout reused a stale auto-height and following doc sections overlapped
    expanded FileTree rows. VirtualList layout-affecting prop diffs now mark layout dirty, measured
    leaves can be marked dirty through `LayoutEngine`, and the focused regression locks sibling
    reflow after auto-height list growth.
- [x] Promote AI FileTree to strict zero-warning diagnostics lint.
  - Result: demo-only selection/action state markers are now hidden semantics anchors, scripts
    assert them with `raw_semantics_hidden_is`, `fret-diag` lint ignores non-focused hidden nodes for
    visible-bounds/missing-label warnings, and the `ui-gallery-ai-file-tree` suite now enforces
    `lint_policy.max_warning_issues=0`. The fresh strict suite passes 4/4 with zero lint errors and
    zero lint warnings.
- [x] Add grid column/row gap layout and max-content measurement coverage.
  - Result: `layout_primitives_v1.json` now locks a 2x2 auto grid with independent column and row
    gaps. The fixture proves final cell positions and `measure_in(MaxContent)` agree on the
    `58 x 28` grid envelope. The first run found a fixture oracle math mistake rather than a
    mechanism defect; the corrected layout primitive harness passes.
- [x] Add flex visual-order plus auto-margin trailing-group coverage.
  - Result: `flex-order-auto-margin-uses-visual-order` exposed a real `fret-ui` mechanism defect:
    flex layout and measurement used visual order, but the auto-margin post-processing in
    `layout_flex_impl_engine` still scanned source-order children. The flex layout path now uses
    `ordered_flex_children` for tail detection, tail sizing, gap preservation, shift application,
    and final layout iteration. The layout primitive harness and `cargo fmt -p fret-ui --check`
    both pass.
- [x] Add the right-side auto-margin RTL recipe analogue for flex visual-order coverage.
  - Result: `flex-order-margin-right-auto-uses-visual-order` covers the `mr-auto` variant used by
    RTL recipe helpers and Gallery surfaces. The fixture passed without a new runtime change,
    proving the current flex engine plus F166's visual-order post-processing keeps right-side
    auto-margin behavior aligned with ordered children.
- [x] Add the vertical `mt-auto` recipe analogue for flex visual-order coverage.
  - Result: `flex-order-margin-top-auto-uses-visual-order` covers the vertical trailing-group path
    used by Sheet/Drawer-style footer placement. The fixture passed without a new runtime change,
    proving the F166 visual-order post-processing path covers ordered vertical flex columns too.
- [x] Add flex gap layout and max-content measurement coverage.
  - Result: `flex-gap-measure-matches-layout` locks child placement and `measure_in(MaxContent)`
    for a horizontal flex row with `gap=8`. The fixture passed without a runtime change, proving
    flex final layout and intrinsic measurement currently agree on the `58 x 12` gap envelope.
- [x] Add flex wrap gap layout and definite-width max-content measurement coverage.
  - Result: `flex-wrap-gap-measure-matches-layout` locks child placement and
    `measure_in(MaxContent)` for a 68px-wide wrapping row with `gap=8`. The fixture passed without
    a runtime change, proving final layout and intrinsic measurement agree on the `68 x 34`
    wrapped gap envelope.
- [x] Add Pressable absolute-only wrapper envelope layout/measurement coverage.
  - Result: `pressable-fractional-absolute-child-envelope-matches-layout` exposed a real `fret-ui`
    mechanism defect: an auto/auto passthrough wrapper with only absolute children was solved by
    the flow engine as `0 x 0`, even though widget layout and hit-testing could place the child.
    The flow engine now treats absolute-only auto wrappers as measured leaves, and passthrough
    measurement contributes the shared absolute-child envelope during final definite probes. The
    fixture locks wrapper bounds, child placement, placeholder measurement, and near-edge
    hit-testing for a fractional inset child.
- [x] Add Pressable mixed flow/absolute wrapper envelope layout/measurement coverage.
  - Result: `pressable-mixed-flow-absolute-child-envelope-matches-layout` exposed the mixed
    companion defect: an auto/auto passthrough wrapper with a normal flow child and a fractional
    absolute child sized itself to the flow child's `20 x 10` envelope while the absolute child
    required `34 x 12`. The fix extends the shared absolute-child envelope path to mixed wrappers,
    uses measured-leaf flow sizing for auto wrappers with absolute children, and keeps flow child
    placement at its measured size while absolute placement uses the union envelope. The fixture
    locks wrapper bounds, flow child bounds, absolute child bounds, placeholder measurement, and
    near-edge hit-testing.
- [x] Add RenderTransform mixed flow/absolute wrapper visual/hit coverage.
  - Result: `render-transform-mixed-flow-absolute-envelope-matches-visual-hit` moves the mixed
    Pressable flow/absolute envelope through a `RenderTransform`. The fixture locks the wrapper
    layout envelope, flow child bounds, absolute child bounds, placeholder measurement, visual/hit
    translation, and both layout-space miss plus visual-space near-edge absolute-child hit. No new
    mechanism defect was reproduced; the F171/F172 absolute-envelope fixes already carry through
    transform visual and hit spaces.
- [x] Add ViewCache clean-reuse movement coverage for a mixed flow/absolute wrapper.
  - Result: `view_cache_hit_moving_mixed_absolute_wrapper_updates_bounds_and_hit_test` moves a
    cached Pressable mixed flow/absolute subtree by inserting a parent spacer while the ViewCache
    child render closure stays clean. The focused test locks the moved wrapper bounds, moved
    element visual bounds, moved absolute-child bounds, fallback hit-testing, and runtime routing
    via `debug_hit_test_routing`. No new mechanism defect was reproduced; the first red assertion
    was a harness-oracle issue because the old point legitimately hit the expanded outer row, not
    the moved absolute child.
- [x] Add FractionalRenderTransform layout/visual/hit contract coverage.
  - Result: `fractional-render-transform-derives-visual-hit-from-layout-size` now locks
    size-derived render-transform translation in the layout primitive fixture. A `20 x 20`
    Pressable wrapped with `FractionalRenderTransform(2.0, 0.5)` keeps layout bounds at
    `0,0 20 x 20` while visual and hit spaces move by `40 x 10`; the layout-space center misses
    the target and the translated visual-space center hits it. No new mechanism defect was
    reproduced.
- [x] Add MaskLayer paint-only hit-test contract coverage.
  - Result: `hit_test_routing_v1.json` now covers `MaskLayer` bounds versus hit-testing in both
    default paint-only and explicit `Overflow::Clip` modes. An escaped child remains targetable
    outside the mask bounds by default, while the same child is suppressed when the wrapper opts
    into overflow clipping. No new mechanism defect was reproduced; the first red run exposed a
    fixture-oracle mistake that tried to prove escaped hit-testing with a width-overflow child
    whose layout was legitimately constrained to the wrapper width.
- [x] Add EffectLayer computation-bound hit-test contract coverage.
  - Result: `hit_test_routing_v1.json` now covers `EffectLayer` bounds versus hit-testing in both
    default computation-bound and explicit `Overflow::Clip` modes. An escaped child remains
    targetable outside the effect bounds by default, while the same child is suppressed when the
    wrapper opts into overflow clipping. No new mechanism defect was reproduced.
- [x] Add CompositeGroup computation-bound hit-test contract coverage.
  - Result: `hit_test_routing_v1.json` now covers `CompositeGroup` bounds versus hit-testing in
    both default computation-bound and explicit `Overflow::Clip` modes. An escaped child remains
    targetable outside the compositing group bounds by default, while the same child is suppressed
    when the wrapper opts into overflow clipping. No new mechanism defect was reproduced.
- [x] Add BackdropSourceGroup computation-bound hit-test contract coverage.
  - Result: `hit_test_routing_v1.json` now covers `BackdropSourceGroup` bounds versus hit-testing
    in both default computation-bound and explicit `Overflow::Clip` modes. An escaped child remains
    targetable outside the backdrop source group bounds by default, while the same child is
    suppressed when the wrapper opts into overflow clipping. No new mechanism defect was
    reproduced.
- [x] Add renderer-level font trace predicates to the Combobox long-text gate.
  - Result: diagnostics could already capture renderer font trace bundles, but scripts could not
    assert the text-preparation facts directly. The new
    `render_text_font_trace_entries_matching_ge` predicate lets the LTR Combobox long-text gate
    prove the selected label is prepared with `font=ui`, `wrap=none`, `overflow=ellipsis`, and
    `missing_glyphs=0`. The first runtime drafts found a static-page `wait_frames` stall in the
    script, not a Combobox or renderer defect; the corrected gate uses semantic convergence waits.
- [x] Add the RTL Combobox renderer font-trace companion.
  - Result: `ui-gallery-combobox-rtl-long-text-geometry.json` now uses the same renderer trace
    predicate as the LTR gate, proving the selected RTL long label is prepared with `font=ui`,
    `wrap=none`, `overflow=ellipsis`, and `missing_glyphs=0` while keeping the existing RTL
    chevron/checkmark geometry and top-flip placement oracles. No new Combobox or renderer defect
    was reproduced.
- [x] Add the Command docs-demo long-query renderer font-trace companion.
  - Result: `ui-gallery-command-docs-demo-long-query-text.json` now enables renderer font tracing
    and proves the long search query is prepared with `font=ui`, `wrap=none`, `overflow=clip`, and
    `missing_glyphs=0`, while retaining TextInput viewport, offset, IME cursor, layout sidecar, and
    screenshot evidence. No Command recipe or renderer defect was reproduced; the first runtime
    drafts only exposed that this runner reports effective window heights 20px taller than the
    requested inner heights, matching the existing requested/effective viewport diagnostics
    contract.
- [x] Add the plain Input/File renderer font-trace companion.
  - Result: `ui-gallery-input-basic-and-file-long-text.json` now starts directly on the Input page,
    removes navigation/static-frame waits, and proves both the Basic Input value and file-composed
    Input value are renderer-prepared with `font=ui`, `wrap=none`, `overflow=clip`, and
    `missing_glyphs=0`. No Input recipe or renderer defect was reproduced.
- [x] Add the Button Group Input Group renderer font-trace companion.
  - Result: `ui-gallery-button-group-input-group-long-text.json` now enables renderer tracing,
    removes static-frame waits, waits on effective-window/page/root predicates, and proves the
    long grouped-input value is renderer-prepared with `font=ui`, `wrap=none`, `overflow=clip`,
    and `missing_glyphs=0`. No Button Group recipe or renderer defect was reproduced; the first
    runtime draft exposed an over-eager script precondition on the direct control's pre-value
    `0 x 0` semantics bounds, so the precondition now checks the owning group root instead.
- [x] Add relative inset final-position and flow-sibling layout primitive coverage.
  - Result: `relative-inset-offsets-final-position-without-affecting-flow-siblings` now locks the
    `PositionStyle::Relative` contract from ADR 0062 and `element.rs`: inset offsets move the
    target's final layout and hit-test position, but its sibling keeps the original flow slot. The
    fixture proves the old flow-slot center misses the moved Pressable while the final-position
    center hits it. No new `fret-ui` mechanism defect was reproduced.
- [x] Add static inset ignore coverage for default flow positioning.
  - Result: `static-inset-ignored-by-default-flow-position` now locks the opposite
    `PositionStyle::Static` contract from `element.rs`: inset offsets are ignored unless the
    element opts into a positioned mode. The fixture proves a static Pressable with `top: 12px`
    remains at the original flow slot, keeps the following sibling at `x=20`, hits at the flow-slot
    center, and does not hit at the hypothetical offset center. No new `fret-ui` mechanism defect
    was reproduced.
