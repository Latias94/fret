# Closeout Audit - 2026-04-20

Status: Closed

Follow-on note (2026-04-26): the first approved second source now lives in
`docs/workstreams/diag-platform-capabilities-environment-v1/WORKSTREAM.json`. This closeout remains
the source of truth for the first `host.monitor_topology` admission contract and the rule that new
sources must land through separate narrow follow-ons.

## Result

This lane landed the first honest diagnostics environment-predicate contract without collapsing the
repo's existing environment families into one erased runtime abstraction.

The shipped outcome is:

- `crates/fret-diag` owns a separate `requires_environment` manifest field alongside
  `requires_capabilities`.
- the first shipped environment requirement stays source-scoped and source-specific:
  - `source_id: "host.monitor_topology"`
  - `predicate.kind: "host_monitor_topology"`
- the first shipped thresholds stay narrow:
  - `monitor_count_ge`
  - `distinct_scale_factor_count_ge`
- campaign admission resolves that source through three honest acquisition lanes:
  - existing filesystem publication,
  - preflight transport/session query,
  - launch-time probe for tool-launched filesystem runs.
- unsatisfied environment requirements write deterministic skip artifacts through
  `check.environment.json` rather than overloading capability evidence.

## Why this is the correct closeout point

The problem this lane opened to solve was also narrow:

1. classify the current environment snapshot taxonomy,
2. decide where a future environment-predicate contract belongs,
3. choose the first admitted source and honest acquisition timing,
4. and land the smallest real end-to-end manifest/admission slice.

That problem is now closed.

What remains is a different question:

- whether the repo has a second admitted environment source,
- whether that second source really needs shared combination semantics,
- and whether a future follow-on should widen authoring surfaces without widening grammar.

Those are not unfinished implementation leftovers here. They require fresh evidence.

## Audited non-candidates for a second admitted source today

### 1) ADR 0246 preference extensions are still per-window committed environment, not preflight source catalog

`text_scale_factor`, `prefers_reduced_transparency`, and `accent_color` live in ADR 0232/0246's
committed per-window environment snapshot and diagnostics export under `debug.environment`.

That is the correct mechanism for render-time dependency tracking and explainability, but it is not
yet a truthful diagnostics orchestration source:

- ownership is still window-scoped rather than run-scoped,
- values are best-effort and runner-specific,
- and tooling would have to promote debug/export semantics into preflight semantics without a new
  admission decision.

Conclusion:

- ADR 0246 does not currently justify a second admitted source id.

### 2) `RendererFontEnvironmentSnapshot` is renderer provenance, not host-environment scheduling input

The renderer font environment exists so resource-loading and SVG/text diagnostics can explain which
font bytes the renderer actually accepted.

That surface still fails the admission rule for a second predicate-capable source:

- it is provenance for renderer/resource state,
- not a stable host/run inventory for campaign scheduling,
- and current evidence does not show a real skip/run decision that capabilities or post-run checks
  cannot already express more honestly.

Conclusion:

- renderer font provenance should remain outside `requires_environment`.

### 3) `scale_factors_seen` remains run evidence only

The previous monitor-topology follow-on already froze the rule that `scale_factors_seen` is
run-observed window evidence, not host monitor inventory.

That still holds after this lane:

- it is a result of what happened during the run,
- not a preflight-grade source catalog fact,
- and promoting it would reintroduce the same taxonomy collapse this lane was opened to prevent.

Conclusion:

- `scale_factors_seen` is not a second admitted source candidate.

## Evidence

- Workstream state and design:
  - `docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json`
  - `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`
  - `docs/workstreams/diag-environment-predicate-contract-v1/TODO.md`
  - `docs/workstreams/diag-environment-predicate-contract-v1/MILESTONES.md`
  - `docs/workstreams/diag-environment-predicate-contract-v1/EVIDENCE_AND_GATES.md`
- Landed first-source slice:
  - `docs/workstreams/diag-environment-predicate-contract-v1/M5_REQUIRES_ENVIRONMENT_HOST_MONITOR_TOPOLOGY_ADMISSION_2026-04-20.md`
  - `crates/fret-diag/src/registry/campaigns.rs`
  - `crates/fret-diag/src/diag_campaign.rs`
  - `tools/diag-campaigns/README.md`
- Source taxonomy / exclusions:
  - `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
  - `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
  - `docs/adr/0246-environment-queries-preference-extensions-v1.md`
  - `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/element_runtime_diagnostics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
  - `crates/fret-runtime/src/font_catalog.rs`
- Validation used for closeout:
  - `cargo nextest run -p fret-diag --lib environment_admission --no-fail-fast`
  - `cargo nextest run -p fret-diag --lib manifest_campaign_parses_host_monitor_topology_environment_requirement --no-fail-fast`
  - `cargo nextest run -p fret-diag --lib capability_preflight_writes_check_summary_and_result_artifacts --no-fail-fast`
  - `cargo nextest run -p fret-diag-protocol --lib environment_sources_get --no-fail-fast`
  - `cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics-ws" --lib environment_sources_get --no-fail-fast`
  - `cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics" --lib refresh_environment_source_files_publishes_launch_time_monitor_topology_sidecars --no-fail-fast`
  - `python tools/gate_imui_workstream_source.py`
  - `python3 tools/check_workstream_catalog.py`
  - `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
  - `python3 -m json.tool docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json > /dev/null`
  - `git diff --check`

## Next-action rule

Keep this lane closed.

If future pressure appears, reopen the topic only through a different narrow follow-on that can
prove all of the following:

1. a second source has one clear owner and a versionable data-only shape,
2. tooling can acquire it honestly before or during launch without scraping debug internals,
3. the source adds a real scheduling or deterministic skip/run decision that capabilities do not
   already express,
4. and that second source creates real pressure for shared combination semantics rather than just
   naming symmetry.

Until that evidence exists, do not widen `requires_environment` into a generic boolean expression
language and do not merge existing environment families into one runtime abstraction.
