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
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness_recipe_layout_cases_match_oracles`
- `cargo check -p fret-bootstrap`
- `python tools/check_layering.py`

## Future Domains

The observed model already has slots for hit-test samples, overlay records, and focus ids, plus
basic overlay bounds predicates. Follow-up suites should add domain-specific adapters and predicates
in this order:

1. hit-test routing matrices: transformed, clipped, transparent, and overlay roots;
2. semantics tree invariants: roles, labels, relationships, active descendant, and hidden nodes;
3. overlay placement/focus containment: anchor spaces, focus trap/restore, and nested modal roots;
4. Material 3 recipe parity: consume the same mechanism fixture/oracle format with Material-specific
   scenarios in the ecosystem layer.

The rule remains: mechanism predicates can live in the shared harness or diagnostics protocol;
component policy stays in ecosystem fixtures and recipe runners.
