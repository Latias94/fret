# Action-First Authoring + View Runtime (Fearless Refactor v1) — App Entry Policy Decision Draft

Last updated: 2026-03-09

Related:

- Caller inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`
- Removal playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`

## Decision summary

Recommended decision:

- `App::view::<V>()` / `App::view_with_hooks::<V>(...)` become the **only default app-author entry path**.
- `App::ui(...)` / `App::ui_with_hooks(...)` are reclassified as **advanced bridge surfaces**, not part of the default teaching path.
- The repo should plan a staged transition of `ui(...)` / `ui_with_hooks(...)` from:
  1. documented-but-non-default advanced bridge,
  2. deprecated advanced bridge,
  3. removed (or moved behind an explicit compat boundary),
  once the remaining advanced demos either migrate to `View` or intentionally move to lower-level bootstrap/driver entry points.

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

## Deprecation window

Recommended window:

- **Deprecation start:** 2026-03-09
  - this is when the in-tree story became coherent enough to freeze the default path:
    - no in-tree example/demo callers remain on `App::ui*`,
    - `ecosystem/fret/src/app_entry.rs` already marks the closure-root entry methods as deprecated,
    - README/rustdoc policy tests already lock `view::<V>()` / `view_with_hooks::<V>(...)` as the
      only default path.
- **Earliest removal date:** 2026-06-09
  - equivalent to a **minimum 90-day downstream migration window**.
- **Additional release condition:** do not remove the APIs before **at least one published release**
  of `fret` has shipped with the deprecation warnings and updated docs.

Practical interpretation:

- The repo can keep cleaning docs, templates, and remaining advanced bridges immediately.
- It should **not** hard-delete `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` before both
  conditions are true:
  1. the date is on or after **2026-06-09**, and
  2. downstream users have had at least one published release carrying the deprecation.

Why this window is the recommended baseline:

- it is short enough to keep momentum,
- long enough to be defensible for a public facade surface,
- and it avoids turning “deprecated” into an indefinite state with no removal checkpoint.

---

## Exit criteria before deprecation starts

These criteria are now satisfied for the start of the deprecation window; they remain the
prerequisites for eventual hard delete:

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
- keep the deprecation warnings in place through the minimum window above.

### Stage 4 — Remove or quarantine

- preferred: remove the public `.ui(...)` surface from `fret`,
- fallback: quarantine it as explicitly advanced/compat if full removal is still too disruptive.
- earliest hard-delete checkpoint: **2026-06-09 + one published deprecated release**.
- use `APP_ENTRY_REMOVAL_PLAYBOOK.md` as the execution checklist once that checkpoint is reached.

---

## Practical verdict

If the product goal is still **“finish the migration and eventually hard-delete old interfaces”**, then the repo should **not** keep `App::ui(...)` as a co-equal long-term entry path.

The recommended decision is:

- **default = `view::<V>()`**
- **temporary bridge = `ui(...)`**
- **destination = deprecate/remove once advanced demo migration is complete**
