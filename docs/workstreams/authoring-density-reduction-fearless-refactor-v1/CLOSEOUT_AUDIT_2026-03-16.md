# Closeout Audit — 2026-03-16

This audit records the M4 closeout pass for the authoring-density reduction lane.

Goal:

- verify that the shorter default path is now the only taught default path across first-contact
  docs/templates/examples,
- refresh the source-policy gates for that baseline,
- and classify any remaining older wording as advanced/history-only context rather than as another
  co-equal default path.

## Audited default teaching surfaces

Docs:

- `docs/first-hour.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/README.md`

Templates / source gates:

- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-cookbook/src/lib.rs`
- `ecosystem/fret/src/lib.rs`

## Findings

### 1. The default docs now teach one consistent shorter path

The current first-contact/default docs now agree on the same baseline:

- `LocalState<T>` is the first-contact state story,
- grouped app helpers stay primary (`cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()`),
- tracked reads stay handle-first (`local.layout(cx)`, `local.paint(cx)`, `handle.layout(cx)`),
- keyed lists stay on `ui::for_each_keyed(...)`,
- single-child late landing stays on `ui::single(cx, child)`,
- explicit `.into_element(cx)` / `AnyElement` seams are described as advanced/helper/interop
  boundaries rather than as the default authoring model.

Conclusion:

- the repo no longer teaches two co-equal default stories for app-facing authoring density.

### 2. Templates and source-policy tests already protect the main default path

The generated `todo` / `simple-todo` scaffolds and their unit assertions already lock:

- `ui::for_each_keyed(...)` for keyed rows,
- `ui::single(cx, child)` for late landing,
- no default-path `clone_model()` selector choreography,
- no displaced single-child landing wording such as `todo_page(...).into_element(cx).into()`.

The `ecosystem/fret` doc/source-policy tests now also guard the first-contact docs directly so the
default guidance does not drift back to:

- `cx.watch_model(...)` wording in onboarding docs,
- or explicit `AnyElement` / `.into_element(cx)` framing as the default app-authoring posture.

Conclusion:

- the closeout does not depend only on prose discipline; the baseline is now test-backed.

### 3. Remaining older wording is now explicitly non-default

The remaining older wording falls into three categories:

1. Advanced/component/runtime docs
   - examples: `docs/component-authoring-contracts.md`, `docs/integrating-sqlite-and-sqlx.md`
   - reason: these surfaces intentionally discuss explicit shared `Model<T>` graphs,
     `ElementContext`/component-layer mechanics, or host/runtime-owned state.
2. Advanced retained/interop examples
   - examples: `apps/fret-cookbook/examples/embedded_viewport_basics.rs`,
     `apps/fret-cookbook/examples/chart_interactions_basics.rs`
   - reason: these files intentionally own retained-subtree, viewport, or interop landing seams.
3. Historical workstream / migration notes
   - examples: older workstream audits, migration matrices, and design notes that document the
     displaced wording as history/evidence rather than as current guidance.

Conclusion:

- any surviving longer wording is now either:
  - advanced by ownership/layering,
  - or historical by design.

## Decision from this audit

Treat M4 as:

- closeout complete,
- default docs/templates/examples aligned,
- source-policy gates aligned,
- remaining older wording classified as advanced/history-only context.

## Immediate execution consequence

This workstream no longer needs another default-path helper pass.

Follow-on work should now be maintenance only:

1. keep default docs/templates/examples on the same shorter path,
2. classify future exceptions explicitly as advanced/history-only before accepting them, and
3. reject new helper growth unless new cross-surface evidence reappears outside the already-closed
   default path.
