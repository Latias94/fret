---
name: fret-diag-workflow
description: "This skill should be used when the user asks to \"reproduce a UI bug\", \"write a diag script\", \"capture a diagnostics bundle\", \"triage/compare bundles\", or \"add a perf gate\". Provides the `fretboard diag` workflow for scripted interactions, shareable bundles, comparisons, and perf attribution with stable evidence."
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
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --env FRET_DIAG_GPU_SCREENSHOTS=1 --pack --launch -- cargo run -p fret-ui-gallery --release`

- Fast “don’t grep bundle.json” triage helpers (native or web-exported bundles):
  - Summary sidecar (cached): `cargo run -p fretboard -- diag meta <bundle_dir|bundle.json> --json`
  - Find the right `test_id` quickly (cached index): `cargo run -p fretboard -- diag query test-id [<bundle_dir|bundle.json>] <pattern> --mode contains --top 50`
  - Export a minimal shareable slice for one target (instead of copying bundle fragments): `cargo run -p fretboard -- diag slice [<bundle_dir|bundle.json>] --test-id <test_id>`
  - Python fallback (bounded output; useful when you only have a `bundle.json` and cannot run `fretboard`):
    - PowerShell:
      ```powershell
      @'
      import json,sys
      from collections import deque
      p, needle = sys.argv[1], sys.argv[2]
      data = json.load(open(p, encoding="utf-8"))
      out = []
      st = deque([("", data)])
      while st and len(out) < 50:
          path, v = st.pop()
          if isinstance(v, dict):
              for k, vv in v.items():
                  st.append(((path + "." + k) if path else k, vv))
          elif isinstance(v, list):
              for i, vv in enumerate(v):
                  st.append((f"{path}[{i}]", vv))
          else:
              s = str(v)
              if needle in s:
                  out.append((path, s[:160]))
      for a, b in out:
          print(f"{a} => {b}")
      print("matches", len(out))
      '@ | python - <bundle.json> <needle>
      ```

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
   - If the script uses `capture_screenshot`: also enable GPU screenshots (`FRET_DIAG_GPU_SCREENSHOTS=1`, alias: `FRET_DIAG_SCREENSHOTS=1`).
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
   - Note: packed bundles include cached sidecars under `_root/` when available (`bundle.meta.json`, `test_ids.index.json`, `test_ids.json`).
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

Use this section when the issue is “it feels janky” (resize/scroll/pointer-move) and you need numbers + **worst-bundle evidence**
for fast attribution.

Fast paths (copy/paste):

- Resize probes: `python3 tools/perf/diag_resize_probes_gate.py --suite ui-resize-probes --attempts 3`
- VirtualList boundary: `python3 tools/perf/diag_vlist_boundary_gate.py --runs 3`
- Triage an out-dir: `python3 .agents/skills/fret-diag-workflow/scripts/triage_perf_gate.py <out-dir> --all --app-snapshot`
- Inspect worst frames: `cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30`
  - Note: `diag stats` defaults to a brief human report; use `--verbose` for the detailed breakdown or `--json` for machine output.

See: `references/perf-handoff.md`.

## Tips

- Add `test_id` at the recipe/component layer (usually `ecosystem/fret-ui-shadcn`) so scripts remain stable across layout refactors.
- Keep scripts minimal: one bug, one script, one or two assertions.
- Prefer `diag meta` / `diag query test-id` / `diag slice` over grepping `bundle.json` (bounded output, cached, shareable).
- When searching the repo, prefer `tools/rg-safe.ps1` over raw `rg` to avoid scanning `bundle.json` artifacts under `target/fret-diag/`.
- Prefer `tools/diag-scripts/` naming that encodes the scenario (component + behavior + expectation).
- Use suites/repros when you want standardized runs:
  - `cargo run -p fretboard -- diag suite <suite-name>`
  - `cargo run -p fretboard -- diag repro <script.json> ...` (convenience wrapper around `diag run` + checks)
- When a selector target is known to jitter (virtualized lists, animated overlays, resize/relayout), use `click_stable`
  rather than retrying `click` with arbitrary sleeps.
- If `click_stable` flakes, set `FRET_DIAG_DEBUG_CLICK_STABLE=1` to capture additional debug context in traces/bundles.
- If a script fails, start from `script.result.json` (reason code + evidence) before opening screenshots.

## Minimal script template (schema v2)

Use schema v2 for new scripts. Start with stable `test_id` selectors and one `capture_bundle`:

- `wait_until` (target exists) → `click`/`click_stable` → `capture_bundle`
- Prefer intent-level stabilization (`click_stable`, `wait_bounds_stable`) over ad-hoc sleeps.

Good in-tree examples to copy from:

- `tools/diag-scripts/ui-gallery-command-palette-shortcut-primary.json` (press_shortcut + exists assertions)
- `tools/diag-scripts/ui-gallery-ai-code-block-demo-copy.json` (click_stable on jittery targets)
- `tools/diag-scripts/ui-gallery-dropdown-submenu-safe-corridor-sweep.json` (move_pointer_sweep hover corridor)
- `tools/diag-scripts/ui-gallery-material3-select-rich-options-screenshots.json` (ensure_visible + screenshot)

If you add `capture_screenshot`, require screenshot capability and enable screenshots:

- Script: add `diag.screenshot_png` to `meta.required_capabilities`
- Native/filesystem: set `FRET_DIAG_GPU_SCREENSHOTS=1` (alias: `FRET_DIAG_SCREENSHOTS=1`)

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
- `bundle.meta.json`: cached bundle summary (windows, snapshots, last-considered frame ids, `test_id` counts)
- `test_ids.index.json`: cached `test_id` index used by `diag query test-id` (fast, bounded query output)
- `slice.<test_id>.json`: minimal extracted slice for a single target (`diag slice`)

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

## Fast query & slices (avoid grepping `bundle.json`)

Use these when you’re trying to quickly find a selector or share a small artifact:

- `fretboard diag meta <bundle_dir|bundle.json> [--warmup-frames <n>] [--json]` (cached summary)
- `fretboard diag query test-id [<bundle_dir|bundle.json>] <pattern> [--mode <contains|prefix|glob>] [--top <n>] [--case-sensitive] [--json]` (cached index)
- `fretboard diag slice [<bundle_dir|bundle.json>] --test-id <test_id> [--frame-id <n>] [--window <id>] [--max-matches <n>] [--max-ancestors <n>] [--json]` (minimal export)

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

- Scripts that call `capture_screenshot` without `FRET_DIAG_GPU_SCREENSHOTS=1` (or the legacy alias).
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
