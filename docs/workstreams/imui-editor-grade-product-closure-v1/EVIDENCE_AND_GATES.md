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
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-devtools/src/main.rs`
- `apps/fret-devtools-mcp/src/main.rs`

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

These are not the only relevant surfaces, but they give the fastest current read on the four-way
gap: authoring, shell, tooling, and hand-feel.

## Current focused gates

### Immediate authoring / adapter gates

- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke`
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_example_keeps_current_facade_teaching_surface`
- `cargo nextest run -p fret-examples --lib first_party_imui_examples_keep_current_facade_teaching_surface immediate_mode_examples_docs_name_the_golden_pair_and_reference_roster immediate_mode_examples_docs_name_the_mounting_rule_for_imui_vs_imui_vstack immediate_mode_examples_docs_name_the_stable_identity_rule immediate_mode_workstream_freezes_the_two_surface_proof_budget_before_helper_widening imui_hello_demo_is_explicitly_demoted_to_smoke_reference compatibility_only_node_graph_imui_demo_is_the_only_first_party_retained_compatibility_example`

This package now locks the current immediate-mode product message at the source-policy layer:

- the golden pair is named explicitly,
- the nested-vs-root mounting rule stays explicit,
- the static-vs-dynamic stable-identity rule stays explicit,
- the reference/advanced/compatibility roster stays classified,
- and the proof budget rule stays frozen before any future helper widening.

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

- `cargo build -p fret-devtools`
- `cargo run -p fretboard-dev -- diag doctor campaigns`

### Lane hygiene gates

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

### P2 first-open devtools workflow gate

We still need one smoke path that validates the whole loop:

- inspect/pick,
- selector reuse,
- script execution,
- bundle output,
- and compare/summarize entry.

### P3 multi-window parity gate

We still need one bounded gate package that names:

- hovered-window selection,
- peek-behind while moving a tear-off window,
- transparent payload overlap behavior,
- and mixed-DPI cursor/follow correctness.

Do not claim P3 closure with prose alone. The parity lane already shows that this needs a bounded
proof surface.
