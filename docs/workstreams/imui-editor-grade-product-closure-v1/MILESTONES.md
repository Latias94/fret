# ImUi Editor-Grade Product Closure v1 - Milestones

Status: maintenance umbrella lane
Last updated: 2026-04-28

Status note (2026-04-22): this file now records umbrella phase state only. Implementation-heavy
execution has moved into closed narrow follow-ons or the active docking parity lane.

Status note (2026-05-14): the workspace shell tab-strip gate set was refreshed, but this umbrella
lane is still maintenance-only. Treat the new evidence as proof-state refresh for the workspace
shell surface, not as a reactivation of the umbrella lane.

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why a new follow-on is warranted instead of reopening older `imui`
  closeout lanes,
- the remaining maturity gap is split into P0/P1/P2/P3,
- and each phase names its current proof family.

Primary evidence:

- `M0_BASELINE_AUDIT_2026-04-12.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`

Current status:

- Closed on 2026-04-12 via `M0_BASELINE_AUDIT_2026-04-12.md`.

## M1 - P0 default authoring lane closure

Exit criteria:

- one first-party default immediate authoring path is named and taught consistently,
- the path explains stable identity, layout defaults, and focus/hover expectations without
  depending on runtime widening,
- and the minimum proof budget for future `fret-ui-kit::imui` helper widening is explicitly frozen
  as the golden pair rather than inferred from ad hoc example pressure.

Primary evidence:

- `DESIGN.md`
- `P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
- `P0_FOOTGUN_AUDIT_2026-04-12.md`
- `P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `P0_ROOT_HOSTING_RULE_2026-04-12.md`
- `P0_STABLE_IDENTITY_RULE_2026-04-12.md`
- `P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Current status:

- In progress as umbrella status only.
- The generic/editor golden pair is now frozen as:
  `apps/fret-cookbook/examples/imui_action_basics.rs` +
  `apps/fret-examples/src/imui_editor_proof_demo.rs`.
- Future `fret-ui-kit::imui` public helper widening now has an explicit minimum proof budget:
  it must name two real first-party proof surfaces, and the current budget floor is the frozen
  golden pair rather than any single reference demo.
- The first-open mounting rule is now explicit:
  root/non-layout parent -> `fret_imui::imui(cx, ...)`,
  explicit layout host + bare sibling emission -> `fret_imui::imui_raw(cx, ...)`.
- The first-open stable-identity rule is now explicit:
  `ui.for_each_unkeyed(...)` is only for static/order-stable lists, while dynamic collections
  should default to `ui.for_each_keyed(...)` or `ui.id(key, ...)`.
- The current footgun audit concludes that documentation and proof-selection dominate; the only
  credible contract fix was to collapse the old split into one safe default plus one explicit raw
  seam.
- The demote/delete plan is now frozen:
  `imui_hello_demo` is smoke/reference, public docs name the golden pair explicitly, and the
  source-policy gates distinguish golden/reference/compatibility roles.
- A 2026-04-13 status pass now records that focused item-local shortcut seams materially improved
  across direct pressables, popup/menu triggers, and combo/combo-model triggers without widening
  the runtime's global shortcut owner model.
- The tested shortcut repeat rule is now explicit at the immediate layer:
  default key repeat does not retrigger activation, and `shortcut_repeat=true` is the opt-in seam.
- A launched P0 authoring proof now exists for the generic/default IMUI path:
  `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
  runs the `imui_action_basics` cookbook example and proves command palette, declarative, GenUI,
  and IMUI triggers dispatch the same typed action into one view-local state path. The focused gate
  command is `python tools/diag_gate_action_first_authoring_v1.py --only
  cookbook-imui-action-basics-cross-frontend`.
