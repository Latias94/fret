# Action-First Authoring + View Runtime (Fearless Refactor v1) — Hard-Delete Status Matrix

Last updated: 2026-03-09

Related:

- Gap analysis: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`
- Execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- App-entry inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`
- Compat-driver inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- `use_state` inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- Command-first widget audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`

---

## Purpose

This note compresses the current hard-delete situation into one matrix:

> Which old/compat surfaces are actually still blocking cleanup, which ones are already policy-closed,
> and which one is the next real implementation target?

---

## Current matrix

| Surface | In-tree migration status | Default-path status | Current decision state | Hard-delete readiness | Next real action |
| --- | --- | --- | --- | --- | --- |
| `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` | No in-tree example/demo callers remain | Removed from default docs; deprecated in code | Policy mostly closed: deprecate now, remove/quarantine later after window | **Closest** | Wait for deprecation window + one published deprecated release, then decide delete vs compat quarantine |
| `run_native_with_compat_driver(...)` | Still has 20 direct in-tree call sites across 3 real families | Explicitly non-default advanced interop seam | Policy draft says “keep for now” | **Deferred** | Do not force deletion; only revisit if a future quarantine boundary or replacement path appears |
| `ViewCx::use_state::<T>()` | 0 direct runtime/teaching-surface callers outside runtime/API substrate | Removed from starter/reference/default teaching path | Policy draft says “keep as explicit raw-model seam, non-default” | **Deferred** | Keep default-path gate stable; revisit only if the repo wants to deprecate the raw-model seam itself |
| Command-first widget builders (`ContextMenu*`, `Menubar*`, remaining app-facing command APIs) | Partial alias migration done; menu-family core remains | Still partially visible on default-facing widget surfaces | Policy direction is clear, implementation still incomplete | **Lowest delete readiness, highest remaining code pressure** | Finish action-first alias pass on remaining menu-family/public builders and then gate default docs/examples |

---

## Practical reading

The four items no longer have equal weight.

### 1. `App::ui*` is no longer a migration problem

What remains is:

- deprecation timing,
- downstream release sequencing,
- and the final delete-vs-quarantine decision.

It is **not** the next meaningful implementation target unless the deprecation window has elapsed.

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

### 4. Command-first widget contracts are now the main remaining code-facing cleanup track

Unlike the other three items, this one still has:

- visible default-surface inconsistency,
- clear implementation-scoped follow-up,
- and a plausible narrow landing plan.

So if the repo wants one more **real code cleanup** step rather than another policy note, this is
the best candidate.

---

## Recommended order from here

1. Keep `App::ui*` in deprecation mode until the documented window expires.
2. Keep compat runner documented as advanced interop; do not spend near-term cleanup budget on it.
3. Keep `use_state` as non-default/raw-model and preserve the default-path gate.
4. Put the next real implementation pass on command-first widget aliases, especially:
   - `ContextMenu*`
   - `Menubar*`
   - any remaining default-facing menu/item builders that still force command-shaped naming.

---

## Verdict

If the goal is “finish migration and eventually hard-delete old interfaces”, the repo is now in a
much narrower endgame:

- one blocker (`App::ui*`) is mostly waiting on time/release policy,
- two blockers (`compat driver`, `use_state`) are currently intentional retained seams,
- and one blocker (**command-first widget contracts**) is the only remaining cleanup track that
  still looks like obvious near-term implementation work.
