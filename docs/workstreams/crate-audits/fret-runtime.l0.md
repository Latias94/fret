# Crate Audit (L0) — `fret-runtime`

Status: L0 complete (quick scan; no deep dive yet)

## Purpose

Portable runtime contracts: models, effects, commands/keymap/menubar, window services and capability signals.

## Snapshot (from `tools/audit_crate.ps1`)

- Largest files:
  - `crates/fret-runtime/src/model/store.rs`
  - `crates/fret-runtime/src/capabilities.rs`
  - `crates/fret-runtime/src/when_expr.rs`
  - `crates/fret-runtime/src/window_command_gating/tests.rs`
  - `crates/fret-runtime/src/input.rs`
  - `crates/fret-runtime/src/keymap/tests.rs`
  - `crates/fret-runtime/src/menu/wire/patch_v2.rs`
- Direct deps (workspace): `fret-core`, `fret-i18n`
- Direct deps (external): `serde`, `serde_json`, `slotmap`, `thiserror`
- Kernel forbidden deps spot check: ok (no obvious executor/backend deps)

## Hazards (top candidates)

- `Model` correctness under leases (re-entrancy, lock ordering, notify semantics).
- Command gating snapshot semantics (stack behavior, fallback rules) — high risk of subtle regressions.
- Wire format stability for keymap/menubar/settings JSON.
- Capability naming/typing drift (string keys used as bools, etc.).

## Recommended next steps (L1 candidates)

1. (Done) Convert the model store into a `model/` directory module split by responsibility.
   - Evidence: `crates/fret-runtime/src/model/mod.rs`
2. (Done) Split menubar wire formats into focused submodules (`menu/wire/*`) for v1/v2 + patch/config decoding.
   - Evidence: `crates/fret-runtime/src/menu/wire/mod.rs`
3. Add a v1 “serialization stability checklist” item for `fret-runtime` config surfaces (ties to `BU-FR-core-014`).
