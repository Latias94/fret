---
name: fret-diag-workflow
description: "Diagnostics workflow for Fret UI: scripted interaction automation (`fretboard diag`), shareable bundles/screenshots, triage/compare, and perf gates (`diag perf`) with worst-bundle attribution. Use to turn flaky UI issues or perf hitches into stable repro gates with evidence."
---

# Fret diagnostics workflow (correctness + perf)

## When to use

Use this skill when:

- A UI bug is hard to reproduce, flaky, or requires “human timing”.
- You need a **shareable artifact** (bundle + optional screenshots) for triage.
- You want to convert a bug into a **CI-friendly gate** (script + assertions).

This skill covers both correctness diagnostics and performance gating/attribution. Use `fret-ui-review` when your goal
is an architecture/UX audit rather than producing repro artifacts.

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

## Smallest starting point (one command)

- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-command-palette-shortcut-primary.json --launch -- cargo run -p fret-ui-gallery --release`

## Quick start

- Run a script and launch the target app (recommended for reproducibility):
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --env FRET_DIAG_SCREENSHOTS=1 --pack --launch -- cargo run -p fret-ui-gallery --release`

- Web runner (WASM): export bundles via DevTools WS (no filesystem access in-browser):
  - Start the loopback WS hub (prints the token): `cargo run -p fret-devtools-ws`
  - Serve the WASM app: `cd apps/fret-ui-gallery-web && trunk serve --port 8080`
  - Open (note the query params): `http://127.0.0.1:8080/?fret_devtools_ws=ws://127.0.0.1:7331/&fret_devtools_token=<token>`
  - Run a script that includes `capture_bundle` and materialize bundles under `.fret/diag/exports/<timestamp>/bundle.json`:
    - `cargo run -p fret-diag-export -- --script tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --token <token>`
  - Optional: run a script over WS via `fretboard` for pass/fail + post-run checks (but note transport limitations):
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --devtools-ws-url ws://127.0.0.1:7331/ --devtools-token <token>`
    - Notes:
      - `--launch/--reuse-launch` is not supported with `--devtools-ws-url` (you run the web app separately).
      - `--pack` is not supported with `--devtools-ws-url` yet; run `fretboard diag pack <bundle_dir|bundle.json>` after.
      - `capture_bundle` steps do not currently auto-materialize a local `bundle.json` in this mode; prefer `fret-diag-export` when you need the bundle artifact.

## Workflow

1. Pick the smallest target that shows the bug.
   - Prefer a UI gallery page or a dedicated demo binary.
2. Create or edit a script in `tools/diag-scripts/`.
   - Use stable `test_id` targets instead of pixel coordinates.
   - Prefer schema v2 for new scripts (more intent-level steps; less flake).
   - Optional: generate scripts from typed Rust templates via `fret-diag-scriptgen` (portable JSON output).
   - Common steps:
     - v1: `click`, `wait_until`, `capture_bundle`, `capture_screenshot`
     - v2: `click_stable`, `ensure_visible`, `scroll_into_view`, `type_text_into`, `press_shortcut`, `menu_select_path`
   - If the target moves/animates during navigation, prefer `click_stable` (schema v2) to avoid “stale click” flake.
     - Key knobs: `stable_frames` and `max_move_px`.
   - Prefer declaring `meta.required_capabilities` for any non-trivial evidence requirements (screenshots, window targeting, etc).
3. Ensure diagnostics are enabled in the running app.
   - If you run via `fretboard diag run ... --launch -- <cmd...>`, the launcher injects `FRET_DIAG=1` for you.
   - Otherwise, set `FRET_DIAG=1` in the target environment.
   - If the script uses `capture_screenshot`: also enable `FRET_DIAG_SCREENSHOTS=1`.
   - If you want a best-effort BMP screenshot alongside bundle dumps (manual `diag poke` / auto-dumps): set `FRET_DIAG_SCREENSHOT=1`.
   - While authoring scripts, consider disabling text redaction: `FRET_DIAG_REDACT_TEXT=0`.
   - Full env reference: `docs/ui-diagnostics-and-scripted-tests.md`
