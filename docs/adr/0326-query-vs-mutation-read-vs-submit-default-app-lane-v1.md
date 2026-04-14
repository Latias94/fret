# ADR 0326: Query vs Mutation Read-vs-Submit Contract on the Default App Lane (v1)

Status: Accepted

## Context

ADR 0175 already locks Fret's driver-boundary apply model:

- background work returns data-only results,
- inbox delivery and wake scheduling are explicit,
- and UI-thread apply stays main-thread owned.

ADR 0225 then narrows the query lane further:

- `stale_time` means freshness, not polling,
- query refetch triggers are explicit and debuggable,
- and `fret-query` is the ecosystem-owned async read/cache helper rather than a kernel contract.

ADR 0319 also locks the broader public state-lane posture:

- `View` + `AppUi` + grouped helpers are the default app lane,
- explicit raw-model and lower-level identity lanes stay separate,
- and first-contact materials should not blur those ownership boundaries.

The repo has since implemented a matching submit lane:

- `ecosystem/fret-executor` keeps the portable execution substrate,
- `ecosystem/fret-mutation` owns the executor-backed explicit mutation/submit state machine,
- and `ecosystem/fret/src/view.rs` now exposes grouped helpers such as
  `cx.data().mutation_async(...)`,
  `cx.data().take_mutation_success(...)`,
  `cx.data().take_mutation_completion(...)`,
  `cx.data().update_after_mutation_completion(...)`,
  and success-gated query invalidation helpers.

However, the consumer-facing contract was still too soft.

The first-contact `api_workbench_lite` probe exposed the exact failure mode:

- a first-time user can still model `Send Request` as `query_async(...)` because the read lane is
  more discoverable than the submit lane,
- query freshness/remount semantics then leak into a discrete user-triggered run,
- same-input retry can force app-local sequence bookkeeping if the completion identity contract is
  not explicit,
- and naming debates can reopen a repo-wide churn loop even though the mechanism split is already
  correct.

Fret needs one hard-contract answer for the default app lane:

- what belongs on the observed read lane,
- what belongs on the explicit submit lane,
- how mutation completion hands off to query refresh,
- and whether the repo should rename the mechanism surface away from `mutation`.

## Goals

1. Freeze one default answer for observed async reads vs explicit submit flows on the `fret`
   golden path.
2. Keep `fret-query` narrow and debuggable instead of widening it to hide submit semantics.
3. Freeze the naming posture without paying repo-wide rename churn when the mechanism split is
   already correct.
4. Lock the handle-owned completion identity story so app authors do not reinvent local dedupe
   tokens.

## Non-goals

- Making every async task a mutation.
- Replacing explicit query refetch/invalidate for ordinary read resources.
- Adding query-style automatic retry/backoff as the default mutation behavior.
- Renaming the existing repo-wide mutation surface to `submit_*` in one sweep.
- Moving submit/mutation policy into `crates/fret-ui`.

## Decision

### 1) `fret-query` stays the observed read lane

Use `fret-query` when the product semantic is a keyed resource observed over time:

- cached remote reads,
- SQLite/history/list/detail reads,
- settings/resource loads,
- or other data where cache, freshness, invalidation, and refetch are the primary concerns.

The default app helpers for that lane remain:

- `cx.data().query_async(...)`
- `cx.data().query_async_local(...)`
- plus explicit invalidation/refetch on the query client or grouped `cx.data()` helpers.

First-contact docs/examples must not teach `query_async(...)` as the default lane for click-driven
submit flows such as Save, Run, Sync, Import, Export, or API-client request execution.

### 2) `fret-mutation` is the default explicit submit lane

Use `fret-mutation` when the product semantic is one discrete user- or app-initiated submission
with terminal running/success/error state, even if the external operation is not semantically a
"write" in HTTP or SQL terms.

Examples:

- Save / Delete / Sync,
- Import / Export,
- Run / Rebuild / Execute,
- and a Postman-like API client's `Send Request`, even when the request method is `GET`.

On the default app lane:

- create handles with `cx.data().mutation_async(...)` or `cx.data().mutation_async_local(...)`,
- observe terminal/inflight state with `handle.read_layout(cx)`,
- and start work only with `handle.submit(...)`, `handle.submit_action(...)`, or
  `handle.retry_last(...)`.

Creating or observing a mutation handle must not start or replay work.

### 2.1) Choose by lifecycle ownership, not by HTTP/SQL verb

This split is about lifecycle semantics, not transport vocabulary.

Use query when:

