# Diag Environment Predicate Contract v1

Status: Active design note
Last updated: 2026-04-20

Related:

- `WORKSTREAM.json`
- `BASELINE_AUDIT_2026-04-20.md`
- `M1_FIRST_SOURCE_AND_TIMING_DECISION_2026-04-20.md`
- `M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/adr/0189-ui-diagnostics-extensibility-and-capabilities-v1.md`
- `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/element_runtime_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- `crates/fret-runtime/src/font_catalog.rs`
- `crates/fret-diag/src/registry/campaigns.rs`

This workstream is the narrow diagnostics follow-on after
`diag-monitor-topology-environment-v1`.

It does not reopen the monitor-topology export lane.
It does not force a generic runtime `EnvironmentSnapshot` abstraction.
It does not treat `debug.environment` or renderer font diagnostics as campaign-preflight inputs by
default.

## Why this lane exists

The repo now has enough evidence to see a real taxonomy problem:

- `ElementEnvironmentSnapshotV1` is the per-window reactive UI environment surface.
- `RendererFontEnvironmentSnapshot` is a renderer/resource-loading provenance surface.
- `UiDiagnosticsEnvFingerprintV1` is the diagnostics-run environment fingerprint.
- campaign orchestration still only understands `requires_capabilities`.

If future diagnostics work adds host-environment predicates carelessly, the likely failure modes are:

1. scraping debug-only per-window snapshots as if they were stable preflight inputs,
2. overloading `requires_capabilities` with environment facts that are not capabilities,
3. or introducing a single erased snapshot abstraction that collapses different ownership and
   lifetime rules into one incorrect contract.

This lane exists to prevent that drift before the first real predicate implementation lands.

## Current taxonomy

### 1. Per-window reactive UI environment

`ElementEnvironmentSnapshotV1` comes from ADR 0232's committed per-window environment snapshot.

It exists so render-time environment reads participate in dependency tracking, invalidation, and
debug explainability. It is window-scoped, frame-committed, and best-effort by runner.

This is not a campaign preflight contract.

### 2. Renderer/resource-loading environment

`RendererFontEnvironmentSnapshot` records the runner-approved renderer font source inventory plus a
monotonic revision.

It exists so resource-loading and SVG-text diagnostics can explain which renderer font bytes were
actually accepted. It is not a general host-environment store, and it is not a generic substitute
for viewport/device queries.

### 3. Diagnostics-run environment fingerprint

`UiDiagnosticsEnvFingerprintV1` is the bundle-level run context owned by diagnostics export.

This is the first place where a true host/run-level environment source can live honestly for
campaign tooling. `monitor_topology` belongs here because it is a host inventory fingerprint,
while `scale_factors_seen` remains run evidence only.

### 4. Diagnostics orchestration preflight

Today `crates/fret-diag` only supports `requires_capabilities`.

That means the repo still has no typed host-environment predicate contract, even though some bundle
fields and debug snapshots now carry richer environment evidence.

## Must-be-true outcomes

1. The repo names these environment lanes explicitly and keeps their ownership separate.
2. Future diagnostics environment predicates bind only to explicit preflight-grade sources, not to
   arbitrary debug snapshot paths.
3. Promotion from a runner/UI snapshot into a diagnostics predicate source requires an explicit
   admission rule, not ad hoc reuse.
4. `requires_capabilities` remains capabilities-only; environment facts do not get smuggled into
   that vocabulary.
5. The repo does not add a one-size-fits-all runtime snapshot registry just because multiple
   snapshots exist.

## Owner split

### `crates/fret-ui`

Owns the per-window committed environment mechanism used by rendering, invalidation, and adaptive
authoring.

### `crates/fret-runtime` and runner crates

Own runner- or renderer-published data-only snapshots when a lower layer genuinely needs a shared
cross-system environment source.

### `ecosystem/fret-bootstrap`

Owns diagnostics export of bundle/debug surfaces derived from those runtime and UI mechanisms.

### `crates/fret-diag`

Should own any future orchestration predicate contract.

That contract belongs here because campaign manifests, preflight summaries, skip/fail reasons, and
tooling-side provenance already live here.

## Rejected direction

### Rejected: generic runtime `EnvironmentSnapshot` family

Do not unify `ElementEnvironmentSnapshotV1`, `RendererFontEnvironmentSnapshot`, and
`bundle.json.env.monitor_topology` behind one generic runtime abstraction.

Why this is wrong:

- the scopes differ (`per-window` vs `runner/renderer-owned` vs `bundle/run-level`),
- the update semantics differ (frame-committed vs monotonic revision vs export-time summary),
- the consumers differ (render invalidation vs resource provenance vs diagnostics orchestration),
- and the explainability requirements differ.

The shared trait they have is "they are all environment-ish data", which is not a sufficient
reason to merge contracts.

### Rejected: debug snapshot scraping as preflight

Do not treat `debug.environment` or other debug-only snapshot lanes as the implicit source for
future campaign predicates.

Those surfaces are useful evidence after or during a run, but they are not yet declared as stable
preflight inputs and may vary per window or per frame.

## Target direction

The correct next contract is not a generic snapshot base class.

The correct next contract is a source-scoped diagnostics predicate layer that can say:

- which environment source is being read,
- who owns that source,
- whether the source is available preflight, launch-time, or only post-run,
- and how missing/unsupported cases fail deterministically.

The first frozen precondition for that layer is now explicit:

- diagnostics should keep a separate environment-source catalog parallel to `capabilities.json`,
- the first persisted filesystem publication name should be `environment.sources.json`,
- the first protocol direction is `FilesystemEnvironmentSourcesV1`,
- the first diagnostics-owned provenance helper is `EnvironmentSourceCatalogProvenance`,
- and availability must stay explicit as:
  - `preflight_filesystem_sidecar`
  - `preflight_transport_session`
  - `launch_time`
  - `post_run_only`

Admission rule for a new predicate-capable source:

1. the source is data-only and versionable,
2. the source has one clear owner,
3. tooling can resolve it before or during launch without scraping debug internals,
4. the source has a stable summary shape suitable for bundles or transport handshakes,
5. and the source adds a real scheduling or skip/run decision that `requires_capabilities` cannot
   express honestly.

First concrete candidate:

- `host.monitor_topology` is the first qualified source candidate.
- `host.monitor_topology` is also the first admitted source id for the future
  `environment.sources.json` family.
- It is qualified because it answers a real scheduling problem that capabilities cannot express.
- It is not yet implementation-ready for campaign predicates because current campaign preflight
  happens before fresh tool-launched runs publish such a source.
- If it is only available through bundle export in a fresh run, its truthful availability class is
  `post_run_only`.

Explicit exclusions for this source family:

- `debug.environment` is not an admitted source id by default.
- `RendererFontEnvironmentSnapshot` is not an admitted source id by default.
- Promoting either one into the catalog requires a new diagnostics admission decision rather than
  silent reuse.

## Intended next slice

This lane should next decide the smallest additive orchestration surface for environment predicates
inside `crates/fret-diag`.

That decision should stay narrow:

- keep `requires_capabilities` unchanged,
- add environment-source catalog/provenance plumbing before any manifest field,
- keep `capability_source` and environment-source provenance separate,
- and start with source IDs plus deterministic provenance/availability before widening into a full
  expression language.

The exact manifest JSON shape is intentionally not frozen yet in this note.
That syntax should be chosen only after the source taxonomy and admission rules are accepted.
