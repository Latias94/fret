# Action-First Authoring + View Runtime (Fearless Refactor v1) — Post-App-Entry Retained Seams Audit (2026-03-10)

Status: draft, maintainer audit
Last updated: 2026-03-10

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/ENDGAME_EXECUTION_OUTLOOK_2026-03-09.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`

---

## Purpose

After the pre-release hard delete of `App::ui*`, the next question is narrower:

> Are the remaining public retained seams actually delete-ready, or are they now intentional
> advanced/runtime boundaries?

This note answers that question only for:

- `ViewCx::use_state::<T>()`
- `fret::run_native_with_compat_driver(...)`

It is an internal maintainer audit, not a new user-facing migration note.

---

## Audit summary

| Surface | Current role | Delete readiness | Current recommendation | Reopen trigger |
| --- | --- | --- | --- | --- |
| `ViewCx::use_state::<T>()` | Explicit raw-model seam and runtime substrate for `use_local*` | Not ready | Keep as a non-default retained seam | Reopen only if the repo decides to shrink the raw-model facade or lands a stronger non-model local-state runtime |
| `run_native_with_compat_driver(...)` | Advanced low-level interop / runner seam for retained-driver callers | Not ready | Keep as a non-default retained seam; quarantine-first if facade reduction is later desired | Reopen only if caller families shrink materially and the repo chooses explicit facade reduction |

---

## `ViewCx::use_state::<T>()`

### Current role

- Public explicit raw-model hook on `ViewCx`.
- Still part of the runtime substrate because `use_local_with(...)` currently layers on
  `use_state_with(...)`.
- No longer part of the default teaching path.

### Evidence

- Runtime surface:
  - `ecosystem/fret/src/view.rs:550`
  - `ecosystem/fret/src/view.rs:559`
  - `ecosystem/fret/src/view.rs:586`
  - `ecosystem/fret/src/view.rs:591`
- Policy / inventory:
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- Default-path gate:
  - `tools/gate_no_use_state_in_default_teaching_surfaces.py`

### Audit conclusion

- The important migration goal is already complete: starter and default teaching surfaces now teach
  `use_local*`.
- The remaining question is not “why does this old surface still exist?”.
- The remaining question is “does the repo want to keep one explicit raw-model seam publicly?”

That makes `use_state` a retained explicit seam, not the next hard-delete candidate.

### Recommended next move

- Keep the current gate that prevents default-path reintroduction.
- Do not deprecate/delete this API as part of the current cleanup pass.
- Revisit only together with a real runtime/facade decision, not as isolated surface cleanup.

---

## `run_native_with_compat_driver(...)`

### Current role

- Advanced low-level interop path on the `fret` facade.
- Still used by retained-driver demos and low-level integration surfaces that intentionally keep a
  `WinitAppDriver`-shaped boundary.
- Already documented as non-default.

### Evidence

- Public facade surface:
  - `ecosystem/fret/src/lib.rs:608`
  - `ecosystem/fret/src/lib.rs:628`
  - `ecosystem/fret/src/lib.rs:654`
  - `ecosystem/fret/src/lib.rs:687`
- Policy / inventory:
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- Default-path gate:
  - `tools/gate_compat_runner_default_surface.py`

### Audit conclusion

- This surface still carries real capability for advanced callers.
- Deleting it now would remove functionality, not merely clean up wording or facade debt.
- If the repo later wants a smaller facade, the correct next step is quarantine/relocation, not
  direct removal.

That makes compat runner a retained advanced seam, not a pending delete lane.

### Recommended next move

- Keep the wording stable: advanced, low-level, interop-oriented, non-default.
- Do not spend cleanup effort on deletion until the repo intentionally chooses a facade-reduction
  pass.
- If that pass happens, start from `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`.

---

## Combined decision

After `App::ui*` removal, the repo should stop reading `use_state` and compat runner as “the next
obvious deletes”.

Current honest stance:

- `App::ui*` = closed historical hard-delete lane
- `use_state` = retained explicit raw-model seam
- compat runner = retained advanced interop seam

So the next cleanup pass should focus on:

- keeping default-path docs and gates stable,
- treating retained seams as policy-governed boundaries,
- and only reopening deletion/quarantine work when a separate product/runtime decision is made.
