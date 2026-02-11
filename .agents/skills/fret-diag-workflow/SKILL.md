---
name: fret-diag-workflow
description: "Reproduce and debug Fret UI issues with `fretboard diag`: scripted interaction automation, diagnostics bundles, screenshots, and triage/compare. Use when authoring or running `tools/diag-scripts/*.json`, turning a flaky UI bug into a stable repro gate, or when you need shareable artifacts for AI/humans."
---

# Fret diag workflow

## When to use

Use this skill when you need:

- A **repro script** for a flaky/self-drawn UI bug.
- A **shareable artifact** (bundle + optional screenshots) for triage.
- A **CI-friendly gate** (script + assertions + lint/check outputs).

If your primary goal is performance quantification (baselines/gates/logs), use `fret-perf-workflow` instead.
If your goal is to **explain a hitch** (tail latency) and choose the next profiler/capture, use `fret-perf-attribution`.

## Quick start

- Native (recommended): run a script and launch the app:
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; $env:FRET_DIAG_SCREENSHOTS=1; cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --pack --launch -- cargo run -p fret-ui-gallery --release"`

- Web/WASM: see `.agents/skills/fret-diag-workflow/references/web-runner.md`.

## Workflow

1. Pick the smallest target that shows the bug.
   - Prefer a UI gallery page or a dedicated demo binary.
2. Create or edit a script in `tools/diag-scripts/`.
   - Use stable `test_id` targets instead of pixel coordinates.
   - Prefer **intent-level v2 steps** over sleeps:
     - `click_stable`: avoid stale coordinates when bounds jump.
     - `wait_bounds_stable`: wait for overlay/content bounds to settle (flip/shift/estimate→measured).
   - Prefer declaring `meta.required_capabilities` for any non-trivial evidence requirements (screenshots, window targeting, etc).
   - Keep scripts reviewable and CI-friendly:
     - `fretboard diag script normalize <script.json> --write`
     - `fretboard diag script validate <script.json>`
     - `fretboard diag script lint <script.json>`
     - You can also lint/validate by directory or glob (the CLI expands patterns):
       - `fretboard diag script lint tools/diag-scripts/ui-gallery-select-*.json`
       - `fretboard diag script validate tools/diag-scripts`
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
   - `fretboard diag lint <bundle_dir|bundle.json> --json` (sanity checks: duplicate `test_id`, focused/active out-of-window, etc.)
   - Note: `fretboard diag suite ...` runs lint for every captured bundle by default (use `--no-lint` to disable).
7. Compare before/after runs for regressions.
   - `fretboard diag compare <bundle_a> <bundle_b> --json`

## Capabilities & fail-fast gating

Goal: missing support should **fail fast with a structured reason**, not degrade into timeouts.

Practical rules:

- Always treat capabilities as namespaced strings (`diag.*`, `devtools.*`).
- Tooling should fail fast when `required_capabilities - available_capabilities` is non-empty.
- When gating fails, look for `check.capabilities.json` in the run output dir.

Where capabilities come from:

- filesystem transport: runner writes `capabilities.json` under `FRET_DIAG_DIR`
- devtools-ws transport: the app advertises capabilities as part of hello/session descriptors

## Evidence-first debugging (what to read)

Evidence and triage checklist: `.agents/skills/fret-diag-workflow/references/evidence-triage.md`.

## Component conformance playbook (example: shadcn `Select`)

Select conformance playbook: `.agents/skills/fret-diag-workflow/references/select-conformance.md`.

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

- `capture_screenshot` without `FRET_DIAG_SCREENSHOTS=1`.
- Pixel/coordinate targeting instead of `test_id` selectors.
- Running a binary not wired through the diagnostics driver (no bundles/scripts).
- Using sleeps instead of `click_stable` / `wait_bounds_stable`.

## Related skills

- `fret-shadcn-source-alignment` (turn Radix/shadcn mismatches into tests + scripts)
- `fret-overlays-and-focus` (overlay/dismiss/focus issues)
- `fret-perf-workflow` (perf baselines/gates)

## Further reading (skill-local references)

- Web/WASM transport: `.agents/skills/fret-diag-workflow/references/web-runner.md`
- Evidence triage checklist: `.agents/skills/fret-diag-workflow/references/evidence-triage.md`
- Select conformance playbook: `.agents/skills/fret-diag-workflow/references/select-conformance.md`
- Layout sweep playbook: `.agents/skills/fret-diag-workflow/references/layout-sweep.md`
- Perf handoff: `.agents/skills/fret-diag-workflow/references/perf-handoff.md`
