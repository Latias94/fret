# Fret Mechanism Harness v2

Fret Mechanism Harness v2 is the shared runtime workbench for self-drawn UI mechanisms. It is not a
component test suite and it is not shadcn-specific. Its job is to make a mechanism controllable,
observable, and assertable before a recipe layer depends on it.

## Purpose

The harness standardizes four artifacts:

- **Scenario**: a stable case id plus input knobs owned by the consuming test or diagnostic surface.
- **Observation**: a portable runtime snapshot of nodes, semantics selectors, layout bounds, visual
  bounds, hit-test samples, overlay records, and focus records.
- **Oracle**: reusable predicates over those observations. The harness can evaluate
  `UiPredicateV1` geometry predicates directly, and adds mechanism-only predicates for bounds
  spaces and hit-test samples.
- **Runner**: a thin fixture runner that parses JSON, runs each case, and reports failures by case id
  and predicate index.

This is the split that keeps `crates/fret-ui` as the mechanism owner while letting
`ecosystem/fret-ui-shadcn`, future Material 3 recipes, and UI Gallery diagnostics reuse the same
oracle vocabulary.

## Ownership

- `crates/fret-mechanism-harness`: shared fixture schema, observed tree/query API, oracle evaluator,
  and case runner. It depends only on `fret-core`, `fret-diag-protocol`, and serialization support.
- `crates/fret-ui`: mechanism scenarios and adapters from `UiTree` / semantics snapshots into the
  observed tree.
- `ecosystem/fret-ui-shadcn`: recipe scenarios that prove shadcn composition still satisfies the
  mechanism oracle.
- `tools/diag-scripts`: portable UI Gallery repros that reuse `UiPredicateV1` predicates, including
  `bounds_metric_delta` for axis-specific geometry gates.

## Phase 1 Coverage

The first fixture suite is
`crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json` and is run by
`mechanism_harness_layout_primitives_match_oracles`.

It covers:

- auto sizing that shrink-wraps text metrics,
- fill sizing under definite containing blocks,
- flex cross-axis stretch,
- transparent semantics wrappers that preserve fill geometry,
- chrome containers whose outer stretched box stays distinct from inner content,
- grid `fr` + `auto` track negotiation,
- render-transform visual bounds and hit-test samples.

The first recipe consumer is
`ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json`, run by
`mechanism_harness_recipe_layout_cases_match_oracles`. It locks the ButtonGroupText center-y
alignment case that originally exposed the layout/chrome mechanism risk in UI Gallery.

## Phase 2.1 Hit-Test Routing Coverage

The second `fret-ui` fixture suite is
`crates/fret-ui/src/declarative/tests/fixtures/hit_test_routing_v1.json` and is run by
`mechanism_harness_hit_test_routing_matches_oracles`.

It covers:

- visual transforms: visual bounds may move while layout and hit-test bounds stay put,
- render transforms: visual and hit-test bounds move together while layout bounds stay put,
- overflow clipping: escaped child layout cannot receive pointer hits outside the clipping parent,
- transparent wrappers: non-hit-testable mechanism wrappers preserve child hit routing,
- non-hit-testable gates: disabled hit-test subtrees keep layout but stop child hit routing,
- overlay roots: topmost overlay roots win over overlapping underlay roots,
- modal barrier roots: hit-test-inert modal roots still scope input and suppress underlay hits.

`ObservedHitTestSample` records the hit node, active layer roots, and barrier root for each sample.
The mechanism-only predicates `hit_test_sample_barrier_root` and
`hit_test_sample_active_layer_root_at` keep overlay/modal routing assertions in the same fixture
oracle instead of scattering them across bespoke tests.

## Phase 2.2 Shell Sizing Coverage

The shadcn recipe fixture suite now includes two shell sizing cases, both run by
`mechanism_harness_recipe_layout_cases_match_oracles`:

- `responsive-drawer-bottom-sheet-uses-eighty-vh`
- `popover-command-shell-wraps-hover-region-max-height`

These cases came from the shadcn parity discovery harness:

- The responsive combobox mobile report showed the command/listbox subtree passing while the outer
  `DrawerContent` shell was too short. The mechanism harness case locks the source-backed rule that
  top/bottom drawer content uses `max-h-[80vh]` without an additional viewport edge-gap clamp.
- The responsive combobox desktop report showed the command/listbox subtree passing while the outer
  `PopoverContent` shell stayed at the placement fallback height. The promoted case opens the
  popover across two frames so the anchor bounds exist, then asserts the shell wraps the command
  height instead of staying at `160px`.

The popover root cause is mechanism-shaped: overlay placement must read size hints from wrapper
elements that are common in self-drawn UI trees, not just from plain containers. In this slice,
`size_hint_px(...)` had to include `HoverRegion` and `Stack` layout constraints, and the content
child had to be built before the Radix dialog wrapper so the opening frame can read the hint.

Use this pattern for future shell-sizing cases:

