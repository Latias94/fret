---
title: Diagnostics DevTools GUI v1 (TODO)
status: draft
date: 2026-02-07
scope: diagnostics, automation, devtools, web-runner
---

# Diagnostics DevTools GUI v1 (TODO)

This file tracks milestones and executable tasks for `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`.

Conventions:

- Prefer “extract and reuse” over re-implementing logic inside the GUI.
- Every milestone should end with a runnable demo path (native and web when applicable).

## Milestones

### M0: Scaffolding + decisions (docs + contracts)

- [x] Add this workstream doc + TODO tracker.
- [x] Decide WS topology for web runner support:
  - [x] DevTools hosts local WS server (recommended).
  - [x] Session token defaults (origin allowlist is implementation-time polish).
- [x] Decide the initial protocol framing:
  - [x] `{"schema_version":1,"type":"...","request_id":...,"payload":...}` (or similar).
  - [x] Correlation rules for request/response vs push events.
- [x] Decide protocol naming and limits:
  - [x] env var + query string keys for web runner.
  - [x] message type naming convention (`inspect.set` vs `inspect_set`).
  - [x] max message size + hover event backpressure rules.
- [x] Decide the default tree shown in the left panel:
  - [x] semantics tree (recommended default),
  - [x] layout tree (debugging layout engine),
        Evidence: `apps/fret-devtools` now exposes a secondary `Layout` tab in the left Inspect
        Workspace. It is a semantics-derived layout-bounds view over the existing bundle/live
        semantics cache, not a full native layout-engine snapshot. Guarded by
        `cargo nextest run -p fret-devtools compute_rows_search_matches_id_parent_and_bounds secondary_tree_labels_surface_layout_and_identity_fields --no-fail-fast`
        and `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
  - [x] element tree (authoring identity / caching boundaries).
        Evidence: `apps/fret-devtools` now exposes a secondary `Elements` tab in the left Inspect
        Workspace. It is a semantics-derived identity/relationship view (`sem_node`, parent,
        `test_id`, `labelled_by`, `described_by`, `controls`) over the existing cache, not a full
        declarative runtime element snapshot. Guarded by the same focused nextest and first-open
        source gate.

### M1: Extract reusable tooling into crates

- [x] Create `crates/fret-diag-protocol` (wasm32-compatible, no std::fs required).
  - [x] Move script/selector/predicate types into it (schema v1/v2).
  - [x] Add serde roundtrip tests for representative scripts from `tools/diag-scripts/`.
- [x] Create `crates/fret-diag` and move core logic from `apps/fretboard` diag CLI into it:
  - [x] pack/share helpers (zip + `_root/` artifacts),
  - [x] bundle stats + gates + compare,
  - [x] file-trigger helpers (touch/write/wait) for the existing transport.
- [x] Make `apps/fretboard` depend on `crates/fret-diag` and keep CLI behavior identical.

### M2: New GUI app skeleton (`apps/fret-devtools`)

- [x] Add `apps/fret-devtools` (native target first, but structured for web runner).
- [x] Implement the minimal 4-panel UX:
  - [x] Inspect (inspect toggle + pick + show selector JSON),
  - [x] Run (run a script; show progress + failures),
  - [x] Artifacts (latest bundle dump payload),
  - [x] Scripts (browse `tools/diag-scripts` + open editor).
  - [x] Semantics tree (virtualized via `VirtualList`; keep selection visible; selected-node inspector).
  - [x] WS message tail (basic event log).
- [x] Add a “watch” loop for `FRET_DIAG_DIR` updates (native transport):
  - [x] auto-refresh latest bundle,
  - [x] auto-refresh `pick.result.json`, `script.result.json`, screenshot results.

### M3: Script Studio (authoring UX)

- [x] Schema-aware script editor:
  - [x] validate schema version (v1/v2) on push/run,
  - [x] step palette (v1 steps + v2 intent steps),
  - [x] structured editor for selector/predicate.
- [x] Pick-to-fill UX:
  - [x] select a JSON pointer (e.g. `/steps/3/target`) and apply pick result into the editor (equivalent of `diag pick-apply`).
  - [x] discover pointer candidates from the current script (faster pointer targeting).
- [x] Script library ergonomics:
  - [x] fork/copy a script into `.fret/diag/scripts/` (avoid editing workspace scripts by default),
  - [x] “Run this script” can produce a shareable zip (Run & Pack / Pack last bundle).

### M4: WebSocket transport (enables web runner)

- [x] Implement WS server (DevTools side):
  - [x] binds to `127.0.0.1`,
  - [x] requires a capability token,
  - [x] supports multiple clients (session ids).
- [x] Implement WS client bridge in diagnostics service:
  - [x] add `FRET_DEVTOOLS_WS=ws://127.0.0.1:<port>` (name TBD) to enable it,
  - [x] wasm32 client via `web_sys::WebSocket`,
  - [x] native client via a non-blocking reader thread + queue (avoid blocking the frame loop).
  - [x] web runner config: support query string and/or `window.__FRET_DEVTOOLS_WS` globals (name TBD).
