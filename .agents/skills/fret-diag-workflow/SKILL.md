---
name: fret-diag-workflow
description: "Runs and triages Fret UI diagnostics via `fretboard diag` (scripts, bundles, shareable artifacts, compare/triage, perf gates). Use when user asks to reproduce a flaky UI bug, write a diag script, capture/share a diagnostics bundle, triage/compare bundles, or add a perf gate."
---

# Fret diagnostics workflow (correctness + perf)

## What this skill does

- Turns flaky UI bugs into deterministic repros (scripts + stable selectors).
- Produces portable, shareable artifacts (bundles, sidecars, AI packets, zips).
- Helps triage quickly without opening/grepping a huge `bundle.json`.
- Supports both correctness debugging and perf gating/attribution.

Use `fret-ui-review` when the goal is an architecture/UX audit rather than producing repro artifacts.

## Safety (bounded output)

- Do **not** run `rg` on `bundle.json` (or on `target/fret-diag/**` / `.fret/diag/**`) — it can explode to tens of
  thousands of lines.
- Do **not** `cat` / `Get-Content` a raw `bundle.json` (same explosion risk; it is frequently megabytes-to-hundreds-of-MB).
- Prefer bounded tooling queries:
  - `fretboard diag meta ...`
  - `fretboard diag query ...`
  - `fretboard diag slice ...`
- When you need repository-wide search, use `tools/rg-safe.ps1` (excludes diag artifact directories and bundle artifacts).

## Success criteria (what “good” looks like)

- A repro runs end-to-end with `--launch` and produces a bounded share artifact (`ai.packet/`, sidecars, and optionally
  `bundle.schema2.json`) without requiring `bundle.json` to be opened/grepped.
- The failure is explained by stable `reason_code` + bounded evidence in `script.result.json` / `triage.json`.
- If the issue is likely to regress: one landable gate exists (script suite and/or a Rust test).

## Best practices (repeatable habits)

- Prefer `--launch` for determinism; avoid relying on parent-shell `FRET_DIAG_*` for tool-launched runs.
- Before rerunning a suspiciously large or inconsistent run:
  - `fretboard diag config doctor --mode launch --print-launch-policy`
  - `fretboard diag config doctor --mode launch --report-json` (inspect `launch_policy` + warnings)
- Keep artifacts small by default:
  - capture only a few bundles at key points (not after every step),
  - prefer sidecars + `bundle.schema2.json` over raw `bundle.json`,
  - avoid `FRET_DIAG_BUNDLE_JSON_FORMAT=pretty` unless you truly need it.
- Use raw `bundle.json` only as an explicit escape hatch in tool-launched mode:
  - `--launch-write-bundle-json` (never for `diag matrix`).
- When triaging: prefer `diag meta/query/slice` over searching JSON.
- When a script is flaky: replace sleeps with stabilization (`click_stable`, `wait_until`, bounds-stable), and shrink.
- Always leave behind the 3-pack: repro script + bounded evidence bundle + regression gate (suite/check/test).

## When to use

- A UI bug is hard to reproduce, flaky, or requires “human timing”.
- You need a **shareable artifact** (bundle + optional screenshots) for triage.
- You want to convert a bug into a **CI-friendly gate** (script + assertions).

## Choose this vs adjacent skills

- Use this skill for **correctness repro + regression gating** (scripts, bundles, post-run checks).
- Use this skill for **perf gates + worst-bundle evidence** (`diag perf` + thresholds/baselines).
- Use `fret-ui-review` when the task is “audit this UI implementation” (not “turn this bug into a script”).

## Inputs to collect (ask the user)

Ask 3–6 questions up front so you don’t “debug the wrong thing”:

- Target: which app/demo/page reproduces it (smallest runnable target)?
- Platform/transport: native launch (filesystem) or web (DevTools WS)?
- Expected invariant: what should be true at the end (exists/focus/selection/command fired)?
- Evidence needs: bundle only, or screenshots/pixel checks as well?
- Flake shape: timing-sensitive, jittery targets, animation/virtualization involved?

Defaults if unclear:

- Use a UI gallery page + stable `test_id` selectors.
- Capture at least one `capture_bundle` step (screenshots only if they add signal).

## Quick start (native, recommended)

Run a promoted script by `script_id` (no path required):

