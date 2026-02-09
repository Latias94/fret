---
name: fret-diag-workflow
description: "Reproduce and debug Fret UI issues with `fretboard diag`: scripted interaction automation, diagnostics bundles, screenshots, and triage/compare. Use when authoring or running `tools/diag-scripts/*.json`, turning a flaky UI bug into a stable repro gate, or when you need shareable artifacts for AI/humans."
---

# Fret diag workflow

## When to use

Use this skill when:

- A UI bug is hard to reproduce, flaky, or requires “human timing”.
- You need a **shareable artifact** (bundle + optional screenshots) for triage.
- You want to convert a bug into a **CI-friendly gate** (script + assertions).

If your primary goal is performance quantification (baselines/gates/logs), use `fret-perf-workflow` instead.
If your goal is to **explain a hitch** (tail latency) and choose the next profiler/capture, use `fret-perf-attribution`.

## Quick start

- Run a script and launch the target app (recommended for reproducibility):
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; $env:FRET_DIAG_SCREENSHOTS=1; cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --pack --launch -- cargo run -p fret-ui-gallery --release"`

- Web runner (WASM): export bundles via devtools-ws (headless-friendly):
  - Start the loopback WS hub (prints the token): `cargo run -p fret-devtools-ws`
  - Serve the WASM app: `cd apps/fret-ui-gallery-web && trunk serve --port 8080`
  - Open (note the query params): `http://127.0.0.1:8080/?fret_devtools_ws=ws://127.0.0.1:7331/&fret_devtools_token=<token>`
  - Run the script and materialize `.fret/diag/exports/<timestamp>/bundle.json`:
    - `cargo run -p fret-diag-export -- --script tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --token <token>`

## Workflow

1. Pick the smallest target that shows the bug.
   - Prefer a UI gallery page or a dedicated demo binary.
2. Create or edit a script in `tools/diag-scripts/`.
   - Use stable `test_id` targets instead of pixel coordinates.
   - Common steps: `click`, `wait_until`, `capture_bundle`, `capture_screenshot`.
   - If the target moves/animates during navigation, prefer `click_stable` (schema v2) to avoid “stale click” flake.
     - Example: click only after the target’s center stays within `eps_px` for `stable_frames`.
3. Ensure diagnostics are enabled in the running app.
   - Minimum: `FRET_DIAG=1`
   - If the script uses `capture_screenshot`: also enable `FRET_DIAG_SCREENSHOTS=1`.
   - While authoring scripts, consider disabling text redaction: `FRET_DIAG_REDACT_TEXT=0`.
   - Full env reference: `docs/ui-diagnostics-and-scripted-tests.md`
4. Run the script via `fretboard` and collect artifacts.
   - Prefer `fretboard diag run ... --launch -- <cmd...>` so env vars are applied consistently.
   - Web runner note: `fretboard diag run` uses the filesystem-trigger transport; for web/WASM use
     devtools-ws + `fret-diag-export` (or `apps/fret-devtools`) to export bundles under
     `.fret/diag/exports/`.
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
   - Tip: `fret-perf-workflow` includes a compact gate triage helper:
     `.agents/skills/fret-perf-workflow/scripts/triage_gate.sh <out-dir>`
3. Use the worst bundle for root cause:
   - `cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30`
4. Turn the hitch class into a stable probe or a stricter gate once it is explainable:
   - Add a `tools/diag-scripts/*.json` script (stable `test_id` targets), then baseline/gate it.

### “Resize jank” fast path (copy/paste)

Run the P0 resize probes (numbers + thresholds):

```bash
tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3
tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes --attempts 3
```

If a gate fails (or you want the worst bundles even on PASS):

```bash
.agents/skills/fret-perf-workflow/scripts/triage_gate.sh <out-dir> --all --app-snapshot
```

Then inspect the worst bundle:

```bash
cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30
```

## Tips

- Add `test_id` at the recipe/component layer (usually `ecosystem/fret-ui-shadcn`) so scripts remain stable across layout refactors.
- Keep scripts minimal: one bug, one script, one or two assertions.
- Prefer `tools/diag-scripts/` naming that encodes the scenario (component + behavior + expectation).
- When a selector target is known to jitter (virtualized lists, animated overlays, resize/relayout), use `click_stable`
  rather than retrying `click` with arbitrary sleeps.

## Evidence anchors

Where the code lives:

- Doc: `docs/ui-diagnostics-and-scripted-tests.md`
- In-app exporter + script executor: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- CLI entry: `apps/fretboard/src/diag.rs`
- Headless exporter (devtools-ws -> `.fret/diag/exports/`): `apps/fret-diag-export`
- Loopback WS hub: `apps/fret-devtools-ws`
- DevTools GUI (optional): `apps/fret-devtools`
- Protocol types (scripts, selectors, results): `crates/fret-diag-protocol`
- Triage/compare engine: `crates/fret-diag`

## Common pitfalls

- Scripts that call `capture_screenshot` without `FRET_DIAG_SCREENSHOTS=1`.
- Targeting pixels/coordinates instead of `test_id`/semantics selectors (scripts become brittle).
- Running the “wrong” binary that isn’t wired through the diagnostics driver (no bundle/script execution).
- Debugging an interaction bug with only geometry snapshots: add scripted steps + focused assertions.
- Web runner:
  - Forgetting `fret_devtools_ws` / `fret_devtools_token` query params (no WS bridge, no scripts/bundles).
  - Assuming the web app can write `target/fret-diag/...` (it cannot; you must export via WS).
  - Running a script that never calls `capture_bundle` (nothing to export).

## Related skills

- `fret-shadcn-source-alignment` (turn Radix/shadcn mismatches into tests + scripts)
- `fret-overlays-and-focus` (overlay/dismiss/focus issues)
- `fret-perf-workflow` (perf baselines/gates)
