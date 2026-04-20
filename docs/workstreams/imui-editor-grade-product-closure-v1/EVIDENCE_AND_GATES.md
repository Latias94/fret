# ImUi Editor-Grade Product Closure v1 - Evidence & Gates

Goal: keep the editor-grade maturity plan tied to real proof surfaces, not just strategy prose.

## Evidence anchors (current)

- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_FOOTGUN_AUDIT_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_DEMOTE_DELETE_PLAN_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_ROOT_HOSTING_RULE_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_STABLE_IDENTITY_RULE_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/DESIGN.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DISCOVERABILITY_ENTRY_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `docs/diagnostics-first-open.md`
- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-imui/src/tests/mod.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `apps/fret-devtools/src/main.rs`
- `apps/fret-devtools-mcp/src/main.rs`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `tools/diag_gate_imui_p2_devtools_first_open.py`
- `tools/diag-campaigns/devtools-first-open-smoke.json`

## First-open repro surfaces

Use these when reopening the lane before diving into older notes:

1. Immediate generic/default proof
   - `cargo run -p fretboard-dev -- dev native --demo imui_action_basics --features cookbook-imui`
2. Immediate/editor proof
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
3. Workspace shell proof
   - `cargo run -p fret-demo --bin workspace_shell_demo`
4. DevTools proof
   - `cargo run -p fret-devtools`
5. Multi-window parity proof
   - `cargo run -p fret-demo --bin docking_arbitration_demo`

These are not the only relevant surfaces, but they give the fastest current read on the lane
without reopening older workstreams first.

## Current focused gates

### Immediate authoring / adapter gates

- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke`
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
- `cargo nextest run -p fret-imui`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_example_keeps_current_facade_teaching_surface`
- `cargo nextest run -p fret-examples --lib first_party_imui_examples_keep_current_facade_teaching_surface immediate_mode_examples_docs_name_the_golden_pair_and_reference_roster immediate_mode_examples_docs_name_the_mounting_rule_for_imui_vs_imui_vstack immediate_mode_examples_docs_name_the_stable_identity_rule immediate_mode_workstream_freezes_the_two_surface_proof_budget_before_helper_widening imui_hello_demo_is_explicitly_demoted_to_smoke_reference compatibility_only_node_graph_imui_demo_is_the_only_first_party_retained_compatibility_example`

This package now locks the current immediate-mode product message at the source-policy layer:

- the golden pair is named explicitly,
- the nested-vs-root mounting rule stays explicit,
- the static-vs-dynamic stable-identity rule stays explicit,
- the reference/advanced/compatibility roster stays classified,
- the proof budget rule stays frozen before any future helper widening,
- focused item-local shortcuts now span direct pressables, popup/menu triggers, and
  combo/combo-model triggers at the ecosystem layer,
- and repeat keydown stays ignored by default unless `shortcut_repeat=true` is explicitly requested.

### Editor shell gates

- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `cargo run -p fretboard-dev -- diag suite diag-hardening-smoke-workspace --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`

This package currently proves:

- `workspace_shell_demo` remains the primary coherent shell proof,
- `editor_notes_demo` remains the minimal shell-mounted rail proof,
- the launched shell smoke floor now reaches beyond tabstrip-only checks,
- and the shell proof set does not silently collapse back into the generic `imui` backlog.

The promoted launched suite now freezes this minimum shell coverage:

- tab close / reorder / split preview,
- dirty-close prompt and discard close,
- content-focus restore via Escape,
- and left-rail / file-tree keep-alive.

### Diagnostics / tooling gates

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p2_first_open_diagnostics_path`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p2_diagnostics_owner_split`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p2_bounded_devtools_smoke_package`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p2_discoverability_entry`
- `python3 tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke`
- `cargo build -p fret-devtools`
- `cargo run -p fretboard-dev -- diag doctor campaigns`

This package currently proves:

- the P2 first-open path starts from CLI-compatible evidence production,
- the P2 diagnostics owner split stays explicit across runtime, tooling, GUI, and MCP surfaces,
- one repo-owned P2 smoke gate now proves the direct first-open loop with a real launched app,
- direct `diag run` leaves named bundle checkpoints and latest-bundle resolution through
  `script.result.json:last_bundle_dir`,
- direct `diag compare` remains a shared artifacts-layer verdict rather than a GUI-only diff mode,
- one bounded campaign root now proves explicit root `diag summarize`,
  aggregate `regression.summary.json` / `regression.index.json`, and `diag dashboard` over the
  same shared contracts,
- one canonical first-open doc now routes diagnostics readers before they open branch/reference
  notes,
- DevTools GUI and MCP stay aligned as consumers of the same artifacts root,
- and compare remains a shared artifacts-layer contract instead of a GUI-only diff mode.

### Multi-window hand-feel gates

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p3_multiwindow_runner_gap_checklist`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p3_bounded_multiwindow_parity_package`
- `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`
- `cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`

This package currently proves:

- one bounded P3 campaign now names hovered-window, peek-behind, transparent payload, and
  mixed-DPI follow-drag as one lane-owned package,
- `docking_arbitration_demo` is the launched proof surface for that package,
- the four expectations map to four repo-owned scripts instead of one vague docking smoke story,
- and `diag-hardening-smoke-docking` remains the small generic docking smoke entry rather than the
  IMUI lane's new umbrella package.

### Lane hygiene gates

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p3_multiwindow_runner_gap_checklist`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json > /dev/null`

## Missing gates that should become real before claiming closure

### P0 launched authoring proof (optional follow-on, not a blocker for the current decision)

The current source-policy/doc gates already prove:

- first-party docs/examples teach the frozen golden pair,
- reference proofs stay explicitly classified as non-default,
- and helper widening requires the frozen two-surface proof budget.

If P0 needs more validation later, the next useful gate should be a launched smoke or diag path for
the first-open immediate authoring loop, not another docs-only classification check.

### P3 multi-window parity gate

The checklist and bounded package are now both explicit:

- `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` freezes the runner-owned parity budget,
- `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md` freezes the lane-owned bounded package,
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json` is the canonical P3 campaign manifest.

Future work should replace or refine items inside that bounded package rather than inventing
another parallel P3 gate entry.
