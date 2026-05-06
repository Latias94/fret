# P0 Product Workflow Coherence Review - 2026-05-06

Status: maintenance review; no new implementation-heavy lane opened from this note
Last updated: 2026-05-06

## Decision

Keep `imui-editor-grade-product-closure-v1` as the phase-ordering umbrella. The next
Dear ImGui-class work should still be product workflow coherence first, but implementation-heavy
work must stay in the narrow owner lanes that already exist.

This review does not justify broad IMUI API widening. The current product chain is:

1. `imui_action_basics` for generic/default immediate authoring.
2. `imui_editor_controls_basics` for first-contact editor controls.
3. `imui_editor_proof_demo` for the heavier editor-panel proof.
4. `workspace_shell_demo` plus `editor_notes_demo` for shell-mounted editor workflow proof.
5. `docs/diagnostics-first-open.md`, `fret-devtools`, and `fret-devtools-mcp` for the diagnostics
   loop.
6. `docking_arbitration_demo` plus `docking-multiwindow-imgui-parity` for runner/backend hand-feel.

## Review Truths

- First-open commands must not require users to infer package ownership from Cargo target errors.
- Cookbook IMUI examples are examples, not `fret-demo` binaries.
- `imui_hello_demo` remains a smoke/reference surface; when run through public `fretboard`, it
  needs explicit package selection because both `fret-demo` and `fret-examples-imui` define a
  binary with that name.
- `imui_editor_proof_demo` is the heavier product proof, but `workspace_shell_demo` remains the
  broader workbench-shell proof.
- Diagnostics and docking maturity are product workflow gaps, not reasons to grow `crates/fret-ui`
  or generic `fret-ui-kit::imui` helpers.

## Current Coherence Read

| Link in chain | Current state | Verdict |
| --- | --- | --- |
| Generic IMUI teaching | `imui_action_basics` uses the root `fret::imui` lane and has a launched action proof | Keep |
| Editor control teaching | `imui_editor_controls_basics` teaches `fret::imui::editor` | Keep |
| Product editor proof | `imui_editor_proof_demo` carries stable identity, app-owned collection depth, commands, editor controls, menus/popups, and `test_id` anchors | Keep as product proof |
| Workbench shell proof | `workspace_shell_demo` and `editor_notes_demo` own shell-mounted workflow proof | Keep out of generic IMUI backlog |
| Diagnostics loop | CLI-first diagnostics path and DevTools/MCP owner split are documented; DevTools discoverability remains a productization priority | Continue in diagnostics lanes |
| Multi-window hand-feel | Active execution lives in `docking-multiwindow-imgui-parity`; local Linux coverage is still bounded by available hosts | Continue in docking lane |
| First-open command clarity | Public `fretboard --bin imui_hello_demo` is ambiguous without `--package fret-demo` | Fix docs now; consider CLI guidance only if this repeats |

## Follow-On Routing

- First-open command/documentation drift: docs/examples and cookbook docs first; then `fretboard`
  diagnostics only if source docs are still insufficient.
- DevTools product discoverability: narrow DevTools follow-on, not this umbrella.
- Multi-window hand-feel: continue `docs/workstreams/docking-multiwindow-imgui-parity/`.
- Shared IMUI helper growth: only after the frozen two-surface proof budget is satisfied.

## Gate Results

2026-05-06 local review commands:

```powershell
rg -n "imui_hello_demo|imui_editor_proof_demo|workspace_shell_demo|docking_arbitration_demo|--package" docs/examples/README.md apps/fret-cookbook/EXAMPLES.md docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
git diff --check
```
