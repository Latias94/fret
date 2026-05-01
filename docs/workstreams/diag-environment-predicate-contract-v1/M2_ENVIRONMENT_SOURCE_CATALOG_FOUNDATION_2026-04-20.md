# M2 Environment Source Catalog Foundation - 2026-04-20

Status: active implementation note

Related:

- `WORKSTREAM.json`
- `DESIGN.md`
- `M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `crates/fret-diag-protocol/src/lib.rs`
- `crates/fret-diag/src/lib.rs`
- `tools/gate_imui_workstream_source.py`

## Purpose

This note records the first code slice after the provenance/availability contract freeze.

The goal of this slice was narrow:

1. land the additive protocol types for an environment-source catalog,
2. land the parallel filesystem loader/provenance helpers in `crates/fret-diag`,
3. and stop before manifest syntax, preflight evaluation, or transport/session consumers.

## Landed result

### 1) `fret-diag-protocol` now has an explicit environment-source catalog family

The protocol now exposes:

- `EnvironmentSourceAvailabilityV1`
- `FilesystemEnvironmentSourceV1`
- `FilesystemEnvironmentSourcesV1`

This gives diagnostics one public serializable shape for source ids plus timing class without
pretending that environment facts are capabilities.

### 2) `fret-diag` now has a parallel loader/provenance seam

`crates/fret-diag/src/lib.rs` now mirrors the capability helper pattern with:

- `resolve_filesystem_environment_sources_path(...)`
- `read_filesystem_environment_sources_payload(...)`
- `normalize_filesystem_environment_sources(...)`
- `EnvironmentSourceCatalogProvenance`
- `resolve_filesystem_environment_sources_provenance(...)`
- `read_filesystem_environment_sources_with_provenance(...)`

The first filesystem catalog path is `environment.sources.json`, as frozen by the earlier decision
note.

### 3) Source-policy gates now freeze the code shape too

`tools/gate_imui_workstream_source.py` now asserts that:

- the workstream keeps the M2 contract note visible,
- the diagnostics protocol keeps the environment-source catalog types visible,
- and `crates/fret-diag` keeps the new catalog loader/provenance names visible.

## Important non-results

This slice intentionally did not:

- add `requires_environment`,
- evaluate environment predicates during campaign preflight,
- emit environment-source provenance in campaign results,
- publish a transport/session environment-source handshake,
- or reinterpret bundle env fingerprints as preflight-ready facts.

Those remain future work.

## Why this is the correct stopping point

The repo needed code-level contract anchors before any manifest or execution work could be honest.

Landing the protocol and loader foundation first keeps the next slice additive and reversible:

- manifest work can bind to explicit source ids and availability classes,
- diagnostics can keep capability provenance and environment-source provenance separate,
- and future consumer work can start from one named catalog path instead of improvising file names.

## Evidence

- Protocol types:
  - `crates/fret-diag-protocol/src/lib.rs`
- Diagnostics loader/provenance:
  - `crates/fret-diag/src/lib.rs`
- Source-policy gate:
  - `tools/gate_imui_workstream_source.py`

## Verification

- `cargo nextest run -p fret-diag --lib environment_source --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `git diff --check`