- The remaining credible P0 backlog is now narrower:
  the closed key-owner closeout record at
  `docs/workstreams/imui-key-owner-surface-v1/`,
  the split item-status lifecycle follow-on at
  `docs/workstreams/imui-response-status-lifecycle-v1/`,
  the closed collection/pane proof-breadth closeout record at
  `docs/workstreams/imui-collection-pane-proof-v1/`,
  the closed collection box-select closeout record at
  `docs/workstreams/imui-collection-box-select-v1/`,
  the closed collection keyboard-owner closeout record at
  `docs/workstreams/imui-collection-keyboard-owner-v1/`,
  the closed collection delete-action closeout record at
  `docs/workstreams/imui-collection-delete-action-v1/`,
  the closed collection context-menu closeout record at
  `docs/workstreams/imui-collection-context-menu-v1/`,
  the closed collection zoom closeout record at
  `docs/workstreams/imui-collection-zoom-v1/`,
  the closed collection select-all closeout record at
  `docs/workstreams/imui-collection-select-all-v1/`,
  the closed collection rename closeout record at
  `docs/workstreams/imui-collection-rename-v1/`,
  the closed collection inline-rename closeout record at
  `docs/workstreams/imui-collection-inline-rename-v1/`,
  the closed collection modularization closeout record at
  `docs/workstreams/imui-editor-proof-collection-modularization-v1/`,
  the closed menu/tab policy closeout at
  `docs/workstreams/imui-menu-tab-policy-depth-v1/`,
  the closed child-region depth closeout record at
  `docs/workstreams/imui-child-region-depth-v1/`,
  the closed trigger-response canonicalization closeout at
  `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/`,
  and the closed helper-owned trigger response-surface follow-on at
  `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/`.
- A 2026-04-23 priority refresh now freezes the next non-multi-window execution order in
  `P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`:
  the app-owned collection command-package closeout first, second proof-surface promotion next,
  and only later reconsideration of any generic collection helper growth.
- The command-package priority is now closed in
  `docs/workstreams/imui-collection-command-package-v1/`,
  where the landed slices are `Primary+D` duplicate-selected plus an explicit rename trigger,
  both kept on the existing collection-local keyboard/button/context-menu owner paths.
- The collection second proof-surface follow-on is now closed in
  `docs/workstreams/imui-collection-second-proof-surface-v1/`, with `editor_notes_demo.rs` as the
  primary shell-mounted second proof candidate, the `Scene collection` left-rail surface landed
  there, `workspace_shell_demo.rs` as supporting evidence, and a no-helper-widening verdict because
  the two collection proof surfaces do not yet need the same shared helper.
- Execution consequence:
  keep this lane as the umbrella recorder. The key-owner / item-local shortcut ownership slice now
  stays closed in `docs/workstreams/imui-key-owner-surface-v1/`,
  where `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md` plus `CLOSEOUT_AUDIT_2026-04-21.md` now freeze
  the no-new-surface verdict over `imui_response_signals_demo` plus the bounded targeted
  `fret-imui` shortcut floor,
  the `ResponseExt` lifecycle vocabulary slice now lives in
  `docs/workstreams/imui-response-status-lifecycle-v1/`,
  the collection/pane proof-breadth closeout record now lives in
  `docs/workstreams/imui-collection-pane-proof-v1/`,
  the collection box-select closeout record now lives in
  `docs/workstreams/imui-collection-box-select-v1/`,
  the collection keyboard-owner closeout record now lives in
  `docs/workstreams/imui-collection-keyboard-owner-v1/`,
  the collection delete-action closeout record now lives in
  `docs/workstreams/imui-collection-delete-action-v1/`,
  the collection context-menu closeout record now lives in
  `docs/workstreams/imui-collection-context-menu-v1/`,
  the collection zoom closeout record now lives in
  `docs/workstreams/imui-collection-zoom-v1/`,
  the collection select-all closeout record now lives in
  `docs/workstreams/imui-collection-select-all-v1/`,
  the collection rename closeout record now lives in
  `docs/workstreams/imui-collection-rename-v1/`,
  the collection inline-rename closeout record now lives in
  `docs/workstreams/imui-collection-inline-rename-v1/`,
  the collection modularization closeout record now lives in
  `docs/workstreams/imui-editor-proof-collection-modularization-v1/`,
  the collection command-package closeout record now lives in
  `docs/workstreams/imui-collection-command-package-v1/`,
  the closed second proof-surface closeout record now lives in
  `docs/workstreams/imui-collection-second-proof-surface-v1/`,
  the broader menu/submenu/tab policy closeout record now lives in
  `docs/workstreams/imui-menu-tab-policy-depth-v1/`,
  the closed `BeginChild()`-scale child-region depth closeout record now lives in
  `docs/workstreams/imui-child-region-depth-v1/`,
  the next non-multi-window order is now frozen in
  `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`,
  the helper-owned menu/submenu/tab trigger response verdict now lives in
  `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/`, the naming cleanup closeout now lives in
  `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/`,
  and any further implementation-heavy P0 work should keep following the same narrow follow-on rule
  instead of turning M1 back into a generic helper-growth backlog.
