# Diag Environment Predicate Contract v1 - Milestones

Status: Active

## M0: Baseline taxonomy freeze

Exit criteria:

- The repo explicitly records the current environment-lane split:
  - per-window reactive UI environment,
  - renderer/resource-loading provenance,
  - diagnostics-run environment fingerprint.
- The lane locks the rule that `requires_capabilities` remains capabilities-only.
- The lane locks the no-erased-runtime-family verdict for these surfaces.

Primary evidence:

- `docs/workstreams/diag-environment-predicate-contract-v1/BASELINE_AUDIT_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`

Status:

- Completed on 2026-04-20.

## M1: Diagnostics predicate owner split

Exit criteria:

- The repo names `crates/fret-diag` as the owner for any future orchestration predicate contract.
- The repo records the admission rule for when a lower-level environment source may participate in
  preflight.
- Living diagnostics docs point at this lane instead of implying that debug snapshots are
  preflight-ready.

Primary evidence:

- `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`
- `docs/ui-diagnostics-and-scripted-tests.md`

Status:

- Completed on 2026-04-20.

## M2: First additive implementation slice

Exit criteria:

- One concrete source qualifies for predicate-capable use under the admission rules.
- The repo chooses the smallest additive manifest/summary contract for that source.
- Diagnostics preflight can emit deterministic provenance for the new decision.

Primary future evidence:

- `crates/fret-diag/src/registry/campaigns.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- a future dated status note in this folder

Status:

- Not started.
