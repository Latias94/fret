# Editor Paint Contract Stabilization Runbook

Status: target-machine runbook for closing P1.5.

This runbook owns the final step after local Editor Canvas replay attribution: validate or deliberately re-seed the
editor paint contracts on the target Windows RTX4090 profile without using macOS M4 evidence to change checked-in
Windows baselines.

## Scope

Required probes:

- Typical autoscroll: `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-typical.json`
- Complex wheel: `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json`
- Resize jitter: `ui-code-editor-resize-probes`, backed by
  `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`

Required launch shape:

- `FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0`
- standard font prewarm and reset-diagnostics prelude
- `FRET_A11Y_DISABLE=1` for non-a11y editor CPU/renderer contracts
- `FRET_UI_GALLERY_VIEW_CACHE=1` and `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`
- `FRET_DIAG_SCRIPT_AUTO_DUMP=0` and `FRET_DIAG_SEMANTICS=0`

Use `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1` for attribution/evidence passes. Keep calibrated baseline validation runs
aligned with the matrix command unless a deliberate policy note says the diagnostic overhead is part of the contract.
For closeout, run the same three probes once with the validation shape plus `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1` and a
dated `--dir target/fret-diag/editor-paint-contract-attribution-<date>-...`; use those bundles for
`code_editor_paint_perf` and per-row gap evidence.

## Build

```powershell
cargo build -p fretboard-dev --release
cargo build -p fret-ui-gallery --release --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness
```

## Preflight

Run this before a long target-machine validation pass:

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-typical.json
python -m json.tool tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json
python -m json.tool tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json
python tools/check_diag_scripts_registry.py
python tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict
```

## Validate Current Contracts First

Resize:

```powershell
python tools/perf/diag_resize_probes_gate.py `
  --suite ui-code-editor-resize-probes `
  --out-dir target/fret-diag/editor-paint-contract-validate-<date>-resize `
  --attempts 3 `
  --repeat 7 `
  --warmup-frames 5 `
  --launch-bin target/release/fret-ui-gallery.exe
```

Typical autoscroll:

```powershell
target/release/fretboard-dev.exe diag perf `
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-typical.json `
  --dir target/fret-diag/editor-paint-contract-validate-<date>-typical `
  --repeat 15 `
  --warmup-frames 5 `
  --reuse-launch `
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json `
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json `
  --sort time `
  --top 15 `
  --json `
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json `
  --env FRET_A11Y_DISABLE=1 `
  --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 `
  --env FRET_UI_GALLERY_VIEW_CACHE=1 `
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 `
  --env FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0 `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 `
  --env FRET_DIAG_SEMANTICS=0 `
  --launch -- target/release/fret-ui-gallery.exe
```

Complex wheel:

```powershell
target/release/fretboard-dev.exe diag perf `
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json `
  --dir target/fret-diag/editor-paint-contract-validate-<date>-complex-wheel `
  --repeat 7 `
  --warmup-frames 5 `
  --reuse-launch `
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json `
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json `
  --sort time `
  --top 15 `
  --json `
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.windows-rtx4090.v1.json `
  --env FRET_A11Y_DISABLE=1 `
  --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 `
  --env FRET_UI_GALLERY_VIEW_CACHE=1 `
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 `
  --env FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0 `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 `
  --env FRET_DIAG_SEMANTICS=0 `
  --launch -- target/release/fret-ui-gallery.exe
```

## Attribution Evidence

For the worst bundle from each probe, run:

```powershell
target/release/fretboard-dev.exe diag stats <bundle.schema2.json> --sort cpu_cycles --top 15 --json
```

The closeout note must record:

- `paint.widget` p50/p95 and top frame.
- `code_editor_paint_perf` p50/p95 for row replay/cache, row store/capture, and windowed-surface per-row gaps.
- renderer text/encode/upload and payload counters.
- `top_code_editor_torture_overlay_us=0`.

## Re-Seed Policy

Do not re-seed to make a failing gate green. Re-seed only when:

- the validation pass is stable enough to explain,
- the old threshold no longer represents the intended overlay-disabled contract surface, and
- `--clamp-threshold-loosening` can preserve existing stricter thresholds unless a policy note explicitly justifies a
  threshold reset.

Use `tools/perf/diag_perf_baseline_select.py` for non-resize editor probes. Keep `--allow-threshold-loosening` absent by
default. If it is used, the perf log must explain the machine-profile or contract reset.

## Closeout Gate

Before closing P1.5:

```powershell
python tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict
python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json
```

Then update:

- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-audit.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-todo.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`

P1.5 is not closed until those docs point at target-machine artifacts, not local macOS evidence.
