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
  - [x] Generated gate result startup restores recent valid `.fret/diag/gate-runs/*.json`
        records, skipping malformed and non-gate JSON so the history survives a DevTools restart.
  - [x] GUI Workflow Runs panel for first-class campaign/suite execution:
        `devtools-first-open-smoke` campaign validation, `imui-p3-multiwindow-parity` campaign
        validation, and selected-session `perf-docking-arbitration-steady` suite execution now run
        through shared `fret_diag::diag_cmd` and write `.fret/diag/workflow-runs/*.json` records.
        Stored command previews and result JSON redact `--devtools-token`.
  - [x] Workflow Runs startup restores recent valid `.fret/diag/workflow-runs/*.json` records,
        skipping malformed and non-workflow JSON, so result history survives a DevTools restart.
  - [x] Workflow suite result records now surface shared evidence outputs:
        selected-session `diag suite ... --dir <out>` runs project `suite.summary.json` and
        `regression.summary.json` through `output_artifacts[]`, and the Workflow Result Summary /
        Details blocks render those artifact paths for handoff.
  - [x] Workflow suite `suite.summary.json` artifacts can be copied or opened directly from
        Workflow Result Details, matching the regression-summary and trace-artifact handoff pattern.
  - [x] Workflow suite `regression.summary.json` artifacts can be copied or opened directly from
        Workflow Result Details, with relative artifact paths resolved against the repo root before
        platform URL opening.
  - [x] Workflow suite `regression.summary.json` artifacts can be loaded into the existing
        Regression Workspace selection, reusing shared drill-down, follow-up, perf evidence, and
        capability provenance surfaces instead of a workflow-private inspector.
  - [x] Workflow Runs exposes `Workflow Handoff Readiness`, a compact next-action projection that
        tells maintainers when to run the workflow, load the workflow `regression.summary.json`, or
        move into Regression Workspace follow-up actions.
  - [x] Workflow Runs exposes `Workflow Summarize Handoff`, deriving a shared
        `diag summarize <regression.summary.json> --dir <same-dir> --json` command from the
        selected suite result so `regression.index.json` generation is explicit when the suite run
        itself only produced `regression.summary.json`.
  - [x] Workflow summarize result records now include `regression.summary.json` and
        `regression.index.json` in `output_artifacts[]`, so result details and summaries can expose
        the aggregate handoff artifacts directly.
  - [x] Workflow `regression.index.json` artifacts can be loaded into Regression Workspace by
        pointing the shared aggregate refresh at the index parent directory, keeping the Workflow
        Runs panel as a thin artifact handoff instead of a private aggregate parser.
  - [x] Workflow `regression.index.json` artifacts can also be copied or opened directly from the
        same Workflow Result Details action row once the aggregate index exists.
  - [x] Workflow Handoff Readiness now reports `aggregate_index_loaded` and an
        `aggregate_next_action`, so a ready `regression.index.json` is distinguishable from an
        aggregate workspace that has actually been loaded from the workflow artifact root.
  - [x] First-open next actions distinguish aggregate regression index readiness from a loaded
        selected summary, so workflow-suite handoff does not misreport single-summary evidence as a
        full aggregate load.
  - [x] Regression Workspace exposes a compact `Follow-up Readiness` block for the selected
        summary, including selected bundle count, runnable/manual follow-up counts, and the first
        runnable command.
  - [x] First-open next actions report when the selected summary already has a selected-bundle
        follow-up result loaded, pointing maintainers to Follow-up Result Summary/History.
  - [x] Selected-summary follow-up commands generated from the selected `bundle_dir`, covering
        stats, layout perf, memory, triage, hotspots, trace, visual compare, and footprint compare.
  - [x] Structured follow-up command projection separates bundle-local runnable commands from
        baseline-required manual compare commands for GUI and MCP consumers.
  - [x] GUI selected-summary inspector can launch bundle-local runnable follow-ups and records
        in-flight/error status without treating baseline-required compare commands as runnable.
  - [x] GUI selected-summary inspector can materialize baseline-required visual and footprint
        compare templates once the user provides a baseline bundle/directory or footprint session,
        then launches them through the same follow-up runner and records the candidate bundle in the
        selected-bundle result history.
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
  - [x] Follow-up result startup restores recent valid `.fret/diag/followups/*.json` records into
        the bounded history while preserving selected-bundle filtering in the Regression Workspace.
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
- [x] Make first-open multi-session targeting explicit.
      Maintenance: `First-open Next Actions` now includes a session-scope line. It distinguishes
      no connected session, connected-but-unselected session choices, one selected session, and
      multiple connected sessions where the Session selector retargets inspect, bundle,
      screenshot, and selected-session suite actions. This documents the current DevTools
      selection model in the GUI without changing the WebSocket transport or adding a new
      multi-session policy layer. The underlying v1 selection rule is now unit-tested in `ws.rs`:
      keep a valid selected session, otherwise fall back to the first advertised session, and filter
      session-scoped payloads to that selection. Source and discovery gates cover this through
      `selected_session_refresh_keeps_valid_selection_or_falls_back_to_first_session`,
      `message_session_matching_uses_selected_session_when_present`,
      `devtools_first_open_next_action_lines_prioritize_stateful_workflow`, and
      `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
- [x] Keep recent GUI-launched evidence visible on first open.
      Maintenance: the Guide tab now starts with `Recent Evidence`, a compact restored-history
      projection over generated gate, workflow, and selected-bundle follow-up result artifacts. It
      surfaces the latest artifact in each lane, counts recent failing evidence, selects the newest
      failed result across lanes from result JSON timestamps first and timestamped path prefixes as
      a compatibility fallback, includes the full failed artifact path plus failed follow-up
      `bundle_dir` in the copied report, and gives a next-action hint without introducing a new
      artifact schema. The block now also exposes
      select/copy/open actions for the compact report, first failed evidence artifact, and its
      producing command, plus direct copy of the failed follow-up bundle directory when present, so
      a first-open maintainer can jump to the existing Gate/Workflow/Follow-up history state, copy
      the rerun command, rerun failed evidence when the result JSON has runnable structured
      `diag_args`, copy the restored result JSON payload, or open the failure JSON without manually
      matching artifact paths across panels. Workflow reruns always re-materialize the same
      workflow id from the current selected session and current token instead of trusting stored
      workflow `diag_args`; missing args, unknown workflow ids, or missing session keep rerun
      disabled while surfacing the concrete unavailable reason, so the GUI never executes
      display-only shell strings. The compact report and the
      button use the same state-aware rerun decision, and the compact next-action line now points
      at the concrete repair step: rerun, select a session, refresh workflow commands, run a current
      workflow, or inspect result JSON. Copied first-open evidence therefore stays aligned with what
      the GUI can actually run.
      `First-open Next Actions` also reports restored failed evidence, its rerun command, current
      rerun availability or unavailable reason, and the same concrete `recent evidence next` repair
      step in the shell header, before the maintainer opens the Guide. The header now also mirrors
      the Guide's copy/select/rerun recent-evidence actions as first-open shortcut buttons, using
      the same command ids and disabled-state rules.
      Source and discovery gates cover this through
      `devtools_recent_evidence_lines_surface_restored_histories`,
      `recent_evidence_status_failed_ignores_empty_placeholder_and_passed_case`,
      `first_open_recent_evidence_action_specs_gate_disabled_states`,
      `recent_evidence_next_action_projects_rerun_and_repair_steps`,
      `devtools_recent_evidence_lines_use_current_workflow_state_for_rerunnable_status`,
      `devtools_recent_evidence_lines_surface_failed_followup_bundle_dir`,
      `recent_failed_evidence_bundle_dir_filters_empty_bundle_dir`,
      `recent_failed_evidence_rerun_command_uses_structured_diag_args`,
      `recent_failed_evidence_rerun_command_rejects_redacted_workflow_args`,
      `recent_failed_evidence_rerun_reason_reports_diag_args_issues`,
      `recent_failed_evidence_rerun_command_recovers_redacted_workflow_from_current_state`,
      `recent_failed_evidence_rerun_command_uses_current_workflow_state_over_stored_args`,
      `recent_failed_evidence_rerun_reason_reports_unregistered_workflow`,
      `recent_failed_evidence_rerun_command_projects_followup_bundle`,
      `devtools_recent_failed_evidence_target_prefers_visible_latest_then_history`,
      `devtools_recent_failed_evidence_target_falls_back_to_lane_order_without_timestamps`,
      `devtools_recent_failed_evidence_target_prefers_result_json_time_over_path_time`,
      `devtools_recent_failed_evidence_target_carries_result_json_payload`,
      `load_recent_gate_run_result_history_prefers_record_time_over_file_mtime`,
      `load_recent_workflow_run_result_history_prefers_record_time_over_file_mtime`,
      `load_recent_followup_result_history_prefers_record_time_over_file_mtime`,
      `devtools_recent_evidence_selection_effect_routes_to_existing_history_state`, and
      `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
      MCP parity: `fret_diag_recent_evidence` now provides a read-only MCP projection over the same
      `.fret/diag/gate-runs`, `.fret/diag/workflow-runs`, and `.fret/diag/followups` records, and
      `fret-diag://first-open.md` points AI clients at that bridge. The same projection is also
      exposed as the sessionless `fret-diag://recent-evidence.json` resource, so MCP clients can
      discover restored GUI-launched evidence from the resource list without adding a MCP-private
      rerun model. It also selects the newest failed result across lanes from result JSON
      `finished_unix_ms` / `started_unix_ms`, falling back to timestamped result paths for older
      records, and uses the same status normalization as the GUI so empty status, `-`, and
      case-varied `passed` values are not treated as failures. The list/template source is locked
      through `sessionless_resource_specs()` so `first-open.md` and `recent-evidence.json` stay
      discoverable together. Workflow reruns still require the GUI's current selected session/token
      state.
      Source and discovery gates cover this through
      `build_recent_evidence_report_reads_gui_result_records`,
      `recent_evidence_status_is_failing_ignores_empty_placeholder_and_passed_case`,
      `recent_evidence_resource_text_matches_report_shape`,
      `build_recent_evidence_report_prefers_latest_failed_result_across_lanes`,
      `load_recent_evidence_entries_prefers_record_time_over_file_mtime`,
      `sessionless_resource_specs_include_first_open_and_recent_evidence`,
      `parse_resource_uri_accepts_recent_evidence_resource`,
      `mcp_first_open_resource_text_surfaces_imui_product_chain`, and
      `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
