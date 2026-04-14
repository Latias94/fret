# Executor-backed Mutation Surface v1 — TODO

## M0 — Baseline and scope freeze

- [x] Record the baseline audit from the `api_workbench_lite` probe.
- [x] Name the new lane as a follow-on instead of reopening the closed LocalState/write lanes.
- [x] Add or update the hard-contract note/ADR for the read-vs-submit split on the default app lane.
- [x] Decide the final shared term:
  - keep `mutation` for the mechanism/API surface,
  - allow "explicit submit flow" as teaching vocabulary,
  - and do not reopen a repo-wide rename unless the mechanism contract itself changes.

## M1 — Mechanism owner freeze (`fret-executor`)

- [x] Freeze the owner split:
  - keep `ecosystem/fret-executor` as the portable execution substrate,
  - add the shared mutation/submission state machine in a new executor-family semantic crate,
  - keep `fret-query` read-only.
- [x] Freeze the first feature topology:
  - add `state-mutation` on `fret`,
  - keep it separate from `state-query`,
  - and widen `state` only after the mutation lane proves itself.
- [x] Define the minimal shared state machine:
  - idle,
  - running,
  - success,
  - error,
  - reset.
- [x] Define the minimal policy surface:
  - cancellation,
  - retry as explicit `retry_last(...)` replay of the last stored submit input,
  - concurrency (latest wins / keep in flight / explicit parallel),
  - while keeping query-style automatic retry out of the default mutation lane.
- [x] Keep driver-boundary apply data-only and model-backed.

## M2 — App-facing surface (`fret`)

- [x] Define the grouped app-facing helper surface on `AppUi` / `UiCx`.
- [x] Keep the trigger path explicit:
  - creating/reading the handle does not start work,
  - only explicit submit starts work.
- [x] Decide whether the default app-facing naming should be:
  - mechanism nouns stay on `mutation*`,
  - teaching copy may say "submit" / "explicit submit flow",
  - and the repo does not pay a wide `submit*` rename churn in v1.
- [x] Ensure the app-facing surface composes with existing `cx.actions()` ownership rather than
  silently creating a second default transaction family.
- [x] Define the default query invalidation handoff after successful submit/mutation.

## M3 — Proof surfaces

- [x] Migrate `apps/fret-examples/src/api_workbench_lite_demo.rs` to the new mutation/submission surface.
- [x] Keep or strengthen the existing shell + response diag proof:
  - `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`
- [x] Add one second non-Todo proof surface beyond the API workbench:
  - likely SQL/SQLite mutation docs/examples,
  - or another real explicit-submit app surface.
- [x] Audit whether the existing GenUI and Sonner executor-backed flows should align to the shared
  contract or stay intentionally recipe-specific.

## M4 — Docs and teaching cleanup

- [x] Update `docs/integrating-tokio-and-reqwest.md` so it teaches `query_async(...)` only for
  observed reads.
- [x] Update `docs/integrating-sqlite-and-sqlx.md` to point at the shared mutation surface once it exists.
- [x] Update `docs/crate-usage-guide.md` with the final owner split and default path.
- [x] Add source-policy tests that prevent first-contact examples from drifting back to
  `query_async(...)` for explicit submit flows.

## Closure rule

- [x] Do not close this lane until the repo has one default answer for explicit async submission on
  the app lane, plus at least one real proof surface and one regression artifact.
