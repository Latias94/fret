---
title: Diagnostics bundle schema compatibility matrix
status: draft
date: 2026-02-23
scope: diagnostics, artifacts, schema
---

# Bundle schema compatibility matrix (v1 / v2)

This doc is the “source of truth” for what schema versions we accept for in-tree workflows, and what each tool expects.

The goal is pragmatic: keep **Option 1 (sidecar-first)** safe to refactor and safe for agentic triage, while we gradually remove
legacy/debt.

## Terms

- **Bundle schema**: the top-level `bundle.json` (or `bundle.schema2.json`) `schema_version`.
- **Inline semantics**: semantics stored under a snapshot (legacy layout), e.g. `windows[].snapshots[].debug.semantics.nodes`.
- **Semantics table**: semantics stored under `tables.semantics.entries[]` keyed by `(window, semantics_fingerprint)`; snapshots can
  omit inline semantics and keep only `semantics_fingerprint`.
- **Sidecars**: bounded JSON files generated from `bundle.json` (e.g. `frames.index.json`) used for fast triage and automation.

## Bundle schema versions

| Schema | Accepted by tooling | Semantics storage | Notes |
|---:|---|---|---|
| 1 | Yes | Inline semantics only | Historically common. Large bundles are more likely because semantics must be repeated per snapshot. |
| 2 | Yes | Inline semantics and/or semantics table | Preferred for “huge bundle” workflows. Enables semantics-table-only bundles plus bounded sidecars. |

## Sidecar schemas (in-tree)

| File | Kind | Schema | Producer | Primary consumers | Notes |
|---|---|---:|---|---|---|
| `bundle.index.json` | `bundle_index` | 1 | `fretboard-dev diag index` | selectors, inventories | Should be treated as required for stable automation. |
| `bundle.meta.json` | `bundle_meta` | 1 | `fretboard-dev diag meta` | preflight / triage | Includes bundle-level metadata used by `diag doctor` and agents. |
| `test_ids.index.json` | `test_ids_index` | 1 | `fretboard-dev diag test-ids-index` | selector suggestions | Optional but recommended for debugging selector drift. |
| `frames.index.json` | `frames_index` | 1 | `fretboard-dev diag frames-index` | lite triage / hotspots | Optional but recommended; bounded by design for agentic use. |

## Tool expectations / contracts

| Tool | Input(s) | Supported bundle schema | Required sidecars | Default-first path | Failure / warning behavior |
|---|---|---|---|---|---|
| `diag doctor` | bundle dir | v1, v2 | `bundle.index.json`, `bundle.meta.json` | sidecar-first | Warn on missing / invalid sidecars; warn on unexpected schema versions; suggest concrete repair commands. |
| `diag triage --lite` | bundle dir | v1, v2 | `frames.index.json` | lite | If missing, suggest `diag doctor --fix-sidecars`. |
| `diag hotspots --lite` | bundle dir | v1, v2 | `frames.index.json` | lite | Same as above. |
| `diag bundle-v2` | `bundle.json` / bundle dir | v1 → v2, v2 passthrough | none | conversion | Refuses oversized `bundle.json` by default; requires `--force` (to avoid OOM). |

## Enforcing the matrix

1. Keep this doc updated whenever we add a new schema version or sidecar format.
2. `diag doctor` must remain the “first step” and should surface schema drift (unexpected schema versions, legacy-only capture
   knobs) as actionable warnings.
3. Once in-tree workflows are consistently schema v2 + sidecar-first, we can delete legacy-only shims (see M4 in `todo.md`).