- keep the parity discovery report as the source-to-evidence triage surface;
- gate effective viewport/root bounds in the diagnostics report before interpreting responsive
  overlay geometry;
- promote source-backed shell invariants into a lightweight recipe mechanism fixture when they can
  be reproduced without launching UI Gallery;
- keep full diagnostics scripts for viewport, portal, screenshot, or multi-frame overlay evidence.

## Phase 2.3 Layout Dirty Invalidation Coverage

The first invalidation fixture suite is
`crates/fret-ui/src/tree/tests/fixtures/layout_dirty_invalidation_v1.json`, run by
`mechanism_harness_layout_dirty_invalidation_matches_oracles`.

This slice extends the observation model with scalar mechanism metrics. Geometry predicates are not
enough for invalidation: the interesting facts are counts, flags, existence after removal, and debug
repair counters. The new `MechanismPredicate::MechanismMetric` keeps those facts inside the same
fixture/oracle runner without turning diagnostics into an in-process test harness.

It covers:

- child dirty transitions hidden behind `layout_dirty_children_suppressed`,
- suppressed parents that still count their own dirty flag,
- dirty child removal under suppressed parents,
- hidden dirty subtree removal while visible dirty siblings remain counted,
- underflow repair rebuilding aggregate counts upward after simulated count drift,
- contained view-cache roots consuming descendant layout dirtiness without forcing declarative
  rerender,
- direct-child/root-boundary dirty frontiers that are fully covered by contained view-cache roots,
- detached dirty cache roots being pruned from contained-relayout and dirty-view follow-up surfaces,
- view-cache layout-dirty expansion attribution for declarative-style element nodes.

The runtime counterpart is the UI Gallery checkbox script
`tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-demo-with-title-toggle-underflow.json`,
which locks the path that first exposed the suppressed-boundary underflow as a component-looking
layout problem.

## Phase 2.4 Scroll-Handle Window-Update Coverage

The scroll-handle invalidation fixture suite is
`crates/fret-ui/src/tree/tests/fixtures/scroll_handle_invalidation_v1.json`, run by
`mechanism_harness_scroll_handle_invalidation_matches_oracles`.

This suite covers the mechanism chain that decides whether a scroll-handle change can stay as a
hit-test/transform update or must dirty a cache root so the visible window can be rebuilt. The
important observation is cache-root dirty/reuse state, not only invalidation-walk detail.

It covers:

- windowed scroll paint dirtying the nearest cache root when an external offset change moves the
  visible paint window,
- revision-only bumps after runtime-owned internal offset updates staying reusable for generic
  windowed scroll paint,
- virtual-list visible windows escaping the cached overscan window and dirtying the cache root,
- virtual-list revision-only bumps still forcing a window update when the visible range has escaped,
- detached same-frame stale scroll-handle bindings being filtered through live layer-tree
  membership before they can dirty cache roots.

The runtime counterparts are the focused `view_cache_scroll` tests, the retained virtual-list host
test `retained_virtual_list_host_updates_window_without_rerendering_view_cache_root`, and scroll
registry classification tests in `crates/fret-ui/src/declarative/frame.rs`.

## Phase 2.5 Environment View-Cache Invalidation Coverage

The environment view-cache invalidation fixture suite is
`crates/fret-ui/src/declarative/tests/fixtures/environment_view_cache_invalidation_v1.json`, run by
`mechanism_harness_environment_view_cache_invalidation_matches_oracles`.

This suite intentionally drives `WindowMetricsService`, which is the source used by real desktop and
web runners. That is different from directly mutating `ElementRuntime`; the latter can make tests
green while missing the platform-to-declarative commit path.

It covers:

- viewport-size changes from render bounds,
- reduced motion,
- color scheme,
- contrast preference,
- forced colors mode,
- text scale factor,
- reduced transparency,
- accent color,
- safe-area insets,
- occlusion insets.

For each case, the dependent cache root must rerender exactly once after the environment change,
while a sibling plain cache root stays reused. The first fixture run exposed a real mechanism bug:
`render_root` synchronized only scale factor, safe-area, and occlusion from `WindowMetricsService`.
It now commits all known platform environment values into `ElementRuntime` before declarative
rendering, so environment dependency fingerprints can invalidate cache roots correctly.

## Phase 2.6 Pointer Occlusion and Capture Routing Coverage

The pointer occlusion routing fixture suite is
`crates/fret-ui/src/tree/tests/fixtures/pointer_occlusion_routing_v1.json`, run by
`mechanism_harness_pointer_occlusion_routing_matches_oracles`.

This suite covers the event-routing side of hit testing: the top-level hit result may still point at
an underlay node, while overlay pointer occlusion, modal barriers, observer passes, and pointer
capture decide which event chains are actually allowed to run.

It covers:

- `BlockMouseExceptScroll` suppressing underlay move/down dispatch while allowing wheel routing,
- `BlockMouse` suppressing underlay wheel dispatch too,
- hit-test-inert modal barriers scoping underlay wheel routing,
- outside-press observers still receiving preview events while pointer occlusion suppresses
  underlay bubble dispatch,
