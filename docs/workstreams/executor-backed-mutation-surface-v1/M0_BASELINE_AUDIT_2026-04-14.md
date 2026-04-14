# M0 Baseline Audit — 2026-04-14

Status: baseline audit for the executor-backed mutation surface lane

This note records why a new workstream is justified instead of reopening the older state/write
lanes.

## Baseline evidence

### 1) The `api_workbench_lite` consumer probe exceeded doc-level friction

The first-contact Postman-like probe did not fail because the request transport was missing.
It failed because the public/default async mental model let a click-driven submit flow be authored
as a render-observed query.

Observed outcome before the local demo fix:

- one click moved the UI into `Loading`,
- success responses arrived,
- but the render-observed query was still eligible for replay semantics that do not belong on a
  submit-like flow,
- producing duplicate requests and unstable terminal state.

Evidence:

- `docs/audits/postman-like-api-client-first-contact.md`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`

Conclusion:

- this is framework-surface confusion, not just missing example polish.

### 2) Older state/write closeouts are still directionally correct

The audit reviewed the closed LocalState/write lanes:

- `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`

Those closeouts freeze:

- one default write budget,
- one grouped-locals organization rule,
- and a no-new-helper verdict for the old `LocalState<T>` pressure set.

Current evidence does **not** yet prove those conclusions were wrong.
The `api_workbench_lite` probe hit a different problem first:

- async submit lifecycle ownership.

Conclusion:

- do not reopen the closed LocalState lanes from this evidence alone.

### 3) The read-vs-mutation split already exists in principle, but not yet in the default product surface

Current repo stance:

- `docs/integrating-tokio-and-reqwest.md` teaches `fret-query` for async read state.
- `docs/integrating-sqlite-and-sqlx.md` explicitly says `fret-query` is read-focused and
  mutations should run through `fret-executor`.
- `ecosystem/fret-executor/src/lib.rs` already provides the execution/inbox substrate.

Conclusion:

- the problem is not "we never designed a mutation lane".
- the problem is "the mutation lane is still substrate/manual, while the read lane is productized".

### 4) There are already repeated executor-backed special cases

Current repeated evidence:

- `docs/integrating-sqlite-and-sqlx.md` manually teaches mutation completion + invalidation.
- `ecosystem/fret-genui-core/src/executor.rs` ships a bounded app-owned action executor.
- `ecosystem/fret-ui-shadcn/src/sonner.rs` ships executor-backed async promise flows.

Conclusion:

- the repo already has multiple bounded submit/executor surfaces.
- that repetition is enough to justify a dedicated lane for a shared default mutation surface.

## Baseline verdict

Start a new narrow follow-on:

- `executor-backed-mutation-surface-v1`

Do **not** reopen broadly:

- `action-write-surface-fearless-refactor-v1`
- `view-locals-authoring-fearless-refactor-v1`
- `dataflow-authoring-surface-fearless-refactor-v1`

The new lane exists because fresh real-app evidence exceeded the older closeouts, but in a
different direction:

- productizing the explicit async submit/mutation lane.
