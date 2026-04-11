# Container-Aware Editor Rail Surface v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-11

## Smallest current repro

Use this gate set to verify the current owner split before extracting any new reusable editor-rail
surface:

```bash
cargo nextest run -p fret-ui-gallery --test sidebar_docs_surface --no-fail-fast
cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --no-fail-fast
cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --no-fail-fast
cargo run -p fretboard -- diag run tools/diag-scripts/container-queries-docking-panel-resize.json --dir target/fret-diag/adaptive-panel-resize-promote --session-auto --pack --include-screenshots --launch target/release/container_queries_docking_demo
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/container-aware-editor-rail-surface-v1/WORKSTREAM.json > /dev/null
```

What this proves now:

- `Sidebar` docs still state that app-shell `is_mobile(...)` / `is_mobile_breakpoint(...)` must not
  become the editor-rail story.
- `workspace_shell_demo` still mounts an editor rail through the existing workspace shell seam.
- `editor_notes_demo` now provides a second app-local shell-mounted rail proof through the same
  workspace shell seam,
- the repo now has repeated proof for "compose through existing slots first, extract later",
- and fixed-window panel resizing still validates container-first adaptive behavior independently of
  viewport/mobile shell choices.

## Current evidence set

- `docs/workstreams/container-aware-editor-rail-surface-v1/M0_BASELINE_AUDIT_2026-04-11.md`
  - assumptions-first baseline for this follow-on.
- `docs/workstreams/container-aware-editor-rail-surface-v1/TARGET_INTERFACE_STATE.md`
  - target owner split and promotion threshold for any future reusable rail surface.
- `docs/workstreams/container-aware-editor-rail-surface-v1/M2_PANEL_RESIZE_GATE_ADOPTION_2026-04-11.md`
  - records adoption of the inherited fixed-window panel-resize proof into this lane's active gate
  set.
- `docs/workstreams/container-aware-editor-rail-surface-v1/M2_SECOND_CONSUMER_PROOF_2026-04-11.md`
  - records the landed second shell-mounted rail consumer in `editor_notes_demo`.
- `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - closes the lane on the current owner split and follow-on policy.
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
  - closed-lane follow-on policy that explicitly defers public `PanelRail` extraction.
- `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`
  - editor-panel owner split: `Sidebar` vs `fret-ui-editor` vs `fret-docking`.
- `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`
  - shell-seam decision: existing `WorkspaceFrame.left/right` is sufficient.
- `docs/workstreams/adaptive-layout-contract-closure-v1/M3_EDITOR_RAIL_COMPOSITION_2026-04-10.md`
  - first reviewable editor-rail composition proof via the existing shell seam.
- `docs/audits/shadcn-sidebar.md`
  - sidebar recommendation that it remains app-shell/device-shell only.
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
  - contract background for app-shell vs editor-panel adaptive naming.
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
  - current docs-path boundary for sidebar as app-shell-only surface.
- `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
  - focused docs-surface source gate for that boundary.
- `apps/fret-examples/src/workspace_shell_demo.rs`
  - current first proof surface for mounting an editor rail through `WorkspaceFrame.right(...)`.
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
  - source gate keeping that owner seam explicit.
- `apps/fret-examples/src/editor_notes_demo.rs`
  - second app-local proof surface for shell-mounted outline + inspector rails through
    `WorkspaceFrame.left/right`.
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
  - source gate keeping the second shell-mounted rail composition explicit.
- `tools/diag-scripts/container-queries-docking-panel-resize.json`
  - stable redirect path for the fixed-window panel-resize diagnostic gate.
- `tools/diag-scripts/docking/container-queries/container-queries-docking-panel-resize.json`
  - current v2 payload for the adopted panel-resize gate.
- `docs/workstreams/adaptive-layout-contract-closure-v1/M2_PANEL_RESIZE_GATE_PROMOTION_2026-04-10.md`
  - inherited evidence record for the promoted panel-resize proof and prior diag artifact.

## Closeout execution note

This lane is being started while `apps/fret-ui-gallery/src/ui/pages/sidebar.rs` and
`apps/fret-examples/src/workspace_shell_demo.rs` may also receive unrelated in-progress changes.

Rule:

- avoid changing those proof surfaces in this lane unless the next slice explicitly owns them,
- and open a narrower follow-on rather than reopening this closed owner-split lane.