- pointer-move observers still running for occluding overlays while underlay move dispatch is
  suppressed,
- captured underlay pointers suppressing unrelated overlay pointer-move observers,
- captured overlay pointers continuing to receive moves while uncaptured pointers remain occluded.

The first run exposed a harness schema constraint rather than a runtime defect: `domains` is a fixed
mechanism enum, not a free-form tag list. Specific subdomain names such as pointer occlusion and
pointer capture should live in suite/case ids and coverage docs unless the shared schema is
intentionally extended.

## Phase 2.7 Focus Barrier Routing Coverage

The focus barrier routing fixture suite is
`crates/fret-ui/src/tree/tests/fixtures/focus_barrier_routing_v1.json`, run by
`mechanism_harness_focus_barrier_routing_matches_oracles`.

This suite covers the tree-level overlay/focus substrate that component-layer focus traps and
restore policies depend on. It intentionally stays below Radix policy: the fixture asserts focus
barrier, modal barrier, and focus traversal outcomes, not when a component should open, close, or
restore focus.

It covers:

- hit-test-inert overlays that still keep an active focus barrier,
- focus barrier activation preserving existing overlay focus,
- underlay focus attempts being rejected while the focus barrier is active,
- `focus.next` / `focus.previous` traversal staying inside the focus barrier even when the input
  barrier is off,
- modal overlays reporting both modal and focus barriers,
- timer dispatch not clearing focus inside a hit-test-inert focus barrier.

The first run exposed a runner type-boundary issue rather than a runtime defect: fixture command ids
must be represented as a closed enum in the runner instead of dynamically borrowed strings, because
`CommandId::from` is intentionally optimized for static command ids in these paths.

## Phase 2.8 Semantics Relation Coverage

The semantics relation fixture suite is
`crates/fret-ui/src/declarative/tests/fixtures/semantics_relations_v1.json`, run by
`mechanism_harness_semantics_relations_match_oracles`.

This suite promotes accessibility relationship facts into the shared mechanism harness instead of
leaving them as one-off assertions in declarative tests. `ObservedTree::from_semantics_snapshot`
now carries active-descendant, labelled-by, described-by, controls, disabled, hidden, focus-barrier,
and captured-node facts from `SemanticsSnapshot`; the oracle can assert relation membership and
boolean semantics flags by selector.

It covers:

- text-input combobox `controls_element` resolving to a listbox semantics node,
- text-input combobox `active_descendant_element` resolving to a listbox option semantics node,
- `attach_semantics` preserving active-descendant, labelled-by, described-by, controls, disabled,
  and hidden state without adding a layout wrapper,
- `SemanticsProps` wrapper exposing the same labelled-by, described-by, controls, disabled, and
  hidden outcomes.

The first run did not expose a runtime defect. It did expose a harness observability gap: the runtime
already produced these relationships in `SemanticsSnapshot`, but the mechanism harness could not
express them as reusable fixture predicates. This blocked proactive parity sweeps for combobox,
listbox, autocomplete, menu, and future Material 3 composite widgets.

The semantics-state extension expands the same suite beyond relations. The observed tree now mirrors
`SemanticsSnapshot` value text, selected/expanded/checked state, collection metadata, editing ranges,
action support, live-region metadata, and structured numeric/scroll metadata. The shared oracle now
supports the corresponding existing `UiPredicateV1` semantics predicates plus harness-native
action/live/range predicates.

New fixture cases:

- `text-input-region-value-and-editing-metadata`: focused `TextInputRegion` value, selection,
  composition, focus action, set-value suppression, and set-text-selection support.
- `pressable-collection-metadata-and-state`: listbox option `pos_in_set`, `set_size`, selected,
  checked/unchecked, and invoke action.
- `semantics-wrapper-live-and-structured-metadata`: `SemanticsProps` live region, live atomic,
  structured slider numeric metadata/action support, and viewport scroll metadata.

Findings from this extension:

- The runtime already emitted the state/action/metadata facts, but `fret-mechanism-harness` could not
  observe or assert them, so recipe parity sweeps could not catch many state-level regressions.
- `role_label` did not cover `Region`, `SpinButton`, `Meter`, or `Splitter`, causing those known core
  roles to appear as `unknown` in observed snapshots.
- `WindowTextInputSnapshot` test constructors had not been updated for the `visual` field, blocking
  targeted `fret-ui` nextest gates before the semantics fixture could run.

Validation:

- `cargo nextest run -p fret-mechanism-harness`
- `cargo nextest run -p fret-ui mechanism_harness_semantics_relations_match_oracles`
- `cargo nextest run -p fret-ui window_text_input_snapshot`

## Phase 2.9 Roving Focus Interaction Coverage

