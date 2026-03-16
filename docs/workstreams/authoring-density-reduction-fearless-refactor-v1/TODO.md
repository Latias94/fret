# Authoring Density Reduction (Fearless Refactor v1) — TODO

This TODO list tracks the next active post-v1 authoring lane.

Because the repo is still pre-release, "done" means:

- land the shorter path,
- move docs/examples/templates to it,
- and delete displaced public-looking wording instead of carrying compatibility baggage.

Companion docs:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TRACKED_READ_AUDIT_2026-03-16.md`

## Current priority checklist

- [ ] Freeze the evidence set for density work.
  - Keep the canonical compare set explicit:
    - `apps/fret-cookbook/examples/simple_todo.rs`
    - `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
    - `apps/fret-examples/src/todo_demo.rs`
    - `apps/fretboard/src/scaffold/templates.rs`
  - Require at least one non-todo proof surface before widening a shared public helper/API.
- [ ] Audit tracked-read repetition on the canonical compare set plus at least one non-todo medium
  surface.
  - Goal: classify each repeated chain as:
    - docs/adoption problem,
    - local helper problem,
    - or real shared-surface gap.
  - 2026-03-16 first pass:
    - the compare set itself is already mostly on the shorter `state.layout(cx).value_*` posture,
    - the first meaningful batch is AppUi adoption cleanup on non-todo surfaces,
    - the likely next shared-surface question narrows to `UiCx` / `ElementContext` model-read
      ergonomics rather than another AppUi read helper.
  - 2026-03-16 second pass:
    - helper-heavy `ElementContext` surfaces now show repeated non-todo pressure as well,
    - the next justified reduction is a narrow handle-first tracked-model helper on the
      declarative/component layer,
    - `app::prelude::*` should stay unchanged while `component` / `advanced` helper surfaces adopt
      the shorter read path.
- [ ] Land the smallest justified tracked-read reduction.
  - Constraints:
    - keep invalidation intent explicit,
    - do not widen `fret::app::prelude::*`,
    - delete displaced taught wording after the new path lands.
- [ ] Audit LocalState-first selector dependency/read ceremony.
  - Goal: stop teaching raw model-handle choreography on first-contact LocalState-first surfaces
    unless shared ownership is the point.
- [ ] Audit query observe/read ceremony.
  - Goal: keep query lifecycle explicit while reducing repeated watch/read/default plumbing on the
    default app path.
- [ ] Re-evaluate keyed/list/default child-collection pressure after the read-side reductions.
  - Decision rule:
    - prefer existing helpers and tighter docs first,
    - only add new shared API if the pressure still repeats beyond the Todo compare set.
- [ ] Keep this lane from turning into a bridge-growth lane.
  - `AppActivateExt` growth is out of scope and should be treated as regression pressure, not as an
    ergonomics win.
- [ ] Keep the lane budget frozen.
  - Do not use this workstream to reopen app/component/advanced taxonomy, root exports, or broad
    prelude growth.
- [ ] Update docs/templates/examples and gates together for each landed batch.

## M0 — Freeze the lane

- [x] Add the workstream docs directory and connect it from `docs/README.md`, `docs/roadmap.md`,
  and `docs/workstreams/README.md`.
- [x] Keep the scope explicit:
  - density reduction,
  - not runtime replacement,
  - not LocalState architecture redesign,
  - not todo-only API design.

## M1 — Tracked-read density

- [ ] Inventory repeated tracked-read shapes on the compare set.
- [ ] Inventory the same pressure on at least one non-todo surface.
- [ ] Decide whether the fix is:
  - tighter docs/adoption,
  - a narrower grouped helper,
  - or a different existing helper that should become the taught default.
- [ ] Land the chosen reduction and remove the displaced taught wording from default docs/examples.

## M2 — Selector/query density

- [ ] Inventory selector LocalState-first boilerplate that still reads like raw model plumbing.
- [ ] Inventory query observe/read boilerplate that still reads like low-level handle plumbing.
- [ ] Decide which parts are:
  - intentional ownership/runtime complexity,
  - versus removable authoring noise.
- [ ] Land the smallest justified shared reduction.
- [ ] Prove the result on the third-rung `todo` path plus at least one non-todo surface.

## M3 — Keyed/list/default child-collection follow-up

- [ ] Re-audit the canonical compare set after M1/M2.
- [ ] Check whether the remaining list/collection pressure is still materially visible.
- [ ] If yes, prove it outside the Todo lane before adding shared API.
- [ ] If no, lock the result as docs/adoption discipline instead of reopening helper growth.

## M4 — Delete and lock

- [ ] Remove displaced public-looking wording from default docs/templates/examples.
- [ ] Refresh/extend source-policy tests or other gates that protect the shorter path.
- [ ] Record which older wording survives only as advanced/history-only context, if any.

## Standing rules

- [ ] No todo-only convenience helper should graduate to shared public surface.
- [ ] No new default-path helper should land without a non-todo proof surface.
- [ ] No density fix should solve the problem by widening `fret::app::prelude::*`.
- [ ] No batch is complete until docs/templates/examples and gates agree on the same taught path.
