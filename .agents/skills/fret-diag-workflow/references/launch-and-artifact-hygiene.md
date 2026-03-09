# Launch and artifact hygiene

Use this note when the main problem is **how to run diagnostics safely and keep artifacts bounded**.

## 1) Bounded-output rules (non-negotiable)

- Do **not** run `rg` on `bundle.json` or on `target/fret-diag/**` / `.fret/diag/**`.
- Do **not** `cat` / `Get-Content` raw `bundle.json` files.
- Prefer bounded tooling queries:
  - `fretboard diag meta ...`
  - `fretboard diag windows ...`
  - `fretboard diag dock-graph ...`
  - `fretboard diag dock-routing ...`
  - `fretboard diag screenshots ...`
  - `fretboard diag resolve latest ...`
  - `fretboard diag query ...`
  - `fretboard diag slice ...`
- When searching the repo, use `tools/rg-safe.ps1` to exclude diag artifact directories and bundle artifacts.

## 2) Launch/session hygiene

- Prefer `--launch` for deterministic runs.
- Avoid relying on inherited parent-shell `FRET_DIAG_*` variables for tool-launched runs.
- Treat `--dir` (`FRET_DIAG_DIR`) as a **session boundary**.
  - never share the same out dir between multiple concurrent runs
  - prefer an explicit unique `--dir` per agent/task
- For tool-launched runs, prefer `--session-auto` so tooling isolates each run under the base dir.
- Avoid relying on a global `latest.txt` outside a session; prefer per-run `manifest.json` + `script.result.json` and session listing commands.

Useful commands:

- `fretboard diag list sessions --dir <base_dir>`
- `fretboard diag sessions clean --dir <base_dir> --keep 50`
- `fretboard diag resolve latest --dir <base_or_session_dir>`

## 3) Artifact size hygiene

Keep artifacts small by default:

- capture only a few bundles at key points
- prefer sidecars + `bundle.schema2.json` over raw `bundle.json`
- avoid `FRET_DIAG_BUNDLE_JSON_FORMAT=pretty` unless truly needed
- keep `script_auto_dump` **off** for suites
  - tool-launched `--launch` runs write `script_auto_dump=false` by default
  - authoring escape hatch: `--env FRET_DIAG_SCRIPT_AUTO_DUMP=1`
- use raw `bundle.json` only as an explicit escape hatch:
  - `--launch-write-bundle-json`
  - never for `diag matrix`

## 4) Screenshot and gate hygiene

- If you use `--check-pixels-changed <test_id>`, the run must capture screenshots.
- Tool-launched `--launch` runs enable screenshots automatically when this gate is requested, but the script still needs explicit `capture_screenshot` steps.
- Motion-sensitive work should run with a fixed timestep when determinism matters.

## 5) Input isolation and multi-window notes

- Tool-launched runs ignore external pointer + keyboard events by default while a script is active.
- Multi-window docking scripts may use runner cursor overrides (`set_cursor_in_window_logical`) to drive hover routing; this updates the runner’s internal cursor model and does **not** warp the OS cursor.
- Cross-window drops: express the final release in the **target window** coordinate space before `pointer_up`.

## 6) Small-by-default run/share flow

Recommended flow:

1. `cargo run -p fretboard -- diag config doctor --mode launch --print-launch-policy`
2. `cargo run -p fretboard -- diag run <script.json|script_id> --pack --ai-packet --launch -- <cmd>`
3. `cargo run -p fretboard -- diag meta <bundle_dir|bundle.schema2.json> --json`
4. `cargo run -p fretboard -- diag pack <bundle_dir> --ai-only`
5. add `--launch-write-bundle-json` only when raw `bundle.json` is truly needed

## 7) Related references

- Web/WASM transport: `web-runner.md`
- Evidence-first triage: `evidence-triage.md`
- Perf handoff: `perf-handoff.md`