4. Run the script via `fretboard` and collect artifacts.
   - Prefer `fretboard diag run ... --launch -- <cmd...>` so env vars are applied consistently.
   - Reserved env note (when using `--launch`): do not pass `--env FRET_DIAG=1` / `--env FRET_DIAG_DIR=...` etc.
     The launcher sets them and treats them as reserved.
   - Web runner note: use DevTools WS transport via `--devtools-ws-url` / `--devtools-token`.
   - If timing flake is the problem, prefer fixed frame delta:
     - CLI: `--fixed-frame-delta-ms 16` (when launching), or
     - env: `FRET_DIAG_FIXED_FRAME_DELTA_MS=16`
5. Turn the repro into a gate (stable assertions first).
   - Prefer geometry/semantics invariants over pixel diffs when possible.
   - If you need pixel diffs, add `capture_screenshot` steps and use `--check-pixels-changed <test_id>`.
6. Package and share.
   - `fretboard diag pack --include-screenshots` (bundle + screenshots)
   - `fretboard diag triage <bundle_dir|bundle.json> --json` (machine-readable summary)
7. Compare before/after runs for regressions.
   - `fretboard diag compare <bundle_a> <bundle_b> --json`

## Definition of done (what to leave behind)

Ship a result that is reviewable and reusable:

- Minimum deliverables (3-pack): Repro (script), Gate (script/test), Evidence (bundle + anchors). See `fret-skills-playbook`.
- A minimal script under `tools/diag-scripts/` (schema v2 for new work) that reproduces the issue deterministically.
- Stable selectors (`test_id`) added/updated so the script survives refactors.
- One portable artifact path to share:
  - native: packed bundle dir (optional screenshots), or
  - web: `.fret/diag/exports/<timestamp>/bundle.json` via `fret-diag-export`.
- If you changed behavior: at least one regression gate (script and/or Rust test) linked from the PR/commit message.

## Performance gates (diag perf)

Use this section when the issue is “it feels janky” (resize/scroll/pointer-move) and you need **numbers**
and **worst-bundle evidence** (not only a correctness repro).

### “Resize jank” fast path (copy/paste)

Run the P0 resize probes (numbers + thresholds):

```bash
python3 tools/perf/diag_resize_probes_gate.py --suite ui-resize-probes --attempts 3
python3 tools/perf/diag_resize_probes_gate.py --suite ui-code-editor-resize-probes --attempts 3
```

Notes:

- For non-default machines/OS, pass `--baseline <baseline.json>` explicitly (defaults may be tuned for a specific host).
- The bash gate (`tools/perf/diag_resize_probes_gate.sh`) is still available, but the Python gate is cross-platform.

If a gate fails (or you want the worst bundles even on PASS), triage the out-dir:

```bash
python3 .agents/skills/fret-diag-workflow/scripts/triage_perf_gate.py <out-dir> --all --app-snapshot
```

Then inspect the worst bundle:

```bash
cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30
```

### VirtualList boundary gate (window shift invariants)

Run the VirtualList window-boundary crossing gate repeatedly (checks + bundles per run):

```bash
python3 tools/perf/diag_vlist_boundary_gate.py --runs 3
```

Notes:

- Default profile is retained (`--retained 1`). For non-retained stress runs: `--retained 0`.
- Each run writes `summary.json` under the out-dir; failures also keep per-run `stdout.log`/`stderr.log`.

### Steady baseline run (when you want a stable “global sanity”)

Run a suite against a baseline file (under `docs/workstreams/perf-baselines/`):

```bash
cargo run -p fretboard -- diag perf ui-gallery-steady \
  --dir target/fret-diag-perf/ui-gallery-steady.<tag> \
  --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 15 --json \
  --perf-baseline <baseline.json> \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

### Baseline selection (anti-outlier)

If you need to generate a new baseline, prefer the baseline-select helper:

- `tools/perf/diag_perf_baseline_select.py` (cross-platform)
  - Bash wrapper: `tools/perf/diag_perf_baseline_select.sh`

## Perf attribution (answer “why did the worst frame hitch?”)

1) Identify the failing script/metric:

- `<out-dir>/check.perf_thresholds.json` (or `attempt-N/check.perf_thresholds.json` for gate scripts)

2) Resolve worst bundles quickly:

- `python3 .agents/skills/fret-diag-workflow/scripts/triage_perf_gate.py <out-dir> --all`

3) Attribute the worst frames in a bundle:

- `cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30`

4) If the failing threshold is “max of a specific metric”, locate the exact max frame with `jq`:

```bash
jq -c '
  .windows[0].snapshots
  | map({frame_id, tick_id, ts: .timestamp_unix_ms, solve: .debug.stats.layout_engine_solve_time_us, paint: .debug.stats.paint_time_us})
  | max_by(.solve)
