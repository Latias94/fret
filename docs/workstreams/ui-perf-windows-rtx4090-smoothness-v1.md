# UI perf: Windows RTX 4090 smoothness v1

Status: Active (local perf worktree)

## Goal

Make Windows (`windows-rtx4090`) UI smoothness a sustainable **performance contract**:

- Gates pass consistently (low tail latency, fewer spikes).
- Worst bundle is explainable (clear attribution, fast diff workflow).
- Optimizations are reversible (small, well-scoped commits + evidence).

This workstream focuses on **CPU-side frame smoothness** (layout/paint/dispatch) first, while keeping
GPU tooling (PIX/Nsight/RenderDoc) available for “GPU is the bottleneck” cases.

## Baselines (source of truth)

- `docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json`
- `docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v1.json`
- `docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v1.json`

Seed policy (how thresholds were derived):

- `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json`

## P0 runbook (fast gate check)

Prebuild (once):

- `cargo build -p fretboard -p fret-ui-gallery --release`

Recommended env (avoid extra I/O + keep cached rendering on):

- `FRET_DIAG_SCRIPT_AUTO_DUMP=0`
- `FRET_DIAG_SEMANTICS=0`
- `FRET_UI_GALLERY_VIEW_CACHE=1`
- `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`
- `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`

P0 commands:

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 3 --warmup-frames 5 --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env ... --launch -- target/release/fret-ui-gallery.exe`
- `target/release/fretboard.exe diag perf ui-resize-probes --repeat 3 --warmup-frames 5 --perf-baseline docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v1.json --env ... --launch -- target/release/fret-ui-gallery.exe`
- `target/release/fretboard.exe diag perf ui-code-editor-resize-probes --repeat 3 --warmup-frames 5 --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v1.json --env ... --launch -- target/release/fret-ui-gallery.exe`

## Failure triage (when a gate fails)

1) Look at the generated perf check:

- `<out_dir>/check.perf_thresholds.json`

2) Open the worst evidence bundle:

- `<out_dir>/worst_overall.bundle.json` (or the `worst_overall.bundle` path printed by `diag perf`)

3) Summarize and attribute:

- `target/release/fretboard.exe diag stats <bundle.json> --sort time --top 30 --json`
- Compare “good vs bad” bundles:
  - `target/release/fretboard.exe diag stats --diff <ok_bundle.json> <bad_bundle.json> --sort time --json`

4) If the summary is not enough, switch to opt-in deeper evidence:

- Node-level layout profiling:
  - `--env FRET_LAYOUT_NODE_PROFILE=1`
  - `--env FRET_LAYOUT_NODE_PROFILE_TOP=15`
  - `--env FRET_LAYOUT_NODE_PROFILE_MIN_US=400`
- Trace artifacts (for a single run, not for gate runs):
  - `target/release/fretboard.exe diag perf ... --trace`
  - `target/release/fretboard.exe diag trace <bundle.json>`

## What “typical perf” means here (not tail)

Tail (spikes) is “max / worst frame”. Typical perf should use **percentiles** (p50/p95) to answer
“is it generally faster/slower”.

Preferred workflow:

- Use `fretboard diag perf ... --json` and review `p50`/`p95` for the top metrics.
- Use `diag stats` for within-bundle averages and budgets (`avg.*`, `budget_pct.*`).

If a change improves p50/p95 but worsens max occasionally, treat it as “needs jitter work” (allocator,
capacity management, background work scheduling).

## Recent finding (2026-02-14): VirtualListMetrics clone caused avoidable churn

Symptom pattern:

- Same logical work (solves/nodes similar), but some runs had slow-path spikes.
- Layout node profiling (`FRET_LAYOUT_NODE_PROFILE=1`) showed VirtualList as a recurring hotspot.

Change:

- Avoid `VirtualListMetrics` cloning in VirtualList layout/measure paths (move-out + write-back).

Evidence:

- `tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json` became consistently under the
  `ui-gallery-steady.windows-rtx4090.v1` thresholds in repeated local runs.

## Next steps

### 1) Reduce remaining tail spikes (Windows-specific)

Hypotheses to validate:

- allocator jitter (large transient allocations outside the frame arena)
- hash/vec capacity growth on “rare” paths
- background thread wakeups competing with the UI thread during resize

Candidate actions (small → large):

- tighten capacity reuse for known hot scratch structures (avoid occasional rehash/grow)
- make “layout request → build roots → solve → apply” phase boundaries visible by default in traces
- add a small set of churn counters (“bytes allocated”, “vec grow events”) for the worst offenders

### 2) Strengthen profiling + stats surfaces (fearless refactor)

This workstream depends on (and should not duplicate) the broader diagnostics effort:

- `docs/workstreams/diag-perf-attribution-v1.md`
- `docs/workstreams/diag-perf-attribution-v1-field-inventory.md`

The delta we want here is “Windows smoothness” oriented:

- faster “good vs bad” comparison loops (1–2 commands)
- clearer typical-perf reporting (p50/p95 as first-class in review)
- stronger linkage from a failing threshold → responsible phase → top hotspots

## References / important code

- Layout pass + phase timers: `crates/fret-ui/src/tree/layout.rs`
- Layout engine (Taffy): `crates/fret-ui/src/layout/engine.rs`
- Stats summary / JSON keys: `crates/fret-diag/src/stats.rs`
- Diagnostics script runner / checks: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