The roving focus interaction fixture suite is
`crates/fret-ui/src/declarative/tests/fixtures/roving_focus_interaction_v1.json`, run by
`mechanism_harness_roving_focus_interaction_matches_oracles`.

This suite lifts declarative roving-focus behavior into a fixture matrix because roving focus is the
shared substrate for menu, select, combobox, command, tab, toolbar, and Material listbox recipes. It
does not assert recipe-specific keyboard policy; it asserts the lower-level mechanism outcome after
the recipe supplies a navigation callback.

It covers:

- arrow navigation skipping disabled items,
- wrap-around from first to last item,
- non-wrapping edge navigation preserving the current focus and selection,
- pointer-region wrapped items still participating in roving item collection,
- printable-key typeahead dispatch moving focus and selection to a component-selected target,
- no-match typeahead preserving current focus and active selection,
- printable-key typeahead through pointer-region wrappers,
- `roving.typeahead.calls` so duplicate handler invocation is visible as a fixture failure.

The first run did not expose a runtime defect. It turned existing focused behavior into
case-id-addressable coverage so future parity sweeps can add recipe-shaped roving scenarios without
editing the harness runner.

The typeahead extension also did not expose a runtime defect. It closed a mechanism coverage gap
that previously lived only in `roving_flex` focused tests.

## Phase 2.10 Focus Scope Interaction Coverage

The focus scope interaction fixture suite is
`crates/fret-ui/src/declarative/tests/fixtures/focus_scope_interaction_v1.json`, run by
`mechanism_harness_focus_scope_interaction_matches_oracles`.

This suite covers the declarative focus-containment primitive that overlay recipes depend on before
they add Radix-style restore policy. It intentionally asserts mechanism outcomes only: traversal,
containment, and pointer-focus arbitration.

It covers:

- trapped `focus.next` traversal staying inside the scope,
- trapped `focus.next` wrapping from the last item back to the first item,
- trapped `focus.previous` wrapping from the first item to the last item,
- non-trapped scopes allowing focus traversal to leave the scope,
- outside pointer activation still running while trapped focus remains inside the scope.

The first run did not expose a runtime defect. It confirmed that `trap_focus=false` does not
accidentally behave as containment and converted existing procedural focus-scope checks into a
case-id-addressable fixture surface.

## Phase 2.11 Shadcn Focus Restore Recipe Coverage

The shadcn focus-restore fixture suite is
`ecosystem/fret-ui-shadcn/tests/fixtures/focus_restore_recipe_cases_v1.json`, run by
`mechanism_harness_focus_restore_recipe_cases_match_oracles`.

This suite is the first recipe-consumer parity slice in the mechanism harness lane. It exercises the
common Radix/shadcn outcome shared by dialog, popover, combobox, select, and dropdown-menu recipes:
trigger opens an overlay, dismissal closes it, and focus follows the recipe policy for the
dismissal reason.

It covers:

- dialog Escape dismissal restoring trigger focus,
- popover Escape dismissal restoring trigger focus,
- combobox Escape dismissal restoring trigger focus,
- select Escape dismissal restoring trigger focus,
- dropdown-menu Escape dismissal restoring trigger focus,
- combobox outside-press dismissal restoring trigger focus while allowing click-through underlay
  activation,
- select modal outside-press dismissal restoring trigger focus while keeping the underlay inactive,
- dropdown-menu non-modal outside-press dismissal clearing focus instead of restoring the trigger,
- context-menu Escape dismissal clearing focus,
- context-menu non-modal outside-press dismissal focusing and activating the underlay target,
- dialog outside-press dismissal restoring trigger focus,
- popover click-through outside-press dismissal focusing and activating the underlay target,
- prevent-default outside-press outcomes for popover, select, dropdown-menu, and context-menu,
- modal prevent-default outcomes keeping the overlay open while blocking underlay activation,
- non-modal prevent-default outcomes keeping the overlay open while preserving click-through
  underlay focus/activation,
- pure focus-outside outcomes for popover, non-modal dropdown-menu, and non-modal context-menu,
- focus-outside `preventDefault` outcomes that keep the overlay open, preserve the externally
  focused target, and assert edge-triggered dismiss-call counts,
- open-state closure and dismiss-call counts as mechanism metrics alongside focus restoration.

The select outside-press fixture uses the trigger pointer-open path. The first draft exposed a
confirmed guard-cache defect: select's mouse-open pointer-up guard reused a suppress decision for a
different pointer id when tick ids were adjacent. The guard now keys cached pointer-up decisions by
pointer id as well as tick id.

The prevent-default matrix exposed a second select defect: the modal barrier installed its
pointer-up dismissal hook with an accumulating `add` API, so a long-lived open select invoked the
dismiss handler once per rendered frame. The barrier now installs a single pointer-up handler for
that owned behavior.

The focus-outside extension did not expose a runtime defect. It closed a coverage gap by making
pure focus transfers explicit instead of relying on outside-click cases that also exercise pointer
routing and underlay activation.