- The shared control-chrome rewrite is now also a closed narrow closeout record at
  `docs/workstreams/imui-control-chrome-fearless-refactor-v1/`, so future control-surface pressure
  should reopen only through another narrow lane instead of widening this umbrella.

## M2 - P1 editor workbench shell closure

Exit criteria:

- the repo can point to one coherent editor/workbench proof rather than isolated shell examples,
- workspace shell, docking, and editor composites are mapped to explicit owners,
- and the default proof set is reviewable as one system.

Primary evidence:

- `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`

Current status:

- Closed in this umbrella.
- The primary P1 workbench-shell proof is now frozen as
  `apps/fret-examples/src/workspace_shell_demo.rs`.
- The minimal shell-mounted editor-rail proof is now frozen as
  `apps/fret-examples/src/editor_notes_demo.rs`.
- `apps/fret-examples/src/imui_editor_proof_demo.rs` remains supporting docking/editor evidence,
  but is no longer the default workbench-shell reading order.
- The current owner split is now explicit:
  `fret-workspace` for shell slots/tabstrip/command scope,
  `fret-docking` for dock choreography,
  `fret-ui-editor` for editor composites,
  and app/example ownership for scene-local center content.
- The promoted P1 shell diagnostics floor is now explicit:
  `diag-hardening-smoke-workspace` remains the launched shell suite, and its frozen minimum now
  spans tab close/reorder/split preview, dirty-close prompt, Escape focus restore, and file-tree
  keep-alive.
- P1 is now split for implementation-heavy work:
  `docs/workstreams/imui-workbench-shell-closure-v1/` owns the active default-workbench shell
  closure follow-on, while this umbrella keeps the phase ordering and status record.
  Update on 2026-04-13:
  that follow-on already closed on a no-new-helper-yet verdict for promoted first-party shell
  helpers, so future active execution should no longer treat P1 shell composition as the default
  open lane.

## M3 - P2 unified diagnostics/devtools surface

Exit criteria:

- one first-open developer loop ties together inspect, selectors, scripts, bundles, and compare,
- GUI, CLI, and MCP are explicitly shown as consumers of the same artifact contract,
- and the lane names one bounded devtools smoke/gate package.

Primary evidence:

- `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
- `P2_DISCOVERABILITY_ENTRY_2026-04-12.md`
- `docs/diagnostics-first-open.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `apps/fret-devtools/src/main.rs`
- `apps/fret-devtools-mcp/src/main.rs`

Current status:

- Closed in this umbrella.
- The first-open P2 developer path is now explicit:
  inspect/pick -> script -> bundle -> compare starts from the CLI-compatible diagnostics contract,
  while DevTools GUI and MCP stay thin consumers of the same artifacts root.
- The compare story is now explicit:
  direct bundle/session diff uses `diag compare`, while aggregate run-set comparison flows through
  `diag summarize` plus shared `regression.summary.json` / `regression.index.json` consumers.
- The owner split is now explicit:
  `fret-bootstrap` owns runtime capture/export, `fret-diag` owns shared orchestration and artifact
  projections, `fret-devtools` owns GUI UX over those contracts, and `fret-devtools-mcp` owns the
  headless automation/resource adapter over the same contracts.
