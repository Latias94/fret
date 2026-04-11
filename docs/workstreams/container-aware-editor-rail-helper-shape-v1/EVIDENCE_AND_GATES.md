# Container-Aware Editor Rail Helper Shape v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-11

## Focused gate set

```bash
cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/container-aware-editor-rail-helper-shape-v1/WORKSTREAM.json > /dev/null
```

What this proves:

- the repo still has two explicit shell-mounted editor-rail proof surfaces,
- the repeated shell seam remains `WorkspaceFrame.left/right`,
- and this lane's no-new-helper verdict is grounded in audited source shape rather than in the
  absence of proof.

## Evidence set

- `docs/workstreams/container-aware-editor-rail-helper-shape-v1/DESIGN.md`
- `docs/workstreams/container-aware-editor-rail-helper-shape-v1/M0_BASELINE_AUDIT_2026-04-11.md`
- `docs/workstreams/container-aware-editor-rail-helper-shape-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `ecosystem/fret-workspace/src/frame.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