Context-menu now also has a focused submenu restore gate matching the existing dropdown-menu gate:
ArrowRight opens a submenu and transfers focus into it, while ArrowLeft restores focus to the
submenu trigger.

The first run exposed a real harness oracle defect. `UiPredicateV1::FocusIs` used the normal
barrier-filtered selector path, so it could not match a trigger outside a still-present pointer/modal
barrier during close transition even when runtime focus had already been restored. The fix adds
unfiltered selector lookup for focus predicates while keeping normal `Exists`-style queries
barrier-filtered.

## Phase 2.12 Focus Interaction Extension Coverage

The declarative focus/semantics extension suites now cover two high-risk intersections that were
previously only implied by narrower tests:

- `crates/fret-ui/src/declarative/tests/fixtures/combobox_active_descendant_interaction_v1.json`,
  run by `mechanism_harness_combobox_active_descendant_interaction_matches_oracles`, drives a real
  text-input query path and asserts the resulting active-descendant relation plus query, visible
  count, and active-index metrics.
- `crates/fret-ui/src/declarative/tests/fixtures/focus_scope_nested_interaction_v1.json`, run by
  `mechanism_harness_nested_focus_scope_interaction_matches_oracles`, asserts inner/outer trapped
  focus scope traversal and outside pointer activation without focus leakage from the trapped inner
  scope.

The first runs did not expose runtime defects. They closed fixture-level gaps before recipe-level
typeahead, submenu, and nested overlay parity work depends on those mechanisms.

## Phase 2.13 Retained Tree Stale-Parent Coverage

The tree-level stale-parent focus scope fixture suite now covers the retained-tree parent-pointer
mutation path that previously existed only as a focused Rust test.

It exercises:

- child reachability through the dispatch snapshot after the parent pointers are cleared,
- pointer activation outside the trapped scope,
- focus staying inside the trapped scope after the pointer click path.

The first draft exposed a harness schema constraint rather than a runtime defect: `domains` must use
the fixed mechanism enum and cannot accept a free-form `dispatch` tag. After aligning the fixture
to the allowed domain set, the suite passed.

## Phase 2.14 Shadcn Recipe Semantics Coverage

The shadcn recipe semantics fixture suite is
`ecosystem/fret-ui-shadcn/tests/fixtures/recipe_semantics_cases_v1.json`, run by
`mechanism_harness_recipe_semantics_cases_match_oracles` in
`ecosystem/fret-ui-shadcn/tests/recipe_semantics_mechanism_harness.rs`.

This suite is the first consumer of the expanded semantics/state/action oracle from a shadcn recipe
surface. It intentionally verifies behavior outcomes instead of source shape.

It covers:

- open combobox search input exposing `role=combo_box`, `expanded=true`, focus/set-value actions,
  `controls` linkage to its listbox, and reciprocal listbox labelling;
- committed combobox selection exposing selected option state plus `pos_in_set` / `set_size`
  collection metadata;
- combobox ArrowDown highlight using `active_descendant` while keeping committed selection false;
- open select listbox ↔ trigger labelled-by/controls relationships across the modal barrier root;
- selected select option active-item, selected-state, collection metadata, and invoke-action
  semantics.

The first run exposed a real mechanism-harness oracle defect. `SemanticsRelationIncludes` resolved
both relation endpoints through the normal barrier-filtered selector path. That made a valid modal
select relation fail: the open listbox lives inside the barrier while its `labelled_by` target is the
underlay trigger. Relation predicates now resolve source and target with unfiltered selectors, while
ordinary `role_is`, `expanded`, and similar current-surface predicates remain barrier-filtered.

This slice also clarified an authoring rule for future recipe fixtures:

- use relation predicates for cross-root semantics edges such as `labelled_by` and `controls`;
- do not assert underlay trigger role/state through normal selectors while a modal barrier is active;
- assert the current accessible surface on the overlay listbox/options, and use metrics or relation
  predicates for hidden/outside nodes when they are still needed as semantics references.

Validation:

- `cargo nextest run -p fret-mechanism-harness`
- `cargo test -p fret-ui-shadcn --test recipe_semantics_mechanism_harness mechanism_harness_recipe_semantics_cases_match_oracles -- --exact --nocapture`

## Phase 2.15 Combobox Overlay Placement and Chrome Sweep

The combobox web-golden overlay sweep is still an ecosystem parity surface rather than a core
mechanism fixture, but it now acts as an active defect-discovery lane for overlay positioning,
controller semantics, and option chrome.

It covers:

- desktop and mobile `combobox-demo`, `combobox-popover`, and `combobox-responsive` listbox sizing,
  option height, option insets, and overlay placement;
- highlighted and focused command-item chrome across light/dark themes and desktop/mobile
  viewports;
- the `device_shell_responsive` desktop Popover shell versus mobile Drawer shell split.

Findings from this sweep:

- `combobox-responsive.overlay_placement` exposed a recipe source drift: Fret used the Base UI
  `ComboboxContent` default `sideOffset=6` and a 200px trigger, while the upstream shadcn responsive
  example is a Popover/Button/Command recipe with a 150px trigger, 200px content, and Popover
  `sideOffset=4`. Responsive mode now keeps those demo-owned defaults separate from ordinary
  `ComboboxContent`.
- `combobox-demo.focus-first` exposed a harness driver gap: overlay chrome fallback logic only
  searched for `TextField` controllers, but Fret correctly exposes the cmdk input as a `ComboBox`
  controller with `aria-activedescendant` semantics. The chrome harness now finds the node that
  controls the listbox and renders once after ArrowDown before reading the semantics snapshot.

Validation:

- `cargo test -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement combobox::fixtures::web_vs_fret_combobox_cases_match_web_fixtures -- --exact --nocapture`
- `cargo test -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome combobox::fixtures::web_vs_fret_combobox_overlay_chrome_cases_match_web_fixtures -- --exact --nocapture`
- `cargo test -p fret-ui-shadcn --lib combobox::tests::responsive_combobox_uses_shadcn_popover_demo_defaults -- --exact --nocapture`
- `cargo test -p fret-ui-shadcn --test combobox_responsive_breakpoint -- --nocapture`

The next slice promoted that uncovered trigger-slot risk into a dedicated fixture suite.

## Phase 2.16 Combobox Trigger Slot Geometry Sweep

The combobox trigger-slot fixture suite is
`ecosystem/fret-ui-shadcn/tests/fixtures/layout_combobox_trigger_cases_v1.json`, run by
`web_vs_fret_layout_combobox_trigger_slots_match_web_fixtures` in the web-vs-Fret layout harness.

It covers:

- `combobox-demo` trigger width/height, right icon size, right inset, center-y alignment, and label
  bounds staying before the icon;
- `combobox-responsive` and `combobox-popover` trigger width/height and the upstream text-only
  trigger shape, where the label owns the full content lane and no icon slot is rendered;
- a long selected-label case that keeps truncation pressure visible while asserting the label does
  not overlap the icon slot.
- trigger label `fontWeight` against the web golden's computed style, so chrome drift can be caught
  without relying on screenshots.
- trigger icon SVG identity for icon-bearing Button examples, so the harness can distinguish an
  upstream `ChevronsUpDown` glyph from a geometrically-correct but visually-wrong `ChevronDown`.

Finding from this sweep:

- `combobox-responsive.trigger_text_only` exposed a real recipe drift. Upstream shadcn responsive
  combobox uses a plain outline Button with `w-[150px] justify-start` and no right-side icon, while
  Fret's responsive `Combobox` recipe inherited the default combobox trigger icon. That extra 16px
  slot reduced the already-narrow text lane and is the kind of issue users see as "text collapses to
  ellipsis" or "arrow positioning looks wrong". Responsive Button triggers now hide the trigger icon
  by default, while an explicit `ComboboxInput::show_trigger(true)` still restores it for callers
  that intentionally want the icon.
- The font-weight extension exposed a second recipe chrome drift. Upstream shadcn Button triggers
  compute to `fontWeight=500`, but Fret's `ComboboxTriggerVariant::Button` rendered the label at
  `400`. Button-like combobox triggers now use `FontWeight::MEDIUM`, while the default input-like
  trigger stays normal-weight.
- UI Gallery docs-surface inspection exposed a diagnostics coverage gap: the Combobox page had
  `Long Text` and `RTL Long Text` follow-up sections, but the docs smoke script did not wait for
  them. The smoke gate now includes both truncation-oriented sections.
- Adding `combobox-popover.trigger_text_only` exposed that the responsive-only icon default was
  still too narrow. Upstream Button-trigger combobox examples are text-only by default; the simple
  `combobox-demo` is the special case that explicitly authors an icon. Fret now treats
  `ComboboxTriggerVariant::Button` as text-only unless `ComboboxInput::show_trigger(true)` is
  explicitly set, and the Usage/Long Text follow-ups opt into the icon where their examples need
  truncation-plus-icon coverage.
- The icon-identity extension first exposed a harness setup gap: the web-vs-Fret layout test app
  applied shadcn theme tokens but did not install the lucide icon pack, so painted SVG assertions
  observed `MISSING_ICON_SVG`. The shared layout harness now installs lucide semantic aliases like
  real gallery/bootstrap paths.
- After the harness installed real icons, the same fixture exposed a recipe glyph drift: explicit
  Button trigger icons rendered `ChevronDown`, while upstream `combobox-demo` authors
  `ChevronsUpDown`. `ComboboxTriggerVariant::Button` now uses the semantic double-chevron icon for
  its explicit trigger slot; the default input-like trigger keeps `ChevronDown`.

Validation:

- `cargo test -p fret-ui-shadcn --test web_vs_fret_layout combobox_trigger -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib button_trigger_hides_icon_unless_explicit -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib combobox_show_trigger_hides_chevron_icon -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib combobox_trigger_long_label_stays_before_chevron -- --nocapture`
- `cargo test -p fret-ui-gallery --test combobox_docs_surface -- --nocapture`

