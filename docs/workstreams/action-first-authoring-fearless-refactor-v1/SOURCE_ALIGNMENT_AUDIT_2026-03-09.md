# Action-First Authoring + View Runtime (Fearless Refactor v1) — Source Alignment Audit (2026-03-09)

Status: draft, source-vs-docs audit
Last updated: 2026-03-09

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_ENDGAME_INDEX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

---

## Purpose

This note records one narrow audit question:

> Do the current post-v1 cleanup decisions already line up with the repo's source-facing surfaces
> and release gates, or is there still drift between docs, crate README/rustdoc, and policy
> scripts?

---

## Summary

Current audit result:

| Lane | Docs / decision state | Source-facing state | Gate state | Audit result |
| --- | --- | --- | --- | --- |
| `App::ui*` | removed pre-release | `ecosystem/fret/README.md` + `ecosystem/fret/src/lib.rs` now document only `view::<V>()` / `view_with_hooks::<V>(...)` as the app-entry path | `tools/gate_fret_builder_only_surface.py` + in-crate `authoring_surface_policy_tests` | Aligned |
| compat runner | keep as advanced low-level interop seam for now | `ecosystem/fret/README.md` + `ecosystem/fret/src/lib.rs` already use advanced/non-default wording | `tools/gate_compat_runner_default_surface.py` now locks the wording + keeps first-contact docs free of compat-runner entrypoints | Aligned after this audit |
| `use_state` | keep as explicit raw-model seam, non-default | starter/reference code and `todo-app-golden-path` already moved to `use_local*` | `tools/gate_no_use_state_in_default_teaching_surfaces.py` + scaffold template assertions | Aligned |
| command-first retained seams | maintenance mode; reopen only on leak/deprecation | default-facing menu/snackbar/navigation aliases already landed; remaining command-shaped seams are catalog/advanced/internal | `tools/gate_menu_action_default_surfaces.py`, `tools/gate_menu_action_curated_internal_surfaces.py`, `tools/gate_material3_snackbar_default_surface.py` | Aligned |

Bottom line:

- the only real source-vs-docs gap found in this audit was the missing compat-runner default-path
  gate,
- that gap is now closed.

---

## Evidence notes

### 1) `App::ui*`

Aligned evidence:

- `ecosystem/fret/README.md`
- `ecosystem/fret/src/lib.rs`
- `tools/gate_fret_builder_only_surface.py`

Reading:

- docs and rustdoc now teach only `view::<V>()` / `view_with_hooks::<V>(...)` on `fret`,
- the closure-root bridge is gone from the public facade,
- and the gate now prevents both old root helpers and old builder-entry symbols from drifting back.

### 2) compat runner

Aligned evidence after this audit:

- `ecosystem/fret/README.md`
- `ecosystem/fret/src/lib.rs`
- `tools/gate_compat_runner_default_surface.py`
- `tools/pre_release.py`

Reading:

- the source-facing wording was already correct,
- but there was no dedicated policy gate protecting first-contact docs from drifting toward
  `run_native_with_compat_driver(...)`,
- this audit adds that gate and hooks it into the canonical pre-release runner.

### 3) `use_state`

Aligned evidence:

- `docs/examples/todo-app-golden-path.md`
- `tools/gate_no_use_state_in_default_teaching_surfaces.py`
- `apps/fretboard/src/scaffold/templates.rs`

Reading:

- the default local-state story is already protected both in docs and in generated templates,
- the remaining `use_state` mentions are now explicit runtime/substrate or migration/contract notes.

### 4) command-first retained seams

Aligned evidence:

- `tools/gate_menu_action_default_surfaces.py`
- `tools/gate_menu_action_curated_internal_surfaces.py`
- `tools/gate_material3_snackbar_default_surface.py`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

Reading:

- default-facing menu/material surfaces are already locked to the action-first spelling,
- retained command-shaped seams are now documented as intentional rather than unclassified residue.

---

## Audit verdict

As of 2026-03-09, the hard-delete / retained-seam workstream is now source-aligned enough that the
remaining risk is mostly future drift, not current contradiction.

The practical rule from here is:

- keep adding narrow gates only when a retained seam is supposed to stay out of the default path,
- avoid turning every retained advanced seam into a new repo-wide ban,
- and use the lane-specific playbooks/decision notes as the contract source for future cleanup.
