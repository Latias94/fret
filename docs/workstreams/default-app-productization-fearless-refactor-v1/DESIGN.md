# Default App Productization Fearless Refactor v1 — Design

Status: Draft
Last updated: 2026-04-02

Related:

- `docs/examples/todo-app-golden-path.md`
- `ecosystem/fret/README.md`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/app-composition-density-follow-on-v1/DESIGN.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/SURFACE_CLASSIFICATION_2026-03-16.md`
- `docs/workstreams/shell-composition-fearless-refactor-v1/PAGE_SHELL_AUDIT_2026-04-02.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/DRIFT_AUDIT_2026-04-02.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/RECIPE_PROMOTION_AUDIT_2026-04-02.md`

Status note on 2026-04-02:

- the core authoring-path and local-state contract questions are already closed elsewhere,
- this lane should inherit those decisions and treat current pain as productization drift,
- do not use this folder to quietly reopen state/runtime contracts or shell ownership.

## Why this workstream exists

Fret now has enough default-path pieces to teach a coherent app story:

- `hello` as the smallest runnable surface,
- `simple-todo` as the first real `LocalState<T>` + typed-action + keyed-list lesson,
- `todo` as the richer third-rung product surface.

However, the shipped first-party surfaces still drift in a few release-facing ways:

- some docs and closed authoring lanes teach grouped view-owned locals as `*Locals::new(cx)`,
- some live demo/template code has drifted back to `*Locals::new(app)`,
- the richer todo template still risks reading like a framework showcase instead of a product
  starting point,
- `todo_demo` contains repeated app-level composition patterns that should be audited as
  app-local helpers vs real reusable recipes.

This lane exists to close that gap before wider external release.

It is a productization lane, not a new contract-design lane.

## Problem statement

The default app path is already structurally correct, but it is not yet productized enough to
serve as one stable first-contact story.

Current symptoms:

1. **Blessed-path drift**
   - closed docs/workstreams teach one grouped-local organization rule,
   - live first-party code teaches another,
   - new users cannot tell which one is the actual recommendation.
2. **Third-rung template density**
   - the richer `todo` starter carries more concepts than a product baseline needs on first open,
   - users may read “this template demonstrates what Fret can do” as “this is how every ordinary
     app should begin”.
3. **Recipe pressure without promotion discipline**
   - `todo_demo` repeats app-level composition that feels reusable,
   - but the recent shell audit already proved that “feels reusable” is not enough to mint a
     shared shell surface,
   - and the repo still needs a narrower promotion rule for recipe-like app helpers.

## Goals

1. Reassert one explicit blessed path for the default app ladder.
2. Productize the richer `todo` starter so it reads like a realistic product starting point rather
   than a feature wall.
3. Keep `todo_demo` as a polished demo/proof surface without letting it silently define framework
   contracts.
4. Audit which repeated app-level helpers should stay app-owned and which may eventually deserve a
   shared recipe owner.
5. Leave behind gates that prevent the default path from drifting again.

## Non-goals

- Reopening the `LocalState<T>` storage contract.
- Reopening selector/query read surfaces or the grouped `cx.actions()` write budget.
- Introducing a universal `AppShell` or promoting page shell by convenience.
- Widening `fret::app::prelude::*` just to save one Todo-shaped example.
- Treating Todo-only pressure as proof that a new public helper family is needed.

## Inherited decisions (do not silently reopen)

### 1) The default local-state story is already closed

This lane inherits:

- `LocalState<T>` / `use_local*` as the default app-facing local-state story,
- the current model-backed `LocalState<T>` contract,
- grouped reads/writes through the existing `LocalState<T>` and `cx.actions()` surfaces.

If that contract needs to change, the work belongs in an ADR-backed architecture lane, not here.

### 2) Grouped local bundles currently teach `new(cx)`

This lane inherits the current grouped-locals teaching rule from the closed view-locals lane:

- keep one or two trivial locals inline,
- once a view owns several related local slots, prefer a small `*Locals` bundle,
- teach `*Locals::new(cx)` as the default construction point unless a new explicit decision
  replaces it.

This lane may decide that the shipped examples/templates drifted away from that frozen rule, but it
must not treat the drift itself as proof that the contract was never decided.

### 3) `todo` is the third rung, not the first rung

The richer `todo` starter is allowed to be denser than `hello` or `simple-todo`.

What it is not allowed to be:

- a confusing mixture of product baseline and framework capability showcase,
- a silent second source of truth for the blessed path,
- or a back door for reopening already-closed default-path decisions.

### 4) There is no universal `AppShell`

This lane inherits the shell-composition decision:

- page shell stays app-owned unless a future multi-surface audit proves a real shared owner,
- `todo_demo` does not justify a new shared shell by itself,
- recipe extraction must stay narrower than “promote a shell”.

## Current diagnosis

### 1) The repo is teaching two grouped-local construction stories

Current docs and closeout notes still point to:

- `*Locals::new(cx)` for grouped view-owned local handles,
- optional `bind_actions(&self, cx)`,
- explicit reads/writes through `LocalState<T>` and `cx.actions()`.

But live first-party demo/template code currently also teaches:

- `*Locals::new(app)` plus `LocalState::from_model(app.models_mut().insert(...))`.

That is not a small naming difference. It changes where users think local organization starts and
how much runtime/storage detail they should carry in their head.

### 2) The richer todo starter still needs productization

The current richer template is functionally useful, but it still carries a lot of concept surface at
once:

- grouped locals,
- filters,
- derived snapshots/selectors,
- query-driven tips or similar async adornments,
- and richer chrome/state rendering than a product baseline strictly needs.

That is acceptable only if:

- those concepts are truly the point of the third rung,
- and the resulting template still feels like a product starting point rather than an audit page.

This lane should make that judgment explicit instead of letting the template accrete concepts by
default.

### 3) `todo_demo` reveals recipe pressure, not shell pressure

`todo_demo` still contains repeated patterns that feel potentially reusable:

- responsive centered page wrapping,
- card header/status/progress composition,
- hover-reveal destructive row actions.

The correct next step is not “extract a shared shell”.

The correct next step is:

- audit those patterns,
- classify them as app-local helper vs reusable recipe candidate,
- and promote only what survives cross-surface evidence.

## Core decisions for this lane

### 1) Open a new productization lane instead of reusing closed historical lanes

The older authoring-density, view-locals, selector/query, and shell-composition lanes remain valid
inputs, but they should be read as inherited decisions or historical rationale.

This lane exists because the remaining work is no longer:

- “what should the contract be?”,

and is now:

- “are the shipped first-party surfaces actually teaching the already-chosen contract well enough
  to release?”

### 2) No new ADR is required to start this lane

Starting this lane does **not** require a new ADR.

The initial slices are teaching-surface and recipe-boundary work:

- docs,
- first-party examples,
- scaffold templates,
- source-policy gates,
- and audit notes.

### 3) Escalate to an ADR only on real contract motion

Add an ADR only if one of these happens:

1. the repo intentionally changes the blessed grouped-local construction rule away from the current
   `*Locals::new(cx)` teaching target,
2. a new stable public authoring surface or shared recipe owner is introduced and intended to stay
   public,
3. the lane discovers that state/action/query contracts must be reopened to achieve the desired
   productization result.

If none of those happen, keep the decision in this workstream.

### 4) Prefer convergence to the already-frozen default story

The default posture for this lane is conservative:

- converge first-party teaching surfaces back to the already-frozen story,
- do not assume that live drift means the old decision should be replaced,
- require an explicit decision note before changing the blessed path.

### 5) Keep recipe promotion narrower than shell promotion

Candidate reusable helpers must pass a narrower recipe audit:

- prove at least three aligned first-party consumers,
- prove the behavior is not tied to one Todo-shaped product surface,
- choose an explicit recipe/component owner,
- stay off the default `fret` root and prelude,
- and leave a gate behind before promotion is considered done.

## Target end state

This lane is successful when all of the following are true:

- `ecosystem/fret/README.md`, `docs/examples/todo-app-golden-path.md`, cookbook examples,
  `todo_demo`, and scaffold templates teach one coherent blessed path,
- the richer `todo` starter still feels like the third rung, but opens as a product starting point
  rather than a framework capability wall,
- repeated app-level helpers have explicit keep-local vs promote decisions,
- no new fake shared shell appears,
- and source-policy plus evidence gates prevent this story from drifting again.

## Execution order

1. Freeze inherited decisions and record the current drift.
2. Decide whether the current blessed grouped-local path still stands unchanged.
3. Slim and productize the richer `todo` starter around that answer.
4. Audit recipe candidates from `todo_demo` and related surfaces.
5. Align docs/examples/templates/source-policy together.
6. Add or refresh gates.

## Evidence surface

Primary default-path detection surface:

- `ecosystem/fret/README.md`
- `docs/examples/todo-app-golden-path.md`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

Secondary proof surfaces:

- `apps/fret-examples/src/simple_todo_demo.rs`
- `docs/examples/README.md`
- `docs/crate-usage-guide.md`
- `apps/fret-examples/src/lib.rs`

Potential future recipe-proof surfaces:

- list rows outside Todo-shaped demos,
- settings/forms surfaces with destructive row actions,
- future first-party product shells that need centered responsive page framing but are not editor
  shells.

## Gate expectations

Every non-trivial slice in this lane should leave at least:

- one source-policy or template-generation gate,
- one evidence note or audit artifact,
- and, when layout/resize behavior is the point, one diagnostics or screenshot-based proof surface.

This is a release-facing productization lane. Drift must be visible quickly.
