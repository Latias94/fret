# ImUi Editor Notes Draft Actions v1 - Closeout Audit - 2026-04-24

Status: closed narrow P1 lane

## Verdict

Treat `imui-editor-notes-draft-actions-v1` as a closed app-owned draft action proof.

The first `Draft actions` row is coherent enough to close the lane because it adds explicit local
action/status affordances to the strongest non-multi-window editor proof surface without claiming
access to the preserved `TextField` draft buffer or proving a reusable runtime/component contract.
Do not reopen this folder for persistence, workspace dirty-close prompts, command-bus integration,
or public IMUI/TextField API work.

## Landed Surface

- `editor_notes_demo.rs` renders one inspector-local `Draft actions` row.
- `Mark draft ready` writes `Draft marked ready` into the app-owned outcome model and summary status.
- `Clear draft marker` writes `Draft marker cleared` into the app-owned outcome model and summary
  status.
- Both actions use stable test ids for source-policy and rail-surface proof tests.

## Future Work Policy

If a later slice needs to submit, discard, or inspect the actual preserved `TextField` draft buffer,
start a new narrow API-proof lane that names that exact contract and why app-owned status markers are
insufficient. Do not widen `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, `fret-ui-editor`, or
`crates/fret-ui` from this closed lane.

## Gates

- `cargo fmt -p fret-examples --check`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `python3 -m json.tool docs/workstreams/imui-editor-notes-draft-actions-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