## Phase 2.17 RTL Combobox Scroll Settle and Overlay Gate

The runtime RTL combobox gate is
`tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-flip-tight-window.json`.

This slice started from a user-visible symptom: after scrolling the UI Gallery Combobox page near
the RTL demo in a tight window, the page appeared to move up and down. The first failing run showed
two separate harness defects:

- `scroll_into_view` drove the page with a fixed 40px wheel delta even when the target was only
  about 5px outside the padded visible region. That made the script overshoot and bounce between
  adjacent offsets instead of converging.
- The RTL flip script waited for an anchored-panel placement trace whose `content_test_id` was the
  internal listbox. Runtime traces correctly report the positioned panel shell
  (`ui-gallery-combobox-rtl-content`), while the listbox remains the inner geometry target.

Fixes:

- Diagnostics `scroll_into_view` now caps each axis delta to the remaining visibility gap and treats
  subpixel target/container jitter as stable progress.
- The RTL flip script targets `ui-gallery-content-viewport` as the scroll viewport, waits for the
  trigger bounds to settle after scrolling, queries overlay placement with
  `ui-gallery-combobox-rtl-content`, and then waits for the content bounds to settle before checking
  the visible listbox.
- `combobox_diag_surface` now locks the content/listbox split so future script edits do not regress
  the panel-vs-inner-listbox contract.

Evidence:

- Failing artifact:
  `target/fret-diag/codex-combobox-rtl-scroll-20260513-1619/sessions/1778660392387-48984/1778660666632-script-step-0018-wait_overlay_placement_trace-timeout`
- Passing artifact:
  `target/fret-diag/codex-combobox-rtl-scroll-fix-20260513-1650/sessions/1778662143311-113768/1778662380264-ui-gallery-combobox-flip-tight-window`

Validation:

- `target/debug/fretboard-dev.exe diag script validate tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-flip-tight-window.json`
- `cargo test -p fret-bootstrap --features "ui-app-driver diagnostics" script_steps_scroll::tests -- --nocapture`
- `cargo test -p fret-ui-gallery --test combobox_diag_surface combobox_rtl_flip_diag_script_separates_overlay_shell_from_listbox_geometry -- --nocapture`
- `target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-flip-tight-window.json --dir target/fret-diag/codex-combobox-rtl-scroll-fix-20260513-1650 --session-auto --launch -- cargo run -p fret-ui-gallery`

Follow-up recommendation:

- Promote "scroll to target, then prove target/content bounds stability before interaction" as the
  default pattern for tight-window overlay scripts. This should be swept across combobox, select,
  dropdown menu, popover, hover card, and future Material 3 anchored surfaces.

## Phase 2.18 Combobox Overlay Content-Shell Sweep

The Phase 2.17 RTL fix exposed a family-level diagnostics drift: many older combobox scripts waited
for `wait_overlay_placement_trace` using the inner `*-listbox` as `content_test_id`. The current
Combobox recipe derives `*-content` from `test_id_prefix` and records overlay placement against that
positioned content shell; `*-listbox` is the inner list geometry target.

Fixes:

- All UI Gallery Combobox overlay trace scripts now query the positioned `*-content` shell.
- Existing `bounds_within_window` and `wait_bounds_stable` checks stay on `*-listbox` where they are
  checking visible list geometry rather than overlay placement.
- `combobox_overlay_trace_scripts_target_content_shells_not_inner_listboxes` scans the whole
  combobox script directory and fails if a future overlay trace query regresses to `*-listbox`.

Evidence:

- Runtime proof:
  `target/fret-diag/codex-combobox-content-shell-sweep-20260513-1745/sessions/1778665071077-117228/1778665303231-ui-gallery-combobox-demo-open-narrow`
- The representative legacy script now records
  `anchor=ui-gallery-combobox-demo-trigger`, `content=ui-gallery-combobox-demo-content`, and
  `chosen_side=top` in the passing run.

Validation:

- All combobox scripts passed `target/debug/fretboard-dev.exe diag script validate`.
- `cargo test -p fret-ui-gallery --test combobox_diag_surface combobox_overlay_trace_scripts_target_content_shells_not_inner_listboxes -- --nocapture`
- `target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-demo-narrow-open-screenshot.json --dir target/fret-diag/codex-combobox-content-shell-sweep-20260513-1745 --session-auto --launch -- cargo run -p fret-ui-gallery`

## Diagnostics Reuse

Diagnostics reuse happens through the protocol predicate layer, not by linking UI Gallery to the Rust
harness crate. `UiPredicateV1::BoundsMetricDelta` expresses scalar bounds relationships such as
`center_y(a) - center_y(b) == 0`, so script-only gates can assert alignment without bespoke app code.