- The bounded devtools smoke package is now explicit:
  `python3 tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke`
  now freezes one repo-owned gate over `tools/diag-scripts/tooling/todo/todo-baseline.json` and
  `tools/diag-campaigns/devtools-first-open-smoke.json`.
- The direct half of that gate now proves:
  real `diag run`, named bundle emission, latest-bundle resolution through
  `script.result.json:last_bundle_dir`, and direct `diag compare` over a state-changing pair.
- The aggregate half of that gate now proves:
  one campaign root with `campaign.manifest.json`, explicit root `diag summarize`,
  `regression.summary.json`, `regression.index.json`, and successful `diag dashboard` projection.
- The discoverability entry is now explicit:
  `docs/diagnostics-first-open.md` is the canonical first-open diagnostics workflow, and the
  inspect, bundles/scripts, GUI dogfood, and diagnostics-v2 navigation docs are now explicit
  branch/reference notes instead of competing start pages.
- 2026-05-14 maintenance hardening keeps the closed P2 contract visible in the GUI itself:
  `apps/fret-devtools/src/native.rs` now renders a first-open evidence panel with the canonical
  diagnostics doc, GUI branch doc, repo preflight, artifacts root, direct run/latest/compare loop,
  campaign summarize/dashboard loop, and the bounded P2 smoke gate.
- 2026-05-14 product-chain maintenance also validates `fretboard-dev list tool-apps --json` as the
  stable DevTools GUI/MCP first-open map from `tools/diag_gate_imui_product_chain.py`, so the
  default IMUI product-chain gate catches drift in repo preflight and per-tool command/docs/gate
  fields. The same discovery gate now covers `fretboard-dev --help` and
  `fretboard-dev list --help`, so the tool-apps index is reachable from the first CLI help screens.
- 2026-05-15 product workflow discovery refresh extends that same map with
  `product_workflows.imui-product-chain`, including the default product-chain command, the
  discovery-only command, the launched `perf-docking` command, the promoted
  `perf-docking-arbitration-steady` suite, and the expected `regression.summary.json` /
  `check.perf_thresholds.json` perf artifacts. This keeps DevTools-style consumers pointed at the
  shared evidence chain instead of inventing GUI-only product workflow metadata.
- 2026-05-15 DevTools GUI product-workflow projection mirrors that same route in
  `apps/fret-devtools/src/native.rs`, so the first-open evidence panel now surfaces the
  `imui-product-chain` command, focused discovery command, launched `perf-docking` command, suite,
  docs, and expected perf artifacts from the shared product-chain vocabulary. The default
  product-chain discovery gate now source-checks that GUI projection.
- 2026-05-15 DevTools GUI demo/metrics/debug route projection adds a persistent
  `demo-metrics-debug` route in `apps/fret-devtools/src/native.rs`, naming the editor proof/editor
  notes/device shell demos plus `diag stats`, `diag layout-perf-summary`, `diag memory-summary`,
  `diag triage`, and `diag hotspots` entrypoints. This is a productization step for
  always-available demo/metrics/debug discoverability, not a `fret-imui` API expansion.
- 2026-05-15 DevTools GUI gate-command projection adds a first-open `Gate Commands` block for
  stale paint/scene, pixels-changed, perf-threshold, and resource-footprint diagnostics entrypoints.
  The owner remains `docs/workstreams/diag-devtools-gui-v1/`; this umbrella records the evidence
  only to keep the broader Dear ImGui-class product goal honest.
- 2026-05-15 DevTools script-target gate command builder adds shared `fret-diag` command
  parameterization for stale paint/scene and pixels-changed profiles plus a GUI selector,
  `script.json`/`test-id` inputs, preview, and `Copy generated command` action. This starts the
  profile-specific gate UI path without moving gate templates into `apps/fret-devtools` or
  widening `fret-imui`.
- 2026-05-15 DevTools script-target gate projection now returns structured `diag_args` and
  `missing_inputs` alongside the command line, giving the next GUI launch/run slice a first-class
  execution contract instead of forcing the GUI to parse shell command strings.
