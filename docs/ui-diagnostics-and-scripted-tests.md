---
title: UI Diagnostics Bundles & Scripted Interaction Tests
status: living
scope: debugging, AI triage, scripted repros
---

# UI Diagnostics Bundles & Scripted Interaction Tests

This doc describes the current **diagnostics bundle** workflow and the **MVP scripted interaction harness**
implemented for Fret apps that run through `fret-bootstrap`'s `UiAppDriver`.

Scope note:

- For the canonical **first-open diagnostics workflow**, start with:
  `docs/diagnostics-first-open.md`.
- This file focuses on **bundles + scripts** (how to dump `bundle.json`, how the script harness is triggered, and
  how to author stable, selector-driven repros).
- For the canonical artifact/evidence taxonomy (source-of-truth vs derived vs optional evidence, the bounded
  first-open set, and consumer expectations), see:
  `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`.
- For the **interactive inspect workflow** (hover/pick overlay, shortcuts, and selector copy UX), see:
  `docs/debugging-ui-with-inspector-and-scripts.md`.
- For the planned **DevTools GUI** that wraps these same contracts (inspect/pick/scripts/bundles) with a user-facing UI,
  see: `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`.

The goal is GPUI/Zed-style "inspectable, shareable repro units":

- capture a portable bundle artifact (`bundle.json` or `bundle.schema2.json`) that can be sent to another developer (or an AI tool),
- select targets by **semantics** (ADR 0033) rather than paint output,
- run deterministic scripted repros without adding ad-hoc debug UI.

Related ADRs:

- ADR 0159: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- ADR 0033 (Semantics/a11y): `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Roadmap/TODO: `docs/workstreams/ui-diagnostics-inspector-v1/ui-diagnostics-inspector-todo.md`

Implementation pointers (where the code lives today):

- In-app exporter + script executor: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` and
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- Script/selector/result types (serde): `crates/fret-diag-protocol`
- CLI tooling engine (pack/stats/gates/compare): `crates/fret-diag` (wrapped by `apps/fretboard/src/diag.rs`)

## Quick Start (manual bundle dump)

1. Run any demo/app wired via `UiAppDriver` and enable diagnostics:

   - `FRET_DIAG=1`

2. Reproduce the issue.

3. Trigger a dump:

   - `cargo run -p fretboard-dev -- diag poke`

4. Locate the most recent bundle directory:

   - `cargo run -p fretboard-dev -- diag latest` (session-aware when `<dir>/sessions/*` exists)
   - If you are using sessions (`--session-auto`) or you have a base dir with multiple session dirs:
     - `cargo run -p fretboard-dev -- diag resolve latest --dir <base_or_session_dir>`
     - Optional: `--within-session <id|latest>` to pin a specific session under `<base_dir>/sessions/`.
   - The primary bundle artifact is `bundle.schema2.json` (preferred) or `bundle.json` under that directory.

By default bundles go under `target/fret-diag/<timestamp>/` and `target/fret-diag/latest.txt` is updated.

Concurrency note (important for automation / AI agents):

- The filesystem transport uses shared control-plane files under the out dir (`script.json`, `script.touch`,
  `script.result.json`, `trigger.touch`, `latest.txt`, etc). **Do not** point multiple concurrent runs (multiple
  terminals, multiple agents, or multiple demos) at the same `FRET_DIAG_DIR`.
- Recommendation: treat `--dir` as a session boundary and always pass a unique out dir per agent/task:
  - `cargo run -p fretboard-dev -- diag run <script> --dir target/fret-diag-agent-a --launch -- <cmd...>`
  - `cargo run -p fretboard-dev -- diag suite <suite> --dir target/fret-diag-issue-1234 --launch -- <cmd...>`
- If you are using `--launch`, prefer `--session-auto` so tooling allocates an isolated session dir automatically:
  - `cargo run -p fretboard-dev -- diag run <script> --dir target/fret-diag-agent-a --session-auto --launch -- <cmd...>`
  - `cargo run -p fretboard-dev -- diag suite <suite> --dir target/fret-diag-agent-a --session-auto --launch -- <cmd...>`
- Tip: most bounded inspection commands accept a base/session out dir directly (not just bundle dirs) and will resolve it
  to the latest bundle under the latest session automatically. When in doubt, use `diag resolve latest` to see what would
  be selected.

## Bundle schema (v2) and semantics mode

The runtime exports **schema v2** bundles (semantics tables + per-snapshot fingerprints).

Artifact note:

- `bundle.json` is the large “raw” view (optional; can be disabled by tooling via config).
- `bundle.schema2.json` is the compact schema2 view (recommended; preferred by tooling when present).

Legacy note:

- Older bundles may still be schema v1 (inline-only semantics, no tables).
- Tooling remains compatible: `fretboard-dev diag meta/query/slice/stats/compare` resolve semantics from either inline
  semantics or the schema-v2 semantics table.
- If you want to “upgrade” a schema v1 bundle (or write a compact schema2 view for a large schema v2 bundle), run
  `cargo run -p fretboard-dev -- diag bundle-v2 <bundle_dir> --mode <all|changed|last|off>` (writes
  `bundle.schema2.json`, and directory-based tooling will prefer it when present).

Defaults:

- Manual dumps default to semantics mode `changed`.
- Script dumps default to semantics mode `last` (keep bundles smaller while still preserving at least one full semantics snapshot).

Overrides (both manual and script dumps):

- `FRET_DIAG_BUNDLE_SEMANTICS_MODE=all|changed|last|off`

Environment fingerprint note:

- `bundle.json.env.monitor_topology` is the host monitor inventory exported by the runner when
  that environment source is available.
- `bundle.json.env.scale_factors_seen` remains the last-known per-window scale factors observed
  during the run.
- Do not treat `scale_factors_seen` as host monitor topology or as a mixed-DPI preflight signal.
- On native diagnostics runs, the runtime may also publish `environment.sources.json` at the run
  `out_dir` root.
- `host.monitor_topology` currently maps to `environment.source.host.monitor_topology.json` when
  that launch-time source is available.
- DevTools WS sessions that advertise `devtools.environment_sources` may also answer
  `environment.sources.get` / `environment.sources.get_ack` with a session-published source
  catalog.
- Campaign result aggregates may report `environment_sources_path`,
  `environment_source_catalog_provenance`, and `environment_sources`.
- Campaign manifests still only gate on `requires_capabilities`.
- If diagnostics later promotes host-environment sources into orchestration, use a separate
  `environment.sources.json` or session-published source catalog rather than `capabilities.json`.
- Do not scrape `debug.environment` or other debug-only snapshot lanes as a substitute for a real
  host-environment preflight contract.

## Sidecars (index/meta/test-id index)

On native filesystem dumps, the runtime also writes bounded sidecars next to the bundle artifact:

- `bundle.meta.json`: compact per-window counts (snapshots, semantics inline vs table, test-id totals for a considered snapshot).
- `bundle.index.json`: per-window/per-snapshot index (frame ids, timestamps, semantics presence/source, and a bounded `test_id` bloom).
- `test_ids.index.json`: a per-window `test_id` index (items + duplicate hints) derived from the last snapshot with resolved semantics.
- `script.result.json` (script dumps only): the most recent script result snapshot (stage/reason + bounded evidence traces).

These sidecars are intended to speed up CLI queries and AI triage without opening or grepping a full `bundle.json`.

## Termination semantics (tool-launched runs)

When using `--launch`, tooling requests the demo to exit by touching `exit.touch` in the session out dir.

Tooling waits for a graceful exit (best effort). If the demo does not exit within a grace window, tooling will
force-kill the process to avoid leaving orphaned demos behind.

Tooling writes `resource.footprint.json` (in the same out dir) with a `killed: bool` field so you can distinguish:

- the script **finished** (`script.result.json: stage=passed`) but the demo still needed to be killed (`killed=true`), vs
- the script **never finished** (`script.result.json: stage=running` or missing) and the tool likely timed out or aborted.

If `killed=true`, treat it as a diagnostics/runtime issue (e.g. exit trigger not observed, deadlock, no-frame stall) and
prefer capturing a bounded failure bundle plus a stable `reason_code` so it can be triaged deterministically.

Tooling note:

- Launch-mode script runs treat `killed=true` as a failure and will report `reason_code=tooling.demo_exit.killed` (even
  if the script itself reported `stage=passed`).

## No-frame liveness (script keepalive)

Some script steps historically only progressed when a window produced redraw callbacks. In multi-window scenarios
(especially docking tear-off + overlap + z-order churn), a relevant window can become fully occluded or idle/throttled
and stop producing redraw callbacks. If your script tail still depends on additional frames (e.g. trailing
`wait_frames`), this can leave `script.result.json` stuck at `stage=running` and push the failure into tooling timeouts.

Launch-mode tooling mitigations (recommended):

- Tool-launched `--launch` runs write `script_keepalive=true` into the per-run `diag.config.json`.
- When enabled, the in-app diagnostics runtime arms a repeating timer while scripts are pending/active. On each tick it
  polls triggers and can advance a conservative subset of steps even if no window is producing redraw callbacks.
- If the next step cannot safely progress without frames for a bounded wall time, the script fails with
  `reason_code=timeout.no_frames` (instead of hanging).

Manual runs (escape hatch):

- Set `FRET_DIAG_SCRIPT_KEEPALIVE=1` (or set `script_keepalive=true` in the config file) when authoring/triaging
  occlusion-heavy scripts that might otherwise hang.

Deterministic repro hook (debug-only):

- Set `FRET_DIAG_SIMULATE_NO_FRAMES=1` to skip snapshot recording and frame-driven script advancement, forcing the
  keepalive/no-frame path to either make progress or fail with `reason_code=timeout.no_frames`.
  - Recommended regression script: `tools/diag-scripts/diag/no-frame/diag-no-frame-timeout-no-frames.json`

Important limitation:

- Keepalive ticks are intentionally conservative: pointer-dispatch steps and semantics-dependent selector resolution
   still require real UI frames/snapshots. Keepalive is a liveness safety net, not a “renderless UI simulator”.

Footgun / recommendation:

- Avoid running `rg`/`grep` directly on `bundle.json` dumps (they can be huge and can easily explode your terminal output).
  - Prefer bounded tooling commands that use sidecars and/or schema2 views:
  - `fretboard-dev diag meta <bundle_dir|bundle.json|bundle.schema2.json> --json`
  - `fretboard-dev diag windows <bundle_dir|bundle.json|bundle.schema2.json> --warmup-frames <n> --json`
  - `fretboard-dev diag dock-graph <bundle_dir|bundle.json|bundle.schema2.json>`
  - `fretboard-dev diag dock-routing <bundle_dir|bundle.json|bundle.schema2.json>`
    - Note: when an adjacent bundle artifact is available, `dock-routing` may regenerate/overwrite `dock.routing.json`
      to keep bounded evidence keys up to date (no manual deletion needed).
    - The report is intentionally compact; it is typically enough to debug multi-window docking routing issues without
      opening `bundle.json`. Look for:
      - `src/cur` (source/current window ids),
      - `pos/start/grab/follow` (window-local cursor position + ImGui-style cursor grab anchor),
      - `scr/scr_used/origin` (screen cursor + client origin evidence for coordinate-space bugs),
      - `sf_cur/sf_move` (scale factor evidence for mixed-DPI follow drags),
      - `scale_factors_seen` / `mixed_dpi_signal_observed` (top-level mixed-DPI signal summary from drag evidence),
      - `under` (hover selection source: platform vs heuristic).
    - For the docking mixed-DPI outer-position sweep runbook, prefer
      `python3 tools/diag_pick_docking_mixed_dpi_acceptance_pair.py <diag_out_dir_or_session_dir>`
      to reuse `diag dock-routing --json` across the three canonical bundle labels and emit one bounded
      acceptance summary (`--json-out <path>`). Add `--note-out <path>` when you also want one
      Markdown evidence-note draft for the workstream lane.
  - `fretboard-dev diag query test-id <bundle_dir|bundle.json|bundle.schema2.json> <pattern> --top 50`
  - `fretboard-dev diag query snapshots <bundle_dir|bundle.index.json|bundle.schema2.json> --test-id <test_id> --top 10`
  - `fretboard-dev diag slice <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id>`
  - `fretboard-dev diag slice <bundle_dir|bundle.json|bundle.schema2.json> --step-index <n> --test-id <test_id> --warmup-frames <n>`
  - `fretboard-dev diag ai-packet <bundle_dir|bundle.json|bundle.schema2.json> --packet-out <dir>`
