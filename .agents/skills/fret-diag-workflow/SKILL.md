---
name: fret-diag-workflow
description: "Reproduce and debug Fret UI issues with `fretboard diag`: scripted UI automation, diagnostics bundles, screenshots, triage/compare, and turning bugs into stable repro gates. Use when authoring or running `tools/diag-scripts/*.json`, packaging bundles for sharing, or analyzing invalidation/perf regressions."
---

# Fret diag workflow

If your primary goal is to quantify performance (baselines/gates/logs), use `fret-perf-workflow` instead.

## Quick start

- Run a script and launch the target app (recommended for reproducibility):
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --env FRET_DIAG=1 --env FRET_DIAG_SCREENSHOTS=1 --pack --launch -- cargo run -p fret-ui-gallery --release`

## Workflow

1. Pick the smallest target that shows the bug.
   - Prefer a UI gallery page or a dedicated demo binary.
2. Create or edit a script in `tools/diag-scripts/`.
   - Use stable `test_id` targets instead of pixel coordinates.
   - Common steps: `click`, `wait_until`, `capture_bundle`, `capture_screenshot`.
3. Ensure diagnostics are enabled in the running app.
   - Minimum: `FRET_DIAG=1`
   - If the script uses `capture_screenshot`: also enable `FRET_DIAG_SCREENSHOTS=1`.
   - While authoring scripts, consider disabling text redaction: `FRET_DIAG_REDACT_TEXT=0`.
   - Full env reference: `docs/ui-diagnostics-and-scripted-tests.md`
4. Run the script via `fretboard` and collect artifacts.
   - Prefer `fretboard diag run ... --launch -- <cmd...>` so env vars are applied consistently.
5. Turn the repro into a gate (stable assertions first).
   - Prefer geometry/semantics invariants over pixel diffs when possible.
   - If you need pixel diffs, add `capture_screenshot` steps and use `--check-pixels-changed <test_id>`.
6. Package and share.
   - `fretboard diag pack --include-screenshots` (bundle + screenshots)
   - `fretboard diag triage <bundle_dir|bundle.json> --json` (machine-readable summary)
7. Compare before/after runs for regressions.
   - `fretboard diag compare <bundle_a> <bundle_b> --json`

## Perf triage handoff (when the “bug” is a hitch)

If the issue is “it feels janky” (resize/scroll/pointer-move) rather than a correctness regression:

1. Switch to `fret-perf-workflow` and run an appropriate gate/suite (`ui-gallery-steady`, `ui-resize-probes`, etc).
2. When a `diag perf` run fails, start with the thresholds file:
   - `<out-dir>/check.perf_thresholds.json` (or `attempt-N/check.perf_thresholds.json` for gate scripts)
3. Use the worst bundle for root cause:
   - `cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30`
4. Turn the hitch class into a stable probe or a stricter gate once it is explainable:
   - Add a `tools/diag-scripts/*.json` script (stable `test_id` targets), then baseline/gate it.

## Tips

- Add `test_id` at the recipe/component layer (usually `ecosystem/fret-ui-shadcn`) so scripts remain stable across layout refactors.
- Keep scripts minimal: one bug, one script, one or two assertions.
- Prefer `tools/diag-scripts/` naming that encodes the scenario (component + behavior + expectation).
