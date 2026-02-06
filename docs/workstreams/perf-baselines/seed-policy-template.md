---
title: Perf Baseline Seed Policy Template
status: draft
date: 2026-02-06
scope: perf, baseline, thresholds, anti-noise
---

# Perf Baseline Seed Policy Template

This template defines how we seed `diag perf --perf-baseline-out` thresholds from run statistics.

## Why this exists

- Keep threshold derivation auditable (`threshold_seed_policy` in baseline JSON).
- Tighten noisy scripts without hardcoding one-off rules in code.
- Keep a reproducible policy footprint in docs and command history.

## Scope syntax (`--perf-baseline-seed`)

Use:

```bash
--perf-baseline-seed <scope@metric=max|p90|p95>
```

Supported `scope`:

- Script path (repo-relative or absolute):
  - `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- Built-in perf suite name:
  - `ui-gallery`
  - `ui-gallery-steady`
- `this-suite`:
  - expands to all scripts of the currently selected named suite in `diag perf`.
- `*`:
  - wildcard for all scripts.

Notes:

- The flag is repeatable; later rules override earlier ones for the same `(script, metric)`.
- Built-in defaults still apply unless overridden by CLI rules.

## Recommended baseline profile (v15)

Current default bias:

- default seed: `max`
- built-in override:
  - `ui-gallery-window-resize-stress-steady` uses `p95` for:
    - `top_total_time_us`
    - `top_layout_time_us`
    - `top_layout_engine_solve_time_us`

Example command (steady suite):

```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --json \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json \
  --perf-baseline-headroom-pct 20 \
  --perf-baseline-seed ui-gallery-steady@top_total_time_us=p90 \
  --perf-baseline-seed ui-gallery-steady@top_layout_time_us=p90 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

## Suggested milestone policy ladder

- Phase A (stabilize): `max` global + script-specific `p95` only for known high-noise scripts.
- Phase B (tighten): move selected scripts/metrics to `p90` when repeat=7 remains stable.
- Phase C (gate-hardening): keep `p90/p95` only where justified by noise evidence; avoid blanket wildcard rules.

## Change checklist

When updating seed policy:

1. Record the exact command in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`.
2. Capture validation run (`--perf-baseline`) and confirm `check.perf_thresholds.json` failures are `0`.
3. Update `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md` with the new policy decision.
4. Keep `docs/workstreams/perf-baselines/*.json` policy header (`threshold_seed_policy`) in sync with the run.