- When searching the repository (not bundle artifacts), prefer `tools/rg-safe.ps1` (it excludes `target/fret-diag/**` and `.fret/diag/**` by default).

To disable sidecar writing (native-only):

- `FRET_DIAG_BUNDLE_WRITE_INDEX=0`

Notes:

- `bundle.index.json` includes `test_id_bloom_hex` only for a bounded tail window (currently the last 64 snapshots per window) so that
  `diag query snapshots --test-id <id>` can quickly prioritize likely-matching snapshots without scanning `bundle.json`.
- `bundle.index.json` also includes bounded `semantics_blooms` keyed by `(window, semantics_fingerprint, semantics_source)` so that
  `--test-id` queries can still filter/sort older snapshots that do not carry a per-snapshot `test_id_bloom_hex`.
- When `script.result.json` is present in the bundle directory, `bundle.index.json` also includes an additive `script.steps` section that
  maps script `step_index` values to the nearest snapshot selector (resolved by `frame_id` first, then by timestamp when needed).

## Recommended env presets (AI loops)

When you expect large bundles (or want fast AI/CLI iteration), keep bundles small and indexes rich:

Minimal “AI triage” preset (portable defaults; copy/paste in PowerShell):

```powershell
$env:FRET_DIAG=1
$env:FRET_DIAG_BUNDLE_SEMANTICS_MODE="last"
$env:FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS=30
$env:FRET_DIAG_BUNDLE_JSON_FORMAT="compact"
```

More aggressive “small bundles” preset (trade off semantics richness; best for test-id driven workflows):

```powershell
$env:FRET_DIAG=1
$env:FRET_DIAG_BUNDLE_SEMANTICS_MODE="changed"
$env:FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=1
$env:FRET_DIAG_MAX_SEMANTICS_NODES=20000
$env:FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS=20
```

Authoring scripts / selectors preset (reduce friction while iterating):

```powershell
$env:FRET_DIAG=1
$env:FRET_DIAG_REDACT_TEXT=0
```

## AI-first triage recipe (avoid sharing full bundle artifacts)

When a bundle artifact is too large to share or inspect directly, prefer a bounded artifact set:

1. Get quick context:
   - `cargo run -p fretboard-dev -- diag meta <bundle_dir|bundle.json|bundle.schema2.json> --meta-report`
2. Find a stable selector (usually `test_id`):
   - `cargo run -p fretboard-dev -- diag query test-id <bundle_dir|bundle.json|bundle.schema2.json> <pattern> --top 50`
   - Optional: find the best snapshot quickly:
     - `cargo run -p fretboard-dev -- diag query snapshots <bundle_dir|bundle.index.json|bundle.schema2.json> --test-id <test_id> --top 10`
     - `cargo run -p fretboard-dev -- diag slice <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id>`
     - If you already know the failing script step: `cargo run -p fretboard-dev -- diag slice <bundle_dir|bundle.json|bundle.schema2.json> --step-index <n> --test-id <test_id> --warmup-frames <n>`
3. Export a compact packet for AI / code review:
   - `cargo run -p fretboard-dev -- diag ai-packet <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id> --packet-out <dir>`

## Optional: dump a frame screenshot alongside the bundle

If you suspect a **rendering** regression (e.g. semantics + layout look correct but pixels look blank),
enable bundle screenshots:

- `FRET_DIAG_BUNDLE_SCREENSHOT=1`

When a bundle is dumped, the runner writes `frame.bmp` into the bundle directory (same folder as
the bundle artifact).

Notes:

- This is **bundle-scoped** and **dump-triggered**:
  - The runtime writes a `screenshot.request` file into the bundle directory when dumping a bundle export.
  - The desktop runner detects that request and writes `frame.bmp` (and `screenshot.done`) as best-effort.
- This is intentionally separate from the on-demand PNG screenshot protocol used by scripted steps
  like `capture_screenshot` (see below).

## Offline bundle viewer (optional)

This repo includes an offline web viewer for bundle artifacts (`bundle.json` / `bundle.schema2.json`) at `tools/fret-bundle-viewer`.

```powershell
$env:HTTP_PROXY='http://127.0.0.1:10809'
$env:HTTPS_PROXY='http://127.0.0.1:10809'

pnpm -C tools/fret-bundle-viewer install
pnpm -C tools/fret-bundle-viewer dev
```

Workflow tip:

- Drag `bundle.json` (or `bundle.schema2.json`) from `target/fret-diag/.../` into the viewer (or use the file picker).
- You can also open a `.zip` that contains `bundle.json` or `bundle.schema2.json` anywhere inside it (handy for sharing a full repro directory).
- To generate a shareable `.zip` for the latest bundle: `cargo run -p fretboard-dev -- diag pack`
- To include nearby artifacts (`script.json`, `script.result.json`, `pick.result.json`), `triage.json`, and screenshots (when present): `cargo run -p fretboard-dev -- diag pack --include-all`
- To validate a per-run artifact directory (manifest + chunks + sidecars + run_id/timestamps): `cargo run -p fretboard-dev -- diag artifact lint <run_dir|out_dir>`
- Prefer viewer-friendly zips when schema2 exists (keeps artifacts smaller than raw `bundle.json`):
  - `cargo run -p fretboard-dev -- diag pack --include-all --pack-schema2-only --warmup-frames <n>`
  - If needed: `cargo run -p fretboard-dev -- diag doctor --fix-schema2 <bundle_dir> --warmup-frames <n>`
- For AI-first sharing, prefer `diag pack --ai-only` (see “AI-first sharing” below).
- The bundle viewer surfaces these zip artifacts (and lets you copy/download them) when they are present under `_root/`.
- To generate a machine-readable `triage.json` next to a bundle: `cargo run -p fretboard-dev -- diag triage <bundle_dir|bundle.json|bundle.schema2.json>`
- To generate a smaller frames-index-backed triage summary: `cargo run -p fretboard-dev -- diag triage <bundle_dir|bundle.json|bundle.schema2.json> --lite --metric <total|layout|paint>`
- To generate a frames-index-backed hotspot summary without loading the full bundle JSON: `cargo run -p fretboard-dev -- diag hotspots <bundle_dir|bundle.json|bundle.schema2.json> --lite --metric <total|layout|paint> --warmup-frames <n> --json`
- To estimate large JSON subtrees for retention/packaging triage: `cargo run -p fretboard-dev -- diag hotspots <bundle_dir|bundle.json|bundle.schema2.json> --hotspots-top <n> --max-depth <n> --min-bytes <n> [--force] --json`
- To generate (or refresh) a cached bundle metadata sidecar (`bundle.meta.json`): `cargo run -p fretboard-dev -- diag meta <bundle_dir|bundle.json|bundle.schema2.json> --json`
- To print a compact human meta report (semantics inline vs table + table size indicators): `cargo run -p fretboard-dev -- diag meta <bundle_dir|bundle.json|bundle.schema2.json> --meta-report`
- To generate (or refresh) a cached `test_ids.index.json` sidecar without opening the viewer: `cargo run -p fretboard-dev -- diag test-ids-index <bundle_dir|bundle.json|bundle.schema2.json> --warmup-frames <n> --json`
- To generate (or refresh) a cached `frames.index.json` sidecar used by lite triage: `cargo run -p fretboard-dev -- diag frames-index <bundle_dir|bundle.json|bundle.schema2.json> --warmup-frames <n> --json`
- To inspect screenshot manifests or aggregated screenshot directories: `cargo run -p fretboard-dev -- diag screenshots <out_dir|bundle_dir|bundle.json|bundle.schema2.json> --json`
- To include `triage.json` in a share zip: `cargo run -p fretboard-dev -- diag pack --include-triage`
- To include screenshots in a share zip: `cargo run -p fretboard-dev -- diag pack --include-screenshots` (packs `target/fret-diag/screenshots/<bundle_timestamp>/` into `_root/screenshots/` when available)
- If you’re sharing via chat, “Paste JSON” is a fast way to load a copied `bundle.json` / `bundle.schema2.json` payload without files.
- Use “Export triage.json” when you want a small, machine-readable artifact for AI triage (selection + bounded debug artifacts).

## AI-first sharing (recommended)

Prefer sharing **bounded artifacts** over the full `bundle.json` (especially in AI loops):

Canonical surface note:

- Use `diag pack --pack-schema2-only`, not the deleted `--schema2-only` alias.
- Use `diag triage --lite` / `--metric`, not the deleted `--frames-index` or `--from-frames-index` aliases.
- Use `diag ai-packet --test-id <test_id>` when targeting a specific selector; the old positional `diag ai-packet <test_id>` form is deleted.
- Use `diag resolve latest` when you need the exact bundle/session resolution that bounded inspection commands will follow.

- Generate an “AI packet” directory (includes `bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`, and a budget report):
  - `cargo run -p fretboard-dev -- diag ai-packet <bundle_dir|bundle.json|bundle.schema2.json> --packet-out <dir>`
  - If you already have sidecars but cannot read the bundle artifact (too large / unavailable), you can build a packet
    without reading the bundle:
    - `cargo run -p fretboard-dev -- diag ai-packet <bundle_dir> --sidecars-only --packet-out <dir>`
    - Requires the sidecars (`bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`, `frames.index.json`) to already exist.
  - If `bundle.schema2.json` is present, the packet may also include it (within the packet budget).
    - To generate it: `cargo run -p fretboard-dev -- diag doctor --fix-schema2 <bundle_dir> --warmup-frames <n>`
- Convenience: generate `ai.packet/` next to a bundle dir during common workflows:
  - After a scripted run: `cargo run -p fretboard-dev -- diag run <script.json> --ai-packet`
  - Before packing a share zip: `cargo run -p fretboard-dev -- diag pack <bundle_dir> --ai-packet`
  - Pack a bounded “AI-only” zip (packs `ai.packet/` + nearby script sources, but does not include the full bundle artifact):
    - `cargo run -p fretboard-dev -- diag pack <bundle_dir> --ai-only`
    - If the bundle dir has sidecars but no readable bundle artifact, `diag pack --ai-only` can still succeed by generating
      `ai.packet/` from sidecars (equivalent to `diag ai-packet <bundle_dir> --sidecars-only`).
  - For a multi-script repro run, pack a bounded `repro.ai.zip`:
    - `cargo run -p fretboard-dev -- diag repro <suite|script.json...> --ai-only`
    - If any repro item bundle artifacts are missing/unreadable but sidecars exist, this may still succeed by generating
      `ai.packet/` from sidecars for each item (equivalent to `diag ai-packet <bundle_dir> --sidecars-only`).
  - Agent plan JSON (includes recommended bounded commands like `diag pack --ai-only`):
    - `cargo run -p fretboard-dev -- diag agent <bundle_dir|bundle.json|bundle.schema2.json>`
- Focus on a specific target when possible (writes a bounded `slice.*.json` alongside the packet):
  - `cargo run -p fretboard-dev -- diag ai-packet <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id> --packet-out <dir>`
- If you only need a semantics-focused subset, slice directly:
  - `cargo run -p fretboard-dev -- diag slice <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id> --out <path>`
  - `cargo run -p fretboard-dev -- diag slice <bundle_dir|bundle.json|bundle.schema2.json> --step-index <n> --test-id <test_id> --warmup-frames <n> --out <path>`

## DevTools GUI (preview)

This repo includes a WIP DevTools GUI app at `apps/fret-devtools` that wraps the same inspect/pick/scripts/bundles
contracts with a low-friction, real-time UI (including web runner support via WebSocket transport).

Run the DevTools GUI (it hosts a loopback-only WS server):

- `cargo run -p fret-devtools`

Connect a target app (native):

- set `FRET_DEVTOOLS_WS=ws://127.0.0.1:7331/`
- set `FRET_DEVTOOLS_TOKEN=<token>`

Connect a target app (web runner):

- add `?fret_devtools_ws=ws://127.0.0.1:7331/&fret_devtools_token=<token>` to the URL

Notes:

- The GUI prints a ready-to-copy `ws://.../?fret_devtools_token=...` URL on startup.
- `FRET_DEVTOOLS_WS_PORT` overrides the default port (`7331`).
- This is workspace-internal and versioned but not yet stabilized; the portable “source of truth” remains the schema v2 bundle artifact
  (`bundle.schema2.json` preferred, with `bundle.json` as an optional large raw view).
