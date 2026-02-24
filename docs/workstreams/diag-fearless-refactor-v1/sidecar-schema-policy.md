---
title: Diagnostics Fearless Refactor v1 (Sidecar Schema Policy)
status: draft
date: 2026-02-22
scope: diagnostics, artifacts, schema, compatibility
---

# Sidecar schema policy

This note defines how diagnostics **sidecars** evolve over time.

Sidecars are intentionally treated as **tools/agents accelerators**:

- they are optional,
- they must be regeneratable from `bundle.json`,
- they must evolve forward-compatibly (additive-only by default).

## Directory placement and packing

For native filesystem dumps:

- Sidecars live next to `bundle.json` in the bundle directory.

For packed zip artifacts:

- Sidecars may also appear under `_root/` (tools should accept both layouts).

## Required top-level fields

Every sidecar JSON object should include:

- `kind`: a stable identifier for the sidecar payload (e.g. `bundle_index`, `bundle_meta`, `test_ids_index`, `frames_index`).
- `schema_version`: an integer schema version for that `kind`.
- `warmup_frames`: the warmup frame count the sidecar was generated with (treat mismatches as invalid).
- `bundle`: a string label/path for traceability (best-effort).

Recommended (when available):

- `generated_unix_ms`: generation timestamp.
- `tool`: generator identifier/version when emitted by CLI tooling.

## Evolution rules

### Additive-only by default

Within a fixed `(kind, schema_version)`:

- new fields may be added,
- existing fields must not change meaning,
- removing or renaming fields is not allowed.

### Breaking changes require a schema bump

If a change requires altering field meaning, deleting fields, or changing key shapes:

- bump `schema_version` for that `kind`, and
- keep older tools working by either:
  - continuing to write the previous schema for a transition period, or
  - regenerating on-demand from `bundle.json` when the sidecar schema is unsupported.

### Unknown fields must be ignored

Readers (CLI + tooling) must ignore unknown fields to allow safe additive growth.

## Missing sidecars behavior

Tools must not hang or silently degrade into timeouts when sidecars are missing.

Preferred behavior order:

1. Regenerate sidecars from `bundle.json` (best-effort).
2. If regeneration is not possible or too expensive, fail fast with guidance:
   - which file is missing,
   - which command regenerates it (e.g. `fretboard diag index <bundle_dir|bundle.json|bundle.schema2.json>`),
   - and what capability/env setting enables sidecar writing during dumps.

## Current sidecars (v1)

- `bundle.index.json` (`kind=bundle_index`, `schema_version=1`): snapshot selectors + semantics bloom accelerators.
- `bundle.meta.json` (`kind=bundle_meta`, `schema_version=1`): high-level counters and uniqueness summaries.
- `test_ids.index.json` (`kind=test_ids_index`, `schema_version=1`): catalog of known test-ids.
- `frames.index.json` (`kind=frames_index`, `schema_version=1`): per-frame lightweight stats + selectors.
  - Uses a columnar encoding: `columns[]` + per-window `rows[]` where each row is an array aligned to `columns`.
  - `semantics_source_tag`: `0=none`, `1=inline`, `2=table`.

## Scope limits

This policy covers only sidecars intended for:

- snapshot lookup/indexing,
- test-id discovery,
- script evidence correlation.

It does not define `bundle.json` itself; that contract is described in:

- `docs/workstreams/diag-fearless-refactor-v1/minimum-useful-bundle.md`