' <bundle.json> | jq .
```

## Tips

- Add `test_id` at the recipe/component layer (usually `ecosystem/fret-ui-shadcn`) so scripts remain stable across layout refactors.
- Keep scripts minimal: one bug, one script, one or two assertions.
- Prefer `tools/diag-scripts/` naming that encodes the scenario (component + behavior + expectation).
- Use suites/repros when you want standardized runs:
  - `cargo run -p fretboard -- diag suite <suite-name>`
  - `cargo run -p fretboard -- diag repro <script.json> ...` (convenience wrapper around `diag run` + checks)
- When a selector target is known to jitter (virtualized lists, animated overlays, resize/relayout), use `click_stable`
  rather than retrying `click` with arbitrary sleeps.
- If `click_stable` flakes, set `FRET_DIAG_DEBUG_CLICK_STABLE=1` to capture additional debug context in traces/bundles.
- If a script fails, start from `script.result.json` (reason code + evidence) before opening screenshots.

## Minimal script template (schema v2)

Use schema v2 for new scripts. Start with stable `test_id` selectors and one `capture_bundle`.

Good in-tree examples to copy from:

- `tools/diag-scripts/ui-gallery-command-palette-shortcut-primary.json` (press_shortcut + exists assertions)
- `tools/diag-scripts/ui-gallery-ai-code-block-demo-copy.json` (click_stable on jittery targets)
- `tools/diag-scripts/ui-gallery-dropdown-submenu-safe-corridor-sweep.json` (move_pointer_sweep hover corridor)
- `tools/diag-scripts/ui-gallery-material3-select-rich-options-screenshots.json` (ensure_visible + screenshot)

```json
{
  "schema_version": 2,
  "meta": {
    "name": "my-scenario",
    "required_capabilities": ["diag.script_v2"]
  },
  "steps": [
    {
      "type": "wait_until",
      "predicate": { "kind": "exists", "target": { "kind": "test_id", "id": "my-trigger" } },
      "timeout_frames": 240
    },
    {
      "type": "click_stable",
      "target": { "kind": "test_id", "id": "my-trigger" },
      "stable_frames": 6,
      "max_move_px": 0.5,
      "timeout_frames": 240
    },
    {
      "type": "type_text_into",
      "target": { "kind": "test_id", "id": "my-input" },
      "text": "hello",
      "timeout_frames": 240
    },
    { "type": "capture_bundle", "label": "after" }
  ]
}
```

If you add `capture_screenshot`, require screenshot capability and enable screenshots:

- Script: add `diag.screenshot_png` to `meta.required_capabilities`
- Native/filesystem: set `FRET_DIAG_SCREENSHOTS=1`

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
- `evidence.web_ime_trace`: wasm textarea bridge debug summary (ADR 0165; debug-only)

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
  - `tools/diag-scripts/ui-gallery-select-roving-skips-disabled-orange.json` (assert roving skips a disabled option via `active_item_is`)
  - `tools/diag-scripts/ui-gallery-select-dismiss-outside-press.json` (outside-press dismiss + click-through policy)
  - `tools/diag-scripts/ui-gallery-select-escape-dismiss-focus-restore.json` (Escape dismiss + focus restore)
  - `tools/diag-scripts/ui-gallery-select-wheel-scroll.json` (wheel scroll stability)
  - `tools/diag-scripts/ui-gallery-select-wheel-up-from-bottom.json` (wheel-up from bottom stability)

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
- `fret-app-ui-builder` (add stable `test_id` targets and leave gates early)
- `fret-ui-review` (audit layering/focus/command gating pitfalls that often cause diag failures)
