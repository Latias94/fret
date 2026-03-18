# Local-State Architecture — Surface Classification Audit

Last updated: 2026-03-16

Related:

- `DESIGN.md`
- `INVARIANT_MATRIX.md`
- `TODO.md`
- `MILESTONES.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md`

---

## Purpose

This audit classifies the current `LocalState<T>` pressure into three buckets:

1. real architecture questions,
2. docs/adoption or already-closed default-path work,
3. intentional hybrid/runtime-owned boundaries.

The goal is to stop treating every remaining `Model<T>` seam as evidence that the local-state
storage contract itself is wrong.

---

## Summary

Current classification:

- the **default app path is already converged** on `LocalState<T>` / `use_local*`,
- the **main unresolved question is architectural**, not default-path migration,
- and many surviving explicit-model seams are **intentional by ownership or widget contract**.

Practical reading:

- this lane should now focus on the long-term storage/ownership decision,
- not on reopening first-contact local-state migration.

---

## A. Real architecture questions

These are the questions that legitimately belong to this workstream.

| Topic | Why it is architectural | Evidence | Consequence |
| --- | --- | --- | --- |
| Should the long-term default `LocalState<T>` stay model-backed? | This is a storage/ownership/runtime contract decision, not a docs tweak. | `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`; `ecosystem/fret/src/view.rs` (`LocalState<T> { model: Model<T> }`) | This is the core question for M2. |
| What is the long-term status of `use_state`? | `use_state` is no longer a default teaching surface, but it is still a public raw-model seam and implementation substrate. | `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`; `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`; `ecosystem/fret/src/view.rs` (`AppUiRawStateExt`) | The lane must decide whether explicit raw-model access stays permanent or becomes a later reduction target. |
| How should a future self-owned story bridge back to shared models? | Any self-owned direction still needs clean interop with shared `Model<T>` graphs, selectors, queries, and widget contracts. | `docs/adr/0031-app-owned-models-and-leasing-updates.md`; `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md` | A future prototype must define bridges explicitly before code rollout. |

---

## B. Already-closed default-path pressure

These items should **not** be reopened as evidence that the default local-state path is still
missing.

| Surface class | Current reading | Evidence | Consequence |
| --- | --- | --- | --- |
| Starter/default docs and templates | Closed on `LocalState<T>` / `use_local*` | `docs/first-hour.md`; `docs/examples/todo-app-golden-path.md`; `apps/fretboard/src/scaffold/templates.rs`; `ecosystem/fret/src/lib.rs` source-policy tests | Do not use first-contact drift as a reason to reopen state-surface design unless new drift actually reappears. |
| Canonical todo compare set | Already converged enough to teach the current default local-state path | `apps/fret-examples/src/todo_demo.rs`; `apps/fret-cookbook/examples/simple_todo_v2_target.rs`; `apps/fretboard/src/scaffold/templates.rs` | Todo pressure alone cannot justify a new shared state API. |
| Default local-state read/write ergonomics | Already productized for the current model-backed contract | `ecosystem/fret/src/view.rs` (`TrackedStateExt`, `LocalStateTxn`, action-aware local writes); `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md` | The next question is architecture, not another helper pass. |
| Default `use_state` migration | Already complete on first-contact surfaces | `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md` | This lane should not spend time re-proving that `use_local*` is the default. |

---

## C. Intentional widget-boundary bridges

These seams are not necessarily evidence against the current local-state storage contract.

| Surface | Current reading | Evidence | Consequence |
| --- | --- | --- | --- |
| `Input` / `Textarea` | Narrow bridge landed; internals remain intentionally model-backed | `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md`; `ecosystem/fret/src/view.rs` (`IntoTextValueModel for LocalState<String>`) | Treat text widgets as contract evidence, not as a generic local-state failure. |
| `DatePicker` controlled path | Intentional controlled-widget bridge still uses explicit models | `apps/fret-cookbook/examples/date_picker_basics.rs`; `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md` | Keep it as hybrid evidence unless the widget contract itself changes. |
| Discrete widgets (`Checkbox`, `Switch`, `Toggle`) | Default-path parity is already closed through snapshot/action surfaces | `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md` | Do not reopen local-state architecture because of already-closed discrete-widget parity. |

---

## D. Intentional hybrid or runtime-owned surfaces

These examples should remain explicit unless the ownership model itself changes.

| Surface class | Why it stays explicit | Evidence | Consequence |
| --- | --- | --- | --- |
| Virtualization / scroll coordination | Collection identity and scroll/runtime coordination are part of the point | `apps/fret-cookbook/examples/virtual_list_basics.rs`; `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md` | Do not force these to look like pure local toy state. |
| Renderer/effect demos | Render-time host effects and capability checks are intentionally explicit | `apps/fret-cookbook/examples/customv1_basics.rs`; `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` | These are proof surfaces for ownership honesty, not local-state simplification targets. |
| Async/runtime/interop demos | Background execution, viewport interop, or host integration are the teaching goal | `apps/fret-cookbook/examples/async_inbox_basics.rs`; `apps/fret-cookbook/examples/embedded_viewport_basics.rs`; `apps/fret-examples/src/embedded_viewport_demo.rs` | Treat them as advanced/runtime-owned evidence. |
| Undo/router/payload/shared-state demos | Shared history, routing, or runtime synchronization is intentional | `apps/fret-cookbook/examples/undo_basics.rs`; `apps/fret-cookbook/examples/router_basics.rs`; `apps/fret-cookbook/examples/payload_actions_basics.rs` | These should not be used to argue that all state should become self-owned. |
| UI Gallery runtime driver / shell state | Gallery shell/app runtime is intentionally model-centric and app-owned | `apps/fret-ui-gallery/src/driver/runtime_driver.rs`; `apps/fret-ui-gallery/src/driver/render_flow.rs` | This is framework-host state, not default view-local state pressure. |

---

## E. Decision rule from this audit

The remaining local-state pressure should now be read as:

1. **architecture pressure** when it questions the storage/ownership contract behind
   `LocalState<T>` or the long-term role of `use_state`,
2. **non-architecture pressure** when it is just first-party wording/adoption drift on already
   closed default surfaces,
3. **intentional explicitness** when the surface is widget-controlled, hybrid, runtime-owned, or
   advanced by design.

---

## Immediate consequence

The next milestone should compare architecture options against the invariant matrix.

It should **not**:

- reopen default local-state migration,
- treat every `clone_model()` bridge as proof that the storage model is wrong,
- or try to erase intentional runtime/shared-state boundaries just to shorten examples.
