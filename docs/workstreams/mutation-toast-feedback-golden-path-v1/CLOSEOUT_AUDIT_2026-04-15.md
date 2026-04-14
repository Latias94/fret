# Closeout Audit — 2026-04-15

Status: closed closeout record

Related:

- `docs/workstreams/mutation-toast-feedback-golden-path-v1/DESIGN.md`
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/TODO.md`
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/MILESTONES.md`
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/WORKSTREAM.json`
- `docs/workstreams/executor-backed-mutation-surface-v1/CLOSEOUT_AUDIT_2026-04-14.md`
- `docs/adr/0326-query-vs-mutation-read-vs-submit-default-app-lane-v1.md`
- `apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `tools/diag-scripts/cookbook/mutation-toast-feedback-basics/cookbook-mutation-toast-feedback-basics-smoke.json`
- `docs/crate-usage-guide.md`
- `docs/integrating-sqlite-and-sqlx.md`

## Verdict

This lane is now closed.

It leaves the repo with a bounded first-party teaching answer for the closed submit-owner contract:

- `fret-mutation` stays the authoritative explicit submit owner,
- app-owned locals/models stay the durable projection lane,
- Sonner stays recipe-owned success/error feedback,
- and the cookbook/docs/diag surfaces now teach that split as one copyable path instead of two
  disconnected partial references.

The lane does not change ownership.
It productizes discoverability and first-contact usability for the owner split that was already
frozen by `executor-backed-mutation-surface-v1`.

## Findings

### 1) The repo now has one copyable submit-plus-feedback path

The new first-party path is explicit and reviewable:

- `apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs`
  teaches:
  - `cx.data().mutation_async(...)`,
  - `handle.submit(...)`,
  - `handle.retry_last(...)`,
  - `cx.data().update_after_mutation_completion(...)`,
  - and `shadcn::Sonner::global(cx.app)` for feedback only.
- `apps/fret-cookbook/src/lib.rs` locks the example away from:
  - `toast_promise_async*`,
  - raw `Executors::new(...)`,
  - and raw `Inbox::new(...)`.
- cookbook/discoverability docs now point users to that example directly.

Conclusion:

- first-contact users no longer need to synthesize the correct pattern from `api_workbench_lite`
  plus `toast_basics` on their own.

### 2) Diagnostics evidence now matches the teaching surface

The lane leaves a dedicated screenshot-capable smoke:

- script:
  `tools/diag-scripts/cookbook/mutation-toast-feedback-basics/cookbook-mutation-toast-feedback-basics-smoke.json`
- suite:
  `tools/diag-scripts/suites/cookbook-mutation-toast-feedback-basics/suite.json`
- successful artifact run:
  - screenshot:
    `target/fret-diag/mutation-toast-feedback-basics/sessions/1776183542106-15688/screenshots/1776183582375-cookbook-mutation-toast-feedback-basics-smoke/window-4294967297-tick-46-frame-46.png`
  - bundle:
    `target/fret-diag/mutation-toast-feedback-basics/sessions/1776183542106-15688/1776183582450-cookbook-mutation-toast-feedback-basics-smoke/`

Conclusion:

- the lane is no longer only a prose/example claim; it has bounded visual evidence.

### 3) No mechanism widening was required

The lane closes without:

- widening `fret-mutation` into a toast-aware runtime,
- teaching `toast_promise_async*` as the default submit owner,
- or reopening the closed query-vs-mutation naming/ownership debate.

That confirms the prior diagnosis:

- the missing piece was a teaching surface and proof surface,
- not another framework-level state rewrite.

Conclusion:

- the correct fix really was a narrow cookbook/docs/diag follow-on.

## Decision from this audit

Treat `mutation-toast-feedback-golden-path-v1` as:

- closed for the first-party cookbook/diag/doc productization goal,
- the current source of truth for how to teach mutation authority plus Sonner feedback projection,
- and a historical closeout record unless future user pressure proves this path is still not enough.

## Gates used for closeout

- `cargo check -p fret-cookbook --features cookbook-mutation --example mutation_toast_feedback_basics`
- `cargo check -p fret-cookbook --features cookbook-mutation,cookbook-diag --example mutation_toast_feedback_basics`
- `cargo nextest run -p fret-cookbook mutation_toast_feedback_example_keeps_submit_and_feedback_projection_split`
- `cargo nextest run -p fret-cookbook cookbook_examples_follow_surface_contracts`
- `cargo nextest run -p fret docs_lock_query_reads_vs_mutation_submit_story`
- `FRET_DIAG=1 cargo run -p fretboard-dev -- diag run tools/diag-scripts/cookbook/mutation-toast-feedback-basics/cookbook-mutation-toast-feedback-basics-smoke.json --dir target/fret-diag/mutation-toast-feedback-basics --session-auto --launch -- cargo run -p fret-cookbook --features cookbook-mutation,cookbook-diag --example mutation_toast_feedback_basics`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/mutation-toast-feedback-golden-path-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`

## Follow-on policy

Do not reopen this lane for:

- Sonner API redesign,
- query invalidation helper growth,
- or another broad async-surface rename discussion.

If future work is needed, start a narrower follow-on such as:

1. a GenUI example that dispatches into app-owned mutation handlers explicitly,
2. a deeper cookbook lane for mutation + query invalidation + optimistic projection,
3. or a fresh consumer audit proving that real tool-app flows still feel wrong even after this
   teaching surface landed.
