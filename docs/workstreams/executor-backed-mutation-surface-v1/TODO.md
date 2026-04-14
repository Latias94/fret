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
- [ ] Define the minimal policy surface:
  - cancellation,
  - retry,
  - concurrency (latest wins / keep in flight / explicit parallel).
- [x] Keep driver-boundary apply data-only and model-backed.

## M2 — App-facing surface (`fret`)

- [x] Define the grouped app-facing helper surface on `AppUi` / `UiCx`.
- [x] Keep the trigger path explicit:
  - creating/reading the handle does not start work,
  - only explicit submit starts work.
- [ ] Decide whether the default app-facing naming should be:
  - `mutation*`,
  - `submit*`,
  - or a split with mechanism nouns on `mutation` and teaching nouns on `submit`.
- [x] Ensure the app-facing surface composes with existing `cx.actions()` ownership rather than
  silently creating a second default transaction family.
- [ ] Define the default query invalidation handoff after successful submit/mutation.

## M3 — Proof surfaces

- [x] Migrate `apps/fret-examples/src/api_workbench_lite_demo.rs` to the new mutation/submission surface.
- [x] Keep or strengthen the existing shell + response diag proof:
  - `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`
- [x] Add one second non-Todo proof surface beyond the API workbench:
  - likely SQL/SQLite mutation docs/examples,
  - or another real explicit-submit app surface.
- [ ] Audit whether the existing GenUI and Sonner executor-backed flows should align to the shared
  contract or stay intentionally recipe-specific.

## M4 — Docs and teaching cleanup

- [x] Update `docs/integrating-tokio-and-reqwest.md` so it teaches `query_async(...)` only for
  observed reads.
- [x] Update `docs/integrating-sqlite-and-sqlx.md` to point at the shared mutation surface once it exists.
- [x] Update `docs/crate-usage-guide.md` with the final owner split and default path.
- [x] Add source-policy tests that prevent first-contact examples from drifting back to
  `query_async(...)` for explicit submit flows.

## Closure rule

- [ ] Do not close this lane until the repo has one default answer for explicit async submission on
  the app lane, plus at least one real proof surface and one regression artifact.
