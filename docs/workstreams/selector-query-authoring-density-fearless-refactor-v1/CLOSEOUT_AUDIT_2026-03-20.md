# Closeout Audit — 2026-03-20

This audit records the final closeout read for the selector/query density v1 lane.

Goal:

- verify whether the repo still has an active default app-lane selector/query density backlog,
- separate the landed query reduction from the rejected selector borrowed-input follow-on,
- and decide whether this lane should remain active or become historical maintenance evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/TODO.md`
- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/SELECTOR_BORROWED_INPUT_AUDIT_2026-03-20.md`

Default-path docs/templates/examples:

- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `apps/fretboard/src/scaffold/templates.rs`

Implementation / proof anchors:

- `ecosystem/fret-query/src/lib.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/lib.rs`

Validation run used for closeout:

- `cargo nextest run -p fret-query query_status_projection_helpers_report_expected_values query_state_projection_helpers_detect_refreshing_and_error_presence`
- `cargo nextest run -p fret-examples --lib selected_view_runtime_examples_prefer_grouped_state_actions_and_effects`
- `cargo nextest run -p fretboard todo_template_uses_default_authoring_dialect template_readmes_capture_authoring_guidance`
- `git diff --check -- apps/fretboard/src/scaffold/templates.rs docs/README.md docs/authoring-golden-path-v2.md docs/examples/todo-app-golden-path.md docs/roadmap.md docs/workstreams/README.md docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/DESIGN.md docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/TARGET_INTERFACE_STATE.md docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/MILESTONES.md docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/TODO.md docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/SELECTOR_BORROWED_INPUT_AUDIT_2026-03-20.md docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Findings

### 1. The shipped query helper batch removes repeated semantic branch noise without hiding lifecycle ownership

The landed query reduction is intentionally narrow:

- create-side ownership remains explicit through `QueryKey`, `QueryPolicy`, and `cx.data().query*`,
- app-lane reading remains explicit through `handle.read_layout(cx)`,
- and the only new surface is semantic projection on already-existing state:
  - `QueryStatus::as_str()`,
  - `QueryStatus::{is_idle,is_loading,is_success,is_error}`,
  - `QueryState::{is_idle,is_loading,is_success,is_error,has_data,has_error,is_refreshing}`.

This matters because the repeated first-party pressure was real but also repetitive in exactly the
same way:

- status label strings,
- loading versus refreshing checks,
- `data.is_some()` / `error.is_some()` semantic branching.

The landed helper batch shortens those sites while keeping the real lifecycle visible.

Conclusion:

- the query side of this lane is landed on a no-regret reduction.

### 2. The selector side does not currently justify a borrowed-input public follow-on

The selector audit now freezes the key decision:

- the Todo scaffold is still the strongest pressure point,
- `async_playground_demo` has some owned intermediate shaping but remains readable and intentional,
- and the rest of the audited selector sites are mostly compact settings/stat projections that do
  not read like a missing API.

That means the current repo evidence does **not** show the same borrowed-input pressure repeating
across:

- the default first-contact path,
- multiple app-facing proof surfaces,
- and additional non-Todo runtime examples.

Conclusion:

- this lane closes the selector question on a no-new-API verdict.

### 3. Docs, templates, proof surfaces, and gates now teach one coherent selector/query posture

After the initial code batch and the final doc/template pass:

- generated scaffold guidance now teaches the new query semantic projections,
- first-party authoring docs point at the same posture,
- the workstream folder records the selector audit and the reopen boundary explicitly,
- and the `fretboard` template tests now guard the shipped scaffold wording.

This is not just prose alignment:

- `fret-query` tests lock the helper semantics,
- `fret-examples` source-policy gates still protect the chosen first-party proof surfaces,
- and the scaffold test suite proves the default generated Todo surface stayed aligned.

Conclusion:

- Milestone 3 is satisfied and the lane no longer owns active migration work.

### 4. The remaining questions now belong to future reopen criteria, not this lane

What remains after closeout is narrower than an active workstream:

1. Future query-side helper growth
   - only reopen if a new semantic pattern repeats across multiple app-facing surfaces and is not
     already covered by the shipped projection helpers.
2. Future selector-side borrowed input work
   - only reopen if the same pressure appears beyond the Todo scaffold on additional non-Todo
     proof surfaces.
3. Adjacent scope
   - router remains explicitly adjacent-only;
   - broader write-surface, composition, or into-element questions stay on their own lanes.

Conclusion:

- this folder should now be read as closeout evidence rather than as another open authoring queue.

## Decision from this audit

Treat `selector-query-authoring-density-fearless-refactor-v1` as:

- closed for the current default app-lane selector/query density question,
- maintenance/historical evidence by default,
- and reopenable only through new cross-surface evidence beyond the current audited set.

## Immediate execution consequence

From this point forward:

1. keep `QueryKey`, `QueryPolicy`, and `cx.data().query*` explicit,
2. keep `handle.read_layout(cx)` as the default app-lane query read,
3. prefer the shipped query semantic projections instead of rebuilding the same status/presence
   checks manually,
4. keep `cx.data().selector_layout(inputs, compute)` as the default LocalState-first selector lane,
5. do not add a borrowed selector helper from Todo-only pressure,
6. keep router/history/link growth out of this lane unless new evidence explicitly reopens it.
