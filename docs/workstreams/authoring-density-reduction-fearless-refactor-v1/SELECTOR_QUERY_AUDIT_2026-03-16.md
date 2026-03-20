# Selector / Query Audit — 2026-03-16

This audit records the next evidence pass after the tracked-read helper landing and the broader
first-party adoption cleanup.

Goal:

- determine whether the remaining pressure in larger first-party surfaces is still a tracked-read
  adoption problem,
- or whether the next active design question has already shifted to selector/query ceremony.

## Evidence surfaces used in this pass

Focused surfaces:

- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Why these two:

- `genui_demo` is a medium first-party surface that still shows a few older tracked-read spellings
  in a non-Todo context.
- `imui_editor_proof_demo` is the largest editor-style proof surface in the repo and is the most
  likely place to reveal whether any remaining pressure is still about read helpers versus
  higher-level derived-state/query choreography.

## Findings

### 1. `genui_demo` confirmed the last low-risk M1 breadth cleanup shape

The older spellings observed in `genui_demo` were straightforward handle-first read candidates:

- `cx.watch_model(&st.auto_apply_standard_actions).layout().read(...)`
- `cx.watch_model(&st.auto_fix_on_apply).layout().read(...)`
- `cx.watch_model(&st.genui_state).layout().read(...)`
- `cx.watch_model(&st.action_queue).layout().read(...)`
- `cx.watch_model(&st.validation_state).layout().read(...)`
- `cx.watch_model(&st.stream_patch_only).layout().read(...)`

These reads already fit the shipped tracked-read posture:

- `model.layout_in(cx)` for `ElementContext`,
- followed by one read/value extraction step.

Conclusion:

- `genui_demo` does **not** justify another shared tracked-read helper.
- It was M1 breadth cleanup on an already-correct surface, and is now the right place to stop
  growing M1 design scope.

### 2. `imui_editor_proof_demo` splits into two different categories

#### A. Simple committed/outcome readouts are still M1 adoption cleanup

Examples:

- committed text/password/notes values read through `cx.watch_model(...).paint().cloned()`
- committed gradient stops read through `cx.watch_model(...).paint().cloned()`
- embedded target reads through `cx.watch_model(&m.target).paint().copied()`

These are the same kind of cleanup as the custom-effect and utility-window families:

- `model.paint_in(cx).cloned()`
- `model.paint_in(cx).copied()`

Conclusion:

- these sites are still M1 breadth cleanup,
- but they do not imply a new tracked-read API problem.

#### B. Assist-state panels reveal M2 pressure, not another M1 API gap

The more important repeated pattern is not the raw `watch_model` call itself.
It is the repeated multi-handle choreography used to derive view state:

- read `query`
- read `dismissed_query`
- read `active_item_id`
- rebuild the controller
- derive visible/expanded/active labels

This shows up in multiple adjacent proof panels for the same authoring story.

What matters here:

- the pain is no longer "tracked reads are missing a short helper",
- the pain is "view-owned derived state still reads like low-level handle plumbing".

Conclusion:

- this is evidence for **M2 selector/query density**,
- not evidence for minting another tracked-read helper for M1.

### 3. Large editor-proof surfaces should no longer block M1 closure as a design milestone

If `imui_editor_proof_demo` is used as the standard for "M1 is not done until every last
`watch_model(...).paint()` is replaced", the workstream will keep confusing two different problems:

- residual breadth cleanup on already-shipped read helpers,
- versus real selector/query derived-state ceremony.

That would make the lane drift.

Conclusion:

- M1 should be treated as **design-complete / closeout mode** once the repo has:
  - the shipped handle-first helper,
  - representative first-party adoption across non-Todo surfaces,
  - and source-policy gates protecting the taught path.
- Remaining editor-proof cleanup can continue opportunistically, but it should not block the next
  active design step.

## Decision from this audit

### M1 status

Treat M1 as:

- shared-surface question resolved,
- representative adoption proven,
- remaining work mostly breadth cleanup and source drift retirement.

Do not reopen another tracked-read API family from the `genui` / `imui_editor_proof` evidence.

### M2 status

Promote M2 to the next active design question:

- LocalState-first selector dependency/read ceremony
- query/assist-derived read ceremony
- repeated multi-model read-and-derive patterns that still feel like plumbing on the default path

## Immediate execution consequence

Next work should bias toward:

1. keep any further M1 cleanup opportunistic and low-risk, especially on editor-proof-only readout
   surfaces, and
2. run a focused M2 design/audit pass using `genui_demo`, query-heavy examples, and the assist-state
   portions of `imui_editor_proof_demo` as evidence.

That keeps the lane honest:

- M1 stops growing after the justified tracked-read helper already landed,
- and M2 starts from real non-Todo evidence instead of Todo-only intuition.

## Addendum — 2026-03-20

The M1 breadth-cleanup items called out in this audit are now landed on the audited proof/example
surfaces:

- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/lib.rs` source-policy gates

This means the earlier "simple committed/outcome readouts are still M1 adoption cleanup" category
should now be read as historical evidence that justified the cleanup, not as remaining open work.

The next live design pressure remains what this audit already identified as M2:

- selector/query ceremony on the default path,
- multi-model derived-state choreography,
- and authoring density rather than missing tracked-read helpers.
