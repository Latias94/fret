# Closeout Audit — 2026-04-14

Status: closed closeout record

Related:

- `docs/workstreams/executor-backed-mutation-surface-v1/DESIGN.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/M0_BASELINE_AUDIT_2026-04-14.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/M1_CONTRACT_FREEZE_2026-04-14.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/TODO.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/MILESTONES.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/WORKSTREAM.json`
- `docs/adr/0326-query-vs-mutation-read-vs-submit-default-app-lane-v1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/integrating-tokio-and-reqwest.md`
- `docs/integrating-sqlite-and-sqlx.md`
- `docs/crate-usage-guide.md`
- `docs/audits/postman-like-api-client-first-contact.md`
- `docs/workstreams/genui-json-render-v1/genui-json-render-v1.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_INTENTIONAL_SURFACES.md`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret-mutation/src/lib.rs`
- `ecosystem/fret-genui-core/src/executor.rs`
- `ecosystem/fret-ui-shadcn/src/sonner.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/toast.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/render.rs`

## Verdict

This lane is now closed.

It leaves the repo with the intended default answer for explicit async submit work on the app lane:

- `fret-query` remains the observed read/cache lane,
- `fret-mutation` owns the shared executor-backed explicit submit state machine,
- `fret` exposes the grouped app-facing helpers and mutation-to-query handoff,
- and the real first-contact proof surface (`api_workbench_lite`) no longer has to abuse
  `query_async(...)` for click-driven request execution.

The remaining special-case executor-backed surfaces were audited during closeout and are now
classified as intentional recipe/app-owned exceptions rather than missing owners for the shared
mutation contract.

## Findings

### 1) The default app-lane contract is now real, not aspirational

The lane's original gap is closed in shipped code and teaching surfaces:

- `ecosystem/fret-mutation/src/lib.rs` owns the explicit submit state machine, completion tokens,
  and explicit replay via `retry_last(...)`.
- `ecosystem/fret/src/view.rs` owns the grouped app-facing helpers for:
  - `mutation_async(...)`,
  - `take_mutation_success(...)`,
  - `take_mutation_completion(...)`,
  - `update_after_mutation_completion(...)`,
  - and success-gated query invalidation.
- `apps/fret-examples/src/api_workbench_lite_demo.rs` proves the split on a real tool-app slice,
  including same-input retry and SQLite-backed history refresh.
- `docs/integrating-tokio-and-reqwest.md`,
  `docs/integrating-sqlite-and-sqlx.md`,
  and `docs/crate-usage-guide.md`
  now teach the same split and are locked by source-policy tests.

Conclusion:

- the repo now has one default answer for read-vs-submit on the default app lane.

### 2) `GenUiActionExecutorV1` is not a missing shared mutation owner

The GenUI executor stays intentionally app-/recipe-owned:

- its contract is action execution over JSON-backed app state and the host command/effect surface,
  not async submit state ownership,
- it is centered on:
  - standard JSON state actions,
  - portable effect emission,
  - confirm gating,
  - `onSuccess` / `onError` chaining,
  - and optional routing into stable `CommandId`s,
- and the GenUI workstream already documents that async execution, dialogs, permissions, and
  broader app-level policy remain app-owned.

The correct bridge point is therefore:

- a GenUI handler or dispatched app command may call into app-owned `fret-mutation` flows when the
  product needs authoritative submit state,
- but `GenUiActionExecutorV1` itself should not widen into a second mutation state machine.

Conclusion:

- GenUI stays a bounded action executor, not a second shared submit owner.

### 3) Sonner async promise helpers are recipe-owned feedback, not authoritative submit state

The Sonner async promise surface also stays intentionally recipe-specific:

- `ecosystem/fret-ui-shadcn/src/sonner.rs` uses `FutureSpawnerHandle` +
  `DispatcherHandle` only to run a future and mirror the result into toast chrome,
- completion is pushed as `ToastAsyncMsg::{Upsert,Dismiss}` through
  `ToastAsyncQueueHandle`,
- and those messages are drained later during the overlay render pass in
  `ecosystem/fret-ui-kit/src/window_overlays/render.rs`.

That surface does **not** own the things the shared mutation lane exists to own:

- typed app-domain input/output state,
- query invalidation handoff,
- retry identity,
- completion dedupe across rerenders,
- or reusable mutation concurrency/cancellation semantics on ordinary app data.

The correct relationship is:

- if the app's domain state matters, run the real work on `fret-mutation`,
- and optionally use Sonner to mirror loading/success/error into toast feedback.

Conclusion:

- Sonner async promise helpers are a recipe-owned feedback wrapper, not a shared mutation lane.

### 4) The older closeout lanes stay closed

This lane closes without reopening:

- `dataflow-authoring-surface-fearless-refactor-v1`,
- `action-write-surface-fearless-refactor-v1`,
- `view-locals-authoring-fearless-refactor-v1`.

The `api_workbench_lite` probe did not prove those old lanes wrong.
It proved the repo was missing the explicit submit surface between query reads and raw executor
substrate.

Conclusion:

- the correct fix was a narrow submit-lane productization, not another broad state-surface rewrite.

## Decision from this audit

Treat `executor-backed-mutation-surface-v1` as:

- closed for the default app-lane async submit/mutation productization goal,
- the latest source of truth for why `query` stays read-only and `mutation` owns explicit submit,
- and historical closeout evidence unless future consumer pressure disproves the shipped split.

## Gates used for closeout

- `cargo nextest run -p fret docs_lock_query_reads_vs_mutation_submit_story`
- `cargo nextest run -p fret-genui-core`
- `cargo nextest run -p fret-ui-shadcn toast_promise_handle_unwrap_reports_missing_spawner toast_promise_handle_unwrap_reports_missing_dispatcher toast_promise_handle_unwrap_resolves_ok_when_spawner_and_dispatcher_present`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/executor-backed-mutation-surface-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`

## Follow-on policy

Do not reopen this lane for:

- recipe-owned toast feedback helpers,
- GenUI action-executor growth,
- or another broad state-surface rewrite debate.

If future work is needed, start a narrower follow-on such as:

1. a first-party `mutation + toast feedback` cookbook/example lane,
2. a GenUI example that dispatches into app-owned mutation handlers explicitly,
3. or a fresh consumer audit that proves the current default submit surface still fails on a real
   non-Todo app after this closeout.
