# Diag Fearless Refactor v2 — TODO

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

## M0 — Scope, vocabulary, and boundaries

- [ ] Confirm the umbrella positioning:
  - [ ] diagnostics is a general automation/debugging/evidence platform,
  - [ ] DevTools GUI is included as a consumer lane, not the architecture center.
- [x] Write a short “where does this change belong?” mapping for:
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/CRATE_AND_MODULE_MAP.md`
  - [ ] protocol/contracts,
  - [ ] runtime service,
  - [ ] transport,
  - [ ] tooling engine,
  - [ ] presentation surfaces.
- [ ] Identify overlapping or stale diag workstream docs and classify them:
  - [ ] still active,
  - [ ] superseded but still useful,
  - [ ] migrate into this v2 folder,
  - [ ] retire later with redirects.

## M1 — Runtime and tooling seam cleanup

- [ ] Audit the main runtime/export modules and list remaining monolith hotspots.
- [ ] Audit `crates/fret-diag` orchestration entry points and list duplication hotspots.
- [x] Write a phased implementation roadmap that maps design docs to code landing order:
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`
- [ ] Choose the next 2–3 high-ROI seam extractions for landable follow-up PRs:
  - [ ] run planning/context,
  - [ ] artifact resolution/materialization,
  - [ ] check planning/execution,
  - [ ] suite/campaign resolution,
  - [ ] transport dispatch.
- [ ] Define “no new blob growth” guardrails for follow-up work.

## M2 — Artifact and evidence consolidation

- [ ] Document one canonical artifact model:
  - [ ] bundle artifact,
  - [ ] sidecars,
  - [ ] `script.result.json`,
  - [ ] `triage.json`,
  - [ ] compact pack/AI packet style artifacts.
- [ ] Define which artifacts are:
  - [ ] source of truth,
  - [ ] derived/cache-like,
  - [ ] optional evidence,
  - [ ] GUI-friendly projections.
- [ ] Define a compatibility policy for artifact field additions/removals.
- [ ] Define one bounded “first-open” artifact set for common triage.

## M3 — Regression orchestration model

- [x] Write a first-pass campaign model for repo-level regression lanes:
  - [x] `smoke`
  - [x] `correctness`
  - [x] `matrix`
  - [x] `perf`
  - [x] `nightly/full`
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- [ ] Write a single vocabulary for regression lanes:
  - [ ] smoke,
  - [ ] correctness,
  - [ ] matrix,
  - [ ] perf,
  - [ ] nightly/full.
- [ ] Define suite metadata needed for scalable execution:
  - [ ] tier,
  - [ ] owner,
  - [ ] platform,
  - [ ] expected duration,
  - [ ] flake policy,
  - [ ] capability/feature tags.
- [ ] Decide whether to introduce a first-class “campaign” or “regression” orchestration layer.
- [ ] Define expected outputs for orchestrated runs:
  - [x] one machine-readable summary,
    - evidence: `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
    - implementation: `diag suite`, `diag repeat`, `diag perf`, and `diag matrix`
      now emit `regression.summary.json`
  - [ ] stable reason codes,
  - [ ] evidence bundle/artifact paths,
    - in progress: current summary emitters already attach bounded artifact/evidence paths,
      but the path vocabulary still needs one explicit repo-level contract
  - [ ] optional compact pack for sharing.

## M4 — DevTools GUI alignment

- [ ] Define which GUI features belong in this workstream now:
  - [ ] artifact browser,
  - [ ] gate runner UX,
  - [ ] live inspect summaries,
  - [ ] script library/editor wiring,
  - [ ] resource subscriptions.
- [ ] Explicitly defer GUI-only polish that should not block core refactors.
- [ ] Ensure GUI uses the same contracts and artifact terminology as CLI/tooling.
- [ ] Add one end-to-end “dogfood” workflow that proves alignment:
  - [ ] pick selector,
  - [ ] patch or choose script,
  - [ ] run,
  - [ ] inspect artifacts,
  - [ ] pack/share.

## M5 — Documentation consolidation

- [ ] Add a concise navigation note that tells contributors where to start for diag work.
- [ ] Cross-link existing v1/v1-architecture docs to this v2 umbrella where appropriate.
- [ ] Record migration intent for large existing diag docs rather than duplicating content forever.
- [ ] Add a short maintainer checklist for new diagnostics features:
  - [ ] which layer changes,
  - [ ] what gate must be added,
  - [ ] what evidence should be left behind,
  - [ ] what docs must be updated.

## M6 — Debt removal and enforcement

- [ ] Identify duplicated logic that should be removed only after seam adoption is proven.
- [ ] Add at least one regression gate or lint/test expectation for each major seam migration.
- [ ] Define “done” criteria for retiring older diag notes or compatibility shims.
- [ ] Keep a visible debt list so future refactors stay incremental instead of reverting to ad-hoc growth.
