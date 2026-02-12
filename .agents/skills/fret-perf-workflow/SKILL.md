---
name: fret-perf-workflow
description: "Profile and gate Fret performance using `fretboard diag perf`, perf baselines, and the `tools/perf/*` helpers (resize probes, baseline selection, log append). Use when investigating perf regressions/hot paths (resize/scroll/pointer-move), recording commit-addressable evidence, or separating CPU vs GPU costs."
---

# Fret performance workflow

This skill is the performance-focused companion to `fret-diag-workflow`.

Use this skill when you need **numbers**, **baselines**, and **gates** (not just repro bundles).

## When to use

- You suspect a perf regression (resize/scroll/pointer-move hitching) and need a repeatable measurement.
- You’re landing performance work and want to protect it with baselines/gates.
- You need to separate “CPU time” vs “renderer/GPU-ish time” with evidence artifacts.

## Inputs to collect (ask the user)

Ask the minimum that makes results comparable:

- Which probe/suite matches the hot path (resize/scroll/pointer-move)?
- What machine tag + build mode are you measuring (and is it stable enough for gates)?
- Are you comparing to an existing baseline (which file), or generating a new baseline?
- What is the acceptance criterion: “must pass gate” vs “improve metric X by Y%”?

Defaults if unclear:

- Run `ui-gallery-steady` + the P0 resize probes with `--attempts 3` and record the worst bundles.

## Smallest starting point (one command)

- `tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3`

## Quick start

### Run the P0 resize gates (recommended “global sanity”)

```bash
tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3
tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes --attempts 3
```

Notes:
- `tools/perf/diag_resize_probes_gate.sh` defaults to `--attempts 1` (fast local check). For tail stability and
  flake resistance, prefer `--attempts 3` and require a strict majority pass.

### Run the steady suite against the canonical baseline

```bash
cargo run -p fretboard -- diag perf ui-gallery-steady \
  --dir target/fret-diag-perf/ui-gallery-steady.<tag> \
  --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 15 --json \
  --perf-baseline <baseline.json> \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Notes:
- `--perf-baseline` requires an explicit file path (pick one under `docs/workstreams/perf-baselines/`).

### Append results to the perf log (commit-addressable)

```bash
python3 tools/perf/perf_log.py append \
  --stdout <captured_stdout.json> \
  --log docs/workstreams/ui-perf-zed-smoothness-v1-log.md \
  --suite <suite-name> \
  --command "<exact command used>" \
  --change "<short change summary>"
```

### Append a gate attempt to the perf log (resize probes)

`diag_resize_probes_gate.sh` writes an `attempt-N/stdout.json` file that includes the perf JSON payload. You can
append that directly:

```bash
python3 tools/perf/perf_log.py append \
  --stdout <gate-out-dir>/attempt-1/stdout.json \
  --log docs/workstreams/ui-perf-zed-smoothness-v1-log.md \
  --suite ui-resize-probes \
  --command "tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3 --out-dir <gate-out-dir>" \
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

## Definition of done (what to leave behind)

When you finish perf work, leave:

- Minimum deliverables (3-pack): Repro (probe/suite), Gate (baseline/threshold), Evidence (worst bundles + command). See `fret-skills-playbook`.
- The exact command(s) used (copy/pasteable) + an output dir with artifacts.
- A baseline or gate result that can be re-run at the same commit.
- The worst bundle path(s) for the relevant scripts (so attribution is deterministic).
- One perf log entry (commit-addressable evidence) when the change is meant to land.

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

## From gate failure to root cause (tail hitch loop)

When a gate fails, the goal is to go from “numbers” → “one concrete hitch class” quickly.

### Optional helper (compact gate triage)

If you have a gate out-dir (from `tools/perf/diag_resize_probes_gate.sh`), you can print a compact triage summary:

```bash
python3 .agents/skills/fret-perf-workflow/scripts/triage_gate.py target/perf-gates/ui-resize-probes.<tag>
```

This reports:

- which attempts passed/failed,
- which script/metric exceeded thresholds, and
- the worst bundle for each failing script (by `top_total_time_us`).

If you want to capture “best-effort internal attribution” from `app_snapshot` (when available), add:

```bash
python3 .agents/skills/fret-perf-workflow/scripts/triage_gate.py target/perf-gates/ui-resize-probes.<tag> --app-snapshot
```

And if you want worst bundles even for passing attempts (useful for logs):

```bash
python3 .agents/skills/fret-perf-workflow/scripts/triage_gate.py target/perf-gates/ui-resize-probes.<tag> --all --app-snapshot
```

If you prefer a bash + `jq` + `awk` version (macOS/Linux), see:

- `.agents/skills/fret-perf-workflow/scripts/triage_gate.sh`

Note:
- `diag perf --json` still prints the JSON payload even when perf thresholds fail (the process exits non-zero, but
  stdout contains the payload), so the helper can resolve worst bundles for failing attempts.

1. Identify the failing script/metric.
   - `<out-dir>/check.perf_thresholds.json`
   - For gate scripts: `<out-dir>/attempt-N/check.perf_thresholds.json`
2. Find the worst bundle for that script.
   - Gate scripts write `attempt-N/stdout.json`, but it may include log lines before the JSON payload.
   - Easiest (cross-platform): use the helper above to resolve the worst bundles per script.
3. Attribute the worst frame in that bundle.
   - `cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30`
4. Decide the lever (and keep it global-optimum safe).
   - If the top cost is `layout_engine_solve_time_us`: focus on measure/shaping reuse, layout root scope, and
     allocation spikes (HashMap growth / rehash).
   - If the top cost is `paint_time_us`: focus on text prepare churn, atlas upload/eviction, scene replay, and
     intermediate pool churn.
   - Validate the change globally: `ui-gallery-steady` + P0 resize probes must pass.

## Common pitfalls

- “One probe win” is not accepted if `ui-gallery-steady` regresses.
- If a change touches text/renderer caches, always re-run at least one resize probe and one steady suite check.
- Prefer a small number of well-maintained gates over lots of flaky scripts.
- If your working tree is “dirty” (unrelated refactors, conflict resolution, or experiments in progress), run perf
  gates from a **clean checkout** at the exact commit you want to measure so the results remain reversible and
  commit-addressable.
  - Recommended: use a temporary clean clone (or a detached HEAD checkout) rather than measuring with unresolved
    conflicts / unrelated diffs present.
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

## Evidence anchors (where to look)

- CLI entry + flags: `apps/fretboard/src/cli.rs`, `crates/fret-diag/src/lib.rs` (perf subcommands)
- Perf helpers: `tools/perf/`
- Baselines: `docs/workstreams/perf-baselines/`
- Perf logs (commit-addressable evidence): `docs/workstreams/*perf*log*.md`
- Tracy workflow: `docs/tracy.md`
- RenderDoc workflow: `docs/renderdoc-inspection.md`

## Related skills

- `fret-diag-workflow` (scripted repro bundles; complements perf numbers)
