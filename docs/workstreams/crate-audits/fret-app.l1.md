# Crate Audit (L1) — `fret-app`

Status: L1 complete (targeted deep dive + one new regression gate)

Supersedes: `docs/workstreams/crate-audits/fret-app.l0.md` (keep for the initial snapshot)

## Purpose

App runtime glue: models + globals, command registry wiring, settings/config loading, menu integration,
and effect draining into a convenient runner-facing surface.

## Audit focus (L1)

- Runner-owned concurrency posture (no forced async runtime; keep background work at the boundary).
- Global leasing discipline and re-entrancy guardrails (avoid “store/global borrowed while user code runs”).

## What changed (evidence-backed)

- Added regression gates ensuring global lease markers are not leaked on panic, and that mutations
  are preserved when unwinding resumes.
  - Evidence:
    - `crates/fret-app/src/app.rs` (`with_global_mut_does_not_leak_lease_marker_after_panic`)
    - `crates/fret-app/src/app.rs` (`with_global_mut_persists_user_mutations_and_restores_access_after_panic`)

## Hazards (top)

- Global re-entrancy borrow bugs (marker leaks causing globals to become permanently inaccessible).
  - Existing gates:
    - `crates/fret-app/src/app.rs` (`global_lease_tests`)
- Effect draining semantics drift (ordering, filtering, implicit side effects).
  - Existing gates:
    - `crates/fret-app/src/app.rs` (`menu_bar_effect_tests`, `command_enabled_effect_tests`)
  - Missing gate to add:
    - a small deterministic-order gate for `flush_effects` when combining redraw requests + explicit effects.

## Recommended next steps

1. Add fixture-driven decoding gates for persisted app-owned surfaces in this crate:
   - settings layering (`settings.json` merge policy),
   - dock layout file v1 (`DockLayoutFileV1`) compatibility behavior.
2. Consider splitting `src/app.rs` by responsibility (globals vs effects vs drags) after gates exist,
   keeping the stable facade surface unchanged.

