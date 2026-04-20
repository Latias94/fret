# M2 Environment Source Provenance And Availability Contract - 2026-04-20

Status: active decision note

Related:

- `WORKSTREAM.json`
- `DESIGN.md`
- `BASELINE_AUDIT_2026-04-20.md`
- `M1_FIRST_SOURCE_AND_TIMING_DECISION_2026-04-20.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- `crates/fret-diag/src/transport/fs.rs`
- `crates/fret-diag-protocol/src/lib.rs`

## Purpose

This note freezes the minimal diagnostics-owned contract that must exist before any real
environment predicate syntax or execution lands.

The goal is to prevent the next implementation slice from making one of two incorrect moves:

1. reusing `capabilities.json` / `CapabilitySource` for environment facts,
2. or inventing a generic runtime `EnvironmentSnapshot` registry just because diagnostics now has
   more than one environment-shaped surface.

## Findings

### 1) Capabilities already separate publication from provenance

Diagnostics capability preflight already has two distinct concepts:

- a publication artifact (`capabilities.json`),
- and an internal normalized provenance helper (`CapabilitySource`).

That split is correct for capabilities and is the right template for future environment sources.

### 2) Environment predicates need the same split, but not capability semantics

Future environment predicates still need:

- one source-catalog publication seam,
- and one diagnostics-owned provenance shape.

But they do not need capability vocabulary.

Environment facts are not capabilities, and a future environment-source catalog must not inherit
capability naming or output semantics.

### 3) Availability/timing is part of the source contract, not an implementation detail

The first candidate source, `host.monitor_topology`, proved that source identity alone is not
enough.

Diagnostics also needs to know when the source is truthfully available:

- before preflight from a persisted sidecar,
- before preflight from a transport/session handshake,
- only after the child launches,
- or only as post-run evidence.

Without this classification, tooling will keep drifting toward dishonest preflight behavior.

## Decision

From this point forward:

1. Diagnostics should grow a separate environment-source catalog parallel to `capabilities.json`.
2. The first persisted filesystem publication name should be `environment.sources.json`.
3. The first additive protocol direction should be a catalog shape parallel to
   `FilesystemCapabilitiesV1`, named `FilesystemEnvironmentSourcesV1`.
4. The first catalog entry direction should be a small `FilesystemEnvironmentSourceV1` shape that
   carries:
   - `source_id`
   - `availability`
5. Diagnostics should keep provenance separate from the catalog payload itself through a dedicated
   internal helper, `EnvironmentSourceCatalogProvenance`, parallel to but distinct from
   `CapabilitySource`.
6. The first frozen availability classes are:
   - `preflight_filesystem_sidecar`
   - `preflight_transport_session`
   - `launch_time`
   - `post_run_only`
7. `host.monitor_topology` is the first admitted source id for this catalog family.
8. `debug.environment` and `RendererFontEnvironmentSnapshot` are not admitted source ids in this
   family by default.
9. `launch_time` and `post_run_only` sources must not be treated as truthful campaign-preflight
   predicates.
10. `post_run_only` sources are evidence-only: they may explain outcomes after the run, but they
    cannot decide whether the run should start.

## Why this is the correct shape

### Separate catalog instead of `capabilities.json`

`capabilities.json` answers "what tooling control-plane/runtime features are available?"

Environment-source publication answers a different question:

- "which diagnostics-admitted environment facts exist, and when may orchestration consume them?"

Those are different contracts and should remain separate on disk and in tooling.

### Catalog-level provenance instead of a generic runtime registry

This contract is diagnostics orchestration metadata.

It does not promise a shared runtime base class for all environment-shaped data.
It only says diagnostics may need one catalog that lists predicate-capable sources and their
availability class.

That is a tooling contract, not a runtime unification.

### Availability classes instead of a boolean "preflight-ready"

A simple boolean would erase the meaningful distinction between:

- filesystem-persisted preflight,
- transport/session preflight,
- launch-time resolution,
- and post-run evidence.

Those paths have different failure modes, reproducibility rules, and future execution hooks.

## Immediate consequence

The next implementation-worthy slice is now clearer:

1. add `environment.sources.json` reading/publication plumbing,
2. add `EnvironmentSourceCatalogProvenance` in `crates/fret-diag`,
3. keep any new summary/provenance output separate from `capability_source`,
4. and do all of that before introducing manifest syntax.

If `host.monitor_topology` still only reaches diagnostics through bundle export for fresh
tool-launched runs, its truthful availability remains `post_run_only` until a better publication
path exists.

## Explicit non-decisions

This note does not freeze:

- the final manifest JSON key,
- an expression language,
- source-specific payload schemas beyond the catalog direction above,
- or whether the first real consumer should evaluate at campaign preflight or at launch-time.

Those decisions should happen only after the separate environment-source catalog and provenance seam
exist.
