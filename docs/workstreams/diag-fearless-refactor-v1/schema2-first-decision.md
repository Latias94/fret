---
title: Diagnostics Fearless Refactor v1 (Schema2-first Decision Draft)
status: draft
date: 2026-02-24
scope: diagnostics, bundle-artifacts, schema-evolution, ai-first
---

# Schema2-first policy (Plan 1)

This note proposes a conservative path to make **schema2-first** the default for day-to-day
diagnostics workflows without breaking existing tooling or offline viewer usage.

Terminology:

- **Raw bundle artifact**: `bundle.json` (may be large; historically the only on-disk artifact name).
- **Schema2 bundle artifact**: `bundle.schema2.json` (portable, compact, and preferred for AI/sidecar-first loops).
  - Today, this is typically **tooling-derived** via `fretboard-dev diag doctor --fix-schema2` (mode=`last`).

## Current behavior (as of 2026-02-24)

- The runtime writes a JSON bundle artifact named `bundle.json` (schema_version=2; semantics are table-capable).
- Tooling can generate `bundle.schema2.json` from `bundle.json`:
  - `fretboard-dev diag doctor --fix-schema2 <bundle_dir> --warmup-frames <n>`
  - This conversion applies a semantics mode (currently `last`) and prunes the semantics table to referenced entries.
- Most day-to-day triage can avoid materializing raw `bundle.json` by using sidecars and AI packets:
  - `bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`, `frames.index.json`, `ai.packet/`, `--ai-only` zips.
  - Tooling should treat sidecars as *bundle-artifact scoped*; when the bundle artifact path changes (e.g. switching from
    `bundle.json` to `bundle.schema2.json`), sidecars should be regenerated to avoid mismatches.

## Goals

1. Make “share to AI” bounded-by-default (already done via `--ai-only`).
2. Make “offline viewer-friendly share” prefer `bundle.schema2.json` when available.
3. Reduce the number of workflows that *require* raw `bundle.json`:
   - raw remains supported for deep debugging, but stops being a default dependency.

## Policy (decision)

This section is the **current recommended policy** for in-repo workflows (2026-02-28). It is intentionally conservative:
it makes tool-launched runs small-by-default without forcing a breaking change on manual/ad-hoc dumps.

### Tool-launched runs (`fretboard-dev diag ... --launch`)

- Tooling MUST write a per-run `diag.config.json` under `--dir` and set `FRET_DIAG_CONFIG_PATH` to it.
- If tooling cannot write `diag.config.json`, treat it as a launch failure (avoid silently falling back to runtime defaults).
- Default per-run config for launched runs SHOULD be small-by-default:
  - `write_bundle_json=false`
  - `write_bundle_schema2=true`
  - `script_dump_max_snapshots<=10`
  - `max_debug_string_bytes<=2048`

### Manual/ad-hoc runs (no tooling)

- If no config file is present, keep runtime defaults compatibility-first (raw `bundle.json` may be written).
- If you want small-by-default manual dumps, use `FRET_DIAG_CONFIG_PATH` and set:
  - `write_bundle_json=false`
  - `write_bundle_schema2=true`
  - (recommended) `script_dump_max_snapshots<=10`
  - (recommended) `max_debug_string_bytes<=2048`

## Proposed knobs (capture-time)

### A) Keep writing `bundle.json` (compat), but optionally emit `bundle.schema2.json` at capture time

Rationale:

- Tooling-derived schema2 requires reading/parsing the raw artifact (which may be huge).
- Runtime already has the structured bundle in memory; writing a compact schema2 companion is cheaper and more reliable.

Proposed knobs (config-file driven; keep conservative and internal until proven):

- Config file key: `write_bundle_schema2=true`
  - When enabled, the runtime writes `bundle.schema2.json` for dumps.
  - The schema2 artifact should be generated using the same semantics mode as tooling uses today (`mode=last`).
- Config file key: `write_bundle_json=false`
  - Optional: for scripted runs, allow skipping large raw `bundle.json` emission.
  - Keep manual dumps writing raw by default unless explicitly configured.

### B) Tooling defaults

- Keep accepting `<bundle_dir|bundle.json|bundle.schema2.json>` everywhere.
- Prefer `bundle.schema2.json` as the **bundle artifact** input when both exist (where it is sufficient).
- `diag doctor --fix-schema2` stays as a self-heal fallback, even if the runtime can emit schema2.
- When tooling launches an app (`--launch`) for schema2/AI-focused flows, it should default to enabling
  runtime schema2 emission (`write_bundle_schema2=true`) unless the caller already set an explicit value.

## Exit criteria to treat raw `bundle.json` as “optional” for common flows

We should only move “raw is optional” from docs into defaults when all are true:

1. The offline viewer can load `bundle.schema2.json` as a primary artifact (already true).
2. Sidecar + AI packet paths are sufficient for first-pass triage:
   - `frames.index.json` exists and `triage --lite` / `hotspots --lite` work.
   - `ai.packet/` is generated automatically in `run|pack|repro` workflows (`--ai-packet` / `--ai-only`).
3. `diag pack --include-all --pack-schema2-only` works without requiring `bundle.json` in the share zip.
4. Any remaining “deep debugging” workflows that truly require raw `bundle.json` are documented as such.

## Exit criteria to flip manual defaults (future)

Flipping manual dumps to be small-by-default is higher risk because manual workflows often rely on “whatever dumps land on
disk” without tooling context. Do not flip until all are true:

1. `docs/ui-diagnostics-and-scripted-tests.md` clearly documents the new default and the “deep debug” escape hatch.
2. At least one “materialize raw on demand” path exists (tooling or runtime) for the rare cases that truly need raw JSON.
3. Common CLI tooling flows (`meta/query/slice/triage/ai-packet/pack/repro`) remain schema2-first and do not regress to
   raw-only assumptions.
4. `diag config doctor --mode manual` provides a clear warning when manual runs are about to produce large artifacts
   (help users self-correct without reading this doc).

## Risks / cautions

- Writing two bundle artifacts can increase disk IO; make it opt-in first.
- A schema2 artifact generated at runtime must remain compatible with tooling’s expectations
  (schema_version=2 + semantics table + semantics mode application).
- We should not remove tooling support for schema v1 bundles for a long time.

## Follow-ups (tracked in TODO)

- Add a runtime option to emit `bundle.schema2.json` (opt-in).
- Decide whether scripted runs should default to emitting schema2 (and whether raw can be skipped).
- Tighten docs to clearly separate “schema v2” (schema_version=2) from “schema2 file name” (`bundle.schema2.json`).
