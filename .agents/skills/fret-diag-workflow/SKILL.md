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

- Web/WASM: see `references/web-runner.md`.

## Common commands (copy/paste)

- Author scripts:
  - `fretboard diag script normalize <script.json> --write`
  - `fretboard diag script validate <script.json>`
  - `fretboard diag script lint <script.json>`
  - Directory/glob inputs work too:
    - `fretboard diag script validate tools/diag-scripts`
    - `fretboard diag script lint tools/diag-scripts/ui-gallery-select-*.json`

- Run + collect artifacts (recommended):
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; cargo run -p fretboard -- diag run <script.json> --launch -- <cmd...>"`
  - Use `FRET_DIAG_SCREENSHOTS=1` when the script captures screenshots.

- Suite runs (batch scripts):
  - `fretboard diag suite ui-gallery-select --launch -- <cmd...>`
  - After a suite run, check `suite.summary.json` under the output dir for a one-file overview.

- Flake triage:
  - `fretboard diag repeat <script.json> --repeat 7 --launch -- <cmd...>`
  - Read `repeat.summary.json` (highlights + evidence aggregates) before opening bundles.

- Minimize a failing script (ddmin):
  - `fretboard diag script shrink <script.json> --reuse-launch --launch -- <cmd...>`

- Compare bundles:
  - `fretboard diag compare <bundle_a> <bundle_b> --json`

- Select conformance suite “template ↔ JSON” closure:
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-select`

## Capabilities & fail-fast gating

Goal: missing support should **fail fast with a structured reason**, not degrade into timeouts.

Practical rules:

- Always treat capabilities as namespaced strings (`diag.*`, `devtools.*`).
- Tooling should fail fast when `required_capabilities - available_capabilities` is non-empty.
- When gating fails, look for `check.capabilities.json` in the run output dir.

Where capabilities come from:

- filesystem transport: runner writes `capabilities.json` under `FRET_DIAG_DIR`
- devtools-ws transport: the app advertises capabilities as part of hello/session descriptors

## Playbooks

- Evidence triage checklist: `references/evidence-triage.md`
- Select conformance playbook: `references/select-conformance.md`
- Layout sweep playbook: `references/layout-sweep.md`
- Perf handoff notes: `references/perf-handoff.md`

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

- Web/WASM transport: `references/web-runner.md`
- Evidence triage checklist: `references/evidence-triage.md`
- Select conformance playbook: `references/select-conformance.md`
- Layout sweep playbook: `references/layout-sweep.md`
- Perf handoff: `references/perf-handoff.md`
