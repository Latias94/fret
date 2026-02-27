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
   - Gestures: `diag.gesture_tap`, `diag.gesture_pinch`

Tip: if the user says “it only happens with touch/pen”, use `pointer_kind` on pointer-driven steps (capability-gated).

## Run + share artifacts (small-by-default)

- Run one script:
  - `fretboard diag run <script.json|script_id> --launch -- <cmd>`
- Pack a bounded share artifact (preferred in chat/AI loops):
  - `fretboard diag pack <bundle_dir> --ai-only`
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

When searching the repository (not bundle artifacts), prefer `tools/rg-safe.ps1` (excludes `target/fret-diag/**` and `.fret/diag/**`).

For evidence-first triage (reason codes + bounded traces), see: `references/evidence-triage.md`.

## Troubleshooting (common issues)

- “missing diag subcommand / script file not found”
  - Use a promoted `script_id` (recommended) or an explicit path under `tools/diag-scripts/`.
  - Run `diag list scripts` to confirm the id exists.
- “timeout”
  - Replace sleeps with `wait_until`, `wait_bounds_stable`, and `click_stable`.
  - Add an intermediate `capture_bundle` close to the suspected failure point.
- “screenshot requested but capability missing”
  - Ensure the runner advertises `diag.screenshot_png` and enable screenshots (`FRET_DIAG_GPU_SCREENSHOTS=1`).
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

- Scripts that call `capture_screenshot` without `FRET_DIAG_GPU_SCREENSHOTS=1`.
- Targeting pixels/coordinates instead of `test_id`/semantics selectors (scripts become brittle).
- Running the “wrong” binary that isn’t wired through the diagnostics driver (no bundle/script execution).
- Debugging an interaction bug with only geometry snapshots: add scripted steps + focused assertions.
- Web runner:
  - Forgetting `fret_devtools_ws` / `fret_devtools_token` query params (no WS bridge, no scripts/bundles).
  - Assuming the web app can write `target/fret-diag/...` (it cannot; you must export via WS).
  - Running a script that never calls `capture_bundle` (nothing to export).

## Troubleshooting

- Symptom: no bundles are produced.
  - Likely cause: diagnostics are not enabled.
  - Fix: launch via `fretboard diag run ... --launch -- <cmd>` or set `FRET_DIAG=1`.
- Symptom: selectors are flaky (misses, clicks wrong node).
  - Fix: prefer `test_id` + v2 intent steps (`click_stable`, `ensure_visible`) over coordinates.

## Related skills

- `fret-shadcn-source-alignment` (turn Radix/shadcn mismatches into tests + scripts)
- `fret-app-ui-builder` (add stable `test_id` targets and leave gates early)
- `fret-ui-review` (audit layering/focus/command gating pitfalls that often cause diag failures)
