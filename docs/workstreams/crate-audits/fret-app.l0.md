# Crate Audit (L0) — `fret-app`

Status: L0 complete (quick scan; no deep dive yet)

## Purpose

App kernel glue: “Models + Commands + Effects” plumbing, app settings surfaces, and portable helpers that sit above `fret-runtime`.

## Snapshot (from `tools/audit_crate.ps1`)

- Largest files:
  - `crates/fret-app/src/app.rs`
  - `crates/fret-app/src/core_commands.rs`
  - `crates/fret-app/src/settings.rs`
- Direct deps (workspace): `fret-core`, `fret-runtime`
- Direct deps (external): `serde`, `serde_json`, `slotmap`, `thiserror`
- Kernel forbidden deps spot check: ok (no obvious executor/backend deps)

## Hazards (top candidates)

- Settings serialization stability (schema drift, defaults, back-compat).
- Core command registration drift (IDs, scopes, default keybindings/menus alignment).
- Effect execution contracts (what must be handled by runner vs app).

## Recommended next steps (L1 candidates)

1. Directory-module split of `app.rs` if it keeps growing (separate plumbing vs helpers vs tests).
2. Add explicit “settings schema stability” tests for key persisted surfaces (ties to `BU-FR-core-014`).

