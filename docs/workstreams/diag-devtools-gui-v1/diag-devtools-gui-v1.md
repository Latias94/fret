---
title: Diagnostics DevTools GUI v1
status: draft
date: 2026-02-07
scope: diagnostics, automation, devtools, web-runner
---

# Diagnostics DevTools GUI v1

This workstream defines a user-facing Diagnostics DevTools GUI for Fret apps.

For the newer umbrella framing that treats DevTools GUI as one diagnostics consumer lane within the
broader runtime/tooling/artifact refactor, see:

- `docs/workstreams/diag-fearless-refactor-v2/README.md`

The key goal is to make the existing diagnostics workflow (bundles, inspect/pick, scripted repros, gates)
available with **low-friction, real-time UX** for app/component authors — without weakening Fret’s layering
and “contract-first” philosophy.

Related foundations:

- Bundles + scripts: `docs/ui-diagnostics-and-scripted-tests.md`
- Artifact + evidence contract (M2): `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- Inspect + pick UX: `docs/debugging-ui-with-inspector-and-scripts.md`
- Debugging playbook: `docs/debugging-playbook.md`
- Base contract ADR: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- Semantics contract ADR: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- CLI tooling baseline: `crates/fret-diag` (wrapped by `apps/fretboard/src/diag.rs`)
- In-app diagnostics service: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- Offline viewer: `tools/fret-bundle-viewer`
- UI prototype (rough): `docs/devtool.html`
- v2 dogfood workflow: `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`
- GUI refresh follow-up: `docs/workstreams/standalone/diag-devtools-gui-refresh-v1.md`

## Problem statement

Today, diagnostics are powerful but CLI-first:

- authors can run scripts (`tools/diag-scripts/*.json`) and collect bundles (`bundle.json`),
- authors can pick stable selectors and apply them (`pick`, `pick-apply`),
- tooling can gate regressions (stale paint/scene, pixels-changed, perf thresholds, footprint limits).

The missing piece for “everyday use” is a **DevTools GUI** that:

- feels like a modern editor devtools (inspect overlay + live state),
- makes script authoring and selector management fast (pick-to-fill, library browsing),
- makes artifacts navigable (latest bundle, pack/share, open in viewer),
- supports **web runner** from day 1 (browser targets cannot rely on filesystem triggers).

## Current status snapshot

- `apps/fret-devtools` already has a first read-only `Regression` details tab.
- The tab reuses the existing artifacts-root contract and reads:
  - `regression.summary.json`,
  - `regression.index.json`.
- The tab now also exposes a thin `Summarize` trigger that runs the existing aggregate summarization flow
  against the current artifacts root and refreshes the panel when the job completes.
- It does not invent a new campaign model; it is only a thin GUI consumer over the same aggregate
  artifacts already used by CLI and MCP.
- The current end-to-end dogfood path is documented in
  `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`.
- A product/UX-focused follow-up track now lives in
  `docs/workstreams/standalone/diag-devtools-gui-refresh-v1.md`.
- The GUI first-open shell now also surfaces a `Gate Commands` block for stale paint/scene,
  pixels-changed, perf-threshold, and resource-footprint diagnostics entrypoints. This keeps the
  gate vocabulary visible in the product surface without moving gate policy into `fret-ui` or
  `fret-imui`.
- That `Gate Commands` block now consumes the shared `fret-diag` DevTools gate profile projection
  instead of hard-coding gate taxonomy in the GUI. The shared projection names each gate profile,
  command template, evidence files, and scan-speed notes, including explicit
  `check.resource_footprint.json` threshold evidence for resource-footprint gates.
- The GUI now renders a `Copy command` action for each shared gate profile, so authors can copy the
  exact stale/pixels/perf/resource-footprint command template from the structured profile row
  instead of selecting text from the raw first-open block.
- The GUI also has a script-target command builder for the stale paint/scene and pixels-changed
  profiles. `fret-diag` owns the script/test-id parameterization API; the GUI only supplies the
  selected profile, input models, preview, and `Copy generated command` action.
- That shared script-target projection now carries structured `diag_args` and `missing_inputs`, so
  the GUI can distinguish copyable templates from runnable gate commands without parsing shell
  command strings.
- The GUI can now launch a generated stale paint/scene or pixels-changed script-target gate command
  through the shared diagnostics engine. Each run records in-flight/error state in the panel and
  writes a lightweight `.fret/diag/gate-runs/*.json` result artifact with command/status/timing
  metadata.
- Generated gate results are retained as a bounded selectable history. The panel shows selected
  result details, a structured summary, raw JSON, copy actions for path/command/JSON, and a
  platform URL open action for the selected result artifact.
- The same generated-gate builder now includes a perf-threshold profile. `fret-diag` owns the
  structured `diag perf` command projection for target, repeat, warmup frames, aggregate, and the
  first threshold fields; the GUI only renders the form and runs the shared `diag_args`.
- Resource-footprint thresholds are also first-class in the generated-gate builder. The slice fixed
  the `diag repro` CLI contract so `--max-working-set-bytes`,
  `--max-peak-working-set-bytes`, and `--max-cpu-avg-percent-total-cores` are real parsed inputs,
  then exposed a GUI form that passes a single launch argv item without shell parsing.
- The selected regression summary inspector now consumes the shared `fret-diag`
  regression-bundle follow-up projection, generating concrete commands from the selected
  `bundle_dir` (`diag stats`, `layout-perf-summary`, `memory-summary`, `triage`, `hotspots`,
  `trace`, visual compare, and footprint compare), so authors can move from failing aggregate
  summary to the next diagnostic command without re-deriving paths by hand.
- The same projection now carries structured follow-up command metadata. The GUI separates
  bundle-local runnable commands from manual compare commands that still require a baseline, so
  placeholder compare commands are visible without being presented as ready-to-run actions.
- The bundle-local runnable set now includes `diag trace <bundle> --json`, which keeps Chrome trace
  artifact generation in the same selected-summary action surface as stats, layout, memory, triage,
  and hotspots.
- Bundle-local runnable follow-ups can now be launched from the selected-summary inspector through
  the shared diagnostics engine, with in-flight/error status recorded in the GUI; baseline-required
  compare commands remain manual until the user supplies a baseline.
- Each GUI-launched follow-up now writes a result JSON under `.fret/diag/followups/` and exposes the
  latest result path for copying, so the GUI path leaves a lightweight evidence artifact.
- Trace follow-up result JSON now includes an `output_artifacts` entry for
  `trace.chrome.json`; the summary and details blocks render that artifact path directly, so a
  successful trace run leaves a clickable/copyable evidence trail instead of only a pass/fail
  status.
- The selected-summary inspector also mirrors the latest selected-bundle follow-up result JSON
  inline, making pass/fail/error/timing metadata visible without leaving the DevTools surface.
- The same inspector projects that selected-bundle follow-up result into a structured summary above
  the raw JSON, keeping status, command, duration, and error preview readable at scan speed.
- GUI-launched follow-up results are also retained as a bounded in-memory history and filtered by
  the selected bundle, so authors do not confuse a previous bundle's last result with the current
  selected-summary evidence.
- That selected-bundle history is now rendered as selectable result entries, allowing authors to
  switch the summary/raw JSON/copy target between recent follow-up artifacts.
- The selected result now has a details block with status, path, command, bundle, and error preview,
  plus a copy action for the exact command that produced that artifact.
- The selected follow-up JSON artifact can also be opened through the platform URL handler via an
  escaped `file://` URL, keeping artifact inspection one click away when the native backend
  supports file URLs.
- The copy action now uses the selected bundle's latest follow-up history entry rather than the
  global last result, so copied evidence paths stay aligned with the current selected summary.
- The same selected-bundle result JSON can be copied directly from the inspector, giving issue
  reports and AI-assisted triage the exact payload shown in the panel.
- Trace follow-up artifacts can now be copied or opened directly from the same selected-result
  actions. The GUI resolves relative `trace.chrome.json` paths against the repo root and still
  keeps the result JSON as the durable evidence record.
- The live inspect path now has a real minimal WS payload loop instead of only a GUI raw-JSON
  receiver. `fret-diag-protocol` owns `UiInspectHoverV1`, `UiInspectFocusV1`, overlay hook, and
  `UiOverlaySummaryV1` types; `fret-bootstrap` publishes changed `inspect.hover`,
  `inspect.focus`, and `overlay.summary` payloads with hovered/focused node bounds, viewport
  bounds, barrier roots, blocking roots, and topmost interactive root hints; `fret-devtools`
  projects them into `Live Inspect Hover Bounds`, `Live Inspect Overlay Hooks`, and raw payload
  panels.
- The GUI first-open shell now includes a `Dogfood Workflow` block for the concrete
  `ui-gallery-button-dogfood` loop: open `fret-ui-gallery`, pick a Button-page selector, generate
  or apply the selector into a script, run with `diag run --pack`, optionally pack a selected
  bundle, and open `tools/fret-bundle-viewer`. The block references
  `tools/diag-scripts/ui-gallery-lite-smoke.json` and
  `tools/diag-scripts/ui-gallery/button/ui-gallery-button-with-icon-non-overlap.json` so the
  visible GUI path stays tied to existing script artifacts instead of a GUI-only workflow model.
- The Semantics tree scalability slice is now locked by code-level gates. The left tree continues
  to render through `VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16)`, while
  `semantics::compute_rows` uses an explicit stack so a 50k-deep expanded tree cannot overflow the
  Rust call stack. Focused tests cover 50k flat rows, 50k deep rows, and large-tree search.
- Low-traffic live selected-node details are also source-gated: `ws::maybe_request_semantics_node_details`
  still sends only on-demand `semantics.node.get` and `hit_test.explain` requests, and the extracted
  `live_semantics_request_decision` test proves unchanged selections are throttled to 1Hz while
  selection changes and manual refreshes bypass the throttle.
- The MCP `fret_diag_regression_dashboard` tool now consumes the same shared regression drill-down
  and follow-up projection, so GUI and AI workflows see the same bundle dirs, capability
  provenance, perf evidence, and next-command hints. It exposes both command-line strings and
  structured follow-up command rows with `diag_args`, so AI clients can run bundle-local actions
  like `trace` without parsing shell text.

## Goals (v1)

1. **Real-time inspect workflow**
   - Toggle inspect on/off, arm pick, capture selector JSON, copy and apply it to script steps quickly.
2. **Script Studio**
   - Browse `tools/diag-scripts/`, fork/edit scripts, validate schema, run and see progress + failures.
3. **Run + gate UX**
   - First-class UI for running `run/suite/repro/perf/matrix/compare` and showing evidence outputs.
4. **Artifacts + sharing**
   - Latest bundle list, pack zip, open offline bundle viewer, surface triage and evidence files.
   - Prefer emitting an explicit `out_dir` in `bundle.dumped` payloads so tooling can compute absolute paths reliably.
5. **Web runner support**
   - A transport that works when the target app runs in the browser (no filesystem access).

## Non-goals (v1)

- Remote debugging across machines (LAN/WAN) as a supported product feature.
- A fully stabilized public “DevTools protocol” (we will version it, but treat it as workspace-internal).
- Replacing `fretboard` CLI workflows; the GUI should complement them and reuse the same contracts.

## Architecture overview

### Key principle: keep bundles as the portable “source of truth”

Real-time UI is for iteration speed, but **the shareable unit remains the bundle**:

- deterministic scripts should still be able to emit `bundle.json`,
- regression gates should still produce evidence JSON files,
- offline inspection should still flow through `tools/fret-bundle-viewer`.

### v1 decisions (lock early; avoid churn)

These are v1 defaults we should treat as “sticky”:

- **Default left-panel tree is the Semantics tree** (`SemanticsSnapshot`).
  - Rationale: stable selectors (`test_id`) and alignment with inspect/pick/scripts.
  - Layout/element trees can exist as secondary views, but scripts should not depend on them.
  - 2026-05-15 status: `apps/fret-devtools` exposes secondary `Layout` and `Elements` tabs derived
    from the same semantics cache. `Layout` shows bounds/parent/test-id rows for layout debugging;
    `Elements` shows semantics-node identity and authoring relationships. These are not full native
    layout-engine or declarative element snapshots.
- **Default real-time transport is WebSocket** (bidirectional, low-latency, web-runner friendly).
  - Session routing uses `session_id` to support multiple app targets concurrently.
- **Minimize live traffic**:
  - Live transport sends only a minimal tree skeleton and small hover/focus summaries.
  - Expensive details are fetched on-demand for the selected node (“inspect-on-demand”).
- **Selected node detail refresh uses low-frequency polling** (e.g. ~1Hz), not per-frame push.
- **Virtualize the tree UI** (target 50k+ semantics nodes) and keep filtering/search cheap.
  - MVP implementations may start non-virtualized but must cap rows/work per frame to remain responsive.
- **Backpressure is allowed** for hover spam: drop intermediate hover events under load.

### UI skeleton (prototype-driven)

This repo contains a rough DevTools UI prototype at `docs/devtool.html`. It is not a contract, but it
is a useful starting point for agreeing on the user-facing information architecture:

- Top toolbar:
  - inspect/pick toggle,
  - refresh/reconnect,
  - node filter/search,
  - basic live perf badges (FPS, frame time).
- Left: a virtualized tree (expand/collapse, hover highlight, selection).
- Center: viewport preview and/or live overlay explanation (selection bounds, label).
- Right: inspector tabs with a stable, discoverable layout:
  - `layout` (box model + computed geometry + layout engine fields),
  - `style` (applied classes/tokens + raw style dump),
  - `attributes` (performance counters, text/value summaries, debug flags).
- Footer: connection/session status (transport + endpoint), diagnostics settings shortcuts.

Important: the production DevTools should dogfood Fret’s own docking panels and code editor, but
the IA above is a good “v1 default” to converge on.

### Split into three layers: protocol, transport, and tooling UX

1) **Protocol crate**: `crates/fret-diag-protocol`

- Owns JSON-serializable types:
  - selectors, predicates, script steps (schema v1/v2),
  - pick/inspect results,
  - run progress and failures,
  - minimal “live inspect” event payloads.
- Must be usable from native + wasm32.
- Versioning rules:
  - every message includes `schema_version`,
  - unknown fields must be ignored by default (forward compatibility).

2) **Tooling client crate**: `crates/fret-diag`

- Owns “devtools client” workflows:
  - file-trigger transport helpers (touch/write/wait),
  - pack/share helpers,
  - bundle stats/gates/compare (moved from `apps/fretboard` diag CLI module),
  - JSON parsing + validation utilities for scripts.
- This is the “engine” used by both:
  - `fretboard` CLI (thin wrapper),
  - `fret-devtools` GUI (primary UX).

3) **GUI app**: `apps/fret-devtools`

- A Fret app that dogfoods editor-grade UI:
  - docking panels, script editor, selectors browser, artifacts list, run timeline.
- Uses `crates/fret-diag` for operations and `crates/fret-diag-protocol` for types.

### Optional (but recommended): MCP surface for AI-driven testing

To make diagnostics automation easy for AI agents and IDE-integrated assistants, we add an MCP server
adapter built on the official Rust MCP SDK (`rmcp`).

This MCP adapter should not invent new capabilities. It should expose a *small* set of tools that
map directly to the same operations the GUI/CLI perform (inspect, pick, run script, pack artifacts,
compare bundles), and (optionally) expose common artifacts as MCP resources.

Resource model (recommended):

- Expose key artifacts as resources under a stable URI scheme (e.g. `fret-diag://sessions/<id>/bundle.json`).
- Support `resources/subscribe` so AI clients can wait for updates without polling.
  - On updates, send notifications:
    - `notifications/resources/list_changed` when the set of resource URIs changes (session add/remove),
    - `notifications/resources/updated` when a subscribed resource’s content changes (e.g. after `bundle.dumped`).

Recommended packaging:

- A dedicated binary `apps/fret-devtools-mcp` (headless) for automation and CI.
- Optionally, the GUI can embed/launch the MCP server for convenience.
- End-to-end workflow guide: `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md`.

### Transport strategy (v1)

We support two transports with the same protocol payloads:

#### A) Native filesystem transport (existing; keep)

- Works via `FRET_DIAG_DIR` files:
  - `script.json` + `script.touch`
  - `script.result.json`
  - `pick.touch` + `pick.result.json`
  - `inspect.json` + `inspect.touch`
  - `screenshots.request.json` + `screenshots.touch` + `screenshots.result.json`
- This remains the “most deterministic” and CI-friendly path.

#### B) WebSocket transport (new; required for web runner)

Browser targets cannot be controlled via file triggers, so we add a WebSocket-based transport.

Recommended topology (enables web runner immediately):

- **DevTools GUI hosts a local WS server** on `127.0.0.1:<port>`.
- The target app (native or web) connects as a WS client.
  - Web runner can connect to localhost via browser WebSocket APIs.
  - Native runner can use a lightweight client (non-blocking).

Why WS (instead of HTTP streaming):

- we need bidirectional, low-latency commands + events,
- SSE/HTTP streaming is one-way and would require a second channel for commands anyway.

HTTP endpoints may still be used (optional) for:

- downloading packed zips or large artifacts,
- serving the offline bundle viewer for convenience.

MCP integration note:

- MCP is a *tooling API* between an AI client and our DevTools, not the runtime transport between DevTools and the app.
- The MCP server should call into `crates/fret-diag` and use whatever app transport is available (filesystem or WS).

### Security / safety (local-only defaults)

The WS server should be “safe-by-default” for local dev:

- bind to loopback only (`127.0.0.1`),
- require a session token (capability string) to connect,
- optionally enforce an Origin allowlist for browser clients.

## Lessons from React DevTools (what to copy, not what to clone)

The React DevTools architecture is a good reference model for keeping “live devtools” fast and usable:

- **Minimize bridge traffic**:
  - send only the minimum data required to render the tree,
  - request expensive details (props/state) on-demand when a node is selected.
- **Use patch/operations streams** instead of resending full trees.
- **Poll selected details** at a low frequency rather than pushing every update (reduces churn during scroll).
- **Dehydrate large values** and fill in deep paths on-demand (avoid expensive serialization).
- **Virtualize large trees** (flattened list with depth/weight).

For Fret DevTools v1, we should adopt the same principles:

- live transport should prioritize small, frequent messages (tree skeleton + hover/focus summaries),
- full evidence remains bundle-based (`capture_bundle` + offline viewer),
- detailed per-node debug data should be “inspect-on-demand”, not “stream everything”.

## Protocol outline (WS + MCP)

This section defines the *transport protocol* used by WebSocket and MCP adapters.

Note: script files already have their own `schema_version` (v1/v2) as documented in
`docs/ui-diagnostics-and-scripted-tests.md`. Do not conflate script schema versions with transport protocol versions.

### Message envelope

All messages are JSON objects with a stable envelope:

- `schema_version`: protocol version (start at `1`).
- `type`: string message kind (e.g. `hello`, `inspect.set`, `script.run`).
- `session_id`: string identifier (optional in the first `hello`).
- `request_id`: u64 (optional; required for request/response pairs).
- `payload`: message-specific object.

Rules:

- Requests that expect a response must include `request_id`.
- Responses must echo the same `request_id`.
- Push events must omit `request_id` (or set it to null).
- Unknown `type` must be ignored (or answered with `error.unsupported_type`).

### Handshake and capability negotiation

The first client message must be `hello`:

- client provides:
  - `client_kind`: `native_app` | `web_app` | `tooling`,
  - `client_version`: semver-ish string (best effort),
  - `capabilities`: feature flags (e.g. `inspect`, `pick`, `scripts`, `screenshots`, `bundles`).
- server responds with `hello_ack` including:
  - `server_version`,
  - server capabilities,
  - auth/limits (max message size, rate hints).

Session assignment rules:

- When `client_kind` is `native_app` or `web_app`, the server assigns a `session_id` and includes it in the `hello_ack` envelope.
- When `client_kind` is `tooling`, the server does not assign an app session. Tooling must target an app by sending commands with `session_id` set.
- After a `tooling` hello, the server sends an initial `session.list` to establish the set of active sessions.

### Sessions (multi-app routing)

`session_id` identifies a single connected app instance (native or web).

The server is responsible for:

- assigning unique `session_id` values to app connections,
- publishing session lifecycle events to tooling clients,
- routing messages so that apps only receive messages for their own session.

Tooling clients must:

- show the current session in the UI (and allow switching),
- include `session_id` on all app-directed commands (`inspect.set`, `pick.arm`, `script.run`, `bundle.dump`, etc.),
- filter session-scoped events (e.g. `pick.result`, `script.result`, `bundle.dumped`) to the selected session by default.

Session discovery messages (server -> tooling):

- `session.list`: `{ "sessions": [ { "session_id", "client_kind", "client_version", "capabilities" } ] }`
- `session.added`: `{ "session": { ... } }`
- `session.removed`: `{ "session_id": "..." }`

### Error model

Errors are normal messages (never out-of-band):

- `type: "error"`
- `payload` includes:
  - `code`: stable string (e.g. `error.unauthorized`, `error.timeout`, `error.invalid_payload`),
  - `message`: human-readable,
  - `details`: optional structured JSON.

### Minimal command set (v1)

These commands must map directly to existing in-app operations (no new semantics):

- `inspect.set` / `inspect.status`
- `pick.arm` / `pick.wait` (or `pick.once`) -> emits `pick.result` event
- `script.push` / `script.run` -> emits `script.progress` events + final `script.result`
- `bundle.dump` -> emits `bundle.dumped` event (includes bundle id/path/handle)
- `screenshot.request` -> emits `screenshot.result` event
- `semantics.node.get` -> emits `semantics.node.get_ack` (on-demand node details from the latest semantics snapshot)

#### `screenshot.request` / `screenshot.result` (v1)

This is a convenience command for capturing a renderer screenshot outside of a script run.

- Request payload:
  - `label` (optional): bundle dump label (ensures a fresh bundle exists).
  - `timeout_frames` (optional, default `300`): how long the app waits for the runner to complete the capture.
  - `window` / `window_ffi` (optional): target a specific app window; if omitted, the first active window is used.
- Response event (`screenshot.result`):
  - `status`: `completed` | `timeout` | `disabled` | `unsupported` | `failed`
  - `request_id`: a stable string (also echoed in `screenshots.result.json` entries)
  - `entry`: best-effort completed entry parsed from `screenshots.result.json` (native only)

Web runner note: as of 2026-02-07, screenshot readback is runner-owned and only implemented for the native runner.
On wasm/web targets, `screenshot.result` returns `status=unsupported` with `reason=screenshots_not_supported_wasm`.

#### `semantics.node.get` / `semantics.node.get_ack` (v1)

This is a low-traffic, on-demand detail fetch for a single semantics node. It is intended to back
the DevTools inspector without requiring frequent full bundle dumps.

- Request payload:
  - `window`: app window ffi id.
  - `node_id`: semantics node id (ffi).
- Response event (`semantics.node.get_ack`):
  - `status`: `ok` | `not_found` | `no_semantics`
  - `semantics_fingerprint`: best-effort fingerprint of the semantics snapshot used.
  - `node`: JSON object for the selected node (shape is the in-app `UiSemanticsNodeV1`).
  - `children`: child node ids (best-effort)

Tooling-side-only operations (do not require app support):

- `pack.create` (zip a bundle + `_root/` artifacts when available)
- `bundle.latest` (native filesystem transport only)
- `bundle.compare` / `bundle.stats` / `gates.run`

### Live inspect payloads (keep minimal)

For real-time UX, we push only the minimum stable summary required by the UI:

- `inspect.hover`:
  - window id, node id, selector JSON string, role, optional test_id, bounds, z/root hints.
- `inspect.focus`:
  - same shape as hover, plus focus-specific flags.
- `overlay.summary`:
  - barrier root id, count of blocking roots, topmost interactive root hints.

Implementation note (2026-05-15): the concrete v1 structs live in `crates/fret-diag-protocol`.
`inspect.hover` and `inspect.focus` carry an `overlay_hook` in window-local logical pixels so the
GUI can project a viewport overlay without asking the runtime to draw into the target app. The
runtime sends these payloads only when their stable JSON changes, keeping hover traffic lossy and
bounded under normal pointer movement.

Full evidence remains bundle-based (`capture_bundle`).

## UX model (v1)

DevTools is organized around the author’s loop:

1) **Inspect**
   - Toggle inspect, arm pick, show selector/path/role/test_id, copy JSON.
2) **Scripts**
   - Script library (`tools/diag-scripts`), fork/new, schema validation, pick-to-fill targets.
3) **Run**
   - Run script/suite, show per-step progress, failures, gates, and evidence file links.
4) **Artifacts**
   - Latest bundles, pack zip, open bundle viewer, show screenshots + triage.

The GUI should treat `test_id` as the primary “stable handle” for scripts, and guide users to add `test_id`
at recipe/component authoring time (`ecosystem/*`) when selectors are unstable.

## Refactor plan (high level)

1. Extract `apps/fretboard` diag CLI into `crates/fret-diag` (CLI becomes a thin wrapper).
2. Extract script + selector + result types into `crates/fret-diag-protocol` and reuse them in:
   - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
   - `crates/fret-diag`
   - `apps/fret-devtools`
3. Add a WS transport:
   - server in `apps/fret-devtools` (or a shared library),
   - client/bridge in `fret-bootstrap` diagnostics service (native + wasm32).
4. Keep filesystem transport fully working and deterministic.

## Resolved v1 defaults (2026-02-07)

These defaults are chosen to get to “web runner works” quickly while keeping the contract surface small.
They can evolve, but treat them as sticky unless we have strong evidence.

1. **WS topology**: DevTools GUI hosts a local WS server (loopback-only).
2. **Port discovery**: default `7331`, override via env (`FRET_DEVTOOLS_WS_PORT`) or explicit URL.
3. **Auth**: a single per-session capability token is required on connect:
   - env on native: `FRET_DEVTOOLS_TOKEN`,
   - query string on web: `?fret_devtools_token=...`.
4. **Web runner configuration**:
   - primary: query string `?fret_devtools_ws=ws://127.0.0.1:7331&fret_devtools_token=...`,
   - hash-routing friendly: also accept query params from `location.hash` (e.g. `#/route?fret_devtools_ws=...`),
   - optional override: `window.__FRET_DEVTOOLS_WS` / `window.__FRET_DEVTOOLS_TOKEN` globals for dev servers.
5. **Protocol naming**:
   - message `type`: dot-separated (`inspect.set`, `script.run`, `bundle.dumped`),
   - envelope: `DiagTransportMessageV1` in `crates/fret-diag-protocol`.
6. **Limits / backpressure**:
   - max message size (soft): 4 MiB,
   - hover/focus updates are lossy under load (drop intermediate hover events).
7. **Tree strategy (live)**:
   - default left panel: semantics tree,
   - secondary left-panel views: semantics-derived layout bounds and element identity,
   - start with JSON messages, add “operations” patches later if needed.
8. **Artifact storage (web runner)**:
   - in-memory store + “download zip” export in v1,
   - optional IndexedDB is deferred.
9. **MCP integration**:
   - add a dedicated headless MCP server (`apps/fret-devtools-mcp`) using `rmcp` (stdio first),
   - keep the tool surface small and map 1:1 to existing CLI/GUI operations.

## Open questions (remaining)

1. **Transport evolution**: do we later add HTTP endpoints for large artifact download, or keep WS-only + zip export?
2. **Multi-session UX**: how DevTools chooses a session/window when multiple apps connect.
3. **Binary encoding**: when (if ever) to add a binary framing for perf-heavy payloads.
