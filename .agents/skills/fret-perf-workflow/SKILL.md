---
name: fret-perf-workflow
description: "Profile and gate Fret performance using `fretboard diag perf`, perf baselines, and the `tools/perf/*` helpers (resize probes, baseline selection, log append). Use when investigating perf regressions/hot paths (resize/scroll/pointer-move), recording commit-addressable evidence, or separating CPU vs GPU costs."
---

# Fret performance workflow

This skill is the performance-focused companion to `fret-diag-workflow`.

Use this skill when you need **numbers**, **baselines**, and **gates** (not just repro bundles).

## Quick start

### Run the P0 resize gates (recommended “global sanity”)

```bash
tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3
tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes --attempts 3
```

### Run the steady suite against the canonical baseline

```bash
cargo run -p fretboard -- diag perf ui-gallery-steady \
  --dir target/fret-diag-perf/ui-gallery-steady.<tag> \
  --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 15 --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.<machine-tag>.v*.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

### Append results to the perf log (commit-addressable)

```bash
python3 tools/perf/perf_log.py append \
  --stdout <captured_stdout.json> \
  --log docs/workstreams/ui-perf-zed-smoothness-v1-log.md \
  --suite <suite-name> \
  --command "<exact command used>" \
  --change "<short change summary>"
```

## Workflow (gate-first, reversible)

1. Pick a probe that matches the hot path and can be gated.
   - Resize: `ui-resize-probes`, `ui-code-editor-resize-probes`
   - Scroll/VirtualList: `tools/perf/diag_vlist_boundary_gate.sh`
   - Pointer-move: use the `--max-pointer-move-*` thresholds in `diag perf`
2. Freeze the measurement protocol (avoid noise).
   - Always pass `--dir`.
   - Use `--reuse-launch` for steady-state comparisons.
   - For perf runs, disable heavy diagnostics that distort I/O:
     - `FRET_DIAG_SCRIPT_AUTO_DUMP=0`
     - `FRET_DIAG_SEMANTICS=0` (unless semantics is the subject)
3. Make one change, then validate **globally**.
   - Must pass: `ui-gallery-steady` baseline + P0 resize probes.
4. Record evidence for reversibility.
   - Add a perf log entry (commit hash + commands + deltas + worst bundles).
   - Update the TODO tracker checkbox if it is milestone work.

## Baselines

### Generate/select a baseline (anti-outlier)

Use `tools/perf/diag_perf_baseline_select.sh` to generate multiple candidates and pick the best.

Example:

```bash
tools/perf/diag_perf_baseline_select.sh \
  --baseline-out docs/workstreams/perf-baselines/ui-code-editor-resize-probes.<machine-tag>.vN.json \
  --suite ui-code-editor-resize-probes \
  --preset docs/workstreams/perf-baselines/policies/ui-code-editor-resize-probes.v1.json \
  --candidates 2 --validate-runs 3 --repeat 7 --warmup-frames 5 --headroom-pct 20 \
  --work-dir target/fret-diag-baseline-select-ui-code-editor-resize-probes-vN \
  --launch-bin target/release/fret-ui-gallery
```

## CPU vs GPU triage (when numbers look fine but it still hitches)

1. Run the probe with extra renderer instrumentation.
   - `FRET_DIAG_RENDERER_PERF=1` (expect overhead; do not use this for strict gates).
2. Capture a worst bundle and use:
   - `fretboard diag stats <bundle.json> --sort time --top 30`
3. If CPU is low but hitch persists, use a trace/capture workflow:
   - Tracy (`docs/tracy.md`)
   - RenderDoc (`docs/renderdoc-inspection.md`)

## Tips / pitfalls

- “One probe win” is not accepted if `ui-gallery-steady` regresses.
- If a change touches text/renderer caches, always re-run at least one resize probe and one steady suite check.
- Prefer a small number of well-maintained gates over lots of flaky scripts.
- If your working tree is “dirty” (unrelated refactors, conflict resolution, or experiments in progress), run perf
  gates from a **detached worktree** at the exact commit you want to measure so the results remain reversible and
  commit-addressable.
  - Example:
    ```bash
    git worktree add --detach ../fret-perf-lab <commit>
    cd ../fret-perf-lab
    cargo build -p fret-ui-gallery --release
    tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes --attempts 3
    ```
- When A/B testing env knobs that are read via `OnceLock`/static initialization, ensure the target process is
  restarted between A/B runs (a single `--reuse-launch` session cannot see changed env values).

## A/B workflow (env knobs)

When evaluating a new cache/knob, use an explicit A/B protocol and keep artifacts separate:

1. Prebuild the release binary once:
   - `cargo build -p fret-ui-gallery --release`
2. Run `A` (off/default) and `B` (on) with separate output dirs.
3. Extract and record:
   - gate pass/fail + attempts summary,
   - worst-frame bundle path(s),
   - 3–5 “load-bearing” metrics (e.g. `top_total_time_us`, `top_layout_engine_solve_time_us`,
     `paint_text_prepare_time_us` if text is involved),
   - the exact env knobs and values.

Example (word-wrap cache experiments):

```bash
FRET_TEXT_UNWRAPPED_LAYOUT_CACHE_ENTRIES=0 \
  tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes --attempts 3 \
    --out-dir target/fret-diag-ab-text-unwrapped-off

FRET_TEXT_UNWRAPPED_LAYOUT_CACHE_ENTRIES=2048 \
  tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes --attempts 3 \
    --out-dir target/fret-diag-ab-text-unwrapped-on
```
