# Closeout Audit - 2026-04-13

Status: closed closeout record

Related:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `M1_DEFAULT_WORKBENCH_ASSEMBLY_DECISION_2026-04-13.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`

## Verdict

This lane is now closed.

It answered the narrow P1 follow-on question left by the immediate-mode product-closure umbrella:

- the repo does not yet need a promoted first-party `WorkspaceWorkbenchShell`,
  `EditorWorkbenchShell`, or similar helper,
- the default workbench answer should remain explicit first-party assembly over the frozen
  `fret-workspace` starter set,
- and future maturity work should move back to the runner/backend multi-window lane rather than
  keep widening shell composition documentation here.

## What this lane settled

### 1) The broadest P1 product proof remains `workspace_shell_demo`

The lane re-read the current shell proof roster and kept the umbrella ordering intact:

- `workspace_shell_demo` remains the primary coherent workbench-shell proof,
- `editor_notes_demo` remains the minimal shell-mounted rail proof,
- `imui_editor_proof_demo` remains supporting docking/editor immediate evidence,
- `diag-hardening-smoke-workspace` remains the promoted launched shell floor.

This lane did not reopen those proofs.
It only used them to decide whether a new shell helper was actually justified.

### 2) The shell starter set is sufficient, but not yet promotion-worthy as a higher-level helper

The current reusable substrate already exists:

- `WorkspaceFrame`
- `WorkspaceTopBar`
- `WorkspaceStatusBar`
- `workspace_pane_tree_element_with_resize`
- `WorkspaceTabStrip`
- `WorkspaceCommandScope`
- `WorkspacePaneContentFocusTarget`

What is still missing is not another shell primitive.
What is still missing would have to be a repeated *high-level assembly shape* above those
primitives, and this lane found that the current first-party consumers are still intentionally
different enough that promotion would be premature.

### 3) The shipped decision is a no-new-helper-yet verdict

This lane closes on one explicit decision:

- keep the default workbench answer as explicit example-local assembly over the frozen starter set,
- keep the current owner for that assembly in app/example composition,
- and reopen helper extraction only if future repeated full-shell evidence exceeds the current
  proof set.

That is enough closure for the P1 shell question in this cycle.

## Validation used for closeout

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json > /dev/null`

## Follow-on policy

Do not reopen this lane for:

- another shell-helper brainstorming pass without stronger repeated full-shell evidence,
- tabstrip parity refinements that belong in the existing `workspace-tabstrip*` lanes,
- or runner/backend multi-window parity fixes that belong in docking parity.

From this point forward:

1. keep this folder as closeout evidence for the no-new-helper-yet verdict,
2. keep the umbrella lane for cross-phase status,
3. and continue the next active execution work in
   `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md` for
   the P3 multi-window hand-feel problem.