- [x] Map protocol commands to existing in-app operations:
  - [x] inspect config updates,
  - [x] pick arm + pick result,
  - [x] script push + script progress + script result,
  - [x] bundle dump,
  - [x] screenshot request.
  - [x] semantics node details on-demand (`semantics.node.get` / `semantics.node.get_ack`).

### M5: Artifacts for web runner

- [x] Define an artifact store abstraction for diagnostics outputs:
  - [x] native: filesystem (existing),
  - [x] web: in-memory export helpers (zip bytes + materialize to exports dir).
- [x] Allow DevTools to pack web runner dumps by materializing `bundle.dumped.bundle` into `.fret/diag/exports/`.
- [x] Ensure the offline bundle viewer can open zips produced by web runs (same structure as `diag pack`).

### M6: Quality gates + “real-time inspect” polish

- [x] First-class UI for gates:
  - [x] First-open `Gate Commands` block for existing stale paint/scene, pixels-changed,
        perf-threshold, and resource-footprint diagnostics command templates.
  - [x] Shared `fret-diag` gate profile projection owns gate ids, command templates, evidence
        files, and notes; the GUI now renders that projection instead of owning the taxonomy.
  - [x] GUI `Gate Commands` profile rows expose a `Copy command` action for each shared profile,
        turning the first-open gate taxonomy into an explicit per-profile action surface.
  - [x] Shared script-target gate command builder parameterizes stale paint/scene and
        pixels-changed profiles from `script.json` + `test-id`; the GUI exposes profile selection,
        inputs, command preview, and `Copy generated command` without making the GUI own command
        templates.
  - [x] Script-target gate projection now includes structured `diag_args` plus `missing_inputs`,
        giving the GUI a runnable contract for the next launch/run slice without parsing shell
        command strings.
  - [x] GUI script-target gate builder can launch the generated stale paint/scene or
        pixels-changed gate command through the shared diagnostics engine and writes a lightweight
        `.fret/diag/gate-runs/*.json` result artifact.
  - [x] GUI generated gate results now keep a bounded selectable history with details, summary,
        raw JSON preview, copy actions, and platform URL open support for the selected artifact.
  - [x] Selected-summary follow-up commands generated from the selected `bundle_dir`, covering
        stats, layout perf, memory, triage, hotspots, trace, visual compare, and footprint compare.
  - [x] Structured follow-up command projection separates bundle-local runnable commands from
        baseline-required manual compare commands for GUI and MCP consumers.
  - [x] GUI selected-summary inspector can launch bundle-local runnable follow-ups and records
        in-flight/error status without treating baseline-required compare commands as runnable.
  - [x] GUI selected-summary inspector can launch the selected-bundle `trace` follow-up through the
        same shared diagnostics runner as stats, layout, memory, triage, and hotspots.
  - [x] GUI-launched follow-ups write `.fret/diag/followups/*.json` result records and expose the
        latest result path for copying.
  - [x] Trace follow-up result records include `output_artifacts[].path` for
        `trace.chrome.json`, and the selected-summary summary/details blocks surface that artifact.
  - [x] GUI selected-summary inspector mirrors the latest selected-bundle follow-up result JSON
        inline for pass/fail/error/timing triage.
  - [x] GUI selected-summary inspector shows a structured selected-bundle follow-up result summary
        above raw JSON for status, command, duration, and error preview.
  - [x] GUI selected-summary inspector keeps a bounded follow-up result history filtered to the
        selected bundle, preventing stale global-last results from masquerading as current evidence.
  - [x] GUI selected-summary inspector renders selected-bundle follow-up history as selectable
        result entries that switch the summary/raw JSON/copy target.
  - [x] GUI selected-summary inspector shows selected follow-up result details and can copy the
        exact command that produced the selected artifact.
  - [x] GUI selected-summary inspector can open the selected follow-up JSON artifact through the
        platform URL handler.
  - [x] Copying a follow-up result path uses the selected bundle's latest history entry instead of
        the global last result artifact.
  - [x] Copying the selected bundle's follow-up JSON is available from the same inspector.
  - [x] Selected trace follow-up artifacts can be copied or opened directly from the inspector,
        with relative `trace.chrome.json` paths resolved against the repo root before copy/open.
  - [x] stale paint/scene launch/run + result artifact history,
  - [x] pixels changed launch/run + result artifact history,
  - [x] perf thresholds,
        Evidence: the generated perf-threshold gate form now uses shared
        `fret-diag` product-chain docking defaults for `perf-docking-arbitration-steady`, including
        repeat/warmup/aggregate run knobs and the full CPU/layout/pointer/renderer threshold flag
        set mirrored from `tools/diag_gate_imui_product_chain.py`. Guarded by
        `cargo nextest run -p fret-diag devtools_gate_perf_threshold_command_preserves_placeholders_until_filled devtools_gate_perf_threshold_command_includes_runnable_diag_args devtools_gate_perf_threshold_command_quotes_target_and_rejects_invalid_numbers devtools_gate_perf_threshold_product_chain_defaults_are_runnable --no-fail-fast`,
        `cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast`,
        and `python tools/diag_gate_imui_product_chain.py --only discovery --reuse-built`.
  - [x] resource footprint thresholds.