- `cargo run -p fretboard -- diag run ui-gallery-command-palette-shortcut-primary --launch -- cargo run -p fret-ui-gallery --release`

Common “share what happened” flow:

- `cargo run -p fretboard -- diag run ui-gallery-intro-idle-screenshot --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --release`

List promoted scripts (discoverability):

- `cargo run -p fretboard -- diag list scripts`

List known suites (from promoted registry `suite_memberships`):

- `cargo run -p fretboard -- diag list suites`
- (filter) `cargo run -p fretboard -- diag list suites --contains perf-`

Check script library drift (taxonomy + redirects + registry):

- `cargo run -p fretboard -- diag doctor scripts`

## Choose a transport

- Native (filesystem-trigger; recommended for day-to-day):
  - Run: `fretboard diag run ... --launch -- <cmd>`
- Web/WASM (DevTools WS loopback):
  - Export: `cargo run -p fret-diag-export -- --script tools/diag-scripts/<script>.json --token <token>`
  - See: `references/web-runner.md`

## Authoring a script (v2-first)

1. Prefer schema v2 for new scripts (more intent-level steps; less flake).
2. Use stable semantics selectors (`test_id`) rather than pixel coordinates.
3. Prefer intent-level stabilization:
   - Use `click_stable` for jittery targets (virtualized lists, overlays, animations).
4. Declare capabilities explicitly when the script is intentionally narrow:
   - Screenshots: `diag.screenshot_png`
   - Touch / pen injection: `diag.pointer_kind_touch`, `diag.pointer_kind_pen`
   - Gestures: `diag.gesture_tap`, `diag.gesture_long_press`, `diag.gesture_swipe`, `diag.gesture_pinch`

Tip: if the user says “it only happens with touch/pen”, use `pointer_kind` on pointer-driven steps (capability-gated).
Supported `pointer_kind` values in scripts: `mouse`, `touch`, `pen`.

## Run + share artifacts (small-by-default)

- Run one script:
  - `fretboard diag run <script.json|script_id> --launch -- <cmd>`
- Deep debugging (opt-in raw `bundle.json` for tool-launched runs):
  - `fretboard diag run <script.json|script_id> --launch-write-bundle-json --launch -- <cmd>`
  - Notes:
    - `--launch-write-bundle-json` must appear **before** `--launch`.
    - Not supported for `diag matrix` (too many runs; high risk of output explosion).
- Pack a bounded share artifact (preferred in chat/AI loops):
  - `fretboard diag pack <bundle_dir> --ai-only`
- Optional (compat): pack a schema2-only zip (still includes the bundle artifact, but avoids raw `bundle.json`):
  - `fretboard diag pack <bundle_dir> --include-all --pack-schema2-only`
- Generate an AI packet directory (bounded, index-rich):
  - `fretboard diag ai-packet <bundle_dir|bundle.json|bundle.schema2.json> --packet-out <dir>`
  - If you only have sidecars: add `--sidecars-only`.
- Validate a run directory quickly:
  - `fretboard diag artifact lint <run_dir|manifest.json|script.result.json>`

## Triage without grepping bundle.json

Prefer bounded queries over `rg bundle.json`:

- `fretboard diag meta <bundle_dir|bundle.json|bundle.schema2.json> --json`
- `fretboard diag query test-id <bundle_dir|bundle.json|bundle.schema2.json> <pattern> --top 50`
- `fretboard diag slice <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id>`

When searching the repository (not bundle artifacts), prefer `tools/rg-safe.ps1` (excludes `target/fret-diag/**`, `.fret/diag/**`, `bundle.json`, and `bundle.schema2.json`).

Useful safe-search templates (PowerShell):

- Find all mentions of a script id (bounded, avoids diag artifacts):
  - `tools/rg-safe.ps1 -n -- "ui-gallery-command-palette-shortcut-primary"`
- Find who writes a specific env var (limit to tooling + runtime config code):
  - `tools/rg-safe.ps1 -n -- "FRET_DIAG_CONFIG_PATH" crates ecosystem`
- Find `--launch` handling sites (limit to tooling crate):
  - `tools/rg-safe.ps1 -n -- "\\-\\-launch" crates/fret-diag`
- Find an error `reason_code` or warning code:
  - `tools/rg-safe.ps1 -n -- "tooling.launch.failed" crates docs`

For evidence-first triage (reason codes + bounded traces), see: `references/evidence-triage.md`.

