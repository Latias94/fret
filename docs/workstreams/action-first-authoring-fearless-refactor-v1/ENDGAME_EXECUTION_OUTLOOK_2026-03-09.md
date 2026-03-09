# Action-First Authoring + View Runtime (Fearless Refactor v1) — Endgame Execution Outlook (2026-03-09)

Status: draft, execution outlook
Last updated: 2026-03-09

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_ENDGAME_INDEX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

---

## Purpose

This note answers a narrower question than the endgame index or execution checklist:

> Given the repo's current evidence, which remaining old/compat surfaces are actually expected to
> be deleted, which are expected to stay, and which are conditional?

This is the repo's best current execution outlook, not a permanent contract.

---

## Outlook table

| Surface | Current most likely outcome | Confidence | Why | Revisit trigger |
| --- | --- | --- | --- | --- |
| `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` | **Delete or quarantine after the deprecation window** | High | No in-tree example/demo callers remain; default docs already converged; code is deprecated | Reach the documented time window and ship at least one published deprecated release |
| `run_native_with_compat_driver(...)` | **Keep for now; maybe quarantine later; unlikely near-term delete** | High | Real retained-driver / renderer / shell caller families still exist; deleting it now removes capability | Only if facade-size pressure rises and the repo is ready to move it behind an explicit compat boundary |
| `ViewCx::use_state::<T>()` | **Keep for now as explicit raw-model seam; possible later reduction, but not imminent** | Medium-high | Default teaching path already moved to `use_local*`; remaining question is facade policy, not migration debt | Only if the repo decides the public raw-model seam should shrink or gains a stronger replacement story |
| Command-first retained seams | **Keep as retained seams; only narrow future deprecations if specific APIs are chosen** | High | Default-facing alias pass is already done; remaining command-shaped surfaces are mostly mechanism/catalog or advanced/internal | Only if a default-facing leak reappears or a specific public API family is explicitly selected for deprecation |

---

## Practical reading

### 1) `App::ui*` is the only lane that currently looks genuinely headed toward removal

This is the most important practical conclusion.

The repo has already done the hard part:

- example/demo callers are gone,
- default docs are off the old path,
- code is deprecated,
- and the removal playbook exists.

So the remaining blocker is mostly release timing, not product uncertainty.

### 2) compat runner currently looks like a retained advanced seam, not future delete debt

The repo has now documented, gated, and audited that:

- it is non-default,
- it still serves real advanced caller families,
- and the realistic future reduction path is quarantine-first, not deletion-first.

That makes the current outlook clear:

- expect retention for now,
- do not plan near-term delete work around this seam.

### 3) `use_state` is not on the same track as `App::ui*`

`use_state` still has a live public/substrate role.

The repo has already solved the important part:

- users are no longer taught to start from it.

That means the remaining decision is a facade-shape question:

- keep one explicit raw-model seam, or
- eventually tighten the surface further.

This is a lower-pressure decision than app-entry cleanup.

### 4) command-first retained seams should be read as “maintenance mode unless specifically reopened”

This lane should not be treated as “the next deletion wave”.

The broad migration work is already done.
What remains is mostly:

- permanent mechanism/catalog contracts,
- retained advanced/internal surfaces,
- and potential future targeted deprecations only if the repo chooses them explicitly.

---

## Recommended repo stance from here

If the repo wants one blunt summary:

1. expect `App::ui*` to leave the default facade path for real once the time/release window is met,
2. expect compat runner to stay unless the repo intentionally performs a quarantine pass,
3. expect `use_state` to stay non-default until there is a stronger public raw-model story,
4. expect command-first retained seams to stay in maintenance mode unless a concrete deprecation
   target is named.

---

## Why this matters

Without this distinction, the repo risks spending cleanup time on the wrong class of work:

- arguing about deleting seams that are still intentionally useful,
- or delaying the one lane (`App::ui*`) that is actually closest to real removal.

This outlook exists to keep the next execution steps honest.
