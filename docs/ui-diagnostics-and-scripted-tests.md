---
title: UI Diagnostics Bundles & Scripted Interaction Tests
status: living
scope: debugging, AI triage, scripted repros
---

# UI Diagnostics Bundles & Scripted Interaction Tests

This doc describes the current **diagnostics bundle** workflow and the **MVP scripted interaction harness**
implemented for Fret apps that run through `fret-bootstrap`'s `UiAppDriver`.

Scope note:

- This file focuses on **bundles + scripts** (how to dump `bundle.json`, how the script harness is triggered, and
  how to author stable, selector-driven repros).
- For the **interactive inspect workflow** (hover/pick overlay, shortcuts, and selector copy UX), see:
  `docs/debugging-ui-with-inspector-and-scripts.md`.

The goal is GPUI/Zed-style "inspectable, shareable repro units":

- capture a portable bundle (`bundle.json`) that can be sent to another developer (or an AI tool),
- select targets by **semantics** (ADR 0033) rather than paint output,
- run deterministic scripted repros without adding ad-hoc debug UI.

Related ADRs:

- ADR 0174: `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- ADR 0033 (Semantics/a11y): `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Roadmap/TODO: `targets/ui-diagnostics-inspector-todo.md`

## Quick Start (manual bundle dump)

1. Run any demo/app wired via `UiAppDriver` and enable diagnostics:

   - `FRET_DIAG=1`

2. Reproduce the issue.

3. Trigger a dump:

   - `cargo run -p fretboard -- diag poke`

4. Locate the most recent bundle directory:

   - `cargo run -p fretboard -- diag latest`
   - The bundle file is `bundle.json` under that directory.

By default bundles go under `target/fret-diag/<timestamp>/` and `target/fret-diag/latest.txt` is updated.

## Optional: dump a frame screenshot alongside the bundle

If you suspect a **rendering** regression (e.g. semantics + layout look correct but pixels look blank),
enable bundle screenshots:

- `FRET_DIAG_SCREENSHOT=1`

When a bundle is dumped, the runner writes `frame.bmp` into the bundle directory (same folder as
`bundle.json`).

Notes:

- This is **bundle-scoped** and **dump-triggered**:
  - The runtime writes a `screenshot.request` file into the bundle directory when dumping `bundle.json`.
  - The desktop runner detects that request and writes `frame.bmp` (and `screenshot.done`) as best-effort.
- This is intentionally separate from the on-demand PNG screenshot protocol used by scripted steps
  like `capture_screenshot` (see below).

## Offline bundle viewer (optional)

This repo includes an offline web viewer for `bundle.json` at `tools/fret-bundle-viewer`.

```powershell
$env:HTTP_PROXY='http://127.0.0.1:10809'
$env:HTTPS_PROXY='http://127.0.0.1:10809'

pnpm -C tools/fret-bundle-viewer install
pnpm -C tools/fret-bundle-viewer dev
```

Workflow tip:

- Drag the `bundle.json` file from `target/fret-diag/.../bundle.json` into the viewer (or use the file picker).
- You can also open a `.zip` that contains a `bundle.json` anywhere inside it (handy for sharing a full repro directory).
- To generate a shareable `.zip` for the latest bundle: `cargo run -p fretboard -- diag pack`
- To include nearby artifacts (`script.json`, `script.result.json`, `pick.result.json`), `triage.json`, and screenshots (when present): `cargo run -p fretboard -- diag pack --include-all`
- The bundle viewer surfaces these zip artifacts (and lets you copy/download them) when they are present under `_root/`.
- To generate a machine-readable `triage.json` next to a bundle: `cargo run -p fretboard -- diag triage <bundle_dir|bundle.json>`
- To include `triage.json` in a share zip: `cargo run -p fretboard -- diag pack --include-triage`
- To include screenshots in a share zip: `cargo run -p fretboard -- diag pack --include-screenshots` (packs `target/fret-diag/screenshots/<bundle_timestamp>/` into `_root/screenshots/` when available)
- If you’re sharing via chat, “Paste JSON” is a fast way to load a copied `bundle.json` payload without files.
- Use “Export triage.json” when you want a small, machine-readable artifact for AI triage (selection + bounded debug artifacts).
## Quick Start (scripted repro)

