# ImUi Editor-Grade Product Closure v1 - TODO

Status: maintenance umbrella lane
Last updated: 2026-04-28

Status note (2026-04-22): keep phase ordering and follow-on decisions here. Do not resume
implementation-heavy work in this folder while the closed child-region depth closeout record lives
in `docs/workstreams/imui-child-region-depth-v1/` and the remaining P3 execution continues in
`docs/workstreams/docking-multiwindow-imgui-parity/`.

Status note (2026-05-14): the workspace shell tab-strip evidence now has a fresh launched gate
set, but this folder remains an umbrella maintenance lane. Keep implementation-heavy follow-ons in
their narrower owners and record new shell evidence here only as proof-state refresh, not as a lane
reopen.

## Lane setup

- [x] Create the lane and record why the older `imui` closeout folders stay closed.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, and
  `docs/todo-tracker.md`.
- [x] Keep the lane narrow: start a dedicated follow-on once a phase becomes implementation-heavy.
      Result: `docs/workstreams/imui-response-status-lifecycle-v1/` now proves this rule for the
      implementation-heavy `ResponseExt` lifecycle vocabulary slice.
- [x] Demote this folder from active execution to umbrella maintenance once the implementation-heavy
      phases moved into narrower lanes.
      Result: this folder now records phase ordering and cross-phase status, while the narrow P0/P1
      closeout records stay closed and the remaining active P3 execution continues in
      `docs/workstreams/docking-multiwindow-imgui-parity/`.
- [x] Close the second proof-surface follow-on without widening shared IMUI helpers.
      Result: `docs/workstreams/imui-collection-second-proof-surface-v1/` now records the closed
      second-proof-surface follow-on, lands the `Scene collection` left-rail surface in
      `editor_notes_demo.rs`, and records that it does not yet prove that both collection proof surfaces
      need the same shared helper.

## P0 - Default authoring lane closure

- [x] Inventory the current first-party teaching surfaces that imply the default immediate path.
      Result: `P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md` with a bounded
      golden/reference/historical table.
- [x] Pick the smallest second proof surface beyond `apps/fret-examples/src/imui_editor_proof_demo.rs`
      that should teach the golden path.
      Result: `apps/fret-cookbook/examples/imui_action_basics.rs`.
- [x] Audit the remaining immediate authoring footguns and separate:
      - documentation/teaching issues,
      - proof-surface selection issues,
      - and genuinely missing helper surface.
      Result: `P0_FOOTGUN_AUDIT_2026-04-12.md`.
- [x] Freeze a demote/delete plan for first-party docs/examples that still imply the wrong layer.
      Result: `P0_DEMOTE_DELETE_PLAN_2026-04-12.md`, public docs/gates now route immediate-mode
      readers through the golden pair and demote `imui_hello_demo` to smoke/reference.
- [x] Freeze the proof budget rule for future `fret-ui-kit::imui` public helper widening.
      Result: `P0_PROOF_BUDGET_RULE_2026-04-12.md` now requires at least two real first-party proof
      surfaces, freezes the current minimum budget as `imui_action_basics` +
      `imui_editor_proof_demo`, and rejects reference/compatibility-only surfaces as sole
      justification.
- [x] Publish the first-open mounting rule for safe-default `imui(...)` versus explicit
      `imui_raw(...)`.
      Result: `P0_ROOT_HOSTING_RULE_2026-04-12.md` and `docs/examples/README.md` now explain the
      safe default for root/non-layout parents versus the advanced explicit-layout seam, without
      reopening helper growth.
- [x] Publish the first-open stable-identity rule for static vs dynamic IMUI collections.
      Result: `P0_STABLE_IDENTITY_RULE_2026-04-12.md` and `docs/examples/README.md` now explain
      when `ui.for_each_unkeyed(...)` is acceptable versus when `ui.for_each_keyed(...)` /
      `ui.id(key, ...)` is the default posture.