## Quick Start (scripted repro)

1. Run the app with diagnostics enabled:

   - `FRET_DIAG=1`

   Note:
   - When using `fretboard-dev diag run --launch`, `--env KEY=VALUE` cannot override reserved variables
     like `FRET_DIAG`/`FRET_DIAG_DIR`/`FRET_DIAG_*_PATH`. Use CLI flags (e.g. `--dir`, `--*-path`),
     a config file (`FRET_DIAG_CONFIG_PATH`), or non-reserved `--env` overrides instead.
   - Tool-launched runs also scrub inherited diagnostics env vars from the parent shell; do not rely on
     setting `FRET_DIAG_*` in the parent shell to affect a `--launch` run. If you truly need a one-off
     override, pass it explicitly via `--env` (non-reserved keys only).

2. (Recommended while authoring scripts) disable redaction so you can see semantics labels in bundles:

   - `FRET_DIAG_REDACT_TEXT=0`

3. Write a `script.json` file (schema v2; schema v1 is deprecated but still accepted):

```json
{
  "schema_version": 2,
  "meta": {
    "required_capabilities": ["diag.script_v2", "diag.screenshot_png"]
  },
  "steps": [
    { "type": "click", "target": { "kind": "role_and_name", "role": "button", "name": "Open" } },
    { "type": "wait_until", "predicate": { "kind": "exists", "target": { "kind": "role_and_name", "role": "dialog", "name": "Settings" } }, "timeout_frames": 60 },
    { "type": "type_text", "text": "hello" },
    { "type": "press_key", "key": "enter" },
    { "type": "assert", "predicate": { "kind": "focus_is", "target": { "kind": "role_and_name", "role": "text_field", "name": "Search" } } },
    { "type": "capture_bundle", "label": "after-typing" },
    { "type": "capture_screenshot", "label": "after-typing" }
  ]
}
```

Optional: generate scripts from typed Rust templates

If you prefer authoring scripts in Rust (type-safe selectors + reusable helpers) but still want the portable/reviewable
JSON artifact, use `fret-diag-scriptgen`:

```bash
cargo run -p fret-diag-scriptgen -- list
cargo run -p fret-diag-scriptgen -- write todo-baseline-v2
```

This writes a JSON file under `.fret/diag/scripts/` by default and prints the path. You can then run it via:

```bash
cargo run -p fretboard-dev -- diag run .fret/diag/scripts/todo-baseline-v2.json --launch -- cargo run -p fret-demo --bin todo_demo
```

Implementation note: templates are built using the `fret_diag_protocol::builder` helpers.

Script tooling (no app required):

- Normalize formatting (stable diffs):
  - `cargo run -p fretboard-dev -- diag script normalize .\\script.json --write`
  - `cargo run -p fretboard-dev -- diag script normalize .\\script.json --check`
- Upgrade legacy schema v1 → v2 (schema-only; does not rewrite v2 scripts):
  - `cargo run -p fretboard-dev -- diag script upgrade .\\script.json --write`
  - `cargo run -p fretboard-dev -- diag script upgrade .\\script.json --check`
  - Note: tool-launched runs (`--launch`) reject `schema_version=1` scripts by default; upgrade first.
- PowerShell note: `diag script validate|lint` accept globs and directories (the CLI expands them):
  - `cargo run -p fretboard-dev -- diag script lint tools/diag-scripts/ui-gallery-select-*.json`
  - `cargo run -p fretboard-dev -- diag script validate tools/diag-scripts`
- Validate schema/parse (writes `check.script_schema.json` under `--dir`, or `--check-out`):
  - `cargo run -p fretboard-dev -- diag script validate .\\script.json`
- Lint scripts (capability inference + hygiene; writes `check.script_lint.json`):
  - `cargo run -p fretboard-dev -- diag script lint .\\script.json`

Repeat-run triage (flake hunting):

- Run the same script N times and write `repeat.summary.json` under `--dir` (includes `highlights` aggregates for quick scanning):
  - `cargo run -p fretboard-dev -- diag repeat .\\script.json --repeat 7 --launch -- cargo run -p fret-ui-gallery --release`

Script shrinking (automated minimal repro):

- Reduce a *failing* script to a smaller script that still reproduces the same failure signal.
  - By default, shrink matches `reason_code` when available (otherwise `reason`). Use `--shrink-any-fail` to accept any failure.
  - Requires either an already-running app, or `--launch -- <cmd...>`.
  - Place tool flags such as `--json`, `--session-auto`, and `--shrink-out` before `--launch -- <cmd...>`.
  - Writes a minimized script to `--shrink-out` (default: `target/fret-diag/shrink/script.min.json`) and a summary to `target/fret-diag/shrink/shrink.summary.json`.
  - Example:
    - `cargo run -p fretboard-dev -- diag script shrink .\\script.json --json --launch -- cargo run -p fret-ui-gallery --release`

4. Push the script into the running app (write `script.json` + touch `script.touch`):

   - `cargo run -p fretboard-dev -- diag script .\\script.json`

   Or run it and wait for a pass/fail result (CI-friendly):

   - `cargo run -p fretboard-dev -- diag run .\\script.json`
   - To also pack a bounded shareable `.zip` for AI triage (does not ship the full bundle artifact): `cargo run -p fretboard-dev -- diag run .\\script.json --bundle-doctor fix --pack --ai-only`
   - If you need an offline viewer-friendly zip (includes the bundle artifact): `cargo run -p fretboard-dev -- diag run .\\script.json --bundle-doctor fix --pack --include-all --pack-schema2-only`

   Or run a pre-defined suite (the app must be running):

   - `cargo run -p fretboard-dev -- diag suite ui-gallery`

5. The app executes **one step per frame** (deterministic), and (by default) auto-dumps after actions.
   Use `cargo run -p fretboard-dev -- diag latest` to grab the newest bundle.

Deterministic termination note (especially for multi-window docking scripts):

- Prefer ending a script with `capture_bundle` as the final step.
- Tool-launched filesystem runs (`fretboard-dev diag run --launch`) now reuse the bundle referenced by the
  final `script.result.json` when `--pack` / `--ai-packet` needs a post-run artifact, instead of
  forcing a second dump after `stage=passed`.
- For promoted `test_id` / role-only proof scripts, the runner now avoids leasing `ElementRuntime`
  across the whole dispatch path; steps that truly need runtime-backed selectors or span geometry
  still borrow it explicitly. This keeps screenshot proofs quiet while preserving
  `GlobalElementId` / selectable-span coverage when requested.
- Avoid ending a script with `wait_frames` / `wait_ms` (these are stabilization yields, not meaningful terminal steps).
- Avoid trailing `wait_frames` / `wait_ms` after the final `capture_bundle` (this can stall indefinitely if the last remaining window becomes occluded/idle and stops producing frames).
- Smoke/gate suites (e.g. `diag-hardening-smoke-*`) run a strict preflight and will fail early if a script ends with `wait_frames` / `wait_ms` or contains `wait_frames` / `wait_ms` after the final `capture_bundle` (see `check.script_termination.json` under the suite `--dir`).

Screenshot note:

- `capture_screenshot` requires the **on-demand PNG screenshot protocol**:
  - Enable via `FRET_DIAG_GPU_SCREENSHOTS=1` (default disabled).
  - This is distinct from `FRET_DIAG_BUNDLE_SCREENSHOT=1`, which only writes `frame.bmp` during bundle dumps.

## Quick Start (scripted perf triage)

Use this when the UI "feels slow" and you need a repeatable way to find the worst frames.

1. Run the app with diagnostics enabled:

   - `FRET_DIAG=1`

2. Run a predefined suite and report the slowest frames:

    - Reuse an already-running app:

      - `cargo run -p fretboard-dev -- diag perf ui-gallery --sort time`

      - Machine-readable JSON:

        - `cargo run -p fretboard-dev -- diag perf ui-gallery --sort time --json`

      - Repeatable perf summary (helps reduce noise; nearest-rank p50/p95 across N runs):

        - `cargo run -p fretboard-dev -- diag perf ui-gallery --repeat 7 --warmup-frames 5 --sort time --json`

    - Or launch a fresh process per script (clean state, slower):

      - `cargo run -p fretboard-dev -- diag perf ui-gallery --sort time --launch -- cargo run -p fret-ui-gallery --release`

Web runner note:

- `diag perf` uses the filesystem-trigger transport, so for web runner workflows you typically:
  1) run the script via `apps/fret-devtools` (or any devtools-ws-capable driver) to export bundles under `.fret/diag/exports/`, then
  2) compute a baseline from those bundle paths:
     - `cargo run -p fretboard-dev -- diag perf-baseline-from-bundles <script.json> .fret/diag/exports/<exported_unix_ms> --perf-baseline-out .fret/perf.web.baseline.json`

### Web runner: exporting bundles headlessly (Trunk + devtools-ws)

This is the simplest repeatable workflow for producing `.fret/diag/exports/<timestamp>/bundle.json` from a web/WASM app.

1. Start the loopback devtools WS hub (prints the token):

   - `cargo run -p fret-devtools-ws`

2. Serve the WASM app with Trunk:

   - `cd apps/fret-ui-gallery-web`
   - `trunk serve --port 8080`

3. Open the app URL with WS parameters:

   - `http://127.0.0.1:8080/?fret_devtools_ws=ws://127.0.0.1:7331/&fret_devtools_token=<token>`

4. Export a bundle by running a script that includes a `capture_bundle` step:

   - `cargo run -p fret-diag-export -- --script tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json --token <token>`

The command prints the export directory path, and writes:

- `.fret/diag/exports/<timestamp>/bundle.json`


