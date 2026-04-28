# ImUi TextField Draft Controller API Proof v1 - Evidence and Gates

Goal: prove a small external commit/discard operation handle for preserved `TextField` drafts
without exposing internal draft models or widening runtime/generic IMUI APIs.

## Evidence Anchors

- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/DESIGN.md`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/TODO.md`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/MILESTONES.md`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- `ecosystem/fret-ui-editor/tests/text_field_api_smoke.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Gates

- Gate name: `textfield-draft-controller-source-policy`
- `cargo nextest run -p fret-ui-editor --test text_field_api_smoke --no-fail-fast`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_starts_the_p1_textfield_draft_controller_api_proof --no-fail-fast`
- `python3 -m json.tool docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

Latest local evidence (2026-04-28):

- `cargo nextest run -p fret-ui-editor -E 'test(draft_controller) or test(text_field_option_defaults_match_buffered_plain_text_baseline)' --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast` passed after the initial cold compile timeout was rerun with warm cache.
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_starts_the_p1_textfield_draft_controller_api_proof --no-fail-fast` passed.
