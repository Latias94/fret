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
python tools/perf/diag_editor_paint_contract_preflight.py
```

This checks the three editor probe JSON files, their required overlay-disabled `meta.env_defaults`, the diag script
registry, and the strict baseline matrix audit without running the long perf validation passes.

## Target-Machine Runner

Prefer the checked-in runner for the full Windows RTX4090 validation pass:

```powershell
python tools/perf/diag_editor_paint_contract_validate.py `
  --date-tag <date>
```

Run the release builds above first; the runner fails fast if the expected Windows binaries are missing.
Use `--dry-run` on non-target hosts to inspect the exact command plan without producing misleading local evidence.
For non-dry-run validation, use a fresh `--date-tag` / `--out-dir`; the runner rejects an existing non-empty output
directory by default so closeout evidence cannot accidentally mix stale dry-run or failed-run artifacts. Use
`--allow-existing-out-dir` only for an intentional debugging/resume run that will not be used as closeout evidence.
Run the command once without `--with-paint-perf`; that baseline-validation pass is the source for
`check.perf_thresholds.json.failures=[]`. Use `--with-paint-perf` only for the follow-up target-machine attribution
pass after baseline validation; do not use its output to loosen checked-in thresholds without the re-seed policy below.
The resize helper already enables `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1` for `ui-code-editor-resize-probes`; the flag
mainly controls the direct `diag perf` typical-autoscroll and complex-wheel probes.
After each validation probe, the runner collects the worst bundle and writes `diag stats --sort cpu_cycles --top 15
--json` output under that probe's `runner-logs/<probe>/stats.stdout.json`. The runner also checks that the stats JSON
contains paint-widget and renderer text/encode/upload fields; `--with-paint-perf` additionally requires
`code_editor_paint_perf`. It also treats any non-empty `check.perf_thresholds.json.failures` as a validation failure.

After copying the two target-machine output directories back into the workspace, verify the synced artifacts without
rerunning the probes:

```powershell
python tools/perf/diag_editor_paint_contract_verify_artifacts.py `
  target/fret-diag/editor-paint-contract-validate-<date> `
  --attribution-dir target/fret-diag/editor-paint-contract-validate-<date>-attrib
```

This verifier checks that both summaries carry a non-empty `date_tag`, that the baseline-validation pass used the
expected repeat/warmup shape, that every probe has `check.perf_thresholds.json` with `failures=[]`, that `diag stats
--json` output exists for every worst bundle, and that the attribution pass includes `code_editor_paint_perf` coverage.
It also rejects target summaries whose stored commands drift from the contract shape: resize must use the Windows
code-editor resize suite plus release `fretboard-dev.exe` / `fret-ui-gallery.exe`, while the direct `diag perf` probes
must use `--reuse-launch`, the standard font prewarm and reset-diagnostics prelude, `--json`, the release gallery
binary, and the required overlay-disabled env set. The baseline-validation direct `diag perf` commands must not set
`FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`; that flag belongs to the attribution directory for direct probes. The resize
helper may still collect its code-editor paint-perf fields internally.

## Minimal Closeout Sequence

Use this sequence on the Windows RTX4090 target host after building the release binaries. It is the shortest command
chain that can satisfy the closeout objective; dry-run output and non-target-host output are not valid substitutes.

```powershell
$DateTag = "YYYYMMDD-editor-paint"

python tools/perf/diag_editor_paint_contract_validate.py `
  --date-tag $DateTag

python tools/perf/diag_editor_paint_contract_validate.py `
  --date-tag "$DateTag-attrib" `
  --with-paint-perf
```

After both output directories are synced back to the workspace that will update the docs, run the verifier/closeout
gate:

```powershell
python tools/perf/diag_editor_paint_contract_closeout.py `
  "target/fret-diag/editor-paint-contract-validate-$DateTag" `
  --attribution-dir "target/fret-diag/editor-paint-contract-validate-$DateTag-attrib"
```

Closeout is valid only when the first directory has a baseline-validation `summary.json` with
`with_paint_perf=false`, the second directory has an attribution `summary.json` with `with_paint_perf=true`, and the
closeout command reports `ok=true`.

## Validate Current Contracts First

Resize:

```powershell
python tools/perf/diag_resize_probes_gate.py `
  --suite ui-code-editor-resize-probes `
  --out-dir target/fret-diag/editor-paint-contract-validate-<date>-resize `
  --attempts 3 `
  --repeat 7 `
  --warmup-frames 5 `
  --fretboard-bin target/release/fretboard-dev.exe `
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

After copying the target-machine validation directories back into the workspace, prefer the one-command local closeout
gate:

```powershell
python tools/perf/diag_editor_paint_contract_closeout.py `
  target/fret-diag/editor-paint-contract-validate-<date> `
  --attribution-dir target/fret-diag/editor-paint-contract-validate-<date>-attrib
```

It verifies the synced artifacts, then runs the strict baseline matrix audit, `WORKSTREAM.json` parsing, workstream
catalog check, and `git diff --check`.
Use `--dry-run` before artifacts arrive to inspect the local closeout plan; dry-run intentionally skips artifact
verification.

Equivalent manual gates before closing P1.5:

```powershell
python tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-<date> --attribution-dir target/fret-diag/editor-paint-contract-validate-<date>-attrib
python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json
```

Then update:

- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-audit.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-todo.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`

P1.5 is not closed until those docs point at target-machine artifacts, not local macOS evidence.
