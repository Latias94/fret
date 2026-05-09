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

## Diagnostics Reuse

Diagnostics reuse happens through the protocol predicate layer, not by linking UI Gallery to the Rust
harness crate. `UiPredicateV1::BoundsMetricDelta` expresses scalar bounds relationships such as
`center_y(a) - center_y(b) == 0`, so script-only gates can assert alignment without bespoke app code.

The promoted seed script
`tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json` now uses this
predicate for ButtonGroupText prefix/suffix alignment against the input control.

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
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness_recipe_layout_cases_match_oracles`
- `cargo check -p fret-bootstrap`
- `python tools/check_layering.py`

## Future Domains

The observed model already has slots for hit-test samples, overlay records, and focus ids, plus
basic overlay bounds predicates. Follow-up suites should add domain-specific adapters and predicates
in this order:

1. hit-test routing matrices: transformed, clipped, transparent, and overlay roots;
2. semantics tree invariants: roles, labels, relationships, active descendant, and hidden nodes;
3. overlay placement/focus containment: anchor spaces, modal roots, focus trap/restore;
4. Material 3 recipe parity: consume the same mechanism fixture/oracle format with Material-specific
   scenarios in the ecosystem layer.

The rule remains: mechanism predicates can live in the shared harness or diagnostics protocol;
component policy stays in ecosystem fixtures and recipe runners.
