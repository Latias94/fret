# Executor-backed Mutation Surface v1 — TODO

## M0 — Baseline and scope freeze

- [x] Record the baseline audit from the `api_workbench_lite` probe.
- [x] Name the new lane as a follow-on instead of reopening the closed LocalState/write lanes.
- [ ] Add or update the hard-contract note/ADR for the read-vs-submit split on the default app lane.
- [ ] Decide the final shared term:
  - `mutation`,
  - `submission`,
  - or another explicit app-facing term,
  while keeping the current repo meaning ("explicit async operation", not only SQL/HTTP writes).

## M1 — Mechanism owner freeze (`fret-executor`)

- [ ] Decide whether the shared mechanism lives:
  - directly in `ecosystem/fret-executor`,
  - or in a tightly coupled companion module that still keeps `fret-query` read-only.
- [ ] Define the minimal shared state machine:
  - idle,
  - running,
  - success,
  - error,
  - reset.
- [ ] Define the minimal policy surface:
  - cancellation,
  - retry,
  - concurrency (latest wins / keep in flight / explicit parallel).
- [ ] Keep driver-boundary apply data-only and model-backed.

## M2 — App-facing surface (`fret`)

- [ ] Define the grouped app-facing helper surface on `AppUi` / `UiCx`.
- [ ] Keep the trigger path explicit:
  - creating/reading the handle does not start work,
  - only explicit submit starts work.
- [ ] Ensure the app-facing surface composes with existing `cx.actions()` ownership rather than
  silently creating a second default transaction family.
- [ ] Define the default query invalidation handoff after successful submit/mutation.

## M3 — Proof surfaces

- [ ] Migrate `apps/fret-examples/src/api_workbench_lite_demo.rs` to the new mutation/submission surface.
- [ ] Keep or strengthen the existing shell + response diag proof:
  - `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`
- [ ] Add one second non-Todo proof surface beyond the API workbench:
  - likely SQL/SQLite mutation docs/examples,
  - or another real explicit-submit app surface.
- [ ] Audit whether the existing GenUI and Sonner executor-backed flows should align to the shared
  contract or stay intentionally recipe-specific.

## M4 — Docs and teaching cleanup

- [ ] Update `docs/integrating-tokio-and-reqwest.md` so it teaches `query_async(...)` only for
  observed reads.
- [ ] Update `docs/integrating-sqlite-and-sqlx.md` to point at the shared mutation surface once it exists.
- [ ] Update `docs/crate-usage-guide.md` with the final owner split and default path.
- [ ] Add source-policy tests that prevent first-contact examples from drifting back to
  `query_async(...)` for explicit submit flows.

## Closure rule

- [ ] Do not close this lane until the repo has one default answer for explicit async submission on
  the app lane, plus at least one real proof surface and one regression artifact.
