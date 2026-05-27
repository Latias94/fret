# `fret-node` Runtime/Store Contract Closure (v1)

Status: closed
Last updated: 2026-05-27
Scope: `ecosystem/fret-node`, with documentation touch points in `docs/`

## Problem

`fret-node` has already moved toward a store-first, declarative-first editor architecture, but a
fresh audit found correctness gaps in the runtime contract layer that make broader cleanup risky:

- `GraphOp` carries many edit semantics, but `NodeGraphChanges` maps only a subset and silently
  drops the rest.
- `NodeGraphLookups` caches fields such as hidden state and reconnectability, but incremental
  update paths do not cover every operation that can affect those caches.
- The public `headless` feature name suggests a mode that is not actually enabled unless default
  features are disabled.
- UI binding/canvas code still carries multiple state mirrors that depend on sync glue staying
  perfectly aligned.
- Large surface-policy tests in `src/lib.rs` make the crate root too heavy and hide the actual
  contract being protected.
- The current dependency and roadmap story for `fret-ui-kit` needs a deliberate boundary decision.

These are not isolated bugs. They are symptoms of a runtime/store contract that is not yet strict
enough to support fearless cleanup of the UI compatibility layers.

## Target State

The target state is a `fret-node` architecture where:

- committed graph edits flow through one store/transaction pipeline,
- every observable graph edit either emits a deliberate `NodeGraphChange` or is explicitly marked
  non-observable,
- lookup caches are correct after every store dispatch without requiring callers to rebuild state
  manually,
- controlled-mode synchronization and callbacks are derived from the same change semantics as store
  dispatch,
- UI surfaces read from store/controller/binding snapshots instead of maintaining long-lived graph
  mirrors where possible,
- compatibility retained paths remain explicitly scoped and delete-planned,
- feature names and docs say exactly which feature set is headless, UI-enabled, or compatibility
  retained,
- tests that guard public surface policy live in focused integration tests or audit helpers rather
  than the crate root.

Closeout result on 2026-05-27:

- The runtime/store correctness target is met for the scoped lane.
- Long-lived `NodeGraphSurfaceBinding` mirrors are quarantined behind an explicit private mirror
  owner.
- Retained `NodeGraphCanvas` mirror cleanup remains a compatibility follow-on, not part of this
  closed runtime/store contract lane.

## Architecture Direction

Use XYFlow as the primary architecture reference for store, lookup, internals, and controlled
change-callback pipelines. Use egui-snarl mainly for API-shape and headless-vs-viewer separation
intuition.

The refactor should run in this order:

1. Close runtime correctness first: `GraphOp` to `NodeGraphChanges`, then lookup cache updates.
2. Tighten the store dispatch pipeline so document mutation, change emission, lookup updates,
   history, and subscribers cannot drift.
3. Reduce UI-owned mirrors only after the runtime contract is reliable.
4. Clean up feature names, dependency-boundary docs, and large policy-test placement once behavior
   gates are in place.

## Relationship To Existing Workstreams

This lane does not replace
`docs/workstreams/fret-node-declarative-fearless-refactor-v1/`. That older workstream remains the
authoritative background for declarative-first public posture, retained compatibility posture, and
controller/binding direction.

This lane is narrower: it closes runtime/store correctness and then uses that stronger contract to
drive the next cleanup slices. It may update the older workstream only when this lane changes its
public posture or closeout evidence.

## Scope

Primary code scope:

- `ecosystem/fret-node/src/ops/`
- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/ui/binding*.rs`
- `ecosystem/fret-node/src/ui/canvas/`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/Cargo.toml`

Primary documentation scope:

- this workstream folder,
- `docs/node-graph-roadmap.md`,
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/` when public posture changes,
- ADR alignment docs only if a hard cross-crate contract changes.

## Non-goals

- Do not redesign `Graph` or `GraphTransaction` from scratch unless a focused task proves the
  existing model cannot express required semantics.
- Do not introduce new long-term retained-widget public APIs.
- Do not move design-system policy into `fret-node` mechanism code to make tests pass.
- Do not treat `compat-retained-canvas` as the long-term default surface.
- Do not perform broad UI rewrites until the runtime/store correctness gates are green.

## Assumptions

- The existing public recommendation remains declarative-first with store/controller/binding as the
  taught app-facing path.
- `compat-retained-canvas` remains a compatibility path.
- `fret-node` may keep a UI feature by default if that is the documented product choice, but the
  `headless` wording must not mislead users about feature resolution.
- A rebuild fallback for lookup caches is acceptable only when it is explicit, tested, and bounded;
  silent partial incremental updates are not acceptable.

## Risk Model

- Highest correctness risk: controlled mode receives incomplete changes and external state drifts.
- Highest state risk: lookup caches return stale hidden/reconnectability or endpoint data after
  incremental dispatch.
- Highest migration risk: removing UI mirrors too early can regress retained compatibility tests.
- Highest process risk: broad cleanup without focused gates will make review impossible.
