# ImUi Editor Notes Dirty Status v1 - Evidence & Gates

Goal: add app-owned notes dirty/clean feedback without widening shell dirty-close, persistence, or
generic editor APIs.

## Evidence Anchors

- `docs/workstreams/imui-editor-notes-dirty-status-v1/DESIGN.md`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/TODO.md`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/MILESTONES.md`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/M1_APP_OWNED_DRAFT_STATUS_SLICE_2026-04-24.md`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-notes-inspector-command-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Focused Gates

- `cargo fmt -p fret-examples --check`

- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_editor_notes_dirty_status_follow_on --no-fail-fast`
- `python3 tools/check_workstream_catalog.py`
- `editor-notes-dirty-status-closeout-source-policy`
- `python3 -m json.tool docs/workstreams/imui-editor-notes-dirty-status-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`
