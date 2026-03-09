# Run Manifest `bundle_json` Chunk Index Contract V1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/BUNDLE_ARTIFACT_ALIAS_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`

## Purpose

This note defines what Layer A run-manifest `bundle_json` means today.

The goal is to remove ambiguity before any manifest-level rename is attempted. The immediate
question is not "should everything become `bundle_artifact`?" The immediate question is whether the
existing run-manifest `bundle_json` subtree is:

- a legacy artifact-path name that should be renamed soon, or
- an intentionally format-specific chunk index for raw `bundle.json` recovery and linting.

## Decision

For the current compatibility window, treat run-manifest `bundle_json` as an intentionally
format-specific **raw bundle chunk index**, not as the generic canonical artifact-path field.

This means:

1. `paths.bundle_artifact` remains the canonical manifest field for locating the main bundle
   artifact.
2. top-level manifest `bundle_json` remains the persisted chunk-index node for the raw
   `bundle.json` alias and its recovery metadata.
3. `chunks/bundle_json/...` remains the expected on-disk chunk directory shape for that raw-bundle
   recovery flow.
4. Layer B payload cleanup around `bundle_artifact` does **not** imply a Layer A rename.

## Why this is the right contract

### 1. Layer A already encodes two different concerns

Key anchors:

- `crates/fret-diag/src/run_artifacts.rs:118`
- `crates/fret-diag/src/run_artifacts.rs:177`

Current state:

- `paths.bundle_artifact` already carries the canonical artifact-location meaning.
- top-level `bundle_json` stores chunking metadata (`mode`, `total_bytes`, `chunk_bytes`, `blake3`,
  `chunks[]`) rather than a plain artifact path.

Interpretation:

- These two nodes are not duplicates.
- They represent different semantics:
  - `paths.bundle_artifact` answers "where is the artifact?",
  - `bundle_json` answers "how can the raw bundle alias be reconstructed and validated?"

### 2. In-tree consumers already treat `bundle_json` as a chunk index

Key anchors:

- `crates/fret-diag/src/artifact_lint.rs:474`
- `crates/fret-diag/src/commands/doctor.rs:718`

Current state:

- `artifact_lint` validates `bundle_json.chunks`, per-chunk hashes, total bytes, and total hash.
- `doctor` summarizes the same `bundle_json.chunks` subtree to diagnose missing or mismatched
  materialized raw bundles.

Interpretation:

- Renaming this field is not cosmetic.
- It would change a persisted integrity/recovery contract that existing tooling already understands.

### 3. Zip packing also treats the directory name as a storage-format seam

Key anchor:

- `crates/fret-diag/src/pack_zip.rs:757`

Current state:

- schema2-oriented zip packing explicitly filters `chunks/bundle_json`.

Interpretation:

- The current name is already part of storage-shape behavior, not only JSON vocabulary.

## Contract rules

### Canonical write/read rules

- Writers should keep emitting `paths.bundle_artifact` as the canonical bundle artifact path.
- Writers should keep using top-level `bundle_json` only for raw-bundle chunk indexing metadata.
- Readers may continue accepting the existing manifest shape without translation.
- New Layer B payload or summary cleanup must not reinterpret Layer A `bundle_json` as a generic
  alias field.

### What should not happen next

- Do not rename top-level manifest `bundle_json` to `bundle_artifact`.
- Do not rename `chunks/bundle_json/...` as part of Layer B vocabulary cleanup.
- Do not silently collapse `paths.bundle_artifact` and manifest `bundle_json` into one field.

## Migration rule for any future change

If we later decide that the manifest should expose a canonical chunk-index node, the safe path is:

1. add a new field with explicit chunk-index semantics,
2. dual-read old `bundle_json` and the new field,
3. keep `paths.bundle_artifact` unchanged,
4. retire `bundle_json` only after explicit compatibility-window review.

The unsafe path would be repurposing `bundle_json` in place or renaming it without documenting the
recovery/integrity semantics.

## Non-goals

- This note does not change manifest schema today.
- This note does not rename files, directories, or JSON fields in code.
- This note does not alter Layer B additive alias policy.
- This note does not require DevTools / MCP internal naming cleanup.

## Practical consequence for the workstream

This closes the immediate Layer A contract question that blocked further naming discussion:

- Layer B can continue to use `bundle_artifact` as the canonical payload term.
- Layer A keeps `bundle_json` as a deliberate chunk-index/storage-format term until a future
  compatibility-window migration is explicitly approved.
