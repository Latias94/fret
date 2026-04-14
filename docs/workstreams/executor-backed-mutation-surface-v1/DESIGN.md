# Executor-backed Mutation Surface v1

Status: active execution lane
Last updated: 2026-04-14

Related:

- `M0_BASELINE_AUDIT_2026-04-14.md`
- `M1_CONTRACT_FREEZE_2026-04-14.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/audits/postman-like-api-client-first-contact.md`
- `docs/integrating-sqlite-and-sqlx.md`
- `docs/integrating-tokio-and-reqwest.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `ecosystem/fret-executor/src/lib.rs`
- `ecosystem/fret-query/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`

This lane exists because the repo already made one important split in principle:

- `fret-query` is the read-state / cache / invalidation engine,
- `fret-executor` is the portable execution + inbox substrate for background work,
- and app-owned writes are supposed to apply at a driver boundary.

That split is correct, but the public/default app-facing story is still incomplete.

The `api_workbench_lite` consumer probe proved the gap:

> a first-time user building a tool app can still reach for `cx.data().query_async(...)` for a
> click-driven, one-shot async submission, because the explicit mutation/submission lane is not yet
> productized on the default app surface.

The result is framework-level confusion:

- one click can silently replay a side-effecting request,
- query freshness/remount semantics leak into a submit-like flow,
- and the app author gets valid code with the wrong lifecycle model.

This is not a reason to reopen the closed LocalState/write-surface lanes blindly.
It is a reason to productize the missing executor-backed mutation lane.

## Why this is a new lane

This should not be forced back into older closed lanes:

- `dataflow-authoring-surface-fearless-refactor-v1` explicitly closed on selector/query read-side
  posture and kept query engine semantics out of scope.
- `action-write-surface-fearless-refactor-v1` explicitly froze the default LocalState write budget
  and should not be reopened from one new app probe alone.
- `view-locals-authoring-fearless-refactor-v1` closed on an organizational rule (`1-2 inline / 3+
  bundle`) rather than new write helpers.

Fresh evidence now exceeds those closeouts, but in a narrower direction:

- the missing app-facing mutation/submission contract,
- not a general `LocalState<T>` redesign,
- not a generic helper-growth pass on `cx.actions()`,
- and not a rewrite of query freshness semantics.

## Assumptions-first baseline

### 1) `fret-query` read semantics are fundamentally correct and should stay read-focused.

- Evidence:
  - `docs/integrating-tokio-and-reqwest.md`
  - `docs/integrating-sqlite-and-sqlx.md`
  - `ecosystem/fret-query/src/lib.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would incorrectly widen `fret-query` to paper over a missing mutation lane.

### 2) The current LocalState/write closeouts are not disproven by the `api_workbench_lite` probe.

- Evidence:
  - `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
  - `docs/workstreams/view-locals-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`
  - `apps/fret-examples/src/api_workbench_lite_demo.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - we would waste this lane reopening the wrong state APIs before fixing the async ownership bug.

### 3) `fret-executor` already owns the correct portability substrate for mutation work.

- Evidence:
  - `ecosystem/fret-executor/src/lib.rs`
  - `docs/integrating-sqlite-and-sqlx.md`
  - `docs/adr/0184-execution-and-concurrency-surface-v1.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - we would be forced into a wider runtime/execution redesign instead of a public-surface refactor.

### 4) The repo already has several special-case executor-backed flows, which is a smell that the shared mutation lane is missing.

- Evidence:
  - `ecosystem/fret-genui-core/src/executor.rs`
  - `ecosystem/fret-ui-shadcn/src/sonner.rs`
  - `docs/integrating-sqlite-and-sqlx.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would risk inventing a generalized surface without enough repeated demand.

### 5) Tool apps need an explicit submit lifecycle even for operations that are not semantic "writes".

