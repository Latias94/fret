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

- `FRET_DIAG_BUNDLE_SEMANTICS_MODE=all|changed|last|off`
  - default: script-driven dumps `last`, non-script dumps `changed`
  - in v2:
    - this controls which snapshots keep semantics after resolution (inline `debug.semantics` if present, otherwise `tables.semantics`)
    - when dropping semantics for a snapshot, tooling writes an explicit `debug.semantics = null` sentinel so consumers do not fall back to the table
    - after applying the mode, `tables.semantics.entries[]` is pruned to only entries still referenced by snapshots that still have semantics
    - `off` drops both inline semantics and the semantics table

Additional size knobs (dump-time policies):

- `FRET_DIAG_BUNDLE_DUMP_MAX_SEMANTICS_NODES=<usize>`
  - caps exported `semantics.nodes[]` (applies to both inline semantics and `tables.semantics` entries)
- `FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY=0|1`
  - when enabled, export only nodes with `test_id` plus their ancestor closure (applies to both inline semantics and `tables.semantics`)
- `FRET_DIAG_BUNDLE_WRITE_INDEX=0|1`
  - controls writing agent/tool-friendly sidecars (`bundle.index.json`, `bundle.meta.json`, `test_ids.index.json`, plus `script.result.json` for script dumps)
  - default: enabled

Schema note:

- The runtime now always emits schema v2 bundles.
- Older schema v1 bundles remain readable by tooling; upgrade via `fretboard diag bundle-v2 <bundle_dir|bundle.json> ...` when needed.
  - The converter writes `bundle.schema2.json` and directory-based tooling prefers it when present.

## Compatibility expectations

- v1 tools keep working for v1 bundles.
- Updated tools:
  - `fretboard diag meta/query/slice/stats/compare` resolve semantics from either inline `debug.semantics` or `tables.semantics`.
  - `tools/fret-bundle-viewer` resolves semantics similarly.

## Risks / tradeoffs

- v2 introduces a second semantics storage location (table). Consumers must prefer inline semantics when present and fall back to the table.
- Dedup requires stable `semantics_fingerprint`. If absent, inline semantics must be kept.
- When semantics are aggressively filtered/capped, some consumers may require updated expectations (e.g. scripts that rely on non-test-id nodes).

## Evidence / tracking

See:

- `docs/workstreams/diag-bundle-schema-v2/diag-bundle-schema-v2-todo.md`
- `docs/workstreams/diag-bundle-schema-v2/diag-bundle-schema-v2-milestones.md`

Operational notes:

- `bundle.meta.json` now includes semantics table metrics (inline vs table-resolved counts, table entries, unique keys).
- Use `cargo run -p fretboard -- diag meta <bundle_dir|bundle.json|bundle.schema2.json> --meta-report` to print a compact human summary.