- [x] Live inspect payloads (keep minimal):
  - [x] hover events (`inspect.hover`) with node id + selector JSON + bounds,
  - [x] focus events (`inspect.focus`) with summary + path (best-effort),
  - [x] hovered node bounds + viewport overlay hooks,
  - [x] overlay barrier root id + blocking roots summary.
        Evidence: IMUI product-closure
        `docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md`
        "DevTools live inspect overlay payload closure - 2026-05-15 follow-up"; focused gates:
        `cargo nextest run -p fret-diag-protocol live_inspect_payloads_roundtrip_bounds_and_overlay_summary --no-fail-fast`,
        `cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics-ws" inspect_node_summary_v1_includes_bounds_and_root_hint overlay_summary_v1_reports_barrier_and_blocking_roots --no-fail-fast`, and
        `cargo nextest run -p fret-devtools inspect_hover_bounds_lines_project_bounds_and_selector inspect_hover_bounds_lines_missing_bounds_returns_none inspect_overlay_hook_lines_project_overlay_summary --no-fail-fast`.
- [x] Add at least one “dogfood” demo workflow:
  - [x] open UI gallery, pick a button, generate a script, run it, pack zip, open viewer.
        The GUI first-open shell now exposes the `ui-gallery-button-dogfood` path with the
        canonical UI gallery launch command, stable button selector, `pick-script` /
        `pick-apply` commands, `diag run --pack`, `diag pack`, and offline bundle viewer command.
        Source gates:
        `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only` and
        `python tools/diag_gate_imui_product_chain.py --only discovery`.