## Troubleshooting (common issues)

- “missing diag subcommand / script file not found”
  - Use a promoted `script_id` (recommended) or an explicit path under `tools/diag-scripts/`.
  - Run `diag list scripts` to confirm the id exists.
  - If scripts were recently moved, run `diag doctor scripts` to detect broken redirects / registry drift.
- “no bundles are produced”
  - Likely cause: diagnostics are not enabled or the app isn't wired to the diagnostics driver.
  - Fix: launch via `fretboard diag run ... --launch -- <cmd>` (recommended) or set `FRET_DIAG=1` for manual runs.
- “tooling.launch.failed”
  - Check the `--dir` path is writable; tool-launched runs require writing `<dir>/diag.config.json`.
  - Inspect `<dir>/script.result.json` for a bounded, machine-readable `reason_code` and error note.
- “timeout”
  - Replace sleeps with `wait_until`, `wait_bounds_stable`, and `click_stable`.
  - Add an intermediate `capture_bundle` close to the suspected failure point.
- “artifacts are unexpectedly huge”
  - Quick self-check (launch policy): `fretboard diag config doctor --mode launch --print-launch-policy`
  - Run `fretboard diag config doctor --mode launch` to spot output-explosion risks before rerunning.
  - Check whether you enabled `--launch-write-bundle-json` or `FRET_DIAG_BUNDLE_JSON_FORMAT=pretty`.
  - Note: tool-launched runs scrub inherited `FRET_DIAG_*` env vars from the parent shell to avoid accidental overrides;
    pass explicit `--env FRET_DIAG_...=...` if you truly need to override a runtime knob for one run.
- “screenshot requested but capability missing”
  - Ensure the runner advertises `diag.screenshot_png` and enable screenshots (prefer config `screenshots_enabled=true`
    via `FRET_DIAG_CONFIG_PATH`; manual escape hatch: `FRET_DIAG_GPU_SCREENSHOTS=1`).
- “selectors flaky”
  - Add/repair `test_id` in the component/recipe layer; run `diag lint` for duplicates/missing ids.

## Performance gates (when the issue is a hitch)

Use `diag perf` + worst-bundle evidence, then inspect the worst frames:

- `fretboard diag stats <bundle.json> --sort time --top 30`

See: `references/perf-handoff.md`.

## Definition of done (what to leave behind)

Ship a result that is reviewable and reusable:

- Minimum deliverables (3-pack): Repro (script), Gate (script/test), Evidence (bundle + anchors). See `fret-skills-playbook`.
- A minimal script under `tools/diag-scripts/` (schema v2 for new work) that reproduces the issue deterministically.
- Stable selectors (`test_id`) added/updated so the script survives refactors.
- One portable artifact path to share:
  - native: packed bundle dir (optional screenshots), or
  - web: `.fret/diag/exports/<timestamp>/bundle.json` via `fret-diag-export`.
- If you changed behavior: at least one regression gate (script and/or Rust test) linked from the PR/commit message.
## References (load as needed)

- Evidence-first triage: `references/evidence-triage.md`
- Web/WASM workflow: `references/web-runner.md`
- Perf handoff: `references/perf-handoff.md`
- Conformance playbooks:
  - `references/select-conformance.md`
  - `references/combobox-conformance.md`
  - `references/layout-sweep.md`

Tip: when a run produces unexpectedly large artifacts, use `fretboard diag config doctor --mode launch` to spot
high-risk env overrides before rerunning.

Reason-code first triage:

- `selector.not_found` ⇒ inspect `selector_resolution_trace` (wrong `test_id`, duplicated ids, hidden nodes)
- `routing.*` / “click didn’t land” ⇒ inspect `hit_test_trace` (barrier/capture/occlusion)
- `focus.*` / “type_text_into stalls” ⇒ inspect `focus_trace` + `text_input_snapshot`
- “overlay jumped/flipped/clipped” ⇒ inspect `overlay_placement_trace` (outer/collision/anchor + chosen side + shift delta)
- `timeout` ⇒ prefer adding an intermediate `capture_bundle` and shrinking the script

## Fast query & slices (avoid grepping `bundle.json`)

Use these when you’re trying to quickly find a selector or share a small artifact:

