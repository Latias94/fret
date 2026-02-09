# Crate Audit (L1) — `fret-core`

Status: L1 complete (targeted deep dive + one new regression gate)

Supersedes: `docs/workstreams/crate-audits/fret-core.l0.md` (keep for the initial snapshot)

## Purpose

Portable kernel vocabulary: IDs, time tokens, input event types, scene recording, docking model, text primitives.

## Audit focus (L1)

- Serialization stability for persisted docking layouts (schema drift risk).
- Dock layout validation invariants as a “contract guardrail” during internal refactors.

## What changed (evidence-backed)

- Added a JSON roundtrip regression gate for `DockLayout` (serde schema stability + `validate()`).
  - Evidence: `crates/fret-core/src/dock/tests.rs` (`dock_layout_json_roundtrips_and_validates`)
  - Dev dependency (tests only): `crates/fret-core/Cargo.toml` (`serde_json`)

## Hazards (top)

- Dock persistence format drift (field renames/defaults/versioning) silently breaking user state.
  - Existing gates:
    - `crates/fret-core/src/dock/tests.rs` (`dock_layout_json_roundtrips_and_validates`)
    - `crates/fret-core/src/dock/tests.rs` (`layout_roundtrips_floatings_with_rect_and_order`)
- Dock graph mutation correctness (normalization / invariants).
  - Missing gate to add:
    - invariant/property-style tests that exercise `DockOp` sequences and assert normalization holds.

## Recommended next steps

1. Add a “serialization stability checklist” entry for dock state (ties to BU-FR-core-014).
2. Expand dock persistence tests to cover:
   - multi-window layout import/export,
   - placement fields,
   - version mismatch behavior (explicit failure modes).
3. Split `crates/fret-core/src/input/mod.rs` further by responsibility if it keeps growing (keep facade minimal).

