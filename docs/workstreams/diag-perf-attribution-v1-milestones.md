# Diag perf attribution v1 (milestones)

## M0: layout observation visibility

Deliverables:

- Bundle snapshots include:
  - `layout_observation_record_time_us`
  - `layout_observation_record_models_items`
  - `layout_observation_record_globals_items`
- `fretboard diag stats <bundle>` surfaces these fields.

Acceptance:

- Running `ui-gallery-window-resize-stress-steady` shows non-zero layout observation costs on a baseline that records observations.
- `fretboard diag stats <bundle.json> --sort time --top 30` prints `layout_obs_record.us(time)=... items(models/globals)=...` for affected frames.
- Running the same scenario with “skip observation recording during interactive resize” shows near-zero layout observation record cost.

## M1: stats diff + budget view

Deliverables:

- `fretboard diag stats --diff <bundle_a> <bundle_b>` outputs:
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
