# Perf Baseline Maintenance

Status: Draft workstream contract.

This directory stores checked-in `diag perf` baselines and seed-policy presets for Fret's editor-grade performance
contracts.

Related:

- Contract matrix: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md`
- Active perf workstream: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1.md`
- Perf log: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- Seed policy template: `docs/workstreams/perf-baselines/seed-policy-template.md`

## Baseline Contract

- New `diag perf --perf-baseline-out` files record `measured_p50`, `measured_p90`, `measured_p95`, and
  `measured_max` per row.
- Thresholds live in `rows[].thresholds`; gates read those threshold values, not the measured fields directly.
- New baselines also declare `threshold_surface`:
  - `ui`: frame/layout/pointer/cache thresholds only. Renderer timings stay in `measured_*` for attribution, but are
    not hard gate thresholds.
  - `ui-renderer-payload`: UI thresholds plus renderer payload thresholds such as instance bytes and encoded text ops.
    Renderer micro-timings stay in `measured_*` for attribution.
  - `renderer-payload`: renderer payload thresholds only.
  - `renderer`: renderer micro-timing thresholds plus renderer payload thresholds.
  - `all`: UI thresholds, renderer micro-timing thresholds, and renderer payload thresholds. Use this only for suites
    that intentionally gate renderer micro timings.
- For repeated runs, the resolved seed policy declares `ui_threshold_mode`:
  - `top` gates `max_top_*` and is the default tail / worst-frame contract.
  - `frame_p95` gates `max_frame_p95_*` for typical-frame contracts.
  - `top_and_frame_p95` gates both when the probe intentionally protects tail and typical smoothness.
  Do not rely on script or suite names to choose this mode.
- Existing baselines created before `measured_p50` was added remain valid. Do not synthesize `measured_p50` into old
  JSON files. Add it only by intentionally re-seeding on the target machine profile.
- Baselines are environment-specific. Do not use a macOS baseline to judge Windows, or a high-end Windows baseline to
  judge a lower-end local machine unless that is the explicit contract being tested.

## Machine Tags

Use stable suffixes in baseline filenames:

- `windows-rtx4090`: primary Windows GPU workstation profile.
- `macos-m4` / `macos-m4pro`: Apple Silicon local profiles.
- `windows-local`, `web-local`: broader smoke profiles where hardware-specific precision is not intended.
- Baselines may include an explicit build-profile segment such as `dev-fast` when the contract is intentionally tied to
  a non-release binary, for example `*.dev-fast.windows-rtx4090.v1.json`. Treat these as workstream regression gates,
  not formal release performance contracts, unless the owning workstream explicitly says otherwise.
- `linux-local`: Linux-local evidence profile. Smoke-only exports may exist here while runner bring-up is in progress,
  but do not infer formal Linux contract coverage from them or from the Windows/macOS baselines. Add a checked-in
  Linux contract baseline and owner profile only when a real Linux runner exists.

If a new machine profile becomes a contract target, add a new baseline file rather than overwriting an unrelated
profile.

## When To Re-Seed

Re-seed a checked-in baseline only when at least one of these is true:

- Suite membership or script behavior changed intentionally.
- A measured optimization changed the expected cost profile and the perf log records before/after evidence.
- The current gate is flaky after normalization hooks and attempts are in place, and evidence shows the baseline
  itself is stale rather than a real regression.
- The machine profile, driver, runtime, or toolchain profile changed enough that old numbers are no longer comparable.
- A baseline lacks a newly required measured field such as `measured_p50`, and the workstream explicitly decides to
  refresh that contract.

Do not re-seed to hide an unexplained gate failure. First inspect `check.perf_thresholds.json`, then run `diag stats`
on the reported worst bundle.

## Required Normalization

Use suite hooks for all formal baseline generation and validation:

- Prewarm: `tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json`
- Prelude: `tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json`
- Env:
  - `FRET_DIAG_SCRIPT_AUTO_DUMP=0`
  - `FRET_DIAG_SEMANTICS=0`
  - gallery gates usually also use `FRET_UI_GALLERY_VIEW_CACHE=1` and `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`