1. Run the app with diagnostics enabled:

   - `FRET_DIAG=1`

2. (Recommended while authoring scripts) disable redaction so you can see semantics labels in bundles:

   - `FRET_DIAG_REDACT_TEXT=0`

3. Write a `script.json` file (schema v1):

```json
{
  "schema_version": 1,
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

4. Push the script into the running app (write `script.json` + touch `script.touch`):

   - `cargo run -p fretboard -- diag script .\\script.json`

   Or run it and wait for a pass/fail result (CI-friendly):

   - `cargo run -p fretboard -- diag run .\\script.json`
   - To also pack the most recent bundle (plus optional artifacts) into a shareable `.zip`: `cargo run -p fretboard -- diag run .\\script.json --pack --include-all`

   Or run a pre-defined suite (the app must be running):

   - `cargo run -p fretboard -- diag suite ui-gallery`

5. The app executes **one step per frame** (deterministic), and (by default) auto-dumps after actions.
   Use `cargo run -p fretboard -- diag latest` to grab the newest bundle.

Screenshot note:

- `capture_screenshot` requires the **on-demand PNG screenshot protocol**:
  - Enable via `FRET_DIAG_SCREENSHOTS=1` (default disabled).
  - This is distinct from `FRET_DIAG_SCREENSHOT=1`, which only writes `frame.bmp` during bundle dumps.

## Quick Start (scripted perf triage)

Use this when the UI "feels slow" and you need a repeatable way to find the worst frames.

1. Run the app with diagnostics enabled:

   - `FRET_DIAG=1`

2. Run a predefined suite and report the slowest frames:

    - Reuse an already-running app:

      - `cargo run -p fretboard -- diag perf ui-gallery --sort time`

      - Machine-readable JSON:

        - `cargo run -p fretboard -- diag perf ui-gallery --sort time --json`

      - Repeatable perf summary (helps reduce noise; nearest-rank p50/p95 across N runs):

        - `cargo run -p fretboard -- diag perf ui-gallery --repeat 7 --warmup-frames 5 --sort time --json`

    - Or launch a fresh process per script (clean state, slower):

      - `cargo run -p fretboard -- diag perf ui-gallery --sort time --launch -- cargo run -p fret-ui-gallery --release`

3. Inspect the slowest snapshots in the resulting bundle:

   - `cargo run -p fretboard -- diag stats <bundle_dir> --sort time --top 20`

Notes:

- When view caching is active, bundles include cache-root stats (replay ops, reuse reasons) to help
  identify "cache misses" vs "we are repainting anyway".
- For a CPU timeline view of these same frames, see: `docs/tracy.md`.

## Quick Start (picking / "inspect target")

This is the fastest way to author stable selectors (GPUI/Zed-style inspect):

1. Run the app with diagnostics enabled:

   - `FRET_DIAG=1`

2. Arm a one-shot pick (this waits for the next click and prints a selector JSON on success):

   - `cargo run -p fretboard -- diag pick`

3. Click the UI element you want to target.

4. The app writes `pick.result.json` (and, by default, also dumps a `bundle.json` labelled `pick`).

Notes:

- While picking is active, the app renders a non-interactive inspect overlay (outline + label) to help confirm which semantics node is being targeted.
- `pick.result.json` includes `selection.element_path` when the picked semantics node can be mapped to a declarative `GlobalElementId` (best-effort; diagnostics-only).

## Quick Start (continuous inspect mode)

This is closer to Zed/GPUI’s inspector workflow: keep an inspect overlay active while you hover, and (optionally) pick targets repeatedly on click.

1. Run the app with diagnostics enabled:

   - `FRET_DIAG=1`

2. Enable inspect mode (writes `inspect.json` and touches `inspect.touch`):

   - `cargo run -p fretboard -- diag inspect on`

   Optional: allow clicks to keep reaching the app UI while still producing pick results:

   - `cargo run -p fretboard -- diag inspect on --consume-clicks false`

3. Hover to see the candidate node; click to write `pick.result.json` (each click updates `run_id`).

4. Disable or toggle:

   - `cargo run -p fretboard -- diag inspect off`
   - `cargo run -p fretboard -- diag inspect toggle`
   - `cargo run -p fretboard -- diag inspect status` (prints a 1-line JSON payload)

In-app shortcuts while inspect mode is active:

- `Esc`: disable inspect (writes `inspect.json` + touches `inspect.touch`)
- `Ctrl+C` / `Cmd+C`: copy the best selector JSON for the current selection (or hovered node) to the clipboard
- `Ctrl+Shift+C` / `Cmd+Shift+C`: copy a multi-line "selector + focus + path" payload (useful for bug reports and AI triage)
- `F`: lock selection to the currently focused semantics node (keyboard-first inspect)
- `L`: lock/unlock selection (freezes hover highlight; uses last hovered node)
- `Alt+Up` / `Alt+Down`: navigate the locked selection up/down the semantics parent chain (uses a small down-stack for “back to child”)

### Generate a runnable script from a pick

To reduce "pick → first repro script" friction, `fretboard` can generate a minimal script skeleton:

- `cargo run -p fretboard -- diag pick-script`

This writes `target/fret-diag/picked.script.json` (override with `--pick-script-out`), which you can then run via:

- `cargo run -p fretboard -- diag run target/fret-diag/picked.script.json`

### Patch an existing script using a pick (JSON Pointer)

When UI structure or labels change, use pick to update a script step's selector in-place:

- Update a click step:
  - `cargo run -p fretboard -- diag pick-apply tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json --ptr /steps/0/target`
- Update a predicate target (e.g. `wait_until` / `assert`):
  - `cargo run -p fretboard -- diag pick-apply tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json --ptr /steps/1/predicate/target`

By default this overwrites the script file; use `--out <path>` to write to a new file.

## What's inside `bundle.json`

Bundles are a per-window ring history plus snapshots (schema is versioned and intended to evolve).

At a high level:

- `windows[].events[]`: recent normalized `fret-core::Event` (with redaction controls)
- `windows[].snapshots[]`: recent `UiDiagnosticsSnapshotV1`
  - `debug.stats`: layout/paint timings and counters
  - `debug.layout_engine_solves`: per-frame layout engine solves (roots + solve/measure time + top measure hotspots)
  - `debug.invalidation_walks`: top invalidation walks (roots, sources, and optional `detail` taxonomy)
  - `debug.cache_roots`: view-cache root stats (reuse + paint replay ops, optional `reuse_reason`, and `contained_relayout_in_frame` to flag which roots were re-laid out in the post-pass)
  - `debug.prepaint_actions`: prepaint-driven invalidations and scheduling requests (useful for ADR 0190 “ephemeral prepaint items” workflows)
  - `debug.virtual_list_windows`: VirtualList window telemetry (used to triage scroll-induced work)
    - `debug.virtual_list_windows[*].source`: whether the record was emitted from `layout` or `prepaint`
  - `debug.overlay_synthesis`: overlay cached-synthesis events (which overlays were synthesized from cached declarations, and why synthesis was suppressed)
  - `debug.viewport_input`: forwarded viewport input events (`Effect::ViewportInput`, ADR 0147)
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
- `FRET_DIAG_TRIGGER_PATH=...`: dump trigger file (default `<dir>/trigger.touch`).
- `FRET_DIAG_MAX_EVENTS=...`: ring size for events.
- `FRET_DIAG_MAX_SNAPSHOTS=...`: ring size for snapshots.

Semantics export:

- `FRET_DIAG_SEMANTICS=0`: disable exporting `debug.semantics` into bundles (default enabled).

Privacy / size:

- `FRET_DIAG_REDACT_TEXT=0`: disable redaction (default enabled).
- `FRET_DIAG_MAX_DEBUG_STRING_BYTES=...`: cap event debug strings and exported semantics text.
- `FRET_DIAG_MAX_GATING_TRACE_ENTRIES=...`: cap `debug.command_gating_trace` entries (default 200; clamped to <= 2000).

Script harness:

- `FRET_DIAG_SCRIPT_PATH=...`: script JSON path (default `<dir>/script.json`).
- `FRET_DIAG_SCRIPT_TRIGGER_PATH=...`: script trigger file (default `<dir>/script.touch`).
- `FRET_DIAG_SCRIPT_RESULT_PATH=...`: script result JSON path (default `<dir>/script.result.json`).
- `FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH=...`: script result trigger file (default `<dir>/script.result.touch`).
- `FRET_DIAG_SCRIPT_AUTO_DUMP=0`: disable auto-dump after steps (default enabled).

Screenshot capture:

- Requires the running app to enable the `fret-launch/diag-screenshots` feature (runner-side readback + PNG encode).
- `FRET_DIAG_SCREENSHOTS=1`: enable GPU readback screenshots (default disabled).
- `FRET_DIAG_SCREENSHOT_REQUEST_PATH=...`: screenshot request JSON path (default `<dir>/screenshots.request.json`).
- `FRET_DIAG_SCREENSHOT_TRIGGER_PATH=...`: screenshot request trigger file (default `<dir>/screenshots.touch`).
- `FRET_DIAG_SCREENSHOT_RESULT_PATH=...`: screenshot completion log JSON path (default `<dir>/screenshots.result.json`).
- `FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH=...`: screenshot completion trigger file (default `<dir>/screenshots.result.touch`).

The screenshot completion log is append-only (bounded) and includes a `request_id` that scripted steps can wait on.

Bundle screenshots (frame dump):

- `FRET_DIAG_SCREENSHOT=1`: write `frame.bmp` into each bundle directory when dumping `bundle.json`.

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

- `click` (optional `button`: `left`/`right`/`middle`; default `left`)
- `move_pointer`
- `drag_pointer` (optional `button`, `steps`)
- `wheel` (optional `delta_x`, `delta_y`; default `0`)
- `press_key` (`key`: `escape`, `enter`, `tab`, `space`, `arrow_up/down/left/right`, `home`, `end`, `page_up/down`;
  optional `modifiers`: `{shift,ctrl,alt,meta}`, optional `repeat`)
- `type_text`
- `reset_diagnostics` (clears the diagnostics ring buffer for the current window; useful to avoid mount/settle frames in perf captures)
- `wait_frames`
- `wait_until`
- `assert`
- `capture_bundle`
- `capture_screenshot` (optional `label`, optional `timeout_frames`)

Notes:

- `capture_bundle` always writes a new `bundle.json` directory. If you need a screenshot for that bundle, follow it with a `capture_screenshot` step.
- `capture_screenshot` requests a screenshot for the **most recent bundle directory** (`last_dump_dir`) and waits for completion (up to `timeout_frames`, default 300). If no bundle exists yet, the harness creates one first.

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
- `{"kind":"focus_is","target":<selector>}`
 - `{"kind":"visible_in_window","target":<selector>}` (target exists and intersects the window bounds)
 - `{"kind":"bounds_within_window","target":<selector>,"padding_px":0}` (target bounds must be fully contained within the window, optionally padded inward)

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

When you use `fretboard diag run`, the running app writes a small status file:

- `script.result.json`: `{run_id, stage, reason, last_bundle_dir, ...}`
- `script.result.touch`: touched whenever the result is updated (useful for external watchers)

`fretboard diag suite` runs multiple scripts sequentially using the same mechanism.

## Regression suites (starter)

The `tools/diag-scripts/` directory contains curated scripts intended to become a baseline suite.
For the UI gallery, run:

- `cargo run -p fretboard -- diag suite ui-gallery`

The UI gallery suite includes lightweight smoke checks for table/grid surfaces:

- `tools/diag-scripts/ui-gallery-table-smoke.json`
- `tools/diag-scripts/ui-gallery-data-table-smoke.json`

These scripts assert that stable semantics anchors exist *and* that their bounds are within the
window (`bounds_within_window`), which is a fast way to catch “layout is broken / clipped to zero”
regressions when a table suddenly “disappears”.

The diagnostics harness also includes docking arbitration scripts (multi-viewport + modal):

- `tools/diag-scripts/docking-arbitration-demo-split-viewports.json`
- `tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json`

You can run them as a built-in suite:

- `cargo run -p fretboard -- diag suite docking-arbitration --launch -- cargo run -p fret-examples --bin docking_arbitration_demo --release`

### View-cache regression gating

Some scripted regressions only matter when view-cache reuse actually happens. To avoid false positives,
you can enforce a minimum number of cache-root reuse events observed in the exported `bundle.json`.

Example (UI gallery):

- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --check-view-cache-reuse-min 1 --warmup-frames 5 --launch -- cargo run -p fret-ui-gallery --release`

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

