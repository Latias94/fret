# Closeout Audit — 2026-04-11

Status: closed closeout record

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `M1_CONTRACT_FREEZE_2026-04-11.md`
- `M2_CONSUMER_AUDIT_2026-04-11.md`
- `M2_PANEL_RESIZE_GATE_ADOPTION_2026-04-11.md`
- `M2_SECOND_CONSUMER_PROOF_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`

## Verdict

This lane is now closed.

It answered the narrow follow-on question left by the adaptive-layout closeout:

- shell placement for editor rails stays on `fret-workspace::WorkspaceFrame.left/right`,
- reusable inner editor content stays on `fret-ui-editor`,
- viewport/device-shell logic remains an outer-shell concern rather than part of a generic rail
  surface,
- and public extraction still requires more than one shell-mounted consumer.

This lane then advanced the proof set far enough to close:

- `workspace_shell_demo` remained the first reviewable shell-mounted rail proof,
- `editor_notes_demo` became the second shell-mounted rail consumer through the same seam,
- and the inherited fixed-window panel-resize diagnostic stayed active as the container-first
  adaptive proof.

## What this lane closes on

### 1) The owner split is now explicit

The repo now has a closed v1 answer for editor-rail ownership:

- workspace shell slots belong to `WorkspaceFrame`,
- reusable editor panel content belongs to `fret-ui-editor`,
- concrete rail recipe/state remains app-local until a narrower extraction lane explicitly owns it,
- and `Sidebar` remains app-shell/device-shell surface area only.

### 2) The repo now has two shell-mounted rail consumers

The current proof surfaces are:

1. `apps/fret-examples/src/workspace_shell_demo.rs`
2. `apps/fret-examples/src/editor_notes_demo.rs`

That is enough to prove repeated composition through the existing shell seam.

### 3) Public extraction is still deferred on purpose

Even with two consumers, this lane still does **not** promote a public `PanelRail` /
`InspectorSidebar` surface yet.

Why:

- the repeated proof is still about shell-slot composition, not a shared container-aware helper,
- outer-shell mobile/device-shell downgrade ownership must stay outside any reusable inner rail
  surface,
- and no evidence in this lane required moving recipe policy into `fret-docking` or widening
  `Sidebar`.

## Gates that define the closed surface

- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --no-fail-fast`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/container-aware-editor-rail-surface-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`

## Follow-on policy

Do not reopen this lane for:

- another source-only duplicate of the same `WorkspaceFrame.left/right` proof,
- widening `Sidebar` into the editor-rail story,
- or a generic public rail extraction without new container-aware evidence.

If future work is needed, open a narrower follow-on such as:

1. a shared container-aware editor-rail helper below any public promotion,
2. a public extraction lane only after shared adaptive behavior is reviewable,
3. or a dedicated outer-shell mobile downgrade composition lane if real app evidence appears.
