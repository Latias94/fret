# Outer-Shell Editor Rail Mobile Downgrade v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-11

## Focused gate set

```bash
cargo nextest run -p fret-ui-gallery --test device_shell_strategy_surface --no-fail-fast
cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/WORKSTREAM.json > /dev/null
```

What this proves:

- the repo still keeps device-shell downgrade branches explicit where reviewability matters,
- the app-shell/mobile `Sidebar` story remains separate from editor rails,
- and the current editor-rail proofs remain desktop shell-mounted rather than pretending that the
  mobile downgrade shape is already shared.

## Evidence set

- `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/DESIGN.md`
- `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/M0_BASELINE_AUDIT_2026-04-11.md`
- `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/container-aware-editor-rail-helper-shape-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
