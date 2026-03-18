# Closeout Audit — 2026-03-16

This audit records the M4 closeout pass for the local-state facade boundary hardening lane.

Follow-on note (2026-03-17):

- a narrower post-closeout cleanup later moved direct `LocalState::update_action*` /
  `set_action(...)` helpers back to internal runtime substrate,
- while leaving the public default/raw/bridge classification unchanged otherwise.

Goal:

- verify whether the O1 follow-on still owns active implementation uncertainty,
- record whether the default/raw/bridge boundary now reads consistently,
- and decide whether any narrower follow-on lane is still required right now.

## Audited evidence

Core workstream docs:

- `docs/workstreams/local-state-facade-boundary-hardening-v1/DESIGN.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/TODO.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/MILESTONES.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/SURFACE_INVENTORY_2026-03-16.md`

Upstream decision context:

- `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/OPTION_MATRIX_2026-03-16.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`

Implementation / gate anchors:

- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`
- `docs/examples/todo-app-golden-path.md`
- `tools/gate_no_use_state_in_default_teaching_surfaces.py`

## Findings

### 1. The lane stayed narrow and solved the right problem

This lane opened only after `local-state-architecture-fearless-refactor-v1` had already closed on
O1.

That means its job was never:

- redesign local-state storage,
- reopen self-owned/plain-Rust prototypes,
- or add another helper-growth wave.

Its actual job was narrower:

> make the repo say the same thing everywhere about default local state, explicit raw-model access,
> and explicit bridge APIs.

Conclusion:

- the lane stayed correctly scoped as facade hardening rather than drifting back into architecture
  redesign.

### 2. The core boundary is now explicit in code, docs, and gates

The initial hardening batch now leaves the repo with one coherent reading:

- `LocalState<T>` is documented as the default app-facing local-state handle,
- `AppUiRawStateExt::use_state*` is documented as the explicit raw-model seam,
- `LocalState::{model, clone_model, *_in, watch_in}` are documented as explicit bridge APIs,
- `fret::advanced` now says directly that `AppUiRawStateExt` belongs on the advanced lane,
- and the default todo golden-path doc no longer reintroduces `use_state` as an example.

This is not only prose:

- `ecosystem/fret/src/view.rs` now contains test-backed wording for the boundary,
- `ecosystem/fret/src/lib.rs` now contains authoring-surface assertions for the advanced-lane
  placement,
- and `tools/gate_no_use_state_in_default_teaching_surfaces.py` still protects first-contact
  surfaces from regressing.

Conclusion:

- the repo now has the minimum hardening needed for the O1 public-facade contract.

### 3. No further export or storage-level move is justified from this lane

The initial review question for this lane was whether wording/gates were enough or whether the repo
still needed:

- export movement,
- API hiding,
- or another code-level surface split.

Current evidence says no:

- `AppUiRawStateExt` is already off `fret::app::prelude::*`,
- the advanced lane is already the discoverability home for the raw-model seam,
- the default docs/templates path is already protected,
- and the ambiguous middle was primarily a wording/classification problem, which is now addressed.

Conclusion:

- there is no immediate reason to keep this lane open as an active execution queue.
- later narrow cleanup was still allowed when evidence showed a direct action-aware `LocalState<T>`
  seam had no first-party proof as a separate public lane.

### 4. Remaining work is maintenance or future separate reduction only

After this batch, the remaining plausible future work falls into two categories:

1. maintenance
   - keep docs/rustdoc/tests aligned if surfaces drift again,
2. future separate reduction work
   - only if the repo later decides to shrink the explicit raw-model seam further.

That second category is not current debt.
It is a different decision and should reopen through a new, narrower lane only if fresh evidence
appears.

Conclusion:

- this lane should now be read as closed / maintenance evidence, not as an active implementation
  track.

## Decision from this audit

Treat `local-state-facade-boundary-hardening-v1` as:

- closeout complete,
- closed / maintenance by default,
- and reopenable only through a new narrower lane if future evidence shows that wording/gates are
  no longer sufficient.

## Immediate execution consequence

From this point forward:

1. keep `LocalState<T>` as the only default local-state story,
2. keep `use_state` explicit and advanced-lane only,
3. keep `LocalState::{model, clone_model, *_in, watch_in}` classified as bridge APIs rather than
   another co-equal default story,
4. and do not reopen this lane unless a new bounded follow-on is clearly justified.
