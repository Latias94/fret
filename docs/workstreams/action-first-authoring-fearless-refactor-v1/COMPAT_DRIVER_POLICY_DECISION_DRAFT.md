# Action-First Authoring + View Runtime (Fearless Refactor v1) — Compat Driver Policy Decision Draft

Last updated: 2026-03-12

Related:

- Caller inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- Gap analysis: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`
- Execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- Quarantine playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`

## Decision summary

Recommended decision:

- `run_native_with_compat_driver(...)` should **remain available**.
- It should be classified as an **advanced low-level interop / runner seam**, not as a default
  app-author entry path.
- It should **not** be treated as a near-term hard-delete target.
- The naked root `fret::run_native_with_compat_driver(...)` entry should be removed in favor of
  `fret::advanced::interop::run_native_with_compat_driver(...)`.

In short:

- **default app path** = `App::view::<V>()` / `App::view_with_hooks::<V>(...)`
- **advanced runner path** = `run_native_with_fn_driver(...)` family
- **advanced low-level interop path kept** =
  `fret::advanced::interop::run_native_with_compat_driver(...)`

---

## Why this is the recommended choice

## 1) The current caller set shows intentional use, not accidental leftovers

The inventory now shows three real in-tree caller families:

1. retained plot/chart demos,
2. low-level renderer / asset-pipeline demos,
3. advanced shell / diagnostics demos.

These are not the same thing as stale app-entry migration leftovers.

They indicate that `run_native_with_compat_driver(...)` is still carrying real value as a low-level
demo/interop seam.

---

## 2) Deleting it now would remove a real advanced capability, not just cleanup debt

Several callers still depend on explicit control over:

- `FnDriver` / retained `UiTree` ownership,
- raw event/render loop hooks,
- low-level image/scene/effect flows,
- multi-window shell wiring and diagnostics hooks.

That means deletion today would be a product decision with feature loss, not a cosmetic cleanup.

---

## 3) It should still stay out of the default teaching path

Keeping the surface does **not** mean promoting it.

The repo should continue to teach:

- `App::view::<V>()`
- `App::view_with_hooks::<V>(...)`
- `run_native_with_fn_driver(...)` family when advanced runner customization is needed

and describe `run_native_with_compat_driver(...)` only as:

- advanced,
- low-level,
- interop-oriented,
- non-default.

---

## Policy proposal

## A) Default teaching policy

For onboarding, templates, README-level first contact, and cookbook pages that teach the normal
authoring loop:

- do **not** recommend `run_native_with_compat_driver(...)`
- keep it out of the default path list except as an explicitly labeled advanced surface

---

## B) Advanced-surface policy

`run_native_with_compat_driver(...)` is allowed for:

- retained driver demos,
- low-level render/asset examples,
- advanced shell or diagnostics harnesses,
- bridge cases where the repo still wants a `WinitAppDriver`-shaped seam.

It should be documented as:

- advanced,
- low-level,
- interop/compat-oriented,
- not the preferred path for new app code.

---

## C) Future reduction policy

If the repo later wants to reduce the `fret` facade surface further, the preferred sequence is:

1. shrink or migrate the caller families,
2. introduce a clearer quarantine boundary if needed,
3. only then consider deprecation/removal.

If that future reduction path is chosen, use `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md` rather than
reconstructing the patch plan ad hoc.

Preferred future shape if reduction becomes desirable:

- keep default users on `App::view::<V>()`
- keep most advanced users on `run_native_with_fn_driver(...)`
- move `run_native_with_compat_driver(...)` behind a more explicit compat/interop naming boundary

What should **not** happen:

- immediate hard delete while the current caller families still exist,
- or continued ambiguous wording that makes the surface look half-default, half-legacy.

---

## Options considered

### Option 1 — Hard delete soon

Rejected for now.

Why:

- inventory shows real caller families,
- no replacement/quarantine plan exists yet,
- would remove working advanced demos for low-level scenarios.

---

### Option 2 — Keep permanently as-is

Not recommended as the final wording.

Why:

- function name and current docs still frame it too much as generic “compatibility” rather than a
  deliberate advanced interop seam,
- leaves the long-term facade shape ambiguous.

---

### Option 3 — Keep now, document clearly, reevaluate later

Recommended.

Why:

- matches current in-tree reality,
- avoids premature deletion,
- preserves one clear default path while still keeping an advanced escape hatch,
- keeps open the possibility of future quarantine if the caller families shrink.

---

## Required documentation alignment

The docs should consistently say:

- `run_native_with_compat_driver(...)` is **advanced low-level interop**
- it is **not** part of the default app-author path
- current hard-delete status is **defer / reevaluate later**, not “delete next”

That wording should be reflected in:

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/README.md`
- `HARD_DELETE_EXECUTION_CHECKLIST.md`
- `HARD_DELETE_GAP_ANALYSIS.md`

Status update (as of 2026-03-12):

- `ecosystem/fret/src/lib.rs` no longer exposes a naked root compat-runner function.
- `ecosystem/fret/src/interop.rs` now owns
  `fret::advanced::interop::run_native_with_compat_driver(...)`.
- `ecosystem/fret/README.md` and `tools/gate_compat_runner_default_surface.py` now align on the
  quarantined advanced path wording.

---

## Practical verdict

The correct current stance is:

- keep `run_native_with_compat_driver(...)`,
- label it as advanced low-level interop,
- quarantine it behind `fret::advanced::interop::run_native_with_compat_driver(...)`,
- remove the naked root entry from the default-facing facade,
- and stop treating it as the next obvious hard-delete item.

If the repo wants one clean sentence:

> `fret::advanced::interop::run_native_with_compat_driver(...)` is an intentionally kept advanced
> interop seam, not a default authoring path and not a near-term hard-delete candidate.
