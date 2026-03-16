# Local-State Architecture (Fearless Refactor v1) — TODO

Status: active decision lane
Last updated: 2026-03-16

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`

## Current priority checklist

- [ ] Freeze the evidence set for this lane.
  - Keep the default-path compare set explicit:
    - `apps/fret-examples/src/hello_counter_demo.rs`
    - `apps/fret-examples/src/query_demo.rs`
    - `apps/fret-examples/src/query_async_tokio_demo.rs`
    - `apps/fret-examples/src/todo_demo.rs`
    - `apps/fretboard/src/scaffold/templates.rs`
    - `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
  - Keep the hybrid proof set explicit:
    - `apps/fret-cookbook/examples/text_input_basics.rs`
    - `apps/fret-cookbook/examples/date_picker_basics.rs`
    - `apps/fret-cookbook/examples/form_basics.rs`
    - `apps/fret-cookbook/examples/virtual_list_basics.rs`
    - `apps/fret-cookbook/examples/theme_switching_basics.rs`
    - `apps/fret-cookbook/examples/customv1_basics.rs`
- [ ] Write the invariant matrix from ADR 0308 + current implementation.
  - Freeze what must not regress:
    - hook/key determinism,
    - explicit invalidation,
    - diagnostics visibility,
    - selector/query compatibility,
    - shared-model interop,
    - typed action write semantics.
- [ ] Classify current pressure into:
  - architecture question,
  - docs/adoption problem,
  - intentional hybrid/runtime-owned boundary.
- [ ] Compare the option set without coding first.
  - Keep `O0/O1/O2/O3` in scope until the matrix rejects them.
- [ ] Decide whether this lane should stop at “keep current contract” or open a prototype.
- [ ] Keep this lane from turning into generic authoring sugar growth.
  - No new prelude/root growth.
  - No Todo-only helper invention.
  - No macro growth.

## M0 — Open the lane correctly

- [x] Add the workstream docs directory and connect it from `docs/README.md`, `docs/roadmap.md`,
  and `docs/workstreams/README.md`.
- [ ] Freeze the reading list and evidence set.
- [ ] State the non-goals and hard constraints explicitly enough that future edits cannot quietly
  turn this into another density-only helper pass.

## M1 — Freeze invariants and ownership rules

- [ ] Record the non-negotiable invariants for local-state architecture.
- [ ] Record which parts of today's local-state story are already working and should be treated as
  settled.
- [ ] Record which current pains are:
  - genuine storage/ownership questions,
  - versus first-party adoption/teaching drift,
  - versus intentional explicit model/runtime seams.
- [ ] Freeze the layering rule:
  - lower portable crates do not learn about app-facing `LocalState<T>`.

## M2 — Compare architecture options

- [ ] Write the option matrix for `O0` / `O1` / `O2` / `O3`.
- [ ] Score each option against:
  - runtime determinism,
  - diagnostics,
  - selector/query layering,
  - widget bridge cost,
  - hybrid/advanced ownership honesty,
  - migration cost.
- [ ] Reject any option that requires hidden reactivity or two co-equal default stories.
- [ ] Choose one of:
  - keep current model-backed contract,
  - harden the facade only,
  - open a split-story prototype,
  - or open a self-owned prototype.

## M3 — Prototype only if justified

- [ ] If M2 chooses a code path, land the smallest proof surface first.
  - Require:
    - one default-path proof,
    - one hybrid proof,
    - one explicit advanced/runtime-owned non-goal boundary.
- [ ] Add the narrowest possible tests / diagnostics gates for the chosen direction.
- [ ] Delete displaced default-path wording if the chosen direction changes the taught contract.

## M4 — Close or spin out

- [ ] If the chosen direction is “keep current contract”, record the closeout audit and stop.
- [ ] If the chosen direction requires broader runtime contract change, update/add ADRs before
  broad rollout.
- [ ] Record which explicit raw/model seams remain intentional after the decision.
- [ ] Move any larger follow-on work into a new narrower lane instead of letting this TODO expand
  indefinitely.

## Standing rules

- [ ] No code refactor should land before the option matrix exists.
- [ ] No default-path sugar should be justified only from Todo pressure.
- [ ] No option should weaken explicit invalidation or diagnostics just to shorten syntax.
- [ ] No option should make shared `Model<T>` ownership look accidental or second-class.