`tools/perf/diag_resize_probes_gate.py`, `tools/perf/diag_resize_probes_gate.sh`,
`tools/perf/diag_perf_baseline_select.py`, and `tools/perf/diag_perf_baseline_select.sh` apply the default prewarm and
prelude hooks unless `--no-default-suite-hooks` is passed.

## Selection Workflow

Prefer the cross-platform Python selector:

```powershell
python tools/perf/diag_perf_baseline_select.py `
  --suite ui-resize-probes `
  --baseline-out docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v2.json `
  --preset docs/workstreams/perf-baselines/policies/ui-resize-probes.v1.json `
  --candidates 2 `
  --validate-runs 3 `
  --repeat 7 `
  --warmup-frames 5 `
  --headroom-pct 20 `
  --threshold-surface ui `
  --work-dir target/fret-diag-baseline-select-ui-resize-probes-windows-rtx4090-v2 `
  --launch-bin target/release/fret-ui-gallery.exe `
  --env FRET_UI_GALLERY_VIEW_CACHE=1 `
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 `
  --env FRET_DIAG_SEMANTICS=0
```

Selection priority:

1. Fewer validation failures.
2. No threshold loosening compared with an existing `--baseline-out` file.
3. Lower suite p90 sum (`rows[].measured_p90.top_total_time_us`).
4. Lower threshold sum (`rows[].thresholds.max_top_total_us`).

The selector validates each candidate using the same repeat count as the generated baseline unless
`--validate-repeat` is passed. The selected candidate must have `fail_total=0`; otherwise the selector exits without
copying to `--baseline-out` unless `--allow-failures` is explicitly passed for an investigation artifact. When
`--baseline-out` already exists, the selected candidate also must not increase/remove existing hard thresholds unless
`--allow-threshold-loosening` is explicitly passed for an intentional machine-profile or contract reset. The selector
writes `selection-summary.json` in `--work-dir`.

For optimization-led re-seeds where only part of the contract should move, pass `--clamp-threshold-loosening`.
Candidate baselines are validated with existing stricter thresholds preserved whenever the candidate's measured value
still fits that older threshold. If a metric really cannot satisfy the old threshold, it remains a loosening and must
either fail selection or be justified with `--allow-threshold-loosening`.

For mixed suites whose scripts declare different launch-time `meta.env_defaults`, pass
`--reuse-launch-per-script` so `diag perf` reuses one launched process per compatible script group instead of forcing a
single launch environment across the entire suite.

## Validation Workflow

After re-seeding, validate the new baseline with the matching gate:

```powershell
python tools/perf/diag_resize_probes_gate.py `
  --suite ui-resize-probes `
  --baseline docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v2.json `
  --attempts 3 `
  --repeat 7 `
  --warmup-frames 5 `
  --launch-bin target/release/fret-ui-gallery.exe
```

For non-resize suites, run `diag perf` directly with `--perf-baseline <baseline>` and the same normalization hooks, or
add a small gate helper before promoting the baseline to a daily contract.

## Review Checklist

Before committing a new or replaced baseline:

- Confirm the baseline filename has the correct suite, machine tag, and version.
- Confirm the JSON row set matches the intended suite.
- Confirm new rows include `measured_p50` unless the baseline is intentionally old and unchanged.
- Confirm `threshold_surface` matches the suite intent. Resize/layout suites normally use `ui`; renderer/effects suites
  use `renderer` or `all` only when those micro timings are the contract. Use `ui-renderer-payload` when the suite
  should gate CPU/UI time plus renderer payload size/counts without gating noisy renderer micro-timings.
- Confirm validation passes with `failures=[]`.
- Confirm a replacement baseline does not loosen existing hard thresholds unless the perf log explains why
  `--allow-threshold-loosening` was intentionally used.
- Record the exact commands, selected candidate, validation result, and worst bundles in the perf log.
- Update `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md` if the suite, baseline path, or gate
  command changes.
- Run `python tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
  before promoting a new or replaced baseline. The audit reports legacy evidence baselines separately so old broad
  suites can remain in the repo without pretending to be the representative p50/p95/max contract.
- Use a docs or perf commit message that identifies the suite and machine profile.
