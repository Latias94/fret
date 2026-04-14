# Mutation Toast Feedback Golden Path v1

Status: closed historical design note
Last updated: 2026-04-15

Status note (2026-04-15): this document remains useful for the lane-opening rationale and owner
split, but the shipped verdict now lives in `CLOSEOUT_AUDIT_2026-04-15.md` and `WORKSTREAM.json`.
Read the execution framing below as historical lane setup rather than an active queue.

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `CLOSEOUT_AUDIT_2026-04-15.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/CLOSEOUT_AUDIT_2026-04-14.md`
- `docs/adr/0326-query-vs-mutation-read-vs-submit-default-app-lane-v1.md`
- `apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `docs/crate-usage-guide.md`
- `docs/integrating-sqlite-and-sqlx.md`

This lane exists because the closed executor-backed mutation lane already froze the owner split, but
the repo still taught that split through two separate partial surfaces:

- `api_workbench_lite` proves the real app-facing mutation owner,
- `toast_basics` proves the Sonner host integration,
- but nothing small and copyable shows how those two surfaces compose without blurring ownership.

That gap is exactly where first-contact users can still drift into the wrong answer:

- treating `toast_promise_async*` as the mutation owner,
- treating Sonner success/error chrome as authoritative app state,
- or skipping the shared mutation helpers and dropping to raw executor/inbox plumbing for an
  ordinary submit flow.

## Why this is a new lane

`executor-backed-mutation-surface-v1` is closed and should stay closed.

Reopening it for cookbook/docs work would blur two different questions:

1. which shared owner owns explicit submit state,
2. and how the repo should teach that owner through first-party examples and diagnostics.

This lane owns only the second question.

## Assumptions-first baseline

### 1) The shared owner split is already correct.

- Evidence:
  - `docs/adr/0326-query-vs-mutation-read-vs-submit-default-app-lane-v1.md`
  - `docs/workstreams/executor-backed-mutation-surface-v1/CLOSEOUT_AUDIT_2026-04-14.md`
  - `apps/fret-examples/src/api_workbench_lite_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would have to stop and hand back to a new contract/mechanism follow-on instead of a
    teaching-surface lane.

### 2) The missing piece is primarily a first-party authoring surface, not a new runtime helper.

- Evidence:
  - `apps/fret-cookbook/examples/toast_basics.rs`
  - `docs/crate-usage-guide.md`
  - `docs/integrating-sqlite-and-sqlx.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the repo would need another mechanism or facade change instead of a cookbook/example/doc lock.

### 3) Sonner should remain feedback-only in this lane.

- Evidence:
  - `ecosystem/fret-ui-shadcn/src/sonner.rs`
  - `docs/workstreams/executor-backed-mutation-surface-v1/CLOSEOUT_AUDIT_2026-04-14.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would accidentally reopen the closed shared-owner debate.

## Goals

1. Leave one official copyable example that combines:
   - `cx.data().mutation_async(...)`,
   - `handle.submit(...)`,
   - `handle.retry_last(...)`,
   - `cx.data().update_after_mutation_completion(...)`,
   - and Sonner success/error message helpers.
2. Keep the app-owned projection explicit enough that users can see what state remains after the
   toast disappears.
3. Leave a screenshot-capable diag smoke so the teaching surface has bounded visual evidence.
4. Make the path discoverable from cookbook/docs entrypoints instead of relying on audit notes.

## Non-goals

- Widening `fret-mutation` into a toast-aware runtime surface
- Treating `toast_promise_async*` as the default submit lane
- Replacing the real `api_workbench_lite` proof surface
- Reopening query-vs-mutation naming or ownership
- Teaching raw `fret-executor` / inbox plumbing as the default app answer

## Owner split

### `apps/fret-cookbook`

Owns:

- the smallest copyable example,
- feature wiring for the example,
- cookbook/source-policy gates,
- and cookbook index/readme discoverability.

### `tools/diag-scripts/*`

Owns:

- one screenshot smoke for the new example,
- and the bounded bundle artifact path for review/regression.

### Docs (`docs/*`)

Own:

- pointing users from the general mutation docs to the new example,
- and keeping the closed executor-backed lane referenced as background, not as the first-open
  teaching surface.

## Intended shipped surface

The repo should be able to teach this sentence without caveats:

> For an ordinary async submit flow on the default app lane, let `fret-mutation` own the
> authoritative lifecycle, project terminal state into app-owned locals/models exactly once, and
> mirror the same completion into Sonner only as feedback.

The concrete first-party path for that sentence is:

- `apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs`
- `tools/diag-scripts/cookbook/mutation-toast-feedback-basics/cookbook-mutation-toast-feedback-basics-smoke.json`

## Success condition

This lane succeeds when the repo no longer needs a verbal explanation to answer:

> "How should I wire a Postman-like save/run action so retries and app state stay authoritative,
> but the user still gets toast feedback?"

The answer should simply be:

- copy the cookbook example,
- follow the docs links,
- and use the diag smoke when visual parity drifts.
