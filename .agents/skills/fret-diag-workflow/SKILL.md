---
name: fret-diag-workflow
description: "Reproduce and debug Fret UI issues with `fretboard diag`: scripted interaction automation, diagnostics bundles, screenshots, and triage/compare. Use when authoring or running `tools/diag-scripts/*.json`, turning a flaky UI bug into a stable repro gate, or when you need shareable artifacts for AI/humans."
---

# Fret diag workflow

## Quick start

- Native (recommended): run a script and launch the app:
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; $env:FRET_DIAG_SCREENSHOTS=1; $env:FRET_DIAG_REDACT_TEXT=1; cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --timeout-ms 240000 --pack --launch -- cargo run -p fret-ui-gallery --release"`
- Suite run (batch scripts):
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; $env:FRET_DIAG_REDACT_TEXT=1; cargo run -p fretboard -- diag suite ui-gallery-select --timeout-ms 240000 --launch -- cargo run -p fret-ui-gallery --release"`
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; $env:FRET_DIAG_REDACT_TEXT=1; cargo run -p fretboard -- diag suite ui-gallery-combobox --timeout-ms 240000 --launch -- cargo run -p fret-ui-gallery --release"`
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; $env:FRET_DIAG_REDACT_TEXT=1; cargo run -p fretboard -- diag suite ui-gallery-text-ime --timeout-ms 240000 --launch -- cargo run -p fret-ui-gallery --release"`
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
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; $env:FRET_DIAG_REDACT_TEXT=1; cargo run -p fretboard -- diag run <script.json> --timeout-ms 240000 --launch -- <cmd...>"`
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

- Component conformance “template ↔ JSON” closure:
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-select`
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-combobox`
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-text-ime`

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
- Scripts that depend on label text while running with redaction enabled (`FRET_DIAG_REDACT_TEXT=1`).
- Running a binary not wired through the diagnostics driver (no bundles/scripts).
- Using sleeps instead of `click_stable` / `wait_bounds_stable`.
- Windows: a previously-launched `fret-ui-gallery.exe` can keep `target/release/*.exe` locked during rebuilds.

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
