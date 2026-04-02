# Default App Productization Drift Audit — 2026-04-02

Status: Initial baseline audit, with M1 blessed-path convergence follow-up landed on 2026-04-02

## Scope

This audit answers one practical release-facing question:

- does the shipped default app ladder already teach one coherent, productized blessed path?

Short answer at lane start:

- not yet; the underlying decisions are mostly closed, but first-party teaching surfaces have
  drifted and need a dedicated productization lane.

Follow-up after M1:

- the live grouped-local construction drift is now closed on the default Todo ladder,
- docs/cookbook and live demo/template surfaces now converge on `*Locals::new(cx)`,
- remaining work in this lane is about richer-template productization and recipe-promotion
  discipline, not blessed-path disagreement.

## Primary evidence set

- `ecosystem/fret/README.md`
- `docs/examples/todo-app-golden-path.md`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

## Findings

### 1) The written golden path still teaches grouped locals as `new(cx)`

Current default-path docs and closeout notes still converge on the same organization rule:

- keep one or two trivial locals inline,
- once a view owns several related `LocalState<T>` slots, prefer a small `*Locals` bundle,
- teach `*Locals::new(cx)` as the default construction point,
- keep reads/writes explicit through `LocalState<T>` and `cx.actions()`.

Evidence:

- `docs/examples/todo-app-golden-path.md`
- `ecosystem/fret/README.md`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/DESIGN.md`

### 2) At lane start, live first-party surfaces drifted back to `new(app)`

At lane start, the live first-party default ladder was no longer aligned with that frozen teaching
rule.

Observed drift at lane start:

- `apps/fret-examples/src/todo_demo.rs` currently constructs `TodoLocals` from `&mut App`,
- `apps/fretboard/src/scaffold/templates.rs` currently emits grouped locals the same way for the
  richer generated starters,
- while the cookbook `simple_todo` lesson still reads like the older blessed-path
  `TodoLocals::new(cx)` organization.

This is enough drift to justify a new productization lane, but not enough to justify reopening the
underlying state contract by itself.

Evidence:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`

Follow-up:

- this drift is now closed by the M1 convergence edits in `apps/fret-examples/src/todo_demo.rs`,
  `apps/fret-examples/src/simple_todo_demo.rs`, and `apps/fretboard/src/scaffold/templates.rs`,
- the refreshed guards live in `apps/fret-examples/src/lib.rs` and
  `apps/fretboard/src/scaffold/templates.rs`.

### 3) The richer todo starter is functionally correct, but not yet clearly productized

The richer todo starter still carries a lot of visible concept surface at once:

- grouped locals,
- filters,
- derived collection snapshots/selectors,
- query-driven “tip” style adornments or similar async extras,
- and more product chrome than the second rung.

That is not automatically wrong because `todo` is the third rung.

The problem is different:

- the template still risks reading like “here is everything Fret can do in one file”,
- instead of “here is a realistic product baseline you can keep or delete from”.

This is a productization problem, not proof that selector/query or local-state contracts need to be
redesigned.

Evidence:

- richer todo template sections in `apps/fretboard/src/scaffold/templates.rs`
- the current ladder wording in `ecosystem/fret/README.md`

### 4) `todo_demo` reveals repeated app-level recipe pressure

`todo_demo` now exposes several patterns that feel like reusable composition, but they are not all
the same kind of candidate:

- responsive centered page wrapping,
- card header/status/progress composition,
- hover-reveal destructive row actions.

The recent shell audit already closed one tempting wrong answer:

- do not promote a shared page shell just because `todo_demo` and other first-party examples all
  have page framing.

That means the correct next step is a recipe audit:

- keep app-owned helpers app-owned by default,
- promote only the patterns that survive cross-surface proof and owner selection.

Evidence:

- `apps/fret-examples/src/todo_demo.rs`
- `docs/workstreams/shell-composition-fearless-refactor-v1/PAGE_SHELL_AUDIT_2026-04-02.md`

## What this audit does not say

This audit does **not** say:

- that `LocalState<T>` should be redesigned,
- that `cx.actions()` needs a new helper family,
- that a universal `AppShell` should exist,
- or that Todo pressure alone justifies new public APIs.

Those questions are already closed or explicitly out of scope for this lane unless fresh
cross-surface evidence proves otherwise.

## Outcome

Open a new release-facing productization lane with this scope:

1. converge the blessed grouped-local path across docs/examples/templates,
2. slim and clarify the richer todo starter,
3. audit app-level recipe candidates without promoting a fake shell,
4. add gates so the default path does not drift again.

## ADR posture

No new ADR is required to start this work.

Escalate to an ADR only if:

1. the lane intentionally changes the current blessed grouped-local rule away from `*Locals::new(cx)`,
2. the lane introduces a new stable public authoring surface,
3. or the lane must reopen state/action/query contracts to finish the productization work.