- `cargo run -p fretboard -- diag matrix ui-gallery --dir target/fret-diag --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint --launch -- cargo run -p fret-ui-gallery --release`

Notes:

- Requires `--launch` so the runner can control `FRET_UI_GALLERY_VIEW_CACHE` (0 vs 1) per run.
- Writes bundles under `--dir/uncached` and `--dir/cached`, then compares each script pair via `diag compare` semantics.
- Default reuse gate is `--check-view-cache-reuse-min 1` (pass `--check-view-cache-reuse-min 0` to disable the gate).
- If `--env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1` is set, the matrix run also defaults to `--check-overlay-synthesis-min 1` for the cached variant (pass `--check-overlay-synthesis-min 0` to disable). The gate is only enforced for overlay-centric scripts (non-overlay scripts in the suite are exempt).

Recommended (CI/automation):

- `pwsh tools/diag_matrix_ui_gallery.ps1 -OutDir target/fret-diag -WarmupFrames 5 -Release -Json`

### Bundle comparison (cached vs uncached)

To build confidence that view-cache is "behavior preserving", compare two captured bundles.
`fretboard diag compare` focuses on stable `debug.semantics.nodes[].test_id` anchors and can also compare
`scene_fingerprint` (paint output fingerprint) for the selected snapshots.

Example:

- `cargo run -p fretboard -- diag compare ./target/fret-diag/uncached ./target/fret-diag/cached --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint --json`

Notes:

- By default, the command compares the last snapshot after `--warmup-frames` (per bundle, first window).
- Use `--compare-ignore-bounds` if you only want structural semantics checks (role/flags/actions).
- Use `--compare-ignore-scene-fingerprint` if the scene fingerprint is expected to differ (e.g. non-deterministic content).

## Troubleshooting

**The app never dumps bundles**

- confirm `FRET_DIAG=1`,
- confirm the app uses `fret-bootstrap` `UiAppDriver`,
- run `cargo run -p fretboard -- diag path` and ensure the trigger file is being touched.

**A scripted click never resolves**

- disable redaction while authoring: `FRET_DIAG_REDACT_TEXT=0`,
- dump a bundle and inspect `debug.semantics.nodes[]` to confirm the label/role,
- if the UI is mid-transition, add `wait_frames` between steps.

**Multiple windows**

- bundles are per-window; scripts currently execute against the first window that picks up the pending script.
