# Action-First Authoring + View Runtime (Fearless Refactor v1) — Hard-Delete Status Matrix

Last updated: 2026-03-09

Related:

- Endgame index: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_ENDGAME_INDEX.md`
- Gap analysis: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`
- Execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- App-entry inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`
- App-entry removal playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- Compat-driver inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- Compat-driver quarantine playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
- `use_state` inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `use_state` surface playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- Command-first widget audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
- Command-first retained-seam decision: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

---

## Purpose

This note compresses the current hard-delete situation into one matrix:

> Which old/compat surfaces are actually still blocking cleanup, which ones are already policy-closed,
> and which one is the next real implementation target?

For the quickest one-page entrypoint before opening this matrix, start with
`HARD_DELETE_ENDGAME_INDEX.md`.

---

## Current matrix

| Surface | In-tree migration status | Default-path status | Current decision state | Hard-delete readiness | Next real action |
| --- | --- | --- | --- | --- | --- |
| `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` | No in-tree example/demo callers remained before delete | Removed from default docs and removed from code | Decision executed: hard-delete pre-release because the surface never shipped publicly | **Closed** | None; keep `APP_ENTRY_REMOVAL_PLAYBOOK.md` only as historical execution evidence |
| `run_native_with_compat_driver(...)` | Still has 20 direct in-tree call sites across 3 real families | Explicitly non-default advanced interop seam | Policy draft says “keep for now” | **Deferred** | Do not force deletion; if future facade reduction is chosen, execute the quarantine-first move via `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md` |
| `ViewCx::use_state::<T>()` | 0 direct runtime/teaching-surface callers outside runtime/API substrate | Removed from starter/reference/default teaching path | Policy draft says “keep as explicit raw-model seam, non-default” | **Deferred** | Keep the default-path gate stable; if future facade reduction is chosen, use `USE_STATE_SURFACE_PLAYBOOK.md` to decide explicit-seam permanence vs deprecation |
| Command-first widget builders (`DropdownMenu*`, `ContextMenu*`, `Menubar*`, remaining app-facing command APIs) | Public alias pass landed; curated internal/app-facing residue is now also aligned (`tab_strip` overflow, GenUI shadcn overlay) | Still partially visible only on intentional advanced/internal surfaces | Policy direction is clear; remaining visible cases are now mostly intentional retained seams | **Lowest delete readiness, low remaining code pressure** | Keep intentional advanced surfaces explicit, follow `COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`, and only reopen migration on leak or deprecation |

---

## Practical reading

The four items no longer have equal weight.

### 1. `App::ui*` is no longer an active endgame lane

What remains is only historical evidence:

- why the facade could drop closure-root app entry pre-release,
- which docs/tests/gates keep it from drifting back,
- and where to look if someone proposes restoring it.

### 2. Compat runner is currently a product-boundary decision, not cleanup debt

The caller inventory now shows:

- retained chart/plot demos,
- low-level renderer/asset demos,
- advanced shell/diagnostics demos.

That means this surface should currently be treated as an intentional advanced seam unless the repo
chooses a separate quarantine effort.

### 3. `use_state` is now a facade-clarity question, not an adoption blocker

The important shift is:

- first-contact surfaces are already on `use_local*`,
- `use_state` survives mostly as explicit substrate/API.

So there is no strong reason to spend immediate implementation effort here unless the repo decides
that explicit raw-model hooks must eventually leave the public facade.

### 4. Command-first widget contracts are no longer a broad migration track

After the public alias pass and curated internal follow-up, this item now mostly has:

- intentional advanced/internal retained surfaces,
- targeted default-surface gates,
- and a smaller policy/deprecation question than before.

So the repo should not plan another wide implementation sweep here by default.
It should only reopen this track when a new default-facing leak appears or when compat APIs
actually enter deprecation/removal.

---

## Recommended order from here

1. Keep compat runner documented as advanced interop; do not spend near-term cleanup budget on it.
2. Keep `use_state` as non-default/raw-model and preserve the default-path gate.
3. Keep the command-first widget track in maintenance mode:
   - preserve the default-surface/docs gates,
   - treat the remaining advanced/internal surfaces as intentional unless product evidence says
     otherwise,
   - and only do more migration work when a new default-facing menu/item surface forces
     command-shaped naming.

---

## Verdict

If the goal is “finish migration and eventually hard-delete old interfaces”, the repo is now in a
much narrower endgame:

- one former blocker (`App::ui*`) is now closed pre-release,
- two current blockers (`compat driver`, `use_state`) are currently intentional retained seams,
- and one current blocker (**command-first widget contracts**) is now mostly an intentional-retention and
  future-deprecation-management track rather than obvious near-term implementation work.
