# Executor-backed Mutation Surface v1 — Milestones

## Current progress (2026-04-14)

- M1 implementation slice is now real code:
  - `ecosystem/fret-mutation` owns the shared mutation state machine over `fret-executor`,
  - `fret` exposes `state-mutation` separately from `state-query`,
  - and driver-boundary completion applies through `ModelId` + UI-thread `update_any(...)` rather
    than capturing main-thread model handles across the inbox boundary.
- M2 now has a first app-facing proof:
  - `cx.data().mutation_async(...)` / `mutation_async_local(...)`,
  - `MutationHandle::submit(...)`,
  - and `api_workbench_lite` wiring through the existing `cx.actions().models::<...>(...)` path.
- M1 now freezes retry semantics on the mutation lane:
  - `ecosystem/fret-mutation/src/lib.rs` adds `MutationHandle::retry_last(...)` and
    `retry_last_action(...)` as the explicit replay path for the last stored submit input,
  - the retry surface stays separate from query freshness semantics,
  - and v1 still does **not** add query-style automatic retry scheduling to the default mutation
    lane.
- M3 has one durable artifact-producing proof:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json --dir target/fret-diag-api-workbench-lite-mutation --session-auto --exit-after-run --launch cargo run -p fret-demo --bin api_workbench_lite_demo`
  - passed on 2026-04-14 and produced layout/screenshot/bundle artifacts under
    `target/fret-diag-api-workbench-lite-mutation/sessions/1776164998268-90687/`.
- M4 teaching cleanup is now locked on the default app lane:
  - `docs/integrating-tokio-and-reqwest.md` stays read-only and points explicit submit work to
    `state-mutation`,
  - `docs/integrating-sqlite-and-sqlx.md` now teaches `cx.data().mutation_async(...)` +
    `handle.submit(...)` +
    `cx.data().invalidate_query_namespace_after_mutation_success(...)` as the default
    mutation-to-query handoff,
  - `docs/crate-usage-guide.md` names `fret-mutation` as the shared submit lane,
  - and `ecosystem/fret/src/lib.rs` now carries source-policy assertions that would fail if the
    first-contact docs drift back to `query_async(...)` for submit flows.
- M2 now has a real default invalidation handoff on the framework surface:
  - `ecosystem/fret/src/view.rs` adds `cx.data().take_mutation_success(...)` for one-shot
    completion gating on the app lane,
  - now also adds `cx.data().take_mutation_completion(...)` for one-shot terminal
    success-or-error apply when app code needs to materialize the latest mutation result into
    ordinary local/shared UI state,
  - and now adds `cx.data().update_after_mutation_completion(...)` so app code can project a fresh
    terminal mutation result into ordinary `LocalState<T>` / shared models without reopening
    `read_layout(cx)` + manual redraw glue at each call site,
  - adds `cx.data().invalidate_query_after_mutation_success(...)` and
    `cx.data().invalidate_query_namespace_after_mutation_success(...)` for explicit read-lane
    refresh after one completed mutation success,
  - `ecosystem/fret-mutation/src/lib.rs` now exposes a handle-owned success token so that this
    one-shot handoff no longer depends on `MutationState.updated_at` wall-clock bookkeeping,
  - and the app lane no longer needs ad hoc `root_state(...)` bookkeeping just to avoid replaying
    invalidation on every render after success.
- M3 now has a second real proof surface on the same product probe:
  - `apps/fret-examples/src/api_workbench_lite_demo.rs` keeps HTTP send on one mutation lane,
  - adds SQLite-backed request history reads on `cx.data().query_async(...)`,
  - adds explicit history writes on `cx.data().mutation_async(...)`,
  - now exposes an explicit `Retry Last Request` command/button that replays the last request and
    persists another history row through the same mutation lane,
  - removes demo-local request sequence bookkeeping in favor of the shared
    `take_mutation_completion(...)` once-per-completion app helper,
  - now further collapses the terminal response projection to
    `cx.data().update_after_mutation_completion(...)` instead of app-owned
    `read_layout(cx)` + manual redraw glue,
  - and now invalidates the saved-history query namespace through the shared
    `invalidate_query_namespace_after_mutation_success(...)` helper instead of local render-owned
    dedupe glue.
- The existing `api-workbench-lite` diag script now has a full SQLite-backed artifact proof:
  - the first run under
    `target/fret-diag-api-workbench-lite-sqlite-history/sessions/1776168169993-16022/`
    reached `api-workbench-lite.history.row.1` and produced layout + bundle artifacts, but timed
    out during `capture_screenshot`,
  - a hot rerun under
    `target/fret-diag-api-workbench-lite-sqlite-history-rerun/sessions/1776168778413-22114/`
    passed and produced layout/screenshot/bundle artifacts on the same SQLite query lane,
  - so the lane now has the stronger dataflow proof plus a passing screenshot artifact, while the
    earlier timeout is treated as diagnostics timing noise rather than a framework-design failure.
- M0 / M2 / M4 contract freeze is now explicit at the ADR layer:
  - `docs/adr/0326-query-vs-mutation-read-vs-submit-default-app-lane-v1.md` freezes the default
    app-lane split between observed reads and explicit submit flows,
  - keeps `mutation` as the mechanism/API term while allowing "explicit submit flow" as teaching
    vocabulary,
  - locks mutation-to-query refresh as explicit invalidation rather than widened query freshness,
  - and locks same-input retry/completion dedupe onto handle-owned completion identity instead of
    app-local sequence bookkeeping.
- M3 closeout audit now classifies the remaining executor-backed side surfaces:
  - `ecosystem/fret-genui-core/src/executor.rs` stays an app-/recipe-owned action executor and
    does not widen into a second shared mutation state machine,
  - `ecosystem/fret-ui-shadcn/src/sonner.rs` keeps async promise helpers as recipe-owned toast
    feedback over `ToastAsyncQueueHandle` rather than as authoritative app-domain mutation state,
  - and `docs/workstreams/executor-backed-mutation-surface-v1/CLOSEOUT_AUDIT_2026-04-14.md`
    closes the lane on the verdict that these are deliberate exceptions, not missing owners for
    the shared default submit contract.

## M0 — Baseline audit and scope freeze

Exit when:

- the lane explicitly records why it exists,
- the baseline evidence names the `api_workbench_lite` failure mode,
- and the older closed lanes are referenced as inherited constraints rather than silently reopened.

## M1 — Mechanism contract freeze

Exit when:

- the repo has one explicit owner decision for the shared mutation mechanism,
- the owner split keeps `fret-executor` as substrate and moves higher-level mutation semantics into
  a dedicated executor-family semantic crate,
- the intended `fret` feature topology (`state-mutation` separate from `state-query`) is written
  down,
- the minimal state machine and policy budget are written down,
- and the lane has not widened `fret-query` beyond read-state semantics.

## M2 — App-facing mutation/submission helper

Exit when:

- the default `fret` app lane has one explicit mutation/submission story,
- observing the handle in `render()` cannot trigger/replay work,
- and the trigger path composes with the current action ownership model.

## M3 — Real proof surfaces

Exit when:

- `api_workbench_lite` proves the new contract on a non-Todo tool app,
- at least one second real proof surface exists beyond the original HTTP submit-only slice,
- and the lane has one durable gate or diag artifact that would fail if submit work regressed back
  into render-observed replay.

## M4 — Teaching surface cleanup

Exit when:

- docs/examples stop teaching `query_async(...)` for submit-like flows,
- the crate-usage guide names the final split clearly,
- and source-policy tests lock the first-contact path.
