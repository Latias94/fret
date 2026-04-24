# ImUi Editor Notes Draft Actions v1 - Evidence & Gates

Goal: add app-owned editor-notes draft action affordances without persistence, dirty-close,
command-bus, or public IMUI/TextField API changes.

## Evidence Anchors

- `docs/workstreams/imui-editor-notes-draft-actions-v1/DESIGN.md`
- `docs/workstreams/imui-editor-notes-draft-actions-v1/TODO.md`
- `docs/workstreams/imui-editor-notes-draft-actions-v1/MILESTONES.md`
- `docs/workstreams/imui-editor-notes-draft-actions-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-editor-notes-draft-actions-v1/M1_APP_OWNED_DRAFT_ACTIONS_SLICE_2026-04-24.md`
- `docs/workstreams/imui-editor-notes-draft-actions-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-editor-notes-draft-actions-v1/WORKSTREAM.json`
- `docs/workstreams/imui-next-gap-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Focused Gates

- `cargo fmt -p fret-examples --check`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_editor_notes_draft_actions_follow_on --no-fail-fast`
- `python3 -m json.tool docs/workstreams/imui-editor-notes-draft-actions-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `editor-notes-draft-actions-closeout-source-policy`
- `git diff --check`