- 2026-05-15 DevTools script-target gate launcher adds `Run generated command` for stale
  paint/scene and pixels-changed profiles. The GUI executes the structured `diag_args` through
  `fret-diag`, records in-flight/error status, and writes `.fret/diag/gate-runs/*.json` result
  artifacts without moving gate policy into `apps/fret-devtools` or widening `fret-imui`.
- 2026-05-15 generated gate result history makes stale paint/scene and pixels-changed gate runs
  reviewable inside the GUI: selected result details, structured summary, raw JSON, path/command/JSON
  copy actions, and platform URL open support all point at `.fret/diag/gate-runs/*.json` artifacts.
- 2026-05-15 DevTools perf-threshold generated gate builder extends that same loop to
  `perf-thresholds`: `fret-diag` owns the structured `diag perf` command projection, while the GUI
  renders target/repeat/warmup/aggregate/threshold inputs and reuses the generated gate runner plus
  `.fret/diag/gate-runs/*.json` result history.
- 2026-05-15 DevTools resource-footprint generated gate builder closes the remaining first-class
  gate UI item by first wiring the missing `diag repro` resource threshold CLI contract, then
  exposing `resource-footprint-thresholds` as a shared generated command with GUI inputs for target,
  working-set/peak/CPU thresholds, and a single launch argv item.
- 2026-05-15 DevTools GUI selected-summary follow-up commands consume a shared `fret-diag`
  projection that converts the selected `bundle_dir` into concrete `diag stats`,
  `layout-perf-summary`, `memory-summary`, `triage`, `hotspots`, visual-compare, and
  footprint-compare commands. This keeps the failing-summary-to-next-command loop productized
  without moving diagnostics policy into `fret-imui` or leaving command composition as GUI-private
  logic.
- 2026-05-16 DevTools selected-summary follow-up projection adds `diag trace <bundle> --json` to
  the same shared GUI/MCP runnable action set, so Chrome trace artifact generation stays in the
  diagnostics owner lane instead of becoming another GUI-private command path.
- 2026-05-16 GUI-launched trace follow-ups now record `trace.chrome.json` in
  `output_artifacts`, and the selected-result summary/details expose that artifact path for reuse.
- 2026-05-15 DevTools MCP regression dashboard now consumes the same shared drill-down/follow-up
  projection, returning bundle dirs, capability provenance, perf evidence, and follow-up command
  lines in both structured JSON and the human summary.
- 2026-05-16 DevTools MCP regression dashboard adds structured follow-up command rows with
  `diag_args` and baseline classification, so AI consumers no longer need to parse command strings
  to run bundle-local trace/stats/triage actions.
- 2026-05-15 shared regression follow-up commands are now structured in
  `crates/fret-diag/src/regression_summary.rs`, with GUI/MCP consumers separating bundle-local
  runnable commands from baseline-required manual compare commands.
- 2026-05-15 DevTools GUI selected-summary runnable follow-ups can now be launched in-app for
  `stats`, `layout-perf-summary`, `memory-summary`, `triage`, and `hotspots`; the GUI records
  in-flight/error status while leaving baseline-required compare commands manual.
- 2026-05-15 GUI-launched regression follow-ups now write
  `.fret/diag/followups/*.json` result records with command/status/error/timing metadata and expose
  the latest result path for copying.
- 2026-05-15 the DevTools GUI selected-summary inspector now mirrors the latest selected-bundle
  follow-up result JSON inline, so authors can inspect pass/fail/error/timing metadata without
  leaving the panel.
- 2026-05-15 the same inspector now projects the latest selected-bundle follow-up result into a
  structured summary above raw JSON, keeping status, command, duration, and error preview
  immediately scannable.
- 2026-05-15 follow-up results are retained as a bounded in-memory history filtered to the selected
  bundle, so selected-summary triage can distinguish current evidence from a previous bundle's last
  launched follow-up.
- 2026-05-15 selected-bundle follow-up history now renders as selectable result entries, allowing
  authors to switch the summary/raw JSON/copy target between recent artifacts.
