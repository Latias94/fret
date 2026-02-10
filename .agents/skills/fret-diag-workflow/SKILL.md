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
   - Prefer declaring `meta.required_capabilities` for any non-trivial evidence requirements (screenshots, window targeting, etc).
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
- If a script fails, start from `script.result.json` (reason code + evidence) before opening screenshots.

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

Start from these portable artifacts:

- `script.result.json`: outcome + stable `reason_code` + step index + bounded evidence
- `bundle.json`: full frame snapshots (semantics/layout/stats/debug surfaces)
- `triage.json`: compact machine-readable summary derived from a bundle

Common `script.result.json` evidence fields (bounded ring buffers):

- `evidence.selector_resolution_trace`: why a selector matched (or didn’t), with top-N candidates
- `evidence.hit_test_trace`: injected pointer position vs hit chain, including barrier/capture/occlusion hints
- `evidence.focus_trace`: focused element/node identity + barrier/capture hints; includes `text_input_snapshot`
- `evidence.shortcut_routing_trace`: explains whether keydown went to IME/widget path or dispatched a command
- `evidence.overlay_placement_trace`: overlay placement decisions (flip/shift/collision inputs + final rect), when available
- `evidence.ime_event_trace`: IME event kinds + length/cursor summaries (no raw text)
- `evidence.web_ime_trace`: wasm textarea bridge debug summary (ADR 0195; debug-only)

Reason-code first triage:

- `selector.not_found` ⇒ inspect `selector_resolution_trace` (wrong `test_id`, duplicated ids, hidden nodes)
- `routing.*` / “click didn’t land” ⇒ inspect `hit_test_trace` (barrier/capture/occlusion)
- `focus.*` / “type_text_into stalls” ⇒ inspect `focus_trace` + `text_input_snapshot`
- “overlay jumped/flipped/clipped” ⇒ inspect `overlay_placement_trace` (outer/collision/anchor + chosen side + shift delta)
- `timeout` ⇒ prefer adding an intermediate `capture_bundle` and shrinking the script

## Component conformance playbook (example: shadcn `Select`)

The goal is not to snapshot *every* internal state; it’s to define stable, explainable **invariants** and
make failures self-diagnosing via evidence.

Recommended invariants to gate:

- **Open/close lifecycle**:
  - trigger click opens content overlay,
  - outside press / Escape dismisses,
  - close restores focus predictably (when applicable).
- **Routing correctness** (why did the click/key not work?):
  - pointer injection lands on intended target (or produces a hit-test trace explaining barriers/capture),
  - keydown shortcuts do not steal reserved IME/navigation keys while composing.
- **Selection outcome**:
  - selecting an item updates the trigger value,
  - disabled items do not apply selection.
- **Placement sanity** (geometry, not pixels):
  - content bounds stay within the window/viewport,
  - the chosen side/align is explainable under collisions.
- **Virtualization stability** (if list is large):
  - scroll-to-item makes the item exist in semantics,
  - identity is stable (`test_id`/value selectors).

Recommended testing layers:

- Placement/collision matrices ⇒ data-driven fixtures (many cases, thin harness).
- State machine/policy ⇒ Rust tests against the component/policy layer (deterministic time/frames).
- End-to-end routing/focus/IME ⇒ `fretboard diag` scripts with evidence assertions.

Practical authoring tips for scripts:

- Put stable `test_id` on trigger/content/items at the shadcn recipe layer.
- Use `click_stable` for jittery overlays/virtualized targets.
- Prefer semantics selectors (`test_id`, role+name) over coordinates.
- Add one `capture_bundle` near the “interesting” step so failures are explainable without rerunning.

Concrete shadcn `Select` scripts (UI Gallery suite):

- Run: `cargo run -p fretboard -- diag suite ui-gallery-select --launch -- cargo run -p fret-ui-gallery --release`
- Scripts:
  - `tools/diag-scripts/ui-gallery-select-commit-and-label-update-bundle.json` (pointer commit)
  - `tools/diag-scripts/ui-gallery-select-keyboard-commit-apple.json` (ArrowDown + Enter commit)
  - `tools/diag-scripts/ui-gallery-select-typeahead-commit-banana.json` (typeahead + Enter commit)
  - `tools/diag-scripts/ui-gallery-select-disabled-item-no-commit.json` (disabled option does not commit)
  - `tools/diag-scripts/ui-gallery-select-dismiss-outside-press.json` (outside-press dismiss + click-through policy)
  - `tools/diag-scripts/ui-gallery-select-escape-dismiss-focus-restore.json` (Escape dismiss + focus restore)
  - `tools/diag-scripts/ui-gallery-select-wheel-scroll.json` (wheel scroll stability)
  - `tools/diag-scripts/ui-gallery-select-wheel-up-from-bottom.json` (wheel-up from bottom stability)

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
