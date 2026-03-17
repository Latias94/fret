# Local-State Architecture — Invariant Matrix

Last updated: 2026-03-16

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
- `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- `ecosystem/fret/src/view.rs`

---

## Purpose

This matrix freezes the non-negotiable constraints for any future `LocalState<T>` architecture
decision.

It exists to prevent the lane from drifting into:

- helper growth disguised as architecture work,
- syntax-first experiments that weaken runtime explainability,
- or storage changes that silently invalidate existing diagnostics/ownership assumptions.

---

## Matrix

| Invariant | Why it is non-negotiable | Current evidence | Implication for future options |
| --- | --- | --- | --- |
| Stable hook/key identity must remain deterministic | View-local state cannot depend on unstable call order or implicit dynamic registration. | `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`; `ecosystem/fret/src/view.rs` (`use_state_keyed`, `local_keyed`, keyed view-state storage) | Any alternative storage model must preserve keyed identity semantics and loop-safety. |
| Explicit observation + invalidation semantics must remain visible | Fret intentionally does not hide dependency tracking behind an opaque reactive graph. | `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`; `ecosystem/fret/src/view.rs` (`watch`, `paint`, `layout`, `hit_test`) | Reject options that replace explicit observation/invalidation with hidden reactivity. |
| Dirty / notify behavior must stay diagnosable | Editor-grade performance and correctness depend on explaining why a view rerendered or a cache was reused. | `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`; `ecosystem/fret/src/view.rs` (`update_action`, `set_action`, redraw + notify path) | Any new local-state contract must keep rerender causes inspectable. |
| Shared `Model<T>` graphs remain first-class | Some state is intentionally cross-view, runtime-owned, or host-integrated and should not be disguised as private local state. | `docs/adr/0031-app-owned-models-and-leasing-updates.md`; `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`; `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md` | Even if default local state changes, explicit shared-model paths must remain supported and honest. |
| Lower portable crates must not depend on app-facing `LocalState<T>` | `fret-selector` / `fret-query` / lower mechanism crates must stay portable and layering-safe. | `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/SELECTOR_QUERY_DIRECTION_2026-03-16.md`; `ecosystem/fret/src/view.rs` (`LocalSelectorInputs` and the internal selector-deps bridge substrate) | Reject options that solve app-facing ergonomics by inverting crate layering. |
| Typed action write semantics must remain explicit | The authoring model already converged on typed actions; state writes cannot become hidden side effects. | `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`; `ecosystem/fret/src/view.rs` (`UiCxActionsExt`, `local_update`, `local_set`, `toggle_local_bool`) | Any future storage model must keep action dispatch and state mutation readable and auditable. |
| Hybrid/runtime-owned surfaces must stay expressible without lying | Some examples intentionally keep render-time host effects, viewport interop, or runtime-bound coordination explicit. | `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`; `apps/fret-cookbook/examples/virtual_list_basics.rs`; `apps/fret-cookbook/examples/customv1_basics.rs` | Reject options that force all surfaces to pretend they are plain local toy state. |
| Existing widget bridges count as contract evidence, not incidental hacks | Text/date/discrete-control bridges already encode where model-backed internals are intentional and where app-facing sugar is enough. | `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md`; `ecosystem/fret/src/view.rs` (`IntoTextValueModel for LocalState<String>`) | Future options must account for widget contract cost, not just view syntax. |
| Default path must remain single-track | The repo already closed the broad default-surface migration. It should not reintroduce two co-equal local-state stories. | `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`; `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md` | If a new direction is explored, the old and new stories cannot both remain default. |
| Pre-release freedom does not remove the need for reversibility | The repo can refactor fearlessly, but still needs a reviewable path with proof surfaces and gates. | `docs/workstreams/local-state-architecture-fearless-refactor-v1/DESIGN.md`; `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md` | No code path should open before the option matrix and proof plan exist. |

---

## Reading

What this matrix means in practice:

1. the lane is allowed to revisit storage/ownership shape,
2. but it is **not** allowed to weaken explicit invalidation, diagnostics, layering, or shared-model
   interop in exchange for shorter syntax,
3. and it is **not** allowed to treat hybrid/editor/runtime-owned surfaces as accidental residue.

---

## Immediate consequence for M2

Any option that fails one of the following should be rejected early:

- deterministic keyed identity,
- explicit observation/invalidation,
- diagnosable dirty/notify behavior,
- shared-model interop,
- portable selector/query layering,
- or a single default local-state story.
