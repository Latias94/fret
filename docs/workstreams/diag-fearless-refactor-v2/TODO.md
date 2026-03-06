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
  - [x] first-pass campaign metadata is now present (`tier`, `owner`, `platforms`, `expected_duration_ms`, `tags`),
  - [ ] flake policy,
  - [ ] capability/feature tags.
- [x] Decide whether to introduce a first-class “campaign” orchestration layer.
  - [x] Land a minimal aggregation/index consumer first via `fretboard diag summarize`.
  - [x] Land a first `fretboard diag campaign` surface that composes existing `suite` + `summarize` flows.
  - [ ] Decide when campaign definitions should move from built-in Rust registry to external manifests.
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
- [x] Land a first GUI consumer over the shared aggregate artifacts:
  - [x] `apps/fret-devtools` now includes a read-only `Regression` details tab,
  - [x] the tab reads `regression.summary.json` and `regression.index.json` from the existing artifacts root,
  - [x] the tab exposes a manual refresh path without defining a parallel campaign model.
- [x] Add the first GUI drill-down over failing regression summaries:
  - [x] the `Regression` tab now lists `failing_summaries` from `regression.index.json`,
  - [x] selecting a row loads the corresponding `regression.summary.json`,
  - [x] selected summary path, first bundle dir, and bundle dir list can be copied for evidence follow-up.
  - [x] selected summary evidence can now be packed directly from the first failing bundle dir.
- [x] Add a thin GUI summarize trigger over the shared aggregate artifacts:
  - [x] the `Regression` tab now includes a `Summarize` action next to `Refresh`,
  - [x] the action runs the existing `diag summarize` flow against the current artifacts root,
  - [x] successful completion refreshes the aggregate artifacts instead of creating a GUI-only summary model.
- [x] Expose aggregate summary/index artifacts through the MCP consumer lane:
  - [x] `apps/fret-devtools-mcp` now exposes `regression.summary.json`,
  - [x] `apps/fret-devtools-mcp` now exposes `regression.index.json`,
  - [x] resources reuse the existing artifacts-root contract instead of defining a new store.
- [x] Add one end-to-end “dogfood” workflow that proves alignment:
  - [x] pick selector,
  - [x] patch or choose script,
  - [x] run,
  - [x] inspect artifacts,
  - [x] pack/share.
  - [x] Documented in `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`.
- [x] Refresh the DevTools GUI enough for daily dogfooding:
  - [x] top-level workspace shell and footer status strip now read as a product surface,
  - [x] `Script Studio` now reads as one workflow (`Workflow Controls` + `Outputs & Bundles` + focused panes),
  - [x] `Regression` now uses a summary-first master/detail layout with inspector sections,
  - [x] failing summary rows now expose lane/failure/item badges for faster scanning,
  - [x] `Regression Workspace` now uses a clearer summary strip (`Aggregate Status` + `Primary Actions` + `Dashboard Preview`).
  - [x] Documented in `docs/workstreams/diag-devtools-gui-refresh-v1.md`.

## Next focus after GUI refresh

- [x] Define the first campaign/suite execution slice over existing diag scripts:
  - [x] Drafted command-surface and output-layout proposal in `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_EXECUTION_ENTRY_V1.md`.
  - [x] Chose the CLI entry shape: `fretboard diag campaign`.
  - [x] Landed a minimal built-in campaign registry with `list` / `show` / `run`.
  - [x] Landed the minimum stable output layout for campaign runs:
    - `campaigns/<campaign_id>/<run_id>/campaign.manifest.json`
    - `campaigns/<campaign_id>/<run_id>/campaign.result.json`
    - `campaigns/<campaign_id>/<run_id>/suite-results/<suite>/...`
    - `campaigns/<campaign_id>/<run_id>/regression.index.json`
    - `campaigns/<campaign_id>/<run_id>/regression.summary.json`
  - [x] Kept DevTools and MCP on the same aggregate artifact handoff (`regression.index.json` + `regression.summary.json`).
- [ ] Expand the campaign surface beyond the first skeleton:
  - [x] move campaign definitions behind an explicit resolver seam (`registry/campaigns.rs`),
  - [x] promote that seam from built-in-only registry to manifest-backed resolver (`tools/diag-campaigns/*.json`),
  - [ ] decide whether to keep JSON-only or add TOML / generated registry inputs later,
  - [x] add first-pass campaign metadata (`owner`, `platforms`, `tier`, `expected_duration_ms`, `tags`),
  - [x] add direct script items in addition to suites,
  - [x] move canonical manifest authoring from top-level `suites`/`scripts` to ordered `items`,
  - [ ] decide when legacy top-level `suites`/`scripts` compatibility can be removed,
  - [ ] decide whether campaign runs should emit a persisted dashboard text or HTML projection.
- [ ] Make failed automation runs leave predictable evidence by default:
  - [ ] summary/index artifacts,
  - [ ] failing evidence bundles,
  - [ ] copy/share-friendly paths.
- [x] Add first campaign discovery filters to keep selection scalable (`--lane`, `--tier`, `--tag`, `--platform`).
- [ ] Add one thin maintainer note that explains the intended automation flow:
  - [ ] author or choose script,
  - [ ] run suite/campaign,
  - [ ] inspect aggregate summary,
  - [ ] pack/share evidence.


## M5 — Documentation consolidation

- [ ] Add a concise navigation note that tells contributors where to start for diag work.
- [ ] Cross-link existing v1/v1-architecture docs to this v2 umbrella where appropriate.
- [ ] Record migration intent for large existing diag docs rather than duplicating content forever.
- [x] Document the first aggregate dashboard/index fields for consumers:
  - [x] counters by lane/status/tool/reason,
  - [x] top reason codes,
  - [x] failing summaries ranking.
- [x] Land one thin consumer over the aggregate index:
  - [x] `fretboard diag dashboard` reads `regression.index.json`,
  - [x] default output gives a first-open human summary,
  - [x] `--json` preserves machine-readable access to the full index.
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
