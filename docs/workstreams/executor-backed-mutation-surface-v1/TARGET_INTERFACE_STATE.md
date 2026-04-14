# Executor-backed Mutation Surface v1 — Target Interface State

Last updated: 2026-04-14

This file freezes the target owner split and teaching posture for the executor-backed mutation lane.

The exact API spellings do not need to be frozen yet.
The contract shape does.

## Target matrix

| Need | Default app lane | Explicit / advanced lane | Owner |
| --- | --- | --- | --- |
| Observed async reads | `cx.data().query*` + `QueryHandle::read_layout(cx)` | direct `fret-query` handle/state surface | `fret-query` engine + `fret` app sugar |
| Explicit async submit / mutation | one default app-facing mutation/submission surface; creating/reading the handle does not start work | raw inbox + executor assembly remains available | executor-family mutation crate + `fret` app sugar |
| Mutation execution substrate | hidden behind the default app-facing mutation surface | direct `Executors`, `Inbox`, `InboxDrainer`, `FutureSpawnerHandle` | `fret-executor` |
| Completion apply | model-backed mutation state and explicit UI-thread apply | manual driver-boundary model updates remain available | executor-family mutation crate + app-owned models |
| Retry / cancellation / concurrency | explicit mutation policy with user-invoked `retry_last(...)` replay, explicit cancellation, and separate concurrency semantics; no query-style automatic retry by default | direct executor/manual orchestration | executor-family mutation crate over `fret-executor` |
| Query refresh after mutation | explicit invalidation or explicit key/epoch change after success | raw `with_query_client(...)` still valid in pure app/driver code | `fret-query` + `fret` app helpers |
| Local UI materialization of terminal state | ordinary `LocalState<T>` / model updates above the mutation state machine, preferably via `update_after_mutation_completion(...)` and optionally gated by `take_mutation_completion(...)` for lower-level once-per-completion control flow | direct model choreography remains valid | existing state lanes; not reopened here |

## Teaching posture

### Default app lane

The default app lane should teach:

- `query*` for observed, cached reads,
- the new mutation/submission surface for explicit submit flows,
- explicit query invalidation after successful mutations/submissions,
- ordinary `LocalState<T>` or shared-model updates for the UI that consumes results.

It should not teach by default:

- `query_async(...)` for click-driven submit-like flows,
- raw inbox wiring for the common mutation case,
- or "be careful with `stale_time`" as the primary submit guidance.

### Advanced / editor-grade lane

The advanced lane must remain strong enough for:

- manual inbox/drainer orchestration,
- long-lived services and background workers,
- shared-model document/workspace graphs,
- multi-window coordination,
- and special executor-backed recipe flows.

Those advanced lanes should stay explicit rather than leaking back into first-contact examples.

### Reusable ecosystem lane

Reusable ecosystem crates should be able to:

- depend directly on `fret-executor` when they only need execution/inbox substrate,
- depend on the new executor-family mutation crate when they need explicit submit semantics without
  query caching,
- stay off `fret` unless they intentionally target the default app lane,
- and keep `fret-query` usage optional when they only need mutation/submit semantics.

## Feature topology target

The long-term `fret` feature split should become:

- `state-selector`
- `state-query`
- `state-mutation`

with:

- `state` eventually expanding to the three grouped state lanes once the mutation lane is stable,
- mutation-specific apps able to enable only `state-mutation` without adopting query semantics,
- and read-only apps able to stay on `state-query` without pulling submit helpers.

## Delete-ready / migration rules

Once the default mutation/submission surface lands:

- first-contact docs/examples should stop teaching `query_async(...)` for explicit submit flows,
- `api_workbench_lite` or an equivalent real tool-app proof should move to the new mutation lane,
- SQL/database write docs should point at the same app-facing mutation story rather than only a raw
  inbox recipe,
- older special-case executor flows must be classified as:
  - aligned with the shared mutation lane,
  - intentionally recipe-specific,
  - or still advanced/manual.

## Reopen rule

Do not reopen the closed state/write lanes from this folder unless fresh cross-surface evidence
shows that the new mutation lane still fails even after the submit-vs-query split is fixed.
