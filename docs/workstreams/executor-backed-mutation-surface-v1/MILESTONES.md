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
- M3 has one durable artifact-producing proof:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json --dir target/fret-diag-api-workbench-lite-mutation --session-auto --exit-after-run --launch cargo run -p fret-demo --bin api_workbench_lite_demo`
  - passed on 2026-04-14 and produced layout/screenshot/bundle artifacts under
    `target/fret-diag-api-workbench-lite-mutation/sessions/1776164998268-90687/`.

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
- at least one second consumer exists,
- and the lane has one durable gate or diag artifact that would fail if submit work regressed back
  into render-observed replay.

## M4 — Teaching surface cleanup

Exit when:

- docs/examples stop teaching `query_async(...)` for submit-like flows,
- the crate-usage guide names the final split clearly,
- and source-policy tests lock the first-contact path.
