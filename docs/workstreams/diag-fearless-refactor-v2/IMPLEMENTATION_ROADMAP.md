# Diag Fearless Refactor v2 — Implementation Roadmap

Status: Draft

Tracking docs:

- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-fearless-refactor-v2/CRATE_AND_MODULE_MAP.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`

## 0) Why this note exists

The v2 docs now answer three design questions:

- what the diagnostics platform should be,
- where changes belong by crate/layer,
- what a future regression campaign model should look like.

This note answers the next question:

- how do we land the work incrementally without reintroducing monoliths or stalling on a big-bang rewrite?

The roadmap is intentionally implementation-facing. It should be usable to slice follow-up PRs.

## 1) Principles for landing work

### 1.1 Prefer seams before renames

Do not start with broad renaming or package reshuffling.

Prefer:

- explicit module seams,
- small wrappers/adapters,
- moving one production path at a time,
- adding one gate per seam migration.

### 1.2 Keep presentation surfaces thin

The first implementation steps should improve shared core behavior for:

- CLI,
- GUI,
- MCP,
- CI,
- offline artifact consumers.

If a step only helps one surface, it should usually wait unless it removes major architectural risk.

### 1.3 Add machine-readable outputs early

Before building more UI around campaigns, land the summary and evidence model first.

### 1.4 Use documentation to drive module boundaries

If a PR changes ownership lines, it should update the relevant v2 docs in the same change.

## 2) Recommended implementation phases

## Phase A — Stabilize routing and ownership

Goal:

- make future implementation PRs easy to scope.

Main outputs:

- `CRATE_AND_MODULE_MAP.md`
- `IMPLEMENTATION_ROADMAP.md`

Code expectations:

- no major behavior changes required yet,
- ok to add small comments/tests/helpers only if they clarify seams.

Done when:

- contributors can choose the correct layer before editing code,
- upcoming PRs can point to a phase in this roadmap.

## Phase B — `crates/fret-diag` orchestration seam cleanup

Goal:

- reduce the cost of adding campaign-style orchestration later.

Primary code areas:

- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/diag_run.rs`
- `crates/fret-diag/src/diag_suite.rs`
- `crates/fret-diag/src/diag_repeat.rs`
- `crates/fret-diag/src/diag_matrix.rs`
- `crates/fret-diag/src/diag_perf.rs`
- `crates/fret-diag/src/registry/`
- `crates/fret-diag/src/commands/`

Suggested seam targets:

- run planning context,
- shared artifact output plumbing,
- check planning vs execution,
- suite selection/resolution,
- summary row emission hooks.

Recommended PR slices:

1. extract a reusable run-summary row writer/helper,
2. centralize per-run evidence path collection,
3. centralize retry/attempt accounting hooks,
4. make suite/campaign resolution an explicit seam instead of buried command logic.

Gate expectations:

- existing `fret-diag` tests stay green,
- add targeted tests for any new summary/evidence helpers.

## Phase C — Runtime artifact and evidence alignment

Goal:

- make runtime exports easy for campaigns to consume without bespoke scraping.

Primary code areas:

- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle*.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_result.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/extensions.rs`

Suggested seam targets:

- ensure stable evidence anchors are written predictably,
- keep sidecars and `script.result.json` easy to locate,
- make lane-specific evidence optional and bounded.

Recommended PR slices:

1. normalize runtime-written evidence locations used by tooling,
2. ensure a stable minimum evidence contract for failed runs,
3. add any missing bounded sidecars required by the future summary model.

Gate expectations:

- no regression in existing script runs,
- at least one end-to-end scripted run validates the expected evidence set.

## Phase D — Land `regression.summary.json` generation

Current status:

- Partially landed.
- `crates/fret-diag/src/regression_summary.rs` now defines the shared summary model.
- `diag suite`, `diag repeat`, `diag perf`, and `diag matrix` now emit
  `regression.summary.json` additively without replacing their existing outputs.
- `diag matrix` also writes `matrix.summary.json` as a stable compare-oriented sidecar.
- `diag summarize` now provides a first consumer-side aggregation/index surface over many
  `regression.summary.json` artifacts.
- the aggregate index already exposes dashboard-oriented helpers such as counters, top reason
  codes, and failing summary rankings.
- Remaining work is now mostly about contract hardening, richer campaign selection, and
  campaign-level orchestration beyond simple aggregation.

Goal:

- produce one machine-readable summary artifact from existing run primitives.

Primary code areas:

- `crates/fret-diag/src/diag_suite.rs`
- `crates/fret-diag/src/diag_repeat.rs`
- `crates/fret-diag/src/diag_matrix.rs`
- `crates/fret-diag/src/diag_perf.rs`
- likely a new summary-focused module under `crates/fret-diag/src/`

Recommended implementation direction:

- start additive,
- generate the summary as an extra artifact for one or two flows first,
- keep existing console output and JSON outputs intact during migration.

Recommended PR slices:

1. [done] add summary model types in `crates/fret-diag`,
2. [done] emit summary rows for `diag suite`,
3. [done] extend summary rows for `diag repeat` and flake classification,
4. [done] extend summary rows for `diag matrix` and `diag perf`,
5. [done] write `regression.summary.json` under a stable location,
6. [done] define a first campaign-level aggregation/index output over many summary artifacts,
7. [next] tighten stable reason-code and evidence-path conventions across all lanes,
8. [next] decide whether aggregation should stay as `diag summarize` or become part of a
   future `diag campaign` surface.

Gate expectations:

- unit tests for summary serialization,
- regression tests for one success case and one failure case,
- no breakage in existing JSON consumers.

## Phase E — Introduce campaign entry surface

Goal:

- expose the lane model as a first-class repo workflow.

Primary code areas:

- `crates/fret-diag/src/commands/`
- `crates/fret-diag/src/registry/`
- `apps/fretboard/src/diag.rs`

Suggested surface:

- a future `diag campaign` or equivalent command,
- lane selection,
- suite filtering,
- output directory and summary emission,
- explicit flake/retry policy knobs.

Recommended PR slices:

1. add campaign config model and resolver,
2. implement `smoke` as the first lane,
3. implement `correctness`,
4. add `matrix` and `perf` composition,
5. add `nightly/full` presets.

Important rule:

- campaign should initially compose existing run primitives, not replace them.

Gate expectations:

- one golden test or fixture proving lane expansion,
- one end-to-end doc example per first implemented lane.

## Phase F — DevTools GUI and MCP alignment

Goal:

- let presentation surfaces consume campaigns and summaries without defining their own semantics.

Current status:

- A first thin consumer now exists via `fretboard diag dashboard`, which reads
  `regression.index.json` for human-oriented inspection.
- The next step in this phase is no longer “whether a consumer is useful”, but how GUI/MCP
  should reuse the same index/summary contracts without forking semantics.

Primary code areas:

- `apps/fret-devtools-mcp/src/native.rs`
- DevTools GUI implementation surfaces referenced by `docs/workstreams/diag-devtools-gui-v1.md`
- offline viewer, if summary browsing becomes useful there

Recommended PR slices:

1. expose summary artifact paths/resources,
2. add campaign-aware run triggers in MCP or GUI,
3. add summary browsing panels,
4. add flake/evidence drill-down UX.

Important rule:

- GUI/MCP should read shared summary and artifact contracts rather than inventing their own run model.

## Phase G — Metadata and selection scaling

Goal:

- make campaign selection sustainable as suite count grows.

Primary code areas:

- `tools/diag-scripts/suites/`
- `crates/fret-diag/src/registry/suites.rs`
- future metadata parsers/helpers in `crates/fret-diag`

Recommended PR slices:

1. define suite/script metadata shape,
2. teach registry to read metadata,
3. support filtering by lane/tier/tag/platform,
4. document authoring rules for new suites.

Gate expectations:

- metadata parsing tests,
- one or two real suites migrated as proof.

## 3) Suggested PR order

Recommended near-term order:

1. Phase B small seam extraction
2. Phase D summary model and summary emission across core lanes
3. Phase E first `smoke` lane or campaign index surface
4. Phase C runtime evidence normalization where needed
5. Phase F presentation surface alignment
6. Phase G suite metadata scaling

Reasoning:

- summary generation already unlocks CLI, CI, GUI, and MCP together,
- campaign entrypoints are easier once a summary/evidence model exists,
- metadata scaling should wait until the first lane model is proven useful.

## 4) What not to do first

Avoid these as initial moves:

- splitting new crates before internal seams are stable,
- rewriting DevTools GUI first,
- introducing a new scripting language,
- over-designing distributed execution,
- deleting older docs before the replacement paths are proven.

## 5) Evidence and gate expectations by phase

Every phase should leave behind at least one of:

- a unit/integration test,
- a diag script or suite,
- a summary artifact example,
- a documented end-to-end command.

Suggested minimums:

- Phase B: tests around new orchestration helpers
- Phase C: one end-to-end evidence contract example
- Phase D: summary serialization + fixture tests
- Phase E: one implemented lane with a stable example command
- Phase F: one thin consumer example is landed; next is GUI/MCP adoption
- Phase G: metadata parsing + filtering tests

## 6) Definition of done for this roadmap

This roadmap is doing its job when:

- the next implementation PR can cite a phase and a small scope,
- v2 no longer reads like architecture-only prose,
- contributors can move from docs to code entry points without guesswork,
- diagnostics refactoring proceeds as a sequence of additive, gated steps.