3. Inspect the slowest snapshots in the resulting bundle:

   - Tail (worst wall-time frames): `cargo run -p fretboard-dev -- diag stats <bundle_dir> --sort time --top 20`
   - "Real work" vs preemption (UI thread CPU deltas):
     - Windows (best signal): `cargo run -p fretboard-dev -- diag stats <bundle_dir> --sort cpu_cycles --top 20`
     - Cross-platform fallback: `cargo run -p fretboard-dev -- diag stats <bundle_dir> --sort cpu_time --top 20`

   Notes:
   - If a frame is high in `--sort time` but near-zero in `cpu_time` / `cpu_cycles`, it's often OS scheduling noise
     (the UI thread didn't run), not a "real work" regression inside the frame phases.
   - For typical perf (ignore rare max spikes), prefer `diag perf --repeat ... --warmup-frames ... --json` and gate on
     `--perf-threshold-agg p95` (see perf workstreams for baseline policy).

4. Compare two bundles (diff by impact):

   - `cargo run -p fretboard-dev -- diag stats --diff <bundle_a> <bundle_b> --top 20`
   - JSON: `cargo run -p fretboard-dev -- diag stats --diff <bundle_a> <bundle_b> --top 50 --json`
   - Stats-lite support matrix: `cargo run -p fretboard-dev -- diag stats --stats-lite-checks-json`

5. Optional: inspect focused summaries without loading the full bundle by hand:

   - Single-bundle layout hotspot summary: `cargo run -p fretboard-dev -- diag layout-perf-summary <bundle_dir> --top 10 --json`
   - Aggregate footprint summary across captured samples: `cargo run -p fretboard-dev -- diag memory-summary target/fret-diag --top 10 --json`

6. Optional: emit a Chrome trace JSON derived from the bundle:

   - During perf runs: `cargo run -p fretboard-dev -- diag perf ui-gallery --trace --launch -- cargo run -p fret-ui-gallery --release`
   - For an existing bundle: `cargo run -p fretboard-dev -- diag trace <bundle_dir|bundle.json> [--trace-out <path>]`

7. Optional: turn on the perf hints gate (heuristic, explainable warnings):

   - `cargo run -p fretboard-dev -- diag perf ui-gallery --check-perf-hints --launch -- cargo run -p fret-ui-gallery --release`
   - Output evidence: `<out_dir>/check.perf_hints.json` (also indexed in `evidence.index.json` and included in `repro.zip` when present)
   - Defaults:
     - `--check-perf-hints` denies *all* hint codes and fails on `warn`/`error` severities.
     - Narrow the deny list with `--check-perf-hints-deny <code1,code2,...>` and/or tune severity with `--check-perf-hints-min-severity <info|warn|error>`.

Notes:

- When view caching is active, bundles include cache-root stats (replay ops, reuse reasons) to help
  identify "cache misses" vs "we are repainting anyway".
- For a CPU timeline view of these same frames, see: `docs/tracy.md`.
- For overall process resource footprint (especially "idle CPU" and memory regressions), use `fretboard-dev diag repro`
  to run a script and capture a `resource.footprint.json` summary (CPU + working set / pagefile on Windows).
  This is designed for CI/automation gates:
  - `cargo run -p fretboard-dev -- diag repro <script.json> --max-cpu-avg-percent-total-cores 2.0 --max-peak-working-set-bytes 800000000 --launch -- <cmd...>`
  - `avg_cpu_percent_total_cores` is normalized to *all* logical cores (e.g. ~3% on a 32-core machine is ~1 full core).

## Quick Start (picking / "inspect target")

This is the fastest way to author stable selectors (GPUI/Zed-style inspect):

1. Run the app with diagnostics enabled:

   - `FRET_DIAG=1`

2. Arm a one-shot pick (this waits for the next click and prints a selector JSON on success):

   - `cargo run -p fretboard-dev -- diag pick`

3. Click the UI element you want to target.

4. The app writes `pick.result.json` (and, by default, also dumps a bundle export labelled `pick`).

Notes:

- While picking is active, the app renders a non-interactive inspect overlay (outline + label) to help confirm which semantics node is being targeted.
- `pick.result.json` includes `selection.element_path` when the picked semantics node can be mapped to a declarative `GlobalElementId` (best-effort; diagnostics-only).

## Quick Start (continuous inspect mode)

This is closer to Zed/GPUI’s inspector workflow: keep an inspect overlay active while you hover, and (optionally) pick targets repeatedly on click.

1. Run the app with diagnostics enabled:

   - `FRET_DIAG=1`

2. Enable inspect mode (writes `inspect.json` and touches `inspect.touch`):

   - `cargo run -p fretboard-dev -- diag inspect on`

   Optional: allow clicks to keep reaching the app UI while still producing pick results:

   - `cargo run -p fretboard-dev -- diag inspect on --consume-clicks false`

3. Hover to see the candidate node; click to write `pick.result.json` (each click updates `run_id`).

4. Disable or toggle:

   - `cargo run -p fretboard-dev -- diag inspect off`
   - `cargo run -p fretboard-dev -- diag inspect toggle`
   - `cargo run -p fretboard-dev -- diag inspect status` (prints a 1-line JSON payload)

In-app shortcuts while inspect mode is active:

- `Esc`: disable inspect (writes `inspect.json` + touches `inspect.touch`)
- `Ctrl+C` / `Cmd+C`: copy the best selector JSON for the current selection (or hovered node) to the clipboard
- `Ctrl+Shift+C` / `Cmd+Shift+C`: copy a multi-line "selector + focus + path" payload (useful for bug reports and AI triage)
- `F`: lock selection to the currently focused semantics node (keyboard-first inspect)
- `L`: lock/unlock selection (freezes hover highlight; uses last hovered node)
- `Alt+Up` / `Alt+Down`: navigate the locked selection up/down the semantics parent chain (uses a small down-stack for “back to child”)

### Generate a runnable script from a pick

To reduce "pick → first repro script" friction, `fretboard` can generate a minimal script skeleton:

- `cargo run -p fretboard-dev -- diag pick-script`

This writes `target/fret-diag/picked.script.json` (override with `--pick-script-out`), which you can then run via:

- `cargo run -p fretboard-dev -- diag run target/fret-diag/picked.script.json`

### Patch an existing script using a pick (JSON Pointer)

When UI structure or labels change, use pick to update a script step's selector in-place:

- Update a click step:
  - `cargo run -p fretboard-dev -- diag pick-apply tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json --ptr /steps/0/target`
- Update a predicate target (e.g. `wait_until` / `assert`):
  - `cargo run -p fretboard-dev -- diag pick-apply tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json --ptr /steps/1/predicate/target`

By default this overwrites the script file; use `--out <path>` to write to a new file.

## What's inside bundle artifacts (`bundle.schema2.json` / `bundle.json`)

Bundles are a per-window ring history plus snapshots (schema is versioned and intended to evolve).

At a high level:

- `windows[].events[]`: recent normalized `fret-core::Event` (with redaction controls)
- `windows[].snapshots[]`: recent `UiDiagnosticsSnapshotV1`
  - `debug.stats`: layout/paint timings and counters
  - `debug.layout_engine_solves`: per-frame layout engine solves (roots + solve/measure time + top measure hotspots)
  - `debug.invalidation_walks`: top invalidation walks (roots, sources, and optional `detail` taxonomy)
  - `debug.cache_roots`: view-cache root stats (reuse + paint replay ops, optional `reuse_reason`, and `contained_relayout_in_frame` to flag which roots were re-laid out in the post-pass)
  - `debug.prepaint_actions`: prepaint-driven invalidations and scheduling requests (useful for ADR 0175 “ephemeral prepaint items” workflows)
  - `debug.virtual_list_windows`: VirtualList window telemetry (used to triage scroll-induced work)
    - `debug.virtual_list_windows[*].source`: whether the record was emitted from `layout` or `prepaint`
  - `debug.overlay_synthesis`: overlay cached-synthesis events (which overlays were synthesized from cached declarations, and why synthesis was suppressed)
  - `debug.viewport_input`: forwarded viewport input events (`Effect::ViewportInput`, ADR 0132)
  - `debug.docking_interaction`: docking interaction ownership snapshot (dock drag + viewport capture)
  - `debug.layers_in_paint_order`: overlay roots / barrier behavior / hit-test intent
  - `debug.hit_test`: last pointer position + hit summary
  - `debug.element_runtime`: `ElementRuntime` window-level state (focus/selection/observed models/globals; includes optional `*_path` strings for key elements)
  - `debug.semantics`: the exported semantics snapshot (ADR 0033) when enabled

For AI triage, the bundle is intentionally self-contained: it is the unit you attach to a bug report.

Common `debug.invalidation_walks[].detail` values (best-effort, may evolve):

- `model_observation`, `global_observation`
- `hover_event`, `focus_event`
- `scroll_handle`
- `focus_visible_policy`, `input_modality_policy`
- `animation_frame_request`

## Environment variables (current)

Core:

- `FRET_DIAG=1`: enable diagnostics collection.
- `FRET_DIAG_DIR=...`: output directory (default `target/fret-diag`).
- `FRET_DIAG_BUNDLE_JSON_FORMAT=pretty`: write pretty-printed raw `bundle.json` (when enabled; default: compact/minified).
- `FRET_DIAG_CONFIG_PATH=...`: optional JSON config file (schema v1) for diagnostics runtime settings and paths.
  - Tooling writes `<dir>/diag.config.json` by default when launching via `fretboard-dev diag run/suite/repro --launch`.
  - For most fields, an env var overrides the config file (compat-first manual escape hatch). A few size-control knobs
    are intentionally config-only for tool-launched determinism (see `tools/diag-configs/README.md`).
  - In `--launch` mode, the config file is expected to be writable. Tooling treats a failure to write `diag.config.json`
    as a launch error (to avoid silently falling back to runtime defaults that may write a large `bundle.json`).
  - In `--launch` mode, tooling scrubs inherited `FRET_DIAG_*` env vars from the parent shell before spawning the child
    process, then re-applies explicit `--env KEY=VALUE` overrides. This reduces "works on my machine" drift and avoids
    accidental output explosions caused by stale shell overrides.
  - In `--launch` mode, tooling-owned env vars and paths are reserved; `--env` cannot override them (use `--dir` / `--*-path` flags instead).
  - Example file to copy/modify: `tools/diag-configs/diag.config.example.json`.
  - Drift audit notes for the example file: `tools/diag-configs/README.md`.
  - Schema v1 script compatibility:
    - Config file key: `allow_script_schema_v1` (default: `true`).
    - Tool-launched runs write `allow_script_schema_v1=false` so the runtime stays v2-only.
  - Bundle artifact writing (size control):
    - Config file key: `write_bundle_json` (default: `true`; tooling typically writes `false` for launched runs).
    - Config file key: `write_bundle_schema2` (default: `false`; tooling typically writes `true` for launched runs).
    - Note: if no config file is used, manual dumps may still write raw `bundle.json` for compatibility. Prefer a config
      file for bounded, shareable artifacts.
    - Tool-launched escape hatch: pass `--launch-write-bundle-json` (must appear before `--launch`) to make tooling
      write a per-run config with `write_bundle_json=true` for that launched run.
  - Tip: print the effective merged config (and highlight unknown keys/envs):
    - `cargo run -p fretboard-dev -- diag config doctor --mode launch --dir .fret/diag`
    - `cargo run -p fretboard-dev -- diag config doctor --mode manual --report-json` (manual apps)
  - The doctor also emits warnings for common “output explosion” risk factors (e.g. large snapshot caps, semantics on every
    snapshot, pretty-printed `bundle.json`).

Canonical env vars (recommended):

- `FRET_DIAG_CONFIG_PATH` (preferred)
- `FRET_DIAG`
- `FRET_DIAG_GPU_SCREENSHOTS`
- `FRET_DIAG_REDACT_TEXT`
- `FRET_DIAG_FIXED_FRAME_DELTA_MS`

Deprecated aliases + removal plan:

- `docs/workstreams/diag-v2-hardening-and-switches-v1/deprecations.md`.

Config resolution order (runtime):

1. Tooling may set reserved env vars in `--launch` mode (including `FRET_DIAG_DIR` and `FRET_DIAG_*_PATH`), overriding any values from the parent shell.
2. If `FRET_DIAG_CONFIG_PATH` points to a readable config file, the runtime loads `UiDiagnosticsConfigFileV1` from it.
3. For most fields, non-empty env vars override config file values (manual escape hatch).
4. Any missing fields fall back to runtime defaults (for example `out_dir=target/fret-diag`, `max_events=2000`, `max_snapshots=300`).

### Tool-launched (`--launch`) env policy

`fretboard-dev diag ... --launch` aims to be deterministic and small-by-default. To reduce drift and accidental output
explosions:

- **Reserved (tooling-owned) keys:** `--env KEY=VALUE` cannot override them (use `--dir` / `--*-path` flags instead).
  - Examples: `FRET_DIAG`, `FRET_DIAG_DIR`, `FRET_DIAG_CONFIG_PATH`, and all `FRET_DIAG_*_PATH` keys.
- **Scrubbed inherited keys:** tooling removes known diagnostics env vars from the parent shell before spawning the child.
  - If you need a one-off override for a launched run, pass it explicitly via `--env`.
- **High-risk overrides (discouraged):** prefer a config file + `diag config doctor` when changing bundle size knobs.
  - Examples: `FRET_DIAG_MAX_SNAPSHOTS`, `FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS`, `FRET_DIAG_SEMANTICS=all`,
    `FRET_DIAG_BUNDLE_JSON_FORMAT=pretty`.
- `diag config doctor --mode launch` surfaces the scrubbed/reserved key lists (via `--report-json`) and reports which
  inherited keys were actually scrubbed for the current shell env.
  - To simulate one-off `--env` overrides for a tool-launched run, pass them to the doctor as well:
    `cargo run -p fretboard-dev -- diag config doctor --mode launch --env FRET_DIAG_MAX_SNAPSHOTS=50`.
  - For a quick human-readable list (no JSON), use:
    `cargo run -p fretboard-dev -- diag config doctor --mode launch --print-launch-policy`.

- `FRET_DIAG_TRIGGER_PATH=...`: dump trigger file (default `<dir>/trigger.touch`).
  - The trigger uses a **stamp** (monotonic integer) rather than mtime. Write a new integer value
    (e.g. unix ms) to trigger a dump; `fretboard-dev diag poke` does this for you.
  - Tooling convenience:
    - `fretboard-dev diag poke --wait` waits for `latest.txt` to update and prints the dump directory.
    - `fretboard-dev diag poke --wait --record-run` writes a tooling-owned per-run manifest directory and chunked bundle copy under `<dir>/<run_id>/`.
  - Optional (filesystem transport): write `<dir>/dump.request.json` (schema v1) before touching the trigger to
    provide dump metadata:
    - `label` (string): appended to the export directory name (`<timestamp>-<label>`),
    - `max_snapshots` (u32): overrides the dump snapshot cap (clamped by runtime config),
    - `request_id` (u64): best-effort correlation id (tooling-owned).
- `FRET_DIAG_MAX_EVENTS=...`: ring size for events.
- `FRET_DIAG_MAX_SNAPSHOTS=...`: ring size for snapshots.

Semantics export:

- `FRET_DIAG_SEMANTICS=0`: disable exporting `debug.semantics` into bundles (default enabled).
- `FRET_DIAG_MAX_SEMANTICS_NODES=...`: cap the number of exported semantics nodes per snapshot (default 50000).
- `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=1`: export only semantics nodes that have a `test_id` (keeps bundles small for large UIs; default disabled).
- `FRET_DIAG_BUNDLE_SEMANTICS_MODE=all|changed|last|off`: controls how often bundles include semantics snapshots.
  - `all`: include semantics on every snapshot.
  - `changed`: include semantics only when `semantics_fingerprint` changes (always keeps the last snapshot's semantics).
  - `last`: include semantics only on the last snapshot (default for script-driven dumps; useful for AI triage and very large UIs).
  - `off`: never include semantics in bundles (perf captures where semantics isn't needed).
- Prefer setting `write_bundle_schema2=true` in the diagnostics config file (`FRET_DIAG_CONFIG_PATH`) when you want a compact schema2 artifact (`bundle.schema2.json`) to be written during dumps.
- `FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE=...`: keep-alive budget for the UI Gallery Inspector torture (retained host; ADR 0177).

Privacy / size:

- `FRET_DIAG_REDACT_TEXT=0`: disable redaction (runtime default enabled; tool-launched runs via `fretboard-dev diag ... --launch` default to disabled).
- `FRET_DIAG_MAX_DEBUG_STRING_BYTES=...`: cap event debug strings and exported semantics text.
- `FRET_DIAG_MAX_GATING_TRACE_ENTRIES=...`: cap `debug.command_gating_trace` entries (default 200; clamped to <= 2000).

Practical bundle size controls (recommended for scripted repros you want to share):

- Prefer smaller dumps: `FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS=10` (or `5` for very small bundles).
- Bound path/debug strings: `FRET_DIAG_MAX_DEBUG_STRING_BYTES=2048` (or `1024`).
- If you mostly target `test_id`, consider `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=1` to keep semantics exports smaller.

Script harness:

- `FRET_DIAG_SCRIPT_PATH=...`: script JSON path (default `<dir>/script.json`).
- `FRET_DIAG_SCRIPT_TRIGGER_PATH=...`: script trigger file (default `<dir>/script.touch`).
- `FRET_DIAG_SCRIPT_RESULT_PATH=...`: script result JSON path (default `<dir>/script.result.json`).
- `FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH=...`: script result trigger file (default `<dir>/script.result.touch`).
- `FRET_DIAG_SCRIPT_AUTO_DUMP=0`: disable auto-dump after steps (default enabled).
- `FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS=...`: cap snapshots included in script-driven bundle dumps (default 30).

Script input isolation (recommended for deterministic playback, especially multi-window docking/tear-off):

- `FRET_DIAG_ISOLATE_POINTER_INPUT=1`: while a script is active, ignore external (non-script) pointer input events so
  accidental real mouse movement/clicks don't perturb scripted runs.
  - `--launch` runs default this to `1` (tooling also writes
    `isolate_external_pointer_input_while_script_running=true` into the per-run `diag.config.json`).
  - Escape hatch: pass `--env FRET_DIAG_ISOLATE_POINTER_INPUT=0` when you need interactive input during a script run.
- `FRET_DIAG_ISOLATE_KEYBOARD_INPUT=1`: while a script is active, ignore external (non-script) keyboard/text/IME events
  so accidental typing doesn't perturb scripted runs.
  - `--launch` runs default this to `1` (tooling also writes
    `isolate_external_keyboard_input_while_script_running=true` into the per-run `diag.config.json`).
  - Escape hatch: pass `--env FRET_DIAG_ISOLATE_KEYBOARD_INPUT=0` when you need interactive keyboard input during a
    script run.

Screenshot capture:

- Requires the running app to enable the `fret-launch/diag-screenshots` feature (runner-side readback + PNG encode).
- `FRET_DIAG_GPU_SCREENSHOTS=1`: enable GPU readback screenshots (default disabled).
  - Alternatively, set `screenshots_enabled=true` in the `FRET_DIAG_CONFIG_PATH` config file.
- `FRET_DIAG_SCREENSHOT_REQUEST_PATH=...`: screenshot request JSON path (default `<dir>/screenshots.request.json`).
- `FRET_DIAG_SCREENSHOT_TRIGGER_PATH=...`: screenshot request trigger file (default `<dir>/screenshots.touch`).
- `FRET_DIAG_SCREENSHOT_RESULT_PATH=...`: screenshot completion log JSON path (default `<dir>/screenshots.result.json`).
- `FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH=...`: screenshot completion trigger file (default `<dir>/screenshots.result.touch`).

The screenshot completion log is append-only (bounded) and includes a `request_id` that scripted steps can wait on.

Bundle screenshots (frame dump):

- `FRET_DIAG_BUNDLE_SCREENSHOT=1`: write `frame.bmp` into each bundle directory when dumping a bundle export.

Picking:

- `FRET_DIAG_PICK_TRIGGER_PATH=...`: pick trigger file (default `<dir>/pick.touch`).
- `FRET_DIAG_PICK_RESULT_PATH=...`: pick result JSON path (default `<dir>/pick.result.json`).
- `FRET_DIAG_PICK_RESULT_TRIGGER_PATH=...`: pick result trigger file (default `<dir>/pick.result.touch`).
- `FRET_DIAG_PICK_AUTO_DUMP=0`: disable auto-dump after a pick (default enabled).

Inspect mode:

- `FRET_DIAG_INSPECT_PATH=...`: inspect config JSON path (default `<dir>/inspect.json`).
- `FRET_DIAG_INSPECT_TRIGGER_PATH=...`: inspect config trigger file (default `<dir>/inspect.touch`).

## Target selection rules (MVP)

Selection is evaluated against the current `SemanticsSnapshot` (ADR 0033).

Supported selectors (v1 MVP):

- `{"kind":"test_id","id":"open-settings"}` (preferred when available; see "Test IDs")
- `{"kind":"role_and_name","role":"button","name":"Open"}`
- `{"kind":"role_and_path","role":"menu_item","name":"Close","ancestors":[{"role":"menu","name":"File"}]}`
- `{"kind":"global_element_id","element":123}` (low-level / best for harness tests; not a user-facing contract)
- `{"kind":"node_id","node":123456789}` (low-level / brittle; avoid for real tests)

## Supported scripted steps (v1 MVP)

- `click` (optional `button`: `left`/`right`/`middle`; default `left`; optional `pointer_kind`; schema v2 only: optional `window` target)
- `tap` (schema v2 only; touch-first gesture; optional `pointer_kind`; default `touch`; optional `window` target; capability-gated behind `diag.gesture_tap`)
- `long_press` (schema v2 only; touch-first gesture; optional `duration_ms` (default 500); optional `pointer_kind`; default `touch`; optional `window` target; capability-gated behind `diag.gesture_long_press`)
- `swipe` (schema v2 only; touch-first gesture; `delta_x`, `delta_y` movement; optional `steps` (default 8); optional `pointer_kind`; default `touch`; optional `window` target; capability-gated behind `diag.gesture_swipe`)
- `pinch` (schema v2 only; touch-first gesture; `delta` zoom amount (positive=in, negative=out); optional `steps` (default 8); optional `pointer_kind`; default `touch`; optional `window` target; capability-gated behind `diag.gesture_pinch`)
- `move_pointer` (schema v2 only: optional `window` target)
- `pointer_down` (schema v2 only; optional `window` target; starts a cross-step pointer session for "drag + key" flows)
- `pointer_move` (schema v2 only; optional `window` target; moves with the pressed buttons from `pointer_down`)
- `pointer_up` (schema v2 only; optional `window` target; ends the `pointer_down` session)
- `drag_pointer` (optional `button`, `steps`; schema v2 only: optional `window` target)
- `wheel` (optional `delta_x`, `delta_y`; default `0`; schema v2 only: optional `window` target)
- `press_key` (`key`: `escape`, `enter`, `tab`, `space`, `arrow_up/down/left/right`, `home`, `end`, `page_up/down`,
  `f1-f12`, `alt`/`alt_left`/`alt_right`, `a-z`, `0-9`,
  `comma`/`,`, `period`/`dot`/`.`, `slash`/`/`, `semicolon`/`;`, `quote`/`apostrophe`/`'`,
  `minus`/`dash`/`-`, `equal`/`=`, `bracket_left`/`left_bracket`/`[`, `bracket_right`/`right_bracket`/`]`,
  `backslash`/`\\`, `backquote`/`grave`/`` ` ``;
  optional `modifiers`: `{shift,ctrl,alt,meta}`, optional `repeat`)
- `press_shortcut` (schema v2 only; shortcut strings like `primary+p`, `primary+shift+p`, `alt+f`; supports
  modifier aliases `primary`/`cmd_or_ctrl`/`command_or_control` and `meta`/`cmd`/`command`)
- `type_text`
- `paste_text_into` (schema v2 only; click + request clipboard write + wait for `ClipboardWriteCompleted` success + `Primary+V`; gates paste-specific code paths with less boilerplate)
- `ime` (schema v2 only; deterministic IME event injection for composition/commit/preedit)
- `reset_diagnostics` (clears the diagnostics ring buffer for the current window; useful to avoid mount/settle frames in perf captures)
- `wait_frames` (schema v2 only: optional `window` target; `n` frames)
- `wait_ms` (schema v2 only: optional `window` target; `n_ms` milliseconds; last-resort stabilization when no semantic predicate exists)
- `wait_until` (schema v2 only: optional `window` target; optional `timeout_frames`, optional `timeout_ms`)
- `wait_shortcut_routing_trace` (schema v2 only; wait until the shortcut routing trace contains a matching entry; optional `timeout_frames`, optional `timeout_ms`)
- `wait_command_dispatch_trace` (schema v2 only; wait until the command dispatch trace contains a matching entry; optional `timeout_frames`, optional `timeout_ms`)
- `wait_overlay_placement_trace` (schema v2 only; wait until overlay placement trace contains a matching entry; optional `timeout_frames`, optional `timeout_ms`)
- `assert` (schema v2 only: optional `window` target)
- `capture_bundle` (optional `label`, optional `max_snapshots`)
- `capture_screenshot` (optional `label`, optional `timeout_frames`, optional `timeout_ms`)
- `set_clipboard_force_unavailable` (schema v2 only; simulates clipboard read/write denial; capability-gated behind `diag.clipboard_force_unavailable`)
- `set_clipboard_text` (schema v2 only; requests an OS clipboard text write; pair with `wait_clipboard_write_result` / `assert_clipboard_write_result` when the script must gate completion; capability-gated behind `diag.clipboard_text`)
- `wait_clipboard_write_result` (schema v2 only; waits for a clipboard write completion matching `outcome`, optional `error_kind`, and optional `message_contains`)
- `assert_clipboard_write_result` (schema v2 only; asserts the cached or next clipboard write completion matches `outcome`, optional `error_kind`, and optional `message_contains`)
- `assert_clipboard_text` (schema v2 only; asserts OS clipboard text equals an expected value; capability-gated behind `diag.clipboard_text`)
- `inspect_help_lock_best_match_and_copy_selector` (schema v2 only; in-app inspector helper: open help, search for `query`, lock the best match, and copy the best selector JSON to the clipboard; intended to avoid relying on shortcut injection in `--launch` runs)
- `set_window_inner_size` (schema v2 only; optional `window` target)
- `set_window_style` (schema v2 only; optional `window` target; best-effort OS window style patch; capability-gated behind `diag.window_style_patch_v1`; desktop-only as of 2026-03-04; supported patch fields: `z_level`, `background_material`, `hit_test`, `opacity_alpha_u8`)
- `set_window_insets` (schema v2 only; overrides safe-area/occlusion insets; capability-gated behind `diag.window_insets_override`)
- `set_window_outer_position` (schema v2 only; optional `window` target)
- `raise_window` (schema v2 only; optional `window` target)
- `set_cursor_screen_pos` (schema v2 only; runner-level cursor screen-position override, physical pixels; intended for cross-window routing in scripted runs)
- `set_cursor_in_window` (schema v2 only; runner-level cursor override using window-client physical pixels; intended for cross-window routing without hardcoding DPI)
- `set_cursor_in_window_logical` (schema v2 only; runner-level cursor override using window-client logical pixels; preferred when you need to seed a cross-window `pointer_down` session for later `pointer_move`/`pointer_up` window migration)
- `set_mouse_buttons` (schema v2 only; runner-level mouse button state override; capability-gated behind `diag.mouse_buttons_override`)
- `inject_incoming_open` (schema v2 only; simulates "open in..." / share-target flows; capability-gated behind `diag.incoming_open_inject`)
- `drag_pointer_until` (schema v2 only; optional `window` target; drag across frames until a predicate passes or timeout; intended for cross-window routing; optional `release_on_success` to end while keeping the pointer pressed)

Pointer kind note (as of 2026-02-27):

- Pointer-driven steps support an optional `pointer_kind` field: `mouse` (default), `touch`, or `pen`.
- `pointer_kind` is omitted from JSON when unset; existing scripts remain mouse-based.
- Requesting `pointer_kind: touch` requires the runner to advertise `diag.pointer_kind_touch`.
  Requesting `pointer_kind: pen` requires `diag.pointer_kind_pen`.
  Tooling fails fast and writes `check.capabilities.json` evidence when the capability is missing.
- For cross-step pointer sessions (`pointer_down`/`pointer_move`/`pointer_up`), `pointer_down.pointer_kind` sets the
  session kind; `pointer_move`/`pointer_up` must either omit `pointer_kind` or match the session kind.

Cross-window docking note (pointer sessions):

- During cross-window docking drags, a script may intentionally release the drag in a different window than where the
  pointer session started (e.g. start the drag in a torn-off window, then `pointer_up` in the main window to ensure the
  drop resolves in the correct dock graph).
- To keep this deterministic, ensure the final drop position is expressed in the *target window* coordinate space:
  - seed the target window coordinates via `set_cursor_in_window_logical` and/or `move_pointer` (with an explicit
    `window` target) before migrating `pointer_move`/`pointer_up` to that window,
  - optionally gate that a drop preview is resolved before releasing (`dock_drop_resolve_source_is`,
    `dock_drop_resolved_is_some`, `dock_drop_resolved_zone_is`).
  - Example: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-chained-tearoff-two-tabs-merge.json`
  - Example: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-title-bar-drag-docks-to-main.json`

Additional predicate kinds are occasionally added to unblock new regression gates (for example menu a11y checks).
When authoring scripts, prefer stable `test_id` selectors and stick to predicates documented here; see
`ecosystem/fret-bootstrap/src/ui_diagnostics.rs` for the authoritative list.

Recent additions:

- `role_is` (assert semantics role equality for a target)
- `checked_is` / `checked_is_none` (assert `checked` flag state; useful for checkbox/radio menu items)
- `active_item_is` (assert the active item for composite widgets: matches either container `active_descendant` or roving focus)
- `window_style_effective_is` (assert effective/clamped OS window style facets such as `transparent`, `appearance`, and `hit_test`)
- `dock_drop_preview_kind_is` (assert coarse docking drop preview decision: `wrap_binary` vs `insert_into_split`)
- `dock_drop_resolve_source_is` (assert which mechanism selected the current docking drop preview)
- `dock_drop_resolved_is_some` (assert whether the drop preview has a resolved target or stays `None`)
- `dock_drop_resolved_zone_is` (assert the resolved docking zone: left/right/top/bottom/center)
- `dock_drop_resolved_insert_index_is` (assert the resolved insert index when dropping into a tab strip)
- `dock_graph_canonical_is` / `dock_graph_has_nested_same_axis_splits_is` (assert N-ary docking canonical-form invariants via a cheap stats snapshot)
- `dock_graph_node_count_le` / `dock_graph_max_split_depth_le` (assert dock graph size/depth stays bounded after repeated operations)
- `known_window_count_ge` / `known_window_count_is` (assert number of currently open windows as best-effort reported by the runner; backed by the runner-owned window lifecycle diagnostics store when available, falling back to the input context service)
- `dock_drag_current_window_is` (assert that a dock drag session is active and its `current_window` matches a window target)
- `dock_drag_kind_is` (assert the active dock drag kind: `dock_panel` vs `dock_tabs`)
- `dock_drag_moving_window_is` (assert the runner-reported moving window for a dock drag; ImGui-style “follow window”)
- `dock_drag_window_under_moving_window_is` (assert the best-effort “window under moving window” selection during a dock drag)
- `dock_drag_window_under_moving_window_source_is` (assert which mechanism selected the “window under moving window”: platform vs heuristic)
- `dock_drag_active_is` (assert that a dock drag session is (or is not) active)
- `dock_drag_transparent_payload_hit_test_passthrough_applied_is` (assert whether the runner successfully applied OS click-through for the moving window during transparent payload)
- `text_composition_is` (assert whether a text surface is currently composing via IME)
- `ime_cursor_area_is_some` (assert whether a window-level IME cursor area snapshot exists)
- `ime_cursor_area_within_window` (assert the IME cursor area stays within the current window bounds; coarse “caret teleported” gate)
- `ime_cursor_area_min_size` (assert the IME cursor area has a meaningful size; catches “zero rect” bugs)
- `wait_shortcut_routing_trace` (assert keyboard routing outcomes like `reserved_for_ime`)
- `wait_overlay_placement_trace` (assert overlay placement decisions by geometry trace rather than screenshots)
- `asset_reload_epoch_ge` (assert the shared runtime asset reload epoch has reached a minimum value)
- `asset_reload_configured_backend_is` / `asset_reload_active_backend_is` (assert configured vs effective auto-reload backend such as `native_watcher` or `poll_metadata`)
- `asset_reload_fallback_reason_is` (assert why the active backend degraded, for example `watcher_install_failed`)

Notes:

- `capture_bundle` always writes a new bundle export directory (primary artifact: `bundle.schema2.json` preferred, with `bundle.json` as an optional large raw view depending on config).
  - When `FRET_DIAG_GPU_SCREENSHOTS=1`, the dump includes a screenshot and the step waits until it is written (so downstream automation can rely on it deterministically).
  - If you want an explicit screenshot step, follow with `capture_screenshot`.
  - Optional `max_snapshots` caps how many snapshots are included in this export (clamped to `FRET_DIAG_MAX_SNAPSHOTS`).
- `capture_screenshot` requests a screenshot for the **most recent bundle directory** (`last_dump_dir`) and waits for completion (up to `timeout_frames` and/or `timeout_ms`). If no bundle exists yet, the harness creates one first.
- `drag_pointer` runs over multiple frames so diagnostics bundles can capture and gate frame-to-frame behavior (prepaint outputs, paint-only invalidations, drag indicators). Roughly: 1 frame for `move+down`, `steps` frames of `move`, then 1 frame for `up`.
  - Pointer synthesis keeps positions slightly inside window bounds when the requested coordinates are still within the window (avoids edge hit-testing misses), but preserves intentionally out-of-bounds positions for tear-off / cross-window docking routes.

## Script schema v2 (intent-level steps)

Schema v2 keeps the same `steps` array shape, but adds higher-level intent steps that are still semantics-first
(selectors + window-bounds predicates), and can internally perform multi-frame behavior (wait/retry loops) without forcing
authors to hand-write brittle `wait_frames` chains.

### Script metadata (v2)

Schema v2 scripts may include a top-level `meta` object. Supported fields:

- `meta.name: string` (display-only)
- `meta.tags: string[]` (display-only)
- `meta.target_hints: string[]` (display-only)
- `meta.required_capabilities: string[]` (gated by `capabilities.json` / DevTools session capabilities)
- `meta.env_defaults: { [key: string]: string | number | boolean } | string[]`
  - A script-authored set of environment defaults applied only when the harness launches a fresh process
    (`fretboard-dev diag run --launch` / `fretboard-dev diag suite --launch`).
  - Command-line `--env KEY=VALUE` always wins over script defaults.
  - Suites fail early if scripts disagree on a default value for the same key.

Supported intent steps (v2):

- `click_stable` (wait for target bounds to settle, then click; optional `window` target)
- `click_selectable_text_span_stable` (stable click a tagged span inside a selectable text node; optional `window` target)
- `wait_bounds_stable` (wait until a target's bounds are stable across frames; optional `window` target)
- `ensure_visible` (wait until visible/within window bounds; optional `window` target)
- `move_pointer_sweep` (move pointer across frames while staying relative to a target; optional `window` target)
- `scroll_into_view` (wheel a container until a target becomes visible; optional `window` target)
- `type_text_into` (wait + click + type; optional `window` target)
- `paste_text_into` (wait + click + request clipboard write + wait for success + paste via clipboard + `Primary+V`; optional `window` target)
- `menu_select` (wait + open menu + click item; optional `window` target)
- `menu_select_path` (wait + open nested menus + click final item; optional `window` target)
- `inspect_help_lock_best_match_and_copy_selector` (open inspector help, search for `query`, lock the best match, and copy the best selector JSON; optional `window` target)
- `drag_to` (drag between two semantics targets; optional `window` target)
- `set_slider_value` (drag a slider to a desired value; optional `window` target; requires a parseable semantics `value`)
- `set_window_inner_size` (emit `WindowRequest::SetInnerSize`)
- `set_window_style` (emit `WindowRequest::SetStyle`)
- `set_window_outer_position` (emit `WindowRequest::SetOuterPosition`)
- `raise_window` (emit `WindowRequest::Raise`)
- `set_cursor_screen_pos` (write a best-effort cursor override for desktop runners to consume during cross-window drags; screen-space physical pixels)
- `set_cursor_in_window` (write a window-targeted cursor override for desktop runners to consume during cross-window drags; window-client physical pixels)
- `drag_pointer_until` (drag until a predicate passes, holding the session active across frames; optional `release_on_success: false` to keep the drag pressed for follow-up evidence steps like screenshots)

Desktop runner note (cursor override wire format):

- The diagnostics runtime writes `${FRET_DIAG_DIR}/cursor_screen_pos.override.txt` and touches
  `${FRET_DIAG_DIR}/cursor_screen_pos.touch` to notify the desktop runner.
- Format is a small key/value text payload (not JSON). Example:

```text
schema_version=1
kind=window_client_physical
window=123
x_px=220.0
y_px=220.0
```

For window-targeted steps, the optional `window` field supports:

- `{ "kind": "current" }` (default)
- `{ "kind": "first_seen" }`
- `{ "kind": "first_seen_other" }`
- `{ "kind": "last_seen" }`
- `{ "kind": "last_seen_other" }`
- `{ "kind": "window_ffi", "window": 123 }`

Example: `tools/diag-scripts/ui-gallery-slider-set-value.json`.

Note: window-targeted steps run against the target window's semantics snapshot. When a step targets
a different window, the diagnostics runtime migrates the active script to that window; subsequent
steps will execute there unless they specify another `window` target.

Note: `drag_pointer` also emits `Event::InternalDrag` (`over` per move + final `drop`). This is
useful for exercising cross-window internal drag routes (e.g. docking drop indicators) in scripted
diagnostics runs, and is ignored unless a matching cross-window drag session is active.

Example: right click a context menu trigger

```json
{ "type": "click", "button": "right", "target": { "kind": "role_and_name", "role": "button", "name": "ContextMenu (right click)" } }
```

Notes on `role_and_path`:

- `ancestors` are matched as an **ordered subsequence** on the parent chain (outermost -> innermost).
  - This allows skipping intermediate unlabeled/internal nodes.
- Order is **outermost -> innermost** (closest parent last).
- When multiple nodes match a selector, the harness prefers the node under the highest-`z_index` semantics root (topmost overlay),
  then prefers the deeper node (more specific).

## Test IDs (optional, debug/test-only)

Test IDs are exported as `debug.semantics.nodes[].test_id` and can be targeted by scripts via:

- `{"kind":"test_id","id":"..."}`

Rules:

- Test IDs do not affect accessibility: they are not mapped into AccessKit `name`/`label`.
- Prefer Test IDs for stable scripts when labels are dynamic or localized.
- Set them at authoring time on semantics props (examples):
  - `SemanticsProps.test_id`
  - `PressableA11y.test_id`
  - `TextInputProps.test_id`
  - `TextAreaProps.test_id`

## Diagnostics-only geometry anchors (retained/canvas UIs)

Some UIs (notably retained/canvas-style widgets like node graphs) have important interactive targets
whose geometry is not naturally represented as a normal semantics subtree (e.g. a port handle that is
painted inside a cached canvas tile).

In those cases, add **diagnostics-only semantics anchors**:

- They **do not paint** and **do not hit-test**.
- They exist solely to expose a stable `test_id` + `bounds` for scripted pointer steps (like `click`,
  `drag_to`) without relying on pixel coordinates.
- They should not become user-visible accessibility nodes in production flows; treat them as
  debug/test-only contract surface.

Node graph example (port anchors):

- Canvas root selector:
  - `{"kind":"test_id","id":"node_graph.canvas"}`
- Per-port anchor selectors used by scripts:
  - `{"kind":"test_id","id":"node_graph_demo.anchor.float_out"}`
  - `{"kind":"test_id","id":"node_graph_demo.anchor.float_in"}`

Implementation pointers:

- Anchor widget (semantics-only): `ecosystem/fret-node/src/ui/diag_anchors.rs` (`NodeGraphDiagAnchor`)
- Wiring anchors to the retained canvas: `ecosystem/fret-node/src/ui/canvas/widget/retained_widget.rs`
  (`NodeGraphCanvas::with_diagnostics_anchor_ports(...)`)
- Script that drives a wire drag via anchors (no pixel coords):
  - `tools/diag-scripts/extras/node-graph-demo-preset-families-paint-only.json`

Script authoring tip:

- Avoid `value_contains` / `value_equals` gates when `FRET_DIAG_REDACT_TEXT=1` (default): text/value fields may be
  redacted. Prefer `exists(test_id)` + intent steps, or add a value-free semantics flag that uses
  `semantics_present()` (see `NodeGraphDiagConnectingFlag`).

### Supported role strings (MVP)

Use the following lowercase role strings (subset of `SemanticsRole`):

`window`, `dialog`, `alert_dialog`, `panel`,
`button`, `text_field`,
`menu_bar`, `menu`, `menu_item`, `menu_item_checkbox`, `menu_item_radio`,
`tab_list`, `tab`, `tab_panel`,
`list`, `list_item`, `list_box`, `list_box_option`,
`checkbox`, `switch`, `slider`, `combo_box`, `radio_group`, `radio_button`,
`tooltip`, `text`, `tree_item`, `viewport`.

If a selector fails to resolve, the harness will wait and retry on the next frame (deterministic).

## `wait_until` and `assert` (avoiding brittle frame waits)

`wait_until` keeps the script deterministic without relying on wall-clock time:

- the predicate is evaluated once per frame against the current semantics snapshot,
- it either succeeds and advances, or times out and dumps a failure bundle.

Predicates (v1 MVP):

- `{"kind":"exists","target":<selector>}`
- `{"kind":"not_exists","target":<selector>}`
- `{"kind":"exists_under","scope":<selector>,"target":<selector>}` (scoped existence; disambiguates when multiple widgets match)
- `{"kind":"not_exists_under","scope":<selector>,"target":<selector>}` (scoped absence; returns false if `scope` does not exist)
- `{"kind":"focus_is","target":<selector>}`
- `{"kind":"focused_descendant_is","scope":<selector>,"target":<selector>}` (focused node equals `target` and is under `scope`)
- `{"kind":"role_is","target":<selector>,"role":"button"}`
- `{"kind":"label_contains","target":<selector>,"text":"Search"}`
- `{"kind":"value_contains","target":<selector>,"text":"foo"}`
- `{"kind":"value_equals","target":<selector>,"text":"foo"}` (exact value match; requires unredacted `value`)
- `{"kind":"pos_in_set_is","target":<selector>,"pos_in_set":2}`
- `{"kind":"set_size_is","target":<selector>,"set_size":10}`
- `{"kind":"checked_is","target":<selector>,"checked":true}`
- `{"kind":"selected_is","target":<selector>,"selected":true}`
- `{"kind":"visible_in_window","target":<selector>}` (target exists and intersects the window bounds)
- `{"kind":"bounds_within_window","target":<selector>,"padding_px":0,"eps_px":0}` (target bounds must be fully contained within the window, optionally padded inward; `eps_px` allows a small tolerance for subpixel rounding at non-1.0 DPI)
- `{"kind":"text_input_ime_cursor_area_within_window","padding_px":0,"eps_px":0}` (focused text input's IME cursor area must be fully contained within the window, optionally padded inward; intended for keyboard-avoidance / caret-visibility gates; requires `diag.text_input_snapshot`)
- `{"kind":"ime_surrounding_text_valid"}` (window-level IME surrounding text excerpt must be present and satisfy winit-style constraints: max bytes + UTF-8 char-boundary offsets; requires `diag.text_input_snapshot`)

Note: this list is intentionally incomplete; additional predicate kinds exist for specialized suites.
The authoritative list lives in `crates/fret-diag-protocol/src/lib.rs` (`UiPredicateV1`).

Docking predicates (require a `WindowInteractionDiagnosticsStore` publisher, typically `docking_arbitration_demo`):

- `{"kind":"dock_drop_preview_kind_is","preview_kind":"wrap_binary"}`
- `{"kind":"dock_drop_preview_kind_is","preview_kind":"insert_into_split"}`
- `{"kind":"dock_drop_resolve_source_is","source":"tab_bar"}`
- `{"kind":"dock_drop_resolved_is_some","some":true}`
- `{"kind":"dock_drop_resolved_zone_is","zone":"right"}`
- `{"kind":"dock_drop_resolved_insert_index_is","index":0}`
- `{"kind":"dock_graph_canonical_is","canonical":true}`
- `{"kind":"dock_graph_has_nested_same_axis_splits_is","has_nested":false}`
- `{"kind":"known_window_count_ge","n":2}`
- `{"kind":"known_window_count_is","n":1}`
- `{"kind":"dock_drag_current_window_is","window":{"kind":"last_seen_other"}}`
- `{"kind":"dock_graph_node_count_le","max":32}`
- `{"kind":"dock_graph_max_split_depth_le","max":8}`

Window style predicates (require runner window style diagnostics, typically provided by desktop runners):

- `{"kind":"window_style_effective_is","window":{"kind":"current"},"style":{"transparent":true}}`
- `{"kind":"window_style_effective_is","window":{"kind":"current"},"style":{"hit_test":"passthrough_all"}}`
- `{"kind":"window_style_effective_is","window":{"kind":"current"},"style":{"hit_test":"passthrough_regions","hit_test_regions_fingerprint64":123}}`
- `{"kind":"window_background_material_effective_is","window":{"kind":"current"},"material":"system_default"}`

Platform receiver predicates (desktop-only; require runner cursor probe diagnostics):

- `{"kind":"platform_window_receiver_at_cursor_is","window":{"kind":"current"}}`

Notes:

- `hit_test` supports: `normal`, `passthrough_all`, `passthrough_regions` (ADR 0324 / ADR 0313).
- `hit_test_regions_fingerprint64` is a stable, canonicalized fingerprint of the effective regions
  union. It is intended for scripted regression gates that want to assert that the runner applied
  a specific region shape without relying on pixels.
- `platform_window_receiver_at_cursor_is` is capability-gated behind
  `diag.platform_window_receiver_at_cursor_v1` and is intended to validate runner-level hit-test
  passthrough behavior without requiring OS mouse injection. Pair it with cursor override steps
  like `set_cursor_in_window_logical`. (Win32 uses `WindowFromPoint`; macOS is best-effort.)

## Debugging recipes (Radix primitives / shadcn / overlays)

### 1) "My click didn't hit the button"

Checklist:

1. Dump a bundle right after the click (or enable auto-dumps in scripted repros).
2. Inspect `debug.layers_in_paint_order` and `debug.hit_test`:
   - confirm the top layer is hit-testable and not blocking unexpectedly,
   - confirm `hit` points to the expected node.
3. Inspect `debug.semantics.nodes[]` to ensure:
   - the target node has the expected `role` and `label`,
   - the node's `bounds` encloses the expected point.

GPUI alignment note: scripted selection wants a future "picking mode" that disables caching for hitbox truth.
Until then, prefer selection by semantics and verify bounds in the bundle.

### 2) Radix-style Dialog / AlertDialog

Radix patterns rely on a modal barrier + focus management.

What to look for:

- `debug.layers_in_paint_order`: the modal root should indicate barrier-like behavior.
- `debug.semantics.barrier_root`: when a modal is open, background semantics are gated by the barrier.

Script tip:

- ensure dialog triggers and primary actions have stable semantics labels
  (e.g. `.a11y_label("Open settings")`, `.a11y_label("Confirm")`),
  then select by `role_and_name`.

### 3) Menus (DropdownMenu / ContextMenu / Menubar)

Menu stacks are overlay-heavy and easy to mis-debug without snapshots.

Recipe:

1. Script:
   - click the trigger (role `button` or `menu_item` depending on your surface),
   - wait 1-2 frames,
   - click the menu item.
2. Verify `debug.layers_in_paint_order` shows the menu layer as hit-testable.
3. Verify semantics nodes exist for:
   - the trigger,
   - the menu root,
   - the menu items (role `menu_item*`).

### 4) shadcn components: make semantics labels your "test handles"

shadcn surfaces often already set labels for accessibility or debugging.

Best practice for stable scripted tests:

- assign explicit `.a11y_label("...")` to:
  - the trigger button,
  - destructive actions,
  - menu items,
  - text fields (search boxes, command palette input).

This keeps tests selector-driven without introducing `test_id` as a styling/policy hook.

## Behavior testing strategy (today)

The current harness is intentionally simple:

- scripts are pushed via file triggers (`script.json` + `script.touch`),
- execution is deterministic and step-based,
- each step can dump a bundle for post-mortem debugging.

Recommended workflow:

1. Repro a bug manually once and dump a bundle.
2. Extract stable selectors from `debug.semantics` (role + label).
3. Encode a script and run it repeatedly.
4. Attach the script + the last failing bundle to an issue.

When you use `fretboard-dev diag run`, the running app writes a small status file:

- `script.result.json`: `{run_id, stage, reason, last_bundle_dir, ...}`
- `script.result.touch`: touched whenever the result is updated (useful for external watchers)

`fretboard-dev diag suite` runs multiple scripts sequentially using the same mechanism.

After a suite run, the CLI writes `suite.summary.json` under the diagnostics output dir (default: `.fret/diag/`).
This file is intended as the “open first” overview (stage/reason-code aggregates, plus small evidence
highlights) before you start opening individual bundles.

## Regression suites (starter)

The `tools/diag-scripts/` directory contains curated scripts intended to become a baseline suite.
For the UI gallery, run:

- `cargo run -p fretboard-dev -- diag suite ui-gallery`

### Liquid glass / CustomV3 degradation suites

The liquid glass demos ship a pair of suites that intentionally force **deterministic renderer degradation**
paths so `diag triage` can surface actionable hints (budget pressure, source aliasing, pyramid level loss).

Suites:

- `tools/diag-scripts/suites/liquid-glass-custom-v3/`
  - Captures baseline screenshots/bundles for the CustomV3 "lens" authoring demo surface (no forced degradation).
- `tools/diag-scripts/suites/liquid-glass-custom-v3-degraded/`
  - Forces an extreme low intermediate budget to trigger **BackdropSourceGroup** degradation.
  - Expected triage hint codes:
    - `renderer.backdrop_source_groups_raw_degraded`
    - `renderer.backdrop_source_groups_pyramid_degraded`
- `tools/diag-scripts/suites/liquid-glass-custom-v3-sources-degraded/`
  - Uses a “sweet spot” budget that keeps **CustomV3 active**, but degrades its requested renderer sources:
    - `src_raw` aliases to `src`,
    - pyramid degrades to 1 level.
  - Expected triage hint codes:
    - `renderer.custom_effect_v3_raw_aliased_to_src`
    - `renderer.custom_effect_v3_pyramid_degraded_to_one`

Run example (native, launch-managed):

- `cargo run -p fretboard-dev -- diag suite liquid-glass-custom-v3-sources-degraded --dir target/fret-diag/lg-v3 --session-auto --launch -- cargo run -p fret-demo --bin liquid_glass_demo`
- `cargo run -p fretboard-dev -- diag triage target/fret-diag/lg-v3/sessions/<session_id> --warmup-frames 0`

Notes:

- These suites require `FRET_DIAG_RENDERER_PERF=1` to populate renderer perf counters.
- If the intermediate budget is *too* low, the renderer may skip emitting a CustomV3 pass entirely.
  `diag triage` should surface `renderer.custom_effect_v3_requested_but_skipped`, and the source-level degradation counters
  will stay at 0. Prefer the curated suite budgets when you want `CustomV3 sources` degradation signals specifically.

### CustomV2 user-image compatibility suite

This suite exercises the deterministic fallback behavior when a CustomV2 user input image is incompatible with the ABI
(e.g. a non-filterable format combined with a filtering sampler). The backend should bind a 1x1 transparent fallback
image instead of triggering a wgpu validation error, and should surface the hint:

- `renderer.custom_effect_v2_user_image_incompatible_fallbacks`

Suite:

- `tools/diag-scripts/suites/cookbook-customv2-basics/`
  - `custom-effect-v2-non-filterable-input-fallback-screenshot.json`

Run example:

- `cargo run -p fretboard-dev -- diag suite cookbook-customv2-basics --dir target/fret-diag/customv2 --session-auto --launch -- cargo run -p fret-demo --bin custom_effect_v2_demo`
- `cargo run -p fretboard-dev -- diag triage target/fret-diag/customv2/sessions/<session_id> --warmup-frames 0`

### CustomV3 user-image compatibility suite

This suite is the CustomV3 counterpart: it binds an intentionally incompatible `user0` image (a non-filterable float
format) while the CustomV3 ABI expects filterable sampled textures. The backend should bind a deterministic 1x1
transparent fallback image instead of triggering a wgpu validation error, and should surface the hint:

- `renderer.custom_effect_v3_user_image_incompatible_fallbacks`

Suite:

- `tools/diag-scripts/suites/cookbook-customv3-basics/`
  - `custom-effect-v3-non-filterable-user0-fallback-screenshot.json`
  - `custom-effect-v3-non-filterable-user1-fallback-screenshot.json`
  - `custom-effect-v3-non-filterable-user01-fallback-screenshot.json`

Run example:

- `cargo run -p fretboard-dev -- diag suite cookbook-customv3-basics --dir target/fret-diag/customv3 --session-auto --launch -- cargo run -p fret-demo --bin custom_effect_v3_demo`
- `cargo run -p fretboard-dev -- diag triage target/fret-diag/customv3/sessions/<session_id> --warmup-frames 0`

Note:

- The script library is modularized via a taxonomy plus a minimal, generated registry for “promoted” scripts
  (`tools/diag-scripts/index.json`, scope: suite-reachable + `_prelude`; regenerate via
  `cargo run -p fretboard-dev -- diag registry write`).
  `fretboard-dev diag run` accepts either an explicit path or a promoted `script_id` from this registry.
  For discoverability, use `fretboard-dev diag list scripts` to print `script_id -> path` mappings.
  Use `fretboard-dev diag list suites` to print known suites (derived from promoted registry `suite_memberships`).
  To detect taxonomy/registry drift without grepping, use `fretboard-dev diag doctor scripts` (read-only;
  suggests repair commands like `migrate-script-library.py --apply --write-redirects` and
  `diag registry write`). Use `--strict` to fail when promoted scripts drift back to schema v1.
  Prefer directory- and glob-based inputs (`--script-dir`, `--glob`) for ad-hoc runs, and avoid assuming scripts live
  only at the top-level. See: `docs/workstreams/diag-v2-hardening-and-switches-v1/README.md`.
- Built-in suites are defined as curated directory inputs under `tools/diag-scripts/suites/<suite-name>/`.
  In-tree suites are expressed via a single `suite.json` manifest (tooling-only) that lists canonical script paths.
  See: `tools/diag-scripts/suites/README.md`.
- Use `--prelude-script <script.json>` to run shared reset/normalization scripts from `tools/diag-scripts/_prelude/*`.
  When the suite reuses a single process, preludes run once before the first script by default; use
  `--prelude-each-run` to run preludes before every script.
- Migration helper (dry-run by default): `python tools/diag-scripts/migrate-script-library.py`.
- During migration, legacy script paths may be left behind as small `script_redirect` JSON stubs. Tooling resolves these
  stubs before pushing scripts to the runtime, so redirects never become part of the runtime contract surface.
- When applying moves, suite manifests (`tools/diag-scripts/suites/**/suite.json`) are rewritten to reference the
  canonical (post-move) script paths, avoiding indirect dependencies on redirect stubs.

For component-focused conformance scripts (built-in suites), run:

- `cargo run -p fretboard-dev -- diag suite ui-gallery-select --timeout-ms 240000 --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard-dev -- diag suite ui-gallery-combobox --timeout-ms 240000 --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard-dev -- diag suite ui-gallery-text-ime --timeout-ms 240000 --launch -- cargo run -p fret-ui-gallery --release`

For Embla-engine-dependent Carousel gates, run:

- `cargo run -p fretboard-dev -- diag suite ui-gallery-carousel-embla-engine --launch -- cargo run -p fret-ui-gallery --release`

To force-disable the Embla engine (debug), set `FRET_DEBUG_CAROUSEL_EMBLA_ENGINE=0`.

To keep “Rust template ↔ JSON script” closure, check that the committed scripts match typed templates:

- `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-select`
- `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-combobox`
- `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-text-ime`

The UI gallery suite includes lightweight smoke checks for table/grid surfaces:

- `tools/diag-scripts/ui-gallery-table-smoke.json`
- `tools/diag-scripts/ui-gallery-data-table-smoke.json`

These scripts assert that stable semantics anchors exist *and* that their bounds are within the
window (`bounds_within_window`), which is a fast way to catch “layout is broken / clipped to zero”
regressions when a table suddenly “disappears”.

The diagnostics harness also includes docking arbitration scripts (multi-viewport + modal):

- `tools/diag-scripts/docking-arbitration-demo-split-viewports.json`
- `tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json`
- `tools/diag-scripts/docking-arbitration-demo-float-zone-floats-in-window.json`
- `tools/diag-scripts/docking-arbitration-demo-tab-bar-drop-end-insert-index.json`
- `tools/diag-scripts/docking-arbitration-demo-tab-bar-drop-end-insert-index-two-tabs.json`
- `tools/diag-scripts/docking-arbitration-demo-tab-bar-drop-end-insert-index-overflow.json`

You can run them as a built-in suite:

- `cargo run -p fretboard-dev -- diag suite docking-arbitration --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`

There are also multi-window (tear-off) docking scripts (require `diag.multi_window` capability):

- `tools/diag-scripts/docking-arbitration-demo-multiwindow-cross-window-hover.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-title-bar-drag-docks-to-main.json`
- `tools/diag-scripts/docking-arbitration-demo-multiwindow-drag-tab-back-to-main.json`
- `tools/diag-scripts/docking-arbitration-demo-multiwindow-tearoff-merge-loop-no-leak.json`

Example (run one script against the demo, launching a fresh process):

- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking-arbitration-demo-multiwindow-drag-tab-back-to-main.json --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`

Local debugging note:

- For “not yet suite-worthy” helper scripts (especially bundle-capture scripts used to debug flaky multi-window cases),
  keep them under a `local-debug/` directory inside the canonical taxonomy location (example:
  `tools/diag-scripts/docking/arbitration/local-debug/`).
- These scripts are intentionally *not* promoted into `tools/diag-scripts/index.json` and should not be relied on by
  CI-style suites.

### View-cache regression gating

Some scripted regressions only matter when view-cache reuse actually happens. To avoid false positives,
you can enforce a minimum number of cache-root reuse events observed in the exported `bundle.json`.

Example (UI gallery):

- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --check-view-cache-reuse-min 1 --warmup-frames 5 --launch -- cargo run -p fret-ui-gallery --release`

Notes:

- `--check-view-cache-reuse-min N` counts `debug.cache_roots[].reused == true` events in snapshots after `--warmup-frames`.
- If `view_cache_active` is false for all snapshots (or `cache_roots` are not exported), the check will fail by design.

### Overlay synthesis regression gating

Some overlay regressions only show up when overlay requests must be synthesized from cached declarations
(because view caching skipped rerendering the producer subtree). To avoid "it passed but never tested
the synthesis seam", you can gate on synthesis events exported in `bundle.json`:

- `--check-overlay-synthesis-min N` counts `debug.overlay_synthesis[].outcome == "synthesized"` events in snapshots after `--warmup-frames`.

### Viewport input regression gating

Some docking / embedded-viewport regressions only matter if viewport input forwarding actually happened
(i.e. `Effect::ViewportInput` was emitted and drained). To avoid “it passed but never exercised viewport tooling”,
you can gate on forwarded viewport input events exported in `bundle.json`:

- `--check-viewport-input-min N` counts `debug.viewport_input[]` events in snapshots after `--warmup-frames`.
- `--check-dock-drag-min N` counts snapshots where `debug.docking_interaction.dock_drag` is present.
- `--check-viewport-capture-min N` counts snapshots where `debug.docking_interaction.viewport_capture` is present.

### Matrix runner (uncached vs cached)

To automate the “view-cache is behavior preserving” check across the UI gallery suite, run the matrix:

- `cargo run -p fretboard-dev -- diag matrix ui-gallery --dir target/fret-diag --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint --launch -- cargo run -p fret-ui-gallery --release`

Notes:

- Requires `--launch` so the runner can control `FRET_UI_GALLERY_VIEW_CACHE` (0 vs 1) per run.
- Writes bundles under `--dir/uncached` and `--dir/cached`, then compares each script pair via `diag compare` semantics.
- Default reuse gate is `--check-view-cache-reuse-min 1` (pass `--check-view-cache-reuse-min 0` to disable the gate).
- If `--env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1` is set, the matrix run also defaults to `--check-overlay-synthesis-min 1` for the cached variant (pass `--check-overlay-synthesis-min 0` to disable). The gate is only enforced for overlay-centric scripts (non-overlay scripts in the suite are exempt).

Recommended (CI/automation):

- `python3 tools/diag_matrix_ui_gallery.py --out-dir target/fret-diag --warmup-frames 5 --release --json`

### Bundle comparison (cached vs uncached)

To build confidence that view-cache is "behavior preserving", compare two captured bundles.
`fretboard-dev diag compare` focuses on stable `debug.semantics.nodes[].test_id` anchors and can also compare
`scene_fingerprint` (paint output fingerprint) for the selected snapshots. For session-level memory drift,
the canonical resource mode is `--footprint`, which reads `resource.footprint.json` from the resolved
session roots instead of the bundle body.

Example:

- `cargo run -p fretboard-dev -- diag compare ./target/fret-diag/uncached ./target/fret-diag/cached --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint --json`
- `cargo run -p fretboard-dev -- diag compare <session_a> <session_b> --footprint --json`

Notes:

- By default, the command compares the last snapshot after `--warmup-frames` (per bundle, first window).
- Use `--compare-ignore-bounds` if you only want structural semantics checks (role/flags/actions).
- Use `--compare-ignore-scene-fingerprint` if the scene fingerprint is expected to differ (e.g. non-deterministic content).

## Troubleshooting

**Tooling `--launch` fails**

- `fretboard-dev diag ... --launch` writes a per-run `diag.config.json` into `--dir` and expects it to be writable.
- On launch failures, tooling writes a best-effort `script.result.json` with `reason_code=tooling.launch.failed` so triage
  stays bounded and machine-readable.

**The app never dumps bundles**

- confirm `FRET_DIAG=1`,
- confirm the app uses `fret-bootstrap` `UiAppDriver`,
- run `cargo run -p fretboard-dev -- diag path` and ensure the trigger file is being touched.

**A scripted click never resolves**

- disable redaction while authoring: `FRET_DIAG_REDACT_TEXT=0`,
- dump a bundle and inspect `debug.semantics.nodes[]` to confirm the label/role,
- if the UI is mid-transition, add `wait_frames` between steps.

**Multiple windows**

- Bundles are per-window; scripts currently execute against the first window that picks up the pending script.
- `known_window_count_*` gates are for **currently open windows** (runner best-effort). Use them to:
  - wait for tear-off window birth (`known_window_count_ge`),
  - wait for auto-close after merge-back (`known_window_count_is`).
  If you need an “ever seen windows” gate, add a dedicated predicate instead of overloading window count.
- For z-order / overlap cases during cross-window drags:
  - prefer tool-launched `--launch` (input isolation + keepalive),
  - seed the target window cursor model with `set_cursor_in_window_logical` before migrating `pointer_move`/`pointer_up`,
  - if a step exposes a `window` target (e.g. `wait_frames.window`), prefer a window that is actively producing frames (e.g. `first_seen`).
