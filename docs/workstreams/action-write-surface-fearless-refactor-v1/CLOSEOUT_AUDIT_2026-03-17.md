# Closeout Audit — 2026-03-17

This audit records the final closeout read for the action-write surface v1 lane.

Goal:

- verify whether the repo still has an active default write-side migration problem on
  `cx.actions()`,
- separate the landed default write budget from advanced/reference seams that remain public,
- and decide whether this lane should remain active or become historical maintenance evidence only.

## Audited evidence

Core workstream docs:

- `docs/workstreams/action-write-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/TODO.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/ONE_SLOT_WRITE_AUDIT_2026-03-17.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/PAYLOAD_ROW_WRITE_AUDIT_2026-03-17.md`

Default teaching surfaces:

- `docs/README.md`
- `docs/first-hour.md`
- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/README.md`
- `docs/examples/todo-app-golden-path.md`
- `ecosystem/fret/README.md`

Implementation / gate anchors:

- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-cookbook/examples/payload_actions_basics.rs`
- `apps/fret-examples/src/markdown_demo.rs`

Validation run used for closeout:

- `git diff --check -- docs/README.md docs/first-hour.md docs/crate-usage-guide.md docs/examples/todo-app-golden-path.md docs/examples/README.md docs/fearless-refactoring.md docs/authoring-golden-path-v2.md docs/ui-ergonomics-and-interop.md ecosystem/fret/README.md apps/fretboard/src/scaffold/templates.rs docs/workstreams/action-write-surface-fearless-refactor-v1/DESIGN.md docs/workstreams/action-write-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md docs/workstreams/action-write-surface-fearless-refactor-v1/MILESTONES.md docs/workstreams/action-write-surface-fearless-refactor-v1/TODO.md docs/workstreams/action-write-surface-fearless-refactor-v1/PAYLOAD_ROW_WRITE_AUDIT_2026-03-17.md docs/workstreams/action-write-surface-fearless-refactor-v1/ONE_SLOT_WRITE_AUDIT_2026-03-17.md`
- `CARGO_TARGET_DIR=target-codex-verify cargo test -p fretboard template`

## Findings

### 1. The one-slot write family is intentionally small and no longer ambiguous

The M1 audit evidence is coherent across cookbook, demos, templates, and default docs:

- `local_update::<A>(...)` covers in-place mutation of one local slot,
- `local_set::<A, T>(...)` covers fixed target-value writes,
- `toggle_local_bool::<A>(...)` covers explicit boolean flips,
- and `locals::<A>(...)` remains the primary coordinated transaction story.

The repo is no longer teaching these as competing transaction dialects.

Conclusion:

- there is no active one-slot write-surface redesign debt left on this lane.

### 2. The keyed payload row-write story is now converged on one default path

The shipped default row-write proof is now consistent:

- `apps/fret-cookbook/examples/simple_todo.rs` and `apps/fret-examples/src/todo_demo.rs` prove
  `payload_local_update_if::<A>(...)` on real runtime surfaces,
- first-contact docs and generated template READMEs now teach
  `payload_local_update_if::<A>(...)` as the only default keyed row-write path,
- template tests in `apps/fretboard/src/scaffold/templates.rs` now explicitly reject
  `payload_locals::<A>(...)` from the generated `todo` README surface.

At the same time:

- `payload::<A>()` remains intentionally lower-level/reference-only in
  `apps/fret-cookbook/examples/payload_actions_basics.rs` and
  `apps/fret-examples/src/markdown_demo.rs`,
- `payload_locals::<A>(...)` remains implemented but is now classified only as an explicit
  advanced/reference seam until first-party proof exists.

Conclusion:

- the repo no longer teaches two co-equal default payload write stories.

### 3. Docs, templates, and gates now agree on the same default write-side budget

The first-contact/default teaching surfaces now align on the same posture:

- teach `locals::<A>(...)` as the primary coordinated local transaction path,
- teach the one-slot trio as a semantics-driven companion family,
- teach `payload_local_update_if::<A>(...)` as the only default keyed row-write helper,
- keep `models::<A>(...)`, `transient::<A>(...)`, `payload::<A>()`, and `payload_locals::<A>(...)`
  explicit rather than default.

This is not only prose alignment:

- `apps/fretboard/src/scaffold/templates.rs` locks the generated README guidance,
- the `fretboard` template test suite passed after the wording change,
- and the workstream audits now record the advanced/reference seam classification explicitly.

Conclusion:

- the lane's original closeout condition is satisfied.

### 4. The remaining write-side questions are now future separate lanes, not unfinished v1 debt

What remains after closeout does not justify keeping this lane active:

1. Re-promotion question
   - if a real first-party proof surface eventually needs `payload_locals::<A>(...)`, that is a
     future re-promotion decision, not unfinished v1 work.
2. Delete-ready question
   - if future evidence shows that `payload_locals::<A>(...)` or `payload::<A>()` should be
     removed entirely, that is a narrower hard-delete follow-on, not default-path migration debt.
3. Broader runtime or dataflow questions
   - selector/query, router, and `LocalState<T>` architecture already live on other closed or
     separate lanes.

Conclusion:

- this workstream no longer owns an active migration queue.

## Decision from this audit

Treat `action-write-surface-fearless-refactor-v1` as:

- closed for the default app-lane write-budget hardening goal,
- maintenance/historical evidence only by default,
- and reopenable only through a new narrower lane if fresh cross-surface evidence appears.

## Immediate execution consequence

From this point forward:

1. keep the shipped default write-side budget stable,
2. keep `payload_local_update_if::<A>(...)` as the only taught default keyed row-write path,
3. keep `payload_locals::<A>(...)` and `payload::<A>()` explicitly advanced/reference unless new
   proof justifies change,
4. do not reopen this lane just to explore helper growth from Todo-shaped pressure alone.
