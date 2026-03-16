# Local-State Architecture (Fearless Refactor v1) — Design

Status: active decision lane (contract-first; no code-level direction chosen yet)
Last updated: 2026-03-16

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/INVARIANT_MATRIX.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/OPTION_MATRIX_2026-03-16.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/SURFACE_CLASSIFICATION_2026-03-16.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/TODO.md`
- `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
- `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`

---

## 0) Why this lane exists

The broad authoring-reset lanes are now closed:

- `authoring-surface-and-ecosystem` closed the default app/component/advanced lane story,
- `into-element-surface` closed the conversion-vocabulary collapse,
- `authoring-density-reduction` closed the shorter default-path teaching surface,
- `action-first-authoring` closed the view-runtime + typed-action migration.

What remains is narrower and deeper:

> should `LocalState<T>` stay fundamentally model-backed as the long-term default local-state
> contract, or should Fret eventually move toward a stronger plain-Rust / self-owned local-state
> story?

This is not another “Todo ergonomics” pass.
It is a runtime/ownership/diagnostics question that now deserves its own lane.

---

## 1) Current implementation facts

Today, the shipped default path is:

- `use_local*` / `LocalState<T>` for default view-owned state,
- grouped action helpers for common local writes,
- explicit shared `Model<T>` graphs when ownership is cross-view or runtime-owned,
- explicit selector/query crates layered on top of that app-facing surface.

Current implementation shape:

- `ecosystem/fret/src/view.rs` defines `LocalState<T>` as a wrapper around `Model<T>`,
- `LocalState<T>` already owns the default read/write helper surface,
- `ViewCx::use_state` remains the explicit raw-model seam,
- default docs/templates/examples now teach `LocalState` first, not `use_state`,
- several intentional hybrid or advanced surfaces still rely on explicit model/runtime boundaries.

Important consequence:

- the remaining question is no longer “do we have a viable default local-state path?”
- the remaining question is whether the long-term storage/ownership contract behind that path
  should remain model-backed.

---

## 2) Goals

### G1 — Freeze the non-negotiable invariants first

Any future local-state direction must preserve:

- deterministic hook/key identity,
- explicit observed-dependency + invalidation semantics,
- diagnosable dirty/notify behavior,
- compatibility with typed action dispatch,
- compatibility with selectors/queries without inverting crate layering,
- and a clear boundary for shared `Model<T>` graphs.

### G2 — Separate architecture questions from authoring-surface sugar

This lane should answer:

- storage/ownership model,
- bridge rules,
- diagnostics implications,
- runtime boundary implications.

It should explicitly avoid smuggling those questions into:

- new helper names,
- prelude growth,
- macro growth,
- or Todo-only surface tweaks.

### G3 — Keep default-path and advanced-path ownership explicit

The lane must preserve the current surface split:

- default app path may stay opinionated and narrow,
- advanced/runtime-owned surfaces may keep explicit `Model<T>` or host/effect boundaries,
- component/query/selector crates must not be forced to depend on `LocalState<T>`.

### G4 — Make any future refactor reversible and evidence-based

If the repo chooses a new local-state contract later, it should land only after:

- one explicit option matrix,
- one invariant matrix,
- at least one default-path proof surface,
- at least one hybrid/advanced proof surface,
- and gates that explain what changed.

---

## 3) Non-goals

This lane is not for:

- widening `fret::app::prelude::*`,
- inventing Todo-only convenience helpers,
- revisiting app/component/advanced lane taxonomy,
- rewriting selector/query ownership,
- introducing a global implicit reactive graph or “signals everywhere” contract,
- deleting explicit shared `Model<T>` surfaces where shared ownership is still the point,
- or doing code refactors before the architecture decision is written down.

---

## 4) Hard constraints

Any acceptable direction must respect these constraints.

### C1 — Layering cannot invert

- `fret-selector` and `fret-query` stay portable.
- Any `LocalState<T>`-aware sugar belongs in the app-facing ecosystem layer, not in lower portable
  crates.

### C2 — Explicit invalidation remains a contract

