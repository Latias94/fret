---
title: Perf Baseline Seed Policy Template
status: draft
date: 2026-02-06
scope: perf, baseline, thresholds, anti-noise
---

# Perf Baseline Seed Policy Template

This template defines how `diag perf --perf-baseline-out` derives thresholds from run statistics.

## Why this exists

- Keep threshold derivation auditable (`threshold_seed_policy` in baseline JSON).
- Tighten noisy scripts without hardcoded one-off policy branches.
- Keep a versioned, reviewable policy footprint in docs and command history.

## Policy inputs

Baseline seed policy is now composed from three layers (last match wins):

1. Built-in defaults in `fretboard`.
2. One or more JSON presets via:
   - `--perf-baseline-seed-preset <path>` (repeatable; applied in CLI order).
3. One or more explicit CLI overrides via:
   - `--perf-baseline-seed <scope@metric=max|p90|p95>` (highest precedence).

## Scope syntax (`--perf-baseline-seed`)

Use:

```bash
--perf-baseline-seed <scope@metric=max|p90|p95>
```

Supported `scope` values:

- Script path (repo-relative or absolute):
  - `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- Built-in perf suite name:
  - `ui-gallery`
  - `ui-gallery-steady`
- `this-suite`:
  - expands to all scripts of the currently selected named suite in `diag perf`.
- `suite:<name>`:
  - explicit named suite expansion without relying on active suite context.
- `*`:
  - wildcard for all scripts.

Notes:

- The flag is repeatable; later rules override earlier ones for the same `(script, metric)`.
- Built-in defaults still apply unless overridden by preset/CLI rules.

## JSON preset schema (`--perf-baseline-seed-preset`)

Preset files are versioned policy artifacts (commit into `docs/workstreams/perf-baselines/policies/`).

```json
{
  "schema_version": 1,
  "kind": "perf_baseline_seed_policy",
  "default_seed": "max",
  "rules": [
    {
      "scope": "ui-gallery-steady",
      "metric": "top_total_time_us",
      "seed": "p90"
    },
    {
      "scope": "tools/diag-scripts/ui-gallery-window-resize-stress-steady.json",
      "metric": "top_layout_time_us",
      "seed": "p95"
    }
  ]
}
```

Requirements:

- `schema_version` must be `1`.
- `kind` must be `perf_baseline_seed_policy`.
- `default_seed` is optional (`max|p90|p95`), and overrides built-in default seed when present.
- `rules` is required (can be empty).
- Each rule requires `scope`, `metric`, `seed`.

## Recommended baseline profile (v15)

Current default bias:

- default seed: `max`
- built-in override:
  - `ui-gallery-window-resize-stress-steady` uses `p95` for:
    - `top_total_time_us`
    - `top_layout_time_us`
    - `top_layout_engine_solve_time_us`

Example command (steady suite + preset + local override):

```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --json \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json \
  --perf-baseline-headroom-pct 20 \
  --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json \
  --perf-baseline-seed this-suite@top_layout_time_us=p90 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

## Suggested milestone ladder

- Phase A (stabilize): `max` global + script-specific `p95` only for high-noise probes.
- Phase B (tighten): move selected scripts/metrics to `p90` where repeat=7 remains stable.
- Phase C (gate hardening): keep `p90/p95` only where justified by noise evidence; avoid blanket wildcard rules.

## Change checklist

When updating seed policy:

1. Record the exact command in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`.
2. Capture validation run (`--perf-baseline`) and confirm `check.perf_thresholds.json` failures are `0`.
3. Update `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md` with the policy decision.
4. Keep `docs/workstreams/perf-baselines/*.json` policy header (`threshold_seed_policy`) in sync with the run.
5. For preset changes, bump preset version file (for example `ui-gallery-steady.v2.json`) instead of in-place overwrite.