- [x] Validate tree scalability:
  - [x] virtualized rendering for 50k+ semantics nodes.
        The Semantics tab keeps `VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16)`;
        row projection is now iterative rather than recursive, with 50k flat, 50k deep, and
        large-search focused tests.
  - [x] low-traffic live updates (operations/polling) under scroll.
        Live selected-node details still use on-demand `semantics.node.get` and
        `hit_test.explain`; request eligibility is isolated in `live_semantics_request_decision`
        and tested to throttle unchanged selection to 1Hz while allowing selection changes and
        manual refreshes.

### M7: MCP server adapter (AI-friendly tooling API)

- [x] Add `apps/fret-devtools-mcp` using `rmcp` (stdio transport first).
- [x] Expose a minimal tool set (names TBD):
  - [x] `fret_diag_sessions_list` (list sessions),
  - [x] `fret_diag_sessions_select` (select a session),
  - [x] `fret_diag_connect` (choose transport: filesystem vs WS; optional),
  - [x] `fret_diag_inspect_set` (on/off/toggle + consume_clicks),
  - [x] `fret_diag_pick` (arm + wait + return selector JSON),
  - [x] `fret_diag_scripts_list` (list `tools/diag-scripts` and `.fret/diag/scripts`),
  - [x] `fret_diag_run_script_file` (run a script by file name or relative path),
  - [x] `fret_diag_run` (run multiple scripts with list/glob; returns structured summary),
  - [x] `fret_diag_run_script_json` (minimal: run a v1/v2 script JSON and wait for pass/fail),
  - [x] `fret_diag_pack_last_bundle` (dump + pack into zip),
  - [x] `fret_diag_pack_last_bundle_zip_bytes` (dump + pack bundle.json zip bytes as base64),
  - [x] `fret_diag_bundle_dump_latest` (best-effort latest bundle.dumped payload),
  - [x] `fret_diag_compare`.
- [x] (Optional) Expose key artifacts as resources:
  - [x] latest `bundle.json`,
  - [x] `repro.summary.json` (when present on disk),
  - [x] `bundle.zip` (generated on read; same layout as `diag pack`).
- [x] Support resource subscriptions + notifications for artifact updates.
- [x] Add an end-to-end AI scenario doc:
  - [x] “Pick selector → patch script → run → pack → open viewer” driven via MCP tools.
        Evidence: `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md`
        now participates in
        `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`
        through the `devtools mcp ai scenario doc` check, which validates the scenario steps,
        artifact resources, subscriptions, and matching `apps/fret-devtools-mcp` tool/resource
        implementation anchors.
  - [x] `fret_diag_regression_dashboard` exposes structured follow-up command rows with
        `diag_args` and baseline classification, alongside the existing command-line strings.

## Cross-cutting hygiene

- [x] Keep `bundle.json` forward-compatible (unknown fields ignored by viewer).
      Evidence: `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`
      now runs the `devtools cross-cutting hygiene` check, which validates the DevTools protocol
      forward-compatibility doc, the bundle viewer best-effort parser/zip input path, and the
      viewer README forward-compatibility note.
- [x] Keep `fret-ui` policy-free; DevTools policy stays in `ecosystem/*` and apps/tooling.
      Evidence: the same `devtools cross-cutting hygiene` check validates the `fret-ui` README
      mechanism-layer boundary and fails if DevTools-specific policy markers appear in
      `crates/fret-ui/src`.
- [x] Prefer authoring `test_id` in recipes to make scripts stable.
      Evidence: the same gate validates the DevTools authoring-loop guidance, the GUI default
      selector kind, the `test_id` selector option, the UI-gallery preferred selector, and the
      gated `devtools.gate.test_id` input.
- [x] Keep the first-open shell summary-first instead of a raw command wall.
      Maintenance: `apps/fret-devtools/src/native.rs` now renders a `First-open Next Actions`
      summary in the shell header, defaults `Evidence & Results` to `Guide`, and keeps the full
      first-open, dogfood, demo/metrics/debug, and gate-command references inside that Guide tab.
      Source and discovery gates now cover that posture through
      `devtools_first_open_next_action_lines_prioritize_stateful_workflow` and
      `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