The repo cannot quietly switch to opaque reactive semantics.

Even if local-state storage changes, Fret still needs:

- explicit observation,
- explicit invalidation intent,
- and diagnosable dirty/cache behavior.

### C3 — Shared-model interop must stay first-class

Even if the default local-state story becomes more self-owned, the framework still needs a clean
bridge to:

- shared `Model<T>` graphs,
- runtime-owned state,
- command/query/selector integration,
- host/effect surfaces,
- and editor-grade multi-view coordination.

### C4 — Editor-grade surfaces matter more than toy syntax

The winning direction is not the one that only shortens `hello world`.

It is the one that:

- still scales to virtualization, shared filters, docking/workspace coordination, async resources,
  and render-time host effects,
- while keeping the default app path boring and teachable.

---

## 5) Option set to evaluate

This workstream starts with four options.

### O0 — Keep model-backed `LocalState<T>` as the long-term contract

Meaning:

- no storage-model change,
- only docs/gates/bridge cleanup continue,
- the repo explicitly accepts model-backed local state as the long-term Fret stance.

### O1 — Keep model-backed storage, but harden the facade boundary

Meaning:

- storage remains model-backed,
- default path continues to hide raw model choreography better,
- explicit `use_state` / `clone_model()` / bridge seams become more clearly advanced-only.

### O2 — Introduce a split local-state story

Meaning:

- default view-local state may become self-owned or otherwise non-`Model<T>`-first,
- explicit shared-state paths remain model-backed,
- the framework provides an intentional bridge between the two.

### O3 — Move the default local-state contract to a self-owned/plain-Rust story

Meaning:

- the repo would treat model-backed `LocalState<T>` as transitional,
- and a future default local-state surface would be built around self-owned state with explicit
  bridges into shared `Model<T>` graphs.

This is the highest-risk option and should be considered only if the invariant matrix still works.

---

## 6) Evaluation matrix

Every option should be judged against the same questions:

1. Does it preserve deterministic hook/key identity?
2. Does it preserve explicit invalidation + dirty/cache semantics?
3. Does it keep typed action writes and redraw behavior understandable?
4. Does it keep diagnostics and scripted repros explainable?
5. Does it preserve a clean bridge to selectors and queries without layering inversion?
6. Does it preserve or improve text/widget bridges that already sit on the current `LocalState`
   path?
7. Does it still fit hybrid/runtime-owned surfaces without lying about ownership?
8. Does it improve the default app path enough to justify migration cost?
9. Can it be landed pre-release without leaving two co-equal default stories behind?

---

## 7) Initial evidence set

The workstream should start from three surface classes.

### Default-path evidence

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`

### Hybrid evidence

- `apps/fret-cookbook/examples/text_input_basics.rs`
- `apps/fret-cookbook/examples/date_picker_basics.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `apps/fret-cookbook/examples/virtual_list_basics.rs`
- `apps/fret-cookbook/examples/theme_switching_basics.rs`
- `apps/fret-cookbook/examples/customv1_basics.rs`

### Advanced/runtime-owned evidence

- explicit `use_state` / raw-model callers,
- editor/runtime-owned `Model<T>` coordination surfaces,
- any surface where render-time host effects, cross-view ownership, or retained interop are the
  point rather than a default-path teaching target.

---

## 8) Expected deliverables

Before any code-level direction is chosen, this workstream should produce:

- one invariant matrix,
- one surface classification note,
- one option comparison note,
- one decision note describing whether the repo keeps the current contract or opens a prototype,
- and, only if the decision is to refactor, the smallest proof-surface + gate plan.

---

## 9) Current recommended stance

Until this lane says otherwise:

- keep teaching `LocalState<T>` / `use_local*` as the default app-facing local-state path,
- keep `use_state` as the explicit raw-model seam,
- keep shared `Model<T>` graphs explicit,
- and do not widen state sugar just to simulate a decision that the repo has not actually made yet.

Decision update on 2026-03-16:

- `OPTION_MATRIX_2026-03-16.md` now recommends **O1**:
  keep model-backed storage, but harden the facade boundary.