- Evidence:
  - `docs/audits/postman-like-api-client-first-contact.md`
  - `apps/fret-examples/src/api_workbench_lite_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would overfit SQL/database mutation language and still fail API-client or export/import flows.

## Goals

1. Freeze the contract split between observed async reads and explicit async submissions.
2. Productize one executor-backed mutation/submission state machine instead of leaving the submit
   lane on inbox-only manual assembly.
3. Make render-time observation unable to replay a submit-like operation by accident.
4. Keep the driver-boundary apply contract intact:
   background work returns data-only results, UI-thread apply stays explicit.
5. Leave one default app-facing mutation/submission story on `fret`, backed by the correct
   ecosystem owner layers.
6. Prove the lane on real tool-app pressure rather than Todo-only pressure.

## Non-goals

- Reopening `LocalState<T>` storage architecture.
- Reopening the closed default `cx.actions()` write-budget lane.
- Rewriting `fret-query` freshness/remount/retry semantics to fit submit-like flows.
- Moving policy-heavy submit UX into `crates/fret-ui`.
- Solving toolbar/window command ownership in this same lane.
- Treating every async task as a cached query.

## Initial target surface

The exact names can still change, but the required contract shape is narrow and explicit.

### 1) `Query` remains the observed read-resource lane

Keep `fret-query` for:

- keyed async reads,
- cache retention,
- freshness,
- invalidation/refetch,
- render-observed `QueryState<T>`.

Do not teach it as the default lane for click-driven request submission, import/export, or other
explicit one-shot operations.

### 2) Add one shared mutation/submission state machine in the executor family

The shared mechanism should live in a new executor-family semantic crate rather than inside
`fret-query`, because the owner split is already:

- execution/inbox/cancellation substrate -> `fret-executor`
- read cache semantics -> `fret-query`

M1 freezes a stronger version of that split:

- keep `fret-executor` as the portable substrate,
- add the higher-level mutation/submission state machine in a new crate on top of that substrate
  (working name: `fret-mutation`),
- and keep `fret-query` read-only.

Target capabilities for the mutation mechanism:

- explicit submit only,
- no render-observation trigger,
- stable terminal state until reset or the next submit,
- cancellation,
- explicit concurrency policy,
- explicit retry policy,
- typed input/output,
- UI-thread apply through a model-backed state surface.

This lane uses the word "mutation" in the repo's existing sense:

- explicit async operation initiated by the app/user,
- not "HTTP verb must be POST/PUT/DELETE".

### 3) Add one app-facing mutation/submission facade on `fret`

The default app lane should not teach raw inbox wiring for the common case.

Target posture:

- app authors stay on `View` + `AppUi`,
- create/read the mutation handle from the grouped app-facing surface,
- trigger submit from explicit action/effect paths,
- and materialize results into ordinary app/local/shared state without reopening a second runtime.

The exact surface can still change, but the contract must make these truths obvious:

- creating/reading the handle does not start work,
- only an explicit submit starts work,
- observing state in `render()` cannot replay work,
- success/error remain reviewable terminal states,
- and query invalidation after success stays explicit.

### 4) Keep the owner split sharp

#### `ecosystem/fret-executor`

Owns:

- inbox/drainer/task/cancellation substrate,
- future-spawner integration,
- wake-at-driver-boundary delivery,
- and testable runtime-agnostic execution helpers.

#### New executor-family mutation crate

Owns:

- the mutation/submission execution state machine,
- typed mutation handles and terminal state semantics,
- cancellation/retry/concurrency policy for explicit submit work,
- and optional low-level UI adoption helpers if they prove necessary.

#### `ecosystem/fret`

Owns:

- the default app-facing helper surface,
- grouped `AppUi` / `UiCx` sugar,
- and first-contact authoring guidance.

#### `ecosystem/fret-query`

Keeps:

- read cache/state semantics,
- query invalidation/refetch,
- and optional integration points that mutation completion may call explicitly.

It should not become the owner of generic click-driven submission semantics.

## First proof surfaces

### Primary proof surface

- `apps/fret-examples/src/api_workbench_lite_demo.rs`

Why:

- it is the strongest current first-contact counterexample,
- it is not Todo-shaped,
- and it exercises explicit submit, terminal response state, command wiring, and diag evidence.

### Secondary proof surfaces

- `docs/integrating-sqlite-and-sqlx.md`
- `ecosystem/fret-genui-core/src/executor.rs`
- `ecosystem/fret-ui-shadcn/src/sonner.rs`

Why:

- SQL/database writes already teach the desired owner split in prose,
- GenUI already owns an action-executor concept,
- and Sonner already ships an executor-backed async recipe.

If the new surface is correct, these existing special cases should either align cleanly with it or
become clearly deliberate exceptions.

## Migration posture

This lane should leave the repo with three explicit outcomes:

1. query examples/docs teach read-state only,
2. submit/mutation examples teach the new executor-backed app-facing lane,
3. older special-case executor flows are either aligned, clearly classified as recipe-specific, or
   explicitly left advanced.

The lane should not close on "we fixed the demo."
It closes only when the contract is clear enough that a new app author no longer has to rediscover
the read-vs-submit split by failing with a real app.

## Success condition

This lane succeeds when the repo can answer one narrow question clearly:

> For a first-time Fret user, what is the correct app-facing way to run an explicit async
> submission, observe its status, and then invalidate or refresh read queries without abusing
> `query_async(...)`?

If the repo still answers that question with ad hoc inbox snippets or "just use query carefully",
the lane is not done.
