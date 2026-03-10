# Action-First Authoring + View Runtime (Fearless Refactor v1) — App Entry Policy Decision Draft

Last updated: 2026-03-10

Related:

- Caller inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`
- Removal playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`

> Update (2026-03-10): the repo chose the **pre-release hard-delete** path. `App::{ui,
> ui_with_hooks, run_ui, run_ui_with_hooks}` are now removed from `fret`; any deprecation-window
> or published-release wording below is historical context only.

## Decision summary

Recommended decision:

- `App::view::<V>()` / `App::view_with_hooks::<V>(...)` become the **only default app-author entry path**.
- `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` were hard-deleted on **2026-03-10**
  before the first published `fret` release.
- Advanced closure-root integrations should use lower-level bootstrap/driver seams instead of
  reviving a parallel top-level entry story on `fret`.

This is the cleanest way to finish the authoring convergence without pretending every closure-style surface is already legacy-free.

---

## Why this is the recommended choice

## 1) One default mental model matters more than preserving two equal entry stories

The repo has already converged most user-facing docs/templates onto:

- `View`
- `ViewCx`
- `LocalState`
- typed actions

Keeping `ui(...)` and `view::<V>()` as equally recommended app-entry stories reintroduces the exact split mental model the refactor was meant to remove.

If the repo wants one boring default, the top-level entry story must also be single-track.

---

## 2) `ui(...)` is still useful, but that does not make it a default surface

`ui(...)` / `ui_with_hooks(...)` still have practical value:

- bridging older `ElementContext`-style demos,
- low-friction migration for advanced examples,
- narrow cases that still want closure-first wiring before a `View` wrapper is introduced.

That makes them good **bridge surfaces**.
It does **not** make them good default teaching surfaces.

The correct classification is therefore:

- **supported temporarily as advanced bridge APIs**,
- **not** first-contact APIs,
- **not** the surface docs/templates should optimize around.

---

## 3) Hard-deleting them immediately would be premature

There is still real in-tree usage in advanced demos.

Examples:

- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/external_texture_imports_demo.rs`

So the recommended path is **not** “delete now”.
The recommended path is:

- stop teaching them,
- label them precisely,
- migrate remaining advanced consumers,
- then deprecate/remove.

---

## Policy proposal

## A) Default authoring policy

For new app authors and all first-contact docs/templates:

- recommend `App::view::<V>()`
- recommend `App::view_with_hooks::<V>(...)` when driver hooks are needed
- do not recommend `App::ui(...)` / `App::ui_with_hooks(...)`

This policy is already mostly true in the docs and should become explicit project policy.

---

## B) Advanced bridge policy

`App::ui(...)` / `App::ui_with_hooks(...)` may remain temporarily for:

- advanced demos still being migrated,
- bridge code that has not yet been wrapped in `View`,
- cases where the better long-term answer may actually be a lower-level bootstrap/driver entry point rather than another top-level authoring API.

These surfaces should be documented as:

- advanced,
- non-default,
- subject to future deprecation/removal once migration is complete.

---

## C) Removal target

The end-state should be one of these two options:

### Option 1 (preferred)

- remove `App::ui(...)` / `App::ui_with_hooks(...)` from `fret`
- keep lower-level customization on:
  - `App::view_with_hooks::<V>(...)`
  - `run_native_with_fn_driver(...)`
  - `fret-bootstrap` / `fret-launch`

Why preferred:

- simplest user story,
- aligns facade with the actual v2 mental model,
- leaves bridge/interop complexity in advanced layers instead of the default facade.

### Option 2 (fallback)

- keep `App::ui(...)` / `App::ui_with_hooks(...)`, but quarantine them conceptually as advanced compatibility APIs
- never teach them in onboarding/templates again
- consider moving them under clearer naming in a future major if hard deletion is not feasible

Why this is second-best:

- keeps extra surface area forever,
- weakens the “one default app-entry path” story,
- makes future docs drift more likely.

---

## Pre-release deletion rule

The path that actually landed was simpler than the staged window:

- if no published `fret` release has shipped with the closure-root app-entry surface,
- and no in-tree callers remain,
- the repo should hard-delete the old entry path instead of carrying a temporary deprecation window.

That is the rule the repo applied on 2026-03-10.

---

## Exit criteria before hard delete

These criteria were satisfied before the hard delete landed:

1. remaining in-tree advanced demos using `ui(...)` / `ui_with_hooks(...)` are inventoried,
2. each such demo is classified as either:
   - migratable to `View`, or
   - should drop lower to bootstrap/driver APIs,
3. `fret` README + facade docs consistently call `view::<V>()` the default path,
4. at least one gate prevents new first-contact docs/examples from reintroducing `ui(...)` as the recommended path.

---

## Suggested staged execution

### Stage 1 — Docs policy lock (now)

- mark `ui(...)` / `ui_with_hooks(...)` as advanced bridge surfaces in the workstream docs,
- keep `view::<V>()` / `view_with_hooks::<V>(...)` as the only recommended facade entry path.

### Stage 2 — Consumer inventory

- list all in-tree callers of `ui(...)` / `ui_with_hooks(...)`,
- classify each caller as `migrate-to-view` or `move-lower-level`.

### Stage 3 — Gate + deprecation

- add a narrow gate that prevents first-contact docs/templates from teaching `.ui(...)`,
- or, if no public release has shipped yet, hard-delete the closure-root surface instead of
  carrying a temporary warning window.

### Stage 4 — Remove or quarantine

- preferred: remove the public `.ui(...)` surface from `fret`,
- fallback: quarantine it as explicitly advanced/compat if full removal is still too disruptive.
- the path actually taken on 2026-03-10 was the pre-release hard delete.
- use `APP_ENTRY_REMOVAL_PLAYBOOK.md` as the historical execution note for that landed patch.

---

## Practical verdict

If the product goal is still **“finish the migration and eventually hard-delete old interfaces”**, then the repo should **not** keep `App::ui(...)` as a co-equal long-term entry path.

The recommended decision is:

- **default = `view::<V>()`**
- **advanced customization = lower-level bootstrap / fn-driver seams**
- **closure-root `App::ui*` = removed pre-release**
