# Local-State Architecture — Option Matrix

Last updated: 2026-03-16

Related:

- `DESIGN.md`
- `INVARIANT_MATRIX.md`
- `SURFACE_CLASSIFICATION_2026-03-16.md`
- `TODO.md`
- `MILESTONES.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`

---

## Decision summary

Recommended direction:

- **Choose O1**: keep model-backed storage, but harden the facade boundary.

Meaning:

- `LocalState<T>` remains model-backed for the current long-term contract,
- `use_local*` / `LocalState<T>` remains the only default local-state teaching story,
- `use_state` remains the explicit raw-model seam rather than becoming a co-equal default path,
- no self-owned/plain-Rust local-state prototype is justified right now,
- and no code-level architecture refactor should open from this lane unless new cross-surface
  evidence appears later.

Short verdict:

> the repo should stabilize and document the current model-backed local-state contract more
> explicitly, not replace it right now.

---

## Why O1 wins

O1 is the only option that currently satisfies all of the following at once:

- preserves deterministic hook/key identity without introducing a second storage regime,
- preserves explicit observation/invalidation and diagnosable dirty/notify behavior,
- keeps selector/query layering unchanged,
- keeps shared `Model<T>` interop first-class,
- stays honest about hybrid/runtime-owned surfaces,
- avoids reopening default-path migration that is already closed,
- and still answers the real unresolved question by clarifying the facade boundary around
  `LocalState<T>` vs `use_state`.

The current evidence does **not** show that model-backed storage itself is the next bottleneck.
It shows that:

- the default path already works,
- `use_state` needs to stay clearly non-default,
- and the remaining raw/model seams are mostly intentional bridges or advanced ownership boundaries.

---

## Option matrix

| Option | Summary | Invariant fit | Migration / implementation cost | Evidence fit today | Verdict |
| --- | --- | --- | --- | --- | --- |
| O0 | Keep model-backed `LocalState<T>` exactly as-is and stop here | Medium | Lowest | Partial | Rejected in favor of O1 |
| O1 | Keep model-backed storage, but harden the facade boundary and explicit raw seam story | High | Low | Strong | Recommended |
| O2 | Introduce a split local-state story (default self-owned-ish + explicit shared model bridge) | Medium-low | High | Weak | Reject for now |
| O3 | Move the default local-state contract to a self-owned/plain-Rust story | Low | Highest | Very weak | Reject for now |

---

## Detailed evaluation

### O0 — Keep model-backed `LocalState<T>` exactly as-is

What it gets right:

- zero implementation churn,
- preserves all current bridges and diagnostics assumptions,
- fully compatible with current widget/query/selector surfaces.

Why it is not enough:

- it under-answers the real remaining policy question,
- it leaves too much ambiguity around the long-term role of `use_state`,
- and it does not explicitly say whether the current boundary is intentional or just transitional.

Verdict:

- **reject as the final answer**
- because the repo still needs the boundary clarification captured by O1.

### O1 — Keep model-backed storage, but harden the facade boundary

What it means:

- keep `LocalState<T>` model-backed,
- keep `use_local*` / `LocalState<T>` as the only default local-state story,
- keep `use_state` as an explicit raw-model seam,
- keep widget bridges and hybrid/runtime-owned surfaces explicit,
- and stop treating the remaining pressure as evidence that another storage model must land now.

Why it fits the current evidence:

- default-path convergence is already closed,
- first-contact `use_state` migration is already done,
- `Model<T>`-heavy survivors are mostly intentional advanced/widget/runtime boundaries,
- and the current repo does not show repeated cross-surface evidence that only a self-owned storage
  model could solve.

Why it is better than O0:

- it gives the repo a real policy answer,
- it freezes the boundary more clearly,
- and it keeps future reopening conditions explicit instead of leaving the question vague.

Verdict:

- **recommended**

### O2 — Introduce a split local-state story

What it would mean:

- keep explicit shared-state/model paths,
- but add a second default-ish local-state storage regime for self-owned view state.

Potential upside:

- could move some app-local examples closer to a plain-Rust feel,
- may reduce visible “this is still a model handle underneath” discomfort.

Why it fails today:

- it risks reintroducing two co-equal local-state stories,
- it adds bridge complexity across widgets/selectors/queries,
- it complicates diagnostics and ownership teaching,
- and the current evidence does not show enough real surfaces blocked on this split.

Verdict:

- **reject for now**
- reopen only if new evidence shows the storage model itself, not merely the explicit seam policy,
  is now the limiting factor.

### O3 — Move the default contract to self-owned/plain-Rust local state

What it would mean:

- treat model-backed `LocalState<T>` as transitional,
- and move the default app-facing local-state story to a different storage contract.

Potential upside:

- this is the closest to the “plain-Rust/self-owned” north-star feel.

Why it fails today:

- highest migration and conceptual cost,
- strongest risk to dirty/notify explainability and widget bridges,
- unclear interop story for selectors/queries/shared models without adding a large bridge surface,
- and no current cross-surface evidence says this is the next justified move.

Verdict:

- **reject for now**

---

## Reopen conditions

The repo should reopen O2/O3 only if all of the following become true:

1. one real non-todo medium/default-facing surface still shows meaningful pressure after O1-style
   boundary hardening,
2. one hybrid/advanced surface also shows that the storage model itself is the blocker,
3. the proposed alternative still passes the invariant matrix,
4. and the repo can name a smallest proof surface + gate plan before touching implementation.

If those conditions are not met, do not open a storage-model prototype.

---

## Immediate execution consequence

The next step for this workstream is **not** M3 prototype work.

The next step is to close the lane cleanly on the O1 decision:

1. record that the current model-backed `LocalState<T>` contract stands,
2. record that `use_state` remains an intentional explicit raw-model seam,
3. keep default docs/templates/examples on `use_local*`,
4. and reopen only if new evidence later shows the storage model itself has become the bottleneck.