- 2026-05-15 selected follow-up results now have a details block with status, path, command,
  bundle, and error preview, and the exact producing command can be copied from the inspector.
- 2026-05-15 the selected follow-up JSON artifact can now be opened through the platform URL
  handler via an escaped `file://` URL, keeping native artifact inspection one click away when file
  URLs are supported.
- 2026-05-15 the follow-up result copy action now resolves the selected bundle's latest history
  path instead of the global last result artifact, keeping copied evidence aligned with selection.
- 2026-05-15 the selected-bundle follow-up JSON is now copyable from the same inspector, keeping
  the exact payload one click away for issue reports and AI triage.
- 2026-05-21 the selected trace follow-up artifact can now be copied or opened directly from the
  same inspector. The action prefers `trace_report.trace_chrome_json_path`, falls back to the
  `trace.chrome.json` output artifact row, and resolves relative paths against the repo root before
  clipboard or platform URL handling.
- 2026-05-15 DevTools GUI perf-evidence drill-down extracts selected regression summary perf
  evidence into a dedicated `Perf Evidence` section above raw JSON. The focused unit gate covers
  `perf_summary_json`, `compare_json`, curated metric lines, and threshold failure counts/JSON.
- 2026-05-15 shared regression drill-down projection moves that selected-summary parsing into
  `crates/fret-diag/src/regression_summary.rs`; `apps/fret-devtools/src/native.rs` now consumes
  `regression_summary_drilldown(&summary)` instead of owning GUI-private perf/capability parsing.
  The source gate now checks the shared projection and the GUI call site together.
- 2026-05-15 DevTools MCP product-workflow projection adds a sessionless
  `fret-diag://first-open.md` resource in `apps/fret-devtools-mcp/src/native.rs` and points the MCP
  server instructions at it. That resource mirrors the same `imui-product-chain` command/focused
  command/launched perf command/suite/docs/artifacts route, while the product-chain discovery gate
  source-checks the MCP projection alongside the GUI projection.
- 2026-04-28 identity warning diagnostics are now an explicit closed P2 branch:
  `diag query identity-warnings` reads captured schema2 identity warnings, `--browser` adds grouped
  JSON, `--html-out` writes a self-contained offline review artifact,
  `--html-check-out` writes `check.identity_browser_html`, and
  `crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json` provides a first-open
  sample bundle without launching a demo.
- P2 is now closed for this lane.
  Future diagnostics/devtools work that becomes implementation-heavy should move into a narrower
  devtools follow-on instead of widening this folder.

## M4 - P3 multi-window hand-feel closure

Exit criteria:

- hovered-window, peek-behind, transparent payload, and mixed-DPI follow-drag responsibilities stay
  runner-owned,
- the current parity matrix and proof package are concise enough to reopen quickly,
- and no `imui` helper growth is used as a workaround for runner/backend gaps.

Primary evidence:

- `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- In progress, but the active execution now lives in the docking parity lane rather than this
  folder.
- The current execution split is now explicit:
  the immediate child-region depth slice is closed in
  `docs/workstreams/imui-child-region-depth-v1/`,
  while runner/backend multi-window work stays active in
  `docs/workstreams/docking-multiwindow-imgui-parity/`.
- The first P3 checklist is now explicit:
  `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` freezes hovered-window, peek-behind,
  transparent payload, and mixed-DPI follow-drag as the runner-owned parity budget for this lane.
- The owner split is now explicit:
  `crates/fret-launch`, runner/backend integrations, and `ecosystem/fret-docking` stay the default
  owners, while `crates/fret-ui` and generic `imui` helpers remain out of scope unless stronger
  evidence reopens them.
- The bounded parity package is now explicit:
  `tools/diag-campaigns/imui-p3-multiwindow-parity.json` now binds four repo-owned scripts into one
  lane-owned package over `docking_arbitration_demo`, and keeps `diag-hardening-smoke-docking`
  small instead of overloading it with all P3 stress coverage.
- 2026-05-14 perf entrypoint refresh:
  `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json` is now wired into the
  product-chain gate as the docking perf/smoothness entrypoint, with a launched `perf-docking`
  slice that verifies `diag perf` writes passing `perf_case` rows, readable item bundle artifacts,
  a readable shared `layout.perf.summary.v1.json` artifact, and lightweight summary metrics to the
  regression evidence chain.
- 2026-05-15 perf threshold refresh:
  the same launched `perf-docking` product-chain slice now passes conservative `diag perf`
  thresholds (`--max-top-total-us`, `--max-top-layout-us`, `--max-top-solve-us`,
  `--max-pointer-move-dispatch-us`, `--max-pointer-move-hit-test-us`, and
  `--max-pointer-move-global-changes`) and verifies the shared `check.perf_thresholds.json`
  artifact before accepting the summary.
- 2026-05-15 renderer threshold refresh:
  `diag perf` now exposes renderer threshold CLI flags, and the `perf-docking` product-chain slice
  gates the renderer telemetry that was previously only projected as summary metrics
  (`--max-renderer-encode-scene-us`, `--max-renderer-upload-us`,
  `--max-renderer-record-passes-us`, `--max-renderer-encoder-finish-us`,
  `--max-renderer-prepare-text-us`, `--max-renderer-prepare-svg-us`,
  `--max-renderer-instance-bytes`, and `--max-renderer-encode-scene-text-ops`).
- 2026-05-16 trace attribution refresh:
  the same launched `perf-docking` product-chain slice now passes `--trace-real-spans`, injects the
  `FRET_DIAG_REAL_SPANS` runtime opt-in through `diag perf`, and requires one readable
  `trace.chrome.json` beside each perf-case bundle. The gate validates `kind=perf_trace_chrome`,
  `trace_source=bundle_synthetic_phases_with_extension_spans`, `real_spans_included=true`,
  a positive `real_span_event_count`, and the `fret.perf.spans.v1` extension key. This makes the
  bounded product-chain perf entrypoint trace-attributable without claiming full smoothness
  maturity across all editor workloads. Runtime capture moved into
  `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` as `UiRealPerfSpanCaptureV1`; the shared
  `ui_app_driver` and custom `docking_arbitration_demo` render path both use it, preserving
  non-zero sub-microsecond driver phases as 1us spans and proving the real-span extension path
  instead of silently falling back to synthetic-only traces. The trace exporter also keeps
  `fret.perf.spans.v1` when synthetic timing counters are zero, matching steady/idle product-chain
  bundles where real driver spans are the attribution source. The latest canonical release gate
  passed at
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233`,
  with two passing perf cases, zero threshold failures, and real-span trace event counts of 40 and
  45 for the promoted suite scripts.
- P3 remains the active global parity lane when real backend/runner acceptance is available, while
  the latest non-multi-window local follow-on is now closed in
  `docs/workstreams/imui-collection-second-proof-surface-v1/` after command-package closeout.
  Continue multi-window implementation-heavy work in the existing docking parity lane or a narrower
  runner follow-on instead of widening this folder.
- The docking parity lane now has an explicit first-open state:
  `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json` and
  `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md` freeze the
  current resume surface and name `DW-P0-dpi-006` as the next bounded execution slice.

## M5 - Narrow follow-ons or closeout

Exit criteria:

- the lane either closes with explicit owner splits and reference links,
- or splits into narrower follow-ons for the implementation-heavy phases that genuinely need their
  own execution folders.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- phase-specific follow-on lanes created after M1/M2/M3/M4 evidence is strong enough

Current status:

- In progress as a maintenance/status umbrella.
- The 2026-05-15 goal-completion audit keeps the lane explicitly not complete after the latest
  product-chain discovery, Wayland source/admission/policy-skip, and perf-threshold refreshes.
- The lane should close only after real-host Wayland hand-feel, DevTools GUI productization, and
  broader perf attribution/smoothness are resolved or explicitly handed off with fresh owner-lane
  closeout evidence.
