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
- `SELECTOR_QUERY_AUDIT_2026-03-16.md`
- `SELECTOR_QUERY_DIRECTION_2026-03-16.md`

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
  - 2026-03-16 third pass:
    - representative advanced/example adoption cleanup now includes custom-effect, liquid-glass,
      markdown, launcher utility-window, hit-test probe, and the custom-effect-v2 web/material
      demo family,
    - the remaining work is mostly breadth cleanup on other first-party `ElementContext` examples
      or larger editor-style surfaces rather than another shared-surface design question.
  - 2026-03-16 selector/query audit:
    - `genui_demo` confirmed the last low-risk tracked-read cleanup shape and does not justify
      another shared read helper,
    - `imui_editor_proof_demo` splits between low-risk `paint_in(cx)` cleanup and a more important
      repeated assist/query-derived-state choreography problem,
    - that larger choreography problem is M2 evidence, not a reason to reopen M1 surface design.
- [ ] Land the smallest justified tracked-read reduction.
  - Constraints:
    - keep invalidation intent explicit,
    - do not widen `fret::app::prelude::*`,
    - delete displaced taught wording after the new path lands.
- [ ] Audit LocalState-first selector dependency/read ceremony.
  - Goal: stop teaching raw model-handle choreography on first-contact LocalState-first surfaces
    unless shared ownership is the point.
  - 2026-03-16 direction:
    - this is now the first likely real M2 shared-surface gap,
    - the helper should live at the `ecosystem/fret` app-facing layer,
    - and `fret-selector` should remain unaware of `LocalState<T>`.
- [ ] Audit query observe/read ceremony.
  - Goal: keep query lifecycle explicit while reducing repeated watch/read/default plumbing on the
    default app path.
  - 2026-03-16 direction:
    - treat current `value_or_else(QueryState::<T>::default)` repetition primarily as adoption/docs
      drift,
    - normalize taught app-path reads to `handle.layout(cx).value_or_default()`,
    - normalize taught declarative/component-path reads to
      `handle.layout_query(cx).value_or_default()`,
    - do not add new shared query sugar until the post-cleanup surface is re-measured.
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
  - Current reading:
    - the tracked-read helper decision is already made,
    - representative non-Todo adoption is already proven,
    - remaining work is closeout/breadth cleanup rather than another shared-surface question.

## M2 — Selector/query density

- [ ] Inventory selector LocalState-first boilerplate that still reads like raw model plumbing.
- [ ] Freeze the layering rule for selector work.
  - Any LocalState-aware dependency helper belongs in `ecosystem/fret`, not `fret-selector`.
- [ ] Inventory query observe/read boilerplate that still reads like low-level handle plumbing.
  - 2026-03-16 starting evidence:
    - `genui_demo` remains a medium non-Todo proof surface for selector/query follow-up after the
      final low-risk tracked-read cleanup landed,
    - `imui_editor_proof_demo` already shows repeated assist-state derived-read choreography that is
      better classified as M2 pressure than as M1 tracked-read debt.
- [ ] Normalize query docs/templates/examples to the already-shipped shorter default reads before
  adding any new shared query helper.
- [ ] Design the smallest LocalState-aware selector dependency bridge that keeps invalidation
  explicit without teaching `clone_model()` on the default app path.
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
