# Local-State Facade Boundary Hardening v1 — TODO

Status: active narrow implementation lane
Last updated: 2026-03-16

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `SURFACE_INVENTORY_2026-03-16.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`

## Current priority checklist

- [x] Open the lane as an O1 follow-on rather than another state-architecture discussion.
- [x] Freeze the initial seam shortlist:
  - `AppUiRawStateExt::use_state*`,
  - `LocalState::{model, clone_model}`,
  - `LocalState::{read_in, value_in*, update_in*, set_in}`,
  - `LocalState::{watch_in, layout_in, paint_in, hit_test_in}`,
  - `fret::app::prelude::*` vs `fret::advanced::prelude::*`,
  - `tools/gate_no_use_state_in_default_teaching_surfaces.py`.
- [x] Record the initial classification in `SURFACE_INVENTORY_2026-03-16.md`.
- [x] Write the target boundary wording for:
  - default app lane,
  - explicit raw-model lane,
  - explicit bridge lane.
- [x] Land the smallest docs/rustdoc/source-policy patch batch that makes the three lanes read
  consistently.
  - 2026-03-16 initial batch:
    - `ecosystem/fret/src/view.rs` now labels `LocalState<T>` as the default app-facing handle and
      classifies `model()`, `clone_model()`, `*_in(...)`, and `watch_in(...)` as explicit bridge
      APIs,
    - `ecosystem/fret/src/lib.rs` now labels `AppUiRawStateExt` as an advanced-lane raw-model
      seam,
    - `docs/examples/todo-app-golden-path.md` no longer reintroduces `use_state` in a default
      teaching surface,
    - targeted crate tests plus `tools/gate_no_use_state_in_default_teaching_surfaces.py` now pass
      on the updated wording.
- [ ] Close the lane once the boundary is stable, or spin out one narrower follow-on if review
  finds a truly separate bounded patch.

## M0 — Open the lane correctly

- [x] Create the workstream directory and companion docs.
- [x] Connect the lane from `docs/README.md`, `docs/roadmap.md`, and `docs/workstreams/README.md`.
- [x] State explicitly that this lane does not reopen storage-model design.

## M1 — Freeze the surface inventory

- [x] Classify the current public surfaces into:
  - default local-state surface,
  - explicit raw-model seam,
  - explicit bridge API,
  - advanced/export placement rule.
- [x] Record the current gate and prelude placement facts.
- [x] Keep the inventory grounded in current code/docs rather than future API wishes.

## M2 — Freeze the target boundary state

- [x] Decide the contract sentence for each remaining seam family.
  - 2026-03-16 result:
    - `LocalState<T>` is the default app-facing local-state handle,
    - `AppUiRawStateExt::use_state*` is the explicit raw-model seam on the advanced lane,
    - `LocalState::{model, clone_model, *_in, watch_in}` are explicit bridge APIs.
- [x] Decide which docs/rustdoc surfaces must change first.
  - 2026-03-16 result:
    - start with `ecosystem/fret/src/view.rs`,
    - then `ecosystem/fret/src/lib.rs`,
    - then the default teaching surface that still mentioned `use_state`.
- [x] Decide whether any export/placement change is actually needed, or whether wording + tests are
  enough.
  - 2026-03-16 result:
    - wording + source-policy hardening is sufficient for the initial batch,
    - no export move is needed for the current O1 boundary.
- [x] Keep any future reduction path optional rather than implied.
  - 2026-03-16 result:
    - the lane hardens wording and gates first,
    - it does not imply deprecation or storage-model reduction from this batch.

## M3 — Land the narrowest hardening batch

- [x] Tighten the wording in `ecosystem/fret/src/view.rs` and related public docs if needed.
- [x] Add or refresh the narrowest source-policy tests protecting the intended lane split.
- [x] Keep `use_state` explicit and advanced-lane only.
- [x] Keep default-path docs/templates/examples on `use_local*` / `LocalState<T>`.

## M4 — Close or spin out

- [ ] Record the closeout once wording, exports, and gates align.
- [ ] If a remaining patch is still needed, spin it into a narrower lane with concrete code/gate
  scope.
- [ ] Do not let this TODO turn into another open-ended authoring ergonomics backlog.

## Standing rules

- [ ] No patch here may justify a new storage model.
- [ ] No patch here may widen `fret::app::prelude::*`.
- [ ] No patch here may reintroduce `use_state` into first-contact teaching surfaces.
- [ ] No patch here may delete legitimate advanced/component/runtime ownership bridges without a
  replacement story.
