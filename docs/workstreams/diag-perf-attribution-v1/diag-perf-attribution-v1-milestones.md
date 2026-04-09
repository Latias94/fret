# Diag perf attribution v1 (milestones)

## M0: layout observation visibility

Deliverables:

- Bundle snapshots include:
  - `layout_observation_record_time_us`
  - `layout_observation_record_models_items`
  - `layout_observation_record_globals_items`
- `fretboard-dev diag stats <bundle>` surfaces these fields.

Acceptance:

- Running `ui-gallery-window-resize-stress-steady` shows non-zero layout observation costs on a baseline that records observations.
- `fretboard-dev diag stats <bundle.json> --sort time --top 30` prints `layout_obs_record.us(time)=... items(models/globals)=...` for affected frames.
- Running the same scenario with “skip observation recording during interactive resize” shows near-zero layout observation record cost.

## M1: stats diff + budget view

Deliverables:

- `fretboard-dev diag stats --diff <bundle_a> <bundle_b>` outputs:
  - top deltas by impact,
  - percent deltas,
  - JSON + human view.
- `diag stats` JSON includes a budget breakdown view (`avg.*` + `budget_pct.*`).

Acceptance:

- Comparing two known runs highlights the expected subsystem delta (e.g. `layout.*` drops after a layout optimization).

## M2: opt-in trace artifacts

Deliverables:

- A canonical “trace capture” switch for `diag perf` runs.
- A local Chrome trace JSON attached to the run out-dir (and referenced by a manifest).

Acceptance:

- A single run can be triaged by:
  1) perf summary,
  2) stats budget view,
  3) opening the exported `trace.chrome.json` in Chrome tracing UI.

## M3: explainability hints + optional gate

Deliverables:

- `triage.json` includes:
  - bounded, rule-based hints (`hints[]`) with severity and evidence,
  - unit-cost estimates (`unit_costs`) derived from the worst frame.
- `diag perf` can optionally treat selected hints as a gate:
  - `--check-perf-hints`
  - evidence: `check.perf_hints.json`

Acceptance:

- A run with known hint triggers produces `triage.json` with stable hint codes.
- Running `fretboard-dev diag perf ... --check-perf-hints`:
  - exits non-zero when denied hints meet the configured minimum severity,
  - writes `check.perf_hints.json` with a non-empty `failures[]` array,
  - supports narrowing via `--check-perf-hints-deny <code,...>` and severity via
    `--check-perf-hints-min-severity <info|warn|error>`.
