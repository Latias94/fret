# Dataflow Authoring Surface (Fearless Refactor v1)

Status: active planning lane (pre-release fearless refactor)
Last updated: 2026-03-17

Related:

- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/MIGRATION_MATRIX.md`
- `docs/workstreams/ecosystem-integration-traits-v1/DESIGN.md`
- `docs/workstreams/router-v1/router-v1.md`
- `docs/workstreams/router-ui-v1/router-ui-v1.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/crate-usage-guide.md`

## Why this workstream exists

The broad authoring cleanup chain is now closed:

- app/component/advanced tiering is already settled,
- `LocalState<T>` is already the default local-state story,
- the default docs/templates/examples no longer teach raw `Model<T>` plumbing as first contact,
- and the conversion surface is already in closeout / maintenance.

The largest remaining day-to-day authoring gap is narrower:

> Fret's default dataflow language is still too wide across `action`, `selector`, and `query`.

The current repo already teaches a consistent high-level posture:

- `cx.state()`
- `cx.actions()`
- `cx.data()`
- `cx.effects()`

But the concrete authoring path under that posture still asks users to choose among too many
nearby spellings and ceremony-heavy sub-paths.

That is now the main remaining ergonomics problem for ordinary app authors.

## Current conclusion

This workstream is **not** about inventing a new runtime, a new state architecture, or a new
reactive paradigm.

It is about collapsing the default app-facing dataflow surface so that:

1. generic app authors see one compact LocalState-first language,
2. editor-grade apps keep an explicit advanced/shared-model lane,
3. reusable ecosystem crates can adapt to the same story without being forced onto `fret`,
4. router/navigation stays compatible without being dragged into the same refactor budget.

The current pain is most visible in Todo-like examples, but the target is larger than Todo:

- generic product UIs,
- query/forms/commands surfaces,
- editor-grade workspaces with shared models and background work,
- and reusable ecosystem libraries that must stay layer-correct.

## Consumer matrix

| Consumer | What should feel easy | What must stay explicit |
| --- | --- | --- |
| Generic app authors | local form/list/dialog/dashboard flows on `fret` | shared `Model<T>` graphs, advanced host/runtime seams |
| Editor-grade app authors | command-heavy, multi-panel, background-data surfaces with a stable default LocalState-first path where applicable | cross-view ownership, workspace/document graphs, router/window policy |
| Reusable ecosystem crate authors | optional action/selector/query adapters that fit the current app-facing language | hard dependency on `fret`, router policy, app-owned install/runtime assumptions |

## Goals

1. Define one compact default dataflow dialect for app-facing `action`, `selector`, and `query`
   authoring.
2. Keep the default path LocalState-first without reopening the closed storage-model decision.
3. Preserve an explicit advanced/editor-grade lane for shared `Model<T>` graphs, long-lived
   services, and host-owned coordination.
4. Keep `fret-selector`, `fret-query`, and `fret-router` semantically narrow instead of teaching
   `LocalState<T>`-specific app sugar in the domain crates.
5. Make ecosystem adaptation predictable:
   - `ecosystem/fret` owns default app-facing helper/binder surfaces,
   - direct ecosystem crate usage stays possible,
   - reusable kits keep selector/query/router optional unless they truly need them.
6. Keep first-party docs/templates/examples/gates aligned with the chosen surface and hard-delete
   displaced spellings before public release.

## Non-goals

- Replacing the current runtime with a second authoring runtime.
- Reopening the app/component/advanced lane split.
- Reopening the `LocalState<T>` storage-model decision.
- Merging routing/history/link semantics into the same lane.
- Forcing all reusable ecosystem crates to depend on `fret`.
- Designing the shortest possible toy-app syntax at the cost of explicit ownership/invalidation.
- Growing a universal `Signal`, `Component`, or ecosystem-wide plugin abstraction.

## Scope

### In scope

- default app-facing action authoring for:
  - one-slot local writes,
  - multi-slot LocalState transactions,
  - keyed-row payload writes,
  - app-only transient/effect handoff,
  - explicit shared-model fallback posture
- LocalState-first selector dependency/read authoring
- query read-side authoring on the default app path
- ecosystem adapter rules for reusable kits and first-party crates
- docs/templates/examples/source-gate alignment for the chosen default dataflow surface

### Out of scope

- route codec/history/link semantics
- router window policy, scroll/focus restoration, or navigation diagnostics
- query engine semantics, cache policy semantics, or transport/runtime internals
- selector engine semantics, memoization internals, or invalidation classes
- component interaction policy in `fret-ui-kit` / `fret-ui-shadcn`

## Ownership model

### `crates/*`

- keep runtime semantics, typed action dispatch semantics, invalidation classes, and query/selector
  engine contracts stable and explicit
- do not grow app-facing sugar here just to hide authoring ceremony

### `ecosystem/fret`

- owns the default app-facing dataflow language
- is the correct owner for LocalState-first helper/binder surfaces that collapse repeated app
  ceremony
- may wrap selector/query/action engine surfaces when that wrapper is clearly app-facing and
  delete-ready

### `ecosystem/fret-selector` / `ecosystem/fret-query`

- keep domain semantics narrow
- do not become `LocalState<T>`-teaching crates
- should remain usable directly by reusable ecosystem libraries and advanced apps without dragging
  in the full `fret` facade

### `ecosystem/fret-ui-kit` / reusable ecosystem crates

- may add optional adapters if the default app-facing dialect needs reusable integration points
- should keep selector/query/router optional where possible
- must not assume a universal app-owned runtime or route stack

## Router relation

Router is **adjacent**, not in scope for this workstream.

Reason:

- not every app needs routing,
- router already owns typed-route/history/link contracts in `router-v1` and `router-ui-v1`,
- mixing route/navigation policy into this lane would widen the problem back into a broad
  ecosystem reset.

Compatibility rule:

- any new default dataflow helper surface must remain composable with route snapshot models,
  route-keyed query invalidation, and typed navigation actions,
- but route authoring itself should continue to evolve in the router workstreams.

## Design rules

1. One default spell per default use case.
   The default app lane should not require users to choose among several nearly equivalent helpers
   before they understand ownership.
2. Keep LocalState-first for the default lane.
   Shared `Model<T>` graphs remain an explicit advanced/editor-grade lane.
3. Keep domain crates narrow.
   If a helper exists mainly to make `LocalState<T>` authoring shorter, it belongs on the
   app-facing facade, not in `fret-selector`, `fret-query`, or router crates.
4. Prove changes on more than Todo.
   No new shared helper should land without proof on:
   - a generic app surface,
   - an editor-grade surface,
   - and at least one reusable-ecosystem-facing consideration.
5. Preserve explainable ownership and invalidation.
   Shorter syntax must not erase whether a value is local, shared, derived, async, or route-owned.
6. Delete, do not accumulate.
   Fret is still pre-release; once a better default path lands, displaced public-looking spellings
   should be retired rather than documented as co-equal choices.

## Required proof surfaces

### Generic app surfaces

- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fretboard/src/scaffold/templates.rs`

### Editor-grade surfaces

- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/launcher_utility_window_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`

### Ecosystem / reusable-library proof surfaces

- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- any first-party reusable adapter point added under `ecosystem/*`

## Target shape

The exact API spellings may still change. The contract target is the important part.

### 1. Action authoring

The default lane should converge on one obvious path for each of these cases:

- one-slot local write
- multi-slot LocalState transaction
- keyed payload row write
- app-only transient/effect handoff
- shared-model coordination (explicit advanced lane)

Current problem:

- the repo has multiple adjacent choices that are each individually defensible but collectively
  widen the mental model too early.

Target reading:

- ordinary app authors should not need to stop and choose between several near-equal transaction
  helper families before they understand the ownership difference,
- keyed-row payload writes should have one canonical happy path,
- shared-model coordination should remain clearly separate and intentionally more explicit.

Illustrative shape only:

```rust
cx.actions().local(&draft_state).set_on::<act::ClearDraft>(String::new());
cx.actions().locals((&draft_state, &next_id_state, &todos_state))
    .on::<act::Add>(|draft, next_id, todos| { ... });
cx.actions().local(&todos_state)
    .on_payload::<act::Toggle>(|rows, id| { ... });
```

### 2. Selector authoring

The default app lane should converge on one LocalState-first dependency/read story.

Current problem:

- the default app path still exposes a two-step `DepsBuilder` + `*_in(cx)` choreography that is
  correct but noisier than it needs to be for LocalState-first apps.

Target reading:

- generic app authors should be able to define derived values without learning the raw dependency
  builder as the first-contact story,
- editor-grade/shared-model surfaces must still retain an explicit raw dependency path,
- reusable ecosystem crates should still be able to use `fret-selector` directly without app-facade
  assumptions.

Illustrative shape only:

```rust
let summary = cx.data().select_local(
    |deps| (deps.layout(&todos_state), deps.layout(&filter_state)),
    |todos, filter| build_summary(todos, filter),
);
```

### 3. Query authoring

The query engine should remain explicit about:

- key,
- policy,
- fetch,
- and status.

This lane should focus on **read-side collapse**, not on hiding query semantics.

Target reading:

- query creation stays explicit,
- common read-side patterns should not require as much repeated handle/status/default plumbing on
  the default app path,
- advanced consumers can still drop to the full handle/state model when they need exact control.

Current app-facing candidate (2026-03-17):

- keep creation explicit on `cx.data().query*`,
- collapse the common app-path fallback to `handle.read_layout(cx)`,
- keep component/advanced `ElementContext` surfaces on explicit `layout_query(cx)` reads unless a
  later cross-lane proof says otherwise.

Illustrative shape only:

```rust
let tip = cx.data().query(key, policy, fetch).read_layout(cx);
let label = tip.map_status(
    || "Loading...",
    |err| format!("Error: {err}"),
    |data| data.text.clone(),
);
```

### 4. Ecosystem adaptation

The long-term story must work for three kinds of consumers:

1. apps that depend on `fret`
2. reusable ecosystem crates that depend directly on `fret-selector` / `fret-query`
3. editor-grade applications that use `fret` but still rely on explicit shared-model lanes

Rules:

- default app sugar lives in `ecosystem/fret`,
- reusable crates may expose optional adapters, but must not be forced onto the `fret` facade,
- editor-grade apps keep an explicit advanced/shared-model path instead of being boxed into
  LocalState-only sugar.

## What success means

This workstream is successful when:

- the default app-facing dataflow language reads as one product surface rather than three partially
  aligned mini-languages,
- generic apps get shorter without losing ownership clarity,
- editor-grade apps can still explain their shared-model/runtime seams cleanly,
- reusable ecosystem crates can adapt without taking on the wrong dependency tier,
- router remains compatible but separate,
- and first-party docs/templates/examples/source-policy gates all teach the same default path.

## Execution order

1. Freeze scope, consumer matrix, and ownership rules.
2. Collapse the action surface first.
3. Collapse the LocalState-first selector surface second.
4. Collapse the query read-side surface third.
5. Audit reusable ecosystem adaptation and teaching surfaces.
6. Delete displaced spellings, refresh docs/templates/examples/gates, and keep router alignment as
   a compatibility check rather than a scope expansion.