- [x] Record the post-shortcut-seam parity status inside the umbrella lane so focused item-local
      shortcuts are no longer treated as the primary P0 blocker.
      Result: `P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md` now records the 2026-04-13 shortcut batch,
      the repeat-semantic test floor, and the narrower remaining P0 backlog.
- [x] Promote a launched first-open authoring proof for the generic/default IMUI path.
      Result: `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
      proves command palette, declarative, GenUI, and IMUI triggers all dispatch the same typed
      action into one view-local state path; `tools/diag_gate_action_first_authoring_v1.py --only
      cookbook-imui-action-basics-cross-frontend` runs that proof without the broader action-first
      gate set.
- [x] Record the current product-workflow coherence review that fixes `imui_hello_demo` package
      selection ambiguity in the source docs.
      Result: `P0_PRODUCT_WORKFLOW_COHERENCE_REVIEW_2026-05-06.md` records the current first-open
      command clarity read and points the docs/indexes at explicit `--package` selection for the
      colliding `imui_hello_demo` binary name.

## P1 - Editor workbench shell closure

- [x] Build one reviewable proof matrix for workspace shell + docking + editor composites.
      Result: `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md` now freezes the current primary proof,
      supporting proofs, and reading order.
- [x] Decide which missing closure belongs in:
      - `ecosystem/fret-workspace`,
      - `ecosystem/fret-docking`,
      - `ecosystem/fret-ui-editor`,
      - or recipe crates.
      Result: `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md` now maps shell slots/tabstrip/command
      scope to `fret-workspace`, docking choreography to `fret-docking`, editor composites to
      `fret-ui-editor`, and scene-local center content to app/recipe ownership.
- [x] Keep shell-level missing pieces out of the generic `imui` backlog by default.
      Result: `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md` now freezes
      `workspace_shell_demo` / `editor_notes_demo` as the shell-first proof order and classifies
      `imui_editor_proof_demo` as supporting docking/editor evidence instead of the default shell
      backlog.
- [x] Promote at least one shell-level diagnostics smoke suite beyond tabstrip-only checks.
      Result: `P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md` now freezes
      `diag-hardening-smoke-workspace` as the promoted P1 shell smoke suite and requires the suite
      minimum to span tab close/reorder/split preview plus dirty-close prompt, Escape focus
      restore, and file-tree keep-alive.

## P2 - Unified diagnostics/devtools surface

- [x] Publish one first-open developer path for:
      inspect -> selector -> script -> bundle -> compare.
      Result: `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md` now freezes a CLI-first
      inspect/pick -> script -> bundle -> compare loop, and keeps DevTools GUI / MCP as thin
      consumers over the same artifacts root and compare semantics.
- [x] Decide what must stay in:
      - `apps/fret-devtools`,
      - `crates/fret-diag`,
      - `ecosystem/fret-bootstrap`,
      - and `apps/fret-devtools-mcp`.
      Result: `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md` now freezes
      `fret-bootstrap` as the in-app runtime/export seam, `fret-diag` as the shared
      orchestration/artifact engine, `fret-devtools` as GUI UX over shared contracts, and
      `fret-devtools-mcp` as the headless automation/resource adapter.
- [x] Add one bounded devtools smoke package that validates the first-open path rather than only
      isolated tooling commands.
      Result: `P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`,
      `tools/diag_gate_imui_p2_devtools_first_open.py`, and
      `tools/diag-campaigns/devtools-first-open-smoke.json` now freeze one repo-owned gate that
      proves direct `diag run` -> named bundles -> latest resolution -> `diag compare`, plus the
      aggregate campaign root -> `diag summarize` -> `regression.summary.json` /
      `regression.index.json` -> `diag dashboard` handoff.
- [x] Stop forcing authors to discover the workflow by hopping across multiple diagnostics notes.
      Result: `P2_DISCOVERABILITY_ENTRY_2026-04-12.md` and `docs/diagnostics-first-open.md` now
      freeze one canonical first-open diagnostics entry, while the existing inspect, bundles/scripts,
      GUI dogfood, and diagnostics-v2 navigation notes are explicitly demoted to branch/reference
      roles instead of competing start pages.
      Maintenance: `apps/fret-devtools/src/native.rs` now surfaces that same first-open evidence
      path directly in the DevTools GUI shell, and
      `tools/diag_gate_imui_p2_devtools_first_open.py` checks the GUI source projection so the
      CLI index does not remain the only discoverability anchor.
      Maintenance: `tools/diag_gate_imui_product_chain.py` now also validates
      `fretboard-dev list tool-apps --json` as the stable DevTools GUI/MCP first-open map, so the
      umbrella product-chain gate catches drift in the machine-readable entrypoint contract. The
      same gate validates `fretboard-dev --help` and `fretboard-dev list --help`, so the tool-apps
      index itself remains discoverable from the first CLI help screens.
      Maintenance: the same `fretboard_tool_apps` JSON now carries a `product_workflows` entry for
      `imui-product-chain`, including the focused discovery command, the launched `perf-docking`
      command, and the expected perf summary/threshold artifacts, so DevTools-style consumers can
      discover the current product-chain evidence path without owning a GUI-private schema.
      Maintenance: `apps/fret-devtools/src/native.rs` now mirrors that
      `imui-product-chain` route in the GUI first-open evidence panel, including the default
      command, focused discovery command, launched `perf-docking` command, suite, docs, and expected
      perf artifacts. This keeps GUI discoverability aligned with `fretboard-dev list tool-apps`
      instead of making the GUI a second source of truth. The default product-chain discovery gate
      now also source-checks that GUI projection.
      Maintenance: `apps/fret-devtools/src/native.rs` now also surfaces a persistent
      `demo-metrics-debug` route in the GUI shell, naming the editor proof/editor notes/device shell
      demos plus `diag stats`, `diag layout-perf-summary`, `diag memory-summary`, `diag triage`,
      `diag hotspots`, and `diag trace` entrypoints. This improves Dear ImGui-style demo/metrics/debug
      discoverability without widening `fret-imui`.
      Maintenance: `fretboard-dev list tool-apps` now also prints the same `demo-metrics-debug`
      route, and `fretboard-dev list tool-apps --json` exposes it under `first_open_routes` with
      grouped demo, metrics, and debug commands. This keeps CLI, GUI, MCP-style consumers, and
      docs on one first-open route vocabulary instead of making the GUI the only productized entry.
      Maintenance: the CLI/JSON route, DevTools GUI guide panel, and MCP `first-open.md` projection
      now include `diag trace <bundle-or-dir> --json` as the trace artifact drill-down next to
      stats/layout/memory/triage/hotspots, keeping perf-attribution handoff visible from the same
      Demo/Metrics/Debug entry.
      Maintenance: the same GUI shell now surfaces a `Gate Commands` block for stale paint/scene,
      pixels-changed, perf-threshold, and resource-footprint diagnostics command templates. This is
      tracked in `docs/workstreams/diag-devtools-gui-v1/`, keeping gate UX in the diagnostics owner
      lane rather than the IMUI runtime.
      Maintenance: the Gate Commands taxonomy now lives in
      `crates/fret-diag/src/devtools_gate_profiles.rs`, and the GUI consumes
      `devtools_gate_profile_lines(...)` rather than owning diagnostics command templates. The
      shared projection includes explicit `check.resource_footprint.json` evidence for thresholded
      resource-footprint gates.
      Maintenance: the DevTools GUI `Gate Commands` profile rows now expose a `Copy command`
      action for each shared profile, making stale/pixels/perf/resource-footprint gate templates
      copyable from structured rows before adding profile-specific run forms.
      Maintenance: `fret-diag` now parameterizes the script-target stale paint/scene and
      pixels-changed gate profiles from `script.json` + `test-id`, and the DevTools GUI exposes a
      selected-profile command builder with preview and `Copy generated command` without moving
      gate templates into the GUI.
      Maintenance: the same script-target gate projection now includes structured `diag_args` and
      `missing_inputs`, giving the next GUI launch/run slice a safe execution contract instead of
      parsing the copied shell command.
      Maintenance: the DevTools GUI script-target gate builder can now launch the generated stale
      paint/scene or pixels-changed command through the shared diagnostics engine and writes
      `.fret/diag/gate-runs/*.json` result artifacts, keeping this as DevTools productization
      rather than `fret-imui` runtime growth.
      Maintenance: generated gate result artifacts now have a bounded selectable GUI history with
      selected-result details, summary, raw JSON, copy actions, and platform URL open support.
      Maintenance: the generated gate builder now supports the `perf-thresholds` profile from a
      shared `fret-diag` structured `diag perf` command projection, so the GUI can copy/run
      target/repeat/warmup/aggregate/threshold commands without parsing shell templates.
      Maintenance: the same generated gate builder now supports `resource-footprint-thresholds`;
      this also repairs the underlying `diag repro` contract so working-set, peak-working-set, and
      CPU-average thresholds are real parsed CLI inputs before the GUI exposes them.
      Maintenance: selected regression summaries now generate concrete `bundle_dir` follow-up
      commands for stats, layout perf, memory, triage, hotspots, trace, visual compare, and
      footprint compare from a shared `fret-diag` projection, reducing GUI-to-CLI friction without
      adding GUI-private diagnostics semantics.
      Maintenance: MCP `fret_diag_regression_dashboard` now consumes that same shared regression
      drill-down/follow-up projection, so AI-driven diagnostics receives the same bundle dirs,
      capability provenance, perf evidence, and concrete next-command hints as the GUI.
      Maintenance: the MCP dashboard now exposes structured follow-up command rows with
      `diag_args`, preserving the runnable/manual split without shell-string parsing.
      Maintenance: the shared follow-up projection now carries structured command metadata and
      separates bundle-local runnable commands from baseline-required manual compare commands, so
      GUI and MCP consumers do not present placeholder compare commands as ready-to-run actions.
      Maintenance: the DevTools GUI selected-summary inspector can now launch bundle-local
      runnable follow-ups (`stats`, `layout-perf-summary`, `memory-summary`, `triage`, `hotspots`,
      `trace`) through the shared diagnostics engine and records in-flight/error status in the GUI.
      Maintenance: trace follow-up results now project `trace.chrome.json` through
      `output_artifacts`, and the selected-result summary/details show that artifact path directly.
      Maintenance: each GUI-launched follow-up now writes a lightweight
      `.fret/diag/followups/*.json` result artifact and exposes the latest result path for copying.
      Maintenance: the selected-summary inspector mirrors that latest selected-bundle follow-up
      result JSON inline, so pass/fail/error/timing metadata is visible without opening the artifact
      manually.
      Maintenance: the selected-summary inspector now adds a structured selected-bundle follow-up
      result summary above the raw JSON, keeping status, command, duration, and error preview
      readable in the GUI.
      Maintenance: follow-up results are retained as a bounded in-memory history filtered to the
      selected bundle, so the GUI no longer implies that a previous bundle's last result belongs to
      the current selected-summary evidence.
      Maintenance: that selected-bundle follow-up history now renders as selectable result entries,
      allowing authors to switch the summary/raw JSON/copy target between recent artifacts.
      Maintenance: the selected follow-up result now has a details block with status, path, command,
      bundle, and error preview, and the exact producing command can be copied from the inspector.
      Maintenance: the selected follow-up JSON artifact can also be opened through the platform URL
      handler via an escaped `file://` URL, keeping artifact inspection one click away where native
      file URLs are supported.
      Maintenance: the follow-up result copy action now uses the selected bundle's latest history
      entry instead of the global last result artifact, keeping copied evidence paths aligned with
      the current selection.
      Maintenance: the selected-bundle follow-up JSON can now be copied directly from the same
      inspector, keeping the exact payload one click away for issue reports and AI triage.
      Maintenance: the selected trace follow-up artifact can now be copied or opened directly from
      the same selected-summary inspector. Relative `trace.chrome.json` paths are resolved against
      the repo root before clipboard or platform URL handling, while the follow-up result JSON
      remains the durable evidence record.
      Maintenance: the DevTools GUI selected-summary drill-down now includes a dedicated
      `Perf Evidence` section above raw JSON, projecting `perf_summary_json`, `compare_json`,
      curated metrics, and threshold failure evidence from regression summaries.
      Maintenance: the selected-summary drill-down projection now lives in
      `crates/fret-diag/src/regression_summary.rs` as shared diagnostics contract code, while
      `apps/fret-devtools/src/native.rs` only loads the JSON and renders the shared projection.
      This prevents GUI, MCP, and future CLI consumers from growing parallel regression-summary
      parsing rules.
      Maintenance: `apps/fret-devtools-mcp/src/native.rs` now exposes a sessionless
      `fret-diag://first-open.md` resource and mirrors the same shared IMUI product-chain route in
      its server instructions/resource text, so the MCP adapter does not invent a parallel
      first-open schema. The default product-chain discovery gate now also source-checks that MCP
      projection.
      Maintenance: `apps/fret-devtools/src/native.rs` now keeps the DevTools first-open header
      summary-first with `First-open Next Actions`, defaults `Evidence & Results` to the `Guide`
      tab, and moves the full first-open / dogfood / demo-metrics-debug / gate-command reference
      panels into that guide surface. The source gate now checks this posture so GUI
      discoverability stays productized without turning the first viewport into a raw command wall.

## P3 - Multi-window hand-feel closure

- [x] Freeze the current runner/backend gap inventory into one short parity checklist for this lane.
      Result: `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` now freezes hovered-window,
      peek-behind, transparent payload, and mixed-DPI follow-drag as the minimum P3 parity budget,
      and keeps the owner split pinned to `crates/fret-launch`, runner/backend integrations, and
      `ecosystem/fret-docking`.
- [x] Promote one bounded multi-window parity gate or diag suite that explicitly names:
      hovered window, peek-behind, transparent payload, and mixed-DPI follow-drag expectations.
      Result: `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md` and
      `tools/diag-campaigns/imui-p3-multiwindow-parity.json` now freeze one lane-owned bounded
      P3 package over four repo-owned scripts, without bloating `diag-hardening-smoke-docking`.
- [x] Keep `crates/fret-ui` contract growth out of runner-gap fixes unless ADR-backed evidence says
      the runtime contract is actually insufficient.
      Result: `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` and
      `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md` now make the source-policy rejection
      explicit and tie the remaining proof surface to runner/backend-owned diagnostics.
- [x] Add a product-chain perf entrypoint before claiming smoothness maturity.
      Result: `tools/diag_gate_imui_product_chain.py` now validates
      `perf-docking-arbitration-steady` in the lightweight source/script pass and can launch the
      `perf-docking` slice explicitly. The slice also repaired `diag perf` summary evidence so
      human stdout mode still writes `perf_case` rows into `regression.summary.json`, and the
      product-chain gate now requires readable item bundle artifacts plus a readable shared layout
      perf summary artifact and lightweight summary metrics. The 2026-05-15 follow-up turns those
      metrics into a conservative threshold gate by requiring shared `check.perf_thresholds.json`
      evidence, empty `threshold_failures`, and CLI-sourced `--max-top-total-us` /
      `--max-pointer-move-dispatch-us` / `--max-pointer-move-global-changes` thresholds.
      A second 2026-05-15 follow-up exposes renderer threshold CLI flags and gates
      `--max-renderer-encode-scene-us` / `--max-renderer-instance-bytes` / related renderer
      thresholds in the same `perf-docking` product-chain slice, so renderer metrics are no longer
      read-only evidence. The 2026-05-16 follow-up also runs that slice with
      `--trace-real-spans` and requires each perf-case bundle to carry a readable
      `trace.chrome.json` with `real_spans_included=true` and the `fret.perf.spans.v1` extension,
      so the product-chain perf entrypoint produces attribution evidence instead of only summary
      counters. The runtime capture repair now lives in
      `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` as `UiRealPerfSpanCaptureV1`; both the
      shared `ui_app_driver` and the custom `docking_arbitration_demo` render path flush through it,
      preserving non-zero sub-microsecond phases as 1us spans and preventing steady/idle perf
      bundles from losing the real-span extension solely due to microsecond rounding or custom
      driver bypass. The trace exporter now also consumes those real spans when synthetic timing
      counters are zero, and the canonical release `perf-docking` gate passes with zero threshold
      failures plus real-span trace artifacts.

## Closeout / follow-on management

- [x] Refresh the goal-completion audit after the latest product-chain discovery, Wayland
      admission/policy-skip including the M18 matrix, and perf-threshold slices.
      Result: `GOAL_COMPLETION_AUDIT_2026-05-15.md` keeps the umbrella in maintenance and
      explicitly not complete. The M18 local policy-skip matrix broadens the M17 gate across
      Windows and Linux/X11 sidecars, but real-host Wayland compositor acceptance, DevTools GUI
      productization, and broader perf attribution/smoothness remain owner-lane work, not
      `fret-imui` or runtime widening.
      2026-05-23 refresh: the Windows RTX4090 editor-paint closeout and the closed
      `editor-canvas-paint-replay-slice-v1` follow-on reduce one concrete perf owner
      (`canvas-paint-replay`) without changing baselines or moving smoothness pressure into
      `fret-imui`. The umbrella remains not complete because real-host Wayland hand-feel,
      broader DevTools GUI productization, and broader perf/smoothness attribution still need their
      owner-lane evidence.
- [x] Keep pure teaching-surface cleanup out of this umbrella unless it becomes the dominant
      remaining P0 pressure.
      Result: the remaining P0 backlog no longer reads as teaching-surface cleanup first, so no
      dedicated authoring-lane follow-on is warranted yet.
- [x] If further P0 work becomes mostly immediate convenience breadth
      (key ownership, item-status lifecycle, richer collection/pane proof), split a narrow follow-on
      instead of widening this umbrella folder.
      Result: `docs/workstreams/imui-response-status-lifecycle-v1/` now owns the narrow
      `ResponseExt` lifecycle vocabulary slice,
      `docs/workstreams/imui-key-owner-surface-v1/` now records the closed key-owner /
      item-local shortcut ownership follow-on with
      `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md` plus
      `CLOSEOUT_AUDIT_2026-04-21.md`, so the current helper-local
      `activate_shortcut` + command-metadata seams remain the shipped answer until stronger
      first-party proof warrants a different narrow lane, and
      `docs/workstreams/imui-collection-pane-proof-v1/` now records the closed collection-first /
      pane-first proof pair with a no-helper-widening verdict, while this umbrella keeps phase
      ordering and the remaining cross-phase backlog read.
- [x] If further P0/P1 pressure becomes mostly shared IMUI control affordance and compact field
      behavior, split a narrow control-surface follow-on instead of turning showcase cleanup into
      the umbrella lane's implementation log.
      Result: `docs/workstreams/imui-control-chrome-fearless-refactor-v1/` now owns the shared
      `fret-ui-kit::imui` control-chrome rewrite for button/switch/slider/combo/input defaults,
      while this umbrella keeps the higher-level product-closure ordering.
- [x] If the remaining P0 pressure becomes helper-owned trigger response shape
      (menu/submenu/tab outward response) rather than public `ResponseExt` vocabulary, split that
      into its own narrow follow-on too.
      Result: `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/` now owns the
      helper-owned menu/submenu/tab trigger response-surface decision instead of reopening either
      the umbrella lane or the lifecycle lane.
- [x] If the helper-owned trigger response lane lands but leaves duplicate public naming behind,
      split a second narrow follow-on for canonicalization instead of rewriting the historical lane.
      Result: `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/` now owns the
      cleanup closeout that removes the duplicate alias layer after the response surface landed.
- [x] If the remaining P0 pressure becomes broader menu/submenu/tab policy depth instead of
      helper-owned outward response shape, split another narrow follow-on instead of reopening the
      closed response lanes.
      Result: `docs/workstreams/imui-menu-tab-policy-depth-v1/` now owns the current hover-switch /
      submenu grace / tab ownership audit, keeping response-surface naming, key ownership,
      collection breadth, shell helpers, and runtime widening in their separate lanes.
- [x] If the remaining P1 pressure becomes `BeginChild()`-scale child-region depth instead of
      proof breadth, split another narrow follow-on instead of reopening the closed
      collection/pane lane.
      Result: `docs/workstreams/imui-child-region-depth-v1/` now records the closed child-region
      depth verdict: the bounded `ChildRegionChrome::{Framed, Bare}` slice is landed, while
      pane-proof breadth, shell-helper promotion, menu/tab policy, and runtime widening remain in
      their separate lanes.
- [x] If the remaining collection depth becomes narrower background marquee / box-select proof
      rather than generic helper widening, split another narrow follow-on and keep it app-owned
      until the frozen proof budget is satisfied.
      Result: `docs/workstreams/imui-collection-box-select-v1/` now records the closed
      background-only box-select slice in `imui_editor_proof_demo`, keeps lasso /
      keyboard-owner depth and shared helper growth out of generic `fret-ui-kit::imui`, and
      leaves broader collection depth to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned keyboard-owner proof rather than
      a reopened generic key-owner or helper-widening question, split another narrow follow-on and
      keep it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-keyboard-owner-v1/` now records the closed
      app-owned collection keyboard-owner slice in `imui_editor_proof_demo`, keeps the generic
      key-owner verdict closed, and leaves lasso / action semantics / shared helper growth to
      future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned delete-selected semantics rather
      than broader collection command breadth or helper growth, split another narrow follow-on and
      keep it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-delete-action-v1/` now records the closed
      app-owned collection delete-selected slice in `imui_editor_proof_demo`, keeps select-all /
      rename / context-menu breadth and shared helper growth out of generic `fret-ui-kit::imui`,
      and leaves lasso / second-proof-surface questions to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned context-menu quick actions
      rather than broader collection command breadth or helper growth, split another narrow
      follow-on and keep it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-context-menu-v1/` now records the closed
      app-owned collection context-menu slice in `imui_editor_proof_demo`, keeps select-all /
      rename / broader command breadth and shared helper growth out of generic `fret-ui-kit::imui`,
      and leaves lasso / second-proof-surface questions to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned zoom/layout depth rather than
      broader collection command breadth or helper growth, split another narrow follow-on and keep
      it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-zoom-v1/` now records the closed app-owned collection zoom/layout slice in `imui_editor_proof_demo`, replaces the frozen column count with viewport-plus-zoom-derived layout metrics, keeps select-all / rename / second-proof-surface / shared helper growth out of generic `fret-ui-kit::imui`, and leaves broader collection product depth to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned select-all breadth rather than
      broader collection command breadth or helper growth, split another narrow follow-on and keep
      it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-select-all-v1/` now records the closed app-owned collection select-all slice in `imui_editor_proof_demo`, routes Primary+A through the existing collection-scope owner, keeps rename / second-proof-surface / shared helper growth out of generic `fret-ui-kit::imui`, and leaves broader collection product depth to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned rename breadth rather than
      broader collection command breadth or helper growth, split another narrow follow-on and keep
      it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-rename-v1/` now records the closed app-owned collection rename slice in `imui_editor_proof_demo`, routes F2 plus the existing context-menu entry through one rename modal, keeps second-proof-surface / shared helper growth out of generic `fret-ui-kit::imui`, and leaves broader collection product depth to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned inline rename posture rather than
      broader collection command breadth or helper growth, split another narrow follow-on and keep
      it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-inline-rename-v1/` now records the closed app-owned collection inline rename slice in `imui_editor_proof_demo`, routes F2 plus the existing context-menu entry through one inline editor mounted inside the active asset tile, keeps second-proof-surface / shared helper growth out of generic `fret-ui-kit::imui`, and leaves broader collection product depth to future narrower follow-ons.
- [x] If the collection-first proof starts carrying too much app-owned implementation in one host file,
      split a narrow demo-local modularization follow-on before arguing for shared helpers from
      maintenance pressure alone.
      Result: `docs/workstreams/imui-editor-proof-collection-modularization-v1/` now records the closed demo-local collection module slice in `imui_editor_proof_demo`, moves collection assets/models/render/unit tests into `collection.rs`, keeps the host on `mod collection;` plus one render call and drag-asset delegation, and reset the default next non-multi-window priority to broader app-owned command-package breadth before that command-package lane later closed.
- [x] After the inline rename closeout lands, refresh the next non-multi-window IMUI follow-on
      order instead of reopening older collection, key-owner, or generic helper lanes by habit.
      Result: `P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md` now freezes the current order as
      closed app-owned collection command-package breadth first, second proof-surface promotion
      next, and only later any reconsideration of shared helper growth, while child-region resize,
      submenu-intent tuning, key-owner reopening, and generic helper widening stay explicitly deferred.
- [x] Start the broader app-owned collection command-package lane locally on the same proof surface
      instead of inventing another generic helper or reopening the structural modularization folder.
      Result: `docs/workstreams/imui-collection-command-package-v1/` now records the closed
      command-package lane, lands duplicate-selected plus explicit rename-trigger slices in
      `imui_editor_proof_demo/collection.rs`, keeps those routes app-owned on the existing
      keyboard/button/context-menu owner paths, rejects a third command verb in this folder, and
      moves the next non-multi-window priority to a second proof surface.
- [x] After the command-package closeout lands, start and close the second proof-surface follow-on instead of
      reopening the closed package or widening shared helpers from one proof.
      Result: `docs/workstreams/imui-collection-second-proof-surface-v1/` now records the closed
      follow-on, names `editor_notes_demo.rs` as the primary shell-mounted candidate, keeps
      `workspace_shell_demo.rs` as supporting evidence, lands the `Scene collection` left-rail
      surface in `editor_notes_demo.rs`, and closes on a no-helper-widening verdict because the two
      collection proof surfaces do not yet need the same shared helper.
- [x] If P1 becomes mostly shell composition work, split it into a narrow workbench-shell follow-on.
      Result: `docs/workstreams/imui-workbench-shell-closure-v1/` now records the narrow P1 shell
      closure decision and already closes on a no-new-helper-yet verdict, leaving this umbrella
      focused on phase ordering and cross-phase status.
- [x] Keep future diagnostics/devtools productization out of this umbrella unless fresh P2 pressure
      becomes implementation-heavy again.
      Result: P2 is closed in this lane; any future tooling UX/productization should start as a
      narrow devtools follow-on instead of widening this folder.
- [x] Record the identity-warning diagnostics/browser chain as a closed P2 evidence branch.
      Result: the structured identity diagnostics, browser query model, offline HTML sidecar,
      structural smoke gate, and committed sample bundle all live in narrow closed follow-ons, while
      this umbrella records them as part of the first-open diagnostics path.
- [x] If P3 becomes mostly platform diagnostics and runner work, continue using the existing docking
      parity lane or start a narrower follow-on there instead of bloating this folder.
      Result: after the P1 shell closeout and the umbrella maintenance refresh, the active
      execution priority continues in `docs/workstreams/docking-multiwindow-imgui-parity/`, with
      `WORKSTREAM.json`, `M0_BASELINE_AUDIT_2026-04-13.md`, and
      `M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md` as the current resume surface. M19 keeps
      local policy-skip evidence from being mistaken for real Wayland compositor acceptance.