- the UI owns a stable keyed resource,
- freshness and invalidation are meaningful product concepts,
- and a user action is merely an explicit refetch of that observed resource.

Use mutation when:

- the UI owns a discrete terminal run,
- retry/cancel/concurrency semantics belong to that run,
- and the user expects an explicit submission lifecycle rather than a cached resource lifecycle.

Therefore, an API-client request can belong on the mutation lane even when it is a semantic read,
while a "Refresh" button can still belong to the query lane when it explicitly refetches an already
observed resource.

### 3) Default retry on the submit lane is explicit replay

The default mutation retry contract is explicit replay of the last stored input:

- `handle.retry_last(...)`
- `retry_last_action(...)`

Query-style automatic retry/backoff belongs to the query lane only.
The repo must not hide submit retry semantics behind query freshness or remount behavior.

Cancellation and concurrency policy for explicit submit work also belong to the mutation lane, not
to the query cache contract.

### 4) Mutation-to-query handoff stays explicit

When a successful mutation changes data that query-backed reads depend on, app code must invalidate
the affected keys or namespaces explicitly.

On the default app lane, prefer:

- `cx.data().invalidate_query_after_mutation_success(...)`
- `cx.data().invalidate_query_namespace_after_mutation_success(...)`

Pure app/driver code may continue to use `fret::query::with_query_client(...)` directly.

The repo must not widen query freshness/remount semantics to compensate for missing submit
semantics.

### 5) Completion identity is owned by the mutation handle

Same-input retry and repeated terminal-state observation must be distinguished by handle-owned
completion identity, not by app-local wall-clock bookkeeping or request sequence counters.

The shared mechanism surface is:

- `MutationHandle::completion_token()`
- `MutationHandle::success_token()`

The default app-lane helpers built on top of that contract are:

- `cx.data().take_mutation_success(...)`
- `cx.data().take_mutation_completion(...)`
- `cx.data().update_after_mutation_completion(...)`

App code may still project results into `LocalState<T>` or shared models, but the once-per-success
or once-per-completion gate should come from the shared mutation handle rather than ad hoc
`root_state(...)` or local request-sequence glue.

### 6) Naming posture is frozen: mechanism uses `mutation`, teaching may say `explicit submit flow`

The repo keeps `mutation` as the mechanism/API noun:

- crate and feature names,
- handle/type names,
- grouped app helpers,
- and docs that name the actual API surface.

Teaching materials may describe this lane as the "explicit submit flow" when that wording makes the
read-vs-submit distinction clearer for users.

The repo should not perform a wide rename from `mutation_*` to `submit_*` unless a future contract
change proves that the mechanism term itself is wrong, not merely that beginner-facing prose can be
clearer.

### 7) Ownership remains layered

- `ecosystem/fret-executor`
  - owns the runtime-agnostic execution substrate, inbox delivery, wake/cancellation plumbing, and
    low-level task helpers.
- `ecosystem/fret-mutation`
  - owns the shared explicit submit state machine and handle semantics.
- `ecosystem/fret-query`
  - owns observed read cache/invalidation/refetch semantics only.
- `ecosystem/fret`
  - owns grouped `AppUi` / `UiCx` sugar and the first-contact app-facing teaching story.

This ADR does not move submit policy into `crates/fret-ui`.

## Consequences

### Positive

- Fret now has one default answer for explicit submit flows across SQL, HTTP workbench, and other
  tool-app surfaces.
- Query semantics stay narrow, explainable, and compatible with ADR 0225.
- Same-input retries no longer force every app to invent local completion-deduplication state.
- Docs can use "submit" as teaching vocabulary without reopening a repo-wide rename churn cycle.

### Costs / follow-ons

- Older docs/examples that still blur read and submit semantics must migrate.
- Recipe-specific executor-backed flows still need classification:
  - align to the shared mutation contract,
  - or stay intentionally specialized with explicit rationale.
- Future API cleanup should prefer fixing examples and teaching surfaces before widening core
  mechanism terms again.

## References

- ADR 0175: `docs/adr/0175-driver-boundary-apply-and-ui-thread-state-v1.md`
- ADR 0225: `docs/adr/0225-query-lifecycle-and-cache-semantics-v1.md`
- ADR 0319: `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- Workstream:
  `docs/workstreams/executor-backed-mutation-surface-v1/DESIGN.md`
- Proof surface:
  `apps/fret-examples/src/api_workbench_lite_demo.rs`
- App-facing helpers:
  `ecosystem/fret/src/view.rs`
- Shared mutation mechanism:
  `ecosystem/fret-mutation/src/lib.rs`
