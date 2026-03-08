# Diag Fearless Refactor v2 — Milestones

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

## M0 — Shared scope and boundary map

Outcome:

- The repo has one clear umbrella statement for what diagnostics includes and what it does not.

Deliverables:

- A v2 umbrella workstream note.
- A boundary map for runtime, tooling, transport, artifacts, and GUI (`CRATE_AND_MODULE_MAP.md`).
- A documented decision that DevTools GUI is included as a consumer lane.

Exit criteria:

- A contributor can answer “should this live in runtime, tooling, or GUI?” without guesswork.

## M1 — Core seam plan agreed

Outcome:

- We have a practical refactor sequence instead of a loose list of local improvements.

Deliverables:

- A hotspot inventory for runtime/tooling.
- A short list of next seam extractions with rationale and risk.
- A phased implementation roadmap (`IMPLEMENTATION_ROADMAP.md`).

Progress update:

- `diag_suite` has now landed twelve consecutive seam slices around post-run and result-summary orchestration:
  - core default check planning,
  - editor/markdown/text default check planning,
  - explicit-or-policy post-run trigger planning,
  - retained-vlist script override planning,
  - suite success/failure summary emit factoring,
  - per-script row payload shaping,
  - failure finalization around stop/emit/return-exit paths,
  - tooling-failure handling around script-result writes plus row/finalize wiring,
  - script-outcome handling for failed/unexpected/lint-failed branches,
  - per-script context assembly for stage/reason accounting plus evidence/lint preparation,
  - transport result decoding around `dump_label`, `run_script_over_transport`, and `tooling.suite.error` fallback,
  - script-execution block assembly for prewarm/prelude/load-script wiring.
- The next remaining high-ROI seam in this area is per-script launch and transport acquisition around
  `maybe_launch_demo` plus connected transport selection rather than re-expanding execution-block
  setup in `cmd_suite`.

Exit criteria:

- Follow-up implementation PRs can be scoped as small seam migrations, not broad rewrites.

## M2 — Artifact model consolidated

Outcome:

- Diagnostics outputs are described as one coherent artifact system.

Progress update:

- The first consolidated artifact and evidence contract now exists in
  `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`.
- The workstream now names source-of-truth artifacts, derived/index artifacts, optional evidence,
  and presentation-facing projections explicitly.
- The remaining gap in this milestone is adoption and enforcement across older notes and any future
  artifact additions.

Deliverables:

- A canonical artifact taxonomy.
- Compatibility and bounded-evidence policy.
- A defined “first-open” artifact set for everyday triage.
- Primary note: `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`.

Exit criteria:

- Humans and tools can perform common triage without assuming the raw largest artifact is always required.

## M3 — Regression orchestration model chosen

Progress update:

- The initial summary/evidence contract is no longer doc-only.
- `diag suite`, `diag repeat`, `diag perf`, and `diag matrix` now emit
  `regression.summary.json` as an additive artifact.
- `diag matrix` also leaves behind `matrix.summary.json` for compare-oriented consumers.
- The remaining gap in this milestone is campaign aggregation and tighter reason/evidence
  vocabulary standardization.

Outcome:

- Regression execution becomes a product surface instead of a loose collection of commands.

Deliverables:

- A documented lane model (`smoke`, `correctness`, `matrix`, `perf`, `nightly/full`).
- A metadata plan for suites/scripts.
- A summary/evidence contract for orchestrated runs.
- Initial design note: `REGRESSION_CAMPAIGN_V1.md`.
- Initial summary schema note: `REGRESSION_SUMMARY_SCHEMA_V1.md`.

Exit criteria:

- It is obvious how to scale from one script to one suite to one repo-level regression run.

## M4 — DevTools GUI aligned to the same contracts

Outcome:

- GUI participates in the same diagnostics architecture instead of drifting into a parallel model.

Deliverables:

- A documented GUI-in-scope boundary.
- At least one end-to-end dogfood workflow that crosses pick/run/artifacts.
- A clear defer list for GUI-only polish.

Exit criteria:

- GUI can be discussed as a presentation surface over shared diagnostics contracts.

## M5 — Documentation migration and maintainer workflow

Outcome:

- Diagnostics docs become easier to navigate and less likely to diverge.

Deliverables:

- Cross-links from older workstreams where needed.
- A maintainer checklist for landing diagnostics changes safely.

Exit criteria:

- A maintainer can add a diagnostics feature and know which docs and gates to touch.

## M6 — Enforcement and debt retirement

Outcome:

- Refactor gains are preserved instead of slowly regressing back into duplication.

Deliverables:

- A visible debt retirement list.
- Seam migrations paired with gates or tests.
- Exit criteria for removing stale compatibility/documentation paths.

Exit criteria:

- The diagnostics stack can continue evolving through additive seam-based changes rather than future monolithic rewrites.
