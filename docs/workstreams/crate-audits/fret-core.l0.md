# Crate Audit (L0) — `fret-core`

Status: L0 complete (quick scan; no deep dive yet)

## Purpose

Portable kernel vocabulary: IDs, time tokens, input event types, scene recording, docking model, text primitives.

## Snapshot (from `tools/audit_crate.ps1`)

- Largest files:
  - `crates/fret-core/src/dock/mutate.rs`
  - `crates/fret-core/src/scene/mod.rs`
  - `crates/fret-core/src/text/mod.rs`
  - `crates/fret-core/src/dock/layout.rs`
  - `crates/fret-core/src/dock/persistence.rs`
  - `crates/fret-core/src/input/mod.rs`
  - `crates/fret-core/src/semantics.rs`
- Public surface: `src/lib.rs` has many `pub mod`/`pub use` entries; keep exports intentional.
- Direct deps (external): `keyboard-types`, `serde`, `slotmap`, `web-time`
- Kernel forbidden deps spot check: ok (no obvious `winit`/`wgpu`/`tokio`/`web-sys` leaks)

## Hazards (top candidates)

- Dock mutation correctness (graph invariants, normalization rules) — must stay deterministic.
- Scene recording/validation drift — needs stable invariants tests.
- Input vocabulary sprawl in `input/mod.rs` — easy to become a dumping ground.
- Persistence formats (dock layout / settings) — accidental schema drift during refactors.

## Recommended next steps (L1 candidates)

1. Split `crates/fret-core/src/input/mod.rs` further by responsibility (pointer/ime/clipboard/external drop), keep facade minimal.
2. Add/strengthen invariants tests around dock normalization + persistence roundtrips.
3. Define a “serialization stability checklist” entry for `fret-core` persisted formats (ties to `BU-FR-core-014`).

