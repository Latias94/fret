# Baseline Audit - 2026-04-20

Status: Active baseline

## Assumptions-first resume set

### 1) The current automation preflight contract is still `requires_capabilities` only

- Area: diagnostics orchestration
- Assumption: campaign manifests and tooling preflight can currently reject on capabilities, but not
  on typed host-environment predicates.
- Evidence:
  - `crates/fret-diag/src/registry/campaigns.rs`
  - `crates/fret-diag/src/diag_campaign.rs`
- Confidence: Confident
- Consequence if wrong: this lane would be solving an already-shipped orchestration contract.

### 2) `debug.environment` is a rendering/debug surface, not a preflight surface

- Area: per-window environment snapshot
- Assumption: `ElementEnvironmentSnapshotV1` reflects ADR 0232's committed per-window environment
  state for explainability and dependency tracking.
- Evidence:
  - `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/element_runtime_diagnostics.rs`
- Confidence: Confident
- Consequence if wrong: the repo could safely reuse that surface for campaign scheduling without a
  new contract.

### 3) `RendererFontEnvironmentSnapshot` is already a second environment snapshot lane, but with different semantics

- Area: renderer/resource-loading provenance
- Assumption: the renderer font environment exists as a runtime-visible snapshot, yet it is source
  provenance for text/font loading rather than a generic host-environment inventory.
- Evidence:
  - `crates/fret-runtime/src/font_catalog.rs`
  - `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
- Confidence: Confident
- Consequence if wrong: the repo may already have a generic environment base contract that this
  lane should reuse instead of designing around.

### 4) `monitor_topology` is the first real diagnostics-run environment fingerprint, not the final contract

- Area: bundle environment fingerprint
- Assumption: `bundle.json.env.monitor_topology` is now a correct host fingerprint, but the repo
  still lacks a general predicate contract that says how campaigns may consume such sources.
- Evidence:
  - `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- Confidence: Confident
- Consequence if wrong: this lane would be documenting a gap that the previous follow-on already
  closed.

## Findings

### 1) The repo already has three environment lanes with different purposes

Those lanes are:

1. per-window reactive UI environment (`ElementEnvironmentSnapshotV1`),
2. renderer/resource-loading provenance (`RendererFontEnvironmentSnapshot`),
3. diagnostics-run environment fingerprint (`UiDiagnosticsEnvFingerprintV1`).

They all describe environment-adjacent facts, but they do not share one owner, one lifetime, or
one consumer.

### 2) Do not generalize them into one erased runtime family yet

The evidence supports a split, not a merge:

- UI environment is frame-committed and render-driven,
- renderer font environment is monotonic provenance,
- diagnostics env fingerprint is export/run-level summary.

Trying to unify them right now would optimize for naming symmetry instead of contract correctness.

### 3) The current automation preflight contract is still `requires_capabilities` only

That is the real missing link.

The repo now has better environment evidence, but it still lacks a typed orchestration rule for
when environment facts are allowed to influence campaign selection or skip/fail decisions.

### 4) The next contract belongs in `crates/fret-diag`, not in a shared erased snapshot helper

Campaign manifests, provenance, summary outputs, and deterministic preflight failures already live
in diagnostics tooling.

That makes diagnostics orchestration the correct owner for the future predicate layer, while UI and
runtime keep owning the lower-level source publication seams.

## Baseline verdict

Open a narrow diagnostics contract lane for source taxonomy and future predicate admission rules.

Do not start with a code refactor that merges existing environment snapshot families.
