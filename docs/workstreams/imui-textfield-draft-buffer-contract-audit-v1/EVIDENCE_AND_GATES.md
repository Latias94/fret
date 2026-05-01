# ImUi TextField Draft Buffer Contract Audit v1 - Evidence & Gates

Goal: decide whether a public preserved draft-buffer contract should be exposed from `TextField`.

## Evidence Anchors

- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/DESIGN.md`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/TODO.md`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/MILESTONES.md`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/M1_DRAFT_BUFFER_CONTRACT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-notes-draft-actions-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- `tools/gate_imui_workstream_source.py`

## Focused Gates

- `cargo fmt -p fret-examples --check`
- `python tools/gate_imui_workstream_source.py`
- `python3 -m json.tool docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `textfield-draft-buffer-contract-audit-source-policy`
- `git diff --check`
