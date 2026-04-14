# M1 Contract Freeze — 2026-04-14

Status: M1 mechanism/owner freeze for the executor-backed mutation surface lane

This note resolves the first hard architecture question for this lane:

- where the shared mutation/submission mechanism should live,
- how that owner relates to `fret-executor`, `fret-query`, and `fret`,
- and which feature topology keeps the long-term surface clean.

## Decision

### 1) Keep `fret-executor` as the portable execution substrate, not the full mutation semantic surface

`ecosystem/fret-executor` remains responsible for:

- `DispatcherHandle`-backed task execution helpers,
- `Inbox` / `InboxDrainer`,
- cancellation tokens and task handles,
- future-spawner integration,
- wake-at-driver-boundary delivery.

Do **not** grow `fret-executor` into the user-facing mutation state machine crate root.

Reason:

- it is currently the low-level execution substrate,
- it has no `ui` feature and no app-facing grouped helper story,
- and making it own higher-level submit state directly would blur the same substrate-vs-semantic
  boundary that `fret-query` already separated successfully for reads.

### 2) Put the shared mutation/submission state machine in a new executor-family crate

The shared mutation/submission mechanism should live in a new ecosystem crate that depends on:

- `fret-executor`
- `fret-runtime`
- optional `fret-ui` only when low-level `ElementContext` helpers are genuinely useful

Working name:

- `ecosystem/fret-mutation`

This lane is freezing the owner shape, not the final published name.
If a better public name appears during M2, the owner split below should still remain the same.

Why this is the most correct placement:

- it keeps `fret-executor` lean as the reusable execution substrate,
- it mirrors the existing `fret-query` layering shape:
  semantic async state crate on top of executor substrate,
- it avoids forcing query/cache semantics onto submit flows,
- it allows a dedicated feature gate and docs story on `fret`,
- and it keeps future mutation-specific policies reviewable without turning the executor crate into
  a mixed abstraction bag.

### 3) `fret-query` remains read-only

`ecosystem/fret-query` stays responsible for:

- observed async reads,
- keyed cache/state retention,
- stale/remount semantics,
- retry for resource fetching,
- invalidation/refetch.

This lane does **not** repurpose `QueryHandle<T>` or `QueryPolicy` into the default submit lane.

### 4) `fret` should expose a separate grouped app-facing mutation lane

The default app-facing surface should become:

- `cx.data().query*` for observed reads,
- one new grouped mutation/submission lane for explicit submit work.

Recommended feature topology:

- keep `state-selector` as-is,
- keep `state-query` as-is,
- add `state-mutation`,
- and later let `state = ["state-selector", "state-query", "state-mutation"]` once the mutation
  lane is stable enough to belong to the default grouped state bundle.

This keeps the feature graph honest:

- apps that only need cached reads do not pull mutation surface by accident,
- apps that only need submit/mutation helpers do not need to depend on query semantics,
- and `fret` can document the three grouped state lanes separately.

## Why not put the semantic surface directly in `fret-executor`?

This was the main open question entering M1.

It is rejected for four reasons:

### 1) The crate already reads as substrate, not as app-facing state semantics

Current `fret-executor` shape is:

- one `src/lib.rs`,
- portable task/inbox/cancellation helpers,
- optional runtime adapters only.

That is the right place for the execution substrate.
It is the wrong place to make the top-level crate meaning drift toward `MutationHandle`,
`MutationState`, or grouped UI adoption helpers.

### 2) The repo already proved the better layering pattern with `fret-query`

Today the architecture already says:

- executor substrate in `fret-executor`,
- semantic read state in `fret-query`,
- app sugar in `fret`.

The correct mutation answer should mirror that layering rather than cutting across it.

### 3) Feature hygiene is cleaner with a dedicated semantic crate

If mutation semantics live directly in `fret-executor`, then one crate would need to carry both:

- low-level inbox/spawner substrate,
- higher-level state machine semantics,
- and possibly future `ui` adoption features.

A dedicated semantic crate keeps those axes separate.

### 4) Release closure and future deletions stay easier

If the first shipped mutation API needs to evolve aggressively, a dedicated semantic crate is
easier to rename, reshape, or narrow without destabilizing every executor substrate consumer.

## Owner split after M1

### `ecosystem/fret-executor`

Owns:

- portable execution substrate,
- wake/inbox/task/cancellation primitives,
- runtime adapters.

### New executor-family mutation crate (`ecosystem/fret-mutation`, working name)

Owns:

- shared mutation/submission state machine,
- typed input/output contract,
- explicit concurrency / retry / reset policy,
- model-backed terminal state semantics,
- optional low-level UI adoption helpers if they earn their keep.

### `ecosystem/fret`

Owns:

- grouped app-facing mutation helper surface,
- default app-lane naming/teaching posture,
- documentation and first-contact story,
- integration with query invalidation helpers on success.

## First implementation consequences

The first implementation slice should now assume:

1. do **not** add `MutationState` directly to `fret-executor` root APIs,
2. create a dedicated semantic crate in the executor family,
3. keep the first mutation crate surface narrow:
   - `MutationStatus`
   - `MutationState<TIn, TOut>`
   - `MutationPolicy`
   - `MutationHandle`
   - explicit `submit`, `reset`, `cancel`
4. add the grouped app helper on `fret` only after the mutation crate surface exists,
5. migrate `api_workbench_lite` against that new surface before widening docs.

## Evidence

- `ecosystem/fret-executor/Cargo.toml`
- `ecosystem/fret-executor/src/lib.rs`
- `ecosystem/fret-query/Cargo.toml`
- `ecosystem/fret-query/src/lib.rs`
- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `docs/crate-usage-guide.md`
- `docs/integrating-sqlite-and-sqlx.md`
- `docs/integrating-tokio-and-reqwest.md`
- `docs/adr/0184-execution-and-concurrency-surface-v1.md`
