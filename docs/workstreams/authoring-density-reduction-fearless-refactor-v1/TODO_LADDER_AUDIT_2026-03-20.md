# Todo Ladder Audit — 2026-03-20

This audit records a post-closeout reading pass for the current Todo authoring ladder.

Goal:

- verify how the shipped `simple-todo` and richer `todo` surfaces now read after the tracked-read
  and selector/query cleanup already landed,
- determine whether the remaining ceremony is still old helper debt,
- and keep future follow-on work aimed at real cross-surface density rather than Todo-only sugar.

## Evidence surfaces used in this pass

Runnable examples and scaffolds:

- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

Docs that teach the ladder:

- `docs/examples/todo-app-golden-path.md`
- `docs/authoring-golden-path-v2.md`

Cross-check for "is this only a Todo problem?":

- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`

## Findings

### 1. `simple-todo` is now the stable first-contact baseline

The current `simple-todo` surfaces already converge on one short default story:

- view-owned `LocalState<T>` slots,
- direct tracked reads such as `layout_value(cx)`,
- grouped action helpers such as `locals_with((...)).on::<A>(...)`,
- `payload_update_if::<A>(...)` for keyed row writes,
- and `ui::for_each_keyed(...)` for dynamic rows.

What matters:

- there is no selector setup,
- there is no query lifecycle surface,
- and there is no raw shared-model choreography.

Conclusion:

- `simple-todo` should now be read as the shipped first-contact product surface, not as an
  intermediate waypoint that still needs another helper pass.

### 2. The richer `todo` scaffold is denser because it is a third-rung exemplar, not because the old default path is still leaking through

The current `todo` scaffold remains visibly heavier than `simple-todo`, but the weight now comes
from real third-rung concepts:

- derived-state projection via `cx.data().selector_layout(...)`,
- explicit `TodoDerived` / `TodoRowSnapshot` shaping,
- query setup via `QueryKey`, `QueryPolicy`, and `cx.data().query(...)`,
- query lifecycle branching via `handle.read_layout(cx)` + `match QueryStatus`,
- and filter controls that intentionally exercise app-facing typed actions.

What is no longer true:

- the scaffold no longer teaches `clone_model()` as the first selector story,
- it no longer depends on older `watch(...).layout().value_or_else(...)` query reads,
- and its remaining ceremony is not primarily tracked-read helper debt.

Conclusion:

- the current richer `todo` surface should be read as an app-grade "third rung" exemplar:
  intentionally broader than `simple-todo`, but no longer accidentally broad because of stale
  plumbing.

### 3. Query density is a real cross-surface concept cost, not a Todo-only artifact

The same explicit query nouns also appear in the dedicated query examples:

- `QueryKey`
- `QueryPolicy`
- `cx.data().query(...)`
- `handle.read_layout(cx)`
- `QueryStatus`
- explicit success/error/loading branching

This means the remaining density is not caused by Todo itself.
It is the current cost of teaching query lifecycle explicitly on the app lane.

Conclusion:

- future work should only reopen this area as a cross-surface selector/query authoring question,
  not as a Todo-template tweak.

### 4. Filter-action ceremony is the only notably Todo-shaped remainder, but it still does not justify reopening this closed lane

The richer scaffold still spends visible surface area on:

- `FilterAll`
- `FilterActive`
- `FilterCompleted`
- and three matching `.set::<...>(TodoFilter::...)` bindings.

This is the most Todo-shaped remaining ceremony in the scaffold, but it is also a write-surface
question rather than a read-surface regression.

Conclusion:

- if this is revisited later, it belongs with action/write-surface follow-on work,
- not with this already-closed density-reduction lane,
- and not with selector/query read helpers.

### 5. Router should stay out of this interpretation pass

The audited Todo ladder does not exercise routing as part of its first-contact or third-rung state
story.

Conclusion:

- router should not be folded into this lane without fresh cross-surface evidence that the same
  authoring-density problem appears on routed app surfaces as well.

## Decision from this audit

Read the shipped Todo ladder this way:

1. `simple-todo` is the stable first-contact baseline.
2. `todo` is the richer third-rung exemplar.
3. The remaining density in `todo` is mainly selector/query lifecycle explicitness plus a small
   filter-action write-side cost.
4. This closed lane should not be reopened just because the third rung is still longer than the
   first rung.

## Immediate execution consequence

Do not start another helper pass from the current Todo evidence alone.

If maintainers reopen anything after this audit, the candidate directions are:

1. a new cross-surface selector/query authoring-density lane, or
2. a write-surface lane if enum-like action families such as Todo filters recur outside this
   scaffold.

Until then:

- keep `simple-todo` as the default first-contact teaching surface,
- keep the richer `todo` scaffold as the explicit selector/query/app-grade rung,
- and reject Todo-only convenience API growth.
