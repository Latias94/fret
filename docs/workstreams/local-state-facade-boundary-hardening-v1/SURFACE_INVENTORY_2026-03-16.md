# Local-State Facade Boundary Hardening — Surface Inventory

Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`

---

## Summary verdict

The repo does not need another local-state architecture lane.

What it needs is a tighter classification of the surviving surfaces so reviewers can tell the
difference between:

- the default app-authoring path,
- the explicit raw-model seam,
- and the explicit bridge APIs that exist for ownership or helper-context reasons.

The current public surface is already close.
The remaining work is boundary clarity, not storage-model invention.

---

## Surface table

| Surface | Current exposure | Correct lane | Why it exists | Near-term action |
| --- | --- | --- | --- | --- |
| `AppUiRawStateExt::use_state*` | Public trait in `ecosystem/fret/src/view.rs`; reexported from `fret::advanced::prelude::*`; omitted from `fret::app::prelude::*` | Explicit raw-model lane | Gives intentional access to a direct `Model<T>` handle and remains runtime substrate for `use_local*` | Keep public for now, but keep wording and docs explicit that this is advanced/non-default |
| `LocalState::model()` / `clone_model()` | Public methods on app-facing `LocalState<T>` | Explicit bridge lane | Some widget/hybrid/runtime boundaries still intentionally need a `Model<T>` handle | Tighten wording so these read as bridges, not as a second normal local-state path |
| `LocalState::read_in(...)` / `value_in*` / `update_in*` / `set_in(...)` | Public methods on `LocalState<T>` | Explicit bridge lane | Needed for explicit `ModelStore` transactions and multi-state writes outside the default happy path | Document as store-transaction bridge APIs, not the default render-loop style |
| internalized `LocalState::update_action*` / `set_action(...)` | No longer public; runtime substrate only | Internal action-write substrate | The grouped `cx.actions()` helpers own the public rerendering-write story; the direct action-aware `LocalState<T>` seam never earned first-party proof as a separate lane | Keep internal unless a real advanced proving surface reappears |
| `LocalState::watch_in(...)` / `layout_in(...)` / `paint_in(...)` / `hit_test_in(...)` | Public methods on `LocalState<T>` | Explicit bridge lane | Helper-heavy `ElementContext` surfaces should be able to observe locals without dropping to `local.model()` manually | Keep, but classify as helper/interop bridge vocabulary |
| `WatchedState` + `TrackedStateExt` | Public tracked-read builder and extension trait | Explicit tracked-read lane | `state.layout(cx).value_*` / `state.paint(cx).value_*` are intentional user-facing read chains, and the same builder also owns explicit `observe()` / `revision()` / `read*()` escape hatches | Keep visible; do not rustdoc-hide this lane just because structural carrier nouns are hidden |
| `use_local*`, `LocalState::watch(...)`, `AppUiState::watch(...)`, `TrackedStateExt`, `cx.actions().locals::<A>(...)` | Public and already taught in docs/templates/examples | Default app lane | This is the shipped default local-state contract | Keep as the only default story; keep `AppUi::watch_local(...)` crate-private as implementation substrate |
| `AppUiState` / `AppUiActions` / `AppUiData` / `AppUiEffects` / `UiCxActions` / `UiCxData` / `LocalStateTxn` | Public only because grouped namespace methods and callbacks need structural carrier types | Structural carrier lane | App authors should discover these APIs from `cx.state()` / `cx.actions()` / `cx.data()` / `cx.effects()` and callback-local autocomplete, not by importing the carrier nouns | Keep public for signature reasons, but keep them rustdoc-hidden and protect that posture with source-policy tests |
| `fret::app::prelude::*` | Curated default prelude; omits `AppUiRawStateExt` and other low-level seams | Default app lane | Keeps first-contact autocomplete and docs narrow | Keep the omission policy and protect it with source-policy tests |
| `fret::advanced::prelude::*` | Reexports `AppUiRawStateExt` and other advanced runtime nouns | Advanced lane | Keeps low-level or host-owned seams explicit without polluting the app prelude | Keep as the discoverability home for explicit raw-model hooks |
| `tools/gate_no_use_state_in_default_teaching_surfaces.py` | Existing source-policy gate | Default-path protection | Prevents starter/reference surfaces from regressing back to `use_state` | Keep running it and widen only if a new default surface is added |

---

## Immediate reading

### 1. The default lane is already correct

The repo already converged on:

- `use_local*`,
- `LocalState<T>`,
- grouped app helpers,
- and tracked reads on the local handle.

This lane should not destabilize that path.

### 2. The raw-model seam is already mostly placed correctly

The strongest placement fact is already in-tree:

- `use_state*` is available on the advanced lane,
- not on the app prelude.

That means the next job is mostly wording and supporting gates, not a blind hard delete.

### 3. The bridge APIs are the ambiguous middle

The least clearly explained family is the one that sits on `LocalState<T>` itself:

- `model()` / `clone_model()`,
- `*_in(...)`,
- `watch_in(...)`.

These APIs are legitimate, but they are currently easier to misread as “also just normal
authoring” than as explicit ownership/context bridges.

That is the main hardening target for this lane.

---

## Initial patch preference

Prefer this order:

1. tighten wording and docs around the bridge APIs,
2. verify prelude/export/source-policy alignment,
3. add only the narrowest extra assertions if wording drift is still too easy,
4. change code placement only if the boundary still cannot be stated honestly afterward.
