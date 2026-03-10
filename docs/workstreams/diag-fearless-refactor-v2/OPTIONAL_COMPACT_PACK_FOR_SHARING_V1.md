# Optional Compact Pack for Sharing V1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/ORCHESTRATED_OUTPUT_EVIDENCE_PATH_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`

## Purpose

This note defines what the optional compact sharing pack means in the diagnostics stack.

The goal is not to introduce a new source-of-truth artifact. The goal is to lock the bounded
handoff package shape that can be attached to CI, passed to another maintainer, or consumed by
automation when the full raw bundle tree is unnecessary.

## Decision

Treat compact sharing packs as **optional handoff packages over canonical artifacts**, not as the
canonical results themselves.

This means:

1. `regression.summary.json` and related result/index artifacts remain the source of truth.
2. compact sharing packs are packaging outputs for bounded handoff.
3. a compact pack may omit raw bundle artifacts entirely.
4. every compact pack must still preserve a path back to canonical result/status artifacts.

## Canonical compact pack surfaces

The current compact sharing surfaces are:

- per-item share zips referenced as `share_artifact`,
- aggregate `share/share.manifest.json`,
- aggregate `share/combined-failures.zip`,
- repro-oriented `repro.ai.zip` / `repro.zip`,
- summary-level `packed_report` when a summary row points to a bounded packed output.

Interpretation:

- these all belong to the same optional compact-pack family,
- but they serve different scopes:
  - item scope,
  - aggregate campaign/batch scope,
  - repro/export scope.

## Aggregate compact pack contract

For campaign or batch handoff, the aggregate compact pack contract is:

### Canonical files

- `share/share.manifest.json`
- `share/combined-failures.zip` when generated

### Canonical guarantees

- `share/share.manifest.json` must reference the root result context rather than replacing it,
- `share/combined-failures.zip` must package bounded evidence for failing items only,
- both remain optional additive outputs.

Key anchors:

- `crates/fret-diag/src/diag_campaign.rs:2518`
- `crates/fret-diag/src/diag_campaign.rs:2188`

## `share.manifest.json` contract

`share/share.manifest.json` is the canonical machine-readable description of a compact sharing set.

Required top-level sections:

- `schema_version`
- `kind`
- `source`
- `selection`
- `counters`
- `share`
- `items`

Semantics:

- `source` ties the handoff back to the run root and summary artifact,
- `selection` explains whether passed items were included,
- `counters` explains what was successfully packed or skipped,
- `share` records aggregate pack outputs such as `combined_zip`,
- `items` carries one bounded row per selected failure or selected run item.

Key anchor:

- `crates/fret-diag/src/diag_campaign.rs:2397`

## `share.manifest.json` item vocabulary

For each selected item, the canonical compact-pack item fields are:

- `bundle_dir`
- `triage_artifact`
- `screenshots_manifest`
- `share_artifact`
- `triage_error`
- `error`

Compatibility rule:

- the share manifest may still dual-write legacy `triage_json` and `share_zip`,
- new docs and new consumers should prefer `triage_artifact` and `share_artifact`.

Key anchor:

- `crates/fret-diag/src/diag_campaign.rs:2766`

## `combined-failures.zip` layout contract

`share/combined-failures.zip` is a convenience package for failing-item handoff.

### Root entries

When present, the zip should contain:

- `_root/share.manifest.json`
- `_root/regression.summary.json`
- `_root/regression.index.json` when present

### Item entries

Each failing item may contribute:

- `items/<nn>-<safe-item-id>.ai.zip`
- `items/<nn>-<safe-item-id>.triage.json`
- `items/<nn>-<safe-item-id>.screenshots.manifest.json`

Interpretation:

- the root area preserves the canonical first-open context,
- the `items/` area preserves bounded per-failure evidence only,
- the pack is intentionally flat and portable rather than mirroring the whole run directory tree.

Key anchors:

- `crates/fret-diag/src/diag_campaign.rs:2345`
- `crates/fret-diag/src/diag_campaign.rs:2370`

## Repro compact pack contract

Repro-oriented compact pack outputs are part of the same family but keep their own bounded shape.

### `repro.ai.zip`

Rules:

- must include `_root/repro.summary.json`,
- must include selected source scripts under `_root/scripts/`,
- may include per-item `ai.packet` contents,
- must not include `bundle.json` or `bundle.schema2.json`.

Key anchor:

- `crates/fret-diag/src/pack_zip.rs:1082`

### `repro.zip`

Interpretation:

- `repro.zip` is still a compact sharing surface when it packages a bounded repro handoff,
- but unlike `repro.ai.zip` it may include richer bundle-sidecar content depending on the workflow.

Non-goal:

- this note does not force campaign/batch packs and repro packs into one identical archive layout.

## Relationship to `packed_report`

`packed_report` remains the canonical summary-level pointer to a bounded packed output when one is
produced for an item or workflow.

Contract rule:

- `packed_report` is the cross-surface summary vocabulary,
- concrete on-disk filenames such as `combined-failures.zip` or `repro.ai.zip` are pack-family
  implementations behind that vocabulary.

This allows the summary contract to stay stable even if future pack shapes diversify.

## Consumer rules

### CLI / CI / GUI / MCP

Consumers should treat compact packs as:

- useful first-handoff attachments,
- bounded convenience packages,
- optional follow-up artifacts after result/summary inspection.

Consumers should **not** treat compact packs as:

- the only machine-readable status artifact,
- a replacement for `regression.summary.json`,
- a requirement for every successful run.

## Non-goals

- This note does not require every workflow to emit a compact pack.
- This note does not require raw bundles to be packed into compact outputs.
- This note does not remove legacy share-manifest aliases yet.
- This note does not define one universal archive layout for every diagnostics command.

## Practical consequence for the workstream

This closes the open `optional compact pack for sharing` TODO as a contract decision:

- compact packs are now explicitly optional and bounded,
- aggregate share-manifest and combined-failure zip surfaces are the canonical campaign/batch
  compact-pack outputs,
- repro compact packs are recognized as the same family with their own bounded rules,
- source-of-truth status remains in result/summary artifacts rather than in the pack itself.