The promoted seed script
`tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json` now uses this
predicate for ButtonGroupText prefix/suffix alignment against the input control.

For hit-test routing, the equivalent UI Gallery diagnostics path is:

- Use stable `test_id` selectors for the intended target, wrapper, overlay panel, and underlay.
- Use pointer steps such as `move_pointer`, `click`, or `click_stable`; these already record
  `UiHitTestTraceEntryV1` entries with hit semantics ids/test ids, hit paths, active layer roots,
  and barrier roots.
- Use `barrier_roots` assertions when the script needs modal/focus barrier state without stable
  node ids.
- Use `capture_layout_sidecar` near layout-vs-hit symptoms and `capture_bundle` for the trace.

This intentionally leaves the Rust harness as the in-process oracle owner while diagnostics remains
the UI Gallery recorder/triage path. If a future UI Gallery page exposes dedicated mechanism samples,
the script-side predicates should map one-to-one to the selectors above rather than importing
`fret-mechanism-harness`.

## Harness vs Diagnostics

The harness crate and diagnostics crates are deliberately different tools:

- `fret-mechanism-harness` is an in-process mechanism lab. It owns fixture loading, observed-tree
  queries, and oracle predicates over controlled runtime scenarios.
- `fret-diag-protocol` owns the serialized vocabulary that can cross process and tool boundaries.
- `fret-bootstrap` owns diagnostics execution inside a running app: script playback, screenshots,
  sidecars, bundles, and portable evidence.

The shared surface should stay narrow: selectors and predicates that describe runtime facts. The
harness should not grow a script runner, screenshot pipeline, bundle writer, or UI Gallery launcher;
diagnostics should not need to link recipe-specific test harnesses to assert basic geometry.

## Gates

- `cargo nextest run -p fret-mechanism-harness`
- `cargo nextest run -p fret-diag-protocol`
- `cargo nextest run -p fret-ui mechanism_harness_layout_primitives_match_oracles`
- `cargo nextest run -p fret-ui mechanism_harness_layout_dirty_invalidation_matches_oracles`
- `cargo nextest run -p fret-ui mechanism_harness_hit_test_routing_matches_oracles`
- `cargo nextest run -p fret-ui mechanism_harness_semantics_relations_match_oracles`
- `cargo nextest run -p fret-ui mechanism_harness_combobox_active_descendant_interaction_matches_oracles`
- `cargo nextest run -p fret-ui mechanism_harness_roving_focus_interaction_matches_oracles`
- `cargo nextest run -p fret-ui mechanism_harness_focus_scope_interaction_matches_oracles`
- `cargo nextest run -p fret-ui mechanism_harness_nested_focus_scope_interaction_matches_oracles`
- `cargo nextest run -p fret-ui mechanism_harness_focus_scope_stale_parent_interaction_matches_oracles`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness_recipe_layout_cases_match_oracles`
- `cargo nextest run -p fret-ui-shadcn --test focus_restore_mechanism_harness mechanism_harness_focus_restore_recipe_cases_match_oracles`
- `cargo test -p fret-ui-shadcn --test recipe_semantics_mechanism_harness mechanism_harness_recipe_semantics_cases_match_oracles -- --exact --nocapture`
- `cargo test -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement combobox::fixtures::web_vs_fret_combobox_cases_match_web_fixtures -- --exact --nocapture`
- `cargo test -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome combobox::fixtures::web_vs_fret_combobox_overlay_chrome_cases_match_web_fixtures -- --exact --nocapture`
- `cargo test -p fret-ui-shadcn --test web_vs_fret_layout combobox_trigger -- --nocapture`
- `cargo test -p fret-bootstrap --features "ui-app-driver diagnostics" script_steps_scroll::tests -- --nocapture`
- `cargo test -p fret-ui-gallery --test combobox_diag_surface combobox_rtl_flip_diag_script_separates_overlay_shell_from_listbox_geometry -- --nocapture`
- `cargo test -p fret-ui-gallery --test combobox_diag_surface combobox_overlay_trace_scripts_target_content_shells_not_inner_listboxes -- --nocapture`
- `cargo check -p fret-bootstrap`
- `python tools/check_layering.py`

## Future Domains

The observed model already has slots for hit-test samples, overlay records, and focus ids, plus
basic overlay bounds predicates. Follow-up suites should add or extend domain-specific adapters and
predicates in this order:

1. hit-test routing matrices: transformed, clipped, transparent, overlay roots, pointer occlusion,
   and captured-pointer routing;
2. semantics tree invariants: value/editing metadata, collection metadata, actions, live regions,
   and hidden-subtree policy;
3. overlay placement/focus containment: anchor spaces, focus trap/restore, active descendants, and
   nested modal roots;
4. Material 3 recipe parity: consume the same mechanism fixture/oracle format with Material-specific
   scenarios in the ecosystem layer.

The rule remains: mechanism predicates can live in the shared harness or diagnostics protocol;
component policy stays in ecosystem fixtures and recipe runners.
