# Local-State Facade Boundary Hardening v1 — Milestones

Last updated: 2026-03-16

Related:

- Design: `docs/workstreams/local-state-facade-boundary-hardening-v1/DESIGN.md`
- TODO: `docs/workstreams/local-state-facade-boundary-hardening-v1/TODO.md`
- Surface inventory: `docs/workstreams/local-state-facade-boundary-hardening-v1/SURFACE_INVENTORY_2026-03-16.md`
- Local-state architecture closeout: `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `use_state` policy draft: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- ADR 0308: `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`

---

## Current status snapshot (as of 2026-03-16)

This lane exists only because the storage-model decision is already closed.

- **M0**: Met (the lane is opened and indexed as the O1 implementation follow-on).
- **M1**: Met (the initial surface inventory now classifies the surviving raw-model and bridge
  seams).
- **M2**: Met (the target wording now explicitly separates default local-state, advanced raw-model,
  and explicit bridge seams).
- **M3**: Met for the initial batch (the narrowest wording/export/gate hardening patch is now
  landed and verified).
- **M4**: Planned (close the lane once the boundary is stable).

Execution rule:

- treat this as boundary hardening, not another ergonomics expansion lane,
- and do not reopen storage-model questions from inside this tracker.

---

## Milestone 0 — Open the narrow lane

Exit target:

- the repo has one explicit place to harden O1 at the public-facade level,
- and the closed architecture lane no longer has to carry implementation hardening as a footnote.

Current result:

- this directory now exists,
- the main docs indices point to it,
- and the lane is framed as implementation hardening rather than architecture exploration.

## Milestone 1 — Freeze the seam inventory

Exit target:

- one inventory exists for the surviving local-state seam families,
- and the repo can distinguish default local-state, raw-model seam, and explicit bridge APIs.

Current result:

- `SURFACE_INVENTORY_2026-03-16.md` now records the initial classification,
- the app vs advanced prelude placement is captured explicitly,
- and the current gate picture is named up front.

## Milestone 2 — Freeze the target boundary state

Exit target:

- the repo can say, in one stable sentence each, what belongs to:
  - the default app lane,
  - the explicit raw-model lane,
  - and the explicit bridge lane.

Decision rule:

- prefer wording/rustdoc/gate clarity before export churn,
- and only change code placement if wording alone cannot express the boundary honestly.

Current result:

- `ecosystem/fret/src/view.rs` now classifies:
  - `LocalState<T>` as the default app-facing handle,
  - `AppUiRawStateExt::use_state*` as the explicit raw-model seam,
  - and `LocalState::{model, clone_model, *_in, watch_in}` as explicit bridge APIs.
- `ecosystem/fret/src/lib.rs` now states that `AppUiRawStateExt` lives on the advanced lane.

## Milestone 3 — Land the smallest hardening batch

Exit target:

- the smallest possible patch aligns:
  - public docs,
  - rustdoc,
  - and source-policy tests

with the chosen boundary wording.

Scope rule:

- keep the patch narrow,
- keep first-contact surfaces stable,
- and avoid mixing this batch with unrelated authoring cleanup.

Current result:

- `ecosystem/fret/src/view.rs` rustdoc is tightened without changing runtime semantics,
- `ecosystem/fret/src/lib.rs` authoring-surface tests now lock the advanced-lane wording for
  `AppUiRawStateExt`,
- `docs/examples/todo-app-golden-path.md` no longer shows `use_state` on a default teaching
  surface,
- validation passed via:
  - `cargo test -p fret --lib local_state_docs_classify_default_and_bridge_surfaces -- --nocapture`
  - `cargo test -p fret --lib advanced_prelude_reexports_app_facing_view_aliases -- --nocapture`
  - `python3 tools/gate_no_use_state_in_default_teaching_surfaces.py`

## Milestone 4 — Close cleanly

Exit target:

- the O1 public facade reads consistently,
- the lane no longer needs active tracking,
- and any leftover work is either maintenance or a separately named narrower patch.

Definition of done:

- default docs still teach one local-state story,
- explicit raw-model seams are clearly advanced,
- bridge APIs are classified rather than ambiguous,
- and no storage-model redesign has been smuggled back in.
