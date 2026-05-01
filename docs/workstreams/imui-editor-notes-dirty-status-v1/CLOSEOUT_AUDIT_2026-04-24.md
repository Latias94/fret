# ImUi Editor Notes Dirty Status v1 - Closeout Audit - 2026-04-24

Status: closed narrow P1 lane

## Verdict

Treat `imui-editor-notes-dirty-status-v1` as a closed app-owned draft-status proof.

The first `Draft status` row is coherent enough to close the lane because it improves editor-grade
local feedback on the already shell-mounted `editor_notes_demo.rs` proof surface without proving a
shared runtime or component API. Do not reopen this folder for workspace dirty-close, persistence,
save commands, or generic editor APIs.

## Landed Surface

- `editor_notes_demo.rs` renders one inspector-local `Draft status` row.
- The row uses `TEST_ID_NOTES_DRAFT_STATUS` for source-policy and surface-test stability.
- `editor_notes_draft_status_label` keeps the copy local to the app-owned proof surface.
- The existing editor rail source test asserts the row, helper, and stable test id.

## Future Work Policy

If a later slice needs document dirty-close, save/persistence, or a reusable editor document-state
contract, start a new narrow follow-on that names the exact app-owned editor behavior still missing
and brings a runnable gate. Do not widen `fret-ui-kit::imui`, `fret-imui`, or `crates/fret-ui` from
this closed lane.

## Gates

- `cargo fmt -p fret-examples --check`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `python3 -m json.tool docs/workstreams/imui-editor-notes-dirty-status-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
