# Diagnostics bundle schema v2 (semantics dedup + AI-friendly slices)

Status: Draft / in progress

## Goals

- Keep `bundle.json` shareable and AI-friendly for triage.
- Reduce `bundle.json` size by **deduplicating semantics** across frames.
- Preserve forward compatibility: older tools should still be able to read v1 bundles; newer tools should read v1/v2.

## Non-goals (initially)

- Perfect lossless reconstruction for every historical consumer. v2 may require updated tooling to view semantics.
- Replacing `bundle.json` with a database format. We stay JSON-first for now.

## Problem statement

Today `bundle.json` can be large because:

- `debug.semantics.nodes[]` is repeated per snapshot, and each node carries many default/empty fields.
- Script-driven runs may dump multiple snapshots where semantics changes minimally (or not at all).

We already reduce size by:

- Omitting default/empty fields in semantics serialization.
- Defaulting script-driven dumps to only include semantics on the last snapshot (`FRET_DIAG_BUNDLE_SEMANTICS_MODE=last`).

Schema v2 targets the remaining large class: **semantics repeated across many snapshots**.

## Proposed model

### Bundle schema version

- `bundle.json.schema_version = 2`

### Semantics table (dedup)

Store semantics snapshots in a bundle-level table keyed by `(window, semantics_fingerprint)`:

- `tables.semantics.schema_version = 1`
- `tables.semantics.entries[]`:
  - `window: u64`
  - `semantics_fingerprint: u64`
  - `semantics: UiSemanticsSnapshotV1`

Snapshots may omit `debug.semantics` entirely and rely on:

- `semantics_fingerprint` already present on `UiDiagnosticsSnapshotV1`
- `tables.semantics` for lookup

### Controls

- `FRET_DIAG_BUNDLE_SCHEMA_VERSION=1|2`
  - default: manual dumps `1`, script-driven dumps `2`
- `FRET_DIAG_BUNDLE_SEMANTICS_MODE=all|changed|last|off`
  - in v2, this controls inline semantics presence; table remains available unless `off`

## Compatibility expectations

- v1 tools keep working for v1 bundles.
- Updated tools:
  - `fretboard diag meta/query/slice/stats/compare` resolve semantics from either inline `debug.semantics` or `tables.semantics`.
  - `tools/fret-bundle-viewer` resolves semantics similarly.

## Risks / tradeoffs

- v2 introduces a second semantics storage location (table). Consumers must prefer inline semantics when present and fall back to the table.
- Dedup requires stable `semantics_fingerprint`. If absent, inline semantics must be kept.

## Evidence / tracking

See:

- `docs/workstreams/diag-bundle-schema-v2-todo.md`
- `docs/workstreams/diag-bundle-schema-v2-milestones.md`