- `fretboard diag meta <bundle_dir|bundle.json|bundle.schema2.json> [--warmup-frames <n>] [--json]` (cached summary)
- `fretboard diag query test-id [<bundle_dir|bundle.json|bundle.schema2.json>] <pattern> [--mode <contains|prefix|glob>] [--top <n>] [--case-sensitive] [--json]` (cached index)
- `fretboard diag slice [<bundle_dir|bundle.json|bundle.schema2.json>] --test-id <test_id> [--frame-id <n>] [--window <id>] [--max-matches <n>] [--max-ancestors <n>] [--json]` (minimal export)

## Component conformance playbooks (reference)

Use invariants-first, evidence-first gates; avoid snapshotting every internal state.

- Select playbook: `references/select-conformance.md`
  - Run: `cargo run -p fretboard -- diag suite ui-gallery-select --launch -- cargo run -p fret-ui-gallery --release`
- Combobox playbook: `references/combobox-conformance.md`
  - Run: `cargo run -p fretboard -- diag suite ui-gallery-combobox --launch -- cargo run -p fret-ui-gallery --release`
- Layout sweep playbook (page-level): `references/layout-sweep.md`
- Web runner transport notes: `references/web-runner.md`

## Maintainer guardrails (keep `--launch` consistent)

When adding or refactoring a diagnostics entrypoint that supports `--launch`:

- Always funnel tool-launched execution through `maybe_launch_demo` so the per-run `<out_dir>/diag.config.json` write +
  `FRET_DIAG_CONFIG_PATH` wiring stays consistent across `run/suite/repro/perf`.
- Plumb `--launch-write-bundle-json` through the same path (never reintroduce a silent fallback to raw `bundle.json`).
- On tooling failures, write a bounded `script.result.json` with a stable `reason_code` (e.g. `tooling.launch.failed`).

## Evidence anchors

Where the code lives:

- Doc: `docs/ui-diagnostics-and-scripted-tests.md`
- In-app exporter + script executor: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- CLI entry + flags: `apps/fretboard/src/cli.rs`, `crates/fret-diag/src/lib.rs`
- Headless exporter (devtools-ws -> `.fret/diag/exports/`): `apps/fret-diag-export`
- Loopback WS hub: `apps/fret-devtools-ws`
- DevTools GUI (optional): `apps/fret-devtools`
- DevTools WS bridge (in-app): `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
- Protocol types (scripts, selectors, results): `crates/fret-diag-protocol`
- Triage/compare engine: `crates/fret-diag`

## Examples

- Example: turn a flaky UI bug into a reproducible gate
  - User says: "This focus bug only happens sometimes—can we script it?"
  - Actions:
    1. Add stable `test_id` targets.
    2. Author a script in `tools/diag-scripts/` using v2 steps (prefer `click_stable`).
    3. Capture a bundle and add a minimal assertion/check.
  - Result: deterministic repro + shareable evidence bundle + CI-friendly gate.

- Example: add a perf regression gate
  - User says: "Scrolling feels janky—make it measurable."
  - Actions: run `diag perf`, keep the worst `evidence_bundle`, gate on an explicit threshold.
  - Result: a number-backed contract with worst-frame evidence.

## Common pitfalls

- Scripts that call `capture_screenshot` without enabling screenshots (`screenshots_enabled=true` in
  `FRET_DIAG_CONFIG_PATH`, or `FRET_DIAG_GPU_SCREENSHOTS=1` in manual runs).
- Targeting pixels/coordinates instead of `test_id`/semantics selectors (scripts become brittle).
- Running the “wrong” binary that isn’t wired through the diagnostics driver (no bundle/script execution).
- Debugging an interaction bug with only geometry snapshots: add scripted steps + focused assertions.
- Enabling raw `bundle.json` writing by accident (avoid `--launch-write-bundle-json` unless you truly need it).
- Web runner:
  - Forgetting `fret_devtools_ws` / `fret_devtools_token` query params (no WS bridge, no scripts/bundles).
  - Assuming the web app can write `target/fret-diag/...` (it cannot; you must export via WS).
  - Running a script that never calls `capture_bundle` (nothing to export).

## Related skills

- `fret-shadcn-source-alignment` (turn Radix/shadcn mismatches into tests + scripts)
- `fret-app-ui-builder` (add stable `test_id` targets and leave gates early)
- `fret-ui-review` (audit layering/focus/command gating pitfalls that often cause diag failures)
